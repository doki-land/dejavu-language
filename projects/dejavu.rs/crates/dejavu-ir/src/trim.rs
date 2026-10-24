//! Whitespace control contract.
//!
//! Tags carry a left trim level on their own node (`trim`) and a right trim
//! level on the tag closer. The parser resolves the right side into the
//! following text node, so IR consumers only ever need the left side.
//!
//! Levels are a ladder of increasingly wider strips at one edge:
//!
//! | level   | marker | strips                                                   |
//! |---------|--------|----------------------------------------------------------|
//! | `none`  | `.`    | nothing                                                  |
//! | `ws`    | `_`    | horizontal whitespace (` ` / `\t`)                       |
//! | `nl`    | `-`    | exactly one line break (`\r\n` / `\n` / `\r`)            |
//! | `ws_nl` | `~`    | horizontal whitespace, one line break, then whitespace   |
//! | `all`   | `=`    | every consecutive whitespace character including breaks  |
//!
//! Unknown levels are treated as `none` so that forward IR stays renderable.

use crate::types::IrNode;

/// Trim level carried by an IR node. Nodes without a `trim` field are `none`.
pub fn level_of(node: &IrNode) -> &str {
    match node {
        IrNode::Interpolation { trim, .. }
        | IrNode::StmtIf { trim, .. }
        | IrNode::StmtElseIf { trim, .. }
        | IrNode::StmtFor { trim, .. }
        | IrNode::StmtBlock { trim, .. }
        | IrNode::StmtPartial { trim, .. }
        | IrNode::StmtMacro { trim, .. }
        | IrNode::StmtImport { trim, .. }
        | IrNode::StmtExtends { trim, .. }
        | IrNode::StmtInclude { trim, .. }
        | IrNode::StmtSuper { trim, .. }
        | IrNode::StmtRaw { trim, .. } => trim.as_str(),
        _ => "none",
    }
}

fn is_horizontal(c: char) -> bool {
    c == ' ' || c == '\t'
}

fn is_whitespace(c: char) -> bool {
    c.is_whitespace()
}

/// Remove whitespace from the start of `value` according to `level`.
pub fn trim_prefix(value: &mut String, level: &str) {
    match level {
        "ws" => {
            let n = value.len() - value.trim_start_matches(is_horizontal).len();
            value.drain(..n);
        }
        "nl" => {
            strip_one_leading_break(value);
        }
        "ws_nl" => {
            let n = value.len() - value.trim_start_matches(is_horizontal).len();
            value.drain(..n);
            if strip_one_leading_break(value) {
                let n = value.len() - value.trim_start_matches(is_horizontal).len();
                value.drain(..n);
            }
        }
        "all" => {
            let n = value.len() - value.trim_start_matches(is_whitespace).len();
            value.drain(..n);
        }
        _ => {}
    }
}

/// Remove whitespace from the end of `value` according to `level`.
pub fn trim_suffix(value: &mut String, level: &str) {
    match level {
        "ws" => {
            let n = value.len() - value.trim_end_matches(is_horizontal).len();
            value.truncate(value.len() - n);
        }
        "nl" => {
            strip_one_trailing_break(value);
        }
        "ws_nl" => {
            let n = value.len() - value.trim_end_matches(is_horizontal).len();
            value.truncate(value.len() - n);
            if strip_one_trailing_break(value) {
                let n = value.len() - value.trim_end_matches(is_horizontal).len();
                value.truncate(value.len() - n);
            }
        }
        "all" => {
            let n = value.len() - value.trim_end_matches(is_whitespace).len();
            value.truncate(value.len() - n);
        }
        _ => {}
    }
}

fn strip_one_leading_break(value: &mut String) -> bool {
    if value.starts_with("\r\n") {
        value.drain(..2);
        true
    } else if value.starts_with('\n') || value.starts_with('\r') {
        value.drain(..1);
        true
    } else {
        false
    }
}

fn strip_one_trailing_break(value: &mut String) -> bool {
    if value.ends_with("\r\n") {
        let n = value.len() - 2;
        value.truncate(n);
        true
    } else if value.ends_with('\n') || value.ends_with('\r') {
        let n = value.len() - 1;
        value.truncate(n);
        true
    } else {
        false
    }
}

/// Clone a sibling list and apply every node's left trim to the text before it.
///
/// Renderers and AOT emit both go through this so that one IR document always
/// produces the same whitespace regardless of host.
pub fn normalized_children(nodes: &[IrNode]) -> Vec<IrNode> {
    let mut out = nodes.to_vec();
    for i in 1..out.len() {
        let level = level_of(&out[i]).to_string();
        if level == "none" {
            continue;
        }
        if let IrNode::Text { value, .. } = &mut out[i - 1] {
            trim_suffix(value, &level);
        }
    }
    out
}
