use std::{
    ffi::{CStr, CString, c_char, c_void},
    io::Write,
    sync::{Arc, Mutex},
    time::Duration,
};

use bytes::Bytes;
use futures::{
    StreamExt,
    channel::{mpsc, oneshot},
};
use http_body_util::BodyExt;
use reqwest::ResponseBuilderExt;

use crate::{Error, RedirectPolicy};

unsafe extern "C" {
    fn warpai_http_start(
        url: *const c_char,
        method: *const c_char,
        headers: *const c_char,
        body: *const u8,
        length: usize,
        path: *const c_char,
        redirects: bool,
        timeout: f64,
        context: *mut c_void,
        response: extern "C" fn(*mut c_void, i32, *const c_char, *const c_char),
        data: extern "C" fn(*mut c_void, *const u8, usize),
        complete: extern "C" fn(*mut c_void, i64),
    ) -> *mut c_void;
    fn warpai_http_resume(task: *mut c_void);
    fn warpai_http_cancel(task: *mut c_void);
    fn warpai_http_release(task: *mut c_void);
}

struct Task(*mut c_void);
// NSURLSessionTask's lifecycle methods are thread-safe; this owns one retained reference.
unsafe impl Send for Task {}
unsafe impl Sync for Task {}
impl Task {
    fn resume(&self) {
        unsafe { warpai_http_resume(self.0) };
    }
}
impl Drop for Task {
    fn drop(&mut self) {
        unsafe {
            warpai_http_cancel(self.0);
            warpai_http_release(self.0);
        }
    }
}

type Head = (u16, http::HeaderMap, reqwest::Url);
struct Context {
    head: Mutex<Option<oneshot::Sender<Result<Head, Error>>>>,
    body: mpsc::UnboundedSender<Result<Bytes, Error>>,
    _upload: Option<tempfile::NamedTempFile>,
}

extern "C" fn response(raw: *mut c_void, status: i32, headers: *const c_char, url: *const c_char) {
    // Native callbacks are serialized and complete is the only owner that frees this box.
    let ctx = unsafe { &*raw.cast::<Context>() };
    let parsed = (|| {
        if headers.is_null() || url.is_null() {
            return Err(Error::InvalidResponse);
        }
        let fields = unsafe { CStr::from_ptr(headers) }
            .to_str()
            .map_err(|_| Error::InvalidResponse)?;
        let mut result = http::HeaderMap::new();
        for field in fields.split("\r\n").filter(|line| !line.is_empty()) {
            let (name, value) = field.split_once(':').ok_or(Error::InvalidResponse)?;
            let name = http::HeaderName::from_bytes(name.as_bytes())
                .map_err(|_| Error::InvalidResponse)?;
            let bytes: Vec<u8> = value
                .trim_start()
                .chars()
                .map(|ch| u8::try_from(ch as u32).map_err(|_| Error::InvalidResponse))
                .collect::<Result<_, _>>()?;
            result.append(
                name,
                http::HeaderValue::from_bytes(&bytes).map_err(|_| Error::InvalidResponse)?,
            );
        }
        let url = reqwest::Url::parse(
            unsafe { CStr::from_ptr(url) }
                .to_str()
                .map_err(|_| Error::InvalidResponse)?,
        )
        .map_err(|_| Error::InvalidResponse)?;
        Ok((status as u16, result, url))
    })();
    if let Some(sender) = ctx.head.lock().unwrap_or_else(|p| p.into_inner()).take() {
        let _ = sender.send(parsed);
    }
}
extern "C" fn data(raw: *mut c_void, bytes: *const u8, length: usize) {
    let ctx = unsafe { &*raw.cast::<Context>() };
    if length > 1024 * 1024 {
        let _ = ctx.body.unbounded_send(Err(Error::InvalidResponse));
        return;
    }
    let bytes = if length == 0 {
        Bytes::new()
    } else {
        Bytes::copy_from_slice(unsafe { std::slice::from_raw_parts(bytes, length) })
    };
    let _ = ctx.body.unbounded_send(Ok(bytes));
}
extern "C" fn complete(raw: *mut c_void, code: i64) {
    let ctx = unsafe { Box::from_raw(raw.cast::<Context>()) };
    if let Some(sender) = ctx.head.lock().unwrap_or_else(|p| p.into_inner()).take() {
        let _ = sender.send(Err(Error::Native(if code == 0 { -1 } else { code })));
    } else if code != 0 {
        let _ = ctx.body.unbounded_send(Err(Error::Native(code)));
    }
}

pub(super) async fn execute(
    mut request: reqwest::Request,
    default_timeout: Option<Duration>,
    redirects: RedirectPolicy,
) -> Result<reqwest::Response, Error> {
    let url = CString::new(request.url().as_str()).map_err(|_| Error::InvalidResponse)?;
    let method = CString::new(request.method().as_str()).map_err(|_| Error::InvalidResponse)?;
    let mut fields = String::new();
    for (name, value) in request.headers() {
        fields.push_str(name.as_str());
        fields.push_str(": ");
        fields.extend(value.as_bytes().iter().map(|&byte| char::from(byte)));
        fields.push_str("\r\n");
    }
    let headers = CString::new(fields).map_err(|_| Error::InvalidResponse)?;
    let started = std::time::Instant::now();
    let timeout = request.timeout().or(default_timeout.as_ref()).copied();
    let body = request.body_mut().take();
    let stage = async move {
        let mut upload = None;
        let mut buffered = Vec::new();
        if let Some(mut body) = body {
            if let Some(bytes) = body.as_bytes() {
                buffered.extend_from_slice(bytes);
            } else {
                // Preserve streaming uploads without buffering whole files in RAM.
                let mut file = tempfile::NamedTempFile::new().map_err(|_| Error::Upload)?;
                while let Some(frame) = body.frame().await {
                    let frame = frame?;
                    if let Ok(bytes) = frame.into_data() {
                        file.write_all(&bytes).map_err(|_| Error::Upload)?;
                    }
                }
                file.flush().map_err(|_| Error::Upload)?;
                upload = Some(file);
            }
        }
        Ok::<_, Error>((upload, buffered))
    };
    let (upload, buffered) = if let Some(timeout) = timeout {
        match futures::future::select(
            Box::pin(stage),
            Box::pin(futures_timer::Delay::new(timeout)),
        )
        .await
        {
            futures::future::Either::Left((result, _)) => result?,
            futures::future::Either::Right(_) => return Err(Error::Native(-1001)),
        }
    } else {
        stage.await?
    };
    let timeout = timeout
        .map(|timeout| {
            timeout
                .checked_sub(started.elapsed())
                .filter(|remaining| !remaining.is_zero())
                .ok_or(Error::Native(-1001))
        })
        .transpose()?
        .map_or(0.0, |duration| duration.as_secs_f64());
    let path = upload
        .as_ref()
        .map(|file| {
            CString::new(file.path().to_str().ok_or(Error::Upload)?).map_err(|_| Error::Upload)
        })
        .transpose()?;
    let (head_tx, head_rx) = oneshot::channel();
    let (body_tx, mut body_rx) = mpsc::unbounded();
    let context = Box::new(Context {
        head: Mutex::new(Some(head_tx)),
        body: body_tx,
        _upload: upload,
    });
    let raw = Box::into_raw(context).cast();
    let handle = unsafe {
        warpai_http_start(
            url.as_ptr(),
            method.as_ptr(),
            headers.as_ptr(),
            buffered.as_ptr(),
            buffered.len(),
            path.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
            matches!(redirects, RedirectPolicy::Follow),
            timeout,
            raw,
            response,
            data,
            complete,
        )
    };
    if handle.is_null() {
        unsafe {
            drop(Box::from_raw(raw.cast::<Context>()));
        }
        return Err(Error::InvalidResponse);
    }
    let task = Arc::new(Task(handle));
    task.resume();
    let (status, mut headers, url) = head_rx.await.map_err(|_| Error::Native(-1))??;
    // URLSession decodes content encodings. Do not advertise the wire encoding/length
    // on its decoded byte stream (including SSE and compressed release metadata).
    let decoded = headers
        .get(http::header::CONTENT_ENCODING)
        .is_some_and(|encoding| {
            cfg!(any(target_os = "macos", target_os = "linux"))
                || matches!(encoding.as_bytes(), b"gzip" | b"deflate")
        });
    if decoded {
        headers.remove(http::header::CONTENT_ENCODING);
        headers.remove(http::header::CONTENT_LENGTH);
    }
    let stream = async_stream::stream! {
        while let Some(chunk) = body_rx.next().await {
            yield chunk;
            task.resume();
        }
    };
    let mut response = http::Response::builder().status(status).url(url);
    *response.headers_mut().ok_or(Error::InvalidResponse)? = headers;
    Ok(response
        .body(reqwest::Body::wrap_stream(stream))
        .map_err(|_| Error::InvalidResponse)?
        .into())
}
