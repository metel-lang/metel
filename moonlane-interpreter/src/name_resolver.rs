use std::collections::{HashMap, HashSet};

use crate::ast::{ImportTree, PathRoot};
use crate::error::MoonlaneError;
use crate::module_loader::{LoadedModule, ModuleGraph};

// ── Public types ──────────────────────────────────────────────────────────────

/// A single resolved import binding within a module's scope.
#[derive(Debug, Clone)]
pub struct ImportBinding {
    /// Canonical module path of the module that provides this name.
    pub source_module: Vec<String>,
    /// The name as declared in the source module.
    pub source_name: String,
    pub kind: BindingKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingKind {
    /// A specific item (type, function, constant, …).
    Item,
    /// A whole module imported as a handle (`import std::math;` → `math::sin()`).
    Module,
}

/// The resolved import scope for a single module.
#[derive(Debug, Clone)]
pub struct ModuleScope {
    pub module_path: Vec<String>,
    /// Explicit bindings: local_name → ImportBinding.
    /// Two explicit bindings with the same local_name are a compile error.
    pub explicit: HashMap<String, ImportBinding>,
    /// Glob-imported module paths (`import path::*`).
    /// Names from these modules are in scope at lower priority than explicit imports.
    /// Ambiguity between two glob-sourced names is deferred to use-site.
    pub globs: Vec<Vec<String>>,
}

/// The output of the name resolution pass: one scope per loaded module.
#[derive(Debug, Clone)]
pub struct ResolvedNames {
    pub scopes: HashMap<Vec<String>, ModuleScope>,
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn resolve(graph: &ModuleGraph) -> Result<ResolvedNames, MoonlaneError> {
    let known_modules: HashSet<Vec<String>> = graph.modules.iter()
        .map(|m| m.module_path.clone())
        .collect();

    let mut scopes = HashMap::new();
    for loaded in &graph.modules {
        let scope = resolve_module(loaded, &known_modules)?;
        scopes.insert(loaded.module_path.clone(), scope);
    }

    Ok(ResolvedNames { scopes })
}

// ── Per-module resolution ─────────────────────────────────────────────────────

fn resolve_module(
    loaded: &LoadedModule,
    known_modules: &HashSet<Vec<String>>,
) -> Result<ModuleScope, MoonlaneError> {
    let mut scope = ModuleScope {
        module_path: loaded.module_path.clone(),
        explicit: HashMap::new(),
        globs: Vec::new(),
    };

    for import in &loaded.program.imports {
        let base = absolute_base(&import.path.root, &loaded.module_path);
        process_tree(&base, &import.path.tree, known_modules, &mut scope)?;
    }

    Ok(scope)
}

/// Compute the absolute path prefix corresponding to a path root,
/// given the importing module's own path.
fn absolute_base(root: &PathRoot, current: &[String]) -> Vec<String> {
    match root {
        PathRoot::Root  => vec![],
        PathRoot::Std   => vec!["std".to_string()],
        PathRoot::Self_ => current.to_vec(),
        PathRoot::Super => {
            if current.is_empty() {
                vec![] // validated as error elsewhere; tolerate gracefully
            } else {
                current[..current.len() - 1].to_vec()
            }
        }
        PathRoot::Name(n) => vec![n.clone()],
    }
}

fn process_tree(
    base: &[String],
    tree: &ImportTree,
    known_modules: &HashSet<Vec<String>>,
    scope: &mut ModuleScope,
) -> Result<(), MoonlaneError> {
    match tree {
        ImportTree::Glob => {
            scope.globs.push(base.to_vec());
        }

        ImportTree::Name { name, alias } => {
            let local = alias.as_deref().unwrap_or(name.as_str()).to_string();

            // Determine whether `base + name` is a known module path —
            // if so this is a module-handle import, not an item import.
            let mut module_candidate = base.to_vec();
            module_candidate.push(name.clone());

            let (source_module, kind) = if known_modules.contains(&module_candidate) {
                (module_candidate, BindingKind::Module)
            } else {
                (base.to_vec(), BindingKind::Item)
            };

            add_explicit(scope, local, ImportBinding {
                source_module,
                source_name: name.clone(),
                kind,
            })?;
        }

        ImportTree::Path { name, tree } => {
            let mut new_base = base.to_vec();
            new_base.push(name.clone());
            process_tree(&new_base, tree, known_modules, scope)?;
        }

        ImportTree::Group(items) => {
            for item in items {
                process_tree(base, item, known_modules, scope)?;
            }
        }
    }

    Ok(())
}

fn add_explicit(
    scope: &mut ModuleScope,
    local_name: String,
    binding: ImportBinding,
) -> Result<(), MoonlaneError> {
    if scope.explicit.contains_key(&local_name) {
        return Err(MoonlaneError::internal(format!(
            "import conflict: `{local_name}` is already bound in this module by a previous import"
        )));
    }
    scope.explicit.insert(local_name, binding);
    Ok(())
}

// ── Query helpers ─────────────────────────────────────────────────────────────

impl ModuleScope {
    /// Look up a local name in this scope.
    /// Returns the explicit binding if one exists, or the glob source modules
    /// that may provide the name (for deferred ambiguity checking).
    pub fn lookup(&self, name: &str) -> ScopeLookup<'_> {
        if let Some(binding) = self.explicit.get(name) {
            return ScopeLookup::Explicit(binding);
        }
        ScopeLookup::MaybeGlob(&self.globs)
    }
}

pub enum ScopeLookup<'a> {
    Explicit(&'a ImportBinding),
    /// The name was not explicitly imported; these glob sources may provide it.
    MaybeGlob(&'a Vec<Vec<String>>),
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ImportDecl, ImportPath, ImportTree, PathRoot, Program, Span};
    use crate::module_loader::{LoadedModule, ModuleGraph};
    use std::path::PathBuf;

    fn span() -> Span {
        Span::new(0, 0, "test")
    }

    fn make_import(root: PathRoot, tree: ImportTree) -> ImportDecl {
        ImportDecl { path: ImportPath { root, tree }, span: span() }
    }

    fn make_program(imports: Vec<ImportDecl>) -> Program {
        Program { imports, exports: vec![], decls: vec![] }
    }

    fn make_graph(modules: Vec<(Vec<String>, Program)>) -> ModuleGraph {
        let root = if modules.is_empty() { PathBuf::new() } else { PathBuf::from("root.mln") };
        let modules = modules.into_iter().map(|(path, program)| LoadedModule {
            module_path: path,
            file_path: PathBuf::from("test.mln"),
            program,
        }).collect();
        ModuleGraph { root, modules }
    }

    #[test]
    fn resolves_explicit_item_import() {
        // import parser::Token;
        let graph = make_graph(vec![
            (vec![], make_program(vec![
                make_import(PathRoot::Name("parser".into()), ImportTree::Name {
                    name: "Token".into(), alias: None,
                }),
            ])),
            (vec!["parser".into()], make_program(vec![])),
        ]);

        let names = resolve(&graph).unwrap();
        let root_scope = &names.scopes[&vec![]];
        let binding = root_scope.explicit.get("Token").expect("Token should be bound");
        assert_eq!(binding.source_module, vec!["parser"]);
        assert_eq!(binding.source_name, "Token");
        assert_eq!(binding.kind, BindingKind::Item);
    }

    #[test]
    fn resolves_alias_import() {
        // import parser::Token as Tok;
        let graph = make_graph(vec![
            (vec![], make_program(vec![
                make_import(PathRoot::Name("parser".into()), ImportTree::Name {
                    name: "Token".into(), alias: Some("Tok".into()),
                }),
            ])),
            (vec!["parser".into()], make_program(vec![])),
        ]);

        let names = resolve(&graph).unwrap();
        let root_scope = &names.scopes[&vec![]];
        assert!(root_scope.explicit.contains_key("Tok"), "alias Tok should be bound");
        assert!(!root_scope.explicit.contains_key("Token"), "original name Token should not be bound");
        let binding = &root_scope.explicit["Tok"];
        assert_eq!(binding.source_name, "Token");
    }

    #[test]
    fn resolves_group_import() {
        // import parser::{Ast, Token};
        let graph = make_graph(vec![
            (vec![], make_program(vec![
                make_import(PathRoot::Name("parser".into()), ImportTree::Group(vec![
                    ImportTree::Name { name: "Ast".into(), alias: None },
                    ImportTree::Name { name: "Token".into(), alias: None },
                ])),
            ])),
            (vec!["parser".into()], make_program(vec![])),
        ]);

        let names = resolve(&graph).unwrap();
        let root_scope = &names.scopes[&vec![]];
        assert!(root_scope.explicit.contains_key("Ast"));
        assert!(root_scope.explicit.contains_key("Token"));
    }

    #[test]
    fn resolves_glob_import() {
        // import parser::*;
        let graph = make_graph(vec![
            (vec![], make_program(vec![
                make_import(PathRoot::Name("parser".into()), ImportTree::Glob),
            ])),
            (vec!["parser".into()], make_program(vec![])),
        ]);

        let names = resolve(&graph).unwrap();
        let root_scope = &names.scopes[&vec![]];
        assert!(root_scope.explicit.is_empty(), "glob should not add explicit bindings");
        assert_eq!(root_scope.globs, vec![vec!["parser".to_string()]]);
    }

    #[test]
    fn resolves_module_handle_import() {
        // import parser; — parser is a known module, so this is a handle import
        let graph = make_graph(vec![
            (vec![], make_program(vec![
                make_import(PathRoot::Root, ImportTree::Name {
                    name: "parser".into(), alias: None,
                }),
            ])),
            (vec!["parser".into()], make_program(vec![])),
        ]);

        let names = resolve(&graph).unwrap();
        let root_scope = &names.scopes[&vec![]];
        let binding = root_scope.explicit.get("parser").expect("parser handle should be bound");
        assert_eq!(binding.kind, BindingKind::Module);
        assert_eq!(binding.source_module, vec!["parser"]);
    }

    #[test]
    fn rejects_duplicate_explicit_import() {
        // import parser::Token;
        // import lexer::Token;  ← conflict
        let graph = make_graph(vec![
            (vec![], make_program(vec![
                make_import(PathRoot::Name("parser".into()), ImportTree::Name {
                    name: "Token".into(), alias: None,
                }),
                make_import(PathRoot::Name("lexer".into()), ImportTree::Name {
                    name: "Token".into(), alias: None,
                }),
            ])),
            (vec!["parser".into()], make_program(vec![])),
            (vec!["lexer".into()],  make_program(vec![])),
        ]);

        let err = resolve(&graph).expect_err("duplicate import should fail");
        assert!(err.to_string().contains("Token"), "error should mention Token");
    }

    #[test]
    fn resolves_root_absolute_path() {
        // import root::parser::Ast;
        let graph = make_graph(vec![
            (vec![], make_program(vec![
                make_import(PathRoot::Root, ImportTree::Path {
                    name: "parser".into(),
                    tree: Box::new(ImportTree::Name { name: "Ast".into(), alias: None }),
                }),
            ])),
            (vec!["parser".into()], make_program(vec![])),
        ]);

        let names = resolve(&graph).unwrap();
        let root_scope = &names.scopes[&vec![]];
        let binding = root_scope.explicit.get("Ast").expect("Ast should be bound");
        assert_eq!(binding.source_module, vec!["parser"]);
    }

    #[test]
    fn resolves_self_relative_path() {
        // In module ["parser"], import self::child::Thing;
        let graph = make_graph(vec![
            (vec!["parser".into()], make_program(vec![
                make_import(PathRoot::Self_, ImportTree::Path {
                    name: "child".into(),
                    tree: Box::new(ImportTree::Name { name: "Thing".into(), alias: None }),
                }),
            ])),
            (vec!["parser".into(), "child".into()], make_program(vec![])),
        ]);

        let names = resolve(&graph).unwrap();
        let parser_scope = &names.scopes[&vec!["parser".to_string()]];
        let binding = parser_scope.explicit.get("Thing").expect("Thing should be bound");
        assert_eq!(binding.source_module, vec!["parser", "child"]);
    }

    #[test]
    fn resolves_super_relative_path() {
        // In module ["parser", "child"], import super::Token;
        let graph = make_graph(vec![
            (vec!["parser".into(), "child".into()], make_program(vec![
                make_import(PathRoot::Super, ImportTree::Name {
                    name: "Token".into(), alias: None,
                }),
            ])),
            (vec!["parser".into()], make_program(vec![])),
        ]);

        let names = resolve(&graph).unwrap();
        let child_scope = &names.scopes[&vec!["parser".to_string(), "child".to_string()]];
        let binding = child_scope.explicit.get("Token").expect("Token should be bound");
        assert_eq!(binding.source_module, vec!["parser"]);
    }
}
