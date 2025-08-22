use crate::http::AppState;

use super::{Error, Result};
use axum::{Router, response::Html, routing::get};
use axum_extra::extract::{CookieJar, cookie::Cookie};

mod github_auth;
mod github_callback;
mod heartbeat;

async fn logout_route(jar: CookieJar) -> Result<CookieJar> {
    let cookie = Cookie::build(("user_id", ""))
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build();

    Ok(jar.add(cookie))
}

async fn success_route(jar: CookieJar) -> Result<Html<&'static str>> {
    match jar.get("user_id") {
        Some(_) => Ok(Html(r#" <h1>Welcome!</h1> <p>You are logged in.</p> "#)),
        _ => Err(Error::Unauthorized {
            message: "You need to login to access this page.".to_string(),
        }),
    }
}

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(success_route))
        .route("/auth/github", get(github_auth::route))
        .route("/auth/github/callback", get(github_callback::route))
        .route("/auth/logout", get(logout_route))
        .route("/heartbeat", get(heartbeat::route))
        .with_state(app_state)
}
