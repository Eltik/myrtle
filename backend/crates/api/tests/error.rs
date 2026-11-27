#![allow(clippy::unwrap_used)]

use axum::body::to_bytes;
use axum::response::IntoResponse;
use myrtle_api::ApiError;
use myrtle_api::error::ErrorCode;
use myrtle_app::AppError;
use myrtle_hypergryph::{Domain, UpstreamEnvelope, UpstreamError};
use myrtle_types::{FieldError, Server};
use serde_json::{Value, json};

async fn rendered(error: AppError) -> (u16, Value) {
    let response = ApiError(error).into_response();
    let status = response.status().as_u16();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap())
}

#[test]
fn codes_and_statuses_are_the_frozen_contract() {
    let expected = [
        ("BAD_REQUEST", 400),
        ("GAME_LOGIN_REQUIRED", 400),
        ("UNAUTHORIZED", 401),
        ("FORBIDDEN", 403),
        ("NOT_FOUND", 404),
        ("CONFLICT", 409),
        ("VALIDATION_FAILED", 422),
        ("RATE_LIMITED", 429),
        ("INTERNAL_ERROR", 500),
        ("SERVICE_UNAVAILABLE", 503),
        ("GAME_MAINTENANCE", 503),
        ("TIMEOUT", 504),
    ];
    let actual: Vec<(&str, u16)> = ErrorCode::ALL
        .iter()
        .map(|c| (c.as_str(), c.status().as_u16()))
        .collect();
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn every_app_error_lands_on_its_code() {
    let cases = [
        (AppError::bad_request("no"), 400, "BAD_REQUEST", "no"),
        (AppError::Unauthorized, 401, "UNAUTHORIZED", "unauthorized"),
        (AppError::Forbidden, 403, "FORBIDDEN", "forbidden"),
        (AppError::NotFound(None), 404, "NOT_FOUND", "not found"),
        (
            AppError::NotFound(Some("no such list".into())),
            404,
            "NOT_FOUND",
            "no such list",
        ),
        (AppError::Conflict("taken".into()), 409, "CONFLICT", "taken"),
        (AppError::RateLimited, 429, "RATE_LIMITED", "rate limited"),
        (
            AppError::GameLoginRequired,
            400,
            "GAME_LOGIN_REQUIRED",
            "no game session - login again",
        ),
        (
            AppError::Unavailable,
            503,
            "SERVICE_UNAVAILABLE",
            "service unavailable",
        ),
        (
            AppError::Upstream(UpstreamError::NotLoggedIn),
            401,
            "UNAUTHORIZED",
            "unauthorized",
        ),
        (
            AppError::Upstream(UpstreamError::ApiCode {
                url: "https://upstream.example/api".into(),
                code: 7,
                message: "no such album".into(),
            }),
            400,
            "BAD_REQUEST",
            "no such album (7)",
        ),
    ];
    for (error, status, code, message) in cases {
        let (got_status, body) = rendered(error).await;
        assert_eq!(got_status, status, "{code}");
        assert_eq!(
            body,
            json!({ "error": { "code": code, "message": message } })
        );
    }
}

#[tokio::test]
async fn an_internal_error_keeps_its_detail_off_the_wire() {
    let (status, body) = rendered(AppError::Internal("pool at 10.0.0.4 refused".into())).await;
    assert_eq!(status, 500);
    assert_eq!(
        body,
        json!({ "error": { "code": "INTERNAL_ERROR", "message": "internal server error" } })
    );
}

#[tokio::test]
async fn validation_carries_details_and_maintenance_carries_its_end() {
    let details = vec![FieldError {
        field: "name".into(),
        message: "too long".into(),
    }];
    let (status, body) = rendered(AppError::Validation(details)).await;
    assert_eq!(status, 422);
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");
    assert_eq!(
        body["error"]["details"],
        json!([{ "field": "name", "message": "too long" }])
    );

    let (status, body) = rendered(AppError::GameMaintenance { ends_at: Some(42) }).await;
    assert_eq!(status, 503);
    assert_eq!(body["error"]["code"], "GAME_MAINTENANCE");
    assert_eq!(body["error"]["ends_at"], 42);
}

#[tokio::test]
async fn an_unloaded_server_is_unavailable_not_unsupported() {
    let (status, body) = rendered(AppError::Upstream(UpstreamError::DomainNotFound {
        server: Server::EN,
        domain: Domain::U8,
    }))
    .await;
    assert_eq!(status, 503);
    assert_eq!(
        body,
        json!({ "error": { "code": "SERVICE_UNAVAILABLE", "message": "service unavailable" } })
    );
}

#[tokio::test]
async fn a_yostar_refusal_reaches_the_client_as_its_message_and_code() {
    let cases = [
        (
            100_303,
            "The code is wrong, has expired, or was already used",
            "The code is wrong, has expired, or was already used (100303)",
        ),
        (
            100_302,
            "Yostar has emailed this address too many codes recently. Wait a while before requesting another",
            "Yostar has emailed this address too many codes recently. Wait a while before requesting another (100302)",
        ),
        (
            100_555,
            "Requests are too frequent",
            "Requests are too frequent (100555)",
        ),
    ];
    for (code, msg, shown) in cases {
        let refusal = UpstreamEnvelope {
            status_code: 200,
            code,
            msg: msg.into(),
            ..UpstreamEnvelope::default()
        };
        let (status, body) = rendered(AppError::Upstream(UpstreamError::Envelope(Box::new(
            refusal,
        ))))
        .await;
        assert_eq!(status, 400);
        assert_eq!(
            body,
            json!({ "error": { "code": "BAD_REQUEST", "message": shown } })
        );
    }
}
