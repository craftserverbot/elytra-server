mod server_info;
mod worker;

use std::sync::Arc;

use clap::Parser;
use elytra_ping::SlpProtocol;
use server_info::resolve_server_info;
use tokio::net::TcpListener;
use tracing::error;
use tracing_subscriber::EnvFilter;

/// Run a fake Minecraft server that appears online in server lists
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to listen for requests on
    #[arg(short, long, default_value_t = 25565)]
    port: u16,

    /// JSON file of custom server info to send to clients
    #[arg(short, long)]
    info_file: Option<String>,

    /// Expose the server to the world outside of localhost
    #[arg(short, long, default_value_t = false)]
    expose: bool,
}

#[tokio::main]
async fn main() {
    match app().await {
        Ok(_) => {}
        Err(error) => {
            error!(error);
            std::process::exit(1);
        }
    }
}

async fn app() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let server_info = Arc::new(resolve_server_info(args.info_file).await?);

    let server =
        TcpListener::bind((if args.expose { "0.0.0.0" } else { "127.0.0.1" }, args.port)).await?;

    tracing::info!("Listening on {}", server.local_addr()?);

    loop {
        let (socket, _) = server.accept().await?;
        let server_info = server_info.clone();
        let server_addr = server.local_addr()?;

        tokio::spawn(async move {
            let mut protocol =
                SlpProtocol::new(server_addr.ip().to_string(), server_addr.port(), socket);

            let result = worker::run_worker(&mut protocol, &server_info).await;
            if let Err(error) = result {
                error!("Failed to respond to ping: {}", error);
            }

            _ = protocol.disconnect().await;
        });
    }
}
