use crate::queries::{QueryResult, user::UserId};
use time::OffsetDateTime;

pub struct CreateParams {
    pub uri: String,
    pub is_write: bool,
    pub language: Option<String>,
    pub line_number: Option<i32>,
    pub cursor_pos: Option<i32>,
    pub user_id: UserId,
    pub now: OffsetDateTime,
}

pub async fn create(db: &sqlx::PgPool, p: &CreateParams) -> QueryResult<()> {
    sqlx::query!(
        r#"
            INSERT INTO events (uri, is_write, language, line_number, cursor_pos, user_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        p.uri,
        p.is_write,
        p.language,
        p.line_number,
        p.cursor_pos,
        *p.user_id,
        p.now,
    )
    .execute(db)
    .await
    .and_then(|_| Ok(()))
}
