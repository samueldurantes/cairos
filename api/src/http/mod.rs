use crate::config::Config;
use anyhow::Context;
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Redirect},
    routing::get,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use chrono::{DateTime, Utc};
use error::Error;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
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
    Router::new()
        .route("/", get(success))
        .route("/auth/github", get(github_auth))
        .route("/auth/github/callback", get(github_callback))
        .route("/auth/logout", get(logout))
        .route("/heartbeat", get(heartbeat))
        .layer((
            CompressionLayer::new(),
            TraceLayer::new_for_http().on_failure(()),
            TimeoutLayer::new(Duration::from_secs(30)),
            CatchPanicLayer::new(),
        ))
        .with_state(app_state)
}

async fn success(jar: CookieJar) -> Result<Html<&'static str>> {
    match jar.get("user_id") {
        Some(_) => Ok(Html(r#" <h1>Welcome!</h1> <p>You are logged in.</p> "#)),
        _ => Err(Error::Unauthorized {
            message: "You need to login to access this page.".to_string(),
        }),
    }
}

async fn github_auth(State(state): State<AppState>) -> Result<Redirect, StatusCode> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = state
        .oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    {
        let mut states = state.oauth_states.write().await;
        states.insert(
            csrf_token.secret().to_string(),
            pkce_verifier.secret().to_string(),
        );
    }

    Ok(Redirect::to(auth_url.as_str()))
}

async fn github_callback(
    Query(params): Query<AuthRequest>,
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), StatusCode> {
    let pkce_verifier = {
        let mut states = state.oauth_states.write().await;
        states.remove(&params.state)
    }
    .ok_or(StatusCode::BAD_GATEWAY)?;

    let client = reqwest::Client::builder()
        .user_agent("CAIROS/1.0.0")
        .build()
        .expect("Error on build Client.");

    let token_result = state
        .oauth_client
        .exchange_code(AuthorizationCode::new(params.code))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
        .request_async(&client)
        .await
        .map_err(|e| {
            log::error!("OAuth token exchange failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let user_response = client
        .get("https://api.github.com/user")
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await
        .map_err(|e| {
            log::error!("Error on get user: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let github_user: GitHubUser = user_response.json().await.map_err(|e| {
        log::error!("Error on jsonfy: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let email = if github_user.email.is_none() {
        let email_response = client
            .get("https://api.github.com/user/emails")
            .bearer_auth(token_result.access_token().secret())
            .send()
            .await
            .map_err(|e| {
                log::error!("Error on request github email: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let emails: Vec<serde_json::Value> = email_response.json().await.map_err(|e| {
            log::error!("Error on request deserialize email_response: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        emails
            .iter()
            .find(|email| email["primary"].as_bool().unwrap_or(false))
            .and_then(|email| email["email"].as_str())
            .map(|s| s.to_string())
    } else {
        github_user.email
    };

    let user_id = 1;
    // let user_id = match store_or_update_user(&state.db, &github_user, email.as_deref()).await {
    //     Ok(id) => id,
    //     Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    // };

    let cookie = Cookie::build(("user_id", user_id.to_string()))
        .path("/")
        .max_age(time::Duration::days(7))
        .http_only(true)
        .build();

    Ok((jar.add(cookie), Redirect::to("/")))
}

async fn logout(jar: CookieJar) -> Result<CookieJar> {
    let cookie = Cookie::build(("user_id", ""))
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build();

    Ok(jar.add(cookie))
}

async fn heartbeat(jar: CookieJar, State(_): State<AppState>) -> Result<(), StatusCode> {
    let _user_id = jar
        .get("user_id")
        .and_then(|cookie| cookie.value().parse::<i32>().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    println!("Heatbeat received");

    Ok(())
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
