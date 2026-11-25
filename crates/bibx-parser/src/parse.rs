use crate::{Bib, Entry, Field, Item, ParseError, ParseErrorKind, Span, Spanned, ValuePart};

/// Parse `src` into items that cover it without gaps. Never fails and never panics.
pub fn parse(src: &str) -> Bib<'_> {
    let mut items = Vec::new();
    let mut cursor = Cursor::new(src);
    while cursor.pos < src.len() {
        let start = cursor.pos;
        cursor.skip_to_item();
        if cursor.pos > start {
            items.push(Item::Comment(Span::new(start, cursor.pos)));
        }
        if cursor.pos == src.len() {
            break;
        }
        let start = cursor.pos;
        match cursor.item() {
            Ok(item) => items.push(item),
            Err(error) => {
                let end = cursor.recovery_point(start);
                items.push(Item::Error {
                    span: Span::new(start, end),
                    error,
                });
                cursor.pos = end;
            }
        }
    }
    Bib { items }
}

struct Cursor<'s> {
    src: &'s str,
    bytes: &'s [u8],
    pos: usize,
}

type Result<T> = std::result::Result<T, ParseError>;

fn closing(open: u8) -> u8 {
    if open == b'(' { b')' } else { b'}' }
}

fn is_ident_stop(b: u8) -> bool {
    b.is_ascii_whitespace()
        || matches!(
            b,
            b'{' | b'}' | b'(' | b')' | b'"' | b'=' | b',' | b'#' | b'%'
        )
}

impl<'s> Cursor<'s> {
    fn new(src: &'s str) -> Self {
        Cursor {
            src,
            bytes: src.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn find_byte(&self, needle: u8) -> Option<usize> {
        let rest = self.bytes.get(self.pos..)?;
        rest.iter().position(|&b| b == needle).map(|i| self.pos + i)
    }

    fn text(&self, start: usize, end: usize) -> &'s str {
        self.src.get(start..end).unwrap_or("")
    }

    fn skip_to_item(&mut self) {
        while let Some(b) = self.peek() {
            match b {
                b'@' => break,
                b'%' => self.skip_line_comment(),
                _ => self.pos += 1,
            }
        }
    }

    fn skip_line_comment(&mut self) {
        self.pos = self.find_byte(b'\n').unwrap_or(self.bytes.len());
    }

    /// Skip whitespace and `%` comments (to the end of the line).
    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            if b == b'%' {
                self.skip_line_comment();
            } else if b.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn take_ident(&mut self) -> Spanned<&'s str> {
        let start = self.pos;
        while self.peek().is_some_and(|b| !is_ident_stop(b)) {
            self.pos += 1;
        }
        Spanned {
            span: Span::new(start, self.pos),
            value: self.text(start, self.pos),
        }
    }

    fn error<T>(&self, kind: ParseErrorKind, message: &str) -> Result<T> {
        Err(ParseError {
            kind,
            message: message.to_string(),
            offset: self.pos,
        })
    }

    /// Start of the next line that begins with `@` (after optional blanks), searching from
    /// the line after `start`; the end of input when there is none.
    fn recovery_point(&self, start: usize) -> usize {
        let mut line = match self
            .bytes
            .get(start..)
            .and_then(|r| r.iter().position(|&b| b == b'\n'))
        {
            Some(i) => start + i + 1,
            None => return self.bytes.len(),
        };
        while line < self.bytes.len() {
            let rest = self.bytes.get(line..).unwrap_or_default();
            let first = rest.iter().find(|&&b| b != b' ' && b != b'\t');
            if first == Some(&b'@') {
                return line;
            }
            match rest.iter().position(|&b| b == b'\n') {
                Some(i) => line += i + 1,
                None => break,
            }
        }
        self.bytes.len()
    }

    /// At an `@`: parse one item.
    fn item(&mut self) -> Result<Item<'s>> {
        let start = self.pos;
        self.pos += 1;
        self.skip_ws();
        let kind = self.take_ident();
        if kind.value.is_empty() {
            return self.error(
                ParseErrorKind::ExpectedType,
                "expected an entry type after `@`",
            );
        }
        let lower = kind.value.to_ascii_lowercase();
        if lower == "comment" {
            return self.comment(start);
        }
        self.skip_ws();
        let open = match self.peek() {
            Some(b @ (b'{' | b'(')) => b,
            _ => {
                return self.error(
                    ParseErrorKind::ExpectedDelimiter,
                    "expected `{` or `(` after the entry type",
                );
            }
        };
        self.pos += 1;
        let close = closing(open);
        match lower.as_str() {
            "preamble" => {
                self.skip_ws();
                self.value()?;
                self.skip_ws();
                self.expect_close(close)?;
                Ok(Item::Preamble(Span::new(start, self.pos)))
            }
            "string" => {
                let empty = Spanned {
                    span: Span::new(self.pos, self.pos),
                    value: "",
                };
                let fields = self.fields(close)?;
                Ok(Item::String(Entry {
                    span: Span::new(start, self.pos),
                    kind,
                    key: empty,
                    fields,
                }))
            }
            _ => {
                self.skip_ws();
                let key = self.take_key(close);
                self.skip_ws();
                let fields = match self.peek() {
                    Some(b',') => {
                        self.pos += 1;
                        self.fields(close)?
                    }
                    _ => {
                        self.expect_close(close)?;
                        Vec::new()
                    }
                };
                Ok(Item::Entry(Entry {
                    span: Span::new(start, self.pos),
                    kind,
                    key,
                    fields,
                }))
            }
        }
    }

    fn take_key(&mut self, close: u8) -> Spanned<&'s str> {
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|b| b != b',' && b != close && !b.is_ascii_whitespace())
        {
            self.pos += 1;
        }
        Spanned {
            span: Span::new(start, self.pos),
            value: self.text(start, self.pos),
        }
    }

    fn expect_close(&mut self, close: u8) -> Result<()> {
        if self.peek() == Some(close) {
            self.pos += 1;
            Ok(())
        } else if self.peek().is_none() {
            self.error(
                ParseErrorKind::Unterminated,
                "entry is not closed before end of file",
            )
        } else {
            self.error(
                ParseErrorKind::ExpectedSeparator,
                "expected `,` or the closing delimiter",
            )
        }
    }

    /// `@comment`: a balanced block when a delimiter follows, otherwise just the keyword.
    fn comment(&mut self, start: usize) -> Result<Item<'s>> {
        let after_kw = self.pos;
        self.skip_ws();
        match self.peek() {
            Some(b'{') => {
                self.balanced()?;
            }
            Some(b'(') => match self.find_byte(b')') {
                Some(i) => self.pos = i + 1,
                None => {
                    self.pos = self.bytes.len();
                    return self.error(
                        ParseErrorKind::Unterminated,
                        "`@comment(` is not closed before end of file",
                    );
                }
            },
            _ => self.pos = after_kw,
        }
        Ok(Item::Comment(Span::new(start, self.pos)))
    }

    /// At `{`: skip to just after the matching `}`. Returns the inner span.
    fn balanced(&mut self) -> Result<Span> {
        let open = self.pos;
        let mut depth = 0usize;
        while let Some(b) = self.peek() {
            self.pos += 1;
            match b {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(Span::new(open + 1, self.pos - 1));
                    }
                }
                _ => {}
            }
        }
        self.error(
            ParseErrorKind::Unterminated,
            "unbalanced `{`: missing closing `}`",
        )
    }

    /// At `"`: skip to the closing quote at brace depth 0. Returns the inner span.
    fn quoted(&mut self) -> Result<Span> {
        let open = self.pos;
        self.pos += 1;
        let mut depth = 0usize;
        while let Some(b) = self.peek() {
            self.pos += 1;
            match b {
                b'{' => depth += 1,
                b'}' => depth = depth.saturating_sub(1),
                b'"' if depth == 0 => return Ok(Span::new(open + 1, self.pos - 1)),
                _ => {}
            }
        }
        self.error(ParseErrorKind::Unterminated, "unterminated quoted value")
    }

    /// `part (# part)*`
    fn value(&mut self) -> Result<Vec<Spanned<ValuePart<'s>>>> {
        let mut parts = Vec::new();
        loop {
            self.skip_ws();
            let start = self.pos;
            let part = match self.peek() {
                Some(b'{') => {
                    let inner = self.balanced()?;
                    ValuePart::Braced(self.text(inner.start, inner.end))
                }
                Some(b'"') => {
                    let inner = self.quoted()?;
                    ValuePart::Quoted(self.text(inner.start, inner.end))
                }
                _ => {
                    let word = self.take_ident();
                    if word.value.is_empty() {
                        return self.error(ParseErrorKind::ExpectedValue, "expected a field value");
                    }
                    if word.value.bytes().all(|b| b.is_ascii_digit()) {
                        ValuePart::Number(word.value)
                    } else {
                        ValuePart::Macro(word.value)
                    }
                }
            };
            parts.push(Spanned {
                span: Span::new(start, self.pos),
                value: part,
            });
            self.skip_ws();
            if self.peek() == Some(b'#') {
                self.pos += 1;
            } else {
                return Ok(parts);
            }
        }
    }

    /// `name = value` fields separated by commas, an optional trailing comma, then `close`.
    fn fields(&mut self, close: u8) -> Result<Vec<Field<'s>>> {
        let mut fields = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(close) {
                self.pos += 1;
                return Ok(fields);
            }
            let name = self.take_ident();
            if name.value.is_empty() {
                return if self.peek().is_none() {
                    self.error(
                        ParseErrorKind::Unterminated,
                        "entry is not closed before end of file",
                    )
                } else {
                    self.error(ParseErrorKind::ExpectedFieldName, "expected a field name")
                };
            }
            self.skip_ws();
            if self.peek() != Some(b'=') {
                return self.error(
                    ParseErrorKind::ExpectedEquals,
                    "expected `=` after the field name",
                );
            }
            self.pos += 1;
            let value = self.value()?;
            let end = value.last().map_or(self.pos, |p| p.span.end);
            fields.push(Field {
                span: Span::new(name.span.start, end),
                name,
                value,
            });
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b) if b == close => {}
                None => {
                    return self.error(
                        ParseErrorKind::Unterminated,
                        "entry is not closed before end of file",
                    );
                }
                _ => {
                    return self.error(
                        ParseErrorKind::ExpectedSeparator,
                        "expected `,` or the closing delimiter after a field",
                    );
                }
            }
        }
    }
}
