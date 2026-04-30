# Type Inference System: Step-by-Step Roadmap

## Overview

Build the type inference system incrementally with tests for each phase. This document shows the exact steps, test structure, and integration points.

**Master Task**: Epic 001 Task - Build Type Inference System Step-by-Step  
**Test File**: `tests/typeinference_tests.rs`  
**Module**: `src/typeinference/mod.rs`  

---

## Phase 1: Type Variables ✅

**What**: Implement type variable generation and basic operations.

**Tasks**:
- [x] Create `TypeVar` struct: newtype wrapper around `u32`
- [x] Implement `Display` for `TypeVar` (format as `?t0`, `?t1`, etc.)
- [x] Create `TypeVarGenerator` for generating fresh type variables
- [x] Implement ordering and hashing for `TypeVar`

**Tests** (in `phase_1_type_variables`):
- ✅ `test_type_var_creation`: Create type variables with different IDs
- ✅ `test_type_var_display`: Display format is correct
- ✅ `test_type_var_generator_fresh`: Generator produces unique variables
- ✅ `test_type_var_generator_counter`: Counter increments correctly
- ✅ `test_type_var_ordering`: Type variables can be ordered
- ✅ `test_type_var_hashable`: Type variables can be used in HashSet/HashMap

**Status**: ✅ Phase 1 Complete

---

## Phase 2: Types During Inference

**What**: Represent types that may contain type variables (unlike `Type` which is concrete).

**Definition** (add to `src/typeinference/mod.rs`):
```rust
pub enum InferType {
    Concrete(Type),          // Resolved: Int, String, etc.
    Var(TypeVar),            // Unknown: ?t0, ?t1
    Fun(Vec<InferType>, Box<InferType>),  // Function: ?t0 -> ?t1
    Tuple(Vec<InferType>),   // Tuple: (?t0, Int, ?t1)
    Array(Box<InferType>),   // Array: ?t0[]
    Named(String, Vec<InferType>),  // Named: List<?t0>
}
```

**Tasks**:
- [ ] Define `InferType` enum with all variants
- [ ] Implement `Display` for `InferType`
- [ ] Add helper constructors: `int()`, `float()`, `bool()`, `str()`, `unit()`, `var()`

**Tests** (add to `phase_2_infer_types`):
```rust
#[test]
fn test_infer_type_concrete() { ... }

#[test]
fn test_infer_type_var() { ... }

#[test]
fn test_infer_type_function() { ... }

#[test]
fn test_infer_type_display() { ... }

#[test]
fn test_infer_type_constructors() { ... }
```

**Acceptance Criteria**:
- [ ] Can create all InferType variants
- [ ] Display output is correct and readable
- [ ] Constructors work as expected

---

## Phase 3: Unification Algorithm

**What**: Implement the core algorithm that solves type equations.

**Key Concept**: Given two types, make them equal by binding type variables.

**Examples**:
```
unify(Int, Int) → Success (already equal)
unify(?t0, Int) → Success, bind ?t0 = Int
unify(?t0, ?t1) → Success, bind ?t0 = ?t1
unify(Int, String) → Error (can't unify)
unify(fun(?t0) -> ?t0, fun(Int) -> Int) → Success, bind ?t0 = Int
```

**Tasks**:
- [ ] Implement `unify(ty1: &InferType, ty2: &InferType) -> Result<Substitution, String>`
- [ ] Handle concrete type unification (must be identical)
- [ ] Handle type variable unification (bind if not occurs check violation)
- [ ] Handle function type unification (unify parameter and return types)
- [ ] Implement occurs check to prevent infinite types (`?t0 = List<?t0>`)
- [ ] Handle composite types (tuple, array, named)

**Tests** (add to `phase_3_unification`):
```rust
#[test]
fn test_unify_same_concrete() { 
    // unify(Int, Int) → success
}

#[test]
fn test_unify_different_concrete() {
    // unify(Int, String) → error
}

#[test]
fn test_unify_var_with_concrete() {
    // unify(?t0, Int) → bind ?t0 = Int
}

#[test]
fn test_unify_var_with_var() {
    // unify(?t0, ?t1) → bind ?t0 = ?t1
}

#[test]
fn test_unify_function_types() {
    // unify(fun(?t0) -> ?t0, fun(Int) -> Int) → bind ?t0 = Int
}

#[test]
fn test_unify_occurs_check() {
    // unify(?t0, Array(?t0)) → error (infinite type)
}

#[test]
fn test_unify_tuple_types() {
    // unify((?t0, Int), (String, Int)) → bind ?t0 = String
}
```

**Acceptance Criteria**:
- [ ] Unifies identical concrete types
- [ ] Rejects incompatible concrete types
- [ ] Binds type variables correctly
- [ ] Prevents infinite types (occurs check)
- [ ] Works with nested/composite types

---

## Phase 4: Substitution

**What**: Represent and apply type variable bindings.

**Key Concept**: After unification produces bindings like `?t0 = Int`, substitution applies them everywhere.

**Definition**:
```rust
pub struct Substitution {
    bindings: HashMap<TypeVar, InferType>,
}
```

**Tasks**:
- [ ] Create `Substitution` struct with `HashMap<TypeVar, InferType>`
- [ ] Implement `bind(var, ty)` to record a binding
- [ ] Implement `lookup(var)` to find a binding
- [ ] Implement `apply(ty)` to replace all variables in a type (recursively)
- [ ] Implement `compose(other)` to combine two substitutions

**Tests** (add to `phase_4_substitution`):
```rust
#[test]
fn test_substitution_bind_and_lookup() { ... }

#[test]
fn test_substitution_apply_simple() {
    // subst: ?t0 = Int
    // apply to ?t0 → Int
}

#[test]
fn test_substitution_apply_nested() {
    // subst: ?t0 = Int, ?t1 = String
    // apply to (?t0, ?t1) → (Int, String)
}

#[test]
fn test_substitution_apply_recursive() {
    // subst: ?t0 = fun(?t1) -> ?t1, ?t1 = Int
    // apply to ?t0 → fun(Int) -> Int
}

#[test]
fn test_substitution_compose() { ... }
```

**Acceptance Criteria**:
- [ ] Bindings are stored and retrieved correctly
- [ ] `apply()` recursively replaces all variables
- [ ] Transitive bindings work (if ?t0 = ?t1 and ?t1 = Int, then ?t0 resolves to Int)
- [ ] `compose()` combines substitutions correctly

---

## Phase 5: Constraints

**What**: Represent type relationships discovered during analysis.

**Key Concept**: As you walk the AST, you collect constraints like "?t0 = Int" or "?t1 = fun(?t2) -> ?t2". Then solve them all at once.

**Definition**:
```rust
pub struct Constraint {
    pub lhs: InferType,
    pub rhs: InferType,
    pub span: Span,
}
```

**Tasks**:
- [ ] Create `Constraint` struct with left/right types and span
- [ ] Implement constraint creation helper
- [ ] Implement batch constraint solving: `solve_constraints(vec) -> Substitution`
- [ ] Implement error reporting with source locations

**Tests** (add to `phase_5_constraints`):
```rust
#[test]
fn test_constraint_creation() { ... }

#[test]
fn test_solve_single_constraint() {
    // Constraint: ?t0 = Int
    // Solve → substitution with ?t0 = Int
}

#[test]
fn test_solve_multiple_constraints() {
    // Constraints: ?t0 = Int, ?t1 = String, (?t0, ?t1) = (Int, String)
    // Solve → success, all satisfied
}

#[test]
fn test_solve_conflicting_constraints() {
    // Constraints: ?t0 = Int, ?t0 = String
    // Solve → error
}

#[test]
fn test_solve_transitive_constraints() {
    // Constraints: ?t0 = ?t1, ?t1 = Int
    // Solve → ?t0 = Int, ?t1 = Int
}
```

**Acceptance Criteria**:
- [ ] Constraints can be created with span information
- [ ] Single constraints are solved correctly
- [ ] Multiple non-conflicting constraints solve together
- [ ] Conflicting constraints produce errors
- [ ] Transitive constraints work (chain of bindings)

---

## Phase 6: Type Schemes (Let-Polymorphism)

**What**: Enable a single let-binding to work with different types.

**Key Concept**: 
- When you bind `let id = fun(x) { x }`, infer its type as `fun(?t0) -> ?t0`
- **Generalize** it to a scheme: `∀?t0. fun(?t0) -> ?t0`
- Each use gets fresh variables: first call binds `?t_fresh1`, second call binds `?t_fresh2`

**Definition**:
```rust
pub struct TypeScheme {
    pub quantified_vars: Vec<TypeVar>,  // Variables bound by ∀
    pub ty: InferType,
}
```

**Tasks**:
- [ ] Create `TypeScheme` struct
- [ ] Implement `generalize(ty, free_vars)` - identify which vars to quantify
- [ ] Implement `instantiate(ctx)` - create fresh variables for each use
- [ ] Add helper to collect free variables in a type
- [ ] Display type schemes in readable form (`∀α. α → α`)

**Tests** (add to `phase_6_type_schemes`):
```rust
#[test]
fn test_type_scheme_generalization() {
    // ty: fun(?t0) -> ?t0, free: {}
    // generalize → scheme with quantified_vars = [?t0]
}

#[test]
fn test_type_scheme_instantiation() {
    // scheme: ∀?t0. fun(?t0) -> ?t0
    // instantiate → fun(?t_fresh) -> ?t_fresh (different each time)
}

#[test]
fn test_type_scheme_polymorphism() {
    // Use same scheme twice:
    // First: instantiate to fun(?t1) -> ?t1, unify ?t1 = Int
    // Second: instantiate to fun(?t2) -> ?t2, unify ?t2 = String
    // Both should succeed (same binding, different types!)
}

#[test]
fn test_type_scheme_display() { ... }
```

**Acceptance Criteria**:
- [ ] Generalization identifies quantified variables correctly
- [ ] Instantiation creates fresh variables on each call
- [ ] Same scheme can be instantiated to different types
- [ ] Display format is readable

---

## Phase 7: Inference Context

**What**: State management for the inference process.

**Definition**:
```rust
pub struct InferContext {
    var_gen: TypeVarGenerator,
    env: HashMap<String, InferType>,        // Monomorphic bindings
    poly_env: HashMap<String, TypeScheme>,  // Polymorphic bindings
    constraints: Vec<Constraint>,
    subst: Substitution,
}
```

**Tasks**:
- [ ] Create `InferContext` struct
- [ ] Implement `fresh_var()` for generating type variables
- [ ] Implement `bind_var()` for monomorphic bindings
- [ ] Implement `bind_polymorphic()` for polymorphic bindings
- [ ] Implement `lookup_var()` - auto-instantiates polymorphic bindings
- [ ] Implement `add_constraint()` and `constraints()` getter
- [ ] Implement substitution getters/setters

**Tests** (add to `phase_7_inference_context`):
```rust
#[test]
fn test_fresh_var_generation() { ... }

#[test]
fn test_monomorphic_binding() { ... }

#[test]
fn test_polymorphic_binding() { ... }

#[test]
fn test_lookup_monomorphic() { ... }

#[test]
fn test_lookup_polymorphic_instantiation() {
    // Bind polymorphic scheme
    // Lookup twice
    // Should get different instantiations both times
}

#[test]
fn test_constraint_collection() { ... }
```

**Acceptance Criteria**:
- [ ] Type variables generated correctly
- [ ] Bindings stored and retrieved
- [ ] Polymorphic lookups auto-instantiate
- [ ] Constraints collected properly
- [ ] Substitution applied correctly

---

## Phase 8: Integration with Typechecker

**What**: Connect the inference system to the actual type checking pipeline.

**Tasks**:
- [ ] Update `typechecker::check()` to use the inference system
- [ ] Implement expression constraint generation (walk AST)
- [ ] Implement statement handling
- [ ] Implement let-binding handling with automatic generalization
- [ ] Integrate constraint solving
- [ ] Generate typed AST from inference results

**Tests** (integration tests):
```rust
#[test]
fn test_infer_literal() {
    // Program: let x = 42;
    // Infer x: Int
}

#[test]
fn test_infer_function_call() { ... }

#[test]
fn test_infer_polymorphic_use() {
    // Program: let id = fun(x) { x };
    //          let a = id(42);
    //          let b = id("hi");
    // Infer: a: Int, b: String, id: polymorphic
}
```

---

## Test Running

Run tests with:
```bash
cd tree-walk-interpreter
cargo test --test typeinference_tests
```

Or run specific phase:
```bash
cargo test --test typeinference_tests phase_1
cargo test --test typeinference_tests phase_2
# etc.
```

---

## Implementation Order

Follow this order to build understanding:

1. **Phase 1** ✅ - Type variables (foundation)
2. **Phase 2** - InferType (what we're inferring)
3. **Phase 3** - Unification (core algorithm)
4. **Phase 4** - Substitution (applying solutions)
5. **Phase 5** - Constraints (collecting relations)
6. **Phase 6** - Type Schemes (polymorphism)
7. **Phase 7** - InferContext (state management)
8. **Phase 8** - Integration (real world)

Each phase builds on previous ones. Complete all tests before moving on.

---

## Notes

- Tests are organized by phase in `typeinference_tests.rs`
- Add placeholder test functions as you go (`#[test] fn test_...() { todo!() }`)
- Once implementation is done, remove `todo!()` and write assertions
- Use descriptive test names that explain what's being tested
- Include comments in tests explaining the test case

---

**Status**: Phase 1 complete, ready to start Phase 2.
