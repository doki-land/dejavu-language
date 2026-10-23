//! path-v1 catalog 解析合同。

use dejavu_ir::catalog::{ResolveRoot, posix_normalize, resolve_ref};
use dejavu_ir::{Diagnostic, TemplateCatalog, codes};

fn root(name: &str, scheme: &str, priority: i32, files: &[&str]) -> ResolveRoot {
    ResolveRoot { name: name.into(), scheme: scheme.into(), priority, files: files.iter().map(|f| (*f).to_string()).collect() }
}

fn extensions() -> Vec<String> {
    vec!["".into(), ".html".into(), ".doki".into(), ".dejavu".into()]
}

fn code(err: Diagnostic) -> String {
    err.code
}

#[test]
fn scheme_ref_hits_registered_path() {
    let roots = vec![root("pages", "pages", 0, &["index.dejavu", "partials/nav.dejavu"])];
    let hit = resolve_ref(&roots, &extensions(), "pages:partials/nav", None).unwrap();
    assert_eq!(hit.root_index, 0);
    assert_eq!(hit.path, "partials/nav.dejavu");
}

#[test]
fn unknown_scheme_is_structured() {
    let roots = vec![root("pages", "pages", 0, &["index.dejavu"])];
    let err = resolve_ref(&roots, &extensions(), "theme:index.dejavu", None).unwrap_err();
    assert_eq!(err.code, codes::TEMPLATE_UNKNOWN_SCHEME);
    assert_eq!(err.args.get("scheme").cloned(), Some("theme".into()));
}

#[test]
fn bare_ref_prefers_high_priority_root() {
    let roots = vec![root("app", "app", 0, &["index.dejavu"]), root("theme", "theme", 10, &["index.dejavu"])];
    let hit = resolve_ref(&roots, &extensions(), "index", None).unwrap();
    assert_eq!(hit.root_index, 1);
}

#[test]
fn bare_ref_ambiguous_at_equal_priority_is_rejected() {
    let roots = vec![root("app", "app", 0, &["index.dejavu"]), root("theme", "theme", 0, &["index.dejavu"])];
    let err = resolve_ref(&roots, &extensions(), "index", None).unwrap_err();
    assert_eq!(err.code, codes::TEMPLATE_AMBIGUOUS_REF);
    assert_eq!(err.args.get("priority").cloned(), Some(0.0.into()));
    assert_eq!(
        err.args.get("roots").cloned(),
        Some(vec!["app".into(), "theme".into()].into())
    );
    assert_eq!(
        err.args.get("candidates").cloned(),
        Some(vec!["app:index.dejavu".into(), "theme:index.dejavu".into()].into())
    );
}

#[test]
fn bare_ref_single_hit_wins_even_with_peer_roots() {
    let roots = vec![
        root("app", "app", 0, &["index.dejavu"]),
        root("theme", "theme", 0, &["layout.dejavu"]),
    ];
    let hit = resolve_ref(&roots, &extensions(), "index", None).unwrap();
    assert_eq!(hit.root_index, 0);
}

#[test]
fn explicit_scheme_disambiguates_equal_priority_roots() {
    let roots = vec![root("app", "app", 0, &["index.dejavu"]), root("theme", "theme", 0, &["index.dejavu"])];
    let hit = resolve_ref(&roots, &extensions(), "theme:index", None).unwrap();
    assert_eq!(hit.root_index, 1);
}

#[test]
fn relative_ref_resolves_against_from() {
    let roots = vec![root("pages", "pages", 0, &["a/b/child.dejavu", "a/shared.dejavu"])];
    let hit = resolve_ref(&roots, &extensions(), "../shared", Some("pages:a/b/child.dejavu")).unwrap();
    assert_eq!(hit.path, "a/shared.dejavu");
}

#[test]
fn relative_ref_without_from_is_rejected() {
    let roots = vec![root("pages", "pages", 0, &["a/shared.dejavu"])];
    let err = resolve_ref(&roots, &extensions(), "./shared", None).unwrap_err();
    assert_eq!(err.code, codes::TEMPLATE_RELATIVE_WITHOUT_FROM);
}

#[test]
fn relative_ref_escaping_root_is_rejected() {
    let roots = vec![root("pages", "pages", 0, &["top.dejavu"])];
    let err = resolve_ref(&roots, &extensions(), "../../outside", Some("pages:top.dejavu")).unwrap_err();
    assert_eq!(err.code, codes::TEMPLATE_RELATIVE_ESCAPE_ROOT);
}

#[test]
fn missing_template_reports_searched_roots() {
    let roots = vec![root("app", "app", 0, &["index.dejavu"]), root("theme", "theme", 5, &["layout.dejavu"])];
    let err = resolve_ref(&roots, &extensions(), "nope", None).unwrap_err();
    assert_eq!(err.code, codes::TEMPLATE_NOT_FOUND);
    assert_eq!(err.args.get("searched").cloned(), Some(vec!["theme".into(), "app".into()].into()));
}

#[test]
fn empty_ref_is_invalid() {
    let roots = vec![root("app", "app", 0, &["index.dejavu"])];
    let err = resolve_ref(&roots, &extensions(), "   ", None).unwrap_err();
    assert_eq!(err.code, codes::TEMPLATE_INVALID_REF);
}

#[test]
fn posix_normalize_rejects_escape() {
    assert_eq!(posix_normalize("a/./b/../c"), Ok("a/c".to_string()));
    assert_eq!(posix_normalize("../a"), Err(()));
}

#[test]
fn catalog_loader_resolves_documents_and_canonical_ids() {
    let source = "<% if a %>x<% end if %>";
    let doc = dejavu_language::parse_named(source, "pages:index.dejavu").unwrap();

    let mut pages = dejavu_ir::CatalogRoot::new("pages", "pages", 0);
    pages.insert("index.dejavu", doc);

    let catalog = TemplateCatalog::new(vec![pages], extensions());
    let hit = catalog.resolve_ref("index", None).unwrap();
    assert_eq!(catalog.canonical_id(&hit).as_deref(), Some("pages:index.dejavu"));
    assert!(catalog.document(&hit).is_some());
}
