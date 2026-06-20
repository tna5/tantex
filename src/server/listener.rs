use std::sync::Arc;

use tokio::net::UnixListener;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::engine::index_manager::IndexManager;

/// True when the connection error is the client side disconnecting mid-write
/// (e.g. dashboard polling tab closed, bench script exited). Not actionable —
/// log at debug, not error. The connection error type is a Box<dyn Error> so
/// we match on the displayed message rather than downcasting.
fn is_expected_disconnect(msg: &str) -> bool {
    msg.contains("Broken pipe")
        || msg.contains("Connection reset")
        || msg.contains("Connection aborted")
        || msg.contains("unexpected end of file")
        || msg.contains("UnexpectedEof")
}

pub async fn start_server(
    config: Arc<RwLock<Config>>,
    index_manager: Arc<RwLock<IndexManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = {
        let cfg = config.read().await;
        cfg.socket_path.clone()
    };

    // Remove old socket file if exists
    let _ = std::fs::remove_file(&socket_path);

    let listener = UnixListener::bind(&socket_path)?;
    log::info!("Listening on {}", socket_path);

    loop {
        let (stream, _addr) = listener.accept().await?;
        let manager = index_manager.clone();
        let cfg = Arc::clone(&config);
        tokio::spawn(async move {
            if let Err(e) = super::connection::handle_connection(stream, manager, cfg).await {
                let msg = e.to_string();
                if is_expected_disconnect(&msg) {
                    log::debug!("Client disconnected: {}", msg);
                } else {
                    log::error!("Connection error: {}", msg);
                }
            }
        });
    }
}
