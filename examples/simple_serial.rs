//! Simple serial port example
//!
//! This example demonstrates basic serial port communication using Termicon.
//!
//! Usage:
//!   cargo run --example simple_serial -- COM3 115200

use termicon_core::{SerialConfig, Session, SessionEvent, Transport};
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    let (port, baud_rate) = match args.len() {
        3 => (args[1].clone(), args[2].parse().unwrap_or(115200)),
        2 => (args[1].clone(), 115200),
        _ => {
            // List available ports
            println!("Usage: simple_serial <port> [baud_rate]");
            println!("\nAvailable ports:");
            for port in serialport::available_ports()? {
                println!("  {}", port.port_name);
            }
            return Ok(());
        }
    };

    println!("Connecting to {} at {} baud...", port, baud_rate);

    // Create serial configuration
    let config = SerialConfig::new(&port, baud_rate);

    // Connect
    let session = Session::connect(Transport::Serial(config)).await?;
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






