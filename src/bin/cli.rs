//! Termicon CLI - Command-line interface
//!
//! Provides feature parity with the GUI for automation and headless operation.

use clap::{Parser, Subcommand, ValueEnum};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::time::Duration;

/// CLI output format
#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// Human-readable text
    Text,
    /// JSON format for scripting
    Json,
    /// CSV format
    Csv,
    /// Hex dump
    Hex,
}

/// Line ending style
#[derive(Debug, Clone, Copy, ValueEnum)]
enum LineEnding {
    /// CR+LF (Windows)
    Crlf,
    /// LF only (Unix)
    Lf,
    /// CR only (old Mac)
    Cr,
    /// No line ending
    None,
}

impl LineEnding {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Crlf => b"\r\n",
            Self::Lf => b"\n",
            Self::Cr => b"\r",
            Self::None => b"",
        }
    }
}

/// Termicon CLI
#[derive(Parser, Debug)]
#[command(
    name = "termicon",
    author = "Termicon Team",
    version = "0.1.0",
    about = "Professional Multi-Protocol Terminal Application",
    long_about = None
)]
struct Cli {
    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Quiet mode (errors only)
    #[arg(short, long)]
    quiet: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List available serial ports
    ListPorts {
        /// Show detailed info
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Connect to a serial port
    Serial {
        /// Serial port name (e.g., COM3, /dev/ttyUSB0)
        #[arg(short, long)]
        port: String,
        
        /// Baud rate
        #[arg(short, long, default_value = "115200")]
        baud: u32,
        
        /// Data bits (5-8)
        #[arg(long, default_value = "8")]
        data_bits: u8,
        
        /// Parity (none, odd, even)
        #[arg(long, default_value = "none")]
        parity: String,
        
        /// Stop bits (1, 2)
        #[arg(long, default_value = "1")]
        stop_bits: u8,
        
        /// Flow control (none, hw, sw)
        #[arg(long, default_value = "none")]
        flow: String,
        
        /// Line ending
        #[arg(long, value_enum, default_value_t = LineEnding::Crlf)]
        line_ending: LineEnding,
        
        /// Exit after timeout (seconds)
        #[arg(long)]
        timeout: Option<u64>,
        
        /// Send command and exit
        #[arg(short = 'c', long)]
        command: Option<String>,
        
        /// Log file path
        #[arg(short = 'l', long)]
        log: Option<PathBuf>,
        
        /// Echo locally
        #[arg(long)]
        echo: bool,
        
        /// Record session
        #[arg(long)]
        record: Option<PathBuf>,
    },
    
    /// Connect to a TCP host
    Tcp {
        /// Host address
        #[arg(short = 'H', long)]
        host: String,
        
        /// Port number
        #[arg(short, long, default_value = "23")]
        port: u16,
        
        /// Connection timeout (seconds)
        #[arg(long, default_value = "10")]
        timeout: u64,
        
        /// Send command and exit
        #[arg(short = 'c', long)]
        command: Option<String>,
        
        /// Log file path
        #[arg(short = 'l', long)]
        log: Option<PathBuf>,
    },
    
    /// Connect to a Telnet host
    Telnet {
        /// Host address
        #[arg(short = 'H', long)]
        host: String,
        
        /// Port number
        #[arg(short, long, default_value = "23")]
        port: u16,
        
        /// Send command and exit
        #[arg(short = 'c', long)]
        command: Option<String>,
    },
    
    /// Connect to an SSH host
    Ssh {
        /// Host address
        #[arg(short = 'H', long)]
        host: String,
        
        /// Port number
        #[arg(short, long, default_value = "22")]
        port: u16,
        
        /// Username
        #[arg(short, long)]
        user: String,
        
        /// Password (use SSH_PASSWORD env or prompt if not provided)
        #[arg(short = 'P', long)]
        password: Option<String>,
        
        /// Private key file
        #[arg(short, long)]
        identity: Option<PathBuf>,
        
        /// Execute command and exit
        #[arg(short = 'c', long)]
        command: Option<String>,
        
        /// Disable PTY allocation
        #[arg(long)]
        no_pty: bool,
    },
    
    /// Scan for Bluetooth devices
    BleScan {
        /// Scan duration (seconds)
        #[arg(short, long, default_value = "5")]
        duration: u64,
        
        /// Filter by name (regex)
        #[arg(short, long)]
        filter: Option<String>,
    },
    
    /// Run as bridge (Serial ↔ TCP)
    Bridge {
        /// Serial port
        #[arg(long)]
        serial_port: String,
        
        /// Serial baud rate
        #[arg(long, default_value = "115200")]
        baud: u32,
        
        /// TCP port to listen on
        #[arg(long, default_value = "8023")]
        tcp_port: u16,
        
        /// Run as daemon
        #[arg(short, long)]
        daemon: bool,
    },
    
    /// Send hex data
    SendHex {
        /// Connection type (serial, tcp)
        #[arg(short = 't', long)]
        conn_type: String,
        
        /// Target (port name or host:port)
        #[arg(short = 'T', long)]
        target: String,
        
        /// Hex data to send
        data: String,
        
        /// Wait for response
        #[arg(short, long)]
        wait: bool,
        
        /// Response timeout (ms)
        #[arg(long, default_value = "1000")]
        timeout: u64,
    },
    
    /// Replay a recorded session
    Replay {
        /// Recording file path
        file: PathBuf,
        
        /// Playback speed multiplier
        #[arg(short, long, default_value = "1.0")]
        speed: f32,
        
        /// Target (optional, otherwise simulates)
        #[arg(short, long)]
        target: Option<String>,
    },
    
    /// Protocol decode
    Decode {
        /// Protocol definition file (YAML/JSON)
        #[arg(short, long)]
        protocol: PathBuf,
        
        /// Hex data to decode
        data: String,
        
        /// Message type (optional)
        #[arg(short, long)]
        message: Option<String>,
    },
    
    /// Profile management
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    
    /// Monitor mode (continuous output)
    Monitor {
        /// Connection type
        #[arg(short = 't', long)]
        conn_type: String,
        
        /// Target
        #[arg(short = 'T', long)]
        target: String,
        
        /// Show timestamps
        #[arg(long)]
        timestamps: bool,
        
        /// Show hex
        #[arg(long)]
        hex: bool,
    },
    
    /// Check system information
    Info,
}

#[derive(Subcommand, Debug)]
enum ProfileAction {
    /// List all profiles
    List,
    /// Show profile details
    Show { name: String },
    /// Connect using a profile
    Connect { name: String },
    /// Export profile to file
    Export { name: String, file: PathBuf },
    /// Import profile from file
    Import { file: PathBuf },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize i18n
    rust_i18n::set_locale("en");
    
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::ListPorts { detailed } => {
            list_ports(&cli, *detailed)?;
        }
        Commands::Serial { port, baud, command, timeout, echo, line_ending, .. } => {
            connect_serial(&cli, port, *baud, command.as_deref(), timeout.as_ref(), *echo, *line_ending).await?;
        }
        Commands::Tcp { host, port, command, timeout, .. } => {
            connect_tcp(&cli, host, *port, command.as_deref(), *timeout).await?;
        }
        Commands::Ssh { host, port, user, password, identity, command, no_pty } => {
            connect_ssh(&cli, host, *port, user, password.as_deref(), identity.as_ref(), command.as_deref(), *no_pty).await?;
        }
        Commands::BleScan { duration, filter } => {
            ble_scan(&cli, *duration, filter.as_deref()).await?;
        }
        Commands::Bridge { serial_port, baud, tcp_port, daemon } => {
            run_bridge(&cli, serial_port, *baud, *tcp_port, *daemon).await?;
        }
        Commands::SendHex { conn_type, target, data, wait, timeout } => {
            send_hex(&cli, conn_type, target, data, *wait, *timeout).await?;
        }
        Commands::Info => {
            show_info(&cli)?;
        }
        Commands::Profile { action } => {
            handle_profile(&cli, action)?;
        }
        _ => {
            eprintln!("Command not yet implemented");
        }
    }
    
    Ok(())
}

fn list_ports(cli: &Cli, detailed: bool) -> anyhow::Result<()> {
    let ports = serialport::available_ports()?;
    
    if ports.is_empty() {
        if !cli.quiet {
            println!("No serial ports found.");
        }
        return Ok(());
    }
    
    match cli.format {
        OutputFormat::Json => {
            let json: Vec<serde_json::Value> = ports.iter().map(|p| {
                serde_json::json!({
                    "name": p.port_name,
                    "type": format!("{:?}", p.port_type)
                })
            }).collect();
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputFormat::Csv => {
            println!("name,type");
            for port in &ports {
                println!("{},{:?}", port.port_name, port.port_type);
            }
        }
        _ => {
            if detailed {
                println!("Available Serial Ports:");
                println!("{:-<60}", "");
                for port in &ports {
                    println!("  {} [{:?}]", port.port_name, port.port_type);
                }
            } else {
                for port in &ports {
                    println!("{}", port.port_name);
                }
            }
        }
    }
    
    Ok(())
}

async fn connect_serial(
    cli: &Cli,
    port: &str,
    baud: u32,
    command: Option<&str>,
    timeout: Option<&u64>,
    echo: bool,
    line_ending: LineEnding,
) -> anyhow::Result<()> {
    use std::io::ErrorKind;
    
    if !cli.quiet {
        eprintln!("Connecting to {} @ {} baud...", port, baud);
    }
    
    let mut serial = serialport::new(port, baud)
        .timeout(Duration::from_millis(100))
        .open()?;
    
    if !cli.quiet {
        eprintln!("Connected. Press Ctrl+C to exit.");
    }
    
    // If command mode, send and exit
    if let Some(cmd) = command {
        let mut data = cmd.as_bytes().to_vec();
        data.extend_from_slice(line_ending.as_bytes());
        serial.write_all(&data)?;
        
        // Wait for response
        std::thread::sleep(Duration::from_millis(500));
        
        let mut buf = vec![0u8; 4096];
        loop {
            match serial.read(&mut buf) {
                Ok(n) if n > 0 => {
                    output_data(cli, &buf[..n]);
                }
                Ok(_) => break,
                Err(e) if e.kind() == ErrorKind::TimedOut => break,
                Err(e) => return Err(e.into()),
            }
        }
        
        return Ok(());
    }
    
    // Interactive mode
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })?;
    
    let timeout_instant = timeout.map(|t| std::time::Instant::now() + Duration::from_secs(*t));
    
    // Spawn reader thread
    let serial_clone = serial.try_clone()?;
    let running_clone = running.clone();
    let format = cli.format;
    let quiet = cli.quiet;
    
    std::thread::spawn(move || {
        let mut serial = serial_clone;
        let mut buf = vec![0u8; 1024];
        
        while running_clone.load(std::sync::atomic::Ordering::SeqCst) {
            match serial.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let cli_ref = Cli {
                        format,
                        verbose: false,
                        quiet,
                        command: Commands::Info,
                    };
                    output_data(&cli_ref, &buf[..n]);
                }
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::TimedOut => {}
                Err(_) => break,
            }
        }
    });
    
    // Read from stdin
    let stdin = io::stdin();
    let mut input = String::new();
    
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        if let Some(timeout) = timeout_instant {
            if std::time::Instant::now() > timeout {
                break;
            }
        }
        
        input.clear();
        if stdin.read_line(&mut input).is_ok() && !input.is_empty() {
            let data = input.trim_end();
            if echo && !cli.quiet {
                eprintln!("> {}", data);
            }
            
            let mut bytes = data.as_bytes().to_vec();
            bytes.extend_from_slice(line_ending.as_bytes());
            serial.write_all(&bytes)?;
        }
    }
    
    if !cli.quiet {
        eprintln!("Disconnected.");
    }
    
    Ok(())
}

async fn connect_tcp(
    cli: &Cli,
    host: &str,
    port: u16,
    command: Option<&str>,
    timeout: u64,
) -> anyhow::Result<()> {
    use tokio::net::TcpStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    
    if !cli.quiet {
        eprintln!("Connecting to {}:{}...", host, port);
    }
    
    let addr = format!("{}:{}", host, port);
    let mut stream = tokio::time::timeout(
        Duration::from_secs(timeout),
        TcpStream::connect(&addr)
    ).await??;
    
    if !cli.quiet {
        eprintln!("Connected. Press Ctrl+C to exit.");
    }
    
    if let Some(cmd) = command {
        stream.write_all(cmd.as_bytes()).await?;
        stream.write_all(b"\r\n").await?;
        
        let mut buf = vec![0u8; 4096];
        match tokio::time::timeout(Duration::from_secs(2), stream.read(&mut buf)).await {
            Ok(Ok(n)) if n > 0 => {
                output_data(cli, &buf[..n]);
            }
            _ => {}
        }
        
        return Ok(());
    }
    
    // Interactive mode would go here
    eprintln!("Interactive TCP mode not implemented in CLI");
    
    Ok(())
}

async fn connect_ssh(
    cli: &Cli,
    host: &str,
    port: u16,
    user: &str,
    password: Option<&str>,
    identity: Option<&PathBuf>,
    command: Option<&str>,
    no_pty: bool,
) -> anyhow::Result<()> {
    if !cli.quiet {
        eprintln!("Connecting to {}@{}:{}...", user, host, port);
    }
    
    let tcp = std::net::TcpStream::connect_timeout(
        &format!("{}:{}", host, port).parse()?,
        Duration::from_secs(10)
    )?;
    
    let mut session = ssh2::Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;
    
    // Authenticate
    if let Some(key_path) = identity {
        session.userauth_pubkey_file(user, None, key_path, None)?;
    } else if let Some(pass) = password {
        session.userauth_password(user, pass)?;
    } else {
        // Try agent
        let mut agent = session.agent()?;
        agent.connect()?;
        agent.list_identities()?;
        
        let identities: Vec<_> = agent.identities()?.into_iter().collect();
        for identity in identities {
            if agent.userauth(user, &identity).is_ok() {
                break;
            }
        }
    }
    
    if !session.authenticated() {
        anyhow::bail!("Authentication failed");
    }
    
    if !cli.quiet {
        eprintln!("Authenticated.");
    }
    
    let mut channel = session.channel_session()?;
    
    if !no_pty {
        channel.request_pty("xterm-256color", None, Some((80, 24, 0, 0)))?;
    }
    
    if let Some(cmd) = command {
        channel.exec(cmd)?;
    } else {
        channel.shell()?;
    }
    
    // Read output
    let mut buf = vec![0u8; 4096];
    use std::io::Read;
    
    loop {
        match channel.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                output_data(cli, &buf[..n]);
            }
            Err(_) => break,
        }
    }
    
    channel.wait_close()?;
    
    if !cli.quiet {
        let exit_status = channel.exit_status()?;
        eprintln!("Exit status: {}", exit_status);
    }
    
    Ok(())
}

async fn ble_scan(cli: &Cli, duration: u64, filter: Option<&str>) -> anyhow::Result<()> {
    if !cli.quiet {
        eprintln!("Scanning for BLE devices for {} seconds...", duration);
    }
    
    use termicon_core::core::transport::BluetoothScanner;
    
    let mut scanner = BluetoothScanner::new().await?;
    scanner.start_scan(duration).await?;
    
    let devices = scanner.get_devices();
    
    let filter_re = filter.map(|f| regex::Regex::new(f).ok()).flatten();
    
    let filtered: Vec<_> = devices.iter()
        .filter(|d| {
            if let Some(ref re) = filter_re {
                re.is_match(&d.name) || re.is_match(&d.address)
            } else {
                true
            }
        })
        .collect();
    
    match cli.format {
        OutputFormat::Json => {
            let json: Vec<serde_json::Value> = filtered.iter().map(|d| {
                serde_json::json!({
                    "name": d.name,
                    "address": d.address,
                    "rssi": d.rssi,
                    "services": d.services
                })
            }).collect();
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            if filtered.is_empty() {
                println!("No devices found.");
            } else {
                println!("{:<30} {:<20} {:>8}", "Name", "Address", "RSSI");
                println!("{:-<60}", "");
                for d in filtered {
                    let rssi = d.rssi.map(|r| format!("{} dBm", r)).unwrap_or_else(|| "N/A".to_string());
                    println!("{:<30} {:<20} {:>8}", d.name, d.address, rssi);
                }
            }
        }
    }
    
    Ok(())
}

async fn run_bridge(cli: &Cli, serial_port: &str, baud: u32, tcp_port: u16, daemon: bool) -> anyhow::Result<()> {
    if !cli.quiet {
        eprintln!("Starting bridge: {} @ {} <-> TCP port {}", serial_port, baud, tcp_port);
    }
    
    if daemon {
        eprintln!("Daemon mode not implemented yet");
    }
    
    // Simplified bridge implementation
    eprintln!("Bridge running. Press Ctrl+C to stop.");
    
    // Would implement actual bridge here
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn send_hex(
    cli: &Cli,
    conn_type: &str,
    target: &str,
    data: &str,
    wait: bool,
    timeout: u64,
) -> anyhow::Result<()> {
    let bytes = hex::decode(data.replace(' ', ""))?;
    
    if !cli.quiet {
        eprintln!("Sending {} bytes...", bytes.len());
    }
    
    match conn_type {
        "serial" => {
            let mut serial = serialport::new(target, 115200)
                .timeout(Duration::from_millis(timeout))
                .open()?;
            
            serial.write_all(&bytes)?;
            
            if wait {
                let mut buf = vec![0u8; 1024];
                if let Ok(n) = serial.read(&mut buf) {
                    output_data(cli, &buf[..n]);
                }
            }
        }
        "tcp" => {
            use tokio::net::TcpStream;
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            
            let mut stream = TcpStream::connect(target).await?;
            stream.write_all(&bytes).await?;
            
            if wait {
                let mut buf = vec![0u8; 1024];
                if let Ok(n) = stream.read(&mut buf).await {
                    output_data(cli, &buf[..n]);
                }
            }
        }
        _ => {
            anyhow::bail!("Unknown connection type: {}", conn_type);
        }
    }
    
    Ok(())
}

fn show_info(cli: &Cli) -> anyhow::Result<()> {
    let info = serde_json::json!({
        "version": "0.1.0",
        "features": [
            "serial", "tcp", "telnet", "ssh", "bluetooth",
            "xmodem", "ymodem", "zmodem", "sftp",
            "modbus", "slip", "cobs",
            "vt100", "vt220", "ansi", "256color", "truecolor"
        ],
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH
    });
    
    match cli.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        _ => {
            println!("Termicon v0.1.0");
            println!("Platform: {} ({})", std::env::consts::OS, std::env::consts::ARCH);
            println!();
            println!("Supported protocols:");
            println!("  • Serial (RS-232/RS-485/USB)");
            println!("  • TCP/IP");
            println!("  • Telnet");
            println!("  • SSH-2");
            println!("  • Bluetooth LE");
            println!();
            println!("File transfer: XMODEM, YMODEM, ZMODEM, SFTP");
            println!("Terminal: VT100/VT220/ANSI, 256-color, True Color");
        }
    }
    
    Ok(())
}

fn handle_profile(cli: &Cli, action: &ProfileAction) -> anyhow::Result<()> {
    match action {
        ProfileAction::List => {
            println!("Profile management not yet implemented");
        }
        ProfileAction::Show { name } => {
            println!("Show profile: {}", name);
        }
        ProfileAction::Connect { name } => {
            println!("Connect using profile: {}", name);
        }
        ProfileAction::Export { name, file } => {
            println!("Export profile {} to {:?}", name, file);
        }
        ProfileAction::Import { file } => {
            println!("Import profile from {:?}", file);
        }
    }
    Ok(())
}

fn output_data(cli: &Cli, data: &[u8]) {
    match cli.format {
        OutputFormat::Hex => {
            println!("{}", hex::encode(data));
        }
        OutputFormat::Json => {
            println!("{}", serde_json::json!({
                "hex": hex::encode(data),
                "ascii": String::from_utf8_lossy(data)
            }));
        }
        _ => {
            let _ = io::stdout().write_all(data);
            let _ = io::stdout().flush();
        }
    }
}
