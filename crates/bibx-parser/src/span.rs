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

    /// The source text of this span, or `None` when the range is reversed, runs past the
    /// end of `src` or splits a character. Use this for spans you built yourself.
    pub fn get(self, src: &str) -> Option<&str> {
        src.get(self.range())
    }

    /// The source text of a span that [`parse`](crate::parse) took from `src`.
    ///
    /// # Panics
    ///
    /// When [`get`](Span::get) would return `None`. For a parser span that means `src` is not
    /// the text that was parsed. A different source that the range happens to fit is not
    /// detected.
    pub fn text(self, src: &str) -> &str {
        &src[self.range()]
    }
}

/// A value together with the span it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_tells_a_misfit_from_an_empty_span() {
        assert_eq!(Span::new(1, 1).get("ab"), Some(""));
        assert_eq!(Span::new(2, 2).get("ab"), Some(""));
        assert_eq!(Span::new(1, 3).get("ab"), None);
        assert_eq!(Span::new(2, 1).get("ab"), None);
        assert_eq!(Span::new(0, 1).get("é"), None);
    }
}
