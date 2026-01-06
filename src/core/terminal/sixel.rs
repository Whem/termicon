//! Sixel Graphics Support
//!
//! Implements basic Sixel graphics rendering for displaying images in the terminal.
//! Sixel is a bitmap graphics format for terminals that uses 6-pixel-high vertical strips.
//!
//! Reference: DEC VT340 Programmer Reference Manual

use std::collections::HashMap;

/// Sixel color definition
#[derive(Debug, Clone, Copy, Default)]
pub struct SixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl SixelColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    
    /// Parse HLS color (Hue, Lightness, Saturation)
    pub fn from_hls(h: u16, l: u16, s: u16) -> Self {
        // Convert HLS to RGB
        // H: 0-360, L: 0-100, S: 0-100
        let h = (h % 360) as f64;
        let l = (l.min(100) as f64) / 100.0;
        let s = (s.min(100) as f64) / 100.0;
        
        if s == 0.0 {
            let v = (l * 255.0) as u8;
            return Self::new(v, v, v);
        }
        
        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;
        
        let hk = h / 360.0;
        
        let tr = hk + 1.0 / 3.0;
        let tg = hk;
        let tb = hk - 1.0 / 3.0;
        
        fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
            if t < 0.0 { t += 1.0; }
            if t > 1.0 { t -= 1.0; }
            if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
            if t < 1.0 / 2.0 { return q; }
            if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
            p
        }
        
        Self::new(
            (hue_to_rgb(p, q, tr) * 255.0) as u8,
            (hue_to_rgb(p, q, tg) * 255.0) as u8,
            (hue_to_rgb(p, q, tb) * 255.0) as u8,
        )
    }
    
    /// Parse RGB color (percentage 0-100)
    pub fn from_rgb_percent(r: u16, g: u16, b: u16) -> Self {
        Self::new(
            ((r.min(100) as u32 * 255) / 100) as u8,
            ((g.min(100) as u32 * 255) / 100) as u8,
            ((b.min(100) as u32 * 255) / 100) as u8,
        )
    }
    
    /// Convert to RGBA u32
    pub fn to_rgba(&self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | 0xFF
    }
}

/// Sixel image representation
#[derive(Debug, Clone)]
pub struct SixelImage {
    /// Image width in pixels
    pub width: usize,
    /// Image height in pixels
    pub height: usize,
    /// Pixel data (RGBA, row-major)
    pub pixels: Vec<u32>,
    /// Whether the image is transparent
    pub transparent: bool,
}

impl SixelImage {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height],
            transparent: false,
        }
    }
    
    /// Set pixel at (x, y)
    pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = color;
        }
    }
    
    /// Get pixel at (x, y)
    pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x]
        } else {
            0
        }
    }
}

/// Sixel parser state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseState {
    Normal,
    ColorDef,
    Repeat,
    Raster,
}

/// Sixel parser
#[derive(Debug)]
pub struct SixelParser {
    /// Current parse state
    state: ParseState,
    /// Color palette (up to 256 colors)
    palette: HashMap<u16, SixelColor>,
    /// Current color index
    current_color: u16,
    /// Current X position
    x: usize,
    /// Current Y position (in sixel rows, each 6 pixels high)
    y: usize,
    /// Maximum X reached
    max_x: usize,
    /// Maximum Y reached
    max_y: usize,
    /// Repeat count
    repeat_count: usize,
    /// Numeric parameter accumulator
    params: Vec<u16>,
    /// Current parameter being parsed
    current_param: u16,
    /// Parsed image data (color index per pixel)
    image_data: Vec<Vec<u16>>,
    /// Pixel aspect ratio (horizontal : vertical)
    aspect_ratio: (u16, u16),
    /// Background color handling (0 = device default, 1 = no change, 2 = transparent)
    background_mode: u8,
}

impl Default for SixelParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SixelParser {
    pub fn new() -> Self {
        let mut palette = HashMap::new();
        
        // Initialize default VGA palette (16 colors)
        let default_colors = [
            (0, 0, 0),       // 0: Black
            (0, 0, 170),     // 1: Blue
            (170, 0, 0),     // 2: Red
            (170, 0, 170),   // 3: Magenta
            (0, 170, 0),     // 4: Green
            (0, 170, 170),   // 5: Cyan
            (170, 170, 0),   // 6: Yellow
            (170, 170, 170), // 7: White
            (85, 85, 85),    // 8: Bright Black
            (85, 85, 255),   // 9: Bright Blue
            (255, 85, 85),   // 10: Bright Red
            (255, 85, 255),  // 11: Bright Magenta
            (85, 255, 85),   // 12: Bright Green
            (85, 255, 255),  // 13: Bright Cyan
            (255, 255, 85),  // 14: Bright Yellow
            (255, 255, 255), // 15: Bright White
        ];
        
        for (i, (r, g, b)) in default_colors.iter().enumerate() {
            palette.insert(i as u16, SixelColor::new(*r, *g, *b));
        }
        
        Self {
            state: ParseState::Normal,
            palette,
            current_color: 0,
            x: 0,
            y: 0,
            max_x: 0,
            max_y: 0,
            repeat_count: 1,
            params: Vec::new(),
            current_param: 0,
            image_data: Vec::new(),
            aspect_ratio: (1, 1),
            background_mode: 0,
        }
    }
    
    /// Reset parser state for new image
    pub fn reset(&mut self) {
        self.state = ParseState::Normal;
        self.current_color = 0;
        self.x = 0;
        self.y = 0;
        self.max_x = 0;
        self.max_y = 0;
        self.repeat_count = 1;
        self.params.clear();
        self.current_param = 0;
        self.image_data.clear();
    }
    
    /// Parse sixel data and return image
    pub fn parse(&mut self, data: &[u8]) -> Option<SixelImage> {
        self.reset();
        
        let mut i = 0;
        while i < data.len() {
            let c = data[i];
            
            match self.state {
                ParseState::Normal => {
                    match c {
                        b'#' => {
                            self.state = ParseState::ColorDef;
                            self.params.clear();
                            self.current_param = 0;
                        }
                        b'!' => {
                            self.state = ParseState::Repeat;
                            self.repeat_count = 0;
                        }
                        b'"' => {
                            self.state = ParseState::Raster;
                            self.params.clear();
                            self.current_param = 0;
                        }
                        b'$' => {
                            // Carriage return - go to start of current sixel row
                            self.x = 0;
                        }
                        b'-' => {
                            // Line feed - move to next sixel row
                            self.x = 0;
                            self.y += 1;
                        }
                        0x3F..=0x7E => {
                            // Sixel data character
                            self.draw_sixel(c - 0x3F);
                        }
                        _ => {}
                    }
                }
                ParseState::ColorDef => {
                    match c {
                        b'0'..=b'9' => {
                            self.current_param = self.current_param * 10 + (c - b'0') as u16;
                        }
                        b';' => {
                            self.params.push(self.current_param);
                            self.current_param = 0;
                        }
                        _ => {
                            self.params.push(self.current_param);
                            self.process_color_def();
                            self.state = ParseState::Normal;
                            continue; // Reprocess this character
                        }
                    }
                }
                ParseState::Repeat => {
                    match c {
                        b'0'..=b'9' => {
                            self.repeat_count = self.repeat_count * 10 + (c - b'0') as usize;
                        }
                        0x3F..=0x7E => {
                            // Sixel data character with repeat
                            let count = self.repeat_count.max(1);
                            for _ in 0..count {
                                self.draw_sixel(c - 0x3F);
                            }
                            self.repeat_count = 1;
                            self.state = ParseState::Normal;
                        }
                        _ => {
                            self.state = ParseState::Normal;
                            continue;
                        }
                    }
                }
                ParseState::Raster => {
                    match c {
                        b'0'..=b'9' => {
                            self.current_param = self.current_param * 10 + (c - b'0') as u16;
                        }
                        b';' => {
                            self.params.push(self.current_param);
                            self.current_param = 0;
                        }
                        _ => {
                            self.params.push(self.current_param);
                            self.process_raster_attributes();
                            self.state = ParseState::Normal;
                            continue;
                        }
                    }
                }
            }
            
            i += 1;
        }
        
        self.build_image()
    }
    
    /// Process color definition: #Pc;Pu;Px;Py;Pz
    fn process_color_def(&mut self) {
        if self.params.is_empty() {
            return;
        }
        
        let color_index = self.params[0];
        
        if self.params.len() == 1 {
            // Just select color
            self.current_color = color_index;
        } else if self.params.len() >= 5 {
            // Define color
            let color_model = self.params[1];
            
            let color = match color_model {
                1 => {
                    // HLS
                    SixelColor::from_hls(self.params[2], self.params[3], self.params[4])
                }
                2 => {
                    // RGB (percentage)
                    SixelColor::from_rgb_percent(self.params[2], self.params[3], self.params[4])
                }
                _ => return,
            };
            
            self.palette.insert(color_index, color);
            self.current_color = color_index;
        }
    }
    
    /// Process raster attributes: "Pan;Pad;Ph;Pv
    fn process_raster_attributes(&mut self) {
        if self.params.len() >= 2 {
            self.aspect_ratio = (
                self.params[0].max(1),
                self.params[1].max(1),
            );
        }
        // Ph and Pv (image dimensions) are optional hints
    }
    
    /// Draw a sixel at current position
    fn draw_sixel(&mut self, sixel: u8) {
        // Ensure image data has enough rows
        let row_y = self.y * 6;
        while self.image_data.len() <= row_y + 5 {
            self.image_data.push(Vec::new());
        }
        
        // Draw 6 pixels vertically
        for bit in 0..6 {
            if (sixel >> bit) & 1 != 0 {
                let pixel_y = row_y + bit;
                
                // Ensure row has enough columns
                while self.image_data[pixel_y].len() <= self.x {
                    self.image_data[pixel_y].push(u16::MAX); // Transparent
                }
                
                self.image_data[pixel_y][self.x] = self.current_color;
            }
        }
        
        self.x += 1;
        self.max_x = self.max_x.max(self.x);
        self.max_y = self.max_y.max(self.y);
    }
    
    /// Build final image from parsed data
    fn build_image(&self) -> Option<SixelImage> {
        if self.image_data.is_empty() {
            return None;
        }
        
        let width = self.image_data.iter().map(|row| row.len()).max().unwrap_or(0);
        let height = self.image_data.len();
        
        if width == 0 || height == 0 {
            return None;
        }
        
        let mut image = SixelImage::new(width, height);
        image.transparent = self.background_mode == 2;
        
        for (y, row) in self.image_data.iter().enumerate() {
            for (x, &color_idx) in row.iter().enumerate() {
                if color_idx == u16::MAX {
                    // Transparent
                    image.set_pixel(x, y, 0);
                } else if let Some(color) = self.palette.get(&color_idx) {
                    image.set_pixel(x, y, color.to_rgba());
                }
            }
        }
        
        Some(image)
    }
}

/// Sixel encoder - converts images to sixel format
#[derive(Debug)]
pub struct SixelEncoder {
    /// Maximum colors to use
    pub max_colors: usize,
    /// Use run-length encoding
    pub use_rle: bool,
}

impl Default for SixelEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl SixelEncoder {
    pub fn new() -> Self {
        Self {
            max_colors: 256,
            use_rle: true,
        }
    }
    
    /// Encode RGBA image to sixel format
    pub fn encode(&self, width: usize, height: usize, pixels: &[u32]) -> Vec<u8> {
        if width == 0 || height == 0 || pixels.is_empty() {
            return Vec::new();
        }
        
        let mut output = Vec::new();
        
        // DCS (Device Control String) introducer
        output.extend_from_slice(b"\x1bP");
        
        // Sixel parameters: P1;P2;P3q
        // P1: pixel aspect ratio numerator
        // P2: background mode
        // P3: horizontal grid size
        output.extend_from_slice(b"0;0;0q");
        
        // Raster attributes
        output.extend_from_slice(format!("\"1;1;{};{}", width, height).as_bytes());
        
        // Build color palette from image
        let (palette, indexed) = self.quantize_colors(width, height, pixels);
        
        // Output color definitions
        for (i, color) in palette.iter().enumerate() {
            let r = ((color >> 24) & 0xFF) * 100 / 255;
            let g = ((color >> 16) & 0xFF) * 100 / 255;
            let b = ((color >> 8) & 0xFF) * 100 / 255;
            output.extend_from_slice(format!("#{};2;{};{};{}", i, r, g, b).as_bytes());
        }
        
        // Output sixel data row by row (6 pixels high)
        for sixel_row in 0..((height + 5) / 6) {
            let y_start = sixel_row * 6;
            
            // For each color, output the sixels
            for (color_idx, _) in palette.iter().enumerate() {
                let mut has_pixels = false;
                let mut sixels = Vec::new();
                
                for x in 0..width {
                    let mut sixel: u8 = 0;
                    
                    for bit in 0..6 {
                        let y = y_start + bit;
                        if y < height {
                            let pixel_idx = y * width + x;
                            if indexed[pixel_idx] == color_idx {
                                sixel |= 1 << bit;
                                has_pixels = true;
                            }
                        }
                    }
                    
                    sixels.push(sixel);
                }
                
                if has_pixels {
                    // Output color selection
                    output.extend_from_slice(format!("#{}", color_idx).as_bytes());
                    
                    // Output sixels with optional RLE
                    if self.use_rle {
                        self.encode_rle(&sixels, &mut output);
                    } else {
                        for &s in &sixels {
                            output.push(s + 0x3F);
                        }
                    }
                    
                    // Carriage return for same row
                    output.push(b'$');
                }
            }
            
            // Line feed to next sixel row
            if sixel_row < (height + 5) / 6 - 1 {
                output.push(b'-');
            }
        }
        
        // ST (String Terminator)
        output.extend_from_slice(b"\x1b\\");
        
        output
    }
    
    /// Quantize colors to palette
    fn quantize_colors(&self, width: usize, height: usize, pixels: &[u32]) -> (Vec<u32>, Vec<usize>) {
        // Simple median cut quantization
        let mut colors: HashMap<u32, usize> = HashMap::new();
        
        for &pixel in pixels {
            if (pixel & 0xFF) > 128 { // Alpha threshold
                *colors.entry(pixel | 0xFF).or_insert(0) += 1;
            }
        }
        
        // Get most common colors
        let mut color_list: Vec<_> = colors.into_iter().collect();
        color_list.sort_by(|a, b| b.1.cmp(&a.1));
        color_list.truncate(self.max_colors);
        
        let palette: Vec<u32> = color_list.iter().map(|(c, _)| *c).collect();
        
        // Map pixels to palette indices
        let indexed: Vec<usize> = pixels.iter().map(|&pixel| {
            if (pixel & 0xFF) <= 128 {
                0 // Transparent -> first color
            } else {
                // Find closest color
                palette.iter()
                    .enumerate()
                    .min_by_key(|(_, &c)| Self::color_distance(pixel, c))
                    .map(|(i, _)| i)
                    .unwrap_or(0)
            }
        }).collect();
        
        (palette, indexed)
    }
    
    /// Calculate color distance (squared)
    fn color_distance(c1: u32, c2: u32) -> u32 {
        let r1 = ((c1 >> 24) & 0xFF) as i32;
        let g1 = ((c1 >> 16) & 0xFF) as i32;
        let b1 = ((c1 >> 8) & 0xFF) as i32;
        let r2 = ((c2 >> 24) & 0xFF) as i32;
        let g2 = ((c2 >> 16) & 0xFF) as i32;
        let b2 = ((c2 >> 8) & 0xFF) as i32;
        
        let dr = r1 - r2;
        let dg = g1 - g2;
        let db = b1 - b2;
        
        (dr * dr + dg * dg + db * db) as u32
    }
    
    /// Encode sixels with run-length encoding
    fn encode_rle(&self, sixels: &[u8], output: &mut Vec<u8>) {
        let mut i = 0;
        while i < sixels.len() {
            let current = sixels[i];
            let mut count = 1;
            
            while i + count < sixels.len() && sixels[i + count] == current && count < 255 {
                count += 1;
            }
            
            if count >= 3 {
                // Use RLE
                output.extend_from_slice(format!("!{}", count).as_bytes());
                output.push(current + 0x3F);
            } else {
                for _ in 0..count {
                    output.push(current + 0x3F);
                }
            }
            
            i += count;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sixel_color_hls() {
        let color = SixelColor::from_hls(0, 50, 100);
        assert_eq!(color.r, 255);
        assert!(color.g < 10);
        assert!(color.b < 10);
    }
    
    #[test]
    fn test_sixel_color_rgb() {
        let color = SixelColor::from_rgb_percent(100, 50, 0);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 127);
        assert_eq!(color.b, 0);
    }
    
    #[test]
    fn test_sixel_parser_basic() {
        let mut parser = SixelParser::new();
        
        // Simple sixel: one white pixel
        let data = b"#7~?";
        let image = parser.parse(data);
        
        assert!(image.is_some());
    }
    
    #[test]
    fn test_sixel_encoder() {
        let encoder = SixelEncoder::new();
        
        // 2x2 red image
        let pixels = vec![0xFF0000FF; 4];
        let sixel = encoder.encode(2, 2, &pixels);
        
        assert!(!sixel.is_empty());
        assert!(sixel.starts_with(b"\x1bP"));
        assert!(sixel.ends_with(b"\x1b\\"));
    }
}


