use openidconnect::{HttpRequest, HttpResponse};

///
/// Asynchronous HTTP client.
///
/// # Errors
/// Fails when communication with endpoint failed
#[inline]
pub async fn async_http_client(
    request: HttpRequest,
) -> Result<HttpResponse, reqwest::Error> {
    let client = reqwest::Client::builder()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let mut request_builder = client
        .request(
            http::Method::from_bytes(request.method.as_str().as_ref())
                .expect("failed to convert Method from http 0.2 to 0.1"),
            request.url.as_str(),
        )
        .body(request.body);
    for (name, value) in &request.headers {
        request_builder = request_builder.header(name.as_str(), value.as_bytes());
    }
    let request = request_builder.build()?;

    let response = client.execute(request).await?;

    let status_code = response.status();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                http_01::header::HeaderName::from_bytes(name.as_str().as_ref())
                    .expect("failed to convert HeaderName from http 0.2 to 0.1"),
                http_01::HeaderValue::from_bytes(value.as_bytes())
                    .expect("failed to convert HeaderValue from http 0.2 to 0.1"),
            )
        })
        .collect::<http_01::HeaderMap>();
    let chunks = response.bytes().await?;
    Ok(HttpResponse {
        status_code: http_01::StatusCode::from_u16(status_code.as_u16())
            .expect("failed to convert StatusCode from http 0.2 to 0.1"),
        headers,
        body: chunks.to_vec(),
    })
}
