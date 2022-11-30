//! Template loop utilities for iteration with metadata
use core::iter::{Enumerate, Peekable};

/// A wrapper around an iterator that provides loop metadata
pub struct TemplateLoop<I>
where
    I: Iterator,
{
    iter: Peekable<Enumerate<I>>,
}

impl<I> TemplateLoop<I>
where
    I: Iterator,
{
    /// Create a new template loop from an iterator
    #[inline]
    pub fn new(iter: I) -> Self {
        TemplateLoop { iter: iter.enumerate().peekable() }
    }
}

impl<I> Iterator for TemplateLoop<I>
where
    I: Iterator,
{
    type Item = (<I as Iterator>::Item, LoopItem);

    #[inline]
    fn next(&mut self) -> Option<(<I as Iterator>::Item, LoopItem)> {
        self.iter.next().map(|(index, item)| (item, LoopItem { index, first: index == 0, last: self.iter.peek().is_none() }))
    }
}

/// Metadata about the current iteration in a template loop
#[derive(Copy, Clone)]
pub struct LoopItem {
    /// Current index (0-based)
    pub index: usize,
    /// Whether this is the first item
    pub first: bool,
    /// Whether this is the last item
    pub last: bool,
}
