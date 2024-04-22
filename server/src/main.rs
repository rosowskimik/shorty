use std::net::{IpAddr, SocketAddr};

use clap::Parser;
use eyre::Context;
use server::{
    intercept::TokenInterceptor,
    shorty::{grpc::shorty_server::ShortyServer, AppShorty},
};
use tonic::transport::Server;
use tracing::{debug, info, Level};
use tracing_subscriber::{filter::Targets, prelude::*};

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

    /// Server token
    #[arg(short, long, env = "SERVER_TOKEN")]
    token: Option<String>,

    /// Logging level
    #[arg(short, long, env = "SERVER_LOG", default_value_t = tracing::Level::INFO)]
    log: Level,
}

fn setup_tracing(lvl: &Level) {
    let filter = Targets::new()
        // For tracing from app use requested level
        .with_target("server", *lvl)
        // For all other, use >error
        .with_default(Level::ERROR);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Cli::parse();
    setup_tracing(&args.log);

    info!("Setting up shorty service");
    let shorty = AppShorty::try_new(args.database)
        .await
        .wrap_err("Failed to initialize shorty service")?;

    let mut builder = Server::builder();
    let builder = if let Some(token) = args.token {
        debug!("Enabling secure token");
        builder.add_service(ShortyServer::with_interceptor(
            shorty,
            TokenInterceptor {
                token: token.clone(),
            },
        ))
    } else {
        builder.add_service(ShortyServer::new(shorty))
    };

    let addr = SocketAddr::new(args.ip, args.port);
    info!(?addr, "Starting GRPC server");
    builder.serve(addr).await?;

    Ok(())
}
