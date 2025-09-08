use time::OffsetDateTime;

use crate::queries::{QueryResult, user::UserId};

pub async fn create(
    db: &sqlx::PgPool,
    user_id: UserId,
    access_token: &str,
    now: OffsetDateTime,
) -> QueryResult<()> {
    sqlx::query!(
        r#"
            INSERT INTO auth_tokens (user_id, token, created_at) VALUES ($1, $2, $3)
        "#,
        *user_id,
        access_token,
        now,
    )
    .execute(db)
    .await
    .and_then(|_| Ok(()))
}
