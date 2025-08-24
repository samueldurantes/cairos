use crate::http::{AppState, Result, extractor::AuthUser};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CaptureRequest {
    uri: String,
    language: String,
    line_number: i32,
    cursor_pos: i32,
}

#[derive(Serialize)]
pub struct CaptureResponse {
    success: bool,
}

pub async fn capture(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CaptureRequest>,
) -> Result<Json<CaptureResponse>> {
    crate::queries::events::create(
        &state.db,
        &crate::queries::events::CreateParams {
            uri: payload.uri,
            language: payload.language,
            line_number: payload.line_number,
            cursor_pos: payload.cursor_pos,
            user_id: auth_user.id,
            now: time::OffsetDateTime::now_utc(),
        },
    )
    .await?;

    Ok(Json(CaptureResponse { success: true }))
}
