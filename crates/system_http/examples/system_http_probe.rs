//! Credential-free helper for disposable native system-proxy acceptance runners.
#[tokio::main]
async fn main() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let target = std::env::args()
        .nth(1)
        .expect("expected a public or fixture URL");
    let mode = std::env::args().nth(2).unwrap_or_default();
    if mode == "--subprocess" {
        let proxy = system_http::subprocess_proxy::SubprocessProxy::resolve(&target)
            .await
            .expect("system proxy lookup failed");
        let mut child = std::process::Command::new(std::env::current_exe().unwrap());
        child
            .args([&target, "--child"])
            .stdin(std::process::Stdio::null());
        // Deliberately conflicting values must never override the system-selected route.
        child
            .env("HTTPS_PROXY", "http://127.0.0.1:1")
            .env("NO_PROXY", "*");
        proxy.apply(&mut child);
        assert!(child.status().expect("child did not start").success());
        return;
    }
    if mode == "--child" {
        // Raw reqwest uses proxy environment variables like the vendor CLI, not our native transport.
        let mut client = reqwest::Client::builder().no_proxy();
        let variable = if target.starts_with("https://") {
            "HTTPS_PROXY"
        } else {
            "HTTP_PROXY"
        };
        if let Ok(proxy) = std::env::var(variable) {
            client = client.proxy(reqwest::Proxy::all(proxy).expect("invalid child proxy URI"));
        }
        let response = client
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap()
            .get(&target)
            .send()
            .await;
        if target.starts_with("https://") {
            // The synthetic proxy rejects CONNECT before TLS; reaching it is asserted by Python.
            assert!(response.is_err());
            println!("connect-rejected");
        } else {
            let response = response.expect("child proxy request failed");
            println!("{}", response.status().as_u16());
            assert_eq!(response.bytes().await.unwrap(), "native-system-proxy");
        }
        return;
    }
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
