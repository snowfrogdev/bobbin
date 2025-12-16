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

// =============================================================================
// Choice Tests
// =============================================================================

#[test]
fn test_choices_basic() {
    let source = include_str!("fixtures/choices_basic.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // Initial line
    assert_eq!(runtime.current_line(), "How are you?");
    assert!(!runtime.is_waiting_for_choice());
    runtime.advance();

    // Now at choice point
    assert!(runtime.is_waiting_for_choice());
    assert_eq!(runtime.current_choices(), &["Good", "Bad"]);

    // Select first choice
    runtime.select_choice(0).unwrap();
    assert!(!runtime.is_waiting_for_choice());
    assert!(!runtime.has_more());
}

#[test]
fn test_choices_with_content() {
    let source = include_str!("fixtures/choices_with_content.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // Initial line
    assert_eq!(runtime.current_line(), "How are you?");
    runtime.advance();

    // At choice point
    assert!(runtime.is_waiting_for_choice());
    assert_eq!(
        runtime.current_choices(),
        &["I'm doing great!", "Not so good..."]
    );

    // Select first choice - should get nested content
    runtime.select_choice(0).unwrap();
    assert!(!runtime.is_waiting_for_choice());
    assert_eq!(runtime.current_line(), "That's wonderful!");

    runtime.advance();
    assert_eq!(runtime.current_line(), "I'm glad to hear it.");
    assert!(!runtime.has_more());
}

#[test]
fn test_choices_with_content_second_option() {
    let source = include_str!("fixtures/choices_with_content.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // Initial line
    assert_eq!(runtime.current_line(), "How are you?");
    runtime.advance();

    // Select second choice
    runtime.select_choice(1).unwrap();
    assert_eq!(runtime.current_line(), "I'm sorry to hear that.");
    assert!(!runtime.has_more());
}

#[test]
fn test_choices_empty() {
    let source = include_str!("fixtures/choices_empty.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // Initial line
    assert_eq!(runtime.current_line(), "What's your name?");
    runtime.advance();

    // At choice point
    assert!(runtime.is_waiting_for_choice());
    assert_eq!(runtime.current_choices(), &["Alice", "Bob"]);

    // Select either choice - should go directly to gather point
    runtime.select_choice(0).unwrap();
    assert!(!runtime.is_waiting_for_choice());
    assert_eq!(runtime.current_line(), "Nice to meet you!");
    assert!(!runtime.has_more());
}

#[test]
fn test_choices_gather() {
    let source = include_str!("fixtures/choices_gather.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // Initial line
    assert_eq!(runtime.current_line(), "Pick a door:");
    runtime.advance();

    // At choice point
    assert!(runtime.is_waiting_for_choice());
    assert_eq!(runtime.current_choices(), &["Door A", "Door B"]);

    // Select first choice
    runtime.select_choice(0).unwrap();
    assert_eq!(runtime.current_line(), "You chose door A.");
    runtime.advance();

    // Should converge to gather point
    assert_eq!(runtime.current_line(), "The adventure continues...");
    runtime.advance();
    assert_eq!(runtime.current_line(), "Goodbye!");
    assert!(!runtime.has_more());
}

#[test]
fn test_choices_gather_second_option() {
    let source = include_str!("fixtures/choices_gather.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // Skip to choice
    runtime.advance();

    // Select second choice
    runtime.select_choice(1).unwrap();
    assert_eq!(runtime.current_line(), "You chose door B.");
    runtime.advance();

    // Should also converge to gather point
    assert_eq!(runtime.current_line(), "The adventure continues...");
    runtime.advance();
    assert_eq!(runtime.current_line(), "Goodbye!");
    assert!(!runtime.has_more());
}

#[test]
fn test_choices_sequential() {
    let source = include_str!("fixtures/choices_sequential.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // First question
    assert_eq!(runtime.current_line(), "First question?");
    runtime.advance();

    // First choice set
    assert!(runtime.is_waiting_for_choice());
    assert_eq!(runtime.current_choices(), &["Yes", "No"]);
    runtime.select_choice(0).unwrap();

    // Second question
    assert_eq!(runtime.current_line(), "Second question?");
    runtime.advance();

    // Second choice set
    assert!(runtime.is_waiting_for_choice());
    assert_eq!(runtime.current_choices(), &["Red", "Blue"]);
    runtime.select_choice(1).unwrap();

    // Done
    assert_eq!(runtime.current_line(), "Done!");
    assert!(!runtime.has_more());
}

#[test]
fn test_choices_mixed() {
    let source = include_str!("fixtures/choices_mixed.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // Initial line
    assert_eq!(runtime.current_line(), "What do you want to do?");
    runtime.advance();

    // At choice point
    assert!(runtime.is_waiting_for_choice());
    assert_eq!(runtime.current_choices(), &["Talk", "Leave"]);

    // Select first choice (has content)
    runtime.select_choice(0).unwrap();
    assert_eq!(runtime.current_line(), "Let's chat!");
    runtime.advance();

    // Should converge to gather
    assert_eq!(runtime.current_line(), "Goodbye!");
    assert!(!runtime.has_more());
}

#[test]
fn test_choices_mixed_empty_option() {
    let source = include_str!("fixtures/choices_mixed.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // Skip to choice
    runtime.advance();

    // Select second choice (no content - should go directly to gather)
    runtime.select_choice(1).unwrap();
    assert_eq!(runtime.current_line(), "Goodbye!");
    assert!(!runtime.has_more());
}

#[test]
fn test_choices_nested_talk_to_alice() {
    let source = include_str!("fixtures/choices_nested.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // Initial line
    assert_eq!(runtime.current_line(), "What do you want to do?");
    runtime.advance();

    // First choice: Talk or Leave
    assert!(runtime.is_waiting_for_choice());
    assert_eq!(runtime.current_choices(), &["Talk to someone", "Leave"]);
    runtime.select_choice(0).unwrap();

    // After selecting "Talk to someone"
    assert_eq!(runtime.current_line(), "Who would you like to talk to?");
    runtime.advance();

    // Nested choice: Alice or Bob
    assert!(runtime.is_waiting_for_choice());
    assert_eq!(runtime.current_choices(), &["Alice", "Bob"]);
    runtime.select_choice(0).unwrap();

    // After selecting "Alice"
    assert_eq!(runtime.current_line(), "You chat with Alice.");
    runtime.advance();

    // Inner gather point
    assert_eq!(runtime.current_line(), "That was a nice conversation.");
    runtime.advance();

    // Outer gather point
    assert_eq!(runtime.current_line(), "The end.");
    assert!(!runtime.has_more());
}

#[test]
fn test_choices_nested_talk_to_bob() {
    let source = include_str!("fixtures/choices_nested.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // Skip to first choice
    runtime.advance();
    runtime.select_choice(0).unwrap();

    // Skip to nested choice
    runtime.advance();

    // Select Bob
    assert_eq!(runtime.current_choices(), &["Alice", "Bob"]);
    runtime.select_choice(1).unwrap();

    // After selecting "Bob"
    assert_eq!(runtime.current_line(), "You chat with Bob.");
    runtime.advance();

    // Inner gather point
    assert_eq!(runtime.current_line(), "That was a nice conversation.");
    runtime.advance();

    // Outer gather point
    assert_eq!(runtime.current_line(), "The end.");
    assert!(!runtime.has_more());
}

#[test]
fn test_choices_nested_leave() {
    let source = include_str!("fixtures/choices_nested.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // Skip to first choice
    runtime.advance();

    // Select "Leave" - should skip the nested choice entirely
    assert_eq!(runtime.current_choices(), &["Talk to someone", "Leave"]);
    runtime.select_choice(1).unwrap();

    // After selecting "Leave"
    assert_eq!(runtime.current_line(), "Goodbye!");
    runtime.advance();

    // Outer gather point
    assert_eq!(runtime.current_line(), "The end.");
    assert!(!runtime.has_more());
}
