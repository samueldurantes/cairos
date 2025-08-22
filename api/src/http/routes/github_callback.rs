use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use oauth2::{AuthorizationCode, PkceCodeVerifier, TokenResponse};

use crate::http::{AppState, AuthRequest, GitHubUser};

pub async fn route(
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
