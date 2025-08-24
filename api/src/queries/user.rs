use std::ops::Deref;

use super::QueryResult;
use time::OffsetDateTime;

pub struct UserId(i32);

impl Deref for UserId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct CreateParams {
    pub username: String,
    pub email: String,
    pub now: OffsetDateTime,
}

pub async fn create(db: &sqlx::PgPool, p: &CreateParams) -> QueryResult<UserId> {
    sqlx::query_scalar!(
        r#"
            INSERT INTO users (username, email, created_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (email)
            DO UPDATE SET
                username = EXCLUDED.username
            WHERE users.email = EXCLUDED.email
            RETURNING id
        "#,
        p.username,
        p.email,
        p.now,
    )
    .fetch_one(db)
    .await
    .and_then(|i| Ok(UserId(i)))
}

pub async fn find_user_id_from_token(
    db: &sqlx::PgPool,
    token: &str,
) -> QueryResult<Option<UserId>> {
    sqlx::query_scalar!(
        r#"
            SELECT users.id
            FROM auth_tokens
            INNER JOIN users ON users.id = auth_tokens.user_id
            WHERE auth_tokens.token = $1 AND auth_tokens.disabled_at IS NULL;
        "#,
        token,
    )
    .fetch_optional(db)
    .await
    .and_then(|o| Ok(o.map(|i| UserId(i))))
}
