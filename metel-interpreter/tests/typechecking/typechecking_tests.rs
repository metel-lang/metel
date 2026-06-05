/// Integration tests for the full typechecker pipeline.
/// Tests the complete flow from parsing through type checking.
/// Source files are organized by language feature under tests/typechecking/sources/<feature>/.

#[cfg(test)]
mod tests {
    use std::path::Path;
    use metel::error::MetelError;
    use metel::parser;
    use metel::typechecker;

    // ── Harness helpers ───────────────────────────────────────────────────────

    fn load_source(path: &str) -> String {
        std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("could not read {path}: {e}"))
    }

    /// Parse `// ERROR[EXXXX]` annotations: returns (1-based line, code string) pairs.
    fn parse_error_annotations(source: &str) -> Vec<(usize, String)> {
        let mut out = vec![];
        for (idx, line) in source.lines().enumerate() {
            if let Some(pos) = line.find("// ERROR[") {
                let rest = &line[pos + 9..];
                if let Some(end) = rest.find(']') {
                    out.push((idx + 1, rest[..end].to_string()));
                }
            }
        }
        out
    }

    fn check_file(path: &str) {
        let source = load_source(path);
        let annotations = parse_error_annotations(&source);
        let filename = Path::new(path).file_name().unwrap().to_str().unwrap();

        let program = parser::parse(&source, filename)
            .unwrap_or_else(|e| panic!("parse error in {filename}: {e}"));
        let result = typechecker::check(program);

        if annotations.is_empty() {
            // Positive test: expect success.
            assert!(
                result.is_ok(),
                "expected Ok for {filename}, got error: {}",
                result.unwrap_err()
            );
        } else {
            // Negative test: expect a TypeError on the annotated line with the annotated code.
            let err = match result {
                Err(e) => e,
                Ok(_) => panic!("expected type error in {filename} but check() returned Ok"),
            };
            match &err {
                MetelError::TypeError { code, line, .. } => {
                    let (expected_line, expected_code) = &annotations[0];
                    assert_eq!(
                        format!("{code}"), *expected_code,
                        "wrong error code in {filename}"
                    );
                    assert_eq!(
                        *line as usize, *expected_line,
                        "wrong error line in {filename}: expected {expected_line}, got {line}"
                    );
                }
                other => panic!("expected TypeError in {filename}, got: {other}"),
            }
        }
    }

    fn test_dir() -> String {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/typechecking/sources").to_string()
    }

    fn check(path: &str) {
        check_file(&format!("{}/{path}", test_dir()));
    }

    // ── Literals ──────────────────────────────────────────────────────────────

    #[test]
    fn stage1_literals() { check("literals/01_literals.mtl"); }

    // ── Arithmetic ────────────────────────────────────────────────────────────

    #[test]
    fn stage1_arithmetic() { check("arithmetic/03_arithmetic.mtl"); }

    #[test]
    fn stage1_chained_arithmetic() { check("arithmetic/09_chained_arithmetic.mtl"); }

    #[test]
    fn stage1_neg_arithmetic_on_bool() { check("arithmetic/neg_03_arithmetic_on_bool.mtl"); }

    #[test]
    fn stage1_neg_neg_on_bool() { check("arithmetic/neg_04_neg_on_bool.mtl"); }

    #[test]
    fn stage1_neg_ordering_on_bool() { check("arithmetic/neg_05_ordering_on_bool.mtl"); }

    #[test]
    fn stage1_neg_string_add_mismatch() { check("arithmetic/neg_06_string_add_mismatch.mtl"); }

    #[test]
    fn question_mark_reports_the_postfix_column() {
        let source = r#"
struct ParseError { msg: String }
struct AppError { msg: String }

fun parse() -> Result<i64, ParseError> {
    Result::Err { error: ParseError { msg: "bad" } }
}

fun load() -> Result<i64, AppError> {
    let value = parse()?;
    Result::Ok { value: value }
}
"#;
        let program = parser::parse(source, "qmark_span.mtl")
            .unwrap_or_else(|e| panic!("parse error: {e}"));
        match typechecker::check(program) {
            Err(MetelError::TypeError { code, line, col, .. }) => {
                assert_eq!(format!("{code}"), "T0007");
                assert_eq!(line, 10);
                assert_eq!(col, 24);
            }
            Err(other) => panic!("expected TypeError, got: {other}"),
            Ok(_) => panic!("expected type error for missing From impl"),
        }
    }

    // ── Functions ─────────────────────────────────────────────────────────────

    #[test]
    fn stage1_annotations() { check("functions/02_annotations.mtl"); }

    #[test]
    fn stage3_function_calls() { check("functions/04_functions.mtl"); }

    #[test]
    fn stage3_nested_calls() { check("functions/05_nested_calls.mtl"); }

    #[test]
    fn stage3_let_polymorphism() { check("functions/06_let_polymorphism.mtl"); }

    #[test]
    fn stage3_forward_reference() { check("functions/07_forward_reference.mtl"); }

    #[test]
    fn stage1_mut_bindings() { check("functions/08_mut_bindings.mtl"); }

    #[test]
    fn stage1_scoping() { check("functions/10_scoping.mtl"); }

    #[test]
    fn stage1_neg_type_mismatch() { check("functions/neg_01_type_mismatch.mtl"); }

    #[test]
    fn stage1_neg_annotation_required() { check("functions/neg_02_annotation_required.mtl"); }

    #[test]
    fn stage4_assign() { check("functions/stage4_01_assign.mtl"); }

    #[test]
    fn stage4_return_diverges() { check("functions/stage4_02_return_diverges.mtl"); }

    #[test]
    fn stage4_index_assign() { check("functions/stage4_03_index_assign.mtl"); }

    #[test]
    fn stage4_neg_assign_to_let() { check("functions/stage4_neg_01_assign_to_let.mtl"); }

    #[test]
    fn stage4_neg_assign_undeclared() { check("functions/stage4_neg_02_assign_undeclared.mtl"); }

    #[test]
    fn stage4_neg_assign_type_mismatch() { check("functions/stage4_neg_03_assign_type_mismatch.mtl"); }

    #[test]
    fn stage4_neg_index_assign_type_mismatch() { check("functions/stage4_neg_04_index_assign_type_mismatch.mtl"); }

    #[test]
    fn ref_mut_receiver_requires_mutable_binding() {
        let source = r#"
struct Counter {
    value: i64,
}

impl Counter {
    fun increment(&mut self) {
        self.value += 1;
    }
}

fun main() {
    let counter = Counter { value: 0 };
    counter.increment();
}
"#;
        let program = parser::parse(source, "ref_mut_receiver_requires_mutable_binding.mtl")
            .unwrap_or_else(|e| panic!("parse error: {e}"));
        match typechecker::check(program) {
            Err(MetelError::TypeError { code, .. }) => {
                assert_eq!(format!("{code}"), "T0006");
            }
            Err(other) => panic!("expected TypeError, got: {other}"),
            Ok(_) => panic!("expected immutable receiver call to fail"),
        }
    }

    #[test]
    fn stage7_return_type_propagation() { check("functions/stage7_01_return_type_propagation.mtl"); }

    #[test]
    fn stage7_match_arm_blocks() { check("functions/stage7_02_match_arm_blocks.mtl"); }

    // ── Control flow ──────────────────────────────────────────────────────────

    #[test]
    fn stage2_if_stmt() { check("control_flow/stage2_01_if_stmt.mtl"); }

    #[test]
    fn stage2_while_stmt() { check("control_flow/stage2_02_while_stmt.mtl"); }

    #[test]
    fn stage2_if_expr() { check("control_flow/stage2_03_if_expr.mtl"); }

    #[test]
    fn stage2_else_if() { check("control_flow/stage2_04_else_if.mtl"); }

    #[test]
    fn stage2_neg_non_bool_condition() { check("control_flow/stage2_neg_01_non_bool_condition.mtl"); }

    #[test]
    fn stage6_for_loops() { check("control_flow/stage6_02_for_loops.mtl"); }

    #[test]
    fn stage6_loop_expr() { check("control_flow/stage6_03_loop_expr.mtl"); }

    #[test]
    fn stage6_nested_loop_break() { check("control_flow/stage6_09_nested_loop_break.mtl"); }

    #[test]
    fn stage6_neg_for_in_non_iterable() { check("control_flow/stage6_neg_01_for_in_non_iterable.mtl"); }

    #[test]
    fn stage6_neg_loop_break_mismatch() { check("control_flow/stage6_neg_02_loop_break_mismatch.mtl"); }

    // ── Types (arrays, tuples, casts) ─────────────────────────────────────────

    #[test]
    fn stage3_tuples() { check("types/stage3_01_tuples.mtl"); }

    #[test]
    fn stage3_arrays() { check("types/stage3_02_arrays.mtl"); }

    #[test]
    fn stage4_if_as_block_tail() { check("types/stage3_03_if_as_block_tail.mtl"); }

    #[test]
    fn stage3_neg_arity_mismatch() { check("types/stage3_neg_01_arity_mismatch.mtl"); }

    #[test]
    fn stage3_neg_index_non_array() { check("types/stage3_neg_02_index_non_array.mtl"); }

    #[test]
    fn stage3_neg_non_function_callee() { check("types/stage3_neg_03_non_function_callee.mtl"); }

    #[test]
    fn stage3_neg_empty_array_no_annotation() { check("types/stage3_neg_04_empty_array_no_annotation.mtl"); }

    #[test]
    fn stage3_neg_array_element_mismatch() { check("types/stage3_neg_05_array_element_mismatch.mtl"); }

    #[test]
    fn stage3_neg_non_int_index() { check("types/stage3_neg_06_non_int_index.mtl"); }

    #[test]
    fn stage4_neg_if_no_else_non_unit() { check("types/stage3_neg_07_if_no_else_non_unit.mtl"); }

    #[test]
    fn stage6_tuple_access() { check("types/stage6_04_tuple_access.mtl"); }

    #[test]
    fn stage6_neg_tuple_access_oob() { check("types/stage6_neg_03_tuple_access_oob.mtl"); }

    #[test]
    fn stage6_cast() { check("types/stage6_06_cast.mtl"); }

    #[test]
    fn stage6_neg_cast_string() { check("types/stage6_neg_04_cast_string.mtl"); }

    #[test]
    fn stage6_neg_cast_bool() { check("types/stage6_neg_10_cast_bool.mtl"); }

    #[test]
    fn stage6_neg_cast_float_to_int() { check("types/stage6_neg_11_cast_float_to_int.mtl"); }

    // ── Structs ───────────────────────────────────────────────────────────────

    #[test]
    fn stage5_structs_and_methods() { check("structs/stage5_01_structs_and_methods.mtl"); }

    #[test]
    fn stage5_builtin_type_methods() { check("structs/stage5_02_builtin_type_methods.mtl"); }

    #[test]
    fn stage5_self_method_signatures() { check("structs/stage5_03_self_method_signatures.mtl"); }

    #[test]
    fn stage5_neg_struct_field_type_mismatch() { check("structs/stage5_neg_01_struct_field_type_mismatch.mtl"); }

    #[test]
    fn stage5_neg_unknown_field() { check("structs/stage5_neg_02_unknown_field.mtl"); }

    #[test]
    fn stage5_neg_method_arg_type_mismatch() { check("structs/stage5_neg_03_method_arg_type_mismatch.mtl"); }

    #[test]
    fn stage5_neg_unknown_method() { check("structs/stage5_neg_04_unknown_method.mtl"); }

    #[test]
    fn stage5_neg_field_access_non_struct() { check("structs/stage5_neg_05_field_access_non_struct.mtl"); }

    #[test]
    fn stage5_neg_field_access_unknown_field() { check("structs/stage5_neg_06_field_access_unknown_field.mtl"); }

    #[test]
    fn stage5_neg_struct_literal_missing_field() { check("structs/stage5_neg_07_struct_literal_missing_field.mtl"); }

    #[test]
    fn stage9_local_struct_scope() { check("structs/stage9_01_local_struct_scope.mtl"); }

    #[test]
    fn stage9_neg_local_struct_not_exported() { check("structs/stage9_neg_01_local_struct_not_exported.mtl"); }

    // ── Enums ─────────────────────────────────────────────────────────────────

    #[test]
    fn stage6_enums() { check("enums/stage6_08_enums.mtl"); }

    #[test]
    fn stage6_enum_literal_types() { check("enums/stage6_10_enum_literal_types.mtl"); }

    #[test]
    fn stage6_neg_match_arm_mismatch() { check("enums/stage6_neg_06_match_arm_mismatch.mtl"); }

    #[test]
    fn stage6_neg_enum_unknown_variant() { check("enums/stage6_neg_08_enum_unknown_variant.mtl"); }

    #[test]
    fn stage6_neg_enum_field_type_mismatch() { check("enums/stage6_neg_09_enum_field_type_mismatch.mtl"); }

    // ── Closures ──────────────────────────────────────────────────────────────

    #[test]
    fn stage6_closures() { check("closures/stage6_05_closures.mtl"); }

    // ── Error handling ────────────────────────────────────────────────────────

    #[test]
    fn stage6_error_propagation() { check("error_handling/stage6_07_error_propagation.mtl"); }

    #[test]
    fn stage6_neg_error_propagation_non_result() { check("error_handling/stage6_neg_05_error_propagation_non_result.mtl"); }

    #[test]
    fn stage6_neg_error_propagation_mismatched_types() { check("error_handling/stage6_neg_06_error_propagation_mismatched_types.mtl"); }

    // ── Builtins and type ascription ──────────────────────────────────────────

    #[test]
    fn stage6_builtins() { check("builtins/stage6_01_builtins.mtl"); }

    #[test]
    fn stage6_neg_builtin_wrong_arg_type() { check("builtins/stage6_neg_07_builtin_wrong_arg_type.mtl"); }

    #[test]
    fn stage8_assert() { check("builtins/stage8_01_assert.mtl"); }

    #[test]
    fn stage8_dbg() { check("builtins/stage8_02_dbg.mtl"); }

    #[test]
    fn stage8_print_numeric() { check("builtins/stage8_03_print_numeric.mtl"); }

    #[test]
    fn stage8_type_ascription() { check("builtins/stage8_04_type_ascription.mtl"); }

    #[test]
    fn stage8_ascription_match_arm() { check("builtins/stage8_05_ascription_match_arm.mtl"); }

    #[test]
    fn stage8_ascription_match_arm_bare() { check("builtins/stage8_05_ascription_match_arm_bare.mtl"); }

    #[test]
    fn stage8_ascription_two_args() { check("builtins/stage8_06_ascription_two_args.mtl"); }

    #[test]
    fn stage8_ascription_two_args_bare() { check("builtins/stage8_06_ascription_two_args_bare.mtl"); }

    #[test]
    fn stage8_ascription_nope_arg() { check("builtins/stage8_07_ascription_nope_arg.mtl"); }

    #[test]
    fn stage8_ascription_nope_arg_bare() { check("builtins/stage8_07_ascription_nope_arg_bare.mtl"); }

    #[test]
    fn stage8_neg_assert_non_bool() { check("builtins/stage8_neg_01_assert_non_bool.mtl"); }

    #[test]
    fn stage8_neg_ascribe_type_mismatch() { check("builtins/stage8_neg_02_ascribe_type_mismatch.mtl"); }

    #[test]
    fn stage8_neg_ascribe_bool_as_int() { check("builtins/stage8_neg_03_ascribe_bool_as_int.mtl"); }

    #[test]
    fn stage8_neg_ascribe_wrong_struct() { check("builtins/stage8_neg_04_ascribe_wrong_struct.mtl"); }

    #[test]
    fn stage8_neg_interpolation_non_display() { check("builtins/stage8_neg_05_interpolation_non_display.mtl"); }

    // ── Generics ──────────────────────────────────────────────────────────────

    #[test]
    fn stage10_generic_function() { check("generics/stage10_01_generic_function.mtl"); }

    #[test]
    fn stage10_type_param_multiple_uses() { check("generics/stage10_02_type_param_multiple_uses.mtl"); }

    #[test]
    fn stage10_generic_return_tuple() { check("generics/stage10_03_generic_return_tuple.mtl"); }

    #[test]
    fn stage10_generic_higher_order() { check("generics/stage10_04_generic_higher_order.mtl"); }

    #[test]
    fn stage10_generic_nested_types() { check("generics/stage10_05_generic_nested_types.mtl"); }

    #[test]
    fn stage10_none_ascribed_generic_return() { check("generics/stage10_06_none_ascribed_generic_return.mtl"); }

    #[test]
    fn stage10_neg_type_param_conflict() { check("generics/stage10_neg_01_type_param_conflict.mtl"); }

    #[test]
    fn stage10_neg_return_type_conflict() { check("generics/stage10_neg_02_return_type_conflict.mtl"); }

    #[test]
    fn stage11_generic_struct_basic() { check("generics/stage11_01_generic_struct_basic.mtl"); }

    #[test]
    fn stage11_generic_struct_two_params() { check("generics/stage11_02_generic_struct_two_params.mtl"); }

    #[test]
    fn stage11_generic_enum_user() { check("generics/stage11_03_generic_enum_user.mtl"); }

    #[test]
    fn stage11_generic_nested() { check("generics/stage11_04_generic_nested.mtl"); }

    #[test]
    fn stage11_neg_generic_struct_field_conflict() { check("generics/stage11_neg_01_generic_struct_field_conflict.mtl"); }

    #[test]
    fn stage13_inline_multi_bound() { check("generics/stage13_01_inline_multi_bound.mtl"); }

    #[test]
    fn stage13_where_clause_struct() { check("generics/stage13_02_where_clause_struct.mtl"); }

    #[test]
    fn stage13_inline_and_where_merged() { check("generics/stage13_03_inline_and_where_merged.mtl"); }

    #[test]
    fn stage13_neg_inline_bound_violated() { check("generics/stage13_neg_01_inline_bound_violated.mtl"); }

    #[test]
    fn stage13_neg_where_bound_violated() { check("generics/stage13_neg_02_where_bound_violated.mtl"); }

    #[test]
    fn stage13_neg_second_of_two_bounds_violated() { check("generics/stage13_neg_03_second_of_two_bounds_violated.mtl"); }

    #[test]
    fn stage14_inline_plus_where_merged() { check("generics/stage14_01_inline_plus_where_merged.mtl"); }

    #[test]
    fn stage14_body_dispatch_all_bounds() { check("generics/stage14_02_body_dispatch_all_bounds.mtl"); }

    #[test]
    fn stage14_impl_aspect_plus_where() { check("generics/stage14_03_impl_aspect_plus_where.mtl"); }

    #[test]
    fn stage14_neg_where_bound_of_merged_violated() { check("generics/stage14_neg_01_where_bound_of_merged_violated.mtl"); }

    #[test]
    fn stage14_two_independent_bounded_params() { check("generics/stage14_04_two_independent_bounded_params.mtl"); }

    #[test]
    fn stage14_unbounded_param_alongside_bounded() { check("generics/stage14_05_unbounded_param_alongside_bounded.mtl"); }

    #[test]
    fn stage14_enum_construction_bound() { check("generics/stage14_06_enum_construction_bound.mtl"); }

    #[test]
    fn stage14_struct_bound_propagates_to_impl_body() { check("generics/stage14_07_struct_bound_propagates_to_impl_body.mtl"); }

    #[test]
    fn stage14_impl_aspect_array_param() { check("generics/stage14_08_impl_aspect_array_param.mtl"); }

    #[test]
    fn stage14_neg_method_not_in_bound() { check("generics/stage14_neg_02_method_not_in_bound.mtl"); }

    #[test]
    fn stage14_neg_second_bounded_param_violated() { check("generics/stage14_neg_03_second_bounded_param_violated.mtl"); }

    #[test]
    fn stage14_neg_enum_construction_bound_violated() { check("generics/stage14_neg_04_enum_construction_bound_violated.mtl"); }

    #[test]
    fn stage14_neg_bound_method_wrong_arity() { check("generics/stage14_neg_05_bound_method_wrong_arity.mtl"); }

    #[test]
    fn stage14_neg_bound_method_wrong_arg_type() { check("generics/stage14_neg_06_bound_method_wrong_arg_type.mtl"); }

    #[test]
    fn stage14_impl_method_with_type_param() { check("generics/stage14_09_impl_method_with_type_param.mtl"); }

    #[test]
    fn stage14_impl_method_with_bounded_type_param() { check("generics/stage14_10_impl_method_with_bounded_type_param.mtl"); }

    #[test]
    fn stage12_fun_bound_satisfied() { check("generics/stage12_01_fun_bound_satisfied.mtl"); }

    #[test]
    fn stage12_bound_method_in_body() { check("generics/stage12_02_bound_method_in_body.mtl"); }

    #[test]
    fn stage12_impl_aspect_param() { check("generics/stage12_03_impl_aspect_param.mtl"); }

    #[test]
    fn stage12_impl_aspect_independent() { check("generics/stage12_04_impl_aspect_independent.mtl"); }

    #[test]
    fn stage12_where_clause_fun_bound() { check("generics/stage12_05_where_clause_fun_bound.mtl"); }

    #[test]
    fn stage12_neg_fun_bound_not_satisfied() { check("generics/stage12_neg_01_fun_bound_not_satisfied.mtl"); }

    #[test]
    fn stage12_neg_impl_aspect_bound_not_satisfied() { check("generics/stage12_neg_02_impl_aspect_bound_not_satisfied.mtl"); }

    // ── Aspects ───────────────────────────────────────────────────────────────

    #[test]
    fn stage12_default_methods() { check("aspects/stage12_01_default_methods.mtl"); }

    #[test]
    fn stage12_override_replaces_default() { check("aspects/stage12_02_override_replaces_default.mtl"); }

    #[test]
    fn stage12_multiple_defaults() { check("aspects/stage12_03_multiple_defaults.mtl"); }

    #[test]
    fn stage12_neg_missing_required_with_defaults() { check("aspects/stage12_neg_01_missing_required_with_defaults.mtl"); }

    // ── Known limitations ─────────────────────────────────────────────────────

    #[test]
    fn limit_rank1_fn_arg() { check("generics/limit_01_rank1_fn_arg.mtl"); }

    #[test]
    fn stage10_let_polymorphism() { check("generics/limit_02_let_closure_mono.mtl"); }

    #[test]
    fn limit_field_access_needs_annotation() { check("generics/limit_03_field_access_needs_annotation.mtl"); }
}
