# Type Inference Concepts

## Overview

This document explains the theoretical concepts underlying Yolang's type inference system, which implements the Hindley-Milner algorithm with let-polymorphism.

## Core Concepts

### Type Variables

Type variables represent unknown types during inference. They act as placeholders that get unified with concrete types as more information becomes available.

```
?t0, ?t1, ?t2  -- Type variables
```

**Properties:**
- Generated fresh for each unknown type
- Can be unified with concrete types or other type variables
- Must satisfy the occurs check to prevent infinite types

### Inference Types vs Concrete Types

**Concrete Types** (`Type` enum): Fully resolved types like `Int`, `String`, `fun(Int) -> String`
**Inference Types** (`InferType` enum): Types that may contain type variables like `?t0`, `fun(?t0) -> ?t1`

Inference types are used during the inference process, then resolved to concrete types.

### Unification

Unification is the process of making two types equal by binding type variables.

```
unify(Int, Int) → Success (already equal)
unify(?t0, Int) → Success, bind ?t0 = Int
unify(?t0, ?t1) → Success, bind ?t0 = ?t1
unify(Int, String) → Error (incompatible)
unify(fun(?t0) -> ?t0, fun(Int) -> Int) → Success, bind ?t0 = Int
```

**Occurs Check**: Prevents infinite types by ensuring a type variable doesn't occur in its own definition:
```
unify(?t0, Array(?t0)) → Error (would create infinite type)
```

### Substitution

A substitution is a mapping from type variables to types that represents the solutions found by unification.

```rust
// Example substitution
?t0 → Int
?t1 → String
?t2 → fun(Int) -> String
```

Substitutions are applied to types to replace variables with their bindings:
```
apply({?t0 → Int}, (?t0, ?t0)) → (Int, Int)
```

### Constraint Generation and Solving

Instead of unifying types immediately, the inference system collects **constraints** during AST traversal, then solves them all together.

```
// From: let x = 42; let y = x + 1;
Constraints:
  ?t_x = Int        (from literal 42)
  ?t_y = ?t_x       (from assignment)
  ?t_x = Int        (from + operation)
```

This approach provides better error reporting and handles complex interdependencies.

## Let-Polymorphism Concepts

### Type Schemes

A **type scheme** is a type with universally quantified variables:

```
∀α. α → α    (identity function — works with any type)
∀α β. α → β → α  (first function — returns first of two arguments)
Int → Int    (monomorphic type — no quantified variables)
```

In code, represented as:
```rust
pub struct TypeScheme {
    pub quantified_vars: Vec<TypeVar>,  // [?t0] for ∀α
    pub ty: InferType,                  // The body: ?t0 → ?t0
}
```

### Generalization

When you bind a polymorphic closure to `let`, its inferred type is **generalized** into a type scheme:

```yolang
let id = fun(x) { x };
```

**Inference steps:**
1. Infer the body `{ x }` has type `?t0` (parameter type)
2. Infer the function has type `fun(?t0) -> ?t0`
3. **Generalize:** Identify free type variables (variables not constrained by context)
   - `?t0` is free (not used elsewhere)
   - Becomes quantified: `∀?t0. fun(?t0) -> ?t0`
4. Bind `id` to the scheme in the polymorphic environment

### Instantiation

When you **use** a polymorphic binding, the scheme is instantiated with fresh type variables:

```yolang
let id = fun(x) { x };  // Scheme: ∀α. α → α
id(42);                 // First use: instantiate to ?t_fresh1 → ?t_fresh1
                        //   Unify: ?t_fresh1 = Int
id("hello");            // Second use: instantiate to ?t_fresh2 → ?t_fresh2
                        //   Unify: ?t_fresh2 = String
```

### Complete Example: Polymorphic Identity Function

```yolang
let id = fun(x) { x };
let y = id(42);
let z = id("hello");
```

**Step 1: Infer function type**
```
Parameter x: type variable ?t0
Body returns x: type ?t0
Function type: fun(?t0) -> ?t0
```

**Step 2: Generalize for let-binding**
```
Free vars in environment: {} (empty context)
All vars in fun(?t0) -> ?t0 are free: {?t0}
Generalized scheme: ∀?t0. fun(?t0) -> ?t0
Bind: id ↦ TypeScheme { quantified_vars: [?t0], ty: fun(?t0) -> ?t0 }
```

**Step 3: First use `id(42)`**
```
Lookup id → get scheme ∀?t0. fun(?t0) -> ?t0
Instantiate: replace ?t0 with fresh ?t1
Instance type: fun(?t1) -> ?t1
Call argument: 42 (type Int)
Generate constraint: ?t1 = Int
Solve → ?t1 = Int
Result: id(42) : Int
```

**Step 4: Second use `id("hello")`**
```
Lookup id → get same scheme ∀?t0. fun(?t0) -> ?t0
Instantiate: replace ?t0 with fresh ?t2 (different variable!)
Instance type: fun(?t2) -> ?t2
Call argument: "hello" (type String)
Generate constraint: ?t2 = String
Solve → ?t2 = String
Result: id("hello") : String
```

The key insight: the **same binding** can be used with **different types** because each use gets fresh type variables.

### Type Annotations and Constraints

Explicit type annotations constrain polymorphism:

```yolang
let id: fun(Int) -> Int = fun(x) { x };  // Monomorphic
let id2 = fun(x) { x };                  // Polymorphic
```

**With annotation:**
1. Infer RHS: `fun(?t0) -> ?t0`
2. Check against annotation: `fun(Int) -> Int`
3. Unify: `?t0 = Int`
4. Generalize: No free variables remain
5. Result: Monomorphic binding

**Without annotation:**
1. Infer RHS: `fun(?t0) -> ?t0`
2. Generalize: `?t0` is free
3. Result: Polymorphic scheme `∀?t0. fun(?t0) -> ?t0`

### Environment and Lexical Scoping

The type environment tracks variable bindings and their types. Free variable analysis is crucial for correct generalization in nested scopes:

```yolang
let make_adder = fun(x) {
    fun(y) { x + y }
};
```

**Analysis:**
```
Outer function:
  Parameter x: ?t0
  
Inner function:
  Parameter y: ?t1
  Body: x + y
  Constraint: ?t0 = ?t1 = Numeric
  
Environment when generalizing inner function:
  Free vars in environment: {?t0} (x is in scope)
  Free vars in inner function: {?t0, ?t1}
  
Generalization:
  Can only quantify vars not constrained by environment
  Quantified: {?t1} (NOT ?t0, it's bound by x)
  Result: ∀?t1. fun(?t1) -> ?t1 where ?t1 = ?t0
```

This ensures each `make_adder` call produces a function specialized to one numeric type.

## Algorithm Components

### Type Variable Generation

```rust
pub struct TypeVar(u32);

pub struct TypeVarGenerator {
    next_id: u32,
}

impl TypeVarGenerator {
    pub fn fresh(&mut self) -> TypeVar {
        let id = self.next_id;
        self.next_id += 1;
        TypeVar(id)
    }
}
```

### Constraint-Based Inference

The modern approach to type inference uses constraint generation followed by constraint solving:

```rust
pub struct Constraint {
    pub lhs: InferType,
    pub rhs: InferType,
    pub span: Span,  // For error reporting
}

// Generate constraints during AST walk
fn infer_expr(expr: &Expr, ctx: &mut InferContext) -> InferType {
    match expr {
        Expr::Literal(lit) => infer_literal(lit),
        Expr::Call { func, args } => {
            let func_ty = infer_expr(func, ctx);
            let arg_tys: Vec<_> = args.iter().map(|a| infer_expr(a, ctx)).collect();
            let ret_ty = ctx.fresh_var();
            let expected = InferType::Fun(arg_tys.clone(), Box::new(ret_ty.clone()));
            ctx.add_constraint(func_ty, expected, expr.span());
            ret_ty
        }
        // ... other expressions
    }
}

// Solve all constraints together
fn solve_constraints(constraints: Vec<Constraint>) -> Result<Substitution, Error> {
    let mut subst = Substitution::new();
    for constraint in constraints {
        let unified = unify(&constraint.lhs, &constraint.rhs)?;
        subst = subst.compose(unified);
    }
    Ok(subst)
}
```

### Hindley-Milner Algorithm Structure

1. **Constraint Generation**: Walk AST, generate type variables and constraints
2. **Constraint Solving**: Unify all constraints to produce substitution
3. **Generalization**: Create type schemes for let-bindings
4. **Instantiation**: Create fresh instances when using polymorphic bindings

## Advanced Concepts

### Recursive Types and Occurs Check

The occurs check prevents infinite types:

```
// This would create infinite type
unify(?t0, List(?t0)) → Error

// Because ?t0 = List(?t0) = List(List(?t0)) = List(List(List(?t0))) ...
```

### Principal Types

The Hindley-Milner algorithm guarantees **principal types** - the most general type that captures all possible uses:

```yolang
let f = fun(x) { x };
// Principal type: ∀α. α → α
// Not: Int → Int (too specific)
// Not: ∀α β. α → β (too general)
```

### Comparison with Other Type Systems

| Feature | Hindley-Milner | Rust | TypeScript |
|---------|----------------|------|-------------|
| Type parameters | Implicit, inferred | Explicit declaration | Both implicit and explicit |
| Polymorphism | Let-polymorphism | Explicit generics | Structural typing |
| Type inference | Complete for rank-1 | Local inference | Gradual typing |
| Error reporting | Unification-based | Trait-based | Structural mismatch |

### Limitations and Extensions

**Rank-1 Restriction**: Standard HM cannot infer higher-rank polymorphism:
```yolang
// This requires rank-2 types (not supported)
let apply_twice = fun(f, x) { f(f(x)) };
let poly_id = fun(g) { g(g) };  // Would need ∀α. (∀β. β → β) → α → α
```

**Possible Extensions:**
- Higher-rank polymorphism (requires type annotations)
- Type classes/traits (constrained polymorphism)
- Dependent types (types depending on values)
- Effect systems (tracking side effects in types)

### Arbitrary-Rank Types (Not Planned)

> **Status**: Not planned for implementation. Documented here for reference.

#### What rank means

The *rank* of a type describes how deeply `∀` quantifiers can appear inside function argument types.

- **Rank-0**: Monotype — no quantifiers at all. `Int`, `fun(Int) -> String`.
- **Rank-1** (HM): `∀` only at the outermost level. `∀α. α → α`. When a rank-1 polymorphic function is passed as an argument, the caller first instantiates it to a monotype — the callee never sees the `∀`.
- **Rank-2**: `∀` may appear once inside a function argument type. The callee, not the caller, chooses the type.
- **Rank-N / arbitrary rank**: `∀` can appear at any depth.

```
-- Rank-1 (HM): id is instantiated before being passed
applyInt : (Int -> Int) -> Int -> Int

-- Rank-2: argument must be polymorphic; callee picks α
applyToAny : (∀α. α → α) → (Int, Bool)

-- Rank-3: the argument takes a rank-2 argument
applyHigher : ((∀α. α → α) → Int) → Int
```

#### Why HM cannot handle rank-2

In Algorithm W, every function argument is unified as a **monotype**. When the inferencer sees a call site it does not know, from the argument alone, whether the callee expects a polymorphic function — that information would have to flow backwards against the direction of inference. For rank ≥ 2, pure bottom-up inference is insufficient.

Full System F (arbitrary-rank) type inference is **undecidable** (Girard 1972, Reynolds 1974).

#### What rank-2 inference requires

Rank-2 inference is decidable, but requires several additions beyond HM:

1. **Richer type representation** — `Fun` argument types must be able to hold polytypes (`∀α. τ`), not just monotypes. In the current `Type` enum, `Fun(Vec<Type>, Box<Type>)` only holds monotypes; argument positions would need a `PolyType` wrapper.

2. **Skolem (rigid) type variables** — When checking an expression against `∀α. τ`, the variable `α` is replaced by a fresh *skolem constant* that cannot be unified with anything. If a skolem variable escapes its lexical scope during unification, it is a type error.

3. **Bidirectional type checking** — Pure bottom-up inference is replaced by two modes:
   - `infer(env, expr) → Type`: synthesize a type with no expected type (equivalent to Algorithm W)
   - `check(env, expr, expected)`: verify an expression has a specific expected type, enabling deep skolemization

4. **Subsumption instead of plain unification** — At argument positions, Robinson unification is replaced by a *subsumption check*: `σ₁ ≼ σ₂` meaning "σ₁ is at least as polymorphic as σ₂". `∀α. α → α` subsumes `Int → Int`; the reverse does not hold.

5. **Mandatory annotations at rank-2 sites** — The inferencer cannot discover that an argument should be polymorphic without a hint. Rank-2 argument types must be annotated explicitly by the programmer.

#### Implementation impact

The following changes would be needed if rank-2 were ever pursued:

| Component | Current state | Change required |
|---|---|---|
| `types/mod.rs` | `Fun(Vec<Type>, Box<Type>)` — monotypes only | `Fun(Vec<ArgType>, Box<Type>)` where `ArgType` may be a polytype |
| `typeinference/mod.rs` | `TypeVar` (flexible only) | Add `SkolemVar` (rigid, non-unifiable) with scope escape checking |
| Inference algorithm | Planned as Algorithm W | Extend with bidirectional `check` mode alongside `infer` |
| Unification | Planned as Robinson unification | Add `subsumes` entry point for argument positions |
| Parser / AST | No higher-rank annotation syntax | Syntax and AST nodes for rank-2 argument annotations |

The bidirectional extension is additive: `infer` mode is Algorithm W, and `check` mode is layered on top. A system designed with the `ArgType` alias in place would limit the refactor surface when upgrading.

The reference algorithm is Peyton Jones et al., *Practical Type Inference for Arbitrary-Rank Types* (JFP 2007), which is the basis for GHC's `RankNTypes` extension.

## Error Handling

Type errors in Hindley-Milner systems typically arise from:

1. **Unification failures**: `unify(Int, String)` 
2. **Occurs check violations**: `?t0 = List(?t0)`
3. **Missing variables**: Using undefined identifiers
4. **Arity mismatches**: `f(x, y)` where `f : α → β`

Good error messages require:
- Source location tracking
- Constraint provenance (why was this constraint generated?)
- Type reconstruction for display
- Suggestion systems

## Summary

Yolang's type inference system implements the Hindley-Milner algorithm with:

- **Complete type inference** for rank-1 polymorphism
- **Let-polymorphism** enabling code reuse without explicit type parameters  
- **Principal type inference** guaranteeing most general types
- **Constraint-based solving** for better error reporting
- **Occurs check** preventing infinite types
- **Lexical scoping** with proper free variable analysis