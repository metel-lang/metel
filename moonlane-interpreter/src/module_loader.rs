use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::{ImportTree, PathRoot, Program};
use crate::error::{MoonlaneError, ParseErrorCode};
use crate::parser;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LoadedModule {
    pub module_path: Vec<String>,
    pub file_path: PathBuf,
    pub program: Program,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ModuleGraph {
    pub root: PathBuf,
    pub modules: Vec<LoadedModule>,
}

pub fn load_root(path: impl AsRef<Path>) -> Result<ModuleGraph, MoonlaneError> {
    let root = canonicalize_existing(path.as_ref())?;
    let mut loader = Loader::default();
    loader.load_module(root.clone(), Vec::new())?;
    Ok(ModuleGraph { root, modules: loader.modules })
}

pub fn load_program(path: impl AsRef<Path>) -> Result<Program, MoonlaneError> {
    let graph = load_root(path)?;
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let mut decls = Vec::new();

    // Flat merge: all module decls are combined into one Program so the typechecker
    // sees every declaration globally. Per-module scope isolation is deferred (ADR-0019).
    // Remove this merge when name_resolver is wired into the typechecker pipeline.
    for loaded in graph.modules {
        imports.extend(loaded.program.imports);
        exports.extend(loaded.program.exports);
        decls.extend(loaded.program.decls);
    }

    Ok(Program { imports, exports, decls })
}

#[derive(Default)]
struct Loader {
    modules: Vec<LoadedModule>,
    visited: HashSet<PathBuf>,
    stack: Vec<PathBuf>,
}

impl Loader {
    fn load_module(&mut self, file_path: PathBuf, module_path: Vec<String>) -> Result<(), MoonlaneError> {
        if let Some(cycle_start) = self.stack.iter().position(|p| p == &file_path) {
            let mut chain: Vec<String> = self.stack[cycle_start..]
                .iter()
                .map(|p| p.display().to_string())
                .collect();
            chain.push(file_path.display().to_string());
            return Err(module_error(
                format!("circular module dependency: {}", chain.join(" -> ")),
                &file_path,
            ));
        }

        if self.visited.contains(&file_path) {
            return Ok(());
        }

        let source = fs::read_to_string(&file_path).map_err(|e| {
            module_error(
                format!("failed to read module '{}': {e}", file_path.display()),
                &file_path,
            )
        })?;
        let filename = file_path.display().to_string();
        let program = parser::parse(&source, &filename)?;

        validate_super_root(&program, &module_path, &file_path)?;

        self.stack.push(file_path.clone());
        for import in &program.imports {
            if let Some((mod_segs, child_file)) = resolve_import_module(&file_path, &import.path.root, &import.path.tree) {
                let child = canonicalize_existing(&child_file)?;
                let mut child_path = module_path.clone();
                child_path.extend(mod_segs);
                self.load_module(child, child_path)?;
            }
        }
        self.stack.pop();

        self.visited.insert(file_path.clone());
        self.modules.push(LoadedModule { module_path, file_path, program });
        Ok(())
    }
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf, MoonlaneError> {
    path.canonicalize().map_err(|e| {
        module_error(
            format!("failed to resolve module '{}': {e}", path.display()),
            path,
        )
    })
}

/// Resolve an import declaration to a module file.
///
/// Returns `(module_path_segments, file_path)` if a `.mln` file exists for the
/// import, or `None` for `root::`, `std::`, `super::`, and imports that don't map
/// to a sibling file. The module path segments are extended onto the caller's
/// `module_path` to form the child's canonical module path.
///
/// Path mapping: `::` separators map directly to `/` directory separators.
/// `import parser::ast::Ast` tries `parser/ast.mln` first, then `parser.mln` —
/// the longest matching prefix wins.
fn resolve_import_module(
    parent_file: &Path,
    root: &PathRoot,
    tree: &ImportTree,
) -> Option<(Vec<String>, PathBuf)> {
    let parent_dir = parent_file.parent().unwrap_or_else(|| Path::new("."));

    let segs: Vec<String> = match root {
        PathRoot::Root | PathRoot::Std => return None,
        PathRoot::Super => return None,
        PathRoot::Self_ => import_tree_segments(tree),
        PathRoot::Name(name) => {
            let mut s = vec![name.clone()];
            s.extend(import_tree_segments(tree));
            s
        }
    };

    find_module_file(parent_dir, &segs)
}

/// Collect all identifier segments from an import tree in path order.
/// Stops at the terminal item(s) — returns their names as the last segment(s).
/// For `ast::Ast` → ["ast", "Ast"]; for `ast::{A, B}` → ["ast"]; for `*` → [].
fn import_tree_segments(tree: &ImportTree) -> Vec<String> {
    match tree {
        ImportTree::Name { name, .. } => vec![name.clone()],
        ImportTree::Path { name, tree } => {
            let mut segs = vec![name.clone()];
            segs.extend(import_tree_segments(tree));
            segs
        }
        ImportTree::Group(_) | ImportTree::Glob => vec![],
    }
}

/// Try path prefixes from longest to shortest, returning the first `.mln` found.
fn find_module_file(base_dir: &Path, segs: &[String]) -> Option<(Vec<String>, PathBuf)> {
    for len in (1..=segs.len()).rev() {
        let prefix = &segs[..len];
        let mut candidate = base_dir.to_path_buf();
        for seg in prefix {
            candidate = candidate.join(seg);
        }
        let file = candidate.with_extension("mln");
        if file.exists() {
            return Some((prefix.to_vec(), file));
        }
    }
    None
}

fn validate_super_root(program: &Program, module_path: &[String], file_path: &Path) -> Result<(), MoonlaneError> {
    if !module_path.is_empty() {
        return Ok(());
    }

    for import in &program.imports {
        if import.path.root == PathRoot::Super || import_tree_contains_super(&import.path.tree) {
            return Err(module_error("`super::` is invalid from the root module", file_path));
        }
    }

    Ok(())
}

fn import_tree_contains_super(tree: &ImportTree) -> bool {
    match tree {
        ImportTree::Name { .. } | ImportTree::Glob => false,
        ImportTree::Group(trees) => trees.iter().any(import_tree_contains_super),
        ImportTree::Path { tree, .. } => import_tree_contains_super(tree),
    }
}

fn module_error(message: impl Into<String>, path: &Path) -> MoonlaneError {
    MoonlaneError::ParseError {
        code: ParseErrorCode::P0001,
        message: message.into(),
        start: 0,
        end: 0,
        filename: path.display().to_string(),
        line: 1,
        col: 1,
        source_line: None,
    }
}
