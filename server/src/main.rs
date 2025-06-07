use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;

mod protocol;
use protocol::{RcCommand, RcResponse};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("WebSocket server listening on: {}", addr);

    let (tx, _) = broadcast::channel::<String>(16);

    while let Ok((stream, addr)) = listener.accept().await {
        let tx = tx.clone();
        tokio::spawn(handle_connection(stream, addr, tx));
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr, tx: broadcast::Sender<String>) {
    info!("New WebSocket connection from: {}", addr);

    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws_stream) => ws_stream,
        Err(e) => {
            error!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();
    let mut rx = tx.subscribe();

    loop {
        tokio::select! {
            Some(message) = read.next() => {
                match message {
                    Ok(msg) => {
                        if let Message::Text(text) = msg {
                            match serde_json::from_str::<RcCommand>(&text) {
                                Ok(command) => {
                                    info!("Received command: {:?}", command);
                                    if let Err(e) = tx.send(text) {
                                        error!("Error broadcasting message: {}", e);
                                    }

                                    let response = RcResponse::Status {
                                        battery_level: 100,
                                        connected: true,
                                        current_speed: match &command {
                                            RcCommand::Move(cmd) => cmd.speed,
                                            _ => 0,
                                        },
                                        current_direction: match &command {
                                            RcCommand::Move(cmd) => cmd.direction.clone(),
                                            _ => protocol::Direction::Stop,
                                        },
                                    };

                                    if let Ok(response_text) = serde_json::to_string(&response) {
                                        if let Err(e) = write.send(Message::Text(response_text)).await {
                                            error!("Error sending response: {}", e);
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Error parsing command: {}", e);
                                    let error_response = RcResponse::Error(format!("Invalid command: {}", e));
                                    if let Ok(response_text) = serde_json::to_string(&error_response) {
                                        if let Err(e) = write.send(Message::Text(response_text)).await {
                                            error!("Error sending error response: {}", e);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error receiving message: {}", e);
                        break;
                    }
                }
            }
            Ok(msg) = rx.recv() => {
                if let Err(e) = write.send(Message::Text(msg)).await {
                    error!("Error forwarding message: {}", e);
                    break;
                }
            }
        }
    }

    info!("Connection closed for: {}", addr);
}
