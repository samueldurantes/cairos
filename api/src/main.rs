use anyhow::Context;
use clap::Parser;
use sqlx::postgres::PgPoolOptions;

mod config;
mod http;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_filename(".env").ok();
    dotenvy::from_filename("api/.env").ok();
    env_logger::init();

    let config = config::Config::parse();
    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.database_url)
        .await
        .context("Error when trying to connect to database")?;

    sqlx::migrate!().run(&db).await?;
    http::serve(config, db).await?;

    Ok(())
}
