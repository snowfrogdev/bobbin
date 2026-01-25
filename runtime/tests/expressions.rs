mod support;

// ============================================
// Basic Equality Tests (==)
// ============================================

#[test]
fn equality_numbers_equal() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_numbers_equal.bobbin"));
}

#[test]
fn equality_numbers_not_equal() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_numbers_not_equal.bobbin"));
}

#[test]
fn equality_strings_equal() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_strings_equal.bobbin"));
}

#[test]
fn equality_strings_not_equal() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_strings_not_equal.bobbin"));
}

#[test]
fn equality_booleans_equal() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_booleans_equal.bobbin"));
}

#[test]
fn equality_booleans_not_equal() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_booleans_not_equal.bobbin"));
}

// ============================================
// Inequality Tests (!=)
// ============================================

#[test]
fn inequality_numbers_different() {
    support::run_output_test(&support::cases_dir().join("expressions/inequality_numbers_different.bobbin"));
}

#[test]
fn inequality_numbers_same() {
    support::run_output_test(&support::cases_dir().join("expressions/inequality_numbers_same.bobbin"));
}

#[test]
fn inequality_strings_different() {
    support::run_output_test(&support::cases_dir().join("expressions/inequality_strings_different.bobbin"));
}

#[test]
fn inequality_strings_same() {
    support::run_output_test(&support::cases_dir().join("expressions/inequality_strings_same.bobbin"));
}

#[test]
fn inequality_booleans_different() {
    support::run_output_test(&support::cases_dir().join("expressions/inequality_booleans_different.bobbin"));
}

#[test]
fn inequality_booleans_same() {
    support::run_output_test(&support::cases_dir().join("expressions/inequality_booleans_same.bobbin"));
}

// ============================================
// Variable Comparison Tests
// ============================================

#[test]
fn equality_variables() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_variables.bobbin"));
}

#[test]
fn equality_variable_literal() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_variable_literal.bobbin"));
}

// ============================================
// Literal on Left Side (Symmetric Expressions)
// ============================================

#[test]
fn equality_literal_left_number() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_literal_left_number.bobbin"));
}

#[test]
fn equality_literal_left_string() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_literal_left_string.bobbin"));
}

#[test]
fn equality_literal_left_bool() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_literal_left_bool.bobbin"));
}

#[test]
fn inequality_literal_left_number() {
    support::run_output_test(&support::cases_dir().join("expressions/inequality_literal_left_number.bobbin"));
}

#[test]
fn inequality_literal_left_string() {
    support::run_output_test(&support::cases_dir().join("expressions/inequality_literal_left_string.bobbin"));
}

#[test]
fn inequality_literal_left_bool() {
    support::run_output_test(&support::cases_dir().join("expressions/inequality_literal_left_bool.bobbin"));
}

// ============================================
// Literal-to-Literal Comparisons
// ============================================

#[test]
fn equality_literals_numbers() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_literals_numbers.bobbin"));
}

#[test]
fn equality_literals_strings() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_literals_strings.bobbin"));
}

#[test]
fn equality_literals_bools() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_literals_bools.bobbin"));
}

// ============================================
// Edge Cases
// ============================================

#[test]
fn equality_same_variable() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_same_variable.bobbin"));
}

#[test]
fn equality_negative_numbers() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_negative_numbers.bobbin"));
}

#[test]
fn equality_empty_strings() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_empty_strings.bobbin"));
}

#[test]
fn equality_float_numbers() {
    support::run_output_test(&support::cases_dir().join("expressions/equality_float_numbers.bobbin"));
}

// ============================================
// Error Tests (Type Mismatch)
// ============================================

#[test]
fn errors_type_mismatch_number_string() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/type_mismatch_number_string.bobbin"),
    );
}

#[test]
fn errors_type_mismatch_number_bool() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/type_mismatch_number_bool.bobbin"),
    );
}

#[test]
fn errors_type_mismatch_string_bool() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/type_mismatch_string_bool.bobbin"),
    );
}

#[test]
fn errors_type_mismatch_inequality() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/type_mismatch_inequality.bobbin"),
    );
}

#[test]
fn errors_type_mismatch_literal_left_number_string() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/type_mismatch_literal_left_number_string.bobbin"),
    );
}

#[test]
fn errors_type_mismatch_literal_left_bool_string() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/type_mismatch_literal_left_bool_string.bobbin"),
    );
}

#[test]
fn errors_type_mismatch_literal_left_number_bool() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/type_mismatch_literal_left_number_bool.bobbin"),
    );
}

#[test]
fn errors_type_mismatch_literal_literal() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/type_mismatch_literal_literal.bobbin"),
    );
}

// ============================================
// Ordering Comparison Tests (<, <=, >, >=)
// ============================================

#[test]
fn comparison_less_true() {
    support::run_output_test(&support::cases_dir().join("expressions/comparison_less_true.bobbin"));
}

#[test]
fn comparison_less_false() {
    support::run_output_test(&support::cases_dir().join("expressions/comparison_less_false.bobbin"));
}

#[test]
fn comparison_less_equal_true() {
    support::run_output_test(&support::cases_dir().join("expressions/comparison_less_equal_true.bobbin"));
}

#[test]
fn comparison_less_equal_false() {
    support::run_output_test(&support::cases_dir().join("expressions/comparison_less_equal_false.bobbin"));
}

#[test]
fn comparison_greater_true() {
    support::run_output_test(&support::cases_dir().join("expressions/comparison_greater_true.bobbin"));
}

#[test]
fn comparison_greater_false() {
    support::run_output_test(&support::cases_dir().join("expressions/comparison_greater_false.bobbin"));
}

#[test]
fn comparison_greater_equal_true() {
    support::run_output_test(&support::cases_dir().join("expressions/comparison_greater_equal_true.bobbin"));
}

#[test]
fn comparison_greater_equal_false() {
    support::run_output_test(&support::cases_dir().join("expressions/comparison_greater_equal_false.bobbin"));
}

#[test]
fn comparison_negative_numbers() {
    support::run_output_test(&support::cases_dir().join("expressions/comparison_negative_numbers.bobbin"));
}

#[test]
fn comparison_float_numbers() {
    support::run_output_test(&support::cases_dir().join("expressions/comparison_float_numbers.bobbin"));
}

#[test]
fn comparison_literal_left() {
    support::run_output_test(&support::cases_dir().join("expressions/comparison_literal_left.bobbin"));
}

#[test]
fn comparison_literals_only() {
    support::run_output_test(&support::cases_dir().join("expressions/comparison_literals_only.bobbin"));
}

// ============================================
// Error Tests (Comparison Requires Numeric)
// ============================================

#[test]
fn errors_comparison_string_operand() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/comparison_string_operand.bobbin"),
    );
}

#[test]
fn errors_comparison_bool_operand() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/comparison_bool_operand.bobbin"),
    );
}

#[test]
fn errors_comparison_string_string() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/comparison_string_string.bobbin"),
    );
}

#[test]
fn errors_comparison_bool_bool() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/comparison_bool_bool.bobbin"),
    );
}

// ============================================
// Error Tests (Undefined Variables in Comparisons)
// ============================================

#[test]
fn errors_undefined_comparison_left() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/undefined_comparison_left.bobbin"),
    );
}

#[test]
fn errors_undefined_comparison_right() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/undefined_comparison_right.bobbin"),
    );
}

#[test]
fn errors_undefined_comparison_both() {
    support::run_error_test(
        &support::cases_dir().join("expressions/errors/undefined_comparison_both.bobbin"),
    );
}
