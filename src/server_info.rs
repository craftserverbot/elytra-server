use const_format::concatcp;
use elytra_ping::protocol::Frame;
use tokio::fs;
use tracing::debug;

const DEFAULT_INFO: &str = concatcp!(
    r#"{
    "description": {
        "text": "Try Elytra Ping!"
    },
    "players": {
        "max": 0,
        "online": 0,
        "sample": []
    },
    "version": {
        "name": "elytra-server v"#,
    env!("CARGO_PKG_VERSION"),
    r#"",
        "protocol": "#,
    Frame::PROTOCOL_VERSION,
    r#"
    },
    "favicon": ""#,
    include_str!("./favicon.txt"),
    r#""
}"#
);

pub async fn resolve_server_info(path: Option<String>) -> std::io::Result<String> {
    match path {
        Some(path) => fs::read_to_string(path).await,
        None => {
            debug!("Falling back to default server info");
            Ok(DEFAULT_INFO.to_string())
        }
    }
}
