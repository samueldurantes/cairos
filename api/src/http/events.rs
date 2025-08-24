use crate::http::{AppState, Error, Result};
use axum::{extract::State, http::StatusCode};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

pub async fn capture(
    State(_): State<AppState>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
) -> Result<StatusCode> {
    let auth = auth_header.ok_or(Error::Unauthorized {
        message: "Missing authorization header".to_string(),
    })?;

    let _token = auth.token();

    println!("Heartbeat received");

    Ok(StatusCode::OK)
}
