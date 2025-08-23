use clap::Parser;

mod cli;
mod clients;
mod commands;

pub struct Ctx {
    pub reqwest: reqwest::Client,
}

impl Ctx {
    fn new() -> Self {
        Self {
            reqwest: reqwest::Client::new(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let email = None;

    let email = if let Some(email) = email {
        email
    } else {
        "default@email.com".to_owned()
    };

    println!("{email:?}");

    anyhow::Ok(())

    // let ctx = Ctx::new();
    // let cli = cli::Cli::parse();

    // cli.run(&ctx).await
}
