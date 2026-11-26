//! The INSPIRE provider against saved responses served by the in-process mock server.

use bibx_core::provider::inspire::Inspire;
use bibx_core::provider::testing::{MockServer, Reply};
use bibx_core::provider::{Provider, single_entry};
use bibx_core::{Error, Identifier};

const RECORD: &str = include_str!("fixtures/inspire/record.bib");
const SEARCH_TWO: &str = include_str!("fixtures/inspire/search_two.bib");
const SEARCH_ZERO: &str = include_str!("fixtures/inspire/search_zero.bib");
const NOT_FOUND: &str = include_str!("fixtures/inspire/not_found.json");

const TEXKEY_SEARCH: &str = "/api/literature?q=texkey%3AMaldacena%3A1997re&size=2&format=bibtex";

fn fetch(server: &MockServer, id: &str) -> Result<bibx_core::provider::Fetched, Error> {
    Inspire::with_base_url(&server.url).fetch(&Identifier::parse(id))
}

#[test]
fn eprint_returns_the_entry_and_its_texkey() {
    let server = MockServer::start(vec![(
        "/api/arxiv/hep-th/9711200?format=bibtex".into(),
        Reply::bibtex(RECORD),
    )]);
    let fetched = fetch(&server, "eprint:hep-th/9711200v3").unwrap();
    assert_eq!(fetched.key(), "Maldacena:1997re");
    assert_eq!(fetched.entry(), RECORD.trim_end());
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].header("user-agent"),
        Some(concat!("bibx/", env!("CARGO_PKG_VERSION")))
    );
}

#[test]
fn doi_and_record_id_use_their_own_routes() {
    let server = MockServer::start(vec![
        (
            "/api/doi/10.4310/ATMP.1998.v2.n2.a1?format=bibtex".into(),
            Reply::bibtex(RECORD),
        ),
        (
            "/api/literature/451647?format=bibtex".into(),
            Reply::bibtex(RECORD),
        ),
    ]);
    assert!(fetch(&server, "doi:10.4310/ATMP.1998.v2.n2.a1").is_ok());
    assert!(fetch(&server, "inspire:451647").is_ok());
}

#[test]
fn texkey_is_a_search_with_exactly_one_hit() {
    let server = MockServer::start(vec![(TEXKEY_SEARCH.into(), Reply::bibtex(RECORD))]);
    assert_eq!(
        fetch(&server, "inspire:Maldacena:1997re").unwrap().key(),
        "Maldacena:1997re"
    );
}

#[test]
fn search_with_two_hits_is_ambiguous() {
    let server = MockServer::start(vec![(TEXKEY_SEARCH.into(), Reply::bibtex(SEARCH_TWO))]);
    let id = "inspire:Maldacena:1997re";
    assert!(matches!(
        fetch(&server, id),
        Err(Error::AmbiguousInspire(query)) if query == Identifier::parse(id)
    ));
}

#[test]
fn search_with_zero_hits_is_not_found() {
    let server = MockServer::start(vec![(TEXKEY_SEARCH.into(), Reply::bibtex(SEARCH_ZERO))]);
    assert!(matches!(
        fetch(&server, "inspire:Maldacena:1997re"),
        Err(Error::NotFound(_))
    ));
}

#[test]
fn status_codes_map_to_errors() {
    let server = MockServer::start(vec![
        (
            "/api/arxiv/1111.1111?format=bibtex".into(),
            Reply::status(404, NOT_FOUND),
        ),
        (
            "/api/arxiv/2222.2222?format=bibtex".into(),
            Reply::status(429, "{}"),
        ),
        (
            "/api/arxiv/3333.3333?format=bibtex".into(),
            Reply::status(500, "oops"),
        ),
    ]);
    assert!(matches!(
        fetch(&server, "eprint:1111.1111"),
        Err(Error::NotFound(_))
    ));
    assert!(
        matches!(fetch(&server, "eprint:2222.2222"), Err(Error::Network(m)) if m.contains("5 s"))
    );
    assert!(
        matches!(fetch(&server, "eprint:3333.3333"), Err(Error::Provider(m)) if m.contains("500"))
    );
}

#[test]
fn unreachable_server_is_a_network_error() {
    let url = {
        let server = MockServer::start(vec![]);
        server.url.clone()
    };
    let err = Inspire::with_base_url(url)
        .fetch(&Identifier::parse("eprint:1.1"))
        .unwrap_err();
    assert!(matches!(err, Error::Network(_)));
}

#[test]
fn a_garbled_body_is_a_parse_error() {
    assert!(matches!(
        single_entry("@article{k, title {x}"),
        Err(Error::Parse { .. })
    ));
    assert!(matches!(single_entry("nothing"), Err(Error::Parse { .. })));
}

#[test]
#[ignore = "needs network"]
fn live_eprint() {
    let fetched = Inspire::new()
        .fetch(&Identifier::parse("eprint:hep-th/9711200"))
        .unwrap();
    assert_eq!(fetched.key(), "Maldacena:1997re");
}

#[test]
#[ignore = "needs network"]
fn live_doi() {
    let fetched = Inspire::new()
        .fetch(&Identifier::parse("doi:10.1103/PhysRevLett.116.061102"))
        .unwrap();
    assert!(fetched.key().starts_with("LIGOScientific"));
}
