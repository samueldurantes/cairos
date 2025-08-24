use crate::http::{AppState, Error, Result};
use axum::extract::{Json, State};
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub access_token: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub email: Option<String>,
}

#[derive(Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    let user_response = state
        .client
        .get("https://api.github.com/user")
        .bearer_auth(&payload.access_token)
        .send()
        .await
        .map_err(|e| {
            log::error!("Error on get user: {e}");
            Error::InternalServerError
        })?;

    let github_user: GitHubUser = user_response.json().await.map_err(|e| {
        log::error!("Error on jsonfy: {e}");
        Error::InternalServerError
    })?;

    let email = if let Some(email) = github_user.email {
        email
    } else {
        let email_response = state
            .client
            .get("https://api.github.com/user/emails")
            .bearer_auth(&payload.access_token)
            .send()
            .await
            .map_err(|e| {
                log::error!("Error on request github email: {e}");
                Error::InternalServerError
            })?;

        let emails: Vec<GitHubEmail> = email_response.json().await.map_err(|e| {
            log::error!("Error on request deserialize email_response: {e}");
            Error::InternalServerError
        })?;

        let Some(email) = emails
            .iter()
            .find(|email| email.primary)
            .map(|s| s.email.clone())
        else {
            return Err(Error::InternalServerError);
        };

        email
    };

    let now = time::OffsetDateTime::now_utc();

    let user_id = crate::queries::user::create(
        &state.db,
        &crate::queries::user::CreateParams {
            username: github_user.login,
            email,
            now,
        },
    )
    .await?;

    let token = generate_token();

    crate::queries::auth_tokens::create(&state.db, user_id, &token).await?;

    Ok(Json(LoginResponse { token }))
}

pub fn generate_token() -> String {
    let mut bytes = vec![0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
