//! Credential-free helper for disposable native system-proxy acceptance runners.
#[tokio::main]
async fn main() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let target = std::env::args()
        .nth(1)
        .expect("expected a public or fixture URL");
    let response = system_http::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap()
        .get(target)
        .send()
        .await
        .expect("native request failed");
    println!("{}", response.status().as_u16());
    assert_eq!(response.bytes().await.unwrap(), "native-system-proxy");
}
