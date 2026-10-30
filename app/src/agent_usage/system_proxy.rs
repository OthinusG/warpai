//! PAC-aware system proxy lookup for usage queries.
//!
//! reqwest's `system-proxy` support only reads static HTTP/HTTPS proxy settings and
//! ignores automatic proxy configuration, so machines configured with a PAC file
//! (the common Clash/Mihomo setup on macOS) end up connecting directly. CFNetwork
//! can evaluate a PAC script against a target URL, which keeps usage queries on the
//! same path as the rest of the system.

#[cfg(not(target_os = "macos"))]
pub(crate) fn custom_proxy() -> Option<reqwest::Proxy> {
    None
}

#[cfg(target_os = "macos")]
pub(crate) use macos::custom_proxy;

#[cfg(target_os = "macos")]
mod macos {
    use std::ffi::{c_char, c_void, CStr};
    use std::ptr;
    use std::time::{Duration, Instant};

    type CFTypeRef = *const c_void;
    type CFStringRef = *const c_void;
    type CFDictionaryRef = *const c_void;
    type CFArrayRef = *const c_void;
    type CFURLRef = *const c_void;
    type CFNumberRef = *const c_void;
    type CFBooleanRef = *const c_void;
    type CFRunLoopRef = *mut c_void;
    type CFRunLoopSourceRef = *mut c_void;
    type CFIndex = isize;

    const UTF8_ENCODING: u32 = 0x0800_0100;
    const SINT32_TYPE: i32 = 3;
    const PAC_TIMEOUT: Duration = Duration::from_secs(3);

    #[repr(C)]
    struct CFStreamClientContext {
        version: CFIndex,
        info: *mut c_void,
        retain: *const c_void,
        release: *const c_void,
        copy_description: *const c_void,
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFAllocatorDefault: CFTypeRef;
        static kCFRunLoopDefaultMode: CFStringRef;
        fn CFRelease(cf: CFTypeRef);
        fn CFStringCreateWithCString(
            allocator: CFTypeRef,
            c_str: *const c_char,
            encoding: u32,
        ) -> CFStringRef;
        fn CFURLCreateWithString(
            allocator: CFTypeRef,
            string: CFStringRef,
            base: CFURLRef,
        ) -> CFURLRef;
        fn CFStringGetCString(
            string: CFStringRef,
            buffer: *mut c_char,
            buffer_size: CFIndex,
            encoding: u32,
        ) -> u8;
        fn CFArrayGetCount(array: CFArrayRef) -> CFIndex;
        fn CFArrayGetValueAtIndex(array: CFArrayRef, index: CFIndex) -> *const c_void;
        fn CFDictionaryGetValue(dictionary: CFDictionaryRef, key: *const c_void) -> *const c_void;
        fn CFNumberGetValue(number: CFNumberRef, number_type: i32, value: *mut c_void) -> u8;
        fn CFGetTypeID(cf: CFTypeRef) -> usize;
        fn CFBooleanGetTypeID() -> usize;
        fn CFBooleanGetValue(boolean: CFBooleanRef) -> u8;
        fn CFRunLoopGetCurrent() -> CFRunLoopRef;
        fn CFRunLoopAddSource(
            run_loop: CFRunLoopRef,
            source: CFRunLoopSourceRef,
            mode: CFStringRef,
        );
        fn CFRunLoopRemoveSource(
            run_loop: CFRunLoopRef,
            source: CFRunLoopSourceRef,
            mode: CFStringRef,
        );
        fn CFRunLoopRunInMode(
            mode: CFStringRef,
            seconds: f64,
            return_after_source_handled: u8,
        ) -> i32;
    }

    #[link(name = "SystemConfiguration", kind = "framework")]
    extern "C" {
        fn SCDynamicStoreCopyProxies(store: *const c_void) -> CFDictionaryRef;
        static kSCPropNetProxiesProxyAutoConfigEnable: CFStringRef;
        static kSCPropNetProxiesProxyAutoConfigURLString: CFStringRef;
    }

    #[link(name = "CFNetwork", kind = "framework")]
    extern "C" {
        fn CFNetworkExecuteProxyAutoConfigurationURL(
            pac_url: CFURLRef,
            target_url: CFURLRef,
            callback: extern "C" fn(*mut c_void, CFArrayRef, CFTypeRef),
            context: *mut CFStreamClientContext,
        ) -> CFRunLoopSourceRef;
        static kCFProxyTypeKey: CFStringRef;
        static kCFProxyHostNameKey: CFStringRef;
        static kCFProxyPortNumberKey: CFStringRef;
    }

    #[derive(Default)]
    struct Resolution {
        finished: bool,
        /// (scheme, host, port) for the first usable proxy in the PAC answer.
        proxy: Option<(&'static str, String, u16)>,
    }

    /// Returns a reqwest proxy that resolves PAC rules per request. Only installed
    /// when the system actually has automatic proxy configuration enabled; static
    /// proxies stay with reqwest's built-in system proxy handling.
    pub(crate) fn custom_proxy() -> Option<reqwest::Proxy> {
        if unsafe { pac_url() }.is_none() {
            return None;
        }
        Some(reqwest::Proxy::custom(|url| resolve(url)))
    }

    fn resolve(target: &reqwest::Url) -> Option<reqwest::Url> {
        let pac = unsafe { pac_url() }?;
        let proxy = unsafe { evaluate_pac(&pac, target.as_str()) }?;
        reqwest::Url::parse(&format!("{}://{}:{}", proxy.0, proxy.1, proxy.2)).ok()
    }

    /// Reads the system PAC URL, if automatic proxy configuration is enabled.
    unsafe fn pac_url() -> Option<String> {
        let proxies = SCDynamicStoreCopyProxies(ptr::null());
        if proxies.is_null() {
            return None;
        }
        let enabled = CFDictionaryGetValue(proxies, kSCPropNetProxiesProxyAutoConfigEnable);
        let enabled = !enabled.is_null()
            && CFGetTypeID(enabled) == CFBooleanGetTypeID()
            && CFBooleanGetValue(enabled as CFBooleanRef) != 0;
        let url = if enabled {
            let value = CFDictionaryGetValue(proxies, kSCPropNetProxiesProxyAutoConfigURLString);
            if value.is_null() {
                None
            } else {
                cf_string(value as CFStringRef)
            }
        } else {
            None
        };
        CFRelease(proxies);
        url
    }

    unsafe fn evaluate_pac(pac_url: &str, target_url: &str) -> Option<(&'static str, String, u16)> {
        let pac_string = CFStringCreateWithCString(
            kCFAllocatorDefault,
            c_string(&pac_url)?.as_ptr(),
            UTF8_ENCODING,
        );
        if pac_string.is_null() {
            return None;
        }
        let pac_ref = CFURLCreateWithString(kCFAllocatorDefault, pac_string, ptr::null());
        CFRelease(pac_string);
        if pac_ref.is_null() {
            return None;
        }
        let target_string = CFStringCreateWithCString(
            kCFAllocatorDefault,
            c_string(&target_url)?.as_ptr(),
            UTF8_ENCODING,
        );
        if target_string.is_null() {
            CFRelease(pac_ref);
            return None;
        }
        let target_ref = CFURLCreateWithString(kCFAllocatorDefault, target_string, ptr::null());
        CFRelease(target_string);
        if target_ref.is_null() {
            CFRelease(pac_ref);
            return None;
        }

        let mut resolution = Resolution::default();
        let mut context = CFStreamClientContext {
            version: 0,
            info: &mut resolution as *mut Resolution as *mut c_void,
            retain: ptr::null(),
            release: ptr::null(),
            copy_description: ptr::null(),
        };
        let source = CFNetworkExecuteProxyAutoConfigurationURL(
            pac_ref,
            target_ref,
            pac_callback,
            &mut context,
        );
        CFRelease(pac_ref);
        CFRelease(target_ref);
        if source.is_null() {
            return None;
        }

        let run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(run_loop, source, kCFRunLoopDefaultMode);
        let deadline = Instant::now() + PAC_TIMEOUT;
        while !resolution.finished && Instant::now() < deadline {
            CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.05, 1);
        }
        CFRunLoopRemoveSource(run_loop, source, kCFRunLoopDefaultMode);
        CFRelease(source as CFTypeRef);
        if resolution.finished {
            resolution.proxy
        } else {
            None
        }
    }

    extern "C" fn pac_callback(client: *mut c_void, proxies: CFArrayRef, _error: CFTypeRef) {
        if client.is_null() || proxies.is_null() {
            return;
        }
        // SAFETY: `client` points at the `Resolution` owned by `evaluate_pac`,
        // which outlives the callback.
        let resolution = unsafe { &mut *(client as *mut Resolution) };
        if resolution.finished {
            return;
        }
        resolution.finished = true;
        unsafe {
            for index in 0..CFArrayGetCount(proxies) {
                let entry = CFArrayGetValueAtIndex(proxies, index) as CFDictionaryRef;
                if entry.is_null() {
                    continue;
                }
                let Some(kind) = dict_string(entry, kCFProxyTypeKey) else {
                    continue;
                };
                let scheme = match kind.as_str() {
                    "HTTP" | "HTTPS" => "http",
                    "SOCKS" => "socks5h",
                    _ => continue,
                };
                let Some(host) = dict_string(entry, kCFProxyHostNameKey) else {
                    continue;
                };
                let Some(port) = dict_port(entry) else {
                    continue;
                };
                resolution.proxy = Some((scheme, host, port));
                return;
            }
        }
    }

    unsafe fn dict_string(dictionary: CFDictionaryRef, key: CFStringRef) -> Option<String> {
        let value = CFDictionaryGetValue(dictionary, key);
        if value.is_null() {
            None
        } else {
            cf_string(value as CFStringRef)
        }
    }

    unsafe fn dict_port(dictionary: CFDictionaryRef) -> Option<u16> {
        let value = CFDictionaryGetValue(dictionary, kCFProxyPortNumberKey);
        if value.is_null() {
            return None;
        }
        let mut port: i32 = 0;
        if CFNumberGetValue(
            value as CFNumberRef,
            SINT32_TYPE,
            &mut port as *mut i32 as *mut c_void,
        ) == 0
            || !(1..=u16::MAX as i32).contains(&port)
        {
            return None;
        }
        Some(port as u16)
    }

    unsafe fn cf_string(string: CFStringRef) -> Option<String> {
        let mut buffer = [0 as c_char; 1024];
        if CFStringGetCString(
            string,
            buffer.as_mut_ptr(),
            buffer.len() as CFIndex,
            UTF8_ENCODING,
        ) == 0
        {
            return None;
        }
        CStr::from_ptr(buffer.as_ptr())
            .to_str()
            .ok()
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    }

    fn c_string(value: &str) -> Option<std::ffi::CString> {
        std::ffi::CString::new(value).ok()
    }
}
