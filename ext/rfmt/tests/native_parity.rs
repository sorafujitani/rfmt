//! Parity tests for the native ruby-prism converter (migration phase 3).
//!
//! Each fixture under tests/fixtures/parity/ exists as a .rb source and the
//! JSON the Ruby PrismBridge produced for it (regenerate with
//! `bundle exec ruby scripts/gen_parity_fixtures.rb`). The .rb goes through
//! the new NativeAdapter, the .json through the legacy PrismAdapter, and the
//! trees must agree on node types, all six location fields, children, and
//! formatting.multiline. Metadata and comments are phase 4 and not compared.

use rfmt::ast::Node;
use rfmt::parser::{NativeAdapter, PrismAdapter, RubyParser};
use std::fs;
use std::path::{Path, PathBuf};

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

fn compare_nodes(path: &str, json: &Node, native: &Node, diffs: &mut Vec<String>) {
    push_diff(diffs, path, "node_type", &json.node_type, &native.node_type);

    let jl = &json.location;
    let nl = &native.location;
    push_diff(
        diffs,
        path,
        "location.start_line",
        &jl.start_line,
        &nl.start_line,
    );
    push_diff(
        diffs,
        path,
        "location.start_column",
        &jl.start_column,
        &nl.start_column,
    );
    push_diff(diffs, path, "location.end_line", &jl.end_line, &nl.end_line);
    push_diff(
        diffs,
        path,
        "location.end_column",
        &jl.end_column,
        &nl.end_column,
    );
    push_diff(
        diffs,
        path,
        "location.start_offset",
        &jl.start_offset,
        &nl.start_offset,
    );
    push_diff(
        diffs,
        path,
        "location.end_offset",
        &jl.end_offset,
        &nl.end_offset,
    );

    push_diff(
        diffs,
        path,
        "formatting.multiline",
        &json.formatting.multiline,
        &native.formatting.multiline,
    );

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
        names.len() >= 7,
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
fn native_adapter_reports_parse_errors_with_position() {
    let err = NativeAdapter::new()
        .parse("x = 1\ndef broken(")
        .unwrap_err();
    let message = err.to_string();
    assert!(message.contains("Parse error"), "{}", message);
    // Same line:column shape the Ruby bridge raises (line 2, byte column)
    assert!(message.contains("2:"), "{}", message);
}
