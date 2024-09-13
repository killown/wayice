use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use tipsy::{Connection, Endpoint, OnConflict, ServerId};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    method: String,
    data: Value,
}

async fn process_message(message: Message) {
    println!("Processing message: {:?}", message);
    match message.method.as_str() {
        "window-info" => {
            println!("Handling window-info with data: {:?}", message.data);
        }
        _ => {
            println!("Unknown method: {}", message.method);
        }
    }
}

async fn handle_connection(mut conn: Connection) {
    let mut buf = [0; 1024];
    loop {
        match conn.read(&mut buf).await {
            Ok(0) => {
                // Connection closed
                break;
            }
            Ok(n) => {
                let message_str = String::from_utf8_lossy(&buf[..n]);
                match serde_json::from_str::<Message>(&message_str) {
                    Ok(message) => {
                        process_message(message).await;

                        // Optionally: Echo the message back to the client
                        if let Err(e) = conn.write_all(message_str.as_bytes()).await {
                            eprintln!("Failed to write to socket: {:?}", e);
                            break;
                        }
                    }
                    Err(e) => eprintln!("Failed to parse message: {:?}", e),
                }
            }
            Err(e) => {
                eprintln!("Failed to read from socket: {:?}", e);
                break;
            }
        }
    }
}

pub async fn start_ipc_server() -> Result<(), Box<dyn Error>> {
    let socket_path = "/tmp/wayice";

    // Create an endpoint for the IPC server
    let endpoint = Endpoint::new(ServerId::new(socket_path), OnConflict::Overwrite)?;

    // Start accepting incoming connections
    let mut incoming = endpoint.incoming()?;

    println!("Server is listening on {}", socket_path);

    // Handle incoming connections
    while let Some(conn) = incoming.next().await {
        match conn {
            Ok(connection) => {
                tokio::spawn(handle_connection(connection));
            }
            Err(e) => eprintln!("Error when receiving connection: {:?}", e),
        }
    }

    Ok(())
}
