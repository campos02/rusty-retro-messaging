use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use hyper::header::CONTENT_TYPE;

pub async fn content_type_xml(mut request: Request, next: Next) -> Response {
    let headers = request.headers_mut();
    if let Some(content_type) = headers.get_mut(CONTENT_TYPE) {
        *content_type = HeaderValue::from_static("application/xml");
    } else {
        headers.append(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
    }

    next.run(request).await
}
