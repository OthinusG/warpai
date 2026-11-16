// Platform libraries own proxy discovery/PAC, TLS, and HTTP framing.
#include <atomic>
#include <chrono>
#include <condition_variable>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <memory>
#include <mutex>
#include <string>
#include <thread>
#include <vector>
#ifdef _WIN32
#include <windows.h>
#include <winhttp.h>
#else
#include <libsoup/soup.h>
#endif

using ResponseCallback = void (*)(void *, int, const char *, const char *);
using DataCallback = void (*)(void *, const unsigned char *, size_t);
using CompleteCallback = void (*)(void *, int64_t);

struct Task {
    std::mutex mutex;
    std::condition_variable changed;
    unsigned credits = 0;
    std::atomic<bool> canceled { false }, done { false }, timedOut { false };
    void *rust;
    ResponseCallback response;
    DataCallback data;
    CompleteCallback complete;
#ifdef _WIN32
    HINTERNET request = nullptr;
#else
    GCancellable *cancellable = g_cancellable_new();
    ~Task() { g_object_unref(cancellable); }
#endif
    bool wait() {
        std::unique_lock<std::mutex> lock(mutex);
        changed.wait(lock, [&] { return credits || canceled; });
        if (canceled) return false;
        --credits;
        return true;
    }
    void cancel() {
        std::lock_guard<std::mutex> lock(mutex);
        canceled = true;
#ifdef _WIN32
        if (request) { WinHttpCloseHandle(request); request = nullptr; }
#else
        g_cancellable_cancel(cancellable);
#endif
        changed.notify_all();
    }
    void finish(int64_t error) {
        {
            std::lock_guard<std::mutex> lock(mutex);
            done = true;
            if (timedOut) error = -1001;
            changed.notify_all();
        }
        complete(rust, error);
    }
};

#ifdef _WIN32
static std::wstring wide(const std::string &s) {
    int size = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, s.c_str(), -1, nullptr, 0);
    if (!size) return {};
    std::wstring result(size, 0);
    MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, s.c_str(), -1, &result[0], size);
    result.pop_back();
    return result;
}
static std::string utf8(const wchar_t *s) {
    int size = WideCharToMultiByte(CP_UTF8, 0, s, -1, nullptr, 0, nullptr, nullptr);
    if (!size) return {};
    std::string result(size, 0);
    WideCharToMultiByte(CP_UTF8, 0, s, -1, &result[0], size, nullptr, nullptr);
    result.pop_back();
    return result;
}
struct InternetHandle {
    HINTERNET value;
    ~InternetHandle() { if (value) WinHttpCloseHandle(value); }
};

struct ProxyLookup {
    std::mutex mutex;
    std::condition_variable changed;
    bool done = false;
    std::string route;
};
static void CALLBACK proxyCallback(HINTERNET resolver, DWORD_PTR context, DWORD status, void *, DWORD) {
    auto owner = reinterpret_cast<std::shared_ptr<ProxyLookup> *>(context);
    if (!owner) return;
    if (status == WINHTTP_CALLBACK_STATUS_HANDLE_CLOSING) { delete owner; return; }
    if (status != WINHTTP_CALLBACK_STATUS_GETPROXYFORURL_COMPLETE
        && status != WINHTTP_CALLBACK_STATUS_REQUEST_ERROR) return;
    auto lookup = *owner;
    std::string route;
    if (status == WINHTTP_CALLBACK_STATUS_GETPROXYFORURL_COMPLETE) {
        WINHTTP_PROXY_RESULT result = {};
        if (WinHttpGetProxyResult(resolver, &result) == ERROR_SUCCESS) {
            if (result.cEntries) {
                auto &entry = result.pEntries[0];
                if (!entry.fProxy || entry.fBypass) route = "direct://";
                else {
                    // WinHTTP's generic SOCKS result is SOCKS4; agy only supports SOCKS5.
                    const char *scheme = entry.ProxyScheme == INTERNET_SCHEME_HTTP ? "http://"
                        : entry.ProxyScheme == INTERNET_SCHEME_HTTPS ? "https://" : nullptr;
                    if (scheme && entry.pwszProxy && entry.ProxyPort) {
                        auto host = utf8(entry.pwszProxy);
                        if (host.find(':') != std::string::npos && host.front() != '[') host = "[" + host + "]";
                        route = scheme + host + ":" + std::to_string(entry.ProxyPort);
                    }
                }
            }
            WinHttpFreeProxyResult(&result);
        }
    }
    std::lock_guard<std::mutex> lock(lookup->mutex);
    lookup->route = route; lookup->done = true; lookup->changed.notify_all();
}
extern "C" int warpai_proxy_for_url(const char *url, char *output, size_t capacity) {
    struct Config {
        WINHTTP_CURRENT_USER_IE_PROXY_CONFIG value = {};
        ~Config() {
            GlobalFree(value.lpszAutoConfigUrl);
            GlobalFree(value.lpszProxy);
            GlobalFree(value.lpszProxyBypass);
        }
    } config;
    if (!WinHttpGetIEProxyConfigForCurrentUser(&config.value)) return -1;
    InternetHandle session { WinHttpOpen(L"Warpai usage", WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
        WINHTTP_NO_PROXY_NAME, WINHTTP_NO_PROXY_BYPASS, WINHTTP_FLAG_ASYNC) };
    if (!session.value) return -1;
    if (WinHttpSetStatusCallback(session.value, proxyCallback,
        WINHTTP_CALLBACK_FLAG_GETPROXYFORURL_COMPLETE | WINHTTP_CALLBACK_FLAG_REQUEST_ERROR
        | WINHTTP_CALLBACK_FLAG_HANDLES, 0) == WINHTTP_INVALID_STATUS_CALLBACK) return -1;
    HINTERNET raw = nullptr;
    if (WinHttpCreateProxyResolver(session.value, &raw) != ERROR_SUCCESS) return -1;
    InternetHandle resolver { raw };
    auto lookup = std::make_shared<ProxyLookup>();
    auto owner = new std::shared_ptr<ProxyLookup>(lookup);
    DWORD_PTR context = reinterpret_cast<DWORD_PTR>(owner);
    if (!WinHttpSetOption(raw, WINHTTP_OPTION_CONTEXT_VALUE, &context, sizeof(context))) {
        delete owner; return -1;
    }
    WINHTTP_AUTOPROXY_OPTIONS options = {};
    options.dwFlags = WINHTTP_AUTOPROXY_ALLOW_STATIC | WINHTTP_AUTOPROXY_ALLOW_CM;
    if (config.value.lpszAutoConfigUrl) {
        options.dwFlags |= WINHTTP_AUTOPROXY_CONFIG_URL;
        options.lpszAutoConfigUrl = config.value.lpszAutoConfigUrl;
    }
    if (config.value.fAutoDetect) {
        options.dwFlags |= WINHTTP_AUTOPROXY_AUTO_DETECT;
        options.dwAutoDetectFlags = WINHTTP_AUTO_DETECT_TYPE_DHCP | WINHTTP_AUTO_DETECT_TYPE_DNS_A;
    }
    auto target = wide(url);
    if (WinHttpGetProxyForUrlEx(raw, target.c_str(), &options, context) != ERROR_IO_PENDING) return -1;
    std::unique_lock<std::mutex> lock(lookup->mutex);
    if (!lookup->changed.wait_for(lock, std::chrono::seconds(5), [&] { return lookup->done; })
        || lookup->route.empty() || lookup->route.size() >= capacity) return -1;
    memcpy(output, lookup->route.c_str(), lookup->route.size() + 1);
    return 0;
}
static void run(const std::shared_ptr<Task> &task, const std::string &url,
    const std::string &method, const std::string &headers,
    const std::vector<unsigned char> &body, const std::string &path) {
    if (!task->wait()) { task->finish(ERROR_OPERATION_ABORTED); return; }
    auto target = wide(url);
    URL_COMPONENTS parts = {};
    parts.dwStructSize = sizeof(parts);
    parts.dwHostNameLength = parts.dwUrlPathLength = parts.dwExtraInfoLength = (DWORD)-1;
    if (!WinHttpCrackUrl(target.c_str(), 0, 0, &parts)) { task->finish(GetLastError()); return; }
    InternetHandle session { WinHttpOpen(L"Warpai", WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
        WINHTTP_NO_PROXY_NAME, WINHTTP_NO_PROXY_BYPASS, 0) };
    if (!session.value) { task->finish(GetLastError()); return; }
    std::wstring host(parts.lpszHostName, parts.dwHostNameLength);
    InternetHandle connection { WinHttpConnect(session.value, host.c_str(), parts.nPort, 0) };
    if (!connection.value) { task->finish(GetLastError()); return; }
    std::wstring resource(parts.lpszUrlPath, parts.dwUrlPathLength);
    resource.append(parts.lpszExtraInfo, parts.dwExtraInfoLength);
    if (resource.empty()) resource = L"/";
    HINTERNET request = WinHttpOpenRequest(connection.value, wide(method).c_str(), resource.c_str(),
        nullptr, WINHTTP_NO_REFERER, WINHTTP_DEFAULT_ACCEPT_TYPES,
        parts.nScheme == INTERNET_SCHEME_HTTPS ? WINHTTP_FLAG_SECURE : 0);
    if (!request) { task->finish(GetLastError()); return; }
    {
        std::lock_guard<std::mutex> lock(task->mutex);
        if (task->canceled) { WinHttpCloseHandle(request); request = nullptr; }
        else task->request = request;
    }
    int64_t error = 0;
    auto operation = [&] {
        if (!request) return false;
        DWORD disabled = WINHTTP_DISABLE_REDIRECTS | WINHTTP_DISABLE_COOKIES;
        DWORD decompress = WINHTTP_DECOMPRESSION_FLAG_GZIP | WINHTTP_DECOMPRESSION_FLAG_DEFLATE;
        if (!WinHttpSetOption(request, WINHTTP_OPTION_DECOMPRESSION, &decompress, sizeof(decompress))) return false;
        DWORD logon = WINHTTP_AUTOLOGON_SECURITY_LEVEL_HIGH;
        if (!WinHttpSetOption(request, WINHTTP_OPTION_AUTOLOGON_POLICY, &logon, sizeof(logon))) return false;
        if (!WinHttpSetOption(request, WINHTTP_OPTION_DISABLE_FEATURE, &disabled, sizeof(disabled))) return false;
        auto fields = wide(headers);
        if (!fields.empty() && !WinHttpAddRequestHeaders(request, fields.c_str(), (DWORD)-1, WINHTTP_ADDREQ_FLAG_ADD)) return false;
        std::ifstream upload;
        uint64_t length = body.size();
        if (!path.empty()) {
            upload.open(wide(path), std::ios::binary | std::ios::ate);
            if (!upload) return false;
            length = (uint64_t)upload.tellg(); upload.seekg(0);
        }
        // shortcut: synchronous WinHTTP upload length is DWORD; use chunked uploads before supporting files above 4 GiB.
        if (length > MAXDWORD) { SetLastError(ERROR_FILE_TOO_LARGE); return false; }
        if (!WinHttpSendRequest(request, WINHTTP_NO_ADDITIONAL_HEADERS, 0,
            path.empty() && length ? (void *)body.data() : WINHTTP_NO_REQUEST_DATA,
            path.empty() ? (DWORD)length : 0, (DWORD)length, 0)) return false;
        unsigned char buffer[64 * 1024];
        while (upload && upload.read((char *)buffer, sizeof(buffer)).gcount() > 0) {
            DWORD count = (DWORD)upload.gcount(), written = 0;
            if (!WinHttpWriteData(request, buffer, count, &written) || count != written) return false;
        }
        if (!WinHttpReceiveResponse(request, nullptr)) return false;
        DWORD status = 0, size = sizeof(status);
        if (!WinHttpQueryHeaders(request, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_HEADER_NAME_BY_INDEX, &status, &size, WINHTTP_NO_HEADER_INDEX)) return false;
        size = 0;
        WinHttpQueryHeaders(request, WINHTTP_QUERY_RAW_HEADERS_CRLF, WINHTTP_HEADER_NAME_BY_INDEX, nullptr, &size, WINHTTP_NO_HEADER_INDEX);
        if (GetLastError() != ERROR_INSUFFICIENT_BUFFER) return false;
        std::vector<wchar_t> raw(size / sizeof(wchar_t) + 1);
        if (!WinHttpQueryHeaders(request, WINHTTP_QUERY_RAW_HEADERS_CRLF, WINHTTP_HEADER_NAME_BY_INDEX, raw.data(), &size, WINHTTP_NO_HEADER_INDEX)) return false;
        std::string fieldsOut = utf8(raw.data());
        auto firstLine = fieldsOut.find("\r\n");
        if (firstLine == std::string::npos) return false;
        fieldsOut.erase(0, firstLine + 2);
        task->response(task->rust, (int)status, fieldsOut.c_str(), url.c_str());
        DWORD read = 0, available = 0;
        while (WinHttpQueryDataAvailable(request, &available)) {
            if (!available) return true;
            DWORD wanted = available < sizeof(buffer) ? available : sizeof(buffer);
            if (!WinHttpReadData(request, buffer, wanted, &read)) return false;
            if (!read) return true;
            task->data(task->rust, buffer, read);
            if (!task->wait()) { SetLastError(ERROR_OPERATION_ABORTED); return false; }
        }
        return false;
    };
    if (!operation()) error = GetLastError() ? GetLastError() : -1;
    {
        std::lock_guard<std::mutex> lock(task->mutex);
        if (task->request) { WinHttpCloseHandle(task->request); task->request = nullptr; }
    }
    task->finish(error);
}
#else
static void ensureMainLoop() {
    static std::once_flag mainLoop;
    std::call_once(mainLoop, [] {
        // GSettings proxy changes need the GLib main context to keep dispatching.
        std::thread([] { g_main_loop_run(g_main_loop_new(nullptr, false)); }).detach();
    });
}
extern "C" int warpai_proxy_for_url(const char *url, char *output, size_t capacity) {
    ensureMainLoop();
    GCancellable *cancel = g_cancellable_new();
    std::mutex mutex;
    std::condition_variable changed;
    bool done = false;
    std::thread deadline([&] {
        std::unique_lock<std::mutex> lock(mutex);
        if (!changed.wait_for(lock, std::chrono::seconds(5), [&] { return done; }))
            g_cancellable_cancel(cancel);
    });
    GError *error = nullptr;
    gchar **proxies = g_proxy_resolver_lookup(g_proxy_resolver_get_default(), url, cancel, &error);
    {
        std::lock_guard<std::mutex> lock(mutex);
        done = true; changed.notify_all();
    }
    deadline.join();
    int result = -1;
    if (!error && proxies && proxies[0] && strlen(proxies[0]) < capacity) {
        memcpy(output, proxies[0], strlen(proxies[0]) + 1);
        result = 0;
    }
    if (error) g_error_free(error);
    g_strfreev(proxies);
    g_object_unref(cancel);
    return result;
}
static void header(const char *name, const char *value, gpointer context) {
    // Native header octets become UTF-8 for the FFI boundary, matching Foundation.
    gchar *text = g_convert(value, -1, "UTF-8", "ISO-8859-1", nullptr, nullptr, nullptr);
    auto &out = *static_cast<std::string *>(context);
    out += name; out += ": "; out += text ? text : ""; out += "\r\n";
    g_free(text);
}
static void run(const std::shared_ptr<Task> &task, const std::string &url,
    const std::string &method, const std::string &headers,
    const std::vector<unsigned char> &body, const std::string &path) {
    if (!task->wait()) { task->finish(-1); return; }
    ensureMainLoop();
    SoupSession *session = soup_session_new_with_options(SOUP_SESSION_PROXY_RESOLVER,
        g_proxy_resolver_get_default(), SOUP_SESSION_ADD_FEATURE_BY_TYPE,
        SOUP_TYPE_CONTENT_DECODER, nullptr);
    SoupMessage *message = soup_message_new(method.c_str(), url.c_str());
    if (!message) { g_object_unref(session); task->finish(-1); return; }
    soup_message_set_flags(message, SOUP_MESSAGE_NO_REDIRECT);
    size_t offset = 0;
    while (offset < headers.size()) {
        size_t end = headers.find("\r\n", offset), colon = headers.find(':', offset);
        if (end == std::string::npos) break;
        if (colon < end) {
            std::string name = headers.substr(offset, colon - offset), value = headers.substr(colon + 2, end - colon - 2);
            gchar *octets = g_convert(value.c_str(), -1, "ISO-8859-1", "UTF-8", nullptr, nullptr, nullptr);
            if (octets) soup_message_headers_append(message->request_headers, name.c_str(), octets);
            g_free(octets);
        }
        offset = end + 2;
    }
    GError *error = nullptr;
    GMappedFile *upload = nullptr;
    if (!path.empty()) {
        upload = g_mapped_file_new(path.c_str(), false, &error);
        if (upload) soup_message_body_append(message->request_body, SOUP_MEMORY_STATIC,
            g_mapped_file_get_contents(upload), g_mapped_file_get_length(upload));
    } else if (!body.empty()) soup_message_body_append(message->request_body, SOUP_MEMORY_COPY,
        (const char *)body.data(), body.size());
    soup_message_body_complete(message->request_body);
    GInputStream *stream = error ? nullptr : soup_session_send(session, message, task->cancellable, &error);
    if (stream) {
        std::string fields;
        soup_message_headers_foreach(message->response_headers, header, &fields);
        task->response(task->rust, message->status_code, fields.c_str(), url.c_str());
        unsigned char buffer[64 * 1024];
        gssize count;
        while ((count = g_input_stream_read(stream, buffer, sizeof(buffer), task->cancellable, &error)) > 0) {
            task->data(task->rust, buffer, (size_t)count);
            if (!task->wait()) break;
        }
        g_object_unref(stream);
    }
    int64_t code = error ? -(10000LL + error->code) : (task->canceled ? -1 : 0);
    if (error) g_error_free(error);
    if (upload) g_mapped_file_unref(upload);
    g_object_unref(message);
    g_object_unref(session);
    task->finish(code);
}
#endif

extern "C" void *warpai_http_start(const char *url, const char *method, const char *headers,
    const unsigned char *body, size_t length, const char *bodyPath, bool, double timeout,
    void *rust, ResponseCallback response, DataCallback data, CompleteCallback complete) {
    auto task = std::make_shared<Task>();
    task->rust = rust; task->response = response; task->data = data; task->complete = complete;
    std::string target(url), verb(method), fields(headers), path(bodyPath ? bodyPath : "");
    std::vector<unsigned char> bytes(body, body + length);
    std::thread([task, target, verb, fields, bytes, path] { run(task, target, verb, fields, bytes, path); }).detach();
    if (timeout > 0) std::thread([task, timeout] {
        std::unique_lock<std::mutex> lock(task->mutex);
        if (!task->changed.wait_for(lock, std::chrono::duration<double>(timeout), [&] { return task->done || task->canceled; })) {
            task->timedOut = true;
            lock.unlock(); task->cancel();
        }
    }).detach();
    return new std::shared_ptr<Task>(task);
}
extern "C" void warpai_http_resume(void *raw) {
    auto &task = *static_cast<std::shared_ptr<Task> *>(raw);
    std::lock_guard<std::mutex> lock(task->mutex);
    ++task->credits; task->changed.notify_all();
}
extern "C" void warpai_http_cancel(void *raw) { (*static_cast<std::shared_ptr<Task> *>(raw))->cancel(); }
extern "C" void warpai_http_release(void *raw) { delete static_cast<std::shared_ptr<Task> *>(raw); }
