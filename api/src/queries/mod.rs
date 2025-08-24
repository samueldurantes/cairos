pub mod auth_tokens;
pub mod events;
pub mod user;

pub(super) type QueryResult<T> = Result<T, sqlx::Error>;
