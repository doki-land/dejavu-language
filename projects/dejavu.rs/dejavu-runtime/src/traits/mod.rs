//! Template trait for rendering templates
use alloc::string::String;
use core::fmt::Write;

/// A trait representing a renderable template
pub trait Template {
    /// File extension for this template type
    const EXTENSION: &'static str;
    /// MIME type for this template type
    const MIME_TYPE: &'static str;
    /// Hint for the expected size of the rendered output
    const SIZE_HINT: usize;

    /// Write the rendered template to a writer
    fn write_fmt<W>(&self, w: &mut W) -> core::fmt::Result
    where
        W: Write + ?Sized;

    /// Render the template to a string
    fn render(&self) -> String {
        let mut out = String::with_capacity(Self::SIZE_HINT);
        self.write_fmt(&mut out).unwrap();
        out
    }
}
