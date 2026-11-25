use std::ops::Range;

/// A half-open byte range `start..end` into the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    pub fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(self) -> bool {
        self.end <= self.start
    }

    pub fn range(self) -> Range<usize> {
        self.start..self.end
    }

    /// The source text of this span, or `""` when it does not fit `src`.
    pub fn text(self, src: &str) -> &str {
        src.get(self.range()).unwrap_or("")
    }
}

/// A value together with the span it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}
