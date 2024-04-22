use clap::{command, Parser};
use tracing::Level;
use url::Url;

#[derive(Parser, Clone, Debug)]
#[command(version, long_about = None)]
pub struct Cli {
    /// Base public URL
    #[arg(short ,long, env = "CLIENT_PUBLIC_URL", default_value = "http://[::1]", value_parser = clap::value_parser!(url::Url))]
    pub url: Url,

    // gRPC server endpoint
    #[arg(
        short,
        long,
        env = "CLIENT_GRPC_ENDPOINT",
        default_value = "http://[::1]:50001"
    )]
    pub grpc: String,

    // gRPC server security token
    #[arg(short, long, env = "CLIENT_GRPC_TOKEN")]
    pub token: Option<String>,

    /// Logging level
    #[arg(short, long, env = "CLIENT_LOG", default_value_t = tracing::Level::INFO)]
    pub log: Level,
}
