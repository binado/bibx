use bibx_parser::parse;
use proptest::prelude::*;

fn check(src: &str) {
    let bib = parse(src);
    let mut at = 0;
    let mut out = String::new();
    for item in &bib.items {
        let span = item.span();
        assert_eq!(span.start, at, "gap or overlap in {src:?}");
        assert!(span.end > span.start, "empty item in {src:?}");
        out.push_str(span.text(src));
        at = span.end;
    }
    assert_eq!(at, src.len());
    assert_eq!(out, src);
}

fn token() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("@article".to_string()),
        Just("@string".to_string()),
        Just("@comment".to_string()),
        Just("@preamble".to_string()),
        Just("{".to_string()),
        Just("}".to_string()),
        Just("(".to_string()),
        Just(")".to_string()),
        Just("\"".to_string()),
        Just("=".to_string()),
        Just(",".to_string()),
        Just("#".to_string()),
        Just("%".to_string()),
        Just("\n".to_string()),
        Just("\r\n".to_string()),
        Just("  ".to_string()),
        Just("é☕".to_string()),
        "[a-z0-9:]{1,6}",
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test]
    fn arbitrary_strings_round_trip(src in any::<String>()) {
        check(&src);
    }

    #[test]
    fn bibtexish_strings_round_trip(tokens in proptest::collection::vec(token(), 0..60)) {
        check(&tokens.concat());
    }
}
