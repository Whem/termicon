# SSH Module

## Overview

The SSH module provides secure shell connections with full terminal emulation, file transfer, and port forwarding capabilities.

## Features

| Feature | Status | Description |
|---------|--------|-------------|
| SSH-2 Protocol | ✅ | Modern SSH protocol |
| Password Auth | ✅ | Password authentication |
| Key Auth | ✅ | RSA, ECDSA, Ed25519 keys |
| SSH Agent | 🔄 | Agent forwarding |
| PTY Allocation | ✅ | Terminal emulation |
| PTY Resize | ✅ | Dynamic terminal size |
| Exec Command | ✅ | Remote command execution |
| SFTP | ✅ | Secure file transfer |
| Port Forwarding | 🔄 | Local/Remote/Dynamic |
| Jump Host | 🔄 | ProxyJump support |
| Compression | ✅ | zlib compression |
| Keepalive | ✅ | Connection keepalive |

## Configuration

```rust
pub struct SshConfig {
    pub host: String,
    pub port: u16,                    // Default: 22
    pub username: String,
    pub auth: SshAuth,
    pub compression: bool,
    pub keepalive_interval: Option<Duration>,
}

pub enum SshAuth {
    Password(String),
    PublicKey {
        private_key_path: PathBuf,
        passphrase: Option<String>,
    },
    Agent,
}
```

## GUI Usage

### Connection Dialog

1. Click **SSH** button in toolbar
2. Enter host and port (default: 22)
3. Enter username
4. Choose authentication method:
   - Password: Enter password
   - Key: Browse for private key file
5. Click **Connect**

### Terminal Features

Once connected:
- Full VT100/VT220/ANSI terminal emulation
- 256-color and true color support
- Mouse reporting
- Automatic terminal resize
- Command history with up/down arrows

### SFTP Browser

1. Connect via SSH
2. Click **SFTP** button in toolbar
3. Browse remote files on the left
4. Browse local files on the right
5. Double-click to navigate directories
6. Use upload/download buttons

## CLI Usage

```bash
# Basic connection
termicon-cli ssh user@hostname

# With port
termicon-cli ssh user@hostname:2222

# With key file
termicon-cli ssh user@hostname --key ~/.ssh/id_rsa

# Execute command
termicon-cli ssh user@hostname --exec "ls -la"

# SFTP operations
termicon-cli sftp user@hostname --get /remote/file /local/path
termicon-cli sftp user@hostname --put /local/file /remote/path
```

## Profile Support

SSH connections can be saved as profiles:

1. Connect to SSH server
2. Click **Save Profile** in toolbar
3. Enter profile name
4. Note: Passwords are NOT saved for security

When connecting from a saved SSH profile:
- The SSH dialog opens with pre-filled settings
- Enter password to complete connection

## Code Examples

### Password Authentication

```rust
use termicon_core::{SshConfig, SshAuth, Transport};

let config = SshConfig {
    host: "example.com".to_string(),
    port: 22,
    username: "user".to_string(),
    auth: SshAuth::Password("secret".to_string()),
    compression: true,
    keepalive_interval: Some(Duration::from_secs(30)),
};

let mut transport = Transport::Ssh(config);
transport.connect().await?;
```

### Key-based Authentication

```rust
use termicon_core::{SshConfig, SshAuth};

let config = SshConfig {
    host: "example.com".to_string(),
    port: 22,
    username: "user".to_string(),
    auth: SshAuth::PublicKey {
        private_key_path: PathBuf::from("/home/user/.ssh/id_ed25519"),
        passphrase: Some("keypass".to_string()),
    },
    compression: true,
    keepalive_interval: None,
};
```

### SFTP Operations

```rust
// List directory
let entries = sftp.list_dir("/home/user")?;
for entry in entries {
    println!("{} {} bytes", entry.name, entry.size);
}

// Download file
sftp.download("/remote/file.txt", "/local/file.txt")?;

// Upload file
sftp.upload("/local/file.txt", "/remote/file.txt")?;

// Create directory
sftp.mkdir("/remote/newdir")?;

// Delete file
sftp.remove("/remote/file.txt")?;
```

### Execute Command

```rust
// Execute single command
let output = ssh.exec("ls -la")?;
println!("Output: {}", String::from_utf8_lossy(&output));

// Execute with environment
let output = ssh.exec_with_env("echo $VAR", &[("VAR", "value")])?;
```

## Terminal Emulation

The SSH module includes full terminal emulation:

### Supported Features

- **Escape Sequences**: CSI, OSC, DCS
- **Colors**: 16 colors, 256 colors, true color (24-bit)
- **Attributes**: Bold, italic, underline, blink, reverse
- **Cursor**: Movement, visibility, styles
- **Screen**: Clear, scroll regions, alternate buffer
- **Mouse**: X10, X11, SGR reporting modes
- **Special**: Bracketed paste, focus events

### ANSI Color Support

SSH connections automatically handle ANSI escape codes:
- Standard 16 colors
- 256-color palette
- RGB true color
- Background/foreground colors
- Text attributes (bold, underline, etc.)

## SFTP Panel

The SFTP panel provides a dual-pane file browser:

### Left Pane (Remote)
- Current directory path
- File/folder listing
- Size, date, permissions
- Navigation buttons

### Right Pane (Local)
- Local directory browser
- Same features as remote

### Operations
- **Upload**: Send local files to remote
- **Download**: Get remote files to local
- **Create Directory**: Make new folder
- **Delete**: Remove files/folders
- **Refresh**: Reload directory listing

## Security Notes

### Key Management

- Private keys should be password-protected
- Use Ed25519 or ECDSA over RSA for new keys
- Never share private keys

### Known Hosts

- Host key verification on first connection
- Warning on host key change
- Keys stored in standard location

### Password Security

- Passwords are NEVER saved in profiles
- Passwords are cleared from memory after use
- Consider using key-based authentication

## Troubleshooting

### Connection Refused

- Check host is reachable: `ping hostname`
- Verify SSH port (default 22)
- Check firewall rules
- Verify SSH server is running

### Authentication Failed

- Check username spelling
- Verify password or key passphrase
- Check key file permissions (600 on Unix)
- Ensure public key is in authorized_keys

### Terminal Issues

- If colors don't work: Check TERM environment
- If special keys fail: Check terminal type
- If text is garbled: Check character encoding

### SFTP Issues

- Permission denied: Check remote file permissions
- Timeout: Large file transfer over slow connection
- Path not found: Use absolute paths

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Ctrl+C | Send interrupt (SIGINT) |
| Ctrl+D | Send EOF |
| Ctrl+Z | Suspend (background) |
| Ctrl+L | Clear screen |
| Tab | Auto-complete |
| Up/Down | Command history |
