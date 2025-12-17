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

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_invalid_tabs_rejected() {
    let source = include_str!("fixtures/invalid_tabs.bobbin");

    // Should return an error, not hang or panic
    match Runtime::new(source) {
        Ok(_) => panic!("Expected error for tabs in indentation"),
        Err(err) => {
            // Use format_with_source to get detailed error messages
            let err_string = err.format_with_source(source);
            assert!(
                err_string.contains("tab") || err_string.contains("Tab"),
                "Error message should mention tabs: {}",
                err_string
            );
        }
    }
}

// =============================================================================
// Variable and Interpolation Tests
// =============================================================================

#[test]
fn test_temp_variable_with_interpolation() {
    let source = include_str!("fixtures/variables_interpolation.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // The interpolation should substitute {name} with "World"
    assert_eq!(runtime.current_line(), "Hello, World!");
    assert!(!runtime.has_more());
}

#[test]
fn test_multiple_variables() {
    let source = include_str!("fixtures/variables_multiple.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    assert_eq!(runtime.current_line(), "Hello, Alice!");
    assert!(!runtime.has_more());
}

#[test]
fn test_number_integer_interpolation() {
    let source = include_str!("fixtures/variables_number_integer.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // Integers should display without decimal point
    assert_eq!(runtime.current_line(), "You have 100 gold and 3 items.");
    assert!(!runtime.has_more());
}

#[test]
fn test_number_float_interpolation() {
    let source = include_str!("fixtures/variables_number_float.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    assert_eq!(runtime.current_line(), "The price is 19.99 plus 0.5 tax.");
    assert!(!runtime.has_more());
}

#[test]
fn test_boolean_interpolation() {
    let source = include_str!("fixtures/variables_boolean.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    assert_eq!(runtime.current_line(), "Door is open: true");
    runtime.advance();
    assert_eq!(runtime.current_line(), "Door is locked: false");
    assert!(!runtime.has_more());
}

#[test]
fn test_negative_number_interpolation() {
    let source = include_str!("fixtures/variables_negative_number.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    assert_eq!(runtime.current_line(), "You owe -500 gold.");
    runtime.advance();
    assert_eq!(runtime.current_line(), "It is -10.5 degrees outside.");
    assert!(!runtime.has_more());
}

#[test]
fn test_empty_string_interpolation() {
    let source = include_str!("fixtures/variables_empty_string.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // Empty string should produce no visible characters
    assert_eq!(runtime.current_line(), "HelloWorld");
    assert!(!runtime.has_more());
}

#[test]
fn test_escaped_braces() {
    let source = include_str!("fixtures/variables_escaped_braces.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    // {{ should produce literal {
    assert_eq!(runtime.current_line(), "Hello {name} is the syntax.");
    runtime.advance();
    assert_eq!(runtime.current_line(), "The value is Alice.");
    runtime.advance();
    assert_eq!(runtime.current_line(), "Use {} for literal braces.");
    assert!(!runtime.has_more());
}

#[test]
fn test_variable_used_multiple_times() {
    let source = include_str!("fixtures/variables_used_multiple_times.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    assert_eq!(runtime.current_line(), "The Knight enters the room.");
    runtime.advance();
    assert_eq!(runtime.current_line(), "The Knight draws a sword.");
    runtime.advance();
    assert_eq!(runtime.current_line(), "Go, Knight, go!");
    assert!(!runtime.has_more());
}

#[test]
fn test_variable_in_choice_text() {
    let source = include_str!("fixtures/variables_in_choice_text.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    assert_eq!(runtime.current_line(), "What will you do with the sword?");
    runtime.advance();

    // Choice text should have interpolation
    assert!(runtime.is_waiting_for_choice());
    assert_eq!(
        runtime.current_choices(),
        &["Keep the sword", "Drop the sword"]
    );

    runtime.select_choice(0).unwrap();
    assert_eq!(runtime.current_line(), "You kept the sword.");
    assert!(!runtime.has_more());
}

#[test]
fn test_variable_in_choice_text_second_option() {
    let source = include_str!("fixtures/variables_in_choice_text.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    runtime.advance(); // Skip to choice
    runtime.select_choice(1).unwrap();
    assert_eq!(runtime.current_line(), "You dropped the sword.");
    assert!(!runtime.has_more());
}

#[test]
fn test_outer_scope_variable_in_choice() {
    let source = include_str!("fixtures/variables_outer_scope_in_choice.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    assert_eq!(runtime.current_line(), "Welcome, Hero!");
    runtime.advance();
    runtime.select_choice(0).unwrap();

    // Variable from outer scope should be accessible
    assert_eq!(runtime.current_line(), "Hero enters the cave.");
    runtime.advance();
    assert_eq!(runtime.current_line(), "It is dark inside.");
    runtime.advance();
    assert_eq!(runtime.current_line(), "The adventure continues for Hero.");
    assert!(!runtime.has_more());
}

#[test]
fn test_outer_scope_variable_second_choice() {
    let source = include_str!("fixtures/variables_outer_scope_in_choice.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    runtime.advance(); // Skip to choice
    runtime.select_choice(1).unwrap();
    assert_eq!(runtime.current_line(), "Hero waits outside.");
    runtime.advance();
    assert_eq!(runtime.current_line(), "The adventure continues for Hero.");
    assert!(!runtime.has_more());
}

#[test]
fn test_variables_in_nested_choice_left() {
    let source = include_str!("fixtures/variables_in_nested_choice.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    assert_eq!(runtime.current_line(), "Choose your path:");
    runtime.advance();
    runtime.select_choice(0).unwrap();

    // Local variable in choice branch
    assert_eq!(runtime.current_line(), "Hero finds gold!");
    assert!(!runtime.has_more());
}

#[test]
fn test_variables_in_nested_choice_right() {
    let source = include_str!("fixtures/variables_in_nested_choice.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    runtime.advance();
    runtime.select_choice(1).unwrap();

    // Different local variable in other choice branch
    assert_eq!(runtime.current_line(), "Hero encounters a dragon!");
    assert!(!runtime.has_more());
}

#[test]
fn test_sibling_scope_variable_reuse() {
    // Same variable name in sibling scopes should be allowed
    // (they don't shadow each other since scopes are popped)
    let source = include_str!("fixtures/variables_sibling_scope_reuse.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    assert_eq!(runtime.current_line(), "Choose wisely:");
    runtime.advance();
    runtime.select_choice(0).unwrap();
    assert_eq!(runtime.current_line(), "Hero gets gold.");
    assert!(!runtime.has_more());
}

#[test]
fn test_sibling_scope_variable_reuse_second() {
    let source = include_str!("fixtures/variables_sibling_scope_reuse.bobbin");
    let mut runtime = Runtime::new(source).unwrap();

    runtime.advance();
    runtime.select_choice(1).unwrap();
    assert_eq!(runtime.current_line(), "Hero gets silver.");
    assert!(!runtime.has_more());
}

// =============================================================================
// Semantic Error Tests
// =============================================================================

#[test]
fn test_error_undefined_variable() {
    let source = include_str!("fixtures/error_undefined_variable.bobbin");

    match Runtime::new(source) {
        Ok(_) => panic!("Expected error for undefined variable"),
        Err(err) => {
            let err_string = err.format_with_source(source);
            assert!(
                err_string.contains("undefined") && err_string.contains("name"),
                "Error message should mention undefined variable 'name': {}",
                err_string
            );
        }
    }
}

#[test]
fn test_error_shadowing_in_choice() {
    let source = include_str!("fixtures/error_shadowing_in_choice.bobbin");

    match Runtime::new(source) {
        Ok(_) => panic!("Expected error for variable shadowing"),
        Err(err) => {
            let err_string = err.format_with_source(source);
            assert!(
                err_string.contains("shadow") && err_string.contains("name"),
                "Error message should mention shadowing of 'name': {}",
                err_string
            );
        }
    }
}

#[test]
fn test_error_redeclaration_same_scope() {
    let source = include_str!("fixtures/error_redeclaration.bobbin");

    match Runtime::new(source) {
        Ok(_) => panic!("Expected error for variable redeclaration"),
        Err(err) => {
            let err_string = err.format_with_source(source);
            assert!(
                err_string.contains("shadow") && err_string.contains("name"),
                "Error message should mention redeclaration of 'name': {}",
                err_string
            );
        }
    }
}

#[test]
fn test_error_unclosed_interpolation() {
    let source = include_str!("fixtures/error_unclosed_interpolation.bobbin");

    match Runtime::new(source) {
        Ok(_) => panic!("Expected error for unclosed interpolation"),
        Err(err) => {
            let err_string = err.format_with_source(source);
            // May get "Invalid character" for bad chars and/or "Unclosed interpolation" for missing }
            assert!(
                err_string.to_lowercase().contains("unclosed")
                    || err_string.to_lowercase().contains("invalid")
                    || err_string.to_lowercase().contains("interpolation"),
                "Error message should mention interpolation problem: {}",
                err_string
            );
        }
    }
}

#[test]
fn test_error_empty_interpolation() {
    let source = include_str!("fixtures/error_empty_interpolation.bobbin");

    match Runtime::new(source) {
        Ok(_) => panic!("Expected error for empty interpolation"),
        Err(err) => {
            let err_string = err.format_with_source(source);
            assert!(
                err_string.to_lowercase().contains("identifier")
                    || err_string.to_lowercase().contains("expected"),
                "Error message should mention expected identifier: {}",
                err_string
            );
        }
    }
}

#[test]
fn test_error_lone_closing_brace() {
    let source = include_str!("fixtures/error_lone_closing_brace.bobbin");

    match Runtime::new(source) {
        Ok(_) => panic!("Expected error for lone closing brace"),
        Err(err) => {
            let err_string = err.format_with_source(source);
            assert!(
                err_string.contains("}") || err_string.to_lowercase().contains("brace"),
                "Error message should mention unexpected brace: {}",
                err_string
            );
        }
    }
}
