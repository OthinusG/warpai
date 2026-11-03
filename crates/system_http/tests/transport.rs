use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};
use system_http::{Client, RedirectPolicy};

fn client() -> Client {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap()
}
fn server(reply: &'static [u8]) -> (String, mpsc::Receiver<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(4)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        loop {
            let count = socket.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..count]);
            if let Some(end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                let text = String::from_utf8_lossy(&request[..end]).to_lowercase();
                let length = text
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length:"))
                    .map_or(0, |v| v.trim().parse::<usize>().unwrap());
                if request.len() >= end + 4 + length {
                    break;
                }
            }
        }
        let _ = tx.send(request);
        let _ = socket.write_all(reply);
    });
    (url, rx)
}

#[tokio::test]
async fn preserves_method_headers_body_and_error_status() {
    let client = client();
    let (url, requests) = server(b"HTTP/1.1 401 Unauthorized\r\nX-Quota: unknown\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}");
    let response = client
        .post(url)
        .header("X-Synthetic", "example")
        .body("payload")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 401);
    assert_eq!(response.headers()["x-quota"], "unknown");
    assert_eq!(response.text().await.unwrap(), "{}");
    let request =
        String::from_utf8(requests.recv_timeout(Duration::from_secs(4)).unwrap()).unwrap();
    assert!(request.starts_with("POST / HTTP/1.1\r\n"));
    assert!(request.to_lowercase().contains("x-synthetic: example\r\n"));
    assert!(request.ends_with("payload"));
}

#[tokio::test]
async fn usage_and_update_policy_rejects_redirects() {
    let _ = client();
    let client = Client::builder()
        .redirect(RedirectPolicy::None)
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    let (url, _) = server(b"HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:1/must-not-connect\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    assert_eq!(client.get(url).send().await.unwrap().status(), 302);
}

#[tokio::test]
async fn redirect_drops_credentials_on_origin_change() {
    let client = client();
    let (destination, requests) =
        server(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin = format!("http://{}/", listener.local_addr().unwrap());
    thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let mut buffer = [0; 2048];
        let _ = socket.read(&mut buffer);
        write!(socket, "HTTP/1.1 302 Found\r\nLocation: {destination}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
    });
    assert_eq!(
        client
            .get(origin)
            .header("X-Synthetic-Credential", "example")
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let request = requests.recv_timeout(Duration::from_secs(4)).unwrap();
    assert!(
        !String::from_utf8_lossy(&request)
            .to_lowercase()
            .contains("x-synthetic-credential")
    );
}

#[tokio::test]
async fn headers_and_chunks_arrive_before_eof_and_drop_cancels_task() {
    let client = client();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let (closed_tx, closed_rx) = mpsc::channel();
    thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(4)))
            .unwrap();
        let mut buffer = [0; 2048];
        let _ = socket.read(&mut buffer);
        socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n5\r\nhello\r\n").unwrap();
        let result = socket.read(&mut buffer);
        let _ = closed_tx.send(
            matches!(result, Ok(0))
                || result.is_err_and(|e| e.kind() == std::io::ErrorKind::ConnectionReset),
        );
    });
    let mut response = tokio::time::timeout(Duration::from_secs(2), client.get(url).send())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.chunk().await.unwrap().unwrap(), "hello");
    drop(response);
    assert!(closed_rx.recv_timeout(Duration::from_secs(5)).unwrap());
}

#[tokio::test]
async fn request_deadline_cancels_waiting_for_headers() {
    let client = client();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    thread::spawn(move || {
        let (_socket, _) = listener.accept().unwrap();
        thread::sleep(Duration::from_secs(2));
    });
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        client.get(url).timeout(Duration::from_millis(100)).send(),
    )
    .await
    .unwrap();
    assert!(result.is_err());
}

#[tokio::test]
async fn errors_do_not_include_request_credentials_or_query() {
    let client = client();
    let error = client
        .get("http://127.0.0.1:1/?sensitive=synthetic")
        .send()
        .await
        .unwrap_err();
    let text = error.to_string();
    assert!(!text.contains("sensitive") && !text.contains("synthetic"));
}
