use axum::body::Body;
use axum::extract::Request;
use axum::http::{Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;

pub async fn conditional(req: Request, next: Next) -> Response {
    let asked = matches!(*req.method(), Method::GET | Method::HEAD)
        .then(|| req.headers().get(header::IF_NONE_MATCH).cloned())
        .flatten();
    let mut response = next.run(req).await;
    let Some(asked) = asked else {
        return response;
    };
    let matches = response.status() == StatusCode::OK
        && response.headers().get(header::ETAG).is_some_and(|etag| {
            asked
                .as_bytes()
                .split(|b| *b == b',')
                .any(|tag| tag.trim_ascii() == etag.as_bytes() || tag.trim_ascii() == b"*")
        });
    if matches {
        *response.status_mut() = StatusCode::NOT_MODIFIED;
        *response.body_mut() = Body::empty();
        for name in [header::CONTENT_TYPE, header::CONTENT_LENGTH] {
            response.headers_mut().remove(name);
        }
    }
    response
}
