use crate::config::Config;
use anyhow::Context;
use axum::{Router, routing::post};
use error::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};
use time::OffsetDateTime;
use tokio::net::TcpListener;
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};

mod auth;
mod error;
mod events;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    client: Client,
}

#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub email: Option<String>,
}

pub async fn serve(config: Config, db: PgPool) -> anyhow::Result<()> {
    let app_state = AppState {
        config: config.clone(),
        client: Client::builder()
            .user_agent("CAIROS/1.0.0")
            .build()
            .expect("Error on build Client."),
        db,
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
        .route("/events/capture", post(events::capture))
        .route("/login", post(auth::login))
        .layer((
            CompressionLayer::new(),
            TraceLayer::new_for_http().on_failure(()),
            TimeoutLayer::new(Duration::from_secs(30)),
            CatchPanicLayer::new(),
        ))
        .with_state(app_state)
}

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
