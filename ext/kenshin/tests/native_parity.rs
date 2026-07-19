//! Parity tests for the native ruby-prism converter.
//!
//! Each fixture under tests/fixtures/parity/ exists as a .rb source and the
//! frozen golden JSON the (now deleted) Ruby PrismBridge produced for it.
//! The .rb goes through NativeAdapter, the .json through the test-support
//! PrismAdapter, and the trees must agree on node types, all six location
//! fields, children, formatting.multiline, metadata, and comments. This pins
//! NativeAdapter's output shape against future ruby-prism crate bumps; only
//! `bundle exec ruby scripts/gen_parity_fixtures.rb` may regenerate the JSON.

use kenshin::ast::{Location, Node};
use kenshin::parser::{NativeAdapter, PrismAdapter, RubyParser};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Keys the bridge still emits but nothing downstream reads; the native
/// converter intentionally drops them, so they are stripped from the JSON
/// side before the exact metadata comparison.
const DEAD_METADATA_KEYS: [&str; 4] = ["parameters_count", "message", "content", "value"];

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parity")
}

fn push_diff<T: std::fmt::Debug + PartialEq>(
    diffs: &mut Vec<String>,
    path: &str,
    field: &str,
    json: &T,
    native: &T,
) {
    if json != native {
        diffs.push(format!(
            "{}.{}: json={:?} native={:?}",
            path, field, json, native
        ));
    }
}

fn live_metadata(metadata: &HashMap<String, String>) -> HashMap<String, String> {
    metadata
        .iter()
        .filter(|(key, _)| !DEAD_METADATA_KEYS.contains(&key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn compare_locations(path: &str, json: &Location, native: &Location, diffs: &mut Vec<String>) {
    push_diff(
        diffs,
        path,
        "start_line",
        &json.start_line,
        &native.start_line,
    );
    push_diff(
        diffs,
        path,
        "start_column",
        &json.start_column,
        &native.start_column,
    );
    push_diff(diffs, path, "end_line", &json.end_line, &native.end_line);
    push_diff(
        diffs,
        path,
        "end_column",
        &json.end_column,
        &native.end_column,
    );
    push_diff(
        diffs,
        path,
        "start_offset",
        &json.start_offset,
        &native.start_offset,
    );
    push_diff(
        diffs,
        path,
        "end_offset",
        &json.end_offset,
        &native.end_offset,
    );
}

fn compare_nodes(path: &str, json: &Node, native: &Node, diffs: &mut Vec<String>) {
    push_diff(diffs, path, "node_type", &json.node_type, &native.node_type);

    compare_locations(
        &format!("{}.location", path),
        &json.location,
        &native.location,
        diffs,
    );

    push_diff(
        diffs,
        path,
        "formatting.multiline",
        &json.formatting.multiline,
        &native.formatting.multiline,
    );

    let json_metadata = live_metadata(&json.metadata);
    push_diff(diffs, path, "metadata", &json_metadata, &native.metadata);

    push_diff(
        diffs,
        path,
        "comments.len",
        &json.comments.len(),
        &native.comments.len(),
    );
    for (i, (jc, nc)) in json.comments.iter().zip(native.comments.iter()).enumerate() {
        let cpath = format!("{}.comments[{}]", path, i);
        push_diff(diffs, &cpath, "text", &jc.text, &nc.text);
        push_diff(
            diffs,
            &cpath,
            "comment_type",
            &jc.comment_type,
            &nc.comment_type,
        );
        push_diff(diffs, &cpath, "position", &jc.position, &nc.position);
        compare_locations(
            &format!("{}.location", cpath),
            &jc.location,
            &nc.location,
            diffs,
        );
    }

    push_diff(
        diffs,
        path,
        "children.len",
        &json.children.len(),
        &native.children.len(),
    );
    for (i, (jc, nc)) in json.children.iter().zip(native.children.iter()).enumerate() {
        compare_nodes(&format!("{}.children[{}]", path, i), jc, nc, diffs);
    }
}

#[test]
fn native_conversion_matches_ruby_bridge() {
    let dir = fixtures_dir();
    let mut names: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("fixtures dir missing; add fixtures under tests/fixtures/parity")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "rb"))
        .collect();
    names.sort();
    assert!(
        names.len() >= 11,
        "expected the full parity fixture set, found {:?}",
        names
    );

    let mut failures = Vec::new();
    for rb_path in &names {
        let name = rb_path.file_stem().unwrap().to_string_lossy().into_owned();
        let source = fs::read_to_string(rb_path).unwrap();
        let json = fs::read_to_string(rb_path.with_extension("json")).unwrap_or_else(|_| {
            panic!(
                "{}.json missing; run `bundle exec ruby scripts/gen_parity_fixtures.rb`",
                name
            )
        });

        let json_tree = PrismAdapter::new().parse(&json).expect("bridge JSON parse");
        let native_tree = NativeAdapter::new().parse(&source).expect("native parse");

        let mut diffs = Vec::new();
        compare_nodes("root", &json_tree, &native_tree, &mut diffs);
        if !diffs.is_empty() {
            failures.push(format!("[{}]\n{}", name, diffs.join("\n")));
        }
    }

    assert!(failures.is_empty(), "\n{}", failures.join("\n\n"));
}

#[test]
fn comparator_detects_a_mutated_tree() {
    let source = fs::read_to_string(fixtures_dir().join("plain.rb")).unwrap();
    let reference = NativeAdapter::new().parse(&source).unwrap();
    let mut mutated = reference.clone();
    let first = mutated.children.first_mut().expect("fixture has children");
    first.location.end_line += 1;

    let mut diffs = Vec::new();
    compare_nodes("root", &reference, &mutated, &mut diffs);
    assert_eq!(diffs.len(), 1, "{diffs:?}");
    assert!(
        diffs[0].starts_with("root.children[0].location.end_line: "),
        "{diffs:?}"
    );
}

#[test]
fn comparator_detects_a_mutated_metadata_value() {
    let reference = NativeAdapter::new()
        .parse("def foo(a)\n  a\nend\n")
        .unwrap();
    let mut mutated = reference.clone();
    let def = mutated.children.first_mut().expect("def child");
    assert_eq!(def.metadata.get("name").map(String::as_str), Some("foo"));
    def.metadata.insert("name".to_string(), "bar".to_string());

    let mut diffs = Vec::new();
    compare_nodes("root", &reference, &mutated, &mut diffs);
    assert_eq!(diffs.len(), 1, "{diffs:?}");
    assert!(
        diffs[0].starts_with("root.children[0].metadata: "),
        "{diffs:?}"
    );
}

#[test]
fn comparator_detects_a_mutated_comment() {
    let reference = NativeAdapter::new().parse("# hello\nx = 1\n").unwrap();
    let mut mutated = reference.clone();
    let comment = mutated.comments.first_mut().expect("root comment");
    assert_eq!(comment.text, "# hello");
    comment.text = "# tampered".to_string();

    let mut diffs = Vec::new();
    compare_nodes("root", &reference, &mutated, &mut diffs);
    assert_eq!(diffs.len(), 1, "{diffs:?}");
    assert!(diffs[0].starts_with("root.comments[0].text: "), "{diffs:?}");
}

#[test]
fn native_adapter_reports_parse_errors_with_position() {
    let err = NativeAdapter::new()
        .parse("x = 1\ndef broken(")
        .unwrap_err();
    let message = err.to_string();
    assert!(message.contains("Parse error"), "{}", message);
    // Same line:column shape the Ruby bridge raises (line 2, byte column)
    assert!(message.contains("2:"), "{}", message);
}
