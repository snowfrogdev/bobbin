use bobbin_runtime::Runtime;

#[test]
fn test_simple_lines() {
    let source = include_str!("fixtures/simple_lines.bobbin");

    let mut runtime = Runtime::new(source).unwrap();

    assert_eq!(runtime.current_line(), "Hello world.");
    assert!(runtime.has_more());

    runtime.advance();
    assert_eq!(runtime.current_line(), "How are you?");
    assert!(runtime.has_more());

    runtime.advance();
    assert_eq!(runtime.current_line(), "Goodbye.");
    assert!(!runtime.has_more());
}

#[test]
fn test_empty_lines_skipped() {
    let source = include_str!("fixtures/empty_lines.bobbin");

    let mut runtime = Runtime::new(source).unwrap();

    assert_eq!(runtime.current_line(), "Line one.");
    runtime.advance();
    assert_eq!(runtime.current_line(), "Line two.");
    assert!(!runtime.has_more());
}

#[test]
fn test_empty_source() {
    let runtime = Runtime::new("").unwrap();

    assert_eq!(runtime.current_line(), "");
    assert!(!runtime.has_more());
}
