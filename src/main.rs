use anyhow::Result;
use clap::Parser;
use env_logger::{Builder, Env, Target};
use log::info;
use rmcp::transport::io::stdio;
use rmcp::ServiceExt;

mod auth;
mod models;
mod services;

use services::applemusic::AppleMusicServer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Apple Developer Team ID
    #[arg(long, help = "Apple Developer Team ID")]
    team_id: Option<String>,

    /// Apple Music Key ID
    #[arg(long, help = "Apple Music Key ID")]
    key_id: Option<String>,

    /// Path to the Apple Music private key file (.p8)
    #[arg(long, help = "Path to the Apple Music private key file (.p8)")]
    private_key_path: Option<String>,

    /// Storefront for Apple Music (e.g. us, jp)
    #[arg(
        long,
        help = "Storefront for Apple Music (e.g. us, jp)",
        default_value = "jp"
    )]
    storefront: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    setup_logger(&args);

    info!("Starting Apple Music MCP Server...");

    let service = AppleMusicServer::new(
        args.team_id.clone(),
        args.key_id.clone(),
        args.private_key_path.clone(),
        args.storefront.clone(),
    )
    .serve(stdio())
    .await?;
    service.waiting().await?;

    info!("Apple Music MCP Server has been terminated");

    Ok(())
}

fn setup_logger(_args: &Args) {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Seconds))
        .format_module_path(true)
        .target(Target::Stderr)
        .init();
}
