mod config;
mod wiring;

use std::error::Error;

use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = config::Config::from_env()?;
    let listener = TcpListener::bind(config.listen_addr).await?;

    axum::serve(listener, wiring::build_app()).await?;
    Ok(())
}
