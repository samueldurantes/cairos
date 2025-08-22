#[derive(clap::Parser, Clone)]
pub struct Config {
    #[clap(long, env)]
    pub database_url: String,

    #[clap(short, env, default_value = "50")]
    pub database_max_connections: u32,

    #[clap(short, env, default_value = "3000")]
    pub port: u16,

    #[clap(long, env)]
    pub github_client_id: String,

    #[clap(long, env)]
    pub github_client_secret: String,
}
