use axum::{extract::Request, middleware::Next, response::Response};

/// Axum middleware that injects standard security response headers.
///
/// Headers set:
/// - `X-Frame-Options: DENY` — prevent clickjacking
/// - `X-Content-Type-Options: nosniff` — prevent MIME sniffing
/// - `X-XSS-Protection: 1; mode=block` — legacy XSS filter
/// - `Strict-Transport-Security` — enforce HTTPS
/// - `Content-Security-Policy` — restrict resource origins
/// - `Referrer-Policy` — limit referrer leakage
/// - `Permissions-Policy` — restrict browser APIs
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    use axum::http::HeaderValue;

    headers.insert("x-frame-options",         HeaderValue::from_static("DENY"));
    headers.insert("x-content-type-options",  HeaderValue::from_static("nosniff"));
    headers.insert("x-xss-protection",        HeaderValue::from_static("1; mode=block"));
    headers.insert(
        "strict-transport-security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.entry("content-security-policy").or_insert_with(|| {
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: blob:; \
             connect-src 'self' wss:;"
        )
    });
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );

    response
}
