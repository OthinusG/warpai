//! Test-only helpers shared across `agent_sdk` unit tests.

/// Exercise the same platform transport as the application.
pub(super) fn build_test_http_client() -> http_client::Client {
    http_client::Client::new_for_test()
}
