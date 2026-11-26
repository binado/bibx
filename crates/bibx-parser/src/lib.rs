//! A lossless BibTeX parser that records where everything is.
//!
//! [`parse`] splits the input into contiguous [`Item`]s whose spans cover every byte, so
//! concatenating the items' source text gives back the input exactly. Entries keep their
//! fields in source order with raw values (braces and LaTeX intact). Nothing is interpreted:
//! no `@string` expansion, no LaTeX decoding, no name parsing.
//!
//! Parsing never fails and never panics. A malformed entry becomes an [`Item::Error`] that
//! runs up to the next `@` that starts a line, and parsing resumes there.
//!
//! # Differences from `biblatex`
//!
//! The `biblatex` crate's raw layer is the reference in this crate's tests. We deliberately
//! differ in three ways:
//!
//! - Braces are counted the way BibTeX counts them: a backslash does not escape `{`, `}` or
//!   `"`. (`biblatex` skips the character after a backslash.)
//! - `@comment{...}` bodies may contain nested braces; they are skipped as a balanced block.
//! - Errors are recovered from (see above) instead of aborting the whole parse.

mod error;
mod parse;
mod span;

pub use error::{ParseError, ParseErrorKind};
pub use parse::parse;
pub use span::{Span, Spanned};

/// A parsed file: items in source order, covering the input without gaps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bib<'s> {
    pub items: Vec<Item<'s>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item<'s> {
    Entry(Entry<'s>),
    /// `@string{name = value}`: same shape as an entry, with an empty key; the fields are
    /// the definitions.
    String(Entry<'s>),
    Preamble(Span),
    /// `@comment` and any text outside entries.
    Comment(Span),
    /// A malformed entry, from its `@` to the start of the next line that begins with `@`.
    Error {
        span: Span,
        error: ParseError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'s> {
    /// From `@` to the closing delimiter, inclusive.
    pub span: Span,
    pub kind: Spanned<&'s str>,
    pub key: Spanned<&'s str>,
    pub fields: Vec<Field<'s>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field<'s> {
    pub span: Span,
    pub name: Spanned<&'s str>,
    /// The parts joined by `#`.
    pub value: Vec<Spanned<ValuePart<'s>>>,
}

/// One part of a field value. The span of the [`Spanned`] wrapper includes the delimiters;
/// the payload is the text inside them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValuePart<'s> {
    Braced(&'s str),
    Quoted(&'s str),
    Number(&'s str),
    Macro(&'s str),
}

impl Item<'_> {
    pub fn span(&self) -> Span {
        match self {
            Item::Entry(e) | Item::String(e) => e.span,
            Item::Preamble(span) | Item::Comment(span) => *span,
            Item::Error { span, .. } => *span,
        }
    }
}

impl<'s> Bib<'s> {
    /// The regular entries (not `@string`), in source order.
    pub fn entries(&self) -> impl Iterator<Item = &Entry<'s>> {
        self.items.iter().filter_map(|item| match item {
            Item::Entry(e) => Some(e),
            _ => None,
        })
    }

    /// The malformed items, in source order.
    pub fn errors(&self) -> impl Iterator<Item = (Span, &ParseError)> {
        self.items.iter().filter_map(|item| match item {
            Item::Error { span, error } => Some((*span, error)),
            _ => None,
        })
    }
}

impl<'s> Entry<'s> {
    /// The first field called `name`, compared case-insensitively.
    pub fn field(&self, name: &str) -> Option<&Field<'s>> {
        self.fields
            .iter()
            .find(|f| f.name.value.eq_ignore_ascii_case(name))
    }
}

impl Field<'_> {
    /// The value's source text, delimiters and `#` included.
    ///
    /// # Panics
    ///
    /// As [`Span::text`], when `src` is not the text that was parsed.
    pub fn raw<'a>(&self, src: &'a str) -> &'a str {
        match (self.value.first(), self.value.last()) {
            (Some(first), Some(last)) => Span::new(first.span.start, last.span.end).text(src),
            _ => "",
        }
    }

    /// A single braced or quoted value without its delimiters; anything else as raw text.
    ///
    /// # Panics
    ///
    /// As [`Span::text`], when `src` is not the text that was parsed.
    pub fn text<'a>(&self, src: &'a str) -> &'a str {
        match self.value.as_slice() {
            [
                Spanned {
                    value: ValuePart::Braced(_) | ValuePart::Quoted(_),
                    span,
                },
            ] => {
                let delimited = span.text(src);
                &delimited[1..delimited.len() - 1]
            }
            _ => self.raw(src),
        }
    }
}
