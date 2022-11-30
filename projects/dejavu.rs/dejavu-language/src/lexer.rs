//! Template-mode scanner (outside delimiters) + helpers.

/// Trim mode encoded on a code-open tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrimMode {
    None,
    Ws,
    Nl,
    WsNl,
    All,
}

impl TrimMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Ws => "ws",
            Self::Nl => "nl",
            Self::WsNl => "ws_nl",
            Self::All => "all",
        }
    }

    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            b'.' => Some(Self::None),
            b'_' => Some(Self::Ws),
            b'-' => Some(Self::Nl),
            b'~' => Some(Self::WsNl),
            b'=' => Some(Self::All),
            _ => None,
        }
    }
}
