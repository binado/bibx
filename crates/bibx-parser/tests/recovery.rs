//! The deliberate differences from `biblatex`, error placement and fixtures that
//! `biblatex` rejects.

mod common;

use bibx_parser::{Bib, Item, ParseErrorKind, ValuePart, parse};

fn keys<'a>(bib: &Bib<'a>) -> Vec<&'a str> {
    bib.entries().map(|e| e.key.value).collect()
}

fn joined(src: &str, bib: &Bib) -> String {
    bib.items.iter().map(|i| i.span().text(src)).collect()
}

#[test]
fn comment_body_may_contain_nested_braces() {
    let src = "@comment{ a { b { c } } d }\n@article{k1, title = {x}}\n";
    let bib = parse(src);
    assert!(
        matches!(bib.items[0], Item::Comment(s) if s.text(src) == "@comment{ a { b { c } } d }")
    );
    assert_eq!(keys(&bib), ["k1"]);
    assert_eq!(bib.errors().count(), 0);
}

#[test]
fn comment_without_delimiter_is_only_the_keyword() {
    let src = "@comment this one has no braces\n@article{k, title = {x}}\n";
    let bib = parse(src);
    assert_eq!(keys(&bib), ["k"]);
    assert_eq!(bib.errors().count(), 0);
}

#[test]
fn backslash_does_not_escape_braces() {
    // BibTeX counts the `}` after the backslash, so the value ends there.
    let src = "@article{k, title = {a \\}}\n";
    let bib = parse(src);
    let e = bib.entries().next().unwrap();
    assert_eq!(e.field("title").unwrap().raw(src), "{a \\}");
}

#[test]
fn truncated_entry_runs_to_the_next_line_starting_with_at() {
    let src = common::read(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/edge/truncated.bib"),
    );
    let bib = parse(&src);
    assert_eq!(keys(&bib), ["ok1", "ok2"]);
    let errors: Vec<_> = bib.errors().collect();
    assert_eq!(errors.len(), 1);
    let (span, error) = errors[0];
    assert!(span.text(&src).starts_with("@article{broken,"));
    assert!(span.text(&src).ends_with("year = 2001\n\n"));
    assert_eq!(error.kind, ParseErrorKind::Unterminated);
    assert_eq!(joined(&src, &bib), src);
}

#[test]
fn missing_equals_is_reported_and_neighbours_survive() {
    let src = common::read(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/edge/missing-equals.bib"),
    );
    let bib = parse(&src);
    assert_eq!(keys(&bib), ["ok1", "ok2"]);
    let (span, error) = bib.errors().next().unwrap();
    assert_eq!(error.kind, ParseErrorKind::ExpectedEquals);
    assert!(span.text(&src).starts_with("@article{bad,"));
    assert_eq!(&src[error.offset..error.offset + 1], "{");
}

#[test]
fn indented_at_still_ends_an_error() {
    let src = "@article{bad,\n  title {x}\n   @article{good, title = {y}}\n";
    let bib = parse(src);
    assert_eq!(keys(&bib), ["good"]);
    let (span, _) = bib.errors().next().unwrap();
    assert_eq!(span.text(src), "@article{bad,\n  title {x}\n");
}

#[test]
fn junk_between_entries_is_comment_text() {
    let src = common::read(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/edge/junk-between.bib"),
    );
    let bib = parse(&src);
    assert_eq!(keys(&bib), ["good1", "good2", "good3"]);
    // `a@b.org` looks like an entry start; it must not hide `good2`.
    assert_eq!(bib.errors().count(), 1);
}

#[test]
fn value_parts() {
    let src = "@string{foo = \"Foo\"}\n@article{k, a = foo # \" L\" # {b} # 2020}\n";
    let bib = parse(src);
    let parts: Vec<_> = bib.entries().next().unwrap().fields[0]
        .value
        .iter()
        .map(|p| p.value)
        .collect();
    assert_eq!(
        parts,
        [
            ValuePart::Macro("foo"),
            ValuePart::Quoted(" L"),
            ValuePart::Braced("b"),
            ValuePart::Number("2020"),
        ]
    );
    assert!(matches!(bib.items[0], Item::String(_)));
}

#[test]
fn field_helpers() {
    let src = "@Article{K, Title = {A {b}}, year = 2020, j = foo # \"x\"}";
    let bib = parse(src);
    let e = bib.entries().next().unwrap();
    assert_eq!(e.field("title").unwrap().text(src), "A {b}");
    assert_eq!(e.field("TITLE").unwrap().raw(src), "{A {b}}");
    assert_eq!(e.field("year").unwrap().text(src), "2020");
    assert_eq!(e.field("j").unwrap().text(src), "foo # \"x\"");
    assert!(e.field("missing").is_none());
}

#[test]
fn entry_span_runs_from_at_to_closing_delimiter() {
    let src = "text\n  @article(k, title = {x})\ntrailing\n";
    let bib = parse(src);
    let e = bib.entries().next().unwrap();
    assert_eq!(e.span.text(src), "@article(k, title = {x})");
    assert_eq!(e.key.span.text(src), "k");
}

#[test]
fn every_fixture_round_trips_and_is_contiguous() {
    for path in common::fixtures() {
        let src = common::read(&path);
        let bib = parse(&src);
        assert_eq!(joined(&src, &bib), src, "{}", path.display());
        let mut at = 0;
        for item in &bib.items {
            assert_eq!(item.span().start, at, "{}", path.display());
            assert!(!item.span().is_empty());
            at = item.span().end;
        }
        assert_eq!(at, src.len());
    }
}

#[test]
fn fixtures_biblatex_rejects_report_errors_where_expected() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/biblatex");
    let src = common::read(&dir.join("incorrect_syntax.bib"));
    let bib = parse(&src);
    assert_eq!(joined(&src, &bib), src);
    for (span, _) in bib.errors() {
        assert!(span.text(&src).starts_with('@'));
    }
}
