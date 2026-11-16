#import <Foundation/Foundation.h>
#import <CFNetwork/CFNetwork.h>

typedef void (*ResponseCallback)(void *, int, const char *, const char *);
typedef void (*DataCallback)(void *, const unsigned char *, size_t);
typedef void (*CompleteCallback)(void *, long);

@interface WarpaiHTTPContext : NSObject
@property void *rust;
@property ResponseCallback response;
@property DataCallback data;
@property CompleteCallback complete;
@property dispatch_source_t timer;
@property BOOL timedOut;
@end
@implementation WarpaiHTTPContext
@end

@interface WarpaiHTTPSession : NSObject <NSURLSessionDataDelegate>
@property NSURLSession *session;
@property NSMutableDictionary<NSNumber *, WarpaiHTTPContext *> *contexts;
@end

@implementation WarpaiHTTPSession
- (instancetype)init {
    if ((self = [super init])) {
        _contexts = [NSMutableDictionary new];
        NSURLSessionConfiguration *config = [NSURLSessionConfiguration ephemeralSessionConfiguration];
        config.HTTPCookieStorage = nil;
        config.HTTPShouldSetCookies = NO;
        config.URLCredentialStorage = nil;
        config.URLCache = nil;
        // Leave connectionProxyDictionary unset: macOS owns system/PAC routing.
        config.timeoutIntervalForRequest = DBL_MAX;
        config.timeoutIntervalForResource = DBL_MAX;
        _session = [NSURLSession sessionWithConfiguration:config delegate:self delegateQueue:nil];
    }
    return self;
}
- (WarpaiHTTPContext *)context:(NSURLSessionTask *)task {
    @synchronized(self.contexts) { return self.contexts[@(task.taskIdentifier)]; }
}
- (void)URLSession:(NSURLSession *)session dataTask:(NSURLSessionDataTask *)task
    didReceiveResponse:(NSURLResponse *)response
    completionHandler:(void (^)(NSURLSessionResponseDisposition))completionHandler {
    WarpaiHTTPContext *ctx = [self context:task];
    if (!ctx || ![response isKindOfClass:[NSHTTPURLResponse class]]) {
        completionHandler(NSURLSessionResponseCancel);
        return;
    }
    NSHTTPURLResponse *http = (NSHTTPURLResponse *)response;
    NSMutableString *headers = [NSMutableString new];
    for (NSString *name in http.allHeaderFields) [headers appendFormat:@"%@: %@\r\n", name, http.allHeaderFields[name]];
    ctx.response(ctx.rust, (int)http.statusCode, headers.UTF8String, response.URL.absoluteString.UTF8String);
    completionHandler(NSURLSessionResponseAllow);
}
- (void)URLSession:(NSURLSession *)session dataTask:(NSURLSessionDataTask *)task didReceiveData:(NSData *)data {
    WarpaiHTTPContext *ctx = [self context:task];
    if (ctx) {
        // Rust resumes once this chunk is consumed; never block the delegate queue.
        [task suspend];
        ctx.data(ctx.rust, data.bytes, data.length);
    }
}
- (void)URLSession:(NSURLSession *)session task:(NSURLSessionTask *)task
    willPerformHTTPRedirection:(NSHTTPURLResponse *)response newRequest:(NSURLRequest *)request
    completionHandler:(void (^)(NSURLRequest *))completionHandler {
    // The shared Rust transport applies redirect and credential policies.
    completionHandler(nil);
}
- (void)URLSession:(NSURLSession *)session task:(NSURLSessionTask *)task didCompleteWithError:(NSError *)error {
    WarpaiHTTPContext *ctx;
    @synchronized(self.contexts) {
        ctx = self.contexts[@(task.taskIdentifier)];
        [self.contexts removeObjectForKey:@(task.taskIdentifier)];
    }
    if (ctx.timer) dispatch_source_cancel(ctx.timer);
    if (ctx) ctx.complete(ctx.rust, ctx.timedOut ? NSURLErrorTimedOut : (error ? (long)error.code : 0));
}
@end

static WarpaiHTTPSession *sharedSession(void) {
    static WarpaiHTTPSession *session;
    static dispatch_once_t once;
    dispatch_once(&once, ^{ session = [WarpaiHTTPSession new]; });
    return session;
}

void *warpai_http_start(const char *url, const char *method, const char *headers,
    const unsigned char *body, size_t length, const char *bodyPath, bool redirects, double timeout,
    void *rust, ResponseCallback response, DataCallback data, CompleteCallback complete) {
    @autoreleasepool {
        WarpaiHTTPSession *owner = sharedSession();
        NSMutableURLRequest *request = [NSMutableURLRequest requestWithURL:[NSURL URLWithString:@(url)]];
        request.HTTPMethod = @(method);
        request.cachePolicy = NSURLRequestReloadIgnoringLocalCacheData;
        request.timeoutInterval = DBL_MAX;
        for (NSString *field in [@(headers) componentsSeparatedByString:@"\r\n"]) {
            NSRange colon = [field rangeOfString:@":"];
            if (colon.location == NSNotFound) continue;
            NSString *name = [field substringToIndex:colon.location];
            NSString *value = [[field substringFromIndex:colon.location + 1] stringByTrimmingCharactersInSet:NSCharacterSet.whitespaceCharacterSet];
            [request addValue:value forHTTPHeaderField:name];
        }
        if (length) request.HTTPBody = [NSData dataWithBytes:body length:length];
        NSURLSessionDataTask *task = bodyPath
            ? [owner.session uploadTaskWithRequest:request fromFile:[NSURL fileURLWithPath:@(bodyPath)]]
            : [owner.session dataTaskWithRequest:request];
        WarpaiHTTPContext *ctx = [WarpaiHTTPContext new];
        ctx.rust = rust;
        ctx.response = response;
        ctx.data = data;
        ctx.complete = complete;
        if (timeout > 0) {
            ctx.timer = dispatch_source_create(DISPATCH_SOURCE_TYPE_TIMER, 0, 0, dispatch_get_global_queue(QOS_CLASS_DEFAULT, 0));
            __weak WarpaiHTTPContext *weakContext = ctx;
            __weak NSURLSessionTask *weakTask = task;
            dispatch_source_set_timer(ctx.timer, dispatch_time(DISPATCH_TIME_NOW, (int64_t)(timeout * NSEC_PER_SEC)), DISPATCH_TIME_FOREVER, 0);
            dispatch_source_set_event_handler(ctx.timer, ^{
                weakContext.timedOut = YES;
                [weakTask cancel];
            });
        }
        @synchronized(owner.contexts) { owner.contexts[@(task.taskIdentifier)] = ctx; }
        if (ctx.timer) dispatch_resume(ctx.timer);
        // Return suspended so Rust can install its cancellation guard first.
        return (__bridge_retained void *)task;
    }
}
void warpai_http_resume(void *task) { [(__bridge NSURLSessionTask *)task resume]; }
void warpai_http_cancel(void *task) { [(__bridge NSURLSessionTask *)task cancel]; }
void warpai_http_release(void *task) { CFBridgingRelease(task); }

struct ProxyResult { CFArrayRef proxies; BOOL completed; };
static void proxyResolved(void *raw, CFArrayRef proxies, CFErrorRef error) {
    struct ProxyResult *result = raw;
    if (!error && proxies) result->proxies = CFRetain(proxies);
    result->completed = YES;
}

int warpai_proxy_for_url(const char *url, char *output, size_t capacity) {
    @autoreleasepool {
        NSURL *target = [NSURL URLWithString:@(url)];
        NSDictionary *settings = CFBridgingRelease(CFNetworkCopySystemProxySettings());
        if (!target || !settings) return -1;
        NSArray *proxies = CFBridgingRelease(CFNetworkCopyProxiesForURL(
            (__bridge CFURLRef)target, (__bridge CFDictionaryRef)settings));
        NSDictionary *proxy = proxies.firstObject;
        NSString *type = proxy[(__bridge NSString *)kCFProxyTypeKey];
        if ([type isEqual:(__bridge NSString *)kCFProxyTypeAutoConfigurationURL]
            || [type isEqual:(__bridge NSString *)kCFProxyTypeAutoConfigurationJavaScript]) {
            struct ProxyResult result = { NULL, NO };
            CFStreamClientContext context = { 0, &result, NULL, NULL, NULL };
            CFRunLoopSourceRef source;
            if ([type isEqual:(__bridge NSString *)kCFProxyTypeAutoConfigurationURL]) {
                NSURL *pac = proxy[(__bridge NSString *)kCFProxyAutoConfigurationURLKey];
                if (!pac) return -1;
                source = CFNetworkExecuteProxyAutoConfigurationURL((__bridge CFURLRef)pac,
                    (__bridge CFURLRef)target, proxyResolved, &context);
            } else {
                NSString *script = proxy[(__bridge NSString *)kCFProxyAutoConfigurationJavaScriptKey];
                if (!script) return -1;
                source = CFNetworkExecuteProxyAutoConfigurationScript((__bridge CFStringRef)script,
                    (__bridge CFURLRef)target, proxyResolved, &context);
            }
            if (!source) return -1;
            CFRunLoopRef loop = CFRunLoopGetCurrent();
            CFRunLoopAddSource(loop, source, kCFRunLoopDefaultMode);
            CFAbsoluteTime deadline = CFAbsoluteTimeGetCurrent() + 5;
            while (!result.completed && CFAbsoluteTimeGetCurrent() < deadline)
                CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.05, true);
            // Invalidate before the stack-owned callback context goes away, including timeout.
            CFRunLoopSourceInvalidate(source);
            CFRunLoopRemoveSource(loop, source, kCFRunLoopDefaultMode);
            CFRelease(source);
            proxies = CFBridgingRelease(result.proxies);
            proxy = proxies.firstObject;
            type = proxy[(__bridge NSString *)kCFProxyTypeKey];
        }
        NSString *route;
        if ([type isEqual:(__bridge NSString *)kCFProxyTypeNone]) route = @"direct://";
        else {
            // CFNetwork's HTTPS type denotes HTTP CONNECT, not TLS to the proxy.
            NSString *scheme = ([type isEqual:(__bridge NSString *)kCFProxyTypeHTTP]
                || [type isEqual:(__bridge NSString *)kCFProxyTypeHTTPS]) ? @"http"
                : [type isEqual:(__bridge NSString *)kCFProxyTypeSOCKS] ? @"socks5" : nil;
            NSString *host = proxy[(__bridge NSString *)kCFProxyHostNameKey];
            NSNumber *port = proxy[(__bridge NSString *)kCFProxyPortNumberKey];
            if (!scheme || !host.length || !port || port.intValue < 1 || port.intValue > 65535) return -1;
            // Never extract credentials from the system credential store.
            if (proxy[(__bridge NSString *)kCFProxyUsernameKey] || proxy[(__bridge NSString *)kCFProxyPasswordKey]) return -1;
            NSURLComponents *parts = [NSURLComponents new];
            parts.scheme = scheme; parts.host = host; parts.port = port;
            route = parts.string;
        }
        const char *bytes = route.UTF8String;
        if (!bytes || strlen(bytes) >= capacity) return -1;
        memcpy(output, bytes, strlen(bytes) + 1);
        return 0;
    }
}
