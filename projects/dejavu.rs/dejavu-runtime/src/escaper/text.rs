//! Text escaper implementation
use super::*;

/// Text escaper that does not escape any characters
pub struct Text;

impl Escaper for Text {
    fn write_escaped<W>(&self, mut fmt: W, string: &str) -> core::fmt::Result
    where
        W: Write,
    {
        fmt.write_str(string)
    }
}
