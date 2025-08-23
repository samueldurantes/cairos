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
                        crate::commands::auth::github_login(&ctx.reqwest).await?
                    }
                }
                AuthCommands::Logout => {}
            },
            Commands::LanguageServer => crate::commands::language_server::run(String::new()).await,
        }

        Ok(())
    }
}
