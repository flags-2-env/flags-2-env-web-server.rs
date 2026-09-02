#![forbid(unsafe_code)]

use flags_2_env_web_server::{config::WebConfig, flags, server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let environment = flags::resolve().map_err(std::io::Error::other)?;
    let config = WebConfig::from_map(&environment);
    server::run(&config).await
}
