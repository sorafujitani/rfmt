//! Spike gate for the ruby-prism migration (#110 Phase 3): proves the crate
//! parses with the byte-offset and comment semantics the formatter relies on.

#[test]
fn parses_heredoc_comments_and_safe_navigation() {
    let source = r#"result = fetch&.body
sql = <<~SQL
  SELECT 1
SQL
# leading comment
value = compute(sql) # trailing comment
"#;

    let result = ruby_prism::parse(source.as_bytes());

    assert!(result.errors().next().is_none());
    assert_eq!(result.comments().count(), 2);

    let comment_offsets: Vec<(usize, usize)> = result
        .comments()
        .map(|c| {
            let loc = c.location();
            (loc.start_offset(), loc.end_offset())
        })
        .collect();
    let leading = &source[comment_offsets[0].0..comment_offsets[0].1];
    let trailing = &source[comment_offsets[1].0..comment_offsets[1].1];
    assert_eq!(leading, "# leading comment");
    assert_eq!(trailing, "# trailing comment");

    let program = result.node();
    let program = program.as_program_node().expect("program node");
    let statements = program.statements();
    assert_eq!(statements.body().iter().count(), 3);

    let first = statements.body().iter().next().expect("first statement");
    let loc = first.location();
    assert_eq!(&source[loc.start_offset()..loc.end_offset()], "result = fetch&.body");
}

#[test]
fn reports_errors_for_invalid_source() {
    let result = ruby_prism::parse(b"def broken(");
    assert!(result.errors().next().is_some());
}

#[test]
fn multibyte_source_keeps_byte_offsets() {
    let source = "x = \"あいう\"\ny = 1\n";
    let result = ruby_prism::parse(source.as_bytes());
    assert!(result.errors().next().is_none());

    let program = result.node();
    let program = program.as_program_node().expect("program node");
    let second = program
        .statements()
        .body()
        .iter()
        .nth(1)
        .expect("second statement");
    let loc = second.location();
    assert_eq!(&source[loc.start_offset()..loc.end_offset()], "y = 1");
}
