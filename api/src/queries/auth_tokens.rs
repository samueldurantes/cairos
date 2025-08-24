use crate::queries::{QueryResult, user::UserId};

pub async fn create(db: &sqlx::PgPool, user_id: UserId, access_token: &str) -> QueryResult<()> {
    sqlx::query!(
        r#"
            INSERT INTO auth_tokens (user_id, token) VALUES ($1, $2)
        "#,
        *user_id,
        access_token,
    )
    .execute(db)
    .await
    .and_then(|_| Ok(()))
}
