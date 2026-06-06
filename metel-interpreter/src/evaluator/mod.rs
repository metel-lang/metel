// PoC evaluator — this implementation will almost certainly be rewritten.
// Implement the simplest correct thing; do not over-engineer.

pub(crate) mod builtins;
mod call;
mod display;
mod lvalue;
mod pattern;
mod type_of;

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::{BinOp, Literal, Param, Span, UnaryOp};
use crate::error::{FrameInfo, RuntimeErrorCode, MetelError};
use crate::typeinference::TypeCtx;

thread_local! {
    static CALL_STACK: RefCell<Vec<FrameInfo>> = const { RefCell::new(Vec::new()) };
}

pub(super) fn push_frame(fn_name: String, call_site: Span) {
    CALL_STACK.with(|s| s.borrow_mut().push(FrameInfo { fn_name, call_site }));
}

pub(super) fn pop_frame() {
    CALL_STACK.with(|s| { s.borrow_mut().pop(); });
}

fn snapshot_stack() -> Vec<FrameInfo> {
    CALL_STACK.with(|s| s.borrow().clone())
}

pub(super) fn attach_stack(err: MetelError) -> MetelError {
    err.with_stack(snapshot_stack())
}
use crate::ast::Block;
use crate::typed_ast::{FunBody, TypedBlock, TypedDecl, TypedExpr, TypedForInit, TypedModuleGraph, TypedProgram, TypedStmt};

// ── Runtime values ────────────────────────────────────────────────────────────

/// One step in a `MutFieldPointer` path.
#[derive(Debug, Clone)]
pub enum PathSegment {
    Field(String),
    TupleIndex(usize),
    ArrayIndex(usize),
}

#[derive(Debug, Clone)]
pub enum Value {
    // ── Primitive types ───────────────────────────────────────────────────────
    I64(i64),
    /// Sized signed integers.
    I8(i8), I16(i16), I32(i32),
    /// Sized unsigned integers.
    U8(u8), U16(u16), U32(u32), U64(u64),
    F64(f64),
    /// 32-bit float.
    F32(f32),
    Char(char),
    Boolean(bool),
    Str(String),
    Unit,
    // ── Compound types ────────────────────────────────────────────────────────
    Tuple(Vec<Value>),
    Array(Rc<RefCell<Vec<Value>>>),
    Struct { name: String, fields: HashMap<String, Value> },
    // Perhaps<T> and Result<T,E> use Value::Enum like all other enums. See ADR-0028.
    Enum { name: String, variant: String, fields: HashMap<String, Value> },
    Closure(Rc<ClosureValue>),
    Builtin(String, fn(Vec<Value>, &Span) -> Result<Value, MetelError>),
    /// Read-only pointer to a named binding cell.
    Pointer(Rc<RefCell<Value>>),
    /// Writable pointer to a named binding cell.
    MutPointer(Rc<RefCell<Value>>),
    /// Fat mutable pointer for sub-element lvalue paths (RFC-0045).
    /// `root` is the binding cell; `path` navigates to the leaf.
    MutFieldPointer { root: Rc<RefCell<Value>>, path: Vec<PathSegment> },
}

/// The body of a closure — either a fully type-checked block (monomorphic) or the
/// original untyped block (generic / let-polymorphic). The evaluator dispatches on
/// this to choose between `eval_block` and `eval_untyped_block`.
#[derive(Debug, Clone)]
pub enum ClosureBody {
    Typed(TypedBlock),
    Untyped(Block),
}

#[derive(Debug, Clone)]
pub struct ClosureValue {
    pub name:     Option<String>,
    pub params:   Vec<Param>,
    pub body:     ClosureBody,
    pub captured: Environment,
    /// Present only when `body` is `ClosureBody::Untyped` (generic function). Provides
    /// the type context for construction-at-call-time so the untyped path is not needed.
    pub type_ctx: Option<std::rc::Rc<TypeCtx>>,
    /// The concrete function type of this closure, if known. Used by `value_to_type` to
    /// recover the closure's parameter/return types when it is passed as a generic argument.
    pub fun_type: Option<crate::types::Type>,
}

/// Deep-clone a value so that arrays get independent copies.
/// Tuples, structs, and enums are recursed into so that nested arrays are also copied.
/// All other value kinds contain no shared mutable state and can be cloned shallowly.
fn deep_clone_value(v: Value) -> Value {
    match v {
        Value::Array(rc) => {
            let cloned: Vec<Value> = rc.borrow().iter().cloned().map(deep_clone_value).collect();
            Value::Array(Rc::new(RefCell::new(cloned)))
        }
        Value::Tuple(items) => Value::Tuple(items.into_iter().map(deep_clone_value).collect()),
        Value::Struct { name, fields } => Value::Struct {
            name,
            fields: fields.into_iter().map(|(k, v)| (k, deep_clone_value(v))).collect(),
        },
        Value::Enum { name, variant, fields } => Value::Enum {
            name,
            variant,
            fields: fields.into_iter().map(|(k, v)| (k, deep_clone_value(v))).collect(),
        },
        other => other,
    }
}

/// Walk a `PathSegment` path into `root`, returning a clone of the leaf value.
fn read_path(root: &Value, path: &[PathSegment], span: &Span) -> Result<Value, MetelError> {
    let mut cur = root.clone();
    for seg in path {
        cur = match (seg, cur) {
            (PathSegment::Field(f), Value::Struct { fields, .. } | Value::Enum { fields, .. }) =>
                fields.get(f.as_str()).cloned().ok_or_else(|| MetelError::panic(
                    RuntimeErrorCode::R0008, format!("fat pointer: no field `{f}`"), span))?,
            (PathSegment::TupleIndex(i), Value::Tuple(elems)) =>
                elems.get(*i).cloned().ok_or_else(|| MetelError::panic(
                    RuntimeErrorCode::R0008, format!("fat pointer: tuple index {i} out of bounds"), span))?,
            (PathSegment::ArrayIndex(i), Value::Array(rc)) =>
                rc.borrow().get(*i).cloned().ok_or_else(|| MetelError::panic(
                    RuntimeErrorCode::R0004, format!("fat pointer: array index {i} out of bounds"), span))?,
            _ => return Err(MetelError::internal("fat pointer path: segment type mismatch")),
        };
    }
    Ok(cur)
}

/// Walk a `PathSegment` path into `root` and write `new_val` at the leaf.
fn write_path(root: &mut Value, path: &[PathSegment], new_val: Value, span: &Span) -> Result<(), MetelError> {
    if path.is_empty() {
        *root = new_val;
        return Ok(());
    }
    match (&path[0], root) {
        (PathSegment::Field(f), Value::Struct { fields, .. } | Value::Enum { fields, .. }) => {
            let child = fields.get_mut(f.as_str()).ok_or_else(|| MetelError::panic(
                RuntimeErrorCode::R0008, format!("fat pointer: no field `{f}`"), span))?;
            write_path(child, &path[1..], new_val, span)
        }
        (PathSegment::TupleIndex(i), Value::Tuple(elems)) => {
            let child = elems.get_mut(*i).ok_or_else(|| MetelError::panic(
                RuntimeErrorCode::R0008, format!("fat pointer: tuple index {i} out of bounds"), span))?;
            write_path(child, &path[1..], new_val, span)
        }
        (PathSegment::ArrayIndex(i), Value::Array(rc)) => {
            let mut borrow = rc.borrow_mut();
            let child = borrow.get_mut(*i).ok_or_else(|| MetelError::panic(
                RuntimeErrorCode::R0004, format!("fat pointer: array index {i} out of bounds"), span))?;
            write_path(child, &path[1..], new_val, span)
        }
        _ => Err(MetelError::internal("fat pointer path: segment type mismatch during write")),
    }
}

/// Like `Pointer`/`MutPointer` deref but also handles `MutFieldPointer` with a proper span.
fn deref_value(value: &Value, span: &Span) -> Result<Option<Value>, MetelError> {
    match value {
        Value::Pointer(rc) | Value::MutPointer(rc) => Ok(Some(rc.borrow().clone())),
        Value::MutFieldPointer { root, path } => Ok(Some(read_path(&root.borrow(), path, span)?)),
        _ => Ok(None),
    }
}

fn receiver_cell_from_value(value: &Value) -> Option<Rc<RefCell<Value>>> {
    match value {
        Value::Pointer(rc) | Value::MutPointer(rc) => Some(Rc::clone(rc)),
        _ => None,
    }
}

// For a FieldAccess receiver like `a.b.c`, returns:
//   (struct_cell, ["a","b","c"], leaf_cell)
// where struct_cell is the Rc for the root variable (pointer-followed if needed),
// the path encodes every field segment, and leaf_cell is a fresh Rc wrapping a clone
// of the leaf value.  After a &mut self call the caller writes leaf_cell's value back.
fn lvalue_field_cell(
    receiver: &crate::typed_ast::TypedExpr,
    env: &Environment,
) -> Option<(Rc<RefCell<Value>>, Vec<String>, Rc<RefCell<Value>>)> {
    use crate::typed_ast::TypedExpr;
    fn walk_path(expr: &TypedExpr, path: &mut Vec<String>) -> Option<String> {
        match expr {
            TypedExpr::Ident(name, _, _) => Some(name.clone()),
            TypedExpr::FieldAccess { object, field, .. } => {
                let root = walk_path(object, path)?;
                path.push(field.clone());
                Some(root)
            }
            _ => None,
        }
    }
    let mut path = Vec::new();
    let root = walk_path(receiver, &mut path)?;

    let root_cell = env.get_rc(&root)?;
    let struct_cell = {
        let inner = match &*root_cell.borrow() {
            Value::Pointer(c) | Value::MutPointer(c) => Some(Rc::clone(c)),
            _ => None,
        };
        inner.unwrap_or(root_cell)
    };
    let leaf_val = {
        let borrowed = struct_cell.borrow();
        let mut cur: &Value = &*borrowed;
        for seg in &path {
            match cur {
                Value::Struct { fields, .. } | Value::Enum { fields, .. } => {
                    cur = fields.get(seg.as_str())?;
                }
                _ => return None,
            }
        }
        cur.clone()
    };
    let leaf_cell = Rc::new(RefCell::new(leaf_val));
    Some((struct_cell, path, leaf_cell))
}

fn is_lvalue_path_typed(expr: &crate::typed_ast::TypedExpr) -> bool {
    use crate::typed_ast::TypedExpr;
    match expr {
        TypedExpr::Ident(..) => true,
        TypedExpr::FieldAccess { object, .. }
        | TypedExpr::TupleAccess { object, .. }
        | TypedExpr::Index { object, .. } => is_lvalue_path_typed(object),
        _ => false,
    }
}

/// Recursively walk a typed lvalue path, collecting `PathSegment`s.
/// Returns the root binding name and the full segment list (root-to-leaf order).
fn build_mut_path(
    expr: &TypedExpr,
    env: &mut Environment,
    span: &Span,
) -> Result<(String, Vec<PathSegment>), MetelError> {
    match expr {
        TypedExpr::Ident(name, _, _) => Ok((name.clone(), vec![])),
        TypedExpr::FieldAccess { object, field, .. } => {
            let (root, mut path) = build_mut_path(object, env, span)?;
            path.push(PathSegment::Field(field.clone()));
            Ok((root, path))
        }
        TypedExpr::TupleAccess { object, index, .. } => {
            let (root, mut path) = build_mut_path(object, env, span)?;
            path.push(PathSegment::TupleIndex(*index));
            Ok((root, path))
        }
        TypedExpr::Index { object, index, .. } => {
            let (root, mut path) = build_mut_path(object, env, span)?;
            let idx_val = eval_expr(index, env)?.into_value();
            let i = match idx_val {
                Value::I64(n) if n >= 0 => n as usize,
                Value::U64(n) => n as usize,
                _ => return Err(MetelError::panic(
                    RuntimeErrorCode::R0004, "&mut: array index must be a non-negative integer", span)),
            };
            path.push(PathSegment::ArrayIndex(i));
            Ok((root, path))
        }
        _ => Err(MetelError::internal("build_mut_path: not a lvalue path")),
    }
}


// ── Control flow signals ──────────────────────────────────────────────────────

/// Returned by evaluation functions to handle non-local control flow.
/// Regular expression evaluation returns Signal::Value.
#[derive(Debug)]
pub enum Signal {
    Value(Value),
    Return(Value),
    Break(Value),       // carries value for `loop { break expr; }`
    Continue,
}

impl Signal {
    /// Extract the inner `Value`, consuming the signal.
    /// Panics for non-Value signals — callers that need the full signal must match directly.
    pub fn into_value(self) -> Value {
        match self {
            Signal::Value(v) => v,
            other => panic!("Signal::into_value called on non-Value signal: {other:?}"),
        }
    }
}

// ── Environment ───────────────────────────────────────────────────────────────

/// Lexically-scoped environment — a stack of hashmaps.
/// Runtime storage stays cell-backed, but closure capture chooses whether to
/// clone cells by value (`capture_clone`) or share them explicitly (`define_rc`,
/// pointers, reference receivers).
#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, Rc<RefCell<Value>>>>,
    /// Type context for construction-at-call-time of generic closures. Set once per module
    /// in `run_passes`; shared via `Rc` so cloning the environment is cheap.
    pub type_ctx: Option<std::rc::Rc<TypeCtx>>,
}

impl Default for Environment {
    fn default() -> Self { Self::new() }
}

impl Environment {
    pub fn new() -> Self {
        Self { scopes: vec![HashMap::new()], type_ctx: None }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a new binding in the current scope.
    /// Arrays are deep-cloned so each binding has an independent copy.
    pub fn define(&mut self, name: &str, value: Value) {
        let cell = Rc::new(RefCell::new(deep_clone_value(value)));
        self.scopes.last_mut().unwrap().insert(name.to_string(), cell);
    }

    pub fn define_rc(&mut self, name: &str, cell: Rc<RefCell<Value>>) {
        self.scopes.last_mut().unwrap().insert(name.to_string(), cell);
    }

    /// Look up a binding, searching from innermost to outermost scope.
    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(cell) = scope.get(name) {
                return Some(cell.borrow().clone());
            }
        }
        None
    }

    /// Assign to an existing binding anywhere in the scope chain.
    /// Arrays are deep-cloned so each binding has an independent copy.
    pub fn set(&self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter().rev() {
            if let Some(cell) = scope.get(name) {
                *cell.borrow_mut() = deep_clone_value(value);
                return true;
            }
        }
        false
    }

    /// Return the Rc for a binding (used by closures to share mutable state).
    pub fn get_rc(&self, name: &str) -> Option<Rc<RefCell<Value>>> {
        for scope in self.scopes.iter().rev() {
            if let Some(cell) = scope.get(name) {
                return Some(Rc::clone(cell));
            }
        }
        None
    }

    pub fn capture_clone(&self) -> Self {
        let scopes = self.scopes.iter()
            .map(|scope| {
                scope.iter()
                    .map(|(name, cell)| {
                        let cloned = deep_clone_value(cell.borrow().clone());
                        (name.clone(), Rc::new(RefCell::new(cloned)))
                    })
                    .collect()
            })
            .collect();
        Self { scopes, type_ctx: self.type_ctx.clone() }
    }
}

// Compute the environment key for an impl method.
//
/// Structured key for impl method dispatch.
///
/// Replaces the old flat string concatenation (`"TypeName::method_name"`) with a
/// typed representation. `From` impls require a three-part key because multiple
/// `From` impls can coexist for the same target type (one per source type).
enum ImplMethodKey<'a> {
    Regular  { type_name: &'a str, method_name: &'a str },
    FromImpl { target: &'a str, source: &'a str },
}

impl<'a> ImplMethodKey<'a> {
    fn from_block(type_name: &'a str, method_name: &'a str, impl_block: &'a crate::typed_ast::TypedImplBlock) -> Self {
        if impl_block.aspect_name.as_deref() == Some("From")
            && method_name == "from"
            && !impl_block.aspect_type_args.is_empty()
        {
            if let crate::ast::TypeExpr::Named(src, _) = &impl_block.aspect_type_args[0] {
                return ImplMethodKey::FromImpl { target: type_name, source: src.as_str() };
            }
        }
        ImplMethodKey::Regular { type_name, method_name }
    }

    fn to_env_key(&self) -> String {
        match self {
            ImplMethodKey::Regular  { type_name, method_name } => format!("{type_name}::{method_name}"),
            ImplMethodKey::FromImpl { target, source }         => format!("{target}::From<{source}>::from"),
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Evaluate a typed module graph produced by `check_graph`.
///
/// Each module is initialised in its own `Environment` seeded with builtins,
/// then cross-linked via the `imported_names` table populated by `check_graph`.
/// Modules are processed in topological order (dependencies before dependents).
/// See ADR-0029 for the isolation design and ADR-0019 for the superseded flat-merge approach.
pub fn evaluate_graph(graph: TypedModuleGraph) -> Result<(), MetelError> {
    CALL_STACK.with(|s| s.borrow_mut().clear());

    // module_envs: path → fully initialised Environment.
    // Built incrementally; later modules can look up values from earlier ones.
    let mut module_envs: HashMap<Vec<String>, Environment> = HashMap::new();

    let root_path = graph.modules.last()
        .map(|m| m.module_path.clone())
        .unwrap_or_default();

    for module in graph.modules {
        let mut env = Environment::new();
        builtins::register_builtins(&mut env);

        // Seed names imported from already-initialised dependency modules.
        for (local_name, (source_module, canonical_name)) in &module.imported_names {
            if let Some(src_env) = module_envs.get(source_module) {
                if let Some(val) = src_env.get(canonical_name) {
                    env.define(local_name, val);
                }
            }
        }

        // Build type context for construction-at-call-time of generic function bodies.
        let type_ctx = std::rc::Rc::new(TypeCtx {
            scheme_env: module.scheme_env.clone(),
            registry: graph.type_registry.clone(),
        });

        // Run the standard 3-pass + alias evaluation on this module's decls.
        run_passes(&module.decls, &module.import_aliases, &mut env, Some(type_ctx))?;

        module_envs.insert(module.module_path, env);
    }

    // Run main() from the root module's environment.
    let dummy = Span { start: 0, end: 0, filename: "<program>".to_string(), line: 0, col: 0 };
    let env = module_envs.get_mut(&root_path)
        .ok_or_else(|| MetelError::panic(RuntimeErrorCode::R0001, "root module not found", &dummy))?;
    run_main(env)
}

#[allow(dead_code)] // public API used by single-file test harness
pub fn evaluate(program: TypedProgram) -> Result<(), MetelError> {
    CALL_STACK.with(|s| s.borrow_mut().clear());
    let mut env = Environment::new();
    builtins::register_builtins(&mut env);
    run_passes(&program, &std::collections::HashMap::new(), &mut env, None)?;
    run_main(&mut env)
}

#[allow(dead_code)] // public API used by single-file test harness
pub fn evaluate_with_ctx(
    program: TypedProgram,
    ctx: TypeCtx,
) -> Result<(), MetelError> {
    CALL_STACK.with(|s| s.borrow_mut().clear());
    let mut env = Environment::new();
    builtins::register_builtins(&mut env);
    let type_ctx_rc = std::rc::Rc::new(ctx);
    run_passes(&program, &std::collections::HashMap::new(), &mut env, Some(type_ctx_rc))?;
    run_main(&mut env)
}

/// Run the standard 3-pass evaluation on `decls` into `env`.
///
/// Pass 1a: placeholder bindings so closures can capture each other's Rc.
/// Pass 1b: replace placeholders with real closures ("ties the knot").
/// Alias registration: bind aliased import names after closures exist.
/// Pass 2: evaluate top-level let/mut/stmt declarations in order.
///
/// `type_ctx` must be set on `env` before calling so that generic function bodies
/// capture it for construction-at-call-time.
fn run_passes(
    decls:    &TypedProgram,
    aliases:  &std::collections::HashMap<String, String>,
    env:      &mut Environment,
    type_ctx: Option<std::rc::Rc<TypeCtx>>,
) -> Result<(), MetelError> {
    env.type_ctx = type_ctx.clone();
    // Pass 1a
    for decl in decls {
        match decl {
            TypedDecl::Fun(f) => { env.define(&f.name, Value::Unit); }
            TypedDecl::Impl(impl_block) => {
                if let crate::ast::TypeExpr::Named(type_name, _) = &impl_block.target_type {
                    for method in &impl_block.methods {
                        let key = ImplMethodKey::from_block(type_name, &method.name, impl_block).to_env_key();
                        env.define(&key, Value::Unit);
                    }
                }
            }
            _ => {}
        }
    }

    // Pass 1b
    for decl in decls {
        match decl {
            TypedDecl::Fun(f) => {
                let (body, ctx) = match &f.body {
                    FunBody::Typed(b)   => (ClosureBody::Typed(b.clone()), None),
                    FunBody::Generic(b) => (ClosureBody::Untyped(b.clone()), env.type_ctx.clone()),
                };
                let captured = env.clone();
                env.set(&f.name, Value::Closure(Rc::new(ClosureValue {
                    name: Some(f.name.clone()), params: f.params.clone(), body, captured,
                    type_ctx: ctx, fun_type: None,
                })));
            }
            TypedDecl::Impl(impl_block) => {
                if let crate::ast::TypeExpr::Named(type_name, _) = &impl_block.target_type {
                    for method in &impl_block.methods {
                        let (body, ctx) = match &method.body {
                            FunBody::Typed(b)   => (ClosureBody::Typed(b.clone()), None),
                            FunBody::Generic(b) => (ClosureBody::Untyped(b.clone()), env.type_ctx.clone()),
                        };
                        let key = ImplMethodKey::from_block(type_name, &method.name, impl_block).to_env_key();
                        let captured = env.clone();
                        env.set(&key, Value::Closure(Rc::new(ClosureValue {
                            name: Some(method.name.clone()), params: method.params.clone(), body, captured,
                            type_ctx: ctx, fun_type: None,
                        })));
                    }
                }
            }
            _ => {}
        }
    }

    // Alias registration
    for (alias, canonical) in aliases {
        if let Some(val) = env.get(canonical) {
            if env.get(alias).is_none() {
                env.define(alias, val);
            }
        }
    }

    // Pass 2
    for decl in decls {
        if !matches!(decl, TypedDecl::Fun(_) | TypedDecl::Impl(_)) {
            eval_decl(decl, env)?;
        }
    }

    Ok(())
}

/// Locate and execute `main()` in `env`. Called after all passes complete.
fn run_main(env: &mut Environment) -> Result<(), MetelError> {
    let dummy = Span { start: 0, end: 0, filename: "<program>".to_string(), line: 0, col: 0 };
    let main_body = match env.get("main") {
        Some(Value::Closure(rc)) => rc.body.clone(),
        Some(Value::Unit) =>
            return Err(MetelError::panic(RuntimeErrorCode::R0002, "main() is generic — not supported", &dummy)),
        Some(_) =>
            return Err(MetelError::panic(RuntimeErrorCode::R0002, "`main` is not a function", &dummy)),
        None =>
            return Err(MetelError::panic(RuntimeErrorCode::R0001, "no main() function defined", &dummy)),
    };
    let main_sig = match &main_body {
        ClosureBody::Typed(b)   => eval_block(b, env),
        ClosureBody::Untyped(_) =>
            return Err(MetelError::panic(RuntimeErrorCode::R0002, "main() body could not be typed", &dummy)),
    };
    match main_sig? {
        Signal::Value(_) | Signal::Return(_) => Ok(()),
        other => Err(MetelError::internal(format!("unexpected signal from main(): {other:?}"))),
    }
}

// ── Block and declaration evaluation ─────────────────────────────────────────

/// Evaluate a block: push scope, run stmts, return tail (or Unit).
/// Non-Value signals (Return, Break, Continue) short-circuit and propagate out.
pub fn eval_block(block: &TypedBlock, env: &mut Environment) -> Result<Signal, MetelError> {
    env.push_scope();
    for decl in &block.stmts {
        let sig = eval_decl(decl, env)?;
        match sig {
            Signal::Value(_) => {}
            other => {
                env.pop_scope();
                return Ok(other);
            }
        }
    }
    let result = match &block.tail {
        Some(tail) => eval_expr(tail, env),
        None       => Ok(Signal::Value(Value::Unit)),
    };
    env.pop_scope();
    result
}

/// Evaluate a single declaration inside a block or at the top level.
fn eval_decl(decl: &TypedDecl, env: &mut Environment) -> Result<Signal, MetelError> {
    match decl {
        TypedDecl::Let(d) => {
            match eval_expr(&d.value, env)? {
                Signal::Value(val) => { env.define(&d.name, val); Ok(Signal::Value(Value::Unit)) }
                other => Ok(other),
            }
        }
        TypedDecl::Mut(d) => {
            match eval_expr(&d.value, env)? {
                Signal::Value(val) => { env.define(&d.name, val); Ok(Signal::Value(Value::Unit)) }
                other => Ok(other),
            }
        }
        TypedDecl::Fun(f) => {
            let (body, ctx) = match &f.body {
                FunBody::Typed(b)   => (ClosureBody::Typed(b.clone()), None),
                FunBody::Generic(b) => (ClosureBody::Untyped(b.clone()), env.type_ctx.clone()),
            };
            // Define a placeholder first so the closure can see itself via shared Rc
            // (enables self-recursion for functions defined inside other functions).
            env.define(&f.name, Value::Unit);
            let captured = env.clone();
            let closure = Value::Closure(Rc::new(ClosureValue {
                name:     Some(f.name.clone()),
                params:   f.params.clone(),
                body,
                captured,
                type_ctx: ctx,
                fun_type: None,
            }));
            env.set(&f.name, closure);
            Ok(Signal::Value(Value::Unit))
        }
        TypedDecl::Stmt(s) => eval_stmt(s, env),
        // Type-level declarations have no runtime representation.
        TypedDecl::Struct(_) | TypedDecl::Enum(_) | TypedDecl::Impl(_) | TypedDecl::Aspect(_) => {
            Ok(Signal::Value(Value::Unit))
        }
    }
}

// ── Statement evaluation ──────────────────────────────────────────────────────

pub fn eval_stmt(stmt: &TypedStmt, env: &mut Environment) -> Result<Signal, MetelError> {
    match stmt {
        TypedStmt::Expr(e) => {
            // Must propagate non-Value signals (Break/Continue/Return) that arise from
            // control-flow expressions used in statement position, e.g. `if (x) { break; }`.
            match eval_expr(e, env)? {
                Signal::Value(_) => Ok(Signal::Value(Value::Unit)),
                other            => Ok(other),
            }
        }
        TypedStmt::Return(r) => {
            let val = match &r.value {
                Some(e) => eval_expr(e, env)?.into_value(),
                None    => Value::Unit,
            };
            Ok(Signal::Return(val))
        }
        TypedStmt::Break(b) => {
            let val = match &b.value {
                Some(e) => eval_expr(e, env)?.into_value(),
                None    => Value::Unit,
            };
            Ok(Signal::Break(val))
        }
        TypedStmt::Continue(_) => Ok(Signal::Continue),

        TypedStmt::While(w) => {
            loop {
                match eval_expr(&w.condition, env)? {
                    Signal::Value(Value::Boolean(false)) => break,
                    Signal::Value(Value::Boolean(true))  => {}
                    Signal::Value(_) => return Err(MetelError::internal(
                        "while: expected boolean condition (typechecker should have caught this)",
                    )),
                    other => return Ok(other), // propagate Return from condition
                }
                match eval_block(&w.body, env)? {
                    Signal::Value(_) | Signal::Continue => {}
                    Signal::Break(_)       => break,
                    Signal::Return(v)      => return Ok(Signal::Return(v)),
                }
            }
            Ok(Signal::Value(Value::Unit))
        }

        TypedStmt::For(f) => {
            // The init binding lives in its own scope so it doesn't leak out.
            // PoC note: if eval_block errors inside the loop, this scope is not
            // popped (errors are fatal so it doesn't matter in practice).
            env.push_scope();
            if let Some(init) = &f.init {
                match init {
                    TypedForInit::Let(d) => {
                        let val = eval_expr(&d.value, env)?.into_value();
                        env.define(&d.name, val);
                    }
                    TypedForInit::Mut(d) => {
                        let val = eval_expr(&d.value, env)?.into_value();
                        env.define(&d.name, val);
                    }
                    TypedForInit::Expr(e) => { eval_expr(e, env)?; }
                }
            }
            let result = loop {
                if let Some(cond) = &f.condition {
                    match eval_expr(cond, env)? {
                        Signal::Value(Value::Boolean(false)) => break Ok(Signal::Value(Value::Unit)),
                        Signal::Value(Value::Boolean(true))  => {}
                        Signal::Value(_) => break Err(MetelError::internal(
                            "for: expected boolean condition (typechecker should have caught this)",
                        )),
                        other => break Ok(other),
                    }
                }
                match eval_block(&f.body, env)? {
                    Signal::Value(_) | Signal::Continue => {}
                    Signal::Break(_)        => break Ok(Signal::Value(Value::Unit)),
                    Signal::Return(v)       => break Ok(Signal::Return(v)),
                }
                if let Some(step) = &f.step {
                    eval_expr(step, env)?;
                }
            };
            env.pop_scope();
            result
        }

        TypedStmt::ForIn(fi) => {
            let iterable = eval_expr(&fi.iterable, env)?.into_value();
            eval_for_in(&fi.binding, fi.mutable, iterable, &fi.body, &fi.span, env)
        }
    }
}

fn eval_for_in(
    binding: &str,
    _mutable: bool,
    iterable: Value,
    body:     &TypedBlock,
    span:     &Span,
    env:      &mut Environment,
) -> Result<Signal, MetelError> {
    // Fast path for built-in sequence types.
    let fast_items: Option<Vec<Value>> = match &iterable {
        Value::Array(rc) => Some(rc.borrow().clone()),
        Value::Struct { name, fields } if name == "Range" => {
            let s = range_field(fields, "start", span)?;
            let e = range_field(fields, "end",   span)?;
            Some((s..e).map(Value::I64).collect())
        }
        Value::Struct { name, fields } if name == "RangeInclusive" => {
            let s = range_field(fields, "start", span)?;
            let e = range_field(fields, "end",   span)?;
            Some((s..=e).map(Value::I64).collect())
        }
        _ => None,
    };

    if let Some(items) = fast_items {
        for item in items {
            env.push_scope();
            env.define(binding, item);
            let sig = eval_block(body, env)?;
            env.pop_scope();
            match sig {
                Signal::Value(_) | Signal::Continue => {}
                Signal::Break(_)        => break,
                Signal::Return(v)       => return Ok(Signal::Return(v)),
            }
        }
        return Ok(Signal::Value(Value::Unit));
    }

    // User-defined Iterable: dispatch through TypeName::next.
    let type_name = match &iterable {
        Value::Struct { name, .. } => name.clone(),
        _ => return Err(MetelError::panic(RuntimeErrorCode::R0011,
            "for-in: expected Array, Range, or Iterable value", span)),
    };
    let next_key = ImplMethodKey::Regular { type_name: &type_name, method_name: "next" }.to_env_key();
    let next_fn = env.get(&next_key).ok_or_else(|| {
        MetelError::panic(RuntimeErrorCode::R0011,
            format!("for-in: `{type_name}` does not implement Iterable (no `next` method)"), span)
    })?;

    let iter_cell = Rc::new(RefCell::new(deep_clone_value(iterable)));
    loop {
        let result = call::call_method_function(
            next_fn.clone(),
            call::ReceiverBinding::Shared(Rc::clone(&iter_cell)),
            vec![],
            span,
        )?.into_value();
        let maybe_item: Option<Value> = match result {
            Value::Enum { name, variant, mut fields } if name == "Perhaps" => {
                if variant == "None" { None } else { Some(fields.remove("value").unwrap_or(Value::Unit)) }
            }
            _ => return Err(MetelError::internal("Iterable::next: expected Perhaps value")),
        };
        match maybe_item {
            None => break,
            Some(item) => {
                env.push_scope();
                env.define(binding, item);
                let sig = eval_block(body, env)?;
                env.pop_scope();
                match sig {
                    Signal::Value(_) | Signal::Continue => {}
                    Signal::Break(_)        => break,
                    Signal::Return(v)       => return Ok(Signal::Return(v)),
                }
            }
        }
    }
    Ok(Signal::Value(Value::Unit))
}

fn range_field(fields: &HashMap<String, Value>, name: &str, _span: &Span) -> Result<i64, MetelError> {
    match fields.get(name) {
        Some(Value::I64(n)) => Ok(*n),
        _ => Err(MetelError::internal(format!("range: missing or non-Int field `{name}`"))),
    }
}

// ── Expression evaluation ─────────────────────────────────────────────────────

pub fn eval_expr(expr: &TypedExpr, env: &mut Environment) -> Result<Signal, MetelError> {
    match expr {
        TypedExpr::Literal(lit, ty, _) => {
            use crate::ast::{IntKind, FloatKind};
            let val = match lit {
                // Unsuffixed int/float literals are polymorphic; their resolved type
                // is determined by context (defaulting to i64/f64 when unconstrained).
                Literal::Int(n) => match ty {
                    crate::types::Type::I8  => Value::I8(*n as i8),
                    crate::types::Type::I16 => Value::I16(*n as i16),
                    crate::types::Type::I32 => Value::I32(*n as i32),
                    crate::types::Type::U8  => Value::U8(*n as u8),
                    crate::types::Type::U16 => Value::U16(*n as u16),
                    crate::types::Type::U32 => Value::U32(*n as u32),
                    crate::types::Type::U64 => Value::U64(*n as u64),
                    _ => Value::I64(*n), // i64 (default) and Int alias
                },
                Literal::Float(f) => match ty {
                    crate::types::Type::F32 => Value::F32(*f as f32),
                    _ => Value::F64(*f), // f64 (default) and Float alias
                },
                Literal::SizedInt { value, kind } => match kind {
                    IntKind::I8  => Value::I8(*value as i8),
                    IntKind::I16 => Value::I16(*value as i16),
                    IntKind::I32 => Value::I32(*value as i32),
                    IntKind::I64 => Value::I64(*value as i64),
                    IntKind::U8  => Value::U8(*value as u8),
                    IntKind::U16 => Value::U16(*value as u16),
                    IntKind::U32 => Value::U32(*value as u32),
                    IntKind::U64 => Value::U64(*value as u64),
                },
                Literal::SizedFloat { value, kind } => match kind {
                    FloatKind::F32 => Value::F32(*value as f32),
                    FloatKind::F64 => Value::F64(*value),
                },
                Literal::Char(c)  => Value::Char(*c),
                Literal::Boolean(b)  => Value::Boolean(*b),
                Literal::Str(s)   => Value::Str(s.clone()),
                Literal::None     => Value::Enum { name: "Perhaps".into(), variant: "None".into(), fields: HashMap::new() },
                Literal::Unit     => Value::Unit,
            };
            Ok(Signal::Value(val))
        }

        TypedExpr::Ident(name, _, span) => {
            match env.get(name) {
                Some(val) => Ok(Signal::Value(val)),
                None => Err(MetelError::panic(
                    RuntimeErrorCode::R0003,
                    format!("undefined variable `{name}`"),
                    span,
                )),
            }
        }

        TypedExpr::Path(segments, _, _) => {
            // Unit enum variant: `Colour::Red` → Value::Enum { name: "Colour", variant: "Red", fields: {} }
            // A single-segment path is treated as an ident lookup.
            if segments.len() == 1 {
                let name = &segments[0];
                let span = expr.span();
                match env.get(name) {
                    Some(val) => Ok(Signal::Value(val)),
                    None => Err(MetelError::panic(
                        RuntimeErrorCode::R0003,
                        format!("undefined variable `{name}`"),
                        span,
                    )),
                }
            } else {
                // Check full qualified name (e.g. "Circle::new" for static methods).
                let key = segments.join("::");
                if let Some(val) = env.get(&key) {
                    return Ok(Signal::Value(val));
                }
                let name    = segments[segments.len() - 2].clone();
                let variant = segments[segments.len() - 1].clone();
                Ok(Signal::Value(Value::Enum { name, variant, fields: HashMap::new() }))
            }
        }

        TypedExpr::Tuple(elems, _, _) => {
            let mut vals = Vec::with_capacity(elems.len());
            for e in elems {
                vals.push(eval_expr(e, env)?.into_value());
            }
            Ok(Signal::Value(Value::Tuple(vals)))
        }

        TypedExpr::Array(elems, _, _) => {
            let mut vals = Vec::with_capacity(elems.len());
            for e in elems {
                vals.push(eval_expr(e, env)?.into_value());
            }
            Ok(Signal::Value(Value::Array(Rc::new(RefCell::new(vals)))))
        }

        TypedExpr::RepeatArray(elem, n, _, _) => {
            let val = eval_expr(elem, env)?.into_value();
            let vals = (0..*n).map(|_| val.clone()).collect::<Vec<_>>();
            Ok(Signal::Value(Value::Array(Rc::new(RefCell::new(vals)))))
        }

        TypedExpr::BinOp(lhs, op, rhs, _, span) => {
            // Short-circuit logical ops before evaluating rhs.
            if matches!(op, BinOp::And) {
                let l = eval_expr(lhs, env)?.into_value();
                return match l {
                    Value::Boolean(false) => Ok(Signal::Value(Value::Boolean(false))),
                    Value::Boolean(true)  => eval_expr(rhs, env),
                    _ => Err(MetelError::internal("&&: expected boolean (typechecker should have caught this)")),
                };
            }
            if matches!(op, BinOp::Or) {
                let l = eval_expr(lhs, env)?.into_value();
                return match l {
                    Value::Boolean(true)  => Ok(Signal::Value(Value::Boolean(true))),
                    Value::Boolean(false) => eval_expr(rhs, env),
                    _ => Err(MetelError::internal("||: expected boolean (typechecker should have caught this)")),
                };
            }

            let lv = eval_expr(lhs, env)?.into_value();
            let rv = eval_expr(rhs, env)?.into_value();
            lvalue::eval_binop(op, lv, rv, span)
        }

        TypedExpr::UnaryOp(op, operand, _, span) => {
            match op {
                UnaryOp::Ref => return match &**operand {
                    TypedExpr::Ident(name, _, _) => env.get_rc(name)
                        .map(|rc| Signal::Value(Value::Pointer(rc)))
                        .ok_or_else(|| MetelError::panic(RuntimeErrorCode::R0003, format!("undefined variable `{name}`"), span)),
                    other if is_lvalue_path_typed(other) => {
                        let v = eval_expr(operand, env)?.into_value();
                        Ok(Signal::Value(Value::Pointer(Rc::new(RefCell::new(v)))))
                    }
                    _ => Err(MetelError::internal("address-of requires an addressable lvalue (identifier, field access, tuple access, or array index)")),
                },
                UnaryOp::RefMut => return match &**operand {
                    TypedExpr::Ident(name, _, _) => env.get_rc(name)
                        .map(|rc| Signal::Value(Value::MutPointer(rc)))
                        .ok_or_else(|| MetelError::panic(RuntimeErrorCode::R0003, format!("undefined variable `{name}`"), span)),
                    other if is_lvalue_path_typed(other) => {
                        let (root_name, path) = build_mut_path(other, env, span)?;
                        let root = env.get_rc(&root_name).ok_or_else(|| MetelError::panic(
                            RuntimeErrorCode::R0003, format!("undefined variable `{root_name}`"), span))?;
                        Ok(Signal::Value(Value::MutFieldPointer { root, path }))
                    }
                    _ => Err(MetelError::internal("mutable address-of requires an addressable lvalue")),
                },
                _ => {}
            }
            let v = eval_expr(operand, env)?.into_value();
            let result = match (op, v) {
                (UnaryOp::Neg, Value::I64(n))   => Value::I64(n.wrapping_neg()),
                (UnaryOp::Neg, Value::I8(n))    => Value::I8(n.wrapping_neg()),
                (UnaryOp::Neg, Value::I16(n))   => Value::I16(n.wrapping_neg()),
                (UnaryOp::Neg, Value::I32(n))   => Value::I32(n.wrapping_neg()),
                (UnaryOp::Neg, Value::F64(f)) => Value::F64(-f),
                (UnaryOp::Neg, Value::F32(f))   => Value::F32(-f),
                (UnaryOp::Not, Value::Boolean(b))  => Value::Boolean(!b),
                (UnaryOp::Deref, Value::Pointer(rc)) | (UnaryOp::Deref, Value::MutPointer(rc)) => rc.borrow().clone(),
                (UnaryOp::Deref, Value::MutFieldPointer { root, path }) =>
                    read_path(&root.borrow(), &path, span)?,
                (UnaryOp::Neg, _) => return Err(MetelError::internal("unary `-`: expected numeric type (typechecker should have caught this)")),
                (UnaryOp::Not, _) => return Err(MetelError::internal("unary `!`: expected boolean (typechecker should have caught this)")),
                (UnaryOp::Deref, _) => return Err(MetelError::internal("unary `*`: expected pointer (typechecker should have caught this)")),
                _ => unreachable!("Ref/RefMut handled above"),
            };
            Ok(Signal::Value(result))
        }

        TypedExpr::Cast { expr: inner, target_type, span, .. } => {
            let v = eval_expr(inner, env)?.into_value();
            // Dispatch through From impl using the full aspect-signature key
            // "Target::From<Source>::from", then fall back to "Target::from"
            // (used by built-in Int::from / Float::from which have no type arg).
            if let crate::ast::TypeExpr::Named(target_name, _) = target_type {
                let src_name = match &v {
                    Value::Struct { name, .. } => Some(name.as_str()),
                    Value::I64(_)  => Some("i64"),
                    Value::I8(_)   => Some("i8"),
                    Value::I16(_)  => Some("i16"),
                    Value::I32(_)  => Some("i32"),
                    Value::U8(_)   => Some("u8"),
                    Value::U16(_)  => Some("u16"),
                    Value::U32(_)  => Some("u32"),
                    Value::U64(_)  => Some("u64"),
                    Value::F64(_) => Some("f64"),
                    Value::F32(_)   => Some("f32"),
                    Value::Char(_)  => Some("Char"),
                    Value::Boolean(_)  => Some("boolean"),
                    Value::Str(_)   => Some("String"),
                    _ => None,
                };
                let from_fn = src_name
                    .and_then(|s| env.get(&ImplMethodKey::FromImpl { target: target_name, source: s }.to_env_key()))
                    .or_else(|| env.get(&ImplMethodKey::Regular { type_name: target_name, method_name: "from" }.to_env_key()));
                if let Some(f) = from_fn {
                    return call::call_function(f, vec![v], span);
                }
            }
            // Identity cast fallback (same type, no from registered).
            Ok(Signal::Value(v))
        }

        TypedExpr::TryCast { expr: inner, target_type, .. } => {
            let v = eval_expr(inner, env)?.into_value();
            let result = try_numeric_cast(&v, target_type);
            let (variant, fields) = match result {
                Some(cast_val) => {
                    let mut f = HashMap::new();
                    f.insert("value".to_string(), cast_val);
                    ("Some".to_string(), f)
                }
                None => ("None".to_string(), HashMap::new()),
            };
            Ok(Signal::Value(Value::Enum {
                name: "Perhaps".to_string(),
                variant,
                fields,
            }))
        }

        TypedExpr::TupleAccess { object, index, span, .. } => {
            let v = eval_expr(object, env)?.into_value();
            match v {
                Value::Tuple(elems) => {
                    elems.into_iter().nth(*index).map(Signal::Value).ok_or_else(|| {
                        MetelError::panic(
                            RuntimeErrorCode::R0005,
                            format!("tuple index {index} out of bounds"),
                            span,
                        )
                    })
                }
                _ => Err(MetelError::internal("tuple access on non-tuple (typechecker should have caught this)")),
            }
        }

        TypedExpr::Index { object, index, span, .. } => {
            let arr = eval_expr(object, env)?.into_value();
            let idx = eval_expr(index, env)?.into_value();
            let i: usize = match idx {
                Value::U64(u) => u as usize,
                _ => return Err(MetelError::internal("index: expected u64 index (typechecker should have caught this)")),
            };
            match arr {
                Value::Array(rc) => {
                    let borrowed = rc.borrow();
                    if i >= borrowed.len() {
                        Err(MetelError::panic(
                            RuntimeErrorCode::R0004,
                            format!("index {i} out of bounds (len {})", borrowed.len()),
                            span,
                        ))
                    } else {
                        Ok(Signal::Value(borrowed[i].clone()))
                    }
                }
                _ => Err(MetelError::internal("index: expected Array (typechecker should have caught this)")),
            }
        }

        TypedExpr::If { condition, then_branch, else_branch, .. } => {
            match eval_expr(condition, env)? {
                Signal::Value(Value::Boolean(true))  => eval_block(then_branch, env),
                Signal::Value(Value::Boolean(false)) => match else_branch {
                    Some(branch) => eval_block(branch, env),
                    None         => Ok(Signal::Value(Value::Unit)),
                },
                Signal::Value(_) => Err(MetelError::internal("if: expected boolean condition (typechecker should have caught this)")),
                other => Ok(other), // propagate Return from condition
            }
        }

        TypedExpr::Loop { body, .. } => {
            loop {
                match eval_block(body, env)? {
                    Signal::Value(_) | Signal::Continue => {}
                    Signal::Break(val)      => return Ok(Signal::Value(val)),
                    Signal::Return(v)       => return Ok(Signal::Return(v)),
                }
            }
        }

        TypedExpr::Match(m) => {
            let scrutinee = eval_expr(&m.scrutinee, env)?.into_value();
            for arm in &m.arms {
                let mut bindings = HashMap::new();
                if !pattern::match_pattern(&arm.pattern, &scrutinee, &mut bindings) {
                    continue;
                }
                // Evaluate the guard (if any) in a scope that includes pattern bindings.
                if let Some(guard) = &arm.guard {
                    env.push_scope();
                    for (k, v) in &bindings { env.define(k, v.clone()); }
                    let guard_val = eval_expr(guard, env)?.into_value();
                    env.pop_scope();
                    match guard_val {
                        Value::Boolean(true)  => {}
                        Value::Boolean(false) => continue,
                        _ => return Err(MetelError::internal("match guard: expected boolean (typechecker should have caught this)")),
                    }
                }
                // Execute the arm body in a scope with pattern bindings.
                env.push_scope();
                for (k, v) in bindings { env.define(&k, v); }
                let result = eval_block(&arm.body, env);
                env.pop_scope();
                return result;
            }
            Err(MetelError::panic(RuntimeErrorCode::R0006, "match: no arm matched scrutinee", &m.span))
        }

        TypedExpr::Assign { target, op, value, span, .. } => {
            use crate::ast::AssignOp;
            use crate::typed_ast::TypedPlace;
            let rhs = eval_expr(value, env)?.into_value();
            match target {
                TypedPlace::Ident(name) => {
                    let new_val = if matches!(op, AssignOp::Assign) {
                        rhs
                    } else {
                        let cur = env.get(name).ok_or_else(|| {
                            MetelError::panic(RuntimeErrorCode::R0003, format!("assign: undefined `{name}`"), span)
                        })?;
                        lvalue::apply_assign_op(op, cur, rhs, span)?
                    };
                    if !env.set(name, new_val) {
                        return Err(MetelError::panic(
                            RuntimeErrorCode::R0003, format!("assign: undefined `{name}`"), span,
                        ));
                    }
                    Ok(Signal::Value(Value::Unit))
                }

                TypedPlace::Deref { object, span: tspan } => {
                    let ptr = eval_expr(object, env)?.into_value();
                    match ptr {
                        Value::Pointer(rc) | Value::MutPointer(rc) => {
                            let new_val = if matches!(op, AssignOp::Assign) {
                                rhs
                            } else {
                                let cur = rc.borrow().clone();
                                lvalue::apply_assign_op(op, cur, rhs, span)?
                            };
                            *rc.borrow_mut() = new_val;
                        }
                        Value::MutFieldPointer { root, path } => {
                            let new_val = if matches!(op, AssignOp::Assign) {
                                rhs
                            } else {
                                let cur = read_path(&root.borrow(), &path, tspan)?;
                                lvalue::apply_assign_op(op, cur, rhs, span)?
                            };
                            write_path(&mut root.borrow_mut(), &path, new_val, tspan)?;
                        }
                        _ => return Err(MetelError::panic(RuntimeErrorCode::R0003, "assign: dereference target is not a pointer", tspan)),
                    }
                    Ok(Signal::Value(Value::Unit))
                }

                TypedPlace::Index { object, index, span: tspan } => {
                    let arr_val = lvalue::eval_typed_place_value(object, env, tspan)?;
                    let idx_val = eval_expr(index, env)?.into_value();
                    let i = match idx_val {
                        Value::U64(u) => u as usize,
                        _ => return Err(MetelError::internal("index: expected u64 index (typechecker should have caught this)")),
                    };
                    match arr_val {
                        Value::Array(rc) => {
                            let len = rc.borrow().len();
                            if i >= len {
                                return Err(MetelError::panic(
                                    RuntimeErrorCode::R0004, format!("index {i} out of bounds (len {len})"), span,
                                ));
                            }
                            let new_val = if matches!(op, AssignOp::Assign) {
                                rhs
                            } else {
                                let cur = rc.borrow()[i].clone();
                                lvalue::apply_assign_op(op, cur, rhs, span)?
                            };
                            rc.borrow_mut()[i] = new_val;
                            Ok(Signal::Value(Value::Unit))
                        }
                        _ => Err(MetelError::internal(
                            "index assign: receiver is not an Array (typechecker should have caught this)",
                        )),
                    }
                }

                TypedPlace::Field { object, field, span: tspan } => {
                    let (rc, path) = lvalue::resolve_field_assign_root(object, field, env, tspan)?;
                    let mut borrowed = rc.borrow_mut();
                    // Navigate intermediate path segments to reach the parent struct.
                    let mut cur: &mut Value = &mut borrowed;
                    for segment in &path[..path.len() - 1] {
                        cur = match cur {
                            Value::Struct { fields, .. } | Value::Enum { fields, .. } => {
                                fields.get_mut(*segment).ok_or_else(|| {
                                    MetelError::panic(RuntimeErrorCode::R0008,
                                        format!("field assign: no field `{segment}`"), tspan)
                                })?
                            }
                            _ => return Err(MetelError::internal(
                                format!("field assign: `{segment}` is not a struct/enum"),
                            )),
                        };
                    }
                    let fields = match cur {
                        Value::Struct { fields, .. } | Value::Enum { fields, .. } => fields,
                        _ => return Err(MetelError::internal(
                            "field assign: receiver is not a struct/enum (typechecker should have caught this)",
                        )),
                    };
                    let leaf = path.last().expect("path is non-empty");
                    let new_val = if matches!(op, AssignOp::Assign) {
                        rhs
                    } else {
                        let cur = fields.get(*leaf).cloned().ok_or_else(|| {
                            MetelError::panic(
                                RuntimeErrorCode::R0008, format!("field assign: no field `{leaf}`"), tspan,
                            )
                        })?;
                        lvalue::apply_assign_op(op, cur, rhs, span)?
                    };
                    fields.insert((*leaf).to_string(), new_val);
                    Ok(Signal::Value(Value::Unit))
                }
            }
        }

        TypedExpr::StructLiteral { path, fields, span: _, .. } => {
            let mut field_vals: HashMap<String, Value> = HashMap::new();
            for (name, expr) in fields {
                field_vals.insert(name.clone(), eval_expr(expr, env)?.into_value());
            }
            if path.len() == 2 {
                Ok(Signal::Value(Value::Enum {
                    name:    path[0].clone(),
                    variant: path[1].clone(),
                    fields:  field_vals,
                }))
            } else {
                let name = path.last().ok_or_else(|| {
                    MetelError::internal("struct literal: empty path")
                })?.clone();
                Ok(Signal::Value(Value::Struct { name, fields: field_vals }))
            }
        }

        TypedExpr::FieldAccess { object, field, span, .. } => {
            let mut val = eval_expr(object, env)?.into_value();
            if let Some(deref) = deref_value(&val, span)? {
                val = deref;
            }
            let fields = match &val {
                Value::Struct { fields, .. } | Value::Enum { fields, .. } => fields,
                _ => return Err(MetelError::internal("field access on non-struct/enum (typechecker should have caught this)")),
            };
            fields.get(field).cloned().map(Signal::Value).ok_or_else(|| {
                MetelError::panic(RuntimeErrorCode::R0008, format!("no field `{field}` on value"), span)
            })
        }

        TypedExpr::MethodCall { receiver, method, args, span, .. } => {
            let recv_val = eval_expr(receiver, env)?.into_value();
            let arg_vals: Vec<Value> = args.iter()
                .map(|a| eval_expr(a, env).map(Signal::into_value))
                .collect::<Result<_, _>>()?;

            // Built-in type methods.
            if let (Value::Str(s), "len") = (&recv_val, method.as_str()) {
                return Ok(Signal::Value(Value::I64(s.chars().count() as i64)));
            }
            if let (Value::Array(arr), "len") = (&recv_val, method.as_str()) {
                return Ok(Signal::Value(Value::I64(arr.borrow().len() as i64)));
            }

            // User-defined struct/enum methods — looked up by "TypeName::method".
            let recv_type_view = deref_value(&recv_val, span)?.unwrap_or_else(|| recv_val.clone());
            let type_name = match &recv_type_view {
                Value::Struct { name, .. } | Value::Enum { name, .. } => name.clone(),
                Value::I64(_)   => "i64".to_string(),
                Value::F64(_) => "f64".to_string(),
                Value::Char(_)  => "Char".to_string(),
                Value::Boolean(_)  => "boolean".to_string(),
                Value::Str(_)   => "String".to_string(),
                _ => return Err(MetelError::panic(
                    RuntimeErrorCode::R0009,
                    format!("method `{method}` not found on this value"), span,
                )),
            };
            let key = format!("{type_name}::{method}");
            let func = env.get(&key).ok_or_else(|| {
                MetelError::panic(RuntimeErrorCode::R0009, format!("no method `{method}` on `{type_name}`"), span)
            })?;
            match &func {
                Value::Closure(rc)
                    if matches!(
                        rc.params.first().and_then(|p| p.receiver.clone()),
                        Some(crate::ast::ReceiverKind::Ref) | Some(crate::ast::ReceiverKind::RefMut)
                    ) =>
                {
                    // For field-access chains (e.g. `pair.a.tick()`) we can't hand the
                    // evaluator a direct Rc to the field because fields are stored by value
                    // inside the parent struct's HashMap.  Instead we clone the leaf value
                    // into a fresh cell, call through it, then write the (possibly mutated)
                    // value back into the parent struct.
                    let mut field_writeback: Option<(Rc<RefCell<Value>>, Vec<String>, Rc<RefCell<Value>>)> = None;

                    let receiver_binding = match receiver.as_ref() {
                        TypedExpr::Ident(name, _, _) => {
                            match env.get_rc(name).map(|cell| {
                                let inner = match &*cell.borrow() {
                                    Value::Pointer(inner) | Value::MutPointer(inner) => Some(Rc::clone(inner)),
                                    _ => None,
                                };
                                inner.unwrap_or(cell)
                            }) {
                                Some(cell) => call::ReceiverBinding::Shared(cell),
                                None => call::ReceiverBinding::Value(recv_type_view.clone()),
                            }
                        }
                        TypedExpr::FieldAccess { .. } => {
                            match lvalue_field_cell(receiver, env) {
                                Some((struct_cell, path, leaf_cell)) => {
                                    let binding = call::ReceiverBinding::Shared(Rc::clone(&leaf_cell));
                                    field_writeback = Some((struct_cell, path, leaf_cell));
                                    binding
                                }
                                None => receiver_cell_from_value(&recv_val)
                                    .map(call::ReceiverBinding::Shared)
                                    .unwrap_or(call::ReceiverBinding::Value(recv_type_view.clone())),
                            }
                        }
                        _ => receiver_cell_from_value(&recv_val)
                            .map(call::ReceiverBinding::Shared)
                            .unwrap_or(call::ReceiverBinding::Value(recv_type_view.clone())),
                    };

                    let result = call::call_method_function(func, receiver_binding, arg_vals, span)?;

                    if let Some((struct_cell, path, leaf_cell)) = field_writeback {
                        let new_val = leaf_cell.borrow().clone();
                        let last = path.last().unwrap();
                        let prefix = &path[..path.len() - 1];
                        let mut borrow = struct_cell.borrow_mut();
                        let mut cur: &mut Value = &mut *borrow;
                        for seg in prefix {
                            match cur {
                                Value::Struct { fields, .. } | Value::Enum { fields, .. } => {
                                    cur = fields.get_mut(seg.as_str()).unwrap();
                                }
                                _ => break,
                            }
                        }
                        if let Value::Struct { fields, .. } | Value::Enum { fields, .. } = cur {
                            fields.insert(last.clone(), new_val);
                        }
                    }

                    Ok(result)
                }
                _ => {
                    let mut all_args = vec![recv_type_view];
                    all_args.extend(arg_vals);
                    call::call_function(func, all_args, span)
                }
            }
        }

        TypedExpr::Call { callee, args, span, .. } => {
            let func_val = eval_expr(callee, env)?.into_value();
            let arg_vals: Vec<Value> = args.iter()
                .map(|a| eval_expr(a, env).map(Signal::into_value))
                .collect::<Result<_, _>>()?;
            call::call_function(func_val, arg_vals, span)
        }

        TypedExpr::Closure { params, body, ty, .. } => {
            let captured = env.capture_clone();
            Ok(Signal::Value(Value::Closure(Rc::new(ClosureValue {
                name:     None,
                params:   params.clone(),
                body:     ClosureBody::Typed(body.clone()),
                captured,
                type_ctx: None,
                fun_type: Some(ty.clone()),
            }))))
        }

        TypedExpr::GenericClosure { name, params, body, .. } => {
            let captured = env.capture_clone();
            Ok(Signal::Value(Value::Closure(Rc::new(ClosureValue {
                name:     name.clone(),
                params:   params.clone(),
                body:     ClosureBody::Untyped(body.clone()),
                captured,
                type_ctx: env.type_ctx.clone(),
                fun_type: None,
            }))))
        }

    }
}

// Returns Some(cast_value) if the value fits in the target type, None otherwise.
fn try_numeric_cast(v: &Value, target: &crate::ast::TypeExpr) -> Option<Value> {
    let target_name = match target {
        crate::ast::TypeExpr::Named(name, _) => name.as_str(),
        _ => return None,
    };

    // Extract a general integer/float from the source value.
    let as_i64: Option<i64> = match v {
        Value::I64(n)  => Some(*n),
        Value::I32(n)  => Some(*n as i64),
        Value::I16(n)  => Some(*n as i64),
        Value::I8(n)   => Some(*n as i64),
        Value::U64(n)  => i64::try_from(*n).ok(),
        Value::U32(n)  => Some(*n as i64),
        Value::U16(n)  => Some(*n as i64),
        Value::U8(n)   => Some(*n as i64),
        _ => None,
    };
    let as_f64: Option<f64> = match v {
        Value::F64(f) => Some(*f),
        Value::F32(f) => Some(*f as f64),
        _ => as_i64.map(|n| n as f64),
    };

    match target_name {
        "i8"  => as_i64.filter(|&n| n >= i8::MIN  as i64 && n <= i8::MAX  as i64).map(|n| Value::I8(n  as i8)),
        "i16" => as_i64.filter(|&n| n >= i16::MIN as i64 && n <= i16::MAX as i64).map(|n| Value::I16(n as i16)),
        "i32" => as_i64.filter(|&n| n >= i32::MIN as i64 && n <= i32::MAX as i64).map(|n| Value::I32(n as i32)),
        "i64" | "Int" => as_i64.map(Value::I64),
        "u8"  => as_i64.filter(|&n| n >= 0 && n <= u8::MAX  as i64).map(|n| Value::U8(n  as u8)),
        "u16" => as_i64.filter(|&n| n >= 0 && n <= u16::MAX as i64).map(|n| Value::U16(n as u16)),
        "u32" => as_i64.filter(|&n| n >= 0 && n <= u32::MAX as i64).map(|n| Value::U32(n as u32)),
        "u64" => match v {
            Value::U64(n) => Some(Value::U64(*n)),
            _ => as_i64.filter(|&n| n >= 0).map(|n| Value::U64(n as u64)),
        },
        "f32"  => as_f64.map(|f| Value::F32(f as f32)),
        "f64" | "Float" => as_f64.map(Value::F64),
        _ => None,
    }
}
