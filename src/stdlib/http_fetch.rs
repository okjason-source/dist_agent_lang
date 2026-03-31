//! Bounded HTTP GET for DAL (`http::fetch_text` / `http::fetch`).
//!
//! Uses the same implementation and env policy as the agent **`fetch_url`** tool
//! ([`crate::stdlib::ai::fetch_url_text_result`]): `DAL_HTTP_FETCH_*`, `http-interface` feature.

/// GET an `http` or `https` URL and return the response body as plain text (HTML stripped best-effort).
/// Requires the **`http-interface`** Cargo feature (enabled by default).
pub fn fetch_text(url: &str) -> Result<String, String> {
    crate::stdlib::ai::fetch_url_text_result(url)
}
