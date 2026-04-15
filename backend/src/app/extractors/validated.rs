use axum::Json;
use axum::extract::{FromRequest, Request};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::app::error::{ApiError, FieldError};
use crate::app::state::AppState;

pub struct Validated<T>(pub T);

impl<T> FromRequest<AppState> for Validated<T>
where
    T: DeserializeOwned + Validate,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &AppState) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;

        value.validate().map_err(|e| {
            let errors = e
                .field_errors()
                .into_iter()
                .flat_map(|(field, errs)| {
                    errs.iter().map(move |err| FieldError {
                        field: field.to_string(),
                        message: err
                            .message
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| err.code.to_string()),
                    })
                })
                .collect();
            ApiError::ValidationFailed(errors)
        })?;

        Ok(Validated(value))
    }
}
