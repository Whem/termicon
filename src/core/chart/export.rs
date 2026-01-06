//! Chart Export to PNG/SVG
//!
//! Provides functionality to export chart data and visualizations to
//! PNG and SVG image formats.

use std::io::Write;
use std::path::Path;

/// Chart export format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    PNG,
    SVG,
    CSV,
    JSON,
}

impl ExportFormat {
    pub fn extension(&self) -> &str {
        match self {
            Self::PNG => "png",
            Self::SVG => "svg",
            Self::CSV => "csv",
            Self::JSON => "json",
        }
    }
    
    pub fn mime_type(&self) -> &str {
        match self {
            Self::PNG => "image/png",
            Self::SVG => "image/svg+xml",
            Self::CSV => "text/csv",
            Self::JSON => "application/json",
        }
    }
}

/// Chart export configuration
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Export format
    pub format: ExportFormat,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Background color (RGBA)
    pub background: u32,
    /// Include grid lines
    pub show_grid: bool,
    /// Include legend
    pub show_legend: bool,
    /// Include axis labels
    pub show_axis: bool,
    /// Include title
    pub title: Option<String>,
    /// DPI for PNG export
    pub dpi: u32,
    /// Margin in pixels
    pub margin: u32,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::PNG,
            width: 1200,
            height: 800,
            background: 0xFFFFFFFF,
            show_grid: true,
            show_legend: true,
            show_axis: true,
            title: None,
            dpi: 96,
            margin: 50,
        }
    }
}

/// Data series for export
#[derive(Debug, Clone)]
pub struct ExportSeries {
    pub name: String,
    pub color: u32,
    pub data: Vec<(f64, f64)>, // (x, y) pairs
    pub line_width: f32,
}

/// SVG chart exporter
pub struct SvgExporter {
    config: ExportConfig,
}

impl SvgExporter {
    pub fn new(config: ExportConfig) -> Self {
        Self { config }
    }
    
    /// Export chart data to SVG string
    pub fn export(&self, series: &[ExportSeries], x_range: (f64, f64), y_range: (f64, f64)) -> String {
        let width = self.config.width;
        let height = self.config.height;
        let margin = self.config.margin;
        
        let plot_width = width - 2 * margin;
        let plot_height = height - 2 * margin;
        
        let bg_r = ((self.config.background >> 24) & 0xFF) as u8;
        let bg_g = ((self.config.background >> 16) & 0xFF) as u8;
        let bg_b = ((self.config.background >> 8) & 0xFF) as u8;
        
        let mut svg = String::new();
        
        // SVG header
        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
  <style>
    .grid {{ stroke: #e0e0e0; stroke-width: 0.5; }}
    .axis {{ stroke: #333; stroke-width: 1; }}
    .label {{ font-family: sans-serif; font-size: 12px; fill: #333; }}
    .title {{ font-family: sans-serif; font-size: 16px; fill: #333; font-weight: bold; }}
    .legend {{ font-family: sans-serif; font-size: 11px; fill: #333; }}
  </style>
  <rect width="100%" height="100%" fill="rgb({},{},{})"/>
"#,
            width, height, width, height, bg_r, bg_g, bg_b
        ));
        
        // Title
        if let Some(title) = &self.config.title {
            svg.push_str(&format!(
                r#"  <text x="{}" y="25" class="title" text-anchor="middle">{}</text>
"#,
                width / 2, title
            ));
        }
        
        // Plot area
        svg.push_str(&format!(
            r#"  <g transform="translate({}, {})">
"#,
            margin, margin
        ));
        
        // Grid lines
        if self.config.show_grid {
            let grid_lines = 10;
            
            // Vertical grid lines
            for i in 0..=grid_lines {
                let x = (i as f32 / grid_lines as f32) * plot_width as f32;
                svg.push_str(&format!(
                    r#"    <line x1="{:.1}" y1="0" x2="{:.1}" y2="{}" class="grid"/>
"#,
                    x, x, plot_height
                ));
            }
            
            // Horizontal grid lines
            for i in 0..=grid_lines {
                let y = (i as f32 / grid_lines as f32) * plot_height as f32;
                svg.push_str(&format!(
                    r#"    <line x1="0" y1="{:.1}" x2="{}" y2="{:.1}" class="grid"/>
"#,
                    y, plot_width, y
                ));
            }
        }
        
        // Axis
        if self.config.show_axis {
            svg.push_str(&format!(
                r#"    <line x1="0" y1="{}" x2="{}" y2="{}" class="axis"/>
    <line x1="0" y1="0" x2="0" y2="{}" class="axis"/>
"#,
                plot_height, plot_width, plot_height, plot_height
            ));
            
            // X axis labels
            let x_step = (x_range.1 - x_range.0) / 5.0;
            for i in 0..=5 {
                let x_val = x_range.0 + i as f64 * x_step;
                let x_pos = (i as f32 / 5.0) * plot_width as f32;
                svg.push_str(&format!(
                    r#"    <text x="{:.1}" y="{}" class="label" text-anchor="middle">{:.1}</text>
"#,
                    x_pos, plot_height as f32 + 20.0, x_val
                ));
            }
            
            // Y axis labels
            let y_step = (y_range.1 - y_range.0) / 5.0;
            for i in 0..=5 {
                let y_val = y_range.0 + i as f64 * y_step;
                let y_pos = plot_height as f32 - (i as f32 / 5.0) * plot_height as f32;
                svg.push_str(&format!(
                    r#"    <text x="-10" y="{:.1}" class="label" text-anchor="end" dominant-baseline="middle">{:.1}</text>
"#,
                    y_pos, y_val
                ));
            }
        }
        
        // Data series
        for s in series {
            if s.data.is_empty() {
                continue;
            }
            
            let r = ((s.color >> 24) & 0xFF) as u8;
            let g = ((s.color >> 16) & 0xFF) as u8;
            let b = ((s.color >> 8) & 0xFF) as u8;
            
            let mut path = String::new();
            let mut first = true;
            
            for (x, y) in &s.data {
                let px = ((x - x_range.0) / (x_range.1 - x_range.0)) * plot_width as f64;
                let py = plot_height as f64 - ((y - y_range.0) / (y_range.1 - y_range.0)) * plot_height as f64;
                
                if first {
                    path.push_str(&format!("M{:.2},{:.2}", px, py));
                    first = false;
                } else {
                    path.push_str(&format!(" L{:.2},{:.2}", px, py));
                }
            }
            
            svg.push_str(&format!(
                r#"    <path d="{}" fill="none" stroke="rgb({},{},{})" stroke-width="{}"/>
"#,
                path, r, g, b, s.line_width
            ));
        }
        
        svg.push_str("  </g>\n");
        
        // Legend
        if self.config.show_legend && !series.is_empty() {
            let legend_x = width - margin - 150;
            let legend_y = margin + 20;
            let legend_height = series.len() * 20 + 10;
            
            svg.push_str(&format!(
                "  <g transform=\"translate({}, {})\">\n    <rect x=\"0\" y=\"0\" width=\"140\" height=\"{}\" fill=\"white\" stroke=\"#ccc\" rx=\"5\"/>\n",
                legend_x, legend_y, legend_height
            ));
            
            for (i, s) in series.iter().enumerate() {
                let y = 15 + i * 20;
                let r = ((s.color >> 24) & 0xFF) as u8;
                let g = ((s.color >> 16) & 0xFF) as u8;
                let b = ((s.color >> 8) & 0xFF) as u8;
                
                svg.push_str(&format!(
                    "    <line x1=\"10\" y1=\"{}\" x2=\"30\" y2=\"{}\" stroke=\"rgb({},{},{})\" stroke-width=\"2\"/>\n    <text x=\"40\" y=\"{}\" class=\"legend\">{}</text>\n",
                    y, y, r, g, b, y + 4, s.name
                ));
            }
            
            svg.push_str("  </g>\n");
        }
        
        svg.push_str("</svg>");
        svg
    }
    
    /// Export to file
    pub fn export_to_file(&self, path: &Path, series: &[ExportSeries], x_range: (f64, f64), y_range: (f64, f64)) -> std::io::Result<()> {
        let svg = self.export(series, x_range, y_range);
        std::fs::write(path, svg)
    }
}

/// PNG chart exporter (using simple PPM as fallback, real PNG requires image crate)
pub struct PngExporter {
    config: ExportConfig,
}

impl PngExporter {
    pub fn new(config: ExportConfig) -> Self {
        Self { config }
    }
    
    /// Render chart to raw RGBA pixels
    pub fn render(&self, series: &[ExportSeries], x_range: (f64, f64), y_range: (f64, f64)) -> Vec<u8> {
        let width = self.config.width as usize;
        let height = self.config.height as usize;
        let margin = self.config.margin as usize;
        
        // Initialize with background color
        let mut pixels = vec![0u8; width * height * 4];
        let bg_r = ((self.config.background >> 24) & 0xFF) as u8;
        let bg_g = ((self.config.background >> 16) & 0xFF) as u8;
        let bg_b = ((self.config.background >> 8) & 0xFF) as u8;
        let bg_a = (self.config.background & 0xFF) as u8;
        
        for i in 0..(width * height) {
            pixels[i * 4] = bg_r;
            pixels[i * 4 + 1] = bg_g;
            pixels[i * 4 + 2] = bg_b;
            pixels[i * 4 + 3] = bg_a;
        }
        
        let plot_width = width - 2 * margin;
        let plot_height = height - 2 * margin;
        
        // Draw grid
        if self.config.show_grid {
            let grid_color = [0xE0u8, 0xE0, 0xE0, 0xFF];
            
            for i in 0..=10 {
                let x = margin + (i * plot_width / 10);
                for y in margin..(height - margin) {
                    Self::set_pixel(&mut pixels, width, x, y, &grid_color);
                }
                
                let y = margin + (i * plot_height / 10);
                for x in margin..(width - margin) {
                    Self::set_pixel(&mut pixels, width, x, y, &grid_color);
                }
            }
        }
        
        // Draw series
        for s in series {
            if s.data.len() < 2 {
                continue;
            }
            
            let r = ((s.color >> 24) & 0xFF) as u8;
            let g = ((s.color >> 16) & 0xFF) as u8;
            let b = ((s.color >> 8) & 0xFF) as u8;
            let color = [r, g, b, 0xFF];
            
            for window in s.data.windows(2) {
                let (x1, y1) = window[0];
                let (x2, y2) = window[1];
                
                let px1 = margin as f64 + ((x1 - x_range.0) / (x_range.1 - x_range.0)) * plot_width as f64;
                let py1 = (height - margin) as f64 - ((y1 - y_range.0) / (y_range.1 - y_range.0)) * plot_height as f64;
                let px2 = margin as f64 + ((x2 - x_range.0) / (x_range.1 - x_range.0)) * plot_width as f64;
                let py2 = (height - margin) as f64 - ((y2 - y_range.0) / (y_range.1 - y_range.0)) * plot_height as f64;
                
                Self::draw_line(&mut pixels, width, height, px1 as i32, py1 as i32, px2 as i32, py2 as i32, &color);
            }
        }
        
        pixels
    }
    
    fn set_pixel(pixels: &mut [u8], width: usize, x: usize, y: usize, color: &[u8; 4]) {
        if x < width {
            let idx = (y * width + x) * 4;
            if idx + 3 < pixels.len() {
                pixels[idx] = color[0];
                pixels[idx + 1] = color[1];
                pixels[idx + 2] = color[2];
                pixels[idx + 3] = color[3];
            }
        }
    }
    
    fn draw_line(pixels: &mut [u8], width: usize, height: usize, x0: i32, y0: i32, x1: i32, y1: i32, color: &[u8; 4]) {
        // Bresenham's line algorithm
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        
        let mut x = x0;
        let mut y = y0;
        
        loop {
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                Self::set_pixel(pixels, width, x as usize, y as usize, color);
            }
            
            if x == x1 && y == y1 {
                break;
            }
            
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
    
    /// Export to PPM file (simple format, works without external dependencies)
    pub fn export_to_ppm(&self, path: &Path, series: &[ExportSeries], x_range: (f64, f64), y_range: (f64, f64)) -> std::io::Result<()> {
        let pixels = self.render(series, x_range, y_range);
        let width = self.config.width;
        let height = self.config.height;
        
        let mut file = std::fs::File::create(path)?;
        
        // PPM header
        writeln!(file, "P6")?;
        writeln!(file, "{} {}", width, height)?;
        writeln!(file, "255")?;
        
        // Pixel data (RGB only, no alpha)
        for chunk in pixels.chunks(4) {
            file.write_all(&[chunk[0], chunk[1], chunk[2]])?;
        }
        
        Ok(())
    }
}

/// Chart data exporter (CSV/JSON)
pub struct DataExporter;

impl DataExporter {
    /// Export to CSV
    pub fn to_csv(series: &[ExportSeries]) -> String {
        let mut csv = String::new();
        
        // Header
        csv.push_str("time");
        for s in series {
            csv.push(',');
            csv.push_str(&s.name);
        }
        csv.push('\n');
        
        // Find all unique x values
        let mut all_x: Vec<f64> = Vec::new();
        for s in series {
            for (x, _) in &s.data {
                if !all_x.iter().any(|&v| (v - x).abs() < 0.0001) {
                    all_x.push(*x);
                }
            }
        }
        all_x.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Data rows
        for x in &all_x {
            csv.push_str(&format!("{:.6}", x));
            
            for s in series {
                csv.push(',');
                if let Some((_, y)) = s.data.iter().find(|(sx, _)| (sx - x).abs() < 0.0001) {
                    csv.push_str(&format!("{:.6}", y));
                }
            }
            csv.push('\n');
        }
        
        csv
    }
    
    /// Export to JSON
    pub fn to_json(series: &[ExportSeries]) -> String {
        let mut json = String::from("{\n  \"series\": [\n");
        
        for (i, s) in series.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!("      \"name\": \"{}\",\n", s.name));
            json.push_str(&format!("      \"color\": \"#{:08X}\",\n", s.color));
            json.push_str("      \"data\": [\n");
            
            for (j, (x, y)) in s.data.iter().enumerate() {
                json.push_str(&format!("        [{:.6}, {:.6}]", x, y));
                if j < s.data.len() - 1 {
                    json.push(',');
                }
                json.push('\n');
            }
            
            json.push_str("      ]\n    }");
            if i < series.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        
        json.push_str("  ]\n}");
        json
    }
    
    /// Export to file
    pub fn export_to_file(path: &Path, series: &[ExportSeries], format: ExportFormat) -> std::io::Result<()> {
        let content = match format {
            ExportFormat::CSV => Self::to_csv(series),
            ExportFormat::JSON => Self::to_json(series),
            _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Use SvgExporter or PngExporter for image formats")),
        };
        
        std::fs::write(path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_series() -> Vec<ExportSeries> {
        vec![
            ExportSeries {
                name: "Temperature".to_string(),
                color: 0xFF0000FF,
                data: vec![(0.0, 20.0), (1.0, 22.0), (2.0, 21.0), (3.0, 25.0)],
                line_width: 2.0,
            },
            ExportSeries {
                name: "Humidity".to_string(),
                color: 0x0000FFFF,
                data: vec![(0.0, 50.0), (1.0, 55.0), (2.0, 52.0), (3.0, 48.0)],
                line_width: 2.0,
            },
        ]
    }
    
    #[test]
    fn test_svg_export() {
        let config = ExportConfig::default();
        let exporter = SvgExporter::new(config);
        let series = test_series();
        
        let svg = exporter.export(&series, (0.0, 3.0), (0.0, 60.0));
        
        assert!(svg.contains("<?xml"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("Temperature"));
    }
    
    #[test]
    fn test_csv_export() {
        let series = test_series();
        let csv = DataExporter::to_csv(&series);
        
        assert!(csv.contains("time,Temperature,Humidity"));
        assert!(csv.contains("0.000000"));
    }
    
    #[test]
    fn test_json_export() {
        let series = test_series();
        let json = DataExporter::to_json(&series);
        
        assert!(json.contains("\"series\""));
        assert!(json.contains("\"name\": \"Temperature\""));
    }
}


