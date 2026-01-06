//! TCP client example
//!
//! This example demonstrates TCP socket communication using Termicon.
//!
//! Usage:
//!   cargo run --example tcp_client -- localhost 23

use termicon_core::{Session, SessionEvent, TcpConfig, Transport};
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    let (host, port) = match args.len() {
        3 => (args[1].clone(), args[2].parse().unwrap_or(23)),
        2 => (args[1].clone(), 23),
        _ => {
            println!("Usage: tcp_client <host> [port]");
            println!("Example: tcp_client localhost 23");
            return Ok(());
        }
    };

    println!("Connecting to {}:{}...", host, port);

    // Create TCP configuration
    let config = TcpConfig::new(&host, port);

    // Connect
    let session = Session::connect(Transport::Tcp(config)).await?;
    println!("Connected! Type to send, Ctrl+C to exit.\n");

    // Subscribe to events
    let mut rx = session.subscribe();

    // Spawn receiver task
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                SessionEvent::DataReceived(data) => {
                    print!("{}", String::from_utf8_lossy(&data));
                }
                SessionEvent::StateChanged(state) => {
                    println!("\n[State: {:?}]", state);
                }
                SessionEvent::Error(e) => {
                    eprintln!("\n[Error: {}]", e);
                }
                _ => {}
            }
        }
    });

    // Read from stdin and send
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    while let Some(line) = reader.next_line().await? {
        let mut data = line.into_bytes();
        data.push(b'\r');
        data.push(b'\n');
        session.send(&data).await?;
    }

    session.disconnect().await?;
    Ok(())
}






