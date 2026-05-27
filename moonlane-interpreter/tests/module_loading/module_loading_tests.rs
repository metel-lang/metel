use std::fs;
use std::path::{Path, PathBuf};

use moonlane::evaluator;
use moonlane::module_loader;
use moonlane::typechecker;

fn fixture_dir(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "moonlane_module_loading_{}_{}_{}",
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

#[test]
fn single_file_program_loads_without_modules() {
    let dir = fixture_dir("single");
    let main = dir.join("main.mln");
    write(&main, "fun main() { }\n");

    let program = module_loader::load_program(&main).unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(program.modules.len(), 0);
    assert_eq!(program.decls.len(), 1);
}

#[test]
fn multi_file_program_loads_declared_modules() {
    let dir = fixture_dir("multi");
    let main = dir.join("main.mln");
    write(&main, "mod parser;\nfun main() { }\n");
    write(&dir.join("parser.mln"), "struct Token { value: Int }\n");

    let graph = module_loader::load_root(&main).unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(graph.modules.len(), 2);
    assert!(graph.modules.iter().any(|m| m.module_path == vec!["parser".to_string()]));
}

#[test]
fn multi_file_program_runs_after_module_loading() {
    let dir = fixture_dir("run_multi");
    let main = dir.join("main.mln");
    write(&main, "mod helper;\nfun main() -> Int { return answer(); }\n");
    write(&dir.join("helper.mln"), "fun answer() -> Int { return 42; }\n");

    let program = module_loader::load_program(&main).unwrap_or_else(|e| panic!("{e}"));
    let typed = typechecker::check(program).unwrap_or_else(|e| panic!("{e}"));

    evaluator::evaluate(typed).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn resolves_name_mod_file_layout() {
    let dir = fixture_dir("mod_file");
    let main = dir.join("main.mln");
    write(&main, "mod parser;\nfun main() { }\n");
    write(&dir.join("parser").join("mod.mln"), "struct Token { value: Int }\n");

    let graph = module_loader::load_root(&main).unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(graph.modules.len(), 2);
}

#[test]
fn rejects_ambiguous_module_layout() {
    let dir = fixture_dir("ambiguous");
    let main = dir.join("main.mln");
    write(&main, "mod parser;\nfun main() { }\n");
    write(&dir.join("parser.mln"), "struct A { value: Int }\n");
    write(&dir.join("parser").join("mod.mln"), "struct B { value: Int }\n");

    let err = module_loader::load_root(&main).expect_err("ambiguous module should fail");
    let msg = err.to_string();

    assert!(msg.contains("ambiguous module `parser`"), "message was: {msg}");
    assert!(msg.contains("parser.mln"), "message was: {msg}");
    assert!(msg.contains("parser/mod.mln"), "message was: {msg}");
}

#[test]
fn rejects_super_from_root_import() {
    let dir = fixture_dir("root_super");
    let main = dir.join("main.mln");
    write(&main, "use super::parser;\nfun main() { }\n");

    let err = module_loader::load_root(&main).expect_err("super from root should fail");
    let msg = err.to_string();

    assert!(msg.contains("super::"), "message was: {msg}");
    assert!(msg.contains("root module"), "message was: {msg}");
}

#[test]
fn accepts_root_self_super_std_and_child_roots_in_non_root_modules() {
    let dir = fixture_dir("roots");
    let main = dir.join("main.mln");
    write(&main, "mod parser;\nfun main() { }\n");
    write(
        &dir.join("parser.mln"),
        r#"
mod child;
use root::parser;
use self::child;
use super::parser;
use std::core;
use child::Thing;

struct Token { value: Int }
"#,
    );
    write(&dir.join("child.mln"), "struct Thing { value: Int }\n");

    let graph = module_loader::load_root(&main).unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(graph.modules.len(), 3);
}

#[cfg(unix)]
#[test]
fn rejects_circular_module_graph() {
    use std::os::unix::fs::symlink;

    let dir = fixture_dir("cycle");
    let main = dir.join("main.mln");
    write(&main, "mod a;\nfun main() { }\n");
    write(&dir.join("a").join("mod.mln"), "mod b;\n");
    symlink(dir.join("a"), dir.join("a").join("b"))
        .unwrap_or_else(|e| panic!("failed to create symlink cycle: {e}"));

    let err = module_loader::load_root(&main).expect_err("cycle should fail");
    let msg = err.to_string();

    assert!(msg.contains("circular module dependency"), "message was: {msg}");
}
