use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::{PathRoot, Program, UseTree};
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
    let mut modules = Vec::new();
    let mut imports = Vec::new();
    let mut decls = Vec::new();

    for loaded in graph.modules {
        modules.extend(loaded.program.modules);
        imports.extend(loaded.program.imports);
        decls.extend(loaded.program.decls);
    }

    Ok(Program { modules, imports, decls })
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
        for module in &program.modules {
            let child = resolve_child_module(&file_path, &module.name)?;
            let mut child_path = module_path.clone();
            child_path.push(module.name.clone());
            self.load_module(child, child_path)?;
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

fn resolve_child_module(parent_file: &Path, name: &str) -> Result<PathBuf, MoonlaneError> {
    let parent_dir = parent_file.parent().unwrap_or_else(|| Path::new("."));
    let file_candidate = parent_dir.join(format!("{name}.mln"));
    let mod_candidate = parent_dir.join(name).join("mod.mln");
    let file_exists = file_candidate.exists();
    let mod_exists = mod_candidate.exists();

    match (file_exists, mod_exists) {
        (true, true) => Err(module_error(
            format!(
                "ambiguous module `{name}`: both `{}` and `{}` exist; remove one to resolve the ambiguity",
                file_candidate.display(),
                mod_candidate.display(),
            ),
            parent_file,
        )),
        (true, false) => canonicalize_existing(&file_candidate),
        (false, true) => canonicalize_existing(&mod_candidate),
        (false, false) => Err(module_error(
            format!(
                "module `{name}` not found; expected `{}` or `{}`",
                file_candidate.display(),
                mod_candidate.display(),
            ),
            parent_file,
        )),
    }
}

fn validate_super_root(program: &Program, module_path: &[String], file_path: &Path) -> Result<(), MoonlaneError> {
    if !module_path.is_empty() {
        return Ok(());
    }

    for import in &program.imports {
        if import.path.root == PathRoot::Super || use_tree_contains_super(&import.path.tree) {
            return Err(module_error("`super::` is invalid from the root module", file_path));
        }
    }

    Ok(())
}

fn use_tree_contains_super(tree: &UseTree) -> bool {
    match tree {
        UseTree::Name { .. } | UseTree::Glob => false,
        UseTree::Group(trees) => trees.iter().any(use_tree_contains_super),
        UseTree::Path { tree, .. } => use_tree_contains_super(tree),
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
