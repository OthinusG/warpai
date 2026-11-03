// Platform libraries own proxy discovery/PAC, TLS, and HTTP framing.
#include <atomic>
#include <chrono>
#include <condition_variable>
#include <cstdint>
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
    static std::once_flag mainLoop;
    std::call_once(mainLoop, [] {
        // GSettings proxy changes need the GLib main context to keep dispatching.
        std::thread([] { g_main_loop_run(g_main_loop_new(nullptr, false)); }).detach();
    });
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
    int64_t code = error ? error->code : (task->canceled ? -1 : 0);
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
