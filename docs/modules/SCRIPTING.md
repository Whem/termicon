# Scripting & Automation Module

## Overview

The Scripting module provides automation capabilities including triggers, macros, snippets, and session recording/replay.

## Features

| Feature | Status | Description |
|---------|--------|-------------|
| Triggers | ✅ | Pattern-based automation |
| Regex Patterns | ✅ | Regular expressions |
| Hex Patterns | ✅ | Binary pattern matching |
| Auto-Response | ✅ | Automatic replies |
| Macro Recording | ✅ | Record actions |
| Macro Playback | ✅ | Replay actions |
| Quick Macros (M1-M24) | ✅ | Fast access macros |
| Command Snippets | ✅ | Saved commands |
| Profile Commands | ✅ | Per-profile history |
| Session Replay | ✅ | Playback sessions |
| Deterministic Mode | ✅ | Reproducible runs |
| Lua Scripting | ❌ | Future |

## Triggers

### Trigger Configuration

```rust
pub struct Trigger {
    pub id: String,
    pub name: String,
    pub condition: TriggerCondition,
    pub action: TriggerAction,
    pub enabled: bool,
    pub one_shot: bool,
}

pub enum TriggerCondition {
    Exact(Vec<u8>),
    Text(String),
    Regex(String),
    Hex(Vec<u8>),
    Timeout(Duration),
}

pub enum TriggerAction {
    Send(Vec<u8>),
    SendText(String),
    Log(String),
    Alert(String),
    RunScript(String),
    Chain(Vec<TriggerAction>),
}
```

### Creating Triggers

```rust
use termicon_core::trigger::{Trigger, TriggerCondition, TriggerAction};

// Auto-login trigger
let login_trigger = Trigger {
    id: "auto-login".to_string(),
    name: "Auto Login".to_string(),
    condition: TriggerCondition::Text("login:".to_string()),
    action: TriggerAction::SendText("admin\r\n".to_string()),
    enabled: true,
    one_shot: false,
};

// Password trigger
let password_trigger = Trigger {
    id: "auto-password".to_string(),
    name: "Auto Password".to_string(),
    condition: TriggerCondition::Text("Password:".to_string()),
    action: TriggerAction::SendText("secret123\r\n".to_string()),
    enabled: true,
    one_shot: true,
};

// Regex trigger
let error_trigger = Trigger {
    id: "error-alert".to_string(),
    name: "Error Alert".to_string(),
    condition: TriggerCondition::Regex(r"ERROR:\s+(.+)".to_string()),
    action: TriggerAction::Alert("Error detected!".to_string()),
    enabled: true,
    one_shot: false,
};

// Hex trigger
let modbus_trigger = Trigger {
    id: "modbus-error".to_string(),
    name: "Modbus Error".to_string(),
    condition: TriggerCondition::Hex(vec![0x01, 0x83]), // Slave 1, exception
    action: TriggerAction::Log("Modbus exception received".to_string()),
    enabled: true,
    one_shot: false,
};
```

### Trigger Manager

```rust
use termicon_core::trigger::TriggerManager;

let mut manager = TriggerManager::new();

manager.add(login_trigger);
manager.add(password_trigger);

// Check incoming data
let triggered = manager.check(&incoming_data);
for trigger in triggered {
    match &trigger.action {
        TriggerAction::SendText(text) => {
            transport.send(text.as_bytes()).await?;
        }
        TriggerAction::Alert(msg) => {
            show_alert(msg);
        }
        _ => {}
    }
}
```

## Macros

### Quick Macros (M1-M24)

24 quick-access macro slots for frequently used commands:

```rust
use termicon_core::macros::{MacroManager, MacroSlot, MacroContent};

let mut manager = MacroManager::load()?;

// Set macro M1
manager.set_slot(1, MacroSlot {
    name: "Show Version".to_string(),
    content: MacroContent::Text("show version\r\n".to_string()),
    append_crlf: false,
});

// Set hex macro M2
manager.set_slot(2, MacroSlot {
    name: "Modbus Query".to_string(),
    content: MacroContent::Hex(vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x0A]),
    append_crlf: false,
});

// Execute macro
let data = manager.get_slot(1)?.execute()?;
transport.send(&data).await?;
```

### GUI Macros Panel

The M1-M24 macro bar appears below the terminal:
- Click button to send macro
- Right-click to edit
- Drag to reorder
- Supports text and hex modes

### Macro Recording

```rust
use termicon_core::macro_recorder::{MacroRecorder, MacroAction};

let mut recorder = MacroRecorder::new();

// Start recording
recorder.start();

// Actions are recorded automatically
recorder.record(MacroAction::Send(b"command\r\n".to_vec()));
recorder.record(MacroAction::Delay(Duration::from_millis(100)));
recorder.record(MacroAction::WaitFor(b"OK".to_vec()));

// Stop recording
let macro_data = recorder.stop();

// Play back
let player = MacroPlayer::new(macro_data);
player.play(&mut transport).await?;

// Play with loop
player.play_loop(&mut transport, 10).await?; // 10 iterations
```

## Command Snippets

### Profile-Specific Commands

Commands typed in a profile are automatically saved:

```rust
use termicon_gui::profiles::{ProfileManager, ProfileSnippet};

let manager = ProfileManager::load();

// Commands are saved when sent
let profile = manager.get_mut("profile-id")?;
profile.add_snippet(ProfileSnippet::new(
    "List Files",
    "ls -la",
    "List files in directory",
));

// Get sorted by usage
let snippets = profile.sorted_snippets();

// Record usage
profile.record_snippet_use(snippet_idx);
```

### Snippet Categories

```rust
pub enum SnippetType {
    Command(String),      // Simple command
    Script(String),       // Multi-line script
    KeySequence(String),  // Key escape sequence
    Binary(Vec<u8>),      // Binary/hex data
}
```

## Session Replay

### Recording Sessions

```rust
use termicon_core::replay::{ReplayRecorder, ReplayEvent};

let mut recorder = ReplayRecorder::new();

// Start recording
recorder.start();

// Events are recorded with timestamps
recorder.record(ReplayEvent::Sent(data.to_vec()));
recorder.record(ReplayEvent::Received(data.to_vec()));
recorder.record(ReplayEvent::Connected);

// Stop and save
let session = recorder.stop();
session.save("session_2024-01-15.replay")?;
```

### Playing Back

```rust
use termicon_core::replay::{ReplayPlayer, ReplaySpeed};

let session = ReplaySession::load("session.replay")?;
let mut player = ReplayPlayer::new(session);

// Set playback speed
player.set_speed(ReplaySpeed::Double); // 2x speed

// Play
player.play().await?;

// Or step through
while let Some(event) = player.next() {
    match event {
        ReplayEvent::Sent(data) => { /* show sent */ }
        ReplayEvent::Received(data) => { /* show received */ }
        _ => {}
    }
    player.wait_for_timing().await;
}
```

## Deterministic Mode

For reproducible test runs:

```rust
use termicon_core::deterministic::{DeterministicSession, DeterministicConfig};

let config = DeterministicConfig {
    seed: 12345,
    normalize_timing: true,
    record_random: true,
};

let session = DeterministicSession::new(config);

// Same seed = same random values
let rand = session.next_random();

// Timing is normalized for reproducibility
session.wait(Duration::from_millis(100)).await;
```

## Adaptive Automation

Self-adjusting automation based on feedback:

```rust
use termicon_core::adaptive::{AdaptiveController, FeedbackRule};

let mut controller = AdaptiveController::new();

// Add feedback rule
controller.add_rule(FeedbackRule {
    condition: "crc_errors > 5",
    action: "baud_rate *= 0.9", // Reduce baud rate
    description: "Reduce speed on CRC errors",
});

// Metrics are tracked
controller.record_metric("crc_errors", 1.0);
controller.record_metric("latency_ms", 50.0);

// Check and apply adjustments
if let Some(adjustment) = controller.check_rules() {
    apply_adjustment(adjustment);
}
```

## GUI Usage

### Triggers Panel

1. Open Settings panel
2. Navigate to Triggers section
3. Add/Edit/Delete triggers
4. Enable/disable with checkbox

### Macros Bar

- Shows M1-M24 buttons when serial connected
- Click to send
- Right-click to configure
- Supports Hex mode

### Commands Panel

1. Click **[C]** in side panel
2. Shows profile-specific commands
3. Sorted by usage frequency
4. Double-click to insert

## CLI Usage

```bash
# Record session
termicon-cli serial COM3 --record session.replay

# Play back session
termicon-cli replay session.replay --speed 2.0

# Run with triggers
termicon-cli serial COM3 --triggers triggers.yaml

# Deterministic mode
termicon-cli serial COM3 --deterministic --seed 12345
```

## Example: Auto-Config Script

```yaml
triggers:
  - name: Wait for Ready
    condition:
      text: "Ready>"
    action:
      chain:
        - send: "config\r\n"
        
  - name: Wait for Config
    condition:
      text: "Config>"
    action:
      chain:
        - send: "set baud 115200\r\n"
        - delay: 100ms
        - send: "set mode auto\r\n"
        - delay: 100ms
        - send: "save\r\n"
        
  - name: Config Done
    condition:
      text: "Configuration saved"
    action:
      alert: "Device configured successfully!"
```

## Troubleshooting

### Triggers Not Firing

- Check pattern matches exactly
- Verify trigger is enabled
- Check for partial matches (use regex)
- Enable debug logging

### Macro Timing Issues

- Increase delays between commands
- Use WaitFor instead of fixed delays
- Check response timing

### Replay Sync Issues

- Use deterministic mode for exact replay
- Normalize timing if needed
- Record complete sessions
