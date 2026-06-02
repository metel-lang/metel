/// Integration tests for the evaluator.
/// All Metel source files live in tests/evaluator/sources/<feature>/.
///
/// Positive files are self-asserting:
///   `let _ok = match (actual == expected) { true => 0, };`
///   If the condition is false no arm matches → runtime panic → test fails.
///
/// Negative files carry one annotation on any line:
///   `// RUNTIME_ERROR[substring]`   — program typechecks but fails at runtime
///   `// TYPECHECK_ERROR[substring]` — program is rejected at typecheck time

#[cfg(test)]
mod tests {
    use std::path::Path;
    use metel::{evaluator, parser, typechecker};

    // ── Harness ───────────────────────────────────────────────────────────────

    fn test_dir() -> String {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/evaluator/sources").to_string()
    }

    fn load_source(path: &str) -> String {
        std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("could not read {path}: {e}"))
    }

    fn parse_annotation(source: &str) -> Option<(String, String)> {
        for line in source.lines() {
            if let Some(pos) = line.find("// PARSE_ERROR[") {
                let rest = &line[pos + 15..];
                if let Some(end) = rest.find(']') {
                    return Some(("parse".into(), rest[..end].to_string()));
                }
            }
            if let Some(pos) = line.find("// RUNTIME_ERROR[") {
                let rest = &line[pos + 17..];
                if let Some(end) = rest.find(']') {
                    return Some(("runtime".into(), rest[..end].to_string()));
                }
            }
            if let Some(pos) = line.find("// TYPECHECK_ERROR[") {
                let rest = &line[pos + 19..];
                if let Some(end) = rest.find(']') {
                    return Some(("typecheck".into(), rest[..end].to_string()));
                }
            }
        }
        None
    }

    fn check_file(path: &str) {
        let source = load_source(path);
        let filename = Path::new(path).file_name().unwrap().to_str().unwrap();
        match parse_annotation(&source) {
            Some((kind, expected)) if kind == "parse" => {
                let err = parser::parse(&source, filename)
                    .expect_err("expected parse error, but parsing succeeded")
                    .to_string();
                assert!(
                    err.contains(&expected),
                    "expected error containing '{expected}', got: {err}"
                );
            }
            Some((kind, expected)) if kind == "runtime" => {
                let ast = parser::parse(&source, filename).expect("parse error");
                let prog = typechecker::check(ast).expect("typecheck error");
                let err = evaluator::evaluate(prog)
                    .expect_err("expected runtime error, but program succeeded")
                    .to_string();
                assert!(
                    err.contains(&expected),
                    "expected error containing '{expected}', got: {err}"
                );
            }
            Some((_, expected)) => {
                let ast = parser::parse(&source, filename).expect("parse error");
                let err = typechecker::check(ast)
                    .expect_err("expected typecheck error, but check() returned Ok")
                    .to_string();
                assert!(
                    err.contains(&expected),
                    "expected error containing '{expected}', got: {err}"
                );
            }
            None => {
                let ast = parser::parse(&source, filename).expect("parse error");
                let prog = typechecker::check(ast).expect("typecheck error");
                evaluator::evaluate(prog).expect("runtime error");
            }
        }
    }

    fn check(path: &str) {
        check_file(&format!("{}/{path}", test_dir()));
    }

    // ── Literals ──────────────────────────────────────────────────────────────

    #[test]
    fn literals() { check("literals/01_literals.mtl"); }

    // ── Arithmetic ────────────────────────────────────────────────────────────

    #[test]
    fn arithmetic() { check("arithmetic/02_arithmetic.mtl"); }

    #[test]
    fn float_arithmetic() { check("arithmetic/03_float_arithmetic.mtl"); }

    #[test]
    fn comparison() { check("arithmetic/04_comparison.mtl"); }

    #[test]
    fn logical() { check("arithmetic/05_logical.mtl"); }

    #[test]
    fn unary() { check("arithmetic/06_unary.mtl"); }

    #[test]
    fn range() { check("arithmetic/07_range.mtl"); }

    #[test]
    fn neg_div_by_zero() { check("arithmetic/neg_01_div_by_zero.mtl"); }

    #[test]
    fn neg_rem_by_zero() { check("arithmetic/neg_02_rem_by_zero.mtl"); }

    // ── Types (arrays, tuples, casts) ─────────────────────────────────────────

    #[test]
    fn cast() { check("types/08_cast.mtl"); }

    #[test]
    fn tuple() { check("types/09_tuple.mtl"); }

    #[test]
    fn array() { check("types/10_array.mtl"); }

    #[test]
    fn from_cast() { check("types/60_from_cast.mtl"); }

    #[test]
    fn from_edge_cases() { check("types/63_from_edge_cases.mtl"); }

    #[test]
    fn neg_array_oob() { check("types/neg_03_array_oob.mtl"); }

    #[test]
    fn neg_array_negative_index() { check("types/neg_04_array_negative_index.mtl"); }

    #[test]
    fn neg_array_index_at_len() { check("types/neg_05_array_index_at_len.mtl"); }

    #[test]
    fn neg_cast_float_to_int() { check("types/neg_08_cast_float_to_int.mtl"); }

    #[test]
    fn neg_tuple_oob() { check("types/neg_09_tuple_oob.mtl"); }

    #[test]
    fn neg_cast_no_from() { check("types/neg_23_cast_no_from.mtl"); }

    // ── Control flow ──────────────────────────────────────────────────────────

    #[test]
    fn if_expression() { check("control_flow/12_if_expression.mtl"); }

    #[test]
    fn loop_expr() { check("control_flow/13_loop.mtl"); }

    #[test]
    fn match_expr() { check("control_flow/14_match.mtl"); }

    #[test]
    fn while_loop() { check("control_flow/15_while.mtl"); }

    #[test]
    fn for_loop() { check("control_flow/16_for_loop.mtl"); }

    #[test]
    fn for_in() { check("control_flow/17_for_in.mtl"); }

    #[test]
    fn loop_if_break() { check("control_flow/35_loop_if_break.mtl"); }

    #[test]
    fn braceless_if() { check("control_flow/47_braceless_if.mtl"); }

    #[test]
    fn match_arm_bare_return() { check("control_flow/65_match_arm_bare_return.mtl"); }

    #[test]
    fn neg_no_arm() { check("control_flow/neg_06_no_arm.mtl"); }

    #[test]
    fn neg_and_rhs_evaluated() { check("control_flow/neg_10_and_rhs_evaluated.mtl"); }

    #[test]
    fn neg_or_rhs_evaluated() { check("control_flow/neg_11_or_rhs_evaluated.mtl"); }

    #[test]
    fn neg_nonexhaustive_match() { check("control_flow/neg_13_nonexhaustive_match.mtl"); }

    #[test]
    fn neg_braceless_if_dangling_else() { check("control_flow/neg_19_braceless_if_dangling_else.mtl"); }

    #[test]
    fn neg_braceless_if_mixed_arms() { check("control_flow/neg_20_braceless_if_mixed_arms.mtl"); }

    // ── Functions ─────────────────────────────────────────────────────────────

    #[test]
    fn return_stmt() { check("functions/18_return.mtl"); }

    #[test]
    fn nested_signals() { check("functions/19_nested_signals.mtl"); }

    #[test]
    fn scoping() { check("functions/20_scoping.mtl"); }

    #[test]
    fn assign() { check("functions/21_assign.mtl"); }

    #[test]
    fn misc() { check("functions/22_misc.mtl"); }

    #[test]
    fn forward_reference() { check("functions/23_forward_reference.mtl"); }

    #[test]
    fn call() { check("functions/31_call.mtl"); }

    #[test]
    fn recursive() { check("functions/32_recursive.mtl"); }

    #[test]
    fn call_edge() { check("functions/36_call_edge.mtl"); }

    #[test]
    fn neg_no_main() { check("functions/neg_07_no_main.mtl"); }

    #[test]
    fn neg_stack_single_frame() { check("functions/neg_14_stack_single_frame.mtl"); }

    #[test]
    fn neg_stack_outer_frame() { check("functions/neg_15_stack_outer_frame.mtl"); }

    #[test]
    fn neg_stack_deep_chain() { check("functions/neg_16_stack_deep_chain.mtl"); }

    #[test]
    fn neg_stack_recursive() { check("functions/neg_17_stack_recursive.mtl"); }

    #[test]
    fn neg_stack_closure_frame() { check("functions/neg_18_stack_closure_frame.mtl"); }

    // ── Closures ──────────────────────────────────────────────────────────────

    #[test]
    fn closure() { check("closures/33_closure.mtl"); }

    #[test]
    fn closure_edge() { check("closures/37_closure_edge.mtl"); }

    #[test]
    fn closures_advanced() { check("closures/42_closures_advanced.mtl"); }

    #[test]
    fn closure_capture_and_pointers() { check("functions/66_closure_capture_and_pointers.mtl"); }

    #[test]
    fn closure_shared_mutable_refs() { check("functions/67_closure_shared_mutable_refs.mtl"); }

    #[test]
    fn closure_different_calls() { check("functions/68_closure_different_calls.mtl"); }

    #[test]
    fn nice_closure_abuse() { check("functions/69_nice_closure_abuse.mtl"); }

    #[test]
    fn fun_ptr_unification() { check("functions/70_fun_ptr_unification.mtl"); }

    #[test]
    fn fun_ptr_autoderef() { check("functions/71_fun_ptr_autodref.mtl"); }

    // ── Structs ───────────────────────────────────────────────────────────────

    #[test]
    fn struct_literal() { check("structs/24_struct_literal.mtl"); }

    #[test]
    fn field_access() { check("structs/26_field_access.mtl"); }

    #[test]
    fn method_call_builtin() { check("structs/27_method_call_builtin.mtl"); }

    #[test]
    fn method_call_user() { check("structs/28_method_call_user.mtl"); }

    #[test]
    fn assign_index() { check("structs/29_assign_index.mtl"); }

    #[test]
    fn assign_field() { check("structs/30_assign_field.mtl"); }

    #[test]
    fn method_chain() { check("structs/40_method_chain.mtl"); }

    #[test]
    fn nested_struct() { check("structs/41_nested_struct.mtl"); }

    #[test]
    fn shorthand_field() { check("structs/43_shorthand_field.mtl"); }

    #[test]
    fn trailing_commas() { check("structs/44_trailing_commas.mtl"); }

    #[test]
    fn lvalue_paths() { check("structs/45_lvalue_paths.mtl"); }

    #[test]
    fn local_struct_scope() { check("structs/46_local_struct_scope.mtl"); }

    #[test]
    fn self_method_signatures() { check("structs/48_self_method_signatures.mtl"); }

    #[test]
    fn receiver_references() { check("structs/67_receiver_references.mtl"); }

    #[test]
    fn receiver_all_forms() { check("structs/68_receiver_all_forms.mtl"); }

    #[test]
    fn neg_ref_mut_through_shared_ptr() { check("structs/neg_24_ref_mut_method_through_shared_ptr.mtl"); }

    #[test]
    fn neg_missing_field() { check("structs/neg_12_missing_field.mtl"); }

    // ── Enums ─────────────────────────────────────────────────────────────────

    #[test]
    fn enum_variant() { check("enums/11_enum_variant.mtl"); }

    #[test]
    fn enum_with_fields() { check("enums/25_enum_with_fields.mtl"); }

    #[test]
    fn perhaps() { check("enums/39_perhaps.mtl"); }

    // ── Generics ──────────────────────────────────────────────────────────────

    #[test]
    fn generics() { check("generics/48_generics.mtl"); }

    #[test]
    fn generic_consistency() { check("generics/50_generic_consistency.mtl"); }

    #[test]
    fn generic_nested_types() { check("generics/51_generic_nested_types.mtl"); }

    #[test]
    fn let_polymorphism() { check("generics/52_let_polymorphism.mtl"); }

    #[test]
    fn generic_struct() { check("generics/53_generic_struct.mtl"); }

    #[test]
    fn generic_enum_user() { check("generics/54_generic_enum_user.mtl"); }

    #[test]
    fn generic_nested() { check("generics/55_generic_nested.mtl"); }

    #[test]
    fn generic_body_annotation() { check("generics/56_generic_body_annotation.mtl"); }

    #[test]
    fn generic_enum_infer_context() { check("generics/57_generic_enum_infer_context.mtl"); }

    #[test]
    fn none_ascribed_generic_return() { check("generics/58_none_ascribed_generic_return.mtl"); }

    #[test]
    fn neg_generic_type_conflict() { check("generics/neg_21_generic_type_conflict.mtl"); }

    // ── Aspects ───────────────────────────────────────────────────────────────

    #[test]
    fn aspect_dispatch() { check("aspects/58_aspect_dispatch.mtl"); }

    #[test]
    fn iterable_aspect() { check("aspects/59_iterable_aspect.mtl"); }

    #[test]
    fn iterable_edge_cases() { check("aspects/62_iterable_edge_cases.mtl"); }

    #[test]
    fn default_methods() { check("aspects/63_default_methods.mtl"); }

    #[test]
    fn neg_missing_aspect_method() { check("aspects/neg_22_missing_aspect_method.mtl"); }

    // ── Error handling ────────────────────────────────────────────────────────

    #[test]
    fn propagate_error() { check("error_handling/34_propagate_error.mtl"); }

    #[test]
    fn propagate_error_coercion() { check("error_handling/61_propagate_error_coercion.mtl"); }

    #[test]
    fn propagate_error_edge_cases() { check("error_handling/64_propagate_error_edge_cases.mtl"); }

    // ── Builtins ──────────────────────────────────────────────────────────────

    #[test]
    fn builtins() { check("builtins/38_builtins.mtl"); }

    // ── Integration ───────────────────────────────────────────────────────────

    #[test]
    fn int_statistics() { check("integration/int_01_statistics.mtl"); }

    #[test]
    fn int_battle() { check("integration/int_02_battle.mtl"); }

    #[test]
    fn int_aspects() { check("integration/int_03_aspects.mtl"); }

    #[test]
    fn int_generic_option_chain() { check("integration/int_03_generic_option_chain.mtl"); }

    #[test]
    fn int_pipeline() { check("integration/int_04_pipeline.mtl"); }

    #[test]
    fn int_generic_algorithms() { check("integration/int_04_generic_algorithms.mtl"); }

    #[test]
    fn int_aspects_combined() { check("integration/int_05_aspects_combined.mtl"); }

    #[test]
    fn int_generic_data_pipeline() { check("integration/int_05_generic_data_pipeline.mtl"); }

    #[test]
    fn int_display() { check("integration/int_06_display.mtl"); }

    #[test]
    fn int_pub_declarations() { check("integration/int_07_pub_declarations.mtl"); }

    #[test]
    fn int_std_core_paths() { check("integration/int_08_std_core_paths.mtl"); }
}
