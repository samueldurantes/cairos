use axum::response::Html;
use axum::{extract::State, http::StatusCode, response::Redirect};
use oauth2::{CsrfToken, PkceCodeChallenge, Scope};

use crate::http::{AppState, Error, Result};

use crate::http::{AuthRequest, GitHubUser};
use axum::extract::Query;
use axum_extra::extract::cookie::{Cookie, CookieJar};
use oauth2::{AuthorizationCode, PkceCodeVerifier, TokenResponse};

pub async fn github(State(state): State<AppState>) -> Result<Redirect, StatusCode> {
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

pub async fn github_callback(
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

pub async fn logout(jar: CookieJar) -> Result<CookieJar> {
    let cookie = Cookie::build(("user_id", ""))
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build();

    Ok(jar.add(cookie))
}

pub async fn success(jar: CookieJar) -> Result<Html<&'static str>> {
    match jar.get("user_id") {
        Some(_) => Ok(Html(r#" <h1>Welcome!</h1> <p>You are logged in.</p> "#)),
        _ => Err(Error::Unauthorized {
            message: "You need to login to access this page.".to_string(),
        }),
    }
}
