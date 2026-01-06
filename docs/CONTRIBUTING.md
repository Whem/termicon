# Contributing to Termicon

Thank you for your interest in contributing to Termicon! This document provides guidelines and information for contributors.

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/termicon.git
   cd termicon
   ```
3. Create a branch for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- Rust 1.75 or later
- On Linux: `libudev-dev` for serial port detection
  ```bash
  sudo apt install libudev-dev pkg-config libssl-dev
  ```
- On Windows: Visual Studio Build Tools

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Run CLI
cargo run --bin termicon-cli -- --help
```

### Code Style

We use standard Rust formatting and linting:

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy -- -W clippy::all -W clippy::pedantic
```

## Code Organization

```
src/
├── main.rs              # GUI entry point
├── lib.rs               # Library exports
├── bin/cli.rs           # CLI entry point
├── core/                # Core functionality
│   ├── transport/       # Serial, TCP, Telnet, SSH, Bluetooth
│   ├── terminal/        # VT100/VT220/ANSI emulation
│   ├── chart/           # Real-time charting
│   ├── protocol/        # Modbus, CRC, framing
│   ├── bridge/          # Network bridging
│   ├── virtual_port/    # Virtual COM ports
│   ├── profile/         # Connection profiles
│   ├── snippet/         # Command snippets
│   ├── transfer/        # File transfer (XMODEM/YMODEM/ZMODEM)
│   ├── session.rs       # Session management
│   ├── codec/           # Data encoding
│   ├── logger.rs        # Session logging
│   ├── trigger.rs       # Pattern triggers
│   ├── macros.rs        # Quick macros M1-M24
│   ├── macro_recorder.rs# Macro recording/playback
│   ├── capability.rs    # Transport capabilities
│   ├── state_machine.rs # Session state machine
│   ├── packet.rs        # Packet abstraction
│   ├── protocol_dsl.rs  # Protocol definitions
│   ├── replay.rs        # Session replay
│   ├── simulator.rs     # Virtual devices
│   ├── vault.rs         # Credential vault
│   ├── knowledge.rs     # Device knowledge base
│   ├── deterministic.rs # Reproducible runs
│   ├── fuzzing.rs       # Protocol fuzzing
│   ├── routing.rs       # Transport routing
│   ├── adaptive.rs      # Adaptive automation
│   ├── arbitration.rs   # Resource arbitration
│   ├── experiment.rs    # Parameter sweeps
│   ├── explain.rs       # Root cause analysis
│   ├── collaborative.rs # Team features
│   ├── external_api.rs  # REST/WebSocket API
│   └── plugin/          # Plugin system
├── gui/                 # GUI components
│   ├── app.rs           # Main application
│   ├── session_tab.rs   # Tab management
│   ├── chart_panel.rs   # Chart view
│   ├── sftp_panel.rs    # SFTP browser
│   ├── macros_panel.rs  # M1-M24 macros
│   ├── profiles.rs      # Profile management
│   ├── command_palette.rs # Command palette
│   └── ansi_parser.rs   # ANSI color parsing
├── config/              # Configuration
├── i18n/                # Internationalization
└── utils/               # Utilities

locales/                 # Translation files
├── en.yml               # English
└── hu.yml               # Hungarian
```

## Making Changes

### Adding a New Transport

1. Create a new file in `src/core/transport/`
2. Implement the `TransportTrait`
3. Add to `Transport` enum in `mod.rs`
4. Update `create_transport()` function
5. Add capability declaration

### Adding Translations

1. Add keys to the relevant section in `locales/en.yml`
2. Add corresponding translations to `locales/hu.yml`
3. Use `t!("key.path")` in code
4. Test by switching languages in Settings

### Adding a New Protocol

1. Create a new file in `src/core/protocol/`
2. Implement the `ProtocolDecoder` trait
3. Add to protocol factory function
4. Update documentation

### Adding GUI Features

1. Create new panel/component in `src/gui/`
2. Register in `src/gui/mod.rs`
3. Integrate into `app.rs`
4. Add translations for UI text
5. Test with both themes (Dark/Light)

## Pull Request Process

1. Ensure your code passes all tests:
   ```bash
   cargo test
   ```

2. Ensure your code is formatted:
   ```bash
   cargo fmt --check
   ```

3. Ensure clippy passes:
   ```bash
   cargo clippy
   ```

4. Update documentation if needed

5. Create a pull request with:
   - Clear description of changes
   - Link to related issue (if any)
   - Screenshots for UI changes

## Commit Messages

Use conventional commit messages:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting)
- `refactor:` Code refactoring
- `test:` Adding tests
- `chore:` Maintenance tasks
- `i18n:` Translation updates

Examples:
```
feat: add ZMODEM file transfer protocol
fix: serial port reconnection on Windows
docs: update installation instructions
i18n: add Hungarian translations for macros panel
```

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

### Writing Tests

- Place unit tests in the same file as the code
- Place integration tests in `tests/`
- Use `#[cfg(test)]` for test modules

## Documentation

- Use rustdoc comments (`///`) for public items
- Include examples in documentation
- Keep README.md updated
- Update FULL_FEATURE_MATRIX.md for new features

## Reporting Issues

When reporting issues, please include:

1. Operating system and version
2. Rust version (`rustc --version`)
3. Steps to reproduce
4. Expected vs actual behavior
5. Error messages or logs
6. Screenshots for UI issues

## Feature Requests

For feature requests:

1. Check existing issues first
2. Describe the use case
3. Suggest an implementation approach
4. Consider i18n implications

## Translation Guidelines

When adding translations:

1. Keep keys hierarchical (e.g., `menu.file.open`)
2. Use descriptive key names
3. Add translations to ALL language files
4. Test with different text lengths
5. Avoid hardcoded strings in code

## Questions?

Feel free to open an issue for questions or discussions.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
