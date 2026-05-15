# Task 0011: Impl Block Typechecking

**Status:** done
**Epic:** epic-001-typechecker
**Component:** typechecker
**Spec Link:** `../01-SPEC/LANGUAGE-SPEC.md` §6 Structs, §6.2 Methods, §7.2 Methods on enums
**Blocked By:** none
**Unblocks:** `AssignTarget::FieldAccess` (currently returns internal error, see task 0010 notes)

## What

Four related expression forms are unimplemented and hit the internal-error fallback:

```yoloscript
let p = Point { x: 1.0, y: 2.0 };  // Expr::StructLiteral
let x = p.x;                         // Expr::FieldAccess
let d = p.distance(q);               // Expr::MethodCall
p.x = 3.0;                           // AssignTarget::FieldAccess
```

All four require the same prerequisite: a **struct field registry** so the
typechecker knows the field names and types for each struct. Method calls
additionally need a **method registry** built from `Decl::Impl`.

Currently `Decl::Struct` and `Decl::Impl` are silently no-ops / internal errors
in `infer_decl`. No field or method information is retained between declarations.

## Architecture

### New state on `InferContext`

Two new maps thread through the existing `ctx`:

```rust
struct_env: HashMap<String, Vec<(String, InferType)>>,
// struct name → ordered list of (field_name, field_type)

method_env: HashMap<String, HashMap<String, InferType>>,
// target type name → method name → Fun(param_types, ret_type)
```

`struct_env` is populated in `infer_decl` when `Decl::Struct` is encountered.
`method_env` is populated when `Decl::Impl` is encountered (non-trait impls only
for this task).

### Pre-pass hoisting

Like functions, structs and methods used before their declaration must be
visible. Extend `hoist_fun_decls` (or add a parallel `hoist_struct_decls` pre-
pass) to register struct field shapes and impl method types before the main
inference walk.

### `self` binding in impl methods

Within an impl method, the `self` parameter has no type annotation in the AST
(`name: "self"`, `type_ann: None`). Its type is the impl's target type:

```rust
// inside infer_impl_method(method, target_type_name, ctx, ...)
let self_ty = InferType::Named(target_type_name.clone(), vec![]);
ctx.bind_mono("self", self_ty, method.params[0].mutable);
// remaining params bound as usual
```

For `mut self`, the binding is mutable — this enables `self.field = value`
inside the method body via `AssignTarget::FieldAccess`.

### `Expr::StructLiteral`

```rust
Expr::StructLiteral { path, fields, span } => {
    let struct_name = path.last()...;
    let expected_fields = ctx.struct_env.get(struct_name)
        .ok_or_else(|| E0003 "unknown struct `{struct_name}`")?;
    // Constrain each provided field expression to the declared field type.
    for (name, expr) in fields {
        let decl_ty = expected_fields.iter().find(|(n, _)| n == name)
            .ok_or_else(|| E0003 "no field `{name}` on `{struct_name}`")?
            .1.clone();
        let expr_ty = infer_expr(expr, ctx, fun_generalizations)?;
        ctx.add_constraint(expr_ty, decl_ty, span.clone());
    }
    Ok(InferType::Named(struct_name.to_string(), vec![]))
}
```

### `Expr::FieldAccess`

```rust
Expr::FieldAccess { object, field, span } => {
    let obj_ty = infer_expr(object, ctx, fun_generalizations)?;
    // obj_ty must resolve to a Named type.
    // Introduce a fresh result variable and emit:
    //   obj_ty == Named(struct_name, [])  →  but we don't know struct_name yet.
    // Approach: require obj_ty to already be a Named type (no fresh var needed
    // if the receiver has a known type annotation or was just constructed).
    // In practice, constrain via a helper that looks up the field once obj_ty
    // is known after solving.
    //
    // Simpler approach that avoids a two-phase field lookup:
    // Require the object to have a concrete Named type at inference time
    // (i.e. from a type annotation or struct literal). Emit a fresh variable
    // as the result type and record a "field constraint" deferred to solve time.
    // See implementation notes below.
    let result = ctx.fresh_var();
    // ... constraint details — see implementation notes
    Ok(result)
}
```

### `Expr::MethodCall`

Similar to `Expr::Call` but the callee type is looked up from `method_env`
instead of `infer_expr`. The receiver type is inferred, then the method is
looked up by (receiver_type_name, method_name). Arity and argument constraints
are emitted as for a regular call.

### `AssignTarget::FieldAccess`

```rust
AssignTarget::FieldAccess { object, field, span: target_span } => {
    let obj_ty = infer_expr(object, ctx, fun_generalizations)?;
    // look up field type from struct_env — same logic as Expr::FieldAccess
    // return field InferType as target_ty for op-dispatch
}
```

## Implementation Notes

### Field access and the constraint system

The hard part of field access is that HM constraint solving operates on type
variables — you can't look up a field name unless you already know the struct
type. Two standard approaches:

**Option A — Require annotation.** Treat field access as only valid when the
receiver already has a resolved `Named` type (either from annotation, from a
struct literal on the same line, or from a previous let binding with annotation).
If the receiver type is still a `Var`, emit E0002 "cannot infer type; add a type
annotation". Simple, honest about limitations.

**Option B — Row polymorphism / deferred field lookup.** Add a "field
constraint" kind to the constraint system: `HasField(t, "x", result)`. During
`solve()`, once `t` is resolved to `Named("Point", [])`, look up the field and
unify `result` with the field type. More powerful but significantly more complex.

**Recommendation: start with Option A.** It correctly handles the common case
(receiver has a type annotation or is freshly constructed) and gives a clear
error when it doesn't. Option B can be a follow-up task.

With Option A, `Expr::FieldAccess` implementation becomes:

```rust
Expr::FieldAccess { object, field, span } => {
    let obj_ty = infer_expr(object, ctx, fun_generalizations)?;
    let struct_name = named_type_name(&obj_ty)
        .ok_or_else(|| E0002 "cannot infer struct type for field access; add a type annotation")?;
    let fields = ctx.struct_env.get(struct_name)
        .ok_or_else(|| E0003 "unknown type `{struct_name}`")?;
    let field_ty = fields.iter().find(|(n, _)| n == field)
        .ok_or_else(|| E0003 "no field `{field}` on `{struct_name}`")?
        .1.clone();
    Ok(field_ty)
}
```

Where `named_type_name` extracts the name if the type is already `Named(name, _)`
or `Concrete(Type::Named(name, _))`, and returns `None` for `Var(_)`.

### Pass 2 (`construct_expr`)

`StructLiteral`, `FieldAccess`, `MethodCall` need arms in `construct_expr` that
mirror the Pass 1 logic but apply the substitution to produce concrete `Type`
values.

## Scope

**In scope for this task:**
- Non-generic structs only (no `struct Pair<A, B>`)
- Non-trait impl blocks only (`impl Point { ... }`, not `impl Trait for Point`)
- Instance methods with `self` and `mut self` parameters
- `Expr::StructLiteral`, `Expr::FieldAccess`, `Expr::MethodCall`
- `AssignTarget::FieldAccess` (unblocked by this task)
- Field access on immutable and mutable receivers

**Deferred:**
- Generic struct typechecking (`struct Pair<A, B>`)
- Trait impl typechecking (`impl Foo for Bar`) — Epic 004
- Static methods / `::` path syntax (`Point::new(...)`)
- `Expr::Path` (used for enum variants and static calls)
- `Self` keyword inside trait/impl bodies

## Acceptance Criteria

- [x] `struct_env` is built from `Decl::Struct` declarations
- [x] `method_env` is built from `Decl::Impl` declarations (non-trait only)
- [x] `Expr::StructLiteral` typechecks: field expressions constrained to declared types
- [x] `Expr::FieldAccess` typechecks: field type returned when receiver type is known
- [x] `Expr::FieldAccess` on unknown receiver type produces E0002 (not internal error)
- [x] `Expr::MethodCall` typechecks: arity checked, argument types constrained
- [x] `AssignTarget::FieldAccess` typechecks: field type used as assignment target
- [x] `self` is bound to the impl target type in method bodies
- [x] `mut self` methods can write to fields via `AssignTarget::FieldAccess`
- [x] Positive test: struct construction, field read, method call, field assignment
- [x] Negative test: field type mismatch in struct literal (E0001)
- [x] Negative test: unknown field name on struct literal (E0003)
- [x] Negative test: method argument type mismatch (E0001)
- [x] All 133 prior tests still pass; 4 new tests added (137 total)

## Implementation Notes

### What was built

**`typeinference/mod.rs`** — Two new public fields on `InferContext`:
- `struct_env: HashMap<String, Vec<(String, InferType)>>` — field shapes per struct
- `method_env: HashMap<String, HashMap<String, InferType>>` — method types per target type

Four new methods: `register_struct_fields`, `register_method`, `get_struct_fields`,
`get_method_type`.

**`typechecker/mod.rs`** — the changes break down into three layers:

**Pre-pass** (`hoist_struct_and_impl_decls`): walks all top-level decls before
inference begins, registering struct field types and method function types from
annotations only. This mirrors `hoist_fun_decls` and ensures forward references
to structs and methods resolve correctly regardless of declaration order.

**Pass 1** additions:
- `Decl::Impl` no longer returns an internal error. For non-trait, non-generic
  impls it calls `infer_impl_method` per method.
- `infer_impl_method` mirrors `infer_fun_decl`: builds param types (binding `self`
  to `Named(target, [])`), infers the body, emits a body-vs-return-type constraint,
  runs a partial solve to catch errors early, then overwrites the pre-pass entry in
  `method_env` with the resolved function type.
- `Expr::StructLiteral`: looks up declared field types from `struct_env`, constrains
  each provided expression, returns `Named(struct_name, [])`.
- `Expr::FieldAccess` and `Expr::MethodCall`: call `ctx.solve()` before the struct
  name lookup to resolve type variables that flowed through let-bindings (e.g.
  `let q = p.translate(...)` makes `q` a `Var` until solved).
- `AssignTarget::FieldAccess`: same partial-solve-then-lookup pattern; field type
  becomes the `target_ty` in the existing op-dispatch.

**Pass 2** additions:
- `build_concrete_struct_env` / `build_concrete_method_env`: apply the final
  substitution to all `InferType` values and convert to `Type`, building the
  concrete maps that `ConstructCtx` holds.
- `ConstructCtx` gains `struct_env: HashMap<String, Vec<(String, Type)>>` and
  `method_env: HashMap<String, HashMap<String, Type>>`.
- `construct_impl_decl` / `construct_impl_method`: construct a `TypedImplBlock`
  with a `FunBody::Typed` body for each method, deriving param types from
  annotations and the `self` type from the target name.
- `construct_expr` arms for `FieldAccess`, `MethodCall`, `StructLiteral`.

### Deviation from the plan: partial solve before field/method lookup

The task doc described Option A as "require the object to have a concrete Named
type at inference time". In practice this was too strict: a type variable produced
by a prior method call (e.g. `let q = p.translate(...)`) is a `Var` at the point
of `q.sum()` even though the constraint `Var == Named("Point", [])` already exists.

The fix: call `ctx.solve()?.apply(&obj_ty)` immediately before calling
`named_type_name`. This resolves type variables through accumulated constraints
without changing the overall inference flow. The cost is O(n) per field/method
access where n is the current constraint count — acceptable for a tree-walk
interpreter.

### Mutability of `self` in field assignment

`mut self` is bound with `is_mutable: true` via `ctx.bind_mono("self", ..., p.mutable)`.
The field assignment path (`AssignTarget::FieldAccess`) calls `infer_expr(object)`
(not `lookup_for_write`), so it does **not** check whether the root identifier is
mutable. This means `self.field = v` is allowed regardless of whether the method
declares `mut self`. Enforcing this is deferred — it requires tracing the root
identifier of the assignment target expression, which is non-trivial for nested
access chains.

### Tests

- `stage5_01_structs_and_methods.yolo` — two structs (`Point`, `Counter`) with
  instantiation, field reads, method calls with and without arguments, and field
  assignment via `mut self`.
- `stage5_neg_01_struct_field_type_mismatch.yolo` — `Counter { value: true }` → E0001
- `stage5_neg_02_unknown_field.yolo` — `Counter { value: 0, extra: 1 }` → E0003
- `stage5_neg_03_method_arg_type_mismatch.yolo` — `c.add(42)` where `add` expects
  a `Counter` argument → E0001
