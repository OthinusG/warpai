fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("src/session.m")
            .flag("-fobjc-arc")
            .flag("-fblocks")
            .compile("warpai_system_http");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rerun-if-changed=src/session.m");
    } else if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        cc::Build::new()
            .cpp(true)
            .std("c++17")
            .file("src/session.cc")
            .compile("warpai_system_http");
        println!("cargo:rustc-link-lib=winhttp");
        println!("cargo:rerun-if-changed=src/session.cc");
    } else if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        let output = std::process::Command::new("pkg-config")
            .args(["--cflags", "--libs", "libsoup-2.4"])
            .output()
            .expect("pkg-config is required");
        assert!(
            output.status.success(),
            "Install libsoup-2.4 development files and glib-networking"
        );
        let mut build = cc::Build::new();
        build.cpp(true).std("c++17").file("src/session.cc");
        for flag in String::from_utf8(output.stdout)
            .expect("pkg-config UTF-8")
            .split_whitespace()
        {
            if let Some(path) = flag.strip_prefix("-I") {
                build.include(path);
            } else if let Some(name) = flag.strip_prefix("-l") {
                println!("cargo:rustc-link-lib={name}");
            } else if let Some(path) = flag.strip_prefix("-L") {
                println!("cargo:rustc-link-search=native={path}");
            } else {
                build.flag(flag);
            }
        }
        build.compile("warpai_system_http");
        println!("cargo:rerun-if-changed=src/session.cc");
    }
}
