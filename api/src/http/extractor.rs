use super::Error;
use crate::{http::AppState, queries::user::UserId};

use axum::{
    RequestPartsExt,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

pub struct AuthUser {
    pub id: UserId,
}

impl AuthUser {
    async fn from_authorization(state: &AppState, token: &str) -> Result<Self, Error> {
        let Some(user_id) = crate::queries::user::find_user_id_from_token(&state.db, token).await?
        else {
            return Err(Error::Unauthorized {
                message: "Invalid token".to_owned(),
            });
        };

        Ok(Self { id: user_id })
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state: AppState = AppState::from_ref(state);

        let authorization = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| Error::Unauthorized {
                message: "Missing token".to_string(),
            })?;

        Self::from_authorization(&app_state, authorization.token()).await
    }
}
