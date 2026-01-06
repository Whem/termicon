//! ANSI Escape Code Parser for Terminal Display
//!
//! Parses ANSI escape sequences and converts them to styled text spans

use eframe::egui::Color32;

/// Text style attributes
#[derive(Debug, Clone, Default)]
pub struct AnsiStyle {
    pub fg_color: Option<Color32>,
    pub bg_color: Option<Color32>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub dim: bool,
    pub blink: bool,
    pub reverse: bool,
    pub strikethrough: bool,
}

impl AnsiStyle {
    /// Get foreground color with default
    pub fn get_fg(&self) -> Color32 {
        if self.dim {
            self.fg_color.unwrap_or(Color32::from_rgb(140, 140, 140))
        } else {
            self.fg_color.unwrap_or(Color32::from_rgb(200, 200, 200))
        }
    }

    /// Get background color with default
    pub fn get_bg(&self) -> Color32 {
        self.bg_color.unwrap_or(Color32::TRANSPARENT)
    }
}

/// A styled text span
#[derive(Debug, Clone)]
pub struct StyledSpan {
    pub text: String,
    pub style: AnsiStyle,
}

/// Standard ANSI colors
fn ansi_color(code: u8, bold: bool) -> Color32 {
    match code {
        0 => if bold { Color32::from_rgb(128, 128, 128) } else { Color32::from_rgb(0, 0, 0) },        // Black
        1 => if bold { Color32::from_rgb(255, 85, 85) } else { Color32::from_rgb(170, 0, 0) },       // Red
        2 => if bold { Color32::from_rgb(85, 255, 85) } else { Color32::from_rgb(0, 170, 0) },       // Green
        3 => if bold { Color32::from_rgb(255, 255, 85) } else { Color32::from_rgb(170, 170, 0) },    // Yellow
        4 => if bold { Color32::from_rgb(85, 85, 255) } else { Color32::from_rgb(0, 0, 170) },       // Blue
        5 => if bold { Color32::from_rgb(255, 85, 255) } else { Color32::from_rgb(170, 0, 170) },    // Magenta
        6 => if bold { Color32::from_rgb(85, 255, 255) } else { Color32::from_rgb(0, 170, 170) },    // Cyan
        7 => if bold { Color32::from_rgb(255, 255, 255) } else { Color32::from_rgb(170, 170, 170) }, // White
        _ => Color32::from_rgb(200, 200, 200),
    }
}

/// 256-color palette
fn color_256(code: u8) -> Color32 {
    if code < 8 {
        ansi_color(code, false)
    } else if code < 16 {
        ansi_color(code - 8, true)
    } else if code < 232 {
        // 216 color cube (6x6x6)
        let code = code - 16;
        let r = (code / 36) % 6;
        let g = (code / 6) % 6;
        let b = code % 6;
        let to_256 = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
        Color32::from_rgb(to_256(r), to_256(g), to_256(b))
    } else {
        // 24 grayscale
        let gray = 8 + (code - 232) * 10;
        Color32::from_rgb(gray, gray, gray)
    }
}

/// Parse ANSI escape sequences and return styled spans
pub fn parse_ansi(text: &str) -> Vec<StyledSpan> {
    let mut spans = Vec::new();
    let mut current_style = AnsiStyle::default();
    let mut current_text = String::new();
    let mut chars = text.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Start of escape sequence
            if let Some(&'[') = chars.peek() {
                chars.next(); // consume '['
                
                // Save current text
                if !current_text.is_empty() {
                    spans.push(StyledSpan {
                        text: current_text.clone(),
                        style: current_style.clone(),
                    });
                    current_text.clear();
                }
                
                // Check for DEC private mode (?)
                let is_private = if let Some(&'?') = chars.peek() {
                    chars.next();
                    true
                } else {
                    false
                };
                
                // Parse CSI sequence parameters
                let mut params = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == ';' || c == ':' {
                        params.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                
                // Get the final character(s)
                if let Some(cmd) = chars.next() {
                    if is_private {
                        // DEC private mode sequences - ignore completely
                        // Examples: ?2004h (bracketed paste), ?1h (cursor keys), ?25h (show cursor)
                        continue;
                    }
                    
                    match cmd {
                        'm' => {
                            // SGR (Select Graphic Rendition)
                            apply_sgr(&params, &mut current_style);
                        }
                        'H' | 'f' => {
                            // Cursor position - ignore for display
                        }
                        'J' | 'K' => {
                            // Erase - ignore for display
                        }
                        'A' | 'B' | 'C' | 'D' => {
                            // Cursor movement - ignore
                        }
                        'h' | 'l' => {
                            // Set/reset mode - ignore
                        }
                        'r' => {
                            // Set scrolling region - ignore
                        }
                        '@' | 'P' | 'X' | 'L' | 'M' => {
                            // Insert/delete chars/lines - ignore
                        }
                        's' | 'u' => {
                            // Save/restore cursor - ignore
                        }
                        _ => {
                            // Unknown escape sequence, ignore
                        }
                    }
                }
            } else if let Some(&']') = chars.peek() {
                // OSC sequence - skip until ST or BEL
                chars.next();
                while let Some(c) = chars.next() {
                    if c == '\x07' || c == '\x1b' {
                        if c == '\x1b' {
                            chars.next(); // consume '\'
                        }
                        break;
                    }
                }
            } else if matches!(chars.peek(), Some(&'(') | Some(&')')) {
                // Character set selection - skip next char
                chars.next();
                chars.next();
            } else if matches!(chars.peek(), Some(&'>') | Some(&'=')) {
                // Keypad mode - skip
                chars.next();
            }
        } else if c == '\r' {
            // Carriage return - ignore for display
            continue;
        } else if c == '\n' {
            // Newline - preserve
            current_text.push(c);
        } else if c.is_control() && c != '\t' {
            // Skip other control characters
            continue;
        } else {
            current_text.push(c);
        }
    }
    
    // Add remaining text
    if !current_text.is_empty() {
        spans.push(StyledSpan {
            text: current_text,
            style: current_style,
        });
    }
    
    spans
}

/// Apply SGR (Select Graphic Rendition) parameters
fn apply_sgr(params: &str, style: &mut AnsiStyle) {
    if params.is_empty() {
        *style = AnsiStyle::default();
        return;
    }
    
    let codes: Vec<u8> = params
        .split(|c| c == ';' || c == ':')
        .filter_map(|s| s.parse().ok())
        .collect();
    
    let mut i = 0;
    while i < codes.len() {
        match codes[i] {
            0 => *style = AnsiStyle::default(),
            1 => style.bold = true,
            2 => style.dim = true,
            3 => style.italic = true,
            4 => style.underline = true,
            5 | 6 => style.blink = true,
            7 => style.reverse = true,
            9 => style.strikethrough = true,
            21 => style.bold = false,
            22 => { style.bold = false; style.dim = false; }
            23 => style.italic = false,
            24 => style.underline = false,
            25 => style.blink = false,
            27 => style.reverse = false,
            29 => style.strikethrough = false,
            
            // Foreground colors
            30..=37 => style.fg_color = Some(ansi_color(codes[i] - 30, style.bold)),
            38 => {
                // Extended foreground color
                if i + 1 < codes.len() {
                    match codes[i + 1] {
                        5 if i + 2 < codes.len() => {
                            style.fg_color = Some(color_256(codes[i + 2]));
                            i += 2;
                        }
                        2 if i + 4 < codes.len() => {
                            style.fg_color = Some(Color32::from_rgb(
                                codes[i + 2],
                                codes[i + 3],
                                codes[i + 4],
                            ));
                            i += 4;
                        }
                        _ => i += 1,
                    }
                }
            }
            39 => style.fg_color = None,
            
            // Background colors
            40..=47 => style.bg_color = Some(ansi_color(codes[i] - 40, false)),
            48 => {
                // Extended background color
                if i + 1 < codes.len() {
                    match codes[i + 1] {
                        5 if i + 2 < codes.len() => {
                            style.bg_color = Some(color_256(codes[i + 2]));
                            i += 2;
                        }
                        2 if i + 4 < codes.len() => {
                            style.bg_color = Some(Color32::from_rgb(
                                codes[i + 2],
                                codes[i + 3],
                                codes[i + 4],
                            ));
                            i += 4;
                        }
                        _ => i += 1,
                    }
                }
            }
            49 => style.bg_color = None,
            
            // Bright foreground colors
            90..=97 => style.fg_color = Some(ansi_color(codes[i] - 90, true)),
            
            // Bright background colors
            100..=107 => style.bg_color = Some(ansi_color(codes[i] - 100, true)),
            
            _ => {}
        }
        i += 1;
    }
}

/// Strip ANSI codes from text
pub fn strip_ansi(text: &str) -> String {
    let spans = parse_ansi(text);
    spans.into_iter().map(|s| s.text).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let text = "Hello \x1b[32mGreen\x1b[0m World";
        let spans = parse_ansi(text);
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].text, "Hello ");
        assert_eq!(spans[1].text, "Green");
        assert!(spans[1].style.fg_color.is_some());
        assert_eq!(spans[2].text, " World");
    }

    #[test]
    fn test_strip_ansi() {
        let text = "\x1b[01;32muser\x1b[0m@\x1b[01;34mhost\x1b[0m";
        let stripped = strip_ansi(text);
        assert_eq!(stripped, "user@host");
    }
}

