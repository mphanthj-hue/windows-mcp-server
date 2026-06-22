use anyhow::Result;
use windows_mcp_server::{Cli, Config, observability, server};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_args();
    let config = Config::from(cli);
    observability::init(config.log_format)?;
    server::run(config).await
}
