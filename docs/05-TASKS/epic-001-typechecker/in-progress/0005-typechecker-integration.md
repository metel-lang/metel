# Task 0005: Integrate Type Inference into Typechecker

**Status:** in-progress  
**Epic:** epic-001-typechecker  
**Component:** typechecker  
**Spec Link:** docs/01-SPEC/LANGUAGE-SPEC.md#32-type-inference  
**Blocked By:** 0002

## What

Wire `InferContext` (from task 0002) into `typechecker::check()` to produce a
real `TypedProgram` from an untyped `Program`, and connect that output to the
evaluator so the full pipeline runs end-to-end.

This covers both *inference* (determining types) and *validation* (rejecting
ill-typed programs). In a constraint-based system these are not separate passes —
constraints both determine and validate simultaneously. A separate validation pass
is therefore not needed.

This is a multi-stage implementation: not all AST node types will be handled
immediately. Each stage adds coverage for more nodes and is tested with `.yolo`
programs in `tests/test_programs/`.

## Architecture

### Two-Pass Design (ADR-0002: Option C — re-derivation)

The full construction sequence is:

```rust
let (type_registry, initial_env) = pre_pass(&program);
let mut ctx = InferContext::new(type_registry, initial_env);
let (substitution, scheme_env) = infer(&program, &mut ctx)?;
let typed_program = construct(&program, &substitution, &scheme_env)?;
```

**Pre-pass** — produces a `TypeRegistry` (struct/enum field types) and an
`InitialEnv` (top-level function type variables). Both are injected into
`InferContext::new`; neither is mutated after construction.

**Pass 1 — Inference (`infer`)**: Walk the AST with `InferContext`, emitting
constraints. Solve all constraints at the end. Returns `(Substitution, SchemeEnv)`.
The `Substitution` maps all type variables to their solved concrete types.
The `SchemeEnv` holds the generalised type schemes for let-bound functions
(polymorphic functions are never pinned to a single concrete type in the
substitution alone).

**Pass 2 — Construction (`construct`)**: Walk the AST again with
`(Substitution, SchemeEnv)` as lookup tables. Re-derives each node's concrete
type by applying the same structural type rules as Pass 1, but with no
unification, no occurs check, and no fresh variable generation. Builds
`TypedExpr`/`TypedDecl` nodes directly. `SchemeEnv` is threaded through to
handle polymorphic call sites.

### Pre-Pass — Function Registration and Registry

The pre-pass has two responsibilities:

1. **TypeRegistry** (ADR-0001: `TypeDef` enum, pre-built and injected): scan
   all `StructDecl`/`EnumDecl`s, convert field types from `TypeExpr` to
   `InferType`, and store in a `TypeRegistry`. The registry is read-only for
   the entire inference walk.

2. **InitialEnv / function hoisting**: scan all top-level `FunDecl`s and
   register their names with fresh type variables. This allows forward references
   and mutual recursion. Concrete types are unified during Pass 1 when the bodies
   are walked.

Within Pass 1, hoisting continues **at every block entry**: when inference enters
a block, it first scans that block's direct `FunDecl`s and registers them before
visiting any other statement. This makes all `fun` declarations in the block
mutually visible regardless of declaration order (spec §4.3). Hoisting is
block-local — nested blocks are not scanned. Pass 2 needs no hoisting; all types
are resolved via `Substitution`/`SchemeEnv` regardless of declaration order.

### Type Conversion Rules

**`TypeExpr → InferType`** (for explicit annotations):
- `Named("Int", [])` → `InferType::int()`
- `Named("Float", [])` → `InferType::float()`
- `Named("Bool", [])` → `InferType::bool()`
- `Named("String", [])` → `InferType::str()`
- `Unit` → `InferType::unit()`
- `Tuple(ts)` → `InferType::Tuple(...)`
- `Array(t)` → `InferType::Array(...)`
- `Fun(params, ret)` → `InferType::Fun(...)`
- Other `Named` → `InferType::Named(name, args)` (user-defined types)

**`InferType → Type`** (after solving, for TypedAST construction):
- `Concrete(t)` → `t`
- `Var(v)` → **error**: "cannot infer type for `?tN`"
- Structural variants → recurse

### BinOp Constraint Rules

Operators are checked strictly via constraints now. In the future, arithmetic
operators will be dispatched through traits (e.g. `Add`, `Sub`) — at that point
this logic should be replaced by trait constraint generation.

> **Future note**: When traits are implemented, `a + b` should generate a
> `T: Add<Output=R>` trait constraint rather than a direct numeric type check.
> The current rules are a deliberate simplification.

Current rules:
- `+`, `-`, `*`, `/`, `%` — both operands must be numeric (`Int` or `Float`),
  result has the same type as the operands
- `==`, `!=`, `<`, `<=`, `>`, `>=` — both operands must be the same type,
  result is `Bool`
- `&&`, `||` — both operands must be `Bool`, result is `Bool`
- `..`, `..=` — both operands must be `Int`, result is a range (use `Named("Range", [Int])`)

### Unsupported Node Handling

AST nodes not yet implemented return `YoloscriptError::Internal` with a clear
"not yet supported" message, so unimplemented features fail loudly rather than
silently producing wrong types.

The following nodes are **permanently unsupported in this epic** and always
return "not yet supported" regardless of stage:

| Node | Reason | Where it lands |
|---|---|---|
| `FunDecl` with `GenericParam`s | Generics are out of scope | Epic 003 |
| `Expr::Cast` | Cast validation requires trait resolution | Epic 004 |
| `Expr::Path` | Needs a dedicated name-resolution pass | Module system (future) |

## Implementation Stages

### Stage 1 — Core expressions and let-bindings
- Literals (`Int`, `Float`, `Bool`, `Str`, `Unit`, `Nope`)
- Identifiers (lookup in `InferContext`)
- `BinOp` with constraint rules above
- `UnaryOp` (`-` on numeric, `!` on bool)
- `Decl::Let` and `Decl::Mut` with optional type annotation
- `Decl::Fun` with typed parameters and return type
- **Tests:** `tests/test_programs/phase8_stage1_*.yolo`

### Stage 2 — Control flow statements
- `Stmt::If` (condition must be `Bool`)
- `Stmt::While` (condition must be `Bool`)
- `Stmt::Return`
- `Stmt::Expr`
- `Expr::If` (both branches must unify)
- **Tests:** `tests/test_programs/phase8_stage2_*.yolo`

### Stage 3 — Composite expressions
- `Expr::Tuple`
- `Expr::Array`
- `Expr::Call`
- `Expr::Index`
- **Tests:** `tests/test_programs/phase8_stage3_*.yolo`

### Stage 4 — Advanced expressions
- `Expr::Closure`
- `Expr::Match`
- `Expr::MethodCall`
- `Expr::FieldAccess`
- `Expr::StructLiteral`
- `Stmt::For`, `Stmt::ForIn`, `Stmt::Loop`
- `Type::Never` / `InferType::Never`: loops with no reachable `break` infer to `!`; `!` coerces to any type (subtype of everything)
- **Tests:** `tests/test_programs/phase8_stage4_*.yolo`

## Acceptance Criteria

### Stage 1
- [ ] `ErrorCode` enum added to `src/error/mod.rs`; `TypeError` carries a code; `type_error()` updated
- [ ] `TypeExpr → InferType` conversion covers all annotation forms
- [ ] `InferType → Type` conversion errors on unresolved variables
- [ ] Pre-pass registers all top-level function names
- [ ] Literals infer to their concrete types
- [ ] Identifiers resolve via `InferContext::lookup`
- [ ] `BinOp` emits correct constraints and produces correct result type
- [ ] `UnaryOp` emits correct constraints
- [ ] `Decl::Let` / `Decl::Mut`: annotation (if present) is unified with inferred value type
- [ ] `Decl::Fun`: parameters and return type unified, body inferred, scheme generalized
- [ ] Stage 1 `.yolo` test programs pass through `check()` without error
- [ ] Type mismatches produce `YoloscriptError::TypeError` with source span

### Stage 2–4
- [ ] (to be filled in as stages begin)

### Final — Evaluator integration
- [ ] `typechecker::check()` output passes into `evaluator::evaluate()` without error
- [ ] Full pipeline `parse() → check() → evaluate()` works on a non-trivial program
- [ ] All previous tests still pass

## Testing

### Test programs

`.yolo` source files live in `tests/test_programs/inference/` and are run through
the full `parse() → check()` pipeline. Ten programs already exist covering the
scenarios for Stage 1:

| File | Scenario |
|---|---|
| `01_literals.yolo` | Each primitive literal infers its concrete type |
| `02_annotations.yolo` | Explicit annotations unified with inferred value type |
| `03_arithmetic.yolo` | BinOp and UnaryOp constraints, comparisons produce `Bool` |
| `04_functions.yolo` | Function declarations, body vs return type, call sites |
| `05_nested_calls.yolo` | Type propagation through chained function calls |
| `06_let_polymorphism.yolo` | Unannotated param → generalized scheme → independent instantiations |
| `07_forward_reference.yolo` | Pre-pass allows calling a function declared later in the file |
| `08_mut_bindings.yolo` | `mut` infers identically to `let` |
| `09_chained_arithmetic.yolo` | Transitive constraint resolution through nested expressions |
| `10_scoping.yolo` | Parameter scopes isolated between functions |

### Test harness

`tests/typeinference_tests.rs` contains a `programs_tests` module with one test
per `.yolo` file. The `load_and_check` helper reads the file, parses it, and calls
`check()`. Each test currently asserts `check()` returns `Ok` (parse is verified
as a side-effect). Each test has a `// TODO` block documenting which inferred
types to assert once `check()` is implemented.

Run the program tests with:
```bash
cargo test --test typeinference_tests programs_tests
```

Negative tests (expected type errors) use a `// ERROR` comment convention to be
defined when the test harness is extended for Stage 1.

## Open Questions

### ~~`Nope` literal type~~ ✓ resolved
`nope` infers to `Perhaps<?t0>` with a fresh `?t0`. If `?t0` is still unresolved
after constraint solving, it is a type error — the user must provide an explicit
annotation (e.g. `let x: Perhaps<Int> = nope`). No special-casing or defaulting.
Spec updated: §8.

### ~~Block return type vs `return` statement~~ ✓ resolved
`InferContext` gains a `current_return_type: Option<InferType>` field. When
inference enters a `FunDecl` body, it is set to the declared return type (or a
fresh type variable if unannotated) and restored to the previous value on exit.
`return expr` unifies `infer(expr)` with `ctx.current_return_type`. The tail
expression of the body is unified with it as well. Outside any function (top-level
expressions) the field is `None` and `return` is a type error.

### ~~Struct and enum type registry~~ ✓ resolved by ADR-0001
`TypeDef` enum with typed lookup methods; pre-built and injected into
`InferContext::new`. See [ADR-0001](../../../06-DECISIONS/closed/ADR-0001-type-registry.md).

### ~~Negative test convention~~ ✓ resolved
Per-line `// ERROR[EXXXX]` comments keyed on error codes, not message strings.
Message strings are volatile (wording improves over time); codes are stable.

```yolo
let x = nope;           // ERROR[E0002]
let y: Int = "hello";   // ERROR[E0001]
```

An optional human-readable hint may follow the code — the harness ignores it:

```yolo
let x = nope;           // ERROR[E0002] annotation required
```

**Initial error codes (coarse-grained; extend as new categories emerge):**

| Code  | Category |
|-------|----------|
| E0001 | Type mismatch — cannot unify two incompatible types |
| E0002 | Annotation required — type variable left unresolved after solving |
| E0003 | Undefined name — identifier not found in scope |
| E0004 | Arity mismatch — wrong number of arguments at a call site |
| E0005 | Invalid operand types — operator applied to wrong types |

**Harness logic** for any `.yolo` test file:
1. Scan source lines for `// ERROR[EXXXX]`; record `(line_number, code)`.
2. Run `check()`.
3. If annotations found: assert `Err`; assert the error's code and line number
   match one of the annotated pairs.
4. If no annotations: assert `Ok`.

**Prerequisite:** `YoloscriptError::TypeError` must carry an `ErrorCode` field
before any negative tests can be written. Add `ErrorCode` enum to
`src/error/mod.rs` and update `TypeError` and `type_error()` accordingly as
part of Stage 1.

### ~~Pass 1 → Pass 2 type transfer — or single-pass?~~ ✓ resolved by ADR-0002
Two-pass with re-derivation (Option C). Pass 1 returns `(Substitution, SchemeEnv)`;
Pass 2 re-derives types structurally with no constraint emission. Pre-pass
produces `(TypeRegistry, InitialEnv)` injected into `InferContext::new`. See
[ADR-0002](../../../06-DECISIONS/closed/ADR-0002-inference-pass-structure.md).

### Multiple error reporting
`solve_constraints` currently stops at the first unification failure. A better
user experience would collect all errors and report them together. This requires
a different solving strategy (continue past errors, collect them, return
`Vec<YoloscriptError>` instead of `Result`). Decide when to tackle this — it is a
cross-cutting change that affects the error type and all call sites.

## Notes

- Run stage 1 tests: `cargo test --test programs_tests stage1`
- Unsupported nodes intentionally error — this is tracked, not hidden
- Inference and validation are unified in the constraint-based approach; no separate validation pass is needed (absorbed from task 0003)
