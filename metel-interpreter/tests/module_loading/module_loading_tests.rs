use std::fs;
use std::path::{Path, PathBuf};

use metel::evaluator;
use metel::module_loader;
use metel::name_resolver;
use metel::path_normalizer;
use metel::typechecker;

fn fixture_dir(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "metel_module_loading_{}_{}_{}",
        std::process::id(),
        nonce,
        name,
    ));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("failed to create {}: {e}", dir.display()));
    dir
}

fn write(path: &Path, source: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| panic!("failed to create {}: {e}", parent.display()));
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

#[test]
fn single_file_program_loads_without_modules() {
    let dir = fixture_dir("single");
    let main = dir.join("main.mtl");
    write(&main, "fun main() { }\n");

    let program = module_loader::load_program(&main).unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(program.imports.len(), 0);
    assert_eq!(program.decls.len(), 1);
}

#[test]
fn multi_file_program_loads_declared_modules() {
    let dir = fixture_dir("multi");
    let main = dir.join("main.mtl");
    write(&main, "import parser::Token;\nfun main() { }\n");
    write(&dir.join("parser.mtl"), "pub struct Token { pub value: i64 }\n");

    let graph = module_loader::load_root(&main).unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(graph.modules.len(), 2);
    assert!(graph.modules.iter().any(|m| m.module_path == vec!["parser".to_string()]));
}

#[test]
fn multi_file_program_runs_after_module_loading() {
    let dir = fixture_dir("run_multi");
    let main = dir.join("main.mtl");
    write(&main, "import helper::answer;\nfun main() -> i64 { return answer(); }\n");
    write(&dir.join("helper.mtl"), "pub fun answer() -> i64 { return 42; }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn facade_module_alongside_directory() {
    let dir = fixture_dir("facade");
    let main = dir.join("main.mtl");
    write(&main, "import parser::Token;\nfun main() { }\n");
    // parser.mtl is the facade; parser/ is the namespace — both can coexist
    write(&dir.join("parser.mtl"), "struct Token { value: i64 }\n");
    fs::create_dir_all(dir.join("parser")).unwrap();
    write(&dir.join("parser").join("ast.mtl"), "pub struct Ast { pub value: i64 }\n");

    let graph = module_loader::load_root(&main).unwrap_or_else(|e| panic!("{e}"));

    // main + parser.mtl loaded; parser/ast.mtl not imported so not loaded
    assert_eq!(graph.modules.len(), 2);
    assert!(graph.modules.iter().any(|m| m.module_path == vec!["parser".to_string()]));
}

#[test]
fn rejects_super_from_root_import() {
    let dir = fixture_dir("root_super");
    let main = dir.join("main.mtl");
    write(&main, "import super::parser;\nfun main() { }\n");

    let err = module_loader::load_root(&main).expect_err("super from root should fail");
    let msg = err.to_string();

    assert!(msg.contains("super::"), "message was: {msg}");
    assert!(msg.contains("root module"), "message was: {msg}");
}

#[test]
fn accepts_root_self_super_std_and_child_roots_in_non_root_modules() {
    let dir = fixture_dir("roots");
    let main = dir.join("main.mtl");
    write(&main, "import parser::Token;\nfun main() { }\n");
    write(
        &dir.join("parser.mtl"),
        r#"
import self::child::Thing;
import root::child::Thing;
import super::child::Thing;
import std::core::i64;
import child::Thing;

struct Token { value: i64 }
"#,
    );
    write(&dir.join("child.mtl"), "struct Thing { value: i64 }\n");

    let graph = module_loader::load_root(&main).unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(graph.modules.len(), 3);
}

#[cfg(unix)]
#[test]
fn rejects_circular_module_graph() {
    use std::os::unix::fs::symlink;

    let dir = fixture_dir("cycle");
    let main = dir.join("main.mtl");
    write(&main, "import a::Thing;\nfun main() { }\n");
    write(&dir.join("a.mtl"), "import b::Other;\n");
    // create b/ as a symlink back to a/ to simulate a cycle
    symlink(dir.join("a.mtl"), dir.join("b.mtl"))
        .unwrap_or_else(|e| panic!("failed to create symlink cycle: {e}"));

    let err = module_loader::load_root(&main).expect_err("cycle should fail");
    let msg = err.to_string();

    assert!(msg.contains("circular module dependency"), "message was: {msg}");
}

#[test]
fn qualified_function_call_via_module_handle() {
    let dir = fixture_dir("qual_fn");
    let main = dir.join("main.mtl");
    // import helper::* loads helper.mtl into the graph.
    // helper::answer() uses a qualified path; the path normalizer rewrites it to "answer".
    write(&main, "import helper::*;\nfun main() -> i64 { return helper::answer(); }\n");
    write(&dir.join("helper.mtl"), "pub fun answer() -> i64 { return 42; }\n");

    let graph = module_loader::load_root(&main).unwrap_or_else(|e| panic!("{e}"));
    let names = name_resolver::resolve(&graph).unwrap_or_else(|e| panic!("{e}"));
    let normalized = path_normalizer::normalize(graph, &names).unwrap_or_else(|e| panic!("{e}"));
    let typed = typechecker::check_graph(normalized, &names, typechecker::StdPrelude::empty())
        .unwrap_or_else(|e| panic!("{e}"));
    evaluator::evaluate_graph(typed).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn qualified_type_in_return_signature_typechecks() {
    let dir = fixture_dir("qual_type");
    let main = dir.join("main.mtl");
    // Import Token from helper and use the bare name in the return annotation.
    write(&main, "import helper::*;\nfun wrap(v: i64) -> Token { return Token { value: v }; }\nfun main() -> i64 { let t = wrap(7); return t.value; }\n");
    write(&dir.join("helper.mtl"), "pub struct Token { pub value: i64 }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn self_qualified_path_in_expression_resolves() {
    let dir = fixture_dir("self_path");
    let main = dir.join("main.mtl");
    // self::answer() — Path(["self","answer"]); the path normalizer rewrites it to "answer".
    write(&main, "fun answer() -> i64 { return 99; }\nfun main() -> i64 { return self::answer(); }\n");

    let graph = module_loader::load_root(&main).unwrap_or_else(|e| panic!("{e}"));
    let names = name_resolver::resolve(&graph).unwrap_or_else(|e| panic!("{e}"));
    let normalized = path_normalizer::normalize(graph, &names).unwrap_or_else(|e| panic!("{e}"));
    let typed = typechecker::check_graph(normalized, &names, typechecker::StdPrelude::empty())
        .unwrap_or_else(|e| panic!("{e}"));
    evaluator::evaluate_graph(typed).unwrap_or_else(|e| panic!("{e}"));
}

// ── #169: module system integration tests ────────────────────────────────────

#[test]
fn pub_enum_imported_and_matched() {
    let dir = fixture_dir("enum_match");
    let main = dir.join("main.mtl");
    write(
        &main,
        r#"
import color::*;
fun main() -> i64 {
    let c = Color::Red;
    match c {
        Color::Red   => { return 1; },
        Color::Green => { return 2; },
        Color::Blue  => { return 3; }
    }
}
"#,
    );
    write(&dir.join("color.mtl"), "pub enum Color { Red, Green, Blue }\n");

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn group_import_makes_both_names_accessible() {
    let dir = fixture_dir("group_import");
    let main = dir.join("main.mtl");
    write(
        &main,
        r#"
import math::{add, mul};
fun main() -> i64 { return add(mul(2, 3), 1); }
"#,
    );
    write(
        &dir.join("math.mtl"),
        "pub fun add(a: i64, b: i64) -> i64 { return a + b; }\npub fun mul(a: i64, b: i64) -> i64 { return a * b; }\n",
    );

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn import_with_alias_loads_module_into_graph() {
    let dir = fixture_dir("alias_load");
    let main = dir.join("main.mtl");
    write(&main, "import helper::answer as compute;\nfun main() { }\n");
    write(&dir.join("helper.mtl"), "pub fun answer() -> i64 { return 42; }\n");

    let graph = module_loader::load_root(&main).unwrap_or_else(|e| panic!("{e}"));

    // The file is loaded even though the local binding uses an alias.
    assert_eq!(graph.modules.len(), 2);
    assert!(graph.modules.iter().any(|m| m.module_path == vec!["helper".to_string()]));
}

#[test]
fn transitive_dependency_loaded_via_facade() {
    let dir = fixture_dir("transitive");
    let main = dir.join("main.mtl");
    write(&main, "import parser::*;\nfun main() -> i64 { return parse(); }\n");
    // parser imports (and thereby loads) lexer; exposes parse() which delegates to tokenize()
    write(
        &dir.join("parser.mtl"),
        "import lexer::*;\npub fun parse() -> i64 { return tokenize(); }\n",
    );
    write(&dir.join("lexer.mtl"), "pub fun tokenize() -> i64 { return 1; }\n");

    let graph = module_loader::load_root(&main).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(graph.modules.len(), 3);

    run_graph(&main).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn import_nonexistent_module_is_a_load_error() {
    // After #186: a missing .mtl file is a hard load error, not a silent skip.
    let dir = fixture_dir("missing_mod");
    let main = dir.join("main.mtl");
    write(
        &main,
        "import nonexistent::Thing;\nfun main() -> i64 { return Thing(); }\n",
    );

    let err = module_loader::load_root(&main).expect_err("missing module should fail at load time");
    let msg = err.to_string();
    assert!(
        msg.contains("nonexistent") || msg.contains("cannot find module"),
        "message was: {msg}",
    );
}

#[test]
fn struct_field_access_across_modules() {
    let dir = fixture_dir("struct_field");
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
