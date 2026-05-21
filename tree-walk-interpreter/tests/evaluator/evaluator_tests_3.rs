/// Integration tests for issue #3 / #55 — control flow expressions and statements.
///
/// Assertion pattern: `let _ok = match (actual == expected) { true => 0, };`
/// If the assertion fails, no arm matches and the runtime panics with "no arm matched".

#[cfg(test)]
mod tests {
    use yoloscript::{evaluator, parser, typechecker};

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn run(src: &str) {
        let ast = parser::parse(src, "test").expect("parse error");
        let prog = typechecker::check(ast).expect("typecheck error");
        evaluator::evaluate(prog).expect("runtime error");
    }

    fn run_err(src: &str) -> String {
        let ast = parser::parse(src, "test").expect("parse error");
        let prog = typechecker::check(ast).expect("typecheck error");
        evaluator::evaluate(prog).expect_err("expected runtime error").to_string()
    }

    // ── If expression ─────────────────────────────────────────────────────────

    #[test]
    fn if_true_branch() {
        run(r#"
            fun main() {
                let x = if (true) { 1 } else { 2 };
                let _ok = match (x == 1) { true => 0, };
            }
        "#);
    }

    #[test]
    fn if_false_branch() {
        run(r#"
            fun main() {
                let x = if (false) { 1 } else { 2 };
                let _ok = match (x == 2) { true => 0, };
            }
        "#);
    }

    #[test]
    fn if_no_else_returns_unit() {
        // if without else in statement position — runs without error.
        run(r#"
            fun main() {
                if (false) { };
            }
        "#);
    }

    // ── Loop expression ───────────────────────────────────────────────────────

    #[test]
    fn loop_break_with_value() {
        run(r#"
            fun main() {
                let x = loop { break 42; };
                let _ok = match (x == 42) { true => 0, };
            }
        "#);
    }

    #[test]
    fn loop_continue_skips_iteration() {
        run(r#"
            fun main() {
                mut count = 0;
                mut i = 0;
                loop {
                    if (i >= 5) { break; }
                    i = i + 1;
                    if (i == 3) { continue; }
                    count = count + 1;
                }
                let _ok = match (count == 4) { true => 0, };
            }
        "#);
    }

    // ── Match expression ──────────────────────────────────────────────────────

    #[test]
    fn match_literal_pattern() {
        run(r#"
            fun main() {
                let x = 2;
                let result = match x {
                    1 => 10,
                    2 => 20,
                    _ => 30,
                };
                let _ok = match (result == 20) { true => 0, };
            }
        "#);
    }

    #[test]
    fn match_wildcard_pattern() {
        run(r#"
            fun main() {
                let result = match 99 {
                    1 => 1,
                    _ => 0,
                };
                let _ok = match (result == 0) { true => 0, };
            }
        "#);
    }

    #[test]
    fn match_binding_pattern() {
        run(r#"
            fun main() {
                let result = match 7 {
                    n => n + 1,
                };
                let _ok = match (result == 8) { true => 0, };
            }
        "#);
    }

    #[test]
    fn match_enum_variant_pattern() {
        run(r#"
            enum Color { Red, Green, Blue }
            fun main() {
                let c = Color::Green;
                let result = match c {
                    Color::Red   => 1,
                    Color::Green => 2,
                    Color::Blue  => 3,
                };
                let _ok = match (result == 2) { true => 0, };
            }
        "#);
    }

    #[test]
    fn match_guard() {
        run(r#"
            fun main() {
                let x = 5;
                let result = match x {
                    n if n < 3 => 0,
                    n if n < 7 => 1,
                    _          => 2,
                };
                let _ok = match (result == 1) { true => 0, };
            }
        "#);
    }

    #[test]
    fn match_no_arm_is_runtime_error() {
        let err = run_err(r#"
            enum Coin { Heads, Tails }
            fun main() {
                let c = Coin::Tails;
                let _x = match c {
                    Coin::Heads => 1,
                };
            }
        "#);
        assert!(err.contains("no arm matched"), "expected no-arm error, got: {err}");
    }

    // ── While statement ───────────────────────────────────────────────────────

    #[test]
    fn while_loop() {
        run(r#"
            fun main() {
                mut i = 0;
                while (i < 5) {
                    i = i + 1;
                }
                let _ok = match (i == 5) { true => 0, };
            }
        "#);
    }

    #[test]
    fn while_break() {
        run(r#"
            fun main() {
                mut i = 0;
                while (true) {
                    if (i == 3) { break; }
                    i = i + 1;
                }
                let _ok = match (i == 3) { true => 0, };
            }
        "#);
    }

    // ── For (C-style) statement ───────────────────────────────────────────────

    #[test]
    fn for_loop_counting() {
        run(r#"
            fun main() {
                mut sum = 0;
                for (mut i = 0; i < 5; i += 1) {
                    sum += i;
                }
                let _ok = match (sum == 10) { true => 0, };
            }
        "#);
    }

    #[test]
    fn for_loop_break() {
        run(r#"
            fun main() {
                mut count = 0;
                for (mut i = 0; i < 100; i += 1) {
                    if (i == 5) { break; }
                    count += 1;
                }
                let _ok = match (count == 5) { true => 0, };
            }
        "#);
    }

    // ── For-in statement ─────────────────────────────────────────────────────

    #[test]
    fn for_in_array() {
        run(r#"
            fun main() {
                let arr = [10, 20, 30];
                mut sum = 0;
                for (let x in arr) {
                    sum += x;
                }
                let _ok = match (sum == 60) { true => 0, };
            }
        "#);
    }

    #[test]
    fn for_in_range() {
        run(r#"
            fun main() {
                mut sum = 0;
                for (let i in 0..5) {
                    sum += i;
                }
                let _ok = match (sum == 10) { true => 0, };
            }
        "#);
    }

    #[test]
    fn for_in_range_inclusive() {
        run(r#"
            fun main() {
                mut sum = 0;
                for (let i in 1..=4) {
                    sum += i;
                }
                let _ok = match (sum == 10) { true => 0, };
            }
        "#);
    }

    // ── Return statement ──────────────────────────────────────────────────────

    #[test]
    fn early_return() {
        run(r#"
            fun main() {
                mut i = 0;
                while (true) {
                    if (i == 3) { return; }
                    i = i + 1;
                }
            }
        "#);
    }

    // ── Program entry point ───────────────────────────────────────────────────

    #[test]
    fn no_main_is_error() {
        let ast = parser::parse("let x = 1;", "test").expect("parse error");
        let prog = typechecker::check(ast).expect("typecheck error");
        let err = evaluator::evaluate(prog).expect_err("expected error");
        assert!(err.to_string().contains("no main"), "expected no-main error, got: {err}");
    }

    #[test]
    fn forward_reference_to_fun() {
        // main() is defined before helper(), but helper is hoisted so the call
        // (once #4 is done) will find it. For now, verify the program typechecks
        // and the entry point runs without the function-call eval crashing during
        // evaluate()'s hoist pass.
        run(r#"
            fun main() {
                let x = 1 + 1;
                let _ok = match (x == 2) { true => 0, };
            }
            fun helper() { }
        "#);
    }
}
