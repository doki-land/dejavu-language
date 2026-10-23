//! The parser must annotate every construct with the source bytes it covers,
//! otherwise codegen cannot report line origins and hosts get empty source maps.

use dejavu_language::parse;
use serde_json::Value;

fn span(node: &Value) -> (u64, u64) {
    let s = &node["span"];
    (
        s["start"].as_u64().unwrap_or_else(|| panic!("node has no span: {node}")),
        s["end"].as_u64().unwrap_or_else(|| panic!("node has no span: {node}")),
    )
}

#[test]
fn parse_annotates_constructs_with_byte_spans() {
    let source = "<h1><% title %></h1>\n<% if ok %>\n<p><% ok %></p>\n<% end if %>\n";
    let doc = parse(source).expect("template should parse");
    let json = serde_json::to_value(&doc).expect("IR should serialize");
    let children = json["body"]["children"].as_array().expect("template children");

    assert_eq!(children.len(), 5, "text, interpolation, text, if, trailing text");
    assert_eq!(span(&children[0]), (0, 4));
    assert_eq!(span(&children[1]), (4, 15));
    assert_eq!(span(&children[2]), (15, 21));
    assert_eq!(span(&children[3]), (21, 61));

    let consequent = children[3]["consequent"].as_array().expect("if body");
    assert_eq!(span(&consequent[0]), (32, 36));
    assert_eq!(span(&consequent[1]), (36, 44));
    assert_eq!(span(&consequent[2]), (44, 49));
}

#[test]
fn parse_spans_cover_the_whole_source_when_tiles_are_adjacent() {
    let source = "a<% x %>b";
    let doc = parse(source).expect("template should parse");
    let json = serde_json::to_value(&doc).expect("IR should serialize");
    let children = json["body"]["children"].as_array().expect("template children");

    let spans: Vec<(u64, u64)> = children.iter().map(span).collect();
    assert_eq!(spans, vec![(0, 1), (1, 8), (8, 9)]);
}
