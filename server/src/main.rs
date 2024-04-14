use std::net::{IpAddr, SocketAddr};

use clap::Parser;
use eyre::Context;
use server::shorty::AppShorty;
use tonic::transport::Server;
use tracing::{info, Level};

#[derive(Parser, Clone, Debug)]
#[command(version, long_about = None)]
pub(crate) struct Cli {
    /// Server port
    #[arg(short, long, env = "SERVER_PORT", default_value_t = 50001, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,

    /// Server IP
    #[arg(short, long, env = "SERVER_BIND_IP", default_value = "::1")]
    ip: IpAddr,

    /// Database connection string
    #[arg(
        short,
        long,
        env = "SERVER_DATABASE",
        default_value = "redis://localhost"
    )]
    database: String,

    /// Logging level
    #[arg(short, long, env = "SERVER_LOG", default_value_t = tracing::Level::INFO)]
    log: Level,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Cli::parse();
    tracing_subscriber::fmt().with_max_level(args.log).init();

    info!("Setting up shorty service");
    let shorty = AppShorty::try_new(args.database)
        .await
        .wrap_err("Failed to initialize shorty service")?;

    let addr = SocketAddr::new(args.ip, args.port);
    info!(?addr, "Starting GRPC server");
    Server::builder()
        .add_service(shorty.grpc_service())
        .serve(addr)
        .await?;

    Ok(())
}
