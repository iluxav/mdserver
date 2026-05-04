use std::sync::Arc;

use anyhow::Result;
use clap::Parser;

mod cache;
mod config;
mod index;
mod introspect;
mod markdown;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::Config::parse();
    let root = cfg.root.canonicalize()?;
    if !root.is_dir() {
        anyhow::bail!("root is not a directory: {}", root.display());
    }

    let state = server::AppState {
        root: root.clone(),
        cache: Arc::new(cache::Cache::new()),
    };

    let listener = tokio::net::TcpListener::bind(cfg.bind).await?;
    eprintln!("mdserver listening on http://{} (root: {})", cfg.bind, root.display());
    axum::serve(listener, server::router(state)).await?;
    Ok(())
}
