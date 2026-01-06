//! ANSI escape sequence parser
//!
//! Parses VT100/VT220/ANSI escape sequences

/// Parser state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Ground,
    Escape,
    EscapeIntermediate,
    CsiEntry,
    CsiParam,
    CsiIntermediate,
    OscString,
}

/// Parsed ANSI event
#[derive(Debug, Clone)]
pub enum AnsiEvent {
    /// Printable character
    Print(char),
    /// Control character (C0/C1)
    Execute(u8),
    /// CSI sequence
    CsiDispatch {
        params: Vec<u16>,
        intermediates: Vec<u8>,
        action: u8,
    },
    /// ESC sequence
    EscDispatch {
        intermediates: Vec<u8>,
        action: u8,
    },
    /// OSC sequence
    OscDispatch {
        params: Vec<Vec<u8>>,
    },
}

/// ANSI escape sequence parser
pub struct AnsiParser {
    state: State,
    intermediates: Vec<u8>,
    params: Vec<u16>,
    current_param: u16,
    osc_data: Vec<Vec<u8>>,
    osc_current: Vec<u8>,
}

impl AnsiParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            state: State::Ground,
            intermediates: Vec::new(),
            params: Vec::new(),
            current_param: 0,
            osc_data: Vec::new(),
            osc_current: Vec::new(),
        }
    }

    /// Reset parser state
    fn reset(&mut self) {
        self.state = State::Ground;
        self.intermediates.clear();
        self.params.clear();
        self.current_param = 0;
        self.osc_data.clear();
        self.osc_current.clear();
    }

    /// Parse bytes and return events
    pub fn parse(&mut self, data: &[u8]) -> Vec<AnsiEvent> {
        let mut events = Vec::new();
        
        for &byte in data {
            if let Some(event) = self.advance(byte) {
                events.push(event);
            }
        }
        
        events
    }

    /// Advance parser with single byte
    fn advance(&mut self, byte: u8) -> Option<AnsiEvent> {
        match self.state {
            State::Ground => self.ground(byte),
            State::Escape => self.escape(byte),
            State::EscapeIntermediate => self.escape_intermediate(byte),
            State::CsiEntry => self.csi_entry(byte),
            State::CsiParam => self.csi_param(byte),
            State::CsiIntermediate => self.csi_intermediate(byte),
            State::OscString => self.osc_string(byte),
        }
    }

    fn ground(&mut self, byte: u8) -> Option<AnsiEvent> {
        match byte {
            // C0 control characters
            0x00..=0x1A | 0x1C..=0x1F => Some(AnsiEvent::Execute(byte)),
            // ESC
            0x1B => {
                self.state = State::Escape;
                None
            }
            // Printable ASCII
            0x20..=0x7E => {
                Some(AnsiEvent::Print(byte as char))
            }
            // DEL - ignore
            0x7F => None,
            // UTF-8 continuation or start
            0x80..=0xFF => {
                // Simple UTF-8 handling - treat as printable
                // TODO: Proper UTF-8 decoding
                Some(AnsiEvent::Print(byte as char))
            }
        }
    }

    fn escape(&mut self, byte: u8) -> Option<AnsiEvent> {
        match byte {
            // Intermediate bytes
            0x20..=0x2F => {
                self.intermediates.push(byte);
                self.state = State::EscapeIntermediate;
                None
            }
            // CSI
            b'[' => {
                self.state = State::CsiEntry;
                None
            }
            // OSC
            b']' => {
                self.state = State::OscString;
                None
            }
            // SS2, SS3
            b'N' | b'O' => {
                self.reset();
                None
            }
            // Final bytes
            0x30..=0x7E => {
                let event = AnsiEvent::EscDispatch {
                    intermediates: std::mem::take(&mut self.intermediates),
                    action: byte,
                };
                self.reset();
                Some(event)
            }
            // C0 in escape - execute and continue
            0x00..=0x1A | 0x1C..=0x1F => {
                Some(AnsiEvent::Execute(byte))
            }
            // ESC again - re-enter
            0x1B => None,
            _ => {
                self.reset();
                None
            }
        }
    }

    fn escape_intermediate(&mut self, byte: u8) -> Option<AnsiEvent> {
        match byte {
            // More intermediates
            0x20..=0x2F => {
                self.intermediates.push(byte);
                None
            }
            // Final byte
            0x30..=0x7E => {
                let event = AnsiEvent::EscDispatch {
                    intermediates: std::mem::take(&mut self.intermediates),
                    action: byte,
                };
                self.reset();
                Some(event)
            }
            // C0 - execute and continue
            0x00..=0x1A | 0x1C..=0x1F => {
                Some(AnsiEvent::Execute(byte))
            }
            // ESC - restart
            0x1B => {
                self.intermediates.clear();
                self.state = State::Escape;
                None
            }
            _ => {
                self.reset();
                None
            }
        }
    }

    fn csi_entry(&mut self, byte: u8) -> Option<AnsiEvent> {
        match byte {
            // Parameter bytes
            0x30..=0x39 => {
                self.current_param = (byte - b'0') as u16;
                self.state = State::CsiParam;
                None
            }
            // ; - parameter separator (empty param)
            b';' => {
                self.params.push(0);
                self.state = State::CsiParam;
                None
            }
            // : - subparameter separator (treat like ;)
            b':' => {
                self.state = State::CsiParam;
                None
            }
            // Intermediate
            0x20..=0x2F => {
                self.intermediates.push(byte);
                self.state = State::CsiIntermediate;
                None
            }
            // Private marker
            b'?' | b'>' | b'<' | b'=' => {
                self.intermediates.push(byte);
                None
            }
            // Final byte
            0x40..=0x7E => {
                let event = AnsiEvent::CsiDispatch {
                    params: std::mem::take(&mut self.params),
                    intermediates: std::mem::take(&mut self.intermediates),
                    action: byte,
                };
                self.reset();
                Some(event)
            }
            // C0 - execute and continue
            0x00..=0x1A | 0x1C..=0x1F => {
                Some(AnsiEvent::Execute(byte))
            }
            // ESC - restart
            0x1B => {
                self.reset();
                self.state = State::Escape;
                None
            }
            _ => {
                self.reset();
                None
            }
        }
    }

    fn csi_param(&mut self, byte: u8) -> Option<AnsiEvent> {
        match byte {
            // More digits
            0x30..=0x39 => {
                self.current_param = self.current_param
                    .saturating_mul(10)
                    .saturating_add((byte - b'0') as u16);
                None
            }
            // ; - parameter separator
            b';' => {
                self.params.push(self.current_param);
                self.current_param = 0;
                None
            }
            // : - subparameter separator (for SGR colors)
            b':' => {
                self.params.push(self.current_param);
                self.current_param = 0;
                None
            }
            // Intermediate
            0x20..=0x2F => {
                self.params.push(self.current_param);
                self.current_param = 0;
                self.intermediates.push(byte);
                self.state = State::CsiIntermediate;
                None
            }
            // Final byte
            0x40..=0x7E => {
                self.params.push(self.current_param);
                let event = AnsiEvent::CsiDispatch {
                    params: std::mem::take(&mut self.params),
                    intermediates: std::mem::take(&mut self.intermediates),
                    action: byte,
                };
                self.reset();
                Some(event)
            }
            // C0 - execute and continue
            0x00..=0x1A | 0x1C..=0x1F => {
                Some(AnsiEvent::Execute(byte))
            }
            // ESC - restart
            0x1B => {
                self.reset();
                self.state = State::Escape;
                None
            }
            _ => {
                self.reset();
                None
            }
        }
    }

    fn csi_intermediate(&mut self, byte: u8) -> Option<AnsiEvent> {
        match byte {
            // More intermediates
            0x20..=0x2F => {
                self.intermediates.push(byte);
                None
            }
            // Final byte
            0x40..=0x7E => {
                let event = AnsiEvent::CsiDispatch {
                    params: std::mem::take(&mut self.params),
                    intermediates: std::mem::take(&mut self.intermediates),
                    action: byte,
                };
                self.reset();
                Some(event)
            }
            // C0 - execute and continue
            0x00..=0x1A | 0x1C..=0x1F => {
                Some(AnsiEvent::Execute(byte))
            }
            // ESC - restart
            0x1B => {
                self.reset();
                self.state = State::Escape;
                None
            }
            _ => {
                self.reset();
                None
            }
        }
    }

    fn osc_string(&mut self, byte: u8) -> Option<AnsiEvent> {
        match byte {
            // BEL terminates OSC
            0x07 => {
                if !self.osc_current.is_empty() {
                    self.osc_data.push(std::mem::take(&mut self.osc_current));
                }
                let event = AnsiEvent::OscDispatch {
                    params: std::mem::take(&mut self.osc_data),
                };
                self.reset();
                Some(event)
            }
            // ESC might be ST
            0x1B => {
                // Need to check for \ next
                // For now, treat as terminator
                if !self.osc_current.is_empty() {
                    self.osc_data.push(std::mem::take(&mut self.osc_current));
                }
                let event = AnsiEvent::OscDispatch {
                    params: std::mem::take(&mut self.osc_data),
                };
                self.reset();
                self.state = State::Escape;
                Some(event)
            }
            // ; - parameter separator
            b';' => {
                self.osc_data.push(std::mem::take(&mut self.osc_current));
                None
            }
            // Other printable
            0x20..=0x7E | 0x80..=0xFF => {
                self.osc_current.push(byte);
                None
            }
            // Ignore C0 except BEL
            _ => None,
        }
    }
}

impl Default for AnsiParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text() {
        let mut parser = AnsiParser::new();
        let events = parser.parse(b"Hello");
        assert_eq!(events.len(), 5);
        for (i, event) in events.iter().enumerate() {
            match event {
                AnsiEvent::Print(c) => {
                    assert_eq!(*c, "Hello".chars().nth(i).unwrap());
                }
                _ => panic!("Expected Print event"),
            }
        }
    }

    #[test]
    fn test_parse_csi_cursor_up() {
        let mut parser = AnsiParser::new();
        let events = parser.parse(b"\x1b[5A");
        assert_eq!(events.len(), 1);
        match &events[0] {
            AnsiEvent::CsiDispatch { params, action, .. } => {
                assert_eq!(params, &[5]);
                assert_eq!(*action, b'A');
            }
            _ => panic!("Expected CSI dispatch"),
        }
    }

    #[test]
    fn test_parse_sgr() {
        let mut parser = AnsiParser::new();
        let events = parser.parse(b"\x1b[1;31m");
        assert_eq!(events.len(), 1);
        match &events[0] {
            AnsiEvent::CsiDispatch { params, action, .. } => {
                assert_eq!(params, &[1, 31]);
                assert_eq!(*action, b'm');
            }
            _ => panic!("Expected CSI dispatch"),
        }
    }
}





