use axum::{extract::State, http::StatusCode, response::Redirect};
use oauth2::{CsrfToken, PkceCodeChallenge, Scope};

use crate::http::AppState;

pub async fn route(State(state): State<AppState>) -> Result<Redirect, StatusCode> {
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
