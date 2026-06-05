/// Integration tests for the v0.6.0 module semantics pipeline:
/// `load_root → resolve → normalize → check_graph → evaluate_graph`

use std::fs;
use std::path::{Path, PathBuf};

use metel::{evaluator, module_loader, name_resolver, path_normalizer, typechecker};

fn fixture_dir(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "metel_module_semantics_{}_{}_{}",
        std::process::id(),
        nonce,
        name,
    ));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("failed to create {}: {e}", dir.display()));
    dir
}

fn write(path: &Path, source: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|e| panic!("failed to create {}: {e}", parent.display()));
    }
    fs::write(path, source).unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
}

fn run_graph(main: &Path) -> Result<(), metel::error::MetelError> {
    let graph = module_loader::load_root(main)?;
    let names = name_resolver::resolve(&graph)?;
    let normalized = path_normalizer::normalize(graph, &names)?;
    let typed = typechecker::check_graph(normalized, &names, typechecker::StdPrelude::empty())?;
    evaluator::evaluate_graph(typed)
}

fn run_graph_std(main: &Path) -> Result<(), metel::error::MetelError> {
    let graph = module_loader::load_root(main)?;
    let names = name_resolver::resolve(&graph)?;
    let normalized = path_normalizer::normalize(graph, &names)?;
    let typed = typechecker::check_graph(normalized, &names, typechecker::StdPrelude::default())?;
    evaluator::evaluate_graph(typed)
}

// ── Basic single-module graph ─────────────────────────────────────────────────

#[test]
fn single_module_check_graph_runs() {
    let dir = fixture_dir("single");
    let main = dir.join("main.mtl");
    write(&main, "fun main() { }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn single_module_with_arithmetic() {
    let dir = fixture_dir("arith");
    let main = dir.join("main.mtl");
    write(&main, "fun main() -> i64 { return 1 + 2; }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── Multi-module graph ────────────────────────────────────────────────────────

#[test]
fn two_module_function_call() {
    let dir = fixture_dir("two_mod");
    let main = dir.join("main.mtl");
    write(&main, "import helper::*;\nfun main() -> i64 { return answer(); }\n");
    write(&dir.join("helper.mtl"), "pub fun answer() -> i64 { return 42; }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn explicit_named_import_function_call() {
    let dir = fixture_dir("named_import");
    let main = dir.join("main.mtl");
    write(&main, "import helper::answer;\nfun main() -> i64 { return answer(); }\n");
    write(&dir.join("helper.mtl"), "pub fun answer() -> i64 { return 7; }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn struct_imported_via_glob() {
    let dir = fixture_dir("struct_glob");
    let main = dir.join("main.mtl");
    write(
        &main,
        r#"
import point::*;
fun main() -> i64 {
    let p = Point { x: 3, y: 4 };
    return p.x;
}
"#,
    );
    write(&dir.join("point.mtl"), "pub struct Point { pub x: i64, pub y: i64 }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn transitive_dependency_via_graph_pipeline() {
    let dir = fixture_dir("transitive_graph");
    let main = dir.join("main.mtl");
    write(&main, "import parser::*;\nfun main() -> i64 { return parse(); }\n");
    write(
        &dir.join("parser.mtl"),
        "import lexer::*;\npub fun parse() -> i64 { return tokenize(); }\n",
    );
    write(&dir.join("lexer.mtl"), "pub fun tokenize() -> i64 { return 1; }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── #176: glob import filters to public items only ───────────────────────────

#[test]
fn glob_import_makes_pub_items_accessible() {
    let dir = fixture_dir("glob_pub");
    let main = dir.join("main.mtl");
    write(&main, "import helper::*;\nfun main() -> i64 { return pub_fn(); }\n");
    write(&dir.join("helper.mtl"), "pub fun pub_fn() -> i64 { return 1; }\nfun private_fn() -> i64 { return 2; }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn glob_import_does_not_expose_private_items() {
    let dir = fixture_dir("glob_priv");
    let main = dir.join("main.mtl");
    // `private_fn` is not pub — should not be callable even after `import helper::*`
    write(&main, "import helper::*;\nfun main() -> i64 { return private_fn(); }\n");
    write(&dir.join("helper.mtl"), "pub fun pub_fn() -> i64 { return 1; }\nfun private_fn() -> i64 { return 2; }\n");

    run_graph(&main).expect_err("private item via glob should fail");
}

// ── #175: alias resolution ────────────────────────────────────────────────────

#[test]
fn alias_import_makes_alias_callable() {
    let dir = fixture_dir("alias_ok");
    let main = dir.join("main.mtl");
    write(&main, "import helper::answer as compute;\nfun main() -> i64 { return compute(); }\n");
    write(&dir.join("helper.mtl"), "pub fun answer() -> i64 { return 42; }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn alias_import_original_name_not_in_scope() {
    // `answer` should not be resolvable after `import helper::answer as compute`
    let dir = fixture_dir("alias_orig_out");
    let main = dir.join("main.mtl");
    write(&main, "import helper::answer as compute;\nfun main() -> i64 { return answer(); }\n");
    write(&dir.join("helper.mtl"), "pub fun answer() -> i64 { return 42; }\n");

    // `answer` is not imported — only `compute` is. Should fail (T0003 or unresolved).
    run_graph(&main).expect_err("original name should not be in scope");
}

// ── #178: re-export propagation ──────────────────────────────────────────────

#[test]
fn facade_re_exports_item_and_consumer_can_use_it() {
    // facade.mtl re-exports `answer` from helper.mtl.
    // main.mtl imports only from facade and calls `answer` without importing helper directly.
    let dir = fixture_dir("re_export");
    let main = dir.join("main.mtl");
    write(&main, "import facade::answer;\nfun main() -> i64 { return answer(); }\n");
    // facade imports answer from helper (so helper is loaded) and re-exports it
    write(&dir.join("facade.mtl"), "import helper::answer;\nexport helper::answer;\n");
    write(&dir.join("helper.mtl"), "pub fun answer() -> i64 { return 42; }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── T0011: import conflict detection ─────────────────────────────────────────

#[test]
fn two_explicit_imports_same_local_name_is_t0011() {
    let dir = fixture_dir("t0011_explicit");
    let main = dir.join("main.mtl");
    // Both `import a::foo` and `import b::foo` bind local name `foo` → conflict
    write(&main, "import a::foo;\nimport b::foo;\nfun main() -> i64 { return foo(); }\n");
    write(&dir.join("a.mtl"), "pub fun foo() -> i64 { return 1; }\n");
    write(&dir.join("b.mtl"), "pub fun foo() -> i64 { return 2; }\n");

    let err = run_graph(&main).expect_err("expected T0011");
    let msg = format!("{err}");
    assert!(msg.contains("T0011"), "expected T0011, got: {msg}");
}

#[test]
fn two_glob_imports_same_name_is_t0011() {
    let dir = fixture_dir("t0011_glob");
    let main = dir.join("main.mtl");
    write(&main, "import a::*;\nimport b::*;\nfun main() -> i64 { return foo(); }\n");
    write(&dir.join("a.mtl"), "pub fun foo() -> i64 { return 1; }\n");
    write(&dir.join("b.mtl"), "pub fun foo() -> i64 { return 2; }\n");

    let err = run_graph(&main).expect_err("expected T0011 on glob/glob conflict");
    let msg = format!("{err}");
    assert!(msg.contains("T0011"), "expected T0011, got: {msg}");
}

#[test]
fn explicit_import_wins_over_glob_same_name() {
    // Explicit import silently wins over glob that exports the same name.
    let dir = fixture_dir("t0011_explicit_wins");
    let main = dir.join("main.mtl");
    write(&main, "import a::foo;\nimport b::*;\nfun main() -> i64 { return foo(); }\n");
    write(&dir.join("a.mtl"), "pub fun foo() -> i64 { return 1; }\n");
    write(&dir.join("b.mtl"), "pub fun foo() -> i64 { return 2; }\n");

    // Should succeed — explicit import from `a` wins
    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn user_glob_wins_over_std_glob_same_name_no_t0011() {
    // Simulates the std::core auto-import scenario: a Std-tier glob and a User-tier
    // glob both export the same name. The User glob must win silently — no T0011.
    // (In production this will be triggered by std::core exporting `print`, `println`,
    //  etc. while user modules may also export or re-export those names.)
    //
    // We test the tier model indirectly by injecting a Std-tier glob directly into
    // the scope before name resolution runs, since there is no user syntax for Std globs.
    // The integration test is in evaluator tests once #201 lands.
    //
    // For now: two User globs with the same name *do* produce T0011 (unchanged),
    // and the test above covers that. This test documents the intended behaviour
    // for cross-tier resolution which will be exercised end-to-end by #201.
    //
    // Placeholder: this test will be expanded to a real end-to-end case in #201.
    let dir = fixture_dir("tier_model_placeholder");
    let main = dir.join("main.mtl");
    write(&main, "import a::*;\nfun main() -> i64 { return foo(); }\n");
    write(&dir.join("a.mtl"), "pub fun foo() -> i64 { return 42; }\n");
    // Single User glob — no conflict possible, must succeed.
    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── std::core auto-import (RFC-0030) ─────────────────────────────────────────

#[test]
fn print_available_without_explicit_import() {
    // print() must be in scope in every module without any import statement.
    let dir = fixture_dir("auto_import_print");
    let main = dir.join("main.mtl");
    write(&main, "fun main() { print(42); }\n");
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn explicit_std_core_import_is_valid() {
    // `import std::core::print` should work and bring print into scope.
    let dir = fixture_dir("explicit_std_import");
    let main = dir.join("main.mtl");
    write(&main, "import std::core::print;\nfun main() { print(\"hi\"); }\n");
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn local_function_shadows_std_core_auto_import() {
    // A user-defined `print` function must shadow the auto-imported std::core::print.
    let dir = fixture_dir("shadow_std_print");
    let main = dir.join("main.mtl");
    write(&main, "fun print(x: i64) -> i64 { return x + 1; }\nfun main() { print(1); }\n");
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── std::core type declarations (#202) ───────────────────────────────────────

#[test]
fn import_std_core_perhaps_is_valid() {
    // `import std::core::Perhaps` must not error.
    let dir = fixture_dir("import_perhaps");
    let main = dir.join("main.mtl");
    write(&main, "import std::core::Perhaps;\nfun main() { let x = Perhaps::Some { value: 1 }; }\n");
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn import_std_core_group_is_valid() {
    // `import std::core::{Perhaps, Result}` must not error.
    let dir = fixture_dir("import_perhaps_result");
    let main = dir.join("main.mtl");
    write(&main, "import std::core::{Perhaps, Result};\nfun main() { let x = Perhaps::Some { value: 1 }; }\n");
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn std_core_perhaps_path_in_struct_literal() {
    // `std::core::Perhaps::Some { value: 42 }` must be resolved to `Perhaps::Some { value: 42 }`.
    let dir = fixture_dir("std_path_struct_lit");
    let main = dir.join("main.mtl");
    write(&main, "fun main() { let x = std::core::Perhaps::Some { value: 42 }; }\n");
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn programs_without_explicit_std_import_still_use_perhaps() {
    // Programs that never mention std::core must still be able to use Perhaps and Result.
    let dir = fixture_dir("no_std_import_perhaps");
    let main = dir.join("main.mtl");
    write(&main, "fun main() { let x = Perhaps::Some { value: 1 }; }\n");
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── T0009: visibility enforcement ────────────────────────────────────────────

#[test]
fn importing_private_item_is_t0009() {
    let dir = fixture_dir("t0009_private");
    let main = dir.join("main.mtl");
    write(&main, "import helper::secret;\nfun main() -> i64 { return secret(); }\n");
    write(&dir.join("helper.mtl"), "fun secret() -> i64 { return 42; }\n");

    let err = run_graph(&main).expect_err("expected T0009");
    let msg = format!("{err}");
    assert!(msg.contains("T0009"), "expected T0009, got: {msg}");
}

#[test]
fn importing_nonexistent_name_is_t0003() {
    let dir = fixture_dir("t0003_absent");
    let main = dir.join("main.mtl");
    write(&main, "import helper::nonexistent;\nfun main() -> i64 { return nonexistent(); }\n");
    write(&dir.join("helper.mtl"), "pub fun answer() -> i64 { return 42; }\n");

    let err = run_graph(&main).expect_err("expected T0003");
    let msg = format!("{err}");
    assert!(msg.contains("T0003"), "expected T0003, got: {msg}");
}

#[test]
fn importing_pub_item_is_accepted() {
    let dir = fixture_dir("t0009_pub_ok");
    let main = dir.join("main.mtl");
    write(&main, "import helper::answer;\nfun main() -> i64 { return answer(); }\n");
    write(&dir.join("helper.mtl"), "pub fun answer() -> i64 { return 42; }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn private_struct_field_access_across_modules_is_t0009() {
    let dir = fixture_dir("t0009_private_struct_field_access");
    let main = dir.join("main.mtl");
    write(
        &main,
        "import token::make;\nfun main() -> i64 { return make().offset; }\n",
    );
    write(
        &dir.join("token.mtl"),
        "pub struct Token { pub kind: i64, offset: i64 }\npub fun make() -> Token { return Token { kind: 1, offset: 7 }; }\n",
    );

    let err = run_graph(&main).expect_err("expected T0009");
    let msg = format!("{err}");
    assert!(msg.contains("T0009"), "expected T0009, got: {msg}");
}

#[test]
fn private_struct_field_construction_across_modules_is_t0009() {
    let dir = fixture_dir("t0009_private_struct_field_construction");
    let main = dir.join("main.mtl");
    write(
        &main,
        "import token::Token;\nfun main() { let t = Token { kind: 1, offset: 7 }; print(t.kind); }\n",
    );
    write(
        &dir.join("token.mtl"),
        "pub struct Token { pub kind: i64, offset: i64 }\n",
    );

    let err = run_graph(&main).expect_err("expected T0009");
    let msg = format!("{err}");
    assert!(msg.contains("T0009"), "expected T0009, got: {msg}");
}

#[test]
fn private_struct_field_assignment_across_modules_is_t0009() {
    let dir = fixture_dir("t0009_private_struct_field_assign");
    let main = dir.join("main.mtl");
    write(
        &main,
        "import token::make;\nfun main() -> i64 { let mut t = make(); t.offset = 9; return t.kind; }\n",
    );
    write(
        &dir.join("token.mtl"),
        "pub struct Token { pub kind: i64, offset: i64 }\npub fun make() -> Token { return Token { kind: 1, offset: 7 }; }\n",
    );

    let err = run_graph(&main).expect_err("expected T0009");
    let msg = format!("{err}");
    assert!(msg.contains("T0009"), "expected T0009, got: {msg}");
}

#[test]
fn mixed_visibility_struct_allows_public_field_access_across_modules() {
    let dir = fixture_dir("mixed_visibility_public_field_ok");
    let main = dir.join("main.mtl");
    write(
        &main,
        "import token::make;\nfun main() -> i64 { let t = make(); return t.kind; }\n",
    );
    write(
        &dir.join("token.mtl"),
        "pub struct Token { pub kind: i64, offset: i64 }\npub fun make() -> Token { return Token { kind: 11, offset: 7 }; }\n",
    );

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn private_struct_fields_remain_accessible_inside_declaring_module() {
    let dir = fixture_dir("private_struct_field_same_module_ok");
    let main = dir.join("main.mtl");
    write(
        &main,
        "pub struct Token { pub kind: i64, offset: i64 }\n\
         fun offset_of(t: Token) -> i64 { return t.offset; }\n\
         fun main() -> i64 { let t = Token { kind: 3, offset: 9 }; return offset_of(t); }\n",
    );

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── T0010: pub declarations require explicit annotations ──────────────────────

#[test]
fn pub_fun_without_return_type_is_t0010() {
    let dir = fixture_dir("t0010_no_return");
    let main = dir.join("main.mtl");
    write(&main, "import helper::*;\nfun main() -> i64 { return answer(); }\n");
    write(&dir.join("helper.mtl"), "pub fun answer() { return 42; }\n");

    let err = run_graph(&main).expect_err("expected T0010 error");
    let msg = format!("{err}");
    assert!(msg.contains("T0010"), "expected T0010, got: {msg}");
}

#[test]
fn pub_fun_with_unannotated_param_is_t0010() {
    let dir = fixture_dir("t0010_no_param_ann");
    let main = dir.join("main.mtl");
    write(&main, "import helper::*;\nfun main() -> i64 { return double(2); }\n");
    write(&dir.join("helper.mtl"), "pub fun double(x) -> i64 { return x * 2; }\n");

    let err = run_graph(&main).expect_err("expected T0010 error");
    let msg = format!("{err}");
    assert!(msg.contains("T0010"), "expected T0010, got: {msg}");
}

#[test]
fn non_pub_fun_without_annotation_is_accepted() {
    let dir = fixture_dir("t0010_non_pub_ok");
    let main = dir.join("main.mtl");
    write(&main, "import helper::*;\nfun main() -> i64 { return call(); }\n");
    write(
        &dir.join("helper.mtl"),
        "fun internal() { }\npub fun call() -> i64 { return 0; }\n",
    );

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── Collision detection ───────────────────────────────────────────────────────

#[test]
fn private_names_in_different_modules_do_not_collide() {
    // Two modules each declare a private function with the same name.
    // Neither is exported (so no T0011 import conflict). With per-module
    // isolated environments each `helper` lives only in its own module's env —
    // no collision, no warning.
    let dir = fixture_dir("collision");
    let main = dir.join("main.mtl");
    write(
        &main,
        "import a::pub_a;\nimport b::pub_b;\nfun main() -> i64 { return pub_a() + pub_b(); }\n",
    );
    write(&dir.join("a.mtl"), "pub fun pub_a() -> i64 { return 1; }\nfun helper() -> i64 { return 10; }\n");
    write(&dir.join("b.mtl"), "pub fun pub_b() -> i64 { return 2; }\nfun helper() -> i64 { return 20; }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── Path normalization ────────────────────────────────────────────────────────

#[test]
fn qualified_call_normalized_to_bare_name() {
    // helper::answer() — the normalizer rewrites this to a bare `answer` lookup
    let dir = fixture_dir("qual_norm");
    let main = dir.join("main.mtl");
    write(&main, "import helper::*;\nfun main() -> i64 { return helper::answer(); }\n");
    write(&dir.join("helper.mtl"), "pub fun answer() -> i64 { return 99; }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn self_qualified_call_normalized() {
    let dir = fixture_dir("self_norm");
    let main = dir.join("main.mtl");
    write(&main, "fun answer() -> i64 { return 5; }\nfun main() -> i64 { return self::answer(); }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── #181: remaining integration coverage ─────────────────────────────────────

#[test]
fn explicit_import_limits_scope_to_named_item() {
    // `import helper::answer` should make `answer` callable but not `other`.
    let dir = fixture_dir("explicit_scope_limit");
    let main = dir.join("main.mtl");
    write(&main, "import helper::answer;\nfun main() -> i64 { return other(); }\n");
    write(
        &dir.join("helper.mtl"),
        "pub fun answer() -> i64 { return 1; }\npub fun other() -> i64 { return 2; }\n",
    );

    run_graph(&main).expect_err("non-imported name should not be in scope");
}

#[test]
fn transitive_item_not_accessible_without_direct_import() {
    // main imports parser; parser imports lexer.
    // main should NOT be able to call tokenize() from lexer without importing lexer directly.
    let dir = fixture_dir("transitive_isolation");
    let main = dir.join("main.mtl");
    write(&main, "import parser::*;\nfun main() -> i64 { return tokenize(); }\n");
    write(
        &dir.join("parser.mtl"),
        "import lexer::*;\npub fun parse() -> i64 { return tokenize(); }\n",
    );
    write(&dir.join("lexer.mtl"), "pub fun tokenize() -> i64 { return 1; }\n");

    run_graph(&main).expect_err("transitive item should not be accessible without direct import");
}

#[test]
fn root_qualified_path_in_non_root_module() {
    // parser.mtl uses root::helper to resolve a sibling module from the root namespace.
    let dir = fixture_dir("root_path");
    let main = dir.join("main.mtl");
    write(&main, "import parser::*;\nfun main() -> i64 { return parse(); }\n");
    write(
        &dir.join("parser.mtl"),
        "import root::helper::*;\npub fun parse() -> i64 { return helper_fn(); }\n",
    );
    write(&dir.join("helper.mtl"), "pub fun helper_fn() -> i64 { return 7; }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── v0.6.0 cross-feature combination tests ───────────────────────────────────

#[test]
fn pub_alias_and_re_export_combined() {
    // Exercises: T0010 (pub must be annotated), alias resolution,
    // and re-export propagation all in one program.
    let dir = fixture_dir("combined_v060");
    let main = dir.join("main.mtl");
    // main imports via alias AND via re-exported name from facade
    write(
        &main,
        "import facade::compute;\nimport util::answer as get_answer;\nfun main() -> i64 { return compute() + get_answer(); }\n",
    );
    // facade re-exports `compute` from impl module
    write(
        &dir.join("facade.mtl"),
        "import impl_mod::compute;\nexport impl_mod::compute;\n",
    );
    write(&dir.join("impl_mod.mtl"), "pub fun compute() -> i64 { return 10; }\n");
    write(&dir.join("util.mtl"), "pub fun answer() -> i64 { return 32; }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn glob_and_explicit_import_from_same_module() {
    // `import a::*` brings pub_a and pub_b; `import a::pub_a` explicitly — explicit wins, no T0011.
    let dir = fixture_dir("glob_explicit_same");
    let main = dir.join("main.mtl");
    write(
        &main,
        "import a::*;\nimport a::pub_a;\nfun main() -> i64 { return pub_a() + pub_b(); }\n",
    );
    write(
        &dir.join("a.mtl"),
        "pub fun pub_a() -> i64 { return 1; }\npub fun pub_b() -> i64 { return 2; }\n",
    );

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn t0010_pub_struct_requires_field_type_annotations() {
    // pub struct fields already require type annotations by grammar; this verifies
    // a properly annotated pub struct compiles and its fields are accessible cross-module.
    let dir = fixture_dir("pub_struct_cross");
    let main = dir.join("main.mtl");
    write(
        &main,
        "import point::Point;\nfun main() -> i64 { let p = Point { x: 5, y: 3 }; return p.x; }\n",
    );
    write(&dir.join("point.mtl"), "pub struct Point { pub x: i64, pub y: i64 }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn group_import_with_alias_subset() {
    // `import math::{add, mul as multiply}` — group import with per-item alias
    let dir = fixture_dir("group_alias");
    let main = dir.join("main.mtl");
    write(
        &main,
        "import math::{add, mul as multiply};\nfun main() -> i64 { return add(multiply(3, 4), 10); }\n",
    );
    write(
        &dir.join("math.mtl"),
        "pub fun add(a: i64, b: i64) -> i64 { return a + b; }\npub fun mul(a: i64, b: i64) -> i64 { return a * b; }\n",
    );

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── Sprint 12: std::core auto-import + module interaction ────────────────────

#[test]
fn std_core_builtins_available_in_each_module_without_import() {
    // Every module in a multi-module graph must see std::core builtins (print,
    // assert, List, .len()) without any explicit import statement.
    let dir = fixture_dir("int_std_auto_import");
    let main = dir.join("main.mtl");
    write(
        &dir.join("helper.mtl"),
        "pub fun sum(arr: i64[]) -> i64 {\
         \n    assert(arr.len() > 0);\
         \n    let mut total = 0;\
         \n    let mut i = 0;\
         \n    while (i < arr.len()) { total += arr[i as u64]; i += 1; }\
         \n    return total;\
         \n}\n",
    );
    write(
        &main,
        "import helper::sum;\
         \nfun main() {\
         \n    let arr = [1, 2, 3, 4, 5];\
         \n    let result = sum(arr);\
         \n    assert(result == 15);\
         \n    print(result);\
         \n}\n",
    );
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn user_glob_overrides_std_core_same_name_in_multi_module() {
    // A User-tier glob export of a function with the same name as a std::core
    // builtin wins silently over the Std-tier auto-import (no T0011).
    let dir = fixture_dir("int_user_glob_overrides_std");
    let main = dir.join("main.mtl");
    write(
        &dir.join("mylib.mtl"),
        "pub fun print(x: i64) -> i64 { return x + 1; }\n",
    );
    write(
        &main,
        "import mylib::*;\
         \nfun main() {\
         \n    let result = print(41);\
         \n    assert(result == 42);\
         \n}\n",
    );
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn local_function_overrides_std_core_builtin() {
    // Local declarations beat the std::core auto-glob in both inference and construction.
    let dir = fixture_dir("local_overrides_std");
    let main = dir.join("main.mtl");
    write(
        &main,
        "fun print(x: i64) -> i64 { return x + 1; }\
         \nfun main() -> i64 { return print(41); }\n",
    );
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn multi_module_perhaps_and_result_without_explicit_std_import() {
    // Perhaps and Result are available in every module via std::core auto-import.
    // No explicit `import std::core::Perhaps` should be needed.
    let dir = fixture_dir("int_module_perhaps");
    let main = dir.join("main.mtl");
    write(
        &dir.join("finder.mtl"),
        "pub fun find_first_positive(arr: i64[]) -> Perhaps<i64> {\
         \n    let mut i = 0;\
         \n    while (i < arr.len()) {\
         \n        if (arr[i as u64] > 0) { return Perhaps::Some { value: arr[i as u64] }; }\
         \n        i += 1;\
         \n    }\
         \n    None\
         \n}\n",
    );
    write(
        &main,
        "import finder::find_first_positive;\
         \nfun main() {\
         \n    let arr = [-1, -2, 7, 3];\
         \n    let r = find_first_positive(arr);\
         \n    match r {\
         \n        Perhaps::Some { value } => assert(value == 7),\
         \n        None => assert(false),\
         \n    };\
         \n}\n",
    );
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn explicit_std_core_import_and_auto_glob_coexist() {
    // A module may `import std::core::Perhaps` explicitly while other std::core
    // names (like assert) are still available via the auto-glob.
    let dir = fixture_dir("int_explicit_and_auto");
    let main = dir.join("main.mtl");
    write(
        &main,
        "import std::core::Perhaps;\
         \nfun main() {\
         \n    let p = Perhaps::Some { value: 42 };\
         \n    match p {\
         \n        Perhaps::Some { value } => assert(value == 42),\
         \n        None => assert(false),\
         \n    };\
         \n}\n",
    );
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── #189: cross-module closure capture and pass-ordering correctness ──────────

#[test]
fn cross_module_closure_captures_imported_function() {
    // Module builder returns a higher-order closure that captures `add` from math.
    // Verifies that a closure created in builder's pass-1b correctly holds a
    // real value for the imported function (not a Unit placeholder).
    let dir = fixture_dir("closure_capture_import");
    let main = dir.join("main.mtl");
    write(&dir.join("math.mtl"), "pub fun add(x: i64, y: i64) -> i64 { return x + y; }\n");
    write(
        &dir.join("builder.mtl"),
        "import math::add;\
         \npub fun make_adder(n: i64) -> (i64) -> i64 {\
         \n    return (x: i64) -> i64 { return add(x, n); };\
         \n}\n",
    );
    write(
        &main,
        "import builder::make_adder;\
         \nfun main() {\
         \n    let add5 = make_adder(5);\
         \n    assert(add5(3) == 8);\
         \n    assert(add5(10) == 15);\
         \n}\n",
    );
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn intra_module_recursion_visible_after_cross_module_import() {
    // Module recur has a recursive function count_down.
    // main imports and calls it, verifying that recur's pass-1b correctly
    // set up the self-referencing closure before main seeded it.
    let dir = fixture_dir("mutual_rec_cross");
    let main = dir.join("main.mtl");
    write(
        &dir.join("recur.mtl"),
        "pub fun count_down(n: i64) -> i64 {\
         \n    if (n <= 0) { return 0; }\
         \n    return 1 + count_down(n - 1);\
         \n}\n",
    );
    write(
        &main,
        "import recur::count_down;\
         \nfun main() {\
         \n    assert(count_down(0) == 0);\
         \n    assert(count_down(5) == 5);\
         \n    assert(count_down(10) == 10);\
         \n}\n",
    );
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn two_same_tier_imports_both_captured_in_closure() {
    // main imports from two independent modules (left and right) at the same tier.
    // A closure in main's pass-1b must capture real values from both — not Unit
    // placeholders — even though left and right are initialized in arbitrary order.
    let dir = fixture_dir("same_tier_closure");
    let main = dir.join("main.mtl");
    write(&dir.join("left.mtl"), "pub fun left_val() -> i64 { return 6; }\n");
    write(&dir.join("right.mtl"), "pub fun right_val() -> i64 { return 10; }\n");
    write(
        &main,
        "import left::left_val;\
         \nimport right::right_val;\
         \nfun make_combiner() -> () -> i64 {\
         \n    return () -> i64 { return left_val() + right_val(); };\
         \n}\
         \nfun main() {\
         \n    let f = make_combiner();\
         \n    assert(f() == 16);\
         \n}\n",
    );
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── #228: diamond dependency (same physical file via multiple paths) ───────────

#[test]
fn diamond_dependency_shared_base_accessible_in_both_importers() {
    // base.mtl is reachable from both left.mtl and right.mtl via their imports.
    // Without the path-alias fix, the name resolver would assign base a path that
    // doesn't exist in the registry when loaded via the second importer, causing T0003.
    let dir = fixture_dir("diamond_dep");
    let main = dir.join("main.mtl");
    write(
        &dir.join("base.mtl"),
        "pub fun shared() -> i64 { return 42; }\n",
    );
    write(
        &dir.join("left.mtl"),
        "import base::shared;\npub fun left_answer() -> i64 { return shared(); }\n",
    );
    write(
        &dir.join("right.mtl"),
        "import base::shared;\npub fun right_answer() -> i64 { return shared() + 1; }\n",
    );
    write(
        &main,
        "import left::left_answer;\
         \nimport right::right_answer;\
         \nfun main() {\
         \n    assert(left_answer() == 42);\
         \n    assert(right_answer() == 43);\
         \n}\n",
    );
    run_graph_std(&main).unwrap_or_else(|e| panic!("{e}"));
}

// ── Full-feature multi-module integration ─────────────────────────────────────

#[test]
fn complex_multi_module_task_system() {
    // Four modules exercising: pub enums/structs (T0010), generic cross-module
    // utilities (filter/map/fold/find_first), Result/Perhaps error handling,
    // closures, pattern matching, recursive functions, all loop forms, tuples,
    // type casts, let-polymorphism, closure capture, and chained operations.
    let dir = fixture_dir("task_system");

    // ── types.mtl ─────────────────────────────────────────────────────────────
    write(
        &dir.join("types.mtl"),
        r#"
pub enum Priority { High, Medium, Low }
pub enum Status { Open, InProgress, Done }
pub struct Task { pub title: String, pub priority: Priority, pub status: Status, pub effort: i64 }
pub struct ValidationError { pub message: String }
"#,
    );

    // ── utils.mtl ─────────────────────────────────────────────────────────────
    write(
        &dir.join("utils.mtl"),
        r#"
pub fun fold_left<T, A>(arr: T[], init: A, f: (A, T) -> A) -> A {
    let mut acc = init;
    for (let x in arr) {
        acc = f(acc, x);
    }
    acc
}

pub fun find_first<T>(arr: T[], pred: (T) -> Bool) -> Perhaps<T> {
    for (let x in arr) {
        if (pred(x)) { return Perhaps::Some { value: x }; }
    }
    None
}

pub fun sum_ints(arr: i64[]) -> i64 {
    let mut total = 0;
    for (let x in arr) { total += x; }
    total
}

pub fun filter_array<T>(arr: T[], pred: (T) -> Bool) -> T[] {
    let mut out: List<T> = List::new();
    for (let x in arr) {
        if (pred(x)) { out.push(x); }
    }
    out.as_slice()
}

pub fun map_array<T, U>(arr: T[], f: (T) -> U) -> U[] {
    let mut out: List<U> = List::new();
    for (let x in arr) { out.push(f(x)); }
    out.as_slice()
}
"#,
    );

    // ── tasks.mtl ─────────────────────────────────────────────────────────────
    write(
        &dir.join("tasks.mtl"),
        r#"
import types::*;
import utils::*;

pub fun validate_task(title: String, priority: Priority, status: Status, effort: i64) -> Result<Task, ValidationError> {
    if (string_len(title) == 0) {
        return Result::Err { error: ValidationError { message: "title cannot be empty" } };
    }
    if (effort < 0) {
        return Result::Err { error: ValidationError { message: "effort cannot be negative" } };
    }
    Result::Ok { value: Task { title: title, priority: priority, status: status, effort: effort } }
}

pub fun total_effort(tasks: Task[]) -> i64 {
    fold_left(tasks, 0, (acc: i64, t: Task) -> i64 { acc + t.effort })
}

pub fun open_task_count(tasks: Task[]) -> i64 {
    let mut count = 0;
    for (let t in tasks) {
        match t.status {
            Status::Open       => { count += 1; },
            Status::InProgress => { count += 1; },
            Status::Done       => (),
        };
    }
    count
}

pub fun is_high_priority(t: Task) -> Bool {
    match t.priority {
        Priority::High   => true,
        Priority::Medium => false,
        Priority::Low    => false,
    }
}

pub fun is_done(t: Task) -> Bool {
    match t.status {
        Status::Done       => true,
        Status::Open       => false,
        Status::InProgress => false,
    }
}

pub fun task_titles(tasks: Task[]) -> String[] {
    map_array(tasks, (t: Task) -> String { t.title })
}

pub fun find_by_title(tasks: Task[], title: String) -> Perhaps<Task> {
    find_first(tasks, (t: Task) -> Bool { t.title == title })
}

pub fun compute_completion_pct(tasks: Task[]) -> i64 {
    let n = tasks.len();
    if (n == 0) { return 0; }
    let mut done = 0;
    for (let t in tasks) {
        if (is_done(t)) { done += 1; }
    }
    (done * 100) / n
}
"#,
    );

    // ── main.mtl ──────────────────────────────────────────────────────────────
    write(
        &dir.join("main.mtl"),
        r#"
import types::*;
import utils::*;
import tasks::*;

fun fib(n: i64) -> i64 {
    if (n <= 1) { n }
    else { fib(n - 1) + fib(n - 2) }
}

fun unwrap_task(r: Result<Task, ValidationError>) -> Task {
    match r {
        Result::Ok  { value } => value,
        Result::Err { error } => Task { title: "error", priority: Priority::Low, status: Status::Done, effort: 0 },
    }
}

fun make_tasks() -> Task[] {
    let t1 = unwrap_task(validate_task("Design API",  Priority::High,   Status::Open,       5));
    let t2 = unwrap_task(validate_task("Write tests", Priority::Medium, Status::InProgress, 3));
    let t3 = unwrap_task(validate_task("Deploy",      Priority::High,   Status::Done,       2));
    let t4 = unwrap_task(validate_task("Write docs",  Priority::Low,    Status::Open,       4));
    let t5 = unwrap_task(validate_task("Fix bug",     Priority::High,   Status::InProgress, 1));
    [t1, t2, t3, t4, t5]
}

fun main() {
    // 1. Recursive Fibonacci
    assert(fib(0) == 0);
    assert(fib(1) == 1);
    assert(fib(5) == 5);
    assert(fib(7) == 13);

    // 2. Validation errors from tasks module
    let bad_empty = validate_task("", Priority::High, Status::Open, 5);
    match bad_empty {
        Result::Err { error } => assert(string_len(error.message) > 0),
        Result::Ok  { value } => assert(false),
    };
    let bad_effort = validate_task("T", Priority::Low, Status::Open, -1);
    match bad_effort {
        Result::Err { error } => assert(error.message == "effort cannot be negative"),
        Result::Ok  { value } => assert(false),
    };

    // 3. Build task list (efforts: 5, 3, 2, 4, 1 = 15 total)
    let tasks = make_tasks();
    assert(tasks.len() == 5);

    // 4. total_effort via cross-module fold_left: 5+3+2+4+1 = 15
    assert(total_effort(tasks) == 15);

    // 5. open_task_count: Open(t1,t4) + InProgress(t2,t5) = 4
    assert(open_task_count(tasks) == 4);

    // 6. Generic filter_array with cross-module predicates
    let high = filter_array(tasks, (t: Task) -> Bool { is_high_priority(t) });
    assert(high.len() == 3);

    let done_list = filter_array(tasks, (t: Task) -> Bool { is_done(t) });
    assert(done_list.len() == 1);

    // 7. map_array: extract efforts, sum via sum_ints
    let efforts = map_array(tasks, (t: Task) -> i64 { t.effort });
    assert(sum_ints(efforts) == 15);
    assert(efforts[0] == 5);
    assert(efforts[4] == 1);

    // 8. task_titles (cross-module map_array inside tasks module)
    let titles = task_titles(tasks);
    assert(titles.len() == 5);
    assert(titles[0] == "Design API");
    assert(titles[2] == "Deploy");

    // 9. find_by_title: found
    let found = find_by_title(tasks, "Write tests");
    match found {
        Perhaps::Some { value } => assert(value.effort == 3),
        None                    => assert(false),
    };

    // 10. find_by_title: not found
    let not_found = find_by_title(tasks, "nonexistent");
    match not_found {
        Perhaps::Some { value } => assert(false),
        None                    => (),
    };

    // 11. compute_completion_pct: 1 done / 5 total = 20%
    assert(compute_completion_pct(tasks) == 20);
    let empty_tasks: Task[] = [];
    assert(compute_completion_pct(empty_tasks) == 0);

    // 12. fold_left directly: product of [1,2,3,4] = 24
    let small: i64[] = [1, 2, 3, 4];
    let product = fold_left(small, 1, (acc: i64, x: i64) -> i64 { acc * x });
    assert(product == 24);

    // 13. find_first: locate the Done task (Deploy, effort=2)
    let first_done = find_first(tasks, (t: Task) -> Bool { is_done(t) });
    match first_done {
        Perhaps::Some { value } => assert(value.title == "Deploy"),
        None                    => assert(false),
    };

    // 14. Closure capturing outer variable (min_effort threshold)
    let min_effort = 3;
    let heavy = filter_array(tasks, (t: Task) -> Bool { t.effort >= min_effort });
    assert(heavy.len() == 3);

    // 15. C-style for: manual effort accumulation
    let mut manual_sum = 0;
    for (let mut i = 0; i < tasks.len(); i += 1) {
        manual_sum += tasks[i as u64].effort;
    }
    assert(manual_sum == 15);

    // 16. while loop: count high-priority non-done tasks
    // t1(High,Open) and t5(High,InProgress) qualify; t3(High,Done) does not
    let mut hp_active = 0;
    let mut idx = 0;
    while (idx < tasks.len()) {
        let t = tasks[idx as u64];
        if (is_high_priority(t) && !is_done(t)) { hp_active += 1; }
        idx += 1;
    }
    assert(hp_active == 2);

    // 17. Range for-in: sum 0+1+2+3+4 = 10
    let mut range_sum = 0;
    for (let i in 0..5) { range_sum += i; }
    assert(range_sum == 10);

    // 18. Type cast: total effort i64 -> f64
    let effort_f = total_effort(tasks) as f64;
    assert(effort_f == 15.0);

    // 19. Tuple: pack (task_count, total_effort)
    let report: (i64, i64) = (tasks.len(), total_effort(tasks));
    assert(report.0 == 5);
    assert(report.1 == 15);

    // 20. Chained map + filter across modules
    // doubled efforts: [10,6,4,8,2]; filter > 5: [10,6,8] = 3 items
    let doubled = map_array(efforts, (x: i64) -> i64 { x * 2 });
    let big = filter_array(doubled, (x: i64) -> Bool { x > 5 });
    assert(big.len() == 3);

    // 21. Let-polymorphism: identity closure used at two different types
    let id = (x) -> { x };
    assert(id(tasks[0u64]).effort == 5);
    assert(id(42) == 42);
}
"#,
    );

    run_graph_std(&dir.join("main.mtl")).unwrap_or_else(|e| panic!("{e}"));
}

// ── METEL-3: cross-module struct field type resolution ────────────────────────

/// Module B imports a struct from module A whose field type is defined in C.
/// B never imports C directly. The type registry accumulator must carry C's type
/// definitions into B's registry so the field type resolves. See METEL-3.
#[test]
fn cross_module_struct_field_type_from_indirect_dependency() {
    let dir = fixture_dir("indirect_dep");

    // Module C: defines Coords
    write(
        &dir.join("coords.mtl"),
        "pub struct Coords { pub x: i64, pub y: i64 }\n",
    );

    // Module A: defines Shape whose field type comes from C
    write(
        &dir.join("shape.mtl"),
        r#"import coords::Coords;
pub struct Shape { pub origin: Coords, pub size: i64 }
pub fun make_shape(x: i64, y: i64, s: i64) -> Shape {
    return Shape { origin: Coords { x: x, y: y }, size: s };
}
"#,
    );

    // Module B: imports Shape from A (not Coords from C) and uses it
    write(
        &dir.join("main.mtl"),
        r#"import shape::Shape;
import shape::make_shape;
fun main() -> i64 {
    let s = make_shape(1, 2, 10);
    return s.size;
}
"#,
    );

    run_graph(&dir.join("main.mtl")).unwrap_or_else(|e| panic!("{e}"));
}
