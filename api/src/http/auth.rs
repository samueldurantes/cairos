use crate::http::{AppState, Error, GitHubUser, Result};
use axum::extract::{Json, State};

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
pub struct AuthTokenPayload {
    pub access_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterResponse {
    pub message: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<AuthTokenPayload>,
) -> Result<Json<RegisterResponse>> {
    let AuthTokenPayload { access_token } = payload;
    let AppState { db, client, .. } = state;

    let user_response = client
        .get("https://api.github.com/user")
        .bearer_auth(&access_token)
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
        let email_response = client
            .get("https://api.github.com/user/emails")
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| {
                log::error!("Error on request github email: {e}");
                Error::InternalServerError
            })?;

        let emails: Vec<serde_json::Value> = email_response.json().await.map_err(|e| {
            log::error!("Error on request deserialize email_response: {e}");
            Error::InternalServerError
        })?;

        let Some(email) = emails
            .iter()
            .find(|email| email["primary"].as_bool().unwrap_or(false))
            .map(|s| s.to_string())
        else {
            return Err(Error::InternalServerError);
        };

        email
    };

    let user_id = match store_or_update_user(
        &db,
        &CreateUser {
            username: github_user.login,
            email,
        },
    )
    .await
    {
        Ok(id) => id,
        Err(_) => return Err(Error::InternalServerError),
    };

    store_auth_token(&db, user_id, &access_token).await?;

    Ok(Json(RegisterResponse {
        message: "Registered succesfully!".to_string(),
    }))
}

struct CreateUser {
    username: String,
    email: String,
}

async fn store_auth_token(
    db: &PgPool,
    user_id: i32,
    access_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#" INSERT INTO auth_tokens (user_id, token) VALUES ($1, $2) "#,
        user_id,
        access_token,
    )
    .fetch_one(db)
    .await?;

    Ok(())
}

async fn store_or_update_user(db: &PgPool, user: &CreateUser) -> Result<i32, sqlx::Error> {
    let user = sqlx::query!(
        r#"
        INSERT INTO users (username, email)
        VALUES ($1, $2)
        ON CONFLICT (email)
        DO UPDATE SET
            username = EXCLUDED.username
        WHERE users.email = EXCLUDED.email
        RETURNING id
        "#,
        user.username,
        user.email,
    )
    .fetch_one(db)
    .await?;

    Ok(user.id)
}
