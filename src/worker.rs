use elytra_ping::{
    protocol::{Frame, ProtocolError},
    SlpProtocol,
};
use snafu::{Backtrace, Snafu};
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::debug;

#[derive(Snafu, Debug)]
pub enum WorkerError {
    #[snafu(display("connection dropped"))]
    ConnectionDropped { backtrace: Backtrace },
    #[snafu(context(false))]
    Protocol {
        source: ProtocolError,
        backtrace: Backtrace,
    },
}

async fn read_frame(protocol: &mut SlpProtocol) -> Result<Frame, WorkerError> {
    let frame = protocol
        .read_frame()
        .await?
        .ok_or_else(|| ConnectionDroppedSnafu.build())?;

    Ok(frame)
}

/// Handle a single client connection
pub async fn run_worker(
    socket: TcpStream,
    server_info: Arc<String>,
    server_addr: (String, u16),
) -> Result<(), WorkerError> {
    let mut protocol = SlpProtocol::new(server_addr.0, server_addr.1, socket);

    debug!("New connection, ready for handshake");

    // read handshake from the client
    let frame = read_frame(&mut protocol).await?;

    Ok(())
}
