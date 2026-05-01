# Type Inference Implementation Roadmap

## Overview

This roadmap breaks down the type inference system implementation into 8 incremental phases, each building on the previous ones. Each phase focuses on a specific component and includes complete test coverage.

---

## Phase 1: Type Variables

**What**: Implement type variable generation and basic operations.

**Definition**:
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

**Tasks**:
- Create `TypeVar` struct: newtype wrapper around `u32`
- Implement `Display` for `TypeVar` (format as `?t0`, `?t1`, etc.)
- Create `TypeVarGenerator` for generating fresh type variables
- Implement ordering and hashing for `TypeVar`

**Test Cases**:
- Type variable creation with different IDs
- Display format verification
- Generator produces unique variables
- Counter increments correctly
- Type variables can be ordered and hashed

---

## Phase 2: Inference Types

**What**: Represent types that may contain type variables (unlike `Type` which is concrete).

**Definition**:
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
- Define `InferType` enum with all variants
- Implement `Display` for `InferType`
- Add helper constructors: `int()`, `float()`, `bool()`, `str()`, `unit()`, `var()`

**Test Cases**:
- Create all InferType variants
- Verify display output format
- Test helper constructors
- Nested type construction

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
- Implement `unify(ty1: &InferType, ty2: &InferType) -> Result<Substitution, String>`
- Handle concrete type unification (must be identical)
- Handle type variable unification (bind if not occurs check violation)
- Handle function type unification (unify parameter and return types)
- Implement occurs check to prevent infinite types (`?t0 = List<?t0>`)
- Handle composite types (tuple, array, named)

**Test Cases**:
- Unify identical concrete types
- Reject incompatible concrete types
- Unify variables with concrete types
- Unify variables with variables
- Unify complex function types
- Occurs check prevention
- Composite type unification

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
- Create `Substitution` struct with `HashMap<TypeVar, InferType>`
- Implement `bind(var, ty)` to record a binding
- Implement `lookup(var)` to find a binding
- Implement `apply(ty)` to replace all variables in a type (recursively)
- Implement `compose(other)` to combine two substitutions

**Test Cases**:
- Bind and lookup operations
- Apply substitutions to simple types
- Apply substitutions to nested types
- Recursive substitution application
- Substitution composition

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
- Create `Constraint` struct with left/right types and span
- Implement constraint creation helper
- Implement batch constraint solving: `solve_constraints(vec) -> Substitution`
- Implement error reporting with source locations

**Test Cases**:
- Create constraints with span information
- Solve single constraints
- Solve multiple non-conflicting constraints
- Handle conflicting constraints with errors
- Transitive constraint resolution

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
- Create `TypeScheme` struct
- Implement `generalize(ty, free_vars)` - identify which vars to quantify
- Implement `instantiate(ctx)` - create fresh variables for each use
- Add helper to collect free variables in a type
- Display type schemes in readable form (`∀α. α → α`)

**Test Cases**:
- Generalize types with different free variable contexts
- Instantiate schemes to fresh variables
- Verify same scheme can instantiate to different types
- Test polymorphic identity function example
- Display format verification

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
- Create `InferContext` struct
- Implement `fresh_var()` for generating type variables
- Implement `bind_var()` for monomorphic bindings
- Implement `bind_polymorphic()` for polymorphic bindings
- Implement `lookup_var()` - auto-instantiates polymorphic bindings
- Implement `add_constraint()` and `constraints()` getter
- Implement substitution getters/setters

**Test Cases**:
- Fresh variable generation
- Monomorphic variable binding and lookup
- Polymorphic binding with automatic instantiation
- Multiple lookups of same polymorphic binding produce different instances
- Constraint collection and management

---

## Phase 8: Integration with Typechecker

**What**: Connect the inference system to the actual type checking pipeline.

**Tasks**:
- Update `typechecker::check()` to use the inference system
- Implement expression constraint generation (walk AST)
- Implement statement handling
- Implement let-binding handling with automatic generalization
- Integrate constraint solving
- Generate typed AST from inference results

**Integration Points**:
```rust
fn infer_expr(expr: &Expr, ctx: &mut InferContext) -> InferType {
    match expr {
        Expr::Literal(lit) => infer_literal(lit),
        Expr::Var(name) => ctx.lookup_var(name).unwrap_or_else(|| {
            // Error: undefined variable
        }),
        Expr::Call { func, args } => {
            let func_ty = infer_expr(func, ctx);
            let arg_tys: Vec<_> = args.iter().map(|a| infer_expr(a, ctx)).collect();
            let ret_ty = ctx.fresh_var();
            let expected = InferType::Fun(arg_tys, Box::new(ret_ty.clone()));
            ctx.add_constraint(func_ty, expected, expr.span());
            ret_ty
        }
        // ... other expressions
    }
}

fn infer_let_binding(name: String, rhs: Expr, ctx: &mut InferContext) {
    let rhs_type = infer_expr(rhs, ctx);
    ctx.bind_polymorphic(name, rhs_type);  // Automatic generalization
}
```

**Test Cases**:
- Infer literal expressions
- Infer variable references
- Infer function calls
- Polymorphic let-binding inference
- Full program inference with multiple bindings
- Error handling for type mismatches

---

## Implementation Order

Follow this order to build understanding incrementally:

1. **Phase 1** - Type variables (foundation)
2. **Phase 2** - InferType (what we're inferring)
3. **Phase 3** - Unification (core algorithm)
4. **Phase 4** - Substitution (applying solutions)
5. **Phase 5** - Constraints (collecting relations)
6. **Phase 6** - Type Schemes (polymorphism)
7. **Phase 7** - InferContext (state management)
8. **Phase 8** - Integration (real world)

Each phase builds on previous ones. Complete all tests for each phase before moving on.

## Test Structure

All tests are organized in `tests/typeinference_tests.rs`:

```rust
#[cfg(test)]
mod phase_1_type_variables { ... }

#[cfg(test)]
mod phase_2_infer_types { ... }

#[cfg(test)]
mod phase_3_unification { ... }

// ... etc for all phases
```

Run tests with:
```bash
# All tests
cargo test --test typeinference_tests

# Specific phase
cargo test --test typeinference_tests phase_3

# With output
cargo test --test typeinference_tests phase_3 -- --nocapture
```

## Key Benefits

This incremental approach provides:

- **Understanding**: Each phase focuses on one concept
- **Testing**: Every component is tested before integration
- **Debugging**: Issues are caught early in small components
- **Foundation**: Later phases build on tested infrastructure
- **Learning**: Clear progression from simple to complex concepts