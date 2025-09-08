use clap::Parser;

mod cli;
mod clients;
mod commands;
mod config;

pub struct Ctx {
    pub reqwest: reqwest::Client,
    pub config: config::Config,
}

impl Ctx {
    fn new() -> Self {
        Self {
            reqwest: reqwest::Client::builder()
                .user_agent("cairos-cli")
                .build()
                .expect("Error when trying to build HTTP client"),
            config: config::Config::load(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ctx = Ctx::new();
    let cli = cli::Cli::parse();

    cli.run(&ctx).await
}
