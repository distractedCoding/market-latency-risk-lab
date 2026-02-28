use std::{env, net::SocketAddr};

const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:8080";

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub listen_addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> Result<Self, std::net::AddrParseError> {
        let listen_addr = env::var("LAB_SERVER_ADDR")
            .unwrap_or_else(|_| DEFAULT_LISTEN_ADDR.to_string())
            .parse()?;

        Ok(Self { listen_addr })
    }
}
