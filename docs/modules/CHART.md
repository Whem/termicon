# Chart Module

## Overview

The Chart module provides real-time data visualization with support for multiple channels, various data formats, and export capabilities.

## Features

| Feature | Status | Description |
|---------|--------|-------------|
| Real-time Plot | ✅ | Live updating charts |
| Multiple Channels | ✅ | Up to 16 data series |
| Auto-scaling | ✅ | Automatic Y-axis range |
| Time Axis | ✅ | Scrolling time axis |
| CSV Parser | ✅ | Comma-separated values |
| JSON Parser | ✅ | JSON data extraction |
| Key-Value Parser | ✅ | key=value format |
| Regex Parser | ✅ | Custom patterns |
| Rolling Stats | ✅ | Min/max/avg |
| CSV Export | ✅ | Export to file |
| Zoom/Pan | ✅ | Interactive navigation |
| Data Markers | 🔄 | Point annotations |
| PNG Export | ❌ | Image export |

## Configuration

```rust
pub struct ChartConfig {
    pub max_points: usize,          // Points per channel
    pub update_interval: Duration,  // Refresh rate
    pub parser: ParserType,         // Data parser
    pub channels: Vec<ChannelConfig>,
}

pub struct ChannelConfig {
    pub name: String,
    pub color: Color,
    pub visible: bool,
    pub y_min: Option<f64>,
    pub y_max: Option<f64>,
}
```

## Data Parsers

### CSV Parser

Parses comma-separated numeric values:

```
123.45, 67.89, 42.0
100.0, 200.0, 300.0
```

```rust
use termicon_core::chart::CsvParser;

let parser = CsvParser::new(","); // delimiter

let values = parser.parse("123.45, 67.89, 42.0")?;
// values = [123.45, 67.89, 42.0]
```

### JSON Parser

Extracts values from JSON:

```json
{"temp": 25.5, "humidity": 60.0, "pressure": 1013.25}
```

```rust
use termicon_core::chart::JsonParser;

let parser = JsonParser::new(vec!["temp", "humidity", "pressure"]);

let values = parser.parse(json_string)?;
// values = [25.5, 60.0, 1013.25]
```

### Key-Value Parser

Parses `key=value` format:

```
temp=25.5 humidity=60 pressure=1013.25
```

```rust
use termicon_core::chart::KeyValueParser;

let parser = KeyValueParser::new(vec!["temp", "humidity", "pressure"]);

let values = parser.parse("temp=25.5 humidity=60")?;
```

### Regex Parser

Custom extraction patterns:

```rust
use termicon_core::chart::RegexParser;

// Extract numbers after "T:" and "H:"
let parser = RegexParser::new(r"T:(\d+\.?\d*)\s+H:(\d+\.?\d*)");

let values = parser.parse("Sensor: T:25.5 H:60.0")?;
// values = [25.5, 60.0]
```

### Column Parser

Fixed-width columns:

```rust
use termicon_core::chart::ColumnParser;

let parser = ColumnParser::new(vec![
    (0, 6),   // Column 1: chars 0-6
    (7, 13),  // Column 2: chars 7-13
    (14, 20), // Column 3: chars 14-20
]);
```

## GUI Usage

### Opening Chart View

1. Click **[G]** (Chart) in side panel
2. Or use Command Palette: Ctrl+K → "chart"

### Chart Panel

The chart panel shows:
- Real-time graph with time axis
- Channel legend with colors
- Statistics (min, max, average)
- Parser configuration
- Export button

### Configuring Channels

1. Click **Configure** button
2. Set channel names and colors
3. Choose parser type
4. Set Y-axis range (optional)
5. Click **Apply**

### Interactive Controls

- **Scroll**: Zoom time axis
- **Drag**: Pan view
- **Double-click**: Reset zoom
- **Click legend**: Toggle channel visibility

## Code Examples

### Basic Chart

```rust
use termicon_core::chart::{Chart, ChartConfig, CsvParser};

let config = ChartConfig {
    max_points: 1000,
    update_interval: Duration::from_millis(100),
    parser: ParserType::Csv,
    channels: vec![
        ChannelConfig {
            name: "Temperature".to_string(),
            color: Color::RED,
            visible: true,
            y_min: Some(0.0),
            y_max: Some(100.0),
        },
        ChannelConfig {
            name: "Humidity".to_string(),
            color: Color::BLUE,
            visible: true,
            y_min: None,
            y_max: None,
        },
    ],
};

let mut chart = Chart::new(config);

// Add data point
chart.add_line("25.5, 60.0");

// Or add values directly
chart.add_values(&[25.5, 60.0]);
```

### Rolling Statistics

```rust
let stats = chart.statistics();

for (i, channel_stats) in stats.iter().enumerate() {
    println!("Channel {}: min={:.2}, max={:.2}, avg={:.2}",
        i,
        channel_stats.min,
        channel_stats.max,
        channel_stats.average);
}
```

### Export to CSV

```rust
chart.export_csv("data.csv")?;

// With custom options
chart.export_csv_with_options("data.csv", ExportOptions {
    include_timestamps: true,
    delimiter: ',',
    decimal_places: 3,
})?;
```

## Data Sources

### From Serial

```rust
// Parse incoming serial data
transport.on_receive(|data| {
    if let Ok(line) = String::from_utf8(data.to_vec()) {
        chart.add_line(&line);
    }
});
```

### From Session

```rust
// Chart data from active session
session.subscribe_data(|data| {
    chart.add_data(&data);
});
```

### Manual Input

```rust
// Add timestamped data
chart.add_point(channel_idx, timestamp, value);

// Add current time
chart.add_value(channel_idx, value);
```

## Display Options

### Time Axis

```rust
chart.set_time_window(Duration::from_secs(60)); // Show last 60s
chart.set_time_format("%H:%M:%S"); // Time format
```

### Y-Axis

```rust
chart.set_auto_scale(true);
chart.set_y_range(Some(0.0), Some(100.0));
chart.set_grid(true);
```

### Appearance

```rust
chart.set_background(Color::BLACK);
chart.set_line_width(2.0);
chart.set_point_size(4.0);
chart.set_legend_position(LegendPosition::TopRight);
```

## Integration with Sessions

Charts integrate with terminal sessions:

```rust
// Create chart for session
let chart = session.create_chart(config);

// Chart automatically receives parsed data
// from session output

// Access from GUI
app.show_chart_panel();
```

## CLI Usage

```bash
# Start with chart view
termicon-cli serial COM3 --chart

# Configure parser
termicon-cli serial COM3 --chart --parser csv

# Export data
termicon-cli serial COM3 --chart --export data.csv --duration 60
```

## Troubleshooting

### No Data Appearing

- Check parser matches data format
- Verify data contains valid numbers
- Check channel visibility
- Verify session is connected

### Wrong Values

- Verify delimiter for CSV
- Check JSON field names
- Verify regex pattern
- Check column indices

### Performance Issues

- Reduce max_points
- Increase update_interval
- Disable unused channels
- Check data rate

## Example Data Formats

### Sensor Array (CSV)

```
23.5, 60.2, 1013.25, 2.3
23.6, 60.1, 1013.20, 2.4
```

### IoT JSON

```json
{"sensors":{"temp":23.5,"humidity":60.2}}
```

### Serial Monitor

```
T:23.5C H:60.2% P:1013.25hPa
```

### NMEA-style

```
$SENSOR,23.5,60.2,1013.25*4F
```
