//! Desktop HTTP transport. macOS delegates routing and TLS to NSURLSession.
use std::{fmt, time::Duration};

use futures::StreamExt;
#[cfg(not(target_family = "wasm"))]
type EventStream =
    futures::stream::BoxStream<'static, Result<reqwest_eventsource::Event, EventSourceError>>;
#[cfg(target_family = "wasm")]
type EventStream =
    futures::stream::LocalBoxStream<'static, Result<reqwest_eventsource::Event, EventSourceError>>;
use reqwest::{IntoUrl, Request, Response};
use serde::Serialize;

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
mod native;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    // Native errors deliberately exclude request URLs, headers and NSError userInfo.
    #[error("System HTTP request failed (code {0})")]
    Native(i64),
    #[error("Invalid system HTTP response")]
    InvalidResponse,
    #[error("Could not stage HTTP upload")]
    Upload,
}

#[derive(Clone, Copy, Default)]
pub enum RedirectPolicy {
    #[default]
    Follow,
    None,
}

#[derive(Clone)]
pub struct Client {
    wrapped: reqwest::Client,
    timeout: Option<Duration>,
    redirects: RedirectPolicy,
    headers: http::HeaderMap,
    https_only: bool,
}

pub struct ClientBuilder {
    wrapped: reqwest::ClientBuilder,
    timeout: Option<Duration>,
    redirects: RedirectPolicy,
    headers: http::HeaderMap,
    https_only: bool,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
impl Client {
    pub fn new() -> Self {
        Self::builder().build().expect("valid system HTTP client")
    }
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            wrapped: reqwest::Client::builder(),
            timeout: None,
            redirects: RedirectPolicy::Follow,
            headers: http::HeaderMap::new(),
            https_only: false,
        }
    }
    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(reqwest::Method::GET, url)
    }
    pub fn post<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(reqwest::Method::POST, url)
    }
    pub fn put<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(reqwest::Method::PUT, url)
    }
    pub fn patch<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(reqwest::Method::PATCH, url)
    }
    pub fn delete<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(reqwest::Method::DELETE, url)
    }
    pub fn request<U: IntoUrl>(&self, method: reqwest::Method, url: U) -> RequestBuilder {
        RequestBuilder {
            wrapped: self.wrapped.request(method, url),
            client: self.clone(),
        }
    }
    pub fn from_parts(&self, request: Request) -> RequestBuilder {
        RequestBuilder {
            wrapped: reqwest::RequestBuilder::from_parts(self.wrapped.clone(), request),
            client: self.clone(),
        }
    }
    pub async fn execute(&self, mut request: Request) -> Result<Response, Error> {
        for (name, value) in &self.headers {
            if !request.headers().contains_key(name) {
                request.headers_mut().insert(name, value.clone());
            }
        }
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        {
            let timeout = request.timeout().copied().or(self.timeout);
            let started = std::time::Instant::now();
            for attempt in 0..=10 {
                if let Some(timeout) = timeout {
                    *request.timeout_mut() = Some(
                        timeout
                            .checked_sub(started.elapsed())
                            .filter(|remaining| !remaining.is_zero())
                            .ok_or(Error::Native(-1001))?,
                    );
                }
                if self.https_only && request.url().scheme() != "https" {
                    return Err(Error::InvalidResponse);
                }
                if !matches!(request.url().scheme(), "http" | "https")
                    || !request.url().username().is_empty()
                    || request.url().password().is_some()
                {
                    return Err(Error::InvalidResponse);
                }
                let next = request.try_clone();
                let response = native::execute(request, self.timeout, RedirectPolicy::None).await?;
                if !matches!(self.redirects, RedirectPolicy::Follow)
                    || !matches!(response.status().as_u16(), 301 | 302 | 303 | 307 | 308)
                {
                    return Ok(response);
                }
                let Some(location) = response.headers().get(http::header::LOCATION) else {
                    return Ok(response);
                };
                let url = response
                    .url()
                    .join(location.to_str().map_err(|_| Error::InvalidResponse)?)
                    .map_err(|_| Error::InvalidResponse)?;
                if attempt == 10 || response.url().scheme() == "https" && url.scheme() != "https" {
                    return Err(Error::InvalidResponse);
                }
                let mut next = next.ok_or(Error::Upload)?;
                if response.status().as_u16() == 303 && next.method() != reqwest::Method::HEAD
                    || matches!(response.status().as_u16(), 301 | 302)
                        && next.method() == reqwest::Method::POST
                {
                    *next.method_mut() = reqwest::Method::GET;
                    *next.body_mut() = None;
                    next.headers_mut().remove(http::header::CONTENT_TYPE);
                    next.headers_mut().remove(http::header::CONTENT_LENGTH);
                    next.headers_mut().remove(http::header::TRANSFER_ENCODING);
                }
                if response.url().origin() != url.origin() {
                    let retained: http::HeaderMap = next
                        .headers()
                        .iter()
                        .filter(|(name, _)| {
                            *name == http::header::ACCEPT || *name == http::header::USER_AGENT
                        })
                        .map(|(name, value)| (name.clone(), value.clone()))
                        .collect();
                    *next.headers_mut() = retained;
                }
                *next.url_mut() = url;
                request = next;
            }
            Err(Error::InvalidResponse)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Ok(self.wrapped.execute(request).await?)
        }
    }
}

impl ClientBuilder {
    pub fn new() -> Self {
        Client::builder()
    }
    pub fn timeout(mut self, value: Duration) -> Self {
        self.timeout = Some(value);
        #[cfg(not(target_family = "wasm"))]
        {
            self.wrapped = self.wrapped.timeout(value);
        }
        self
    }
    pub fn redirect(mut self, value: RedirectPolicy) -> Self {
        self.redirects = value;
        #[cfg(not(target_family = "wasm"))]
        {
            self.wrapped = self.wrapped.redirect(match value {
                RedirectPolicy::None => reqwest::redirect::Policy::none(),
                RedirectPolicy::Follow => reqwest::redirect::Policy::limited(10),
            });
        }
        self
    }
    pub fn user_agent(mut self, value: &str) -> Self {
        if let Ok(value) = http::HeaderValue::from_str(value) {
            self.headers.insert(http::header::USER_AGENT, value);
        }
        #[cfg(not(target_family = "wasm"))]
        {
            self.wrapped = self.wrapped.user_agent(value);
        }
        self
    }
    pub fn default_headers(mut self, value: http::HeaderMap) -> Self {
        self.headers.extend(value.clone());
        self.wrapped = self.wrapped.default_headers(value);
        self
    }
    pub fn https_only(mut self, value: bool) -> Self {
        self.https_only = value;
        #[cfg(not(target_family = "wasm"))]
        {
            self.wrapped = self.wrapped.https_only(value);
        }
        self
    }
    #[cfg(not(target_family = "wasm"))]
    pub fn pool_idle_timeout(mut self, value: Duration) -> Self {
        self.wrapped = self.wrapped.pool_idle_timeout(value);
        self
    }
    #[cfg(not(target_family = "wasm"))]
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.wrapped = self.wrapped.connect_timeout(value);
        self
    }
    pub fn build(self) -> reqwest::Result<Client> {
        Ok(Client {
            wrapped: self.wrapped.build()?,
            timeout: self.timeout,
            redirects: self.redirects,
            headers: self.headers,
            https_only: self.https_only,
        })
    }
}

pub struct RequestBuilder {
    wrapped: reqwest::RequestBuilder,
    client: Client,
}
impl RequestBuilder {
    pub fn build(self) -> reqwest::Result<Request> {
        self.wrapped.build()
    }
    pub fn json<T: Serialize + ?Sized>(mut self, value: &T) -> Self {
        self.wrapped = self.wrapped.json(value);
        self
    }
    pub fn form<T: Serialize + ?Sized>(mut self, value: &T) -> Self {
        self.wrapped = self.wrapped.form(value);
        self
    }
    pub fn body<T: Into<reqwest::Body>>(mut self, value: T) -> Self {
        self.wrapped = self.wrapped.body(value);
        self
    }
    pub fn timeout(mut self, value: Duration) -> Self {
        #[cfg(not(target_family = "wasm"))]
        {
            self.wrapped = self.wrapped.timeout(value);
        }
        self
    }
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        http::HeaderName: TryFrom<K>,
        <http::HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        http::HeaderValue: TryFrom<V>,
        <http::HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.wrapped = self.wrapped.header(key, value);
        self
    }
    pub fn bearer_auth<T: fmt::Display>(mut self, value: T) -> Self {
        self.wrapped = self.wrapped.bearer_auth(value);
        self
    }
    pub fn basic_auth<U: fmt::Display, P: fmt::Display>(
        mut self,
        user: U,
        password: Option<P>,
    ) -> Self {
        self.wrapped = self.wrapped.basic_auth(user, password);
        self
    }
    pub async fn send(self) -> Result<Response, Error> {
        let request = self.wrapped.build()?;
        self.client.execute(request).await
    }
    pub fn eventsource(self) -> EventStream {
        use eventsource_stream::Eventsource;
        let stream = async_stream::stream! {
            let response = match self.header("Accept", "text/event-stream").send().await {
                Ok(response) => response,
                Err(error) => { yield Err(EventSourceError::Transport(error)); return; }
            };
            if response.status() != http::StatusCode::OK {
                yield Err(reqwest_eventsource::Error::InvalidStatusCode(response.status(), response).into()); return;
            }
            let content_type = response.headers().get(http::header::CONTENT_TYPE).cloned().unwrap_or_else(|| http::HeaderValue::from_static(""));
            if !content_type.as_bytes().starts_with(b"text/event-stream") {
                yield Err(reqwest_eventsource::Error::InvalidContentType(content_type, response).into()); return;
            }
            yield Ok(reqwest_eventsource::Event::Open);
            let mut events = response.bytes_stream().eventsource();
            while let Some(event) = events.next().await {
                match event {
                    Ok(event) => yield Ok(reqwest_eventsource::Event::Message(event)),
                    Err(error) => { yield Err(reqwest_eventsource::Error::from(error).into()); return; }
                }
            }
            yield Err(reqwest_eventsource::Error::StreamEnded.into());
        };
        #[cfg(target_family = "wasm")]
        {
            stream.boxed_local()
        }
        #[cfg(not(target_family = "wasm"))]
        {
            stream.boxed()
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventSourceError {
    #[error(transparent)]
    Transport(#[from] Error),
    #[error(transparent)]
    EventSource(#[from] reqwest_eventsource::Error),
}

impl<'a> oauth2::AsyncHttpClient<'a> for Client {
    type Error = oauth2::HttpClientError<Error>;
    #[cfg(target_family = "wasm")]
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<oauth2::HttpResponse, Self::Error>> + 'a>,
    >;
    #[cfg(not(target_family = "wasm"))]
    type Future = std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<oauth2::HttpResponse, Self::Error>>
                + Send
                + Sync
                + 'a,
        >,
    >;
    fn call(&'a self, request: oauth2::HttpRequest) -> Self::Future {
        Box::pin(async move {
            let request: Request = request.try_into().map_err(|e: reqwest::Error| {
                oauth2::HttpClientError::Reqwest(Box::new(e.into()))
            })?;
            let response = self
                .execute(request)
                .await
                .map_err(|e| oauth2::HttpClientError::Reqwest(Box::new(e)))?;
            let mut result = http::Response::builder().status(response.status());
            for (name, value) in response.headers() {
                result = result.header(name, value);
            }
            let bytes = response
                .bytes()
                .await
                .map_err(|e| oauth2::HttpClientError::Reqwest(Box::new(e.into())))?;
            result
                .body(bytes.to_vec())
                .map_err(oauth2::HttpClientError::Http)
        })
    }
}
