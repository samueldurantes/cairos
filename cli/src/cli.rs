use anyhow::Context;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Auth(AuthArgs),
    #[command(arg_required_else_help = true)]
    Setup {
        #[arg(long)]
        base_url: String,
    },
    LanguageServer,
}

#[derive(Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommands,
}

#[derive(Subcommand)]
pub enum AuthCommands {
    #[command(arg_required_else_help = true)]
    Login {
        #[arg(long)]
        github: bool,
    },
    Logout,
}

impl Cli {
    pub async fn run(self, ctx: &crate::Ctx) -> anyhow::Result<()> {
        match self.command {
            Commands::Auth(auth) => match auth.command {
                AuthCommands::Login { github } => {
                    if github {
                        crate::commands::auth::github_login(&ctx.reqwest, &ctx.config.base_url)
                            .await?
                    }
                }
                AuthCommands::Logout => {}
            },
            Commands::Setup { base_url } => crate::commands::config::setup(base_url)?,
            Commands::LanguageServer => {
                crate::commands::language_server::run(
                    ctx.reqwest.clone(),
                    &ctx.config.base_url,
                    &ctx.config
                        .token
                        .as_ref()
                        .context("you are not authenticated")?,
                )
                .await
            }
        }

        Ok(())
    }
}
