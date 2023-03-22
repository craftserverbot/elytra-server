use elytra_ping::{
    protocol::{Frame, ProtocolError, ProtocolState, ServerState},
    SlpProtocol,
};
use snafu::{Backtrace, Snafu};
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

async fn read_frame(protocol: &mut SlpProtocol, state: ServerState) -> Result<Frame, WorkerError> {
    let frame = protocol
        .read_frame(Some(state))
        .await?
        .ok_or_else(|| ConnectionDroppedSnafu.build())?;

    Ok(frame)
}

/// Handle a single client connection
pub async fn run_worker(protocol: &mut SlpProtocol, server_info: &str) -> Result<(), WorkerError> {
    let mut state = ServerState::Handshake;

    debug!("New connection, ready for handshake");

    loop {
        let frame = read_frame(protocol, state).await?;
        debug!("Recived frame: {:?}", frame);

        match frame {
            Frame::Handshake {
                state: new_state, ..
            } => {
                let new_state: i32 = new_state.into();
                if new_state != ProtocolState::Status as i32 {
                    debug!("Client attempted to log in to the server - closing connection");
                    break;
                }

                state = ServerState::Status;

                debug!("Recived handshake, ready for status request");
            }
            Frame::StatusRequest => {
                debug!("Recived status request, sending server info");
                let response = Frame::StatusResponse {
                    json: server_info.to_string(),
                };
                protocol.write_frame(response).await?;
            }
            Frame::PingRequest { payload } => {
                debug!("Recived ping, sending pong");
                let response = Frame::PingResponse { payload };
                protocol.write_frame(response).await?;
            }
            _ => {
                debug!("Recived unexpected frame, closing connection");
                break;
            }
        }
    }

    Ok(())
}
