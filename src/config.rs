use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "mdserver", about = "Markdown filesystem HTTP server")]
pub struct Config {
    /// Root directory to serve
    #[arg(long, env = "MDSERVER_ROOT")]
    pub root: PathBuf,

    /// Address to bind, e.g. 127.0.0.1:8080
    #[arg(long, env = "MDSERVER_BIND", default_value = "127.0.0.1:8080")]
    pub bind: SocketAddr,
}
