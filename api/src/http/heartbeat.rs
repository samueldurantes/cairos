use axum::{extract::State, http::StatusCode};
use axum_extra::extract::cookie::CookieJar;

use crate::http::AppState;

pub async fn heartbeat(jar: CookieJar, State(_): State<AppState>) -> Result<(), StatusCode> {
    let _user_id = jar
        .get("user_id")
        .and_then(|cookie| cookie.value().parse::<i32>().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    println!("Heatbeat received");

    Ok(())
}
