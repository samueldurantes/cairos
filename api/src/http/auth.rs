use crate::http::{AppState, Error, GitHubUser, Result, User};
use axum::{extract::State, response::Json};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize)]
pub struct AuthTokenPayload {
    access_token: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<AuthTokenPayload>,
) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("CAIROS/1.0.0")
        .build()
        .expect("Error on build Client.");

    let user_response = client
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
        let email_response = client
            .get("https://api.github.com/user/emails")
            .bearer_auth(&payload.access_token)
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
        &state.db,
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

    Ok(())
}

struct CreateUser {
    username: String,
    email: String,
}

async fn store_or_update_user(db: &PgPool, user: &CreateUser) -> Result<i32, sqlx::Error> {
    // let other = sqlx::query!(
    //     r#"
    //     INSERT INTO users (username, email)
    //     VALUES ($1, $2)
    //     ON CONFLICT (email)
    //     DO UPDATE SET
    //         username = EXCLUDED.username
    //     WHERE users.email = EXCLUDED.email
    //     RETURNING id
    //     "#,
    //     user.username,
    //     user.email,
    // )
    // .fetch_one(db)
    // .await?;

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

async fn get_user_by_id(db: &PgPool, user_id: i32) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email, created_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(db)
    .await?;

    Ok(user)
}
