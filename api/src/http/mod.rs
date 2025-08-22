use crate::config::Config;
use anyhow::Context;
use axum::Router;
use chrono::{DateTime, Utc};
use error::Error;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, EndpointNotSet, EndpointSet, RedirectUrl, TokenUrl,
    basic::BasicClient,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tokio::net::TcpListener;
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};

mod error;
mod routes;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub oauth_client:
        BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    pub oauth_states: Arc<tokio::sync::RwLock<HashMap<String, String>>>,
}

#[derive(Debug)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub code: String,
    pub state: String,
}

pub async fn serve(config: Config, db: PgPool) -> anyhow::Result<()> {
    let github_client_id = dotenvy::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set");
    let github_client_secret =
        dotenvy::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set");

    let github_client_id = ClientId::new(github_client_id);
    let github_client_secret = ClientSecret::new(github_client_secret);
    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())?;
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())?;

    let oauth_client = BasicClient::new(github_client_id)
        .set_client_secret(github_client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(RedirectUrl::new(
            "http://0.0.0.0:3000/auth/github/callback".to_string(),
        )?);

    let app_state = AppState {
        config: config.clone(),
        db,
        oauth_client,
        oauth_states: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
    };

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));
    let listener = TcpListener::bind(addr).await?;
    let app = app_router(app_state);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Error when trying to run HTTP server")
}

fn app_router(app_state: AppState) -> Router {
    Router::new().merge(routes::router(app_state)).layer((
        CompressionLayer::new(),
        TraceLayer::new_for_http().on_failure(()),
        TimeoutLayer::new(Duration::from_secs(30)),
        CatchPanicLayer::new(),
    ))
}

// async fn store_or_update_user(
//     db: &PgPool,
//     github_user: &GitHubUser,
//     email: Option<&str>,
// ) -> Result<i32, sqlx::Error> {
//     let now = chrono::Utc::now();

//     let user = sqlx::query!(
//         r#"
//         INSERT INTO users (username, email, created_at)
//         VALUES ($1, $2, $3)
//         ON CONFLICT (email)
//         DO UPDATE SET
//             username = EXCLUDED.username
//         WHERE users.email = EXCLUDED.email
//         RETURNING id
//         "#,
//         github_user.login,
//         email,
//         now
//     )
//     .fetch_one(db)
//     .await?;

//     Ok(user.id)
// }

// async fn get_user_by_id(db: &PgPool, user_id: i32) -> Result<Option<User>, sqlx::Error> {
//     let user = sqlx::query_as!(
//         User,
//         r#"
//         SELECT id, username, email, created_at
//         FROM users
//         WHERE id = $1
//         "#,
//         user_id
//     )
//     .fetch_optional(db)
//     .await?;

//     Ok(user)
// }

async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C signal handler");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install terminate signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
