//! HTML escaper implementation
use super::*;

/// HTML escaper that escapes HTML special characters
pub struct Html;

impl Escaper for Html {
    fn write_escaped<W>(&self, mut fmt: W, string: &str) -> core::fmt::Result
    where
        W: Write,
    {
        let mut last = 0;
        for (index, byte) in string.bytes().enumerate() {
            const MIN_CHAR: u8 = b'"';
            const MAX_CHAR: u8 = b'>';
            const TABLE: [Option<&&str>; (MAX_CHAR - MIN_CHAR + 1) as usize] = {
                let mut table = [None; (MAX_CHAR - MIN_CHAR + 1) as usize];
                table[(b'<' - MIN_CHAR) as usize] = Some(&"&lt;");
                table[(b'>' - MIN_CHAR) as usize] = Some(&"&gt;");
                table[(b'&' - MIN_CHAR) as usize] = Some(&"&amp;");
                table[(b'"' - MIN_CHAR) as usize] = Some(&"&quot;");
                table[(b'\'' - MIN_CHAR) as usize] = Some(&"&#x27;");
                table
            };

            let escaped = match byte {
                MIN_CHAR..=MAX_CHAR => TABLE[(byte - MIN_CHAR) as usize],
                _ => None,
            };
            if let Some(escaped) = escaped {
                fmt.write_str(&string[last..index])?;
                fmt.write_str(escaped)?;
                last = index + 1;
            }
        }
        fmt.write_str(&string[last..])
    }
}
