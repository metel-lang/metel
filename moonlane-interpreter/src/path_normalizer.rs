/// Path normalization pass (#185).
///
/// Rewrites `Expr::Path` nodes with module qualifiers to `Expr::ResolvedPath`.
/// Single-segment paths and type member accesses (e.g. `Color::Red`) are left as-is.
///
/// A path `[s1, s2, ...]` is considered module-qualified when `s1` is:
/// - the reserved keyword `"root"`, `"self"`, or `"super"`, or
/// - the name of a loaded module in the `ModuleGraph`.
///
/// Everything else (e.g. `Color::Red` where Color is a struct/enum) passes through
/// unchanged so the typechecker's existing type-member handling works unmodified.

use std::collections::HashSet;

use crate::ast::{
    Block, Decl, Expr, ForInit, FunDecl, ImplBlock, MatchArm, MutDecl, Span, Stmt,
};
use crate::error::MoonlaneError;
use crate::module_loader::{LoadedModule, ModuleGraph};
use crate::name_resolver::{ModuleScope, ResolvedNames};

// ── Public API ────────────────────────────────────────────────────────────────

/// Opaque wrapper around `ModuleGraph` that proves the normalization pass has run.
/// `check_graph` requires this type; calling it with a raw `ModuleGraph` is a
/// compile-time error.
pub struct NormalizedModuleGraph(pub(crate) ModuleGraph);

impl NormalizedModuleGraph {
    pub fn modules(&self) -> &[LoadedModule] { &self.0.modules }
}

/// Run the path normalization pass on `graph`, rewriting qualified `Expr::Path`
/// nodes to `Expr::ResolvedPath` using the scope information in `names`.
///
/// Returns `NormalizedModuleGraph` — a newtype that downstream passes must accept
/// to enforce that normalization ran before typechecking.
pub fn normalize(
    mut graph: ModuleGraph,
    names: &ResolvedNames,
) -> Result<NormalizedModuleGraph, MoonlaneError> {
    let module_names: HashSet<String> = graph.modules.iter()
        .flat_map(|m| m.module_path.first().cloned())
        .collect();

    for loaded in &mut graph.modules {
        let scope = names.scopes.get(&loaded.module_path);
        normalize_program_decls(&mut loaded.program.decls, scope, &module_names)?;
    }
    Ok(NormalizedModuleGraph(graph))
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn normalize_program_decls(
    decls: &mut Vec<Decl>,
    scope: Option<&ModuleScope>,
    module_names: &HashSet<String>,
) -> Result<(), MoonlaneError> {
    for decl in decls {
        normalize_decl(decl, scope, module_names)?;
    }
    Ok(())
}

fn normalize_decl(
    decl: &mut Decl,
    scope: Option<&ModuleScope>,
    module_names: &HashSet<String>,
) -> Result<(), MoonlaneError> {
    match decl {
        Decl::Let(ld)  => normalize_expr(&mut ld.value, scope, module_names),
        Decl::Mut(md)  => normalize_expr(&mut md.value, scope, module_names),
        Decl::Fun(fd)  => normalize_fun(fd, scope, module_names),
        Decl::Impl(ib) => normalize_impl(ib, scope, module_names),
        Decl::Stmt(s)  => normalize_stmt(s, scope, module_names),
        Decl::Struct(_) | Decl::Enum(_) | Decl::Aspect(_) => Ok(()),
    }
}

fn normalize_fun(
    fun: &mut FunDecl,
    scope: Option<&ModuleScope>,
    module_names: &HashSet<String>,
) -> Result<(), MoonlaneError> {
    normalize_block(&mut fun.body, scope, module_names)
}

fn normalize_impl(
    ib: &mut ImplBlock,
    scope: Option<&ModuleScope>,
    module_names: &HashSet<String>,
) -> Result<(), MoonlaneError> {
    for method in &mut ib.methods {
        normalize_fun(method, scope, module_names)?;
    }
    Ok(())
}

fn normalize_block(
    block: &mut Block,
    scope: Option<&ModuleScope>,
    module_names: &HashSet<String>,
) -> Result<(), MoonlaneError> {
    for decl in &mut block.stmts {
        normalize_decl(decl, scope, module_names)?;
    }
    if let Some(tail) = &mut block.tail {
        normalize_expr(tail, scope, module_names)?;
    }
    Ok(())
}

fn normalize_stmt(
    stmt: &mut Stmt,
    scope: Option<&ModuleScope>,
    module_names: &HashSet<String>,
) -> Result<(), MoonlaneError> {
    match stmt {
        Stmt::Expr(e) => normalize_expr(e, scope, module_names),
        Stmt::Return(r) => {
            if let Some(v) = &mut r.value { normalize_expr(v, scope, module_names) } else { Ok(()) }
        }
        Stmt::Break(b) => {
            if let Some(v) = &mut b.value { normalize_expr(v, scope, module_names) } else { Ok(()) }
        }
        Stmt::Continue(_) => Ok(()),
        Stmt::While(w) => {
            normalize_expr(&mut w.condition, scope, module_names)?;
            normalize_block(&mut w.body, scope, module_names)
        }
        Stmt::For(f) => {
            if let Some(init) = &mut f.init {
                match init {
                    ForInit::Expr(e) => normalize_expr(e, scope, module_names)?,
                    ForInit::Mut(md) => normalize_mut_decl(md, scope, module_names)?,
                }
            }
            if let Some(cond) = &mut f.condition { normalize_expr(cond, scope, module_names)?; }
            if let Some(step) = &mut f.step      { normalize_expr(step, scope, module_names)?; }
            normalize_block(&mut f.body, scope, module_names)
        }
        Stmt::ForIn(fi) => {
            normalize_expr(&mut fi.iterable, scope, module_names)?;
            normalize_block(&mut fi.body, scope, module_names)
        }
    }
}

fn normalize_mut_decl(
    md: &mut MutDecl,
    scope: Option<&ModuleScope>,
    module_names: &HashSet<String>,
) -> Result<(), MoonlaneError> {
    normalize_expr(&mut md.value, scope, module_names)
}

fn normalize_expr(
    expr: &mut Expr,
    scope: Option<&ModuleScope>,
    module_names: &HashSet<String>,
) -> Result<(), MoonlaneError> {
    match expr {
        Expr::Literal(_, _) | Expr::Ident(_, _) | Expr::ResolvedPath { .. } => Ok(()),

        Expr::Path(segments, span) => {
            if let Some(resolved) = try_resolve_path(segments, scope, module_names) {
                let original = std::mem::take(segments);
                *expr = Expr::ResolvedPath { resolved, original, span: span.clone() };
            }
            Ok(())
        }

        Expr::Tuple(elems, _) => {
            for e in elems { normalize_expr(e, scope, module_names)?; }
            Ok(())
        }
        Expr::Array(elems, _) => {
            for e in elems { normalize_expr(e, scope, module_names)?; }
            Ok(())
        }
        Expr::BinOp(lhs, _, rhs, _) => {
            normalize_expr(lhs, scope, module_names)?;
            normalize_expr(rhs, scope, module_names)
        }
        Expr::UnaryOp(_, operand, _) => normalize_expr(operand, scope, module_names),
        Expr::Cast { expr: inner, .. } | Expr::Ascribe { expr: inner, .. } => {
            normalize_expr(inner, scope, module_names)
        }
        Expr::Assign { value, .. } => normalize_expr(value, scope, module_names),
        Expr::Call { callee, args, .. } => {
            normalize_expr(callee, scope, module_names)?;
            for a in args { normalize_expr(a, scope, module_names)?; }
            Ok(())
        }
        Expr::MethodCall { receiver, args, .. } => {
            normalize_expr(receiver, scope, module_names)?;
            for a in args { normalize_expr(a, scope, module_names)?; }
            Ok(())
        }
        Expr::FieldAccess { object, .. }
        | Expr::TupleAccess { object, .. } => normalize_expr(object, scope, module_names),
        Expr::Index { object, index, .. } => {
            normalize_expr(object, scope, module_names)?;
            normalize_expr(index, scope, module_names)
        }
        Expr::If { condition, then_branch, else_branch, .. } => {
            normalize_expr(condition, scope, module_names)?;
            normalize_block(then_branch, scope, module_names)?;
            if let Some(eb) = else_branch { normalize_block(eb, scope, module_names)?; }
            Ok(())
        }
        Expr::Loop { body, .. } => normalize_block(body, scope, module_names),
        Expr::Closure { body, .. } => normalize_block(body, scope, module_names),
        Expr::Match(m) => {
            normalize_expr(&mut m.scrutinee, scope, module_names)?;
            for arm in &mut m.arms { normalize_arm(arm, scope, module_names)?; }
            Ok(())
        }
        Expr::StructLiteral { fields, .. } => {
            for (_, v) in fields { normalize_expr(v, scope, module_names)?; }
            Ok(())
        }
        Expr::PropagateError { expr: inner, .. } => normalize_expr(inner, scope, module_names),
    }
}

fn normalize_arm(
    arm: &mut MatchArm,
    scope: Option<&ModuleScope>,
    module_names: &HashSet<String>,
) -> Result<(), MoonlaneError> {
    if let Some(guard) = &mut arm.guard { normalize_expr(guard, scope, module_names)?; }
    normalize_block(&mut arm.body, scope, module_names)
}

// ── Path resolution logic ─────────────────────────────────────────────────────

/// Try to resolve a multi-segment path to a bare local name.
///
/// Returns `Some(resolved_name)` if the path is module-qualified and can be
/// rewritten. Returns `None` to leave the path unchanged (type member access,
/// single-segment, or unresolvable).
fn try_resolve_path(
    segments: &[String],
    scope: Option<&ModuleScope>,
    module_names: &HashSet<String>,
) -> Option<String> {
    if segments.len() < 2 {
        return None; // single-segment paths are already Ident
    }

    let first = &segments[0];
    let declared_name = segments.last().unwrap();

    // Keywords: root/self/super — the declared name is the last segment
    if first == "root" || first == "self" || first == "super" {
        let name = declared_name.clone();
        // Check the scope for an alias, otherwise use the declared name
        if let Some(s) = scope {
            if let Some((local, _)) = s.explicit.iter().find(|(_, b)| &b.source_name == declared_name) {
                return Some(local.clone());
            }
        }
        return Some(name);
    }

    // Non-keyword first segment: only rewrite if it matches a loaded module name
    if !module_names.contains(first.as_str()) {
        return None; // e.g. Color::Red — Color is a type, not a module
    }

    // first is a known module name — find the local alias for this import
    if let Some(s) = scope {
        // 1. Explicit import with matching source
        for (local_name, binding) in &s.explicit {
            if binding.source_module.first().map(|s| s.as_str()) == Some(first.as_str())
                && &binding.source_name == declared_name
            {
                return Some(local_name.clone());
            }
        }
        // 2. Glob import from this module — local name == source name
        let source_module: Vec<String> = segments[..segments.len() - 1].to_vec();
        if s.globs.iter().any(|g| g == &source_module || g.first() == Some(first)) {
            return Some(declared_name.clone());
        }
    }

    // Module is known but no import binding found for this name — treat as bare name
    // (the typechecker will error if it's actually undefined)
    Some(declared_name.clone())
}

// ── Unused span helper ────────────────────────────────────────────────────────
#[allow(dead_code)]
fn dummy_span() -> Span { Span::new(0, 0, "") }
