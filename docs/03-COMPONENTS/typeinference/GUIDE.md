# Type Inference Implementation Guide

## Overview

This guide provides the step-by-step workflow for implementing Yolang's type inference system incrementally through 8 phases.

## Implementation Workflow

### General Approach

Each phase follows the same pattern:

1. **Read the specification** in ROADMAP.md for the phase
2. **Examine test structure** in `tests/typeinference_tests.rs`
3. **Implement in source** in `src/typeinference/mod.rs`
4. **Write test assertions** replacing `todo!()` placeholders
5. **Run and verify tests** until all pass
6. **Move to next phase**

### Phase Implementation Template

```bash
# 1. Read the roadmap
→ Open ROADMAP.md
→ Find "Phase N: ___"  
→ Read "What", "Definition", "Tasks", "Test Cases"

# 2. Examine test stubs
→ Open tests/typeinference_tests.rs
→ Go to phase_N_xxx section
→ Look at test names and comments

# 3. Implement in source
→ Open src/typeinference/mod.rs
→ Add your structs, enums, functions
→ Implement Display, Debug as needed

# 4. Write test assertions
→ Replace todo!() with actual test code
→ Follow test case specifications from roadmap

# 5. Run tests
→ cargo test --test typeinference_tests phase_N

# 6. Debug and fix
→ Read error messages carefully
→ Use --nocapture for debugging output
→ Add println!() statements if needed

# 7. Verify completion
→ All tests for the phase pass
→ Move to next phase
```

## Test Structure and Usage

### Test Organization

All tests are in `tests/typeinference_tests.rs` organized by phase:

```rust
#[cfg(test)]
mod phase_1_type_variables {
    // Phase 1 tests here
}

#[cfg(test)]
mod phase_2_infer_types {
    // Phase 2 tests here
}

// ... etc for all 8 phases
```

### Running Tests

**All tests:**
```bash
cargo test --test typeinference_tests
```

**Specific phase:**
```bash
cargo test --test typeinference_tests phase_2
```

**Specific test:**
```bash
cargo test --test typeinference_tests test_type_var_creation
```

**With debug output:**
```bash
cargo test --test typeinference_tests phase_2 -- --nocapture
```

### Test Development Pattern

Tests are pre-structured with placeholders:

```rust
#[test]
fn test_something() {
    todo!("Implement this test")
}
```

Replace `todo!()` with actual assertions:

```rust
#[test]
fn test_something() {
    let result = my_function();
    assert_eq!(result, expected);
    
    // More specific assertions
    assert!(condition);
    assert_eq!(actual, expected);
    assert_ne!(actual, unexpected);
}
```

## Implementation Guidelines

### Code Structure

All implementation goes in `src/typeinference/mod.rs`:

```rust
// Phase 1: Type Variables
pub struct TypeVar(u32);
pub struct TypeVarGenerator { /* ... */ }

// Phase 2: Inference Types  
pub enum InferType { /* ... */ }

// Phase 3: Unification
pub fn unify(ty1: &InferType, ty2: &InferType) -> Result<Substitution, String> {
    // Implementation
}

// ... etc for all phases
```

### Error Handling

Use `Result` types for operations that can fail:

```rust
pub fn unify(ty1: &InferType, ty2: &InferType) -> Result<Substitution, String> {
    match (ty1, ty2) {
        (InferType::Concrete(a), InferType::Concrete(b)) if a == b => {
            Ok(Substitution::empty())
        }
        (InferType::Concrete(a), InferType::Concrete(b)) => {
            Err(format!("Cannot unify {} with {}", a, b))
        }
        // ... other cases
    }
}
```

### Display Implementation

Implement readable `Display` for debugging:

```rust
impl std::fmt::Display for InferType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InferType::Concrete(ty) => write!(f, "{}", ty),
            InferType::Var(var) => write!(f, "{}", var),
            InferType::Fun(params, ret) => {
                write!(f, "fun(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            // ... other variants
        }
    }
}
```

## Debugging Strategies

### When Tests Fail

1. **Read error messages carefully** - they often point to the exact issue
2. **Run single test** to focus: `cargo test test_name -- --nocapture`  
3. **Add debug output** with `println!` statements
4. **Check test expectations** - ensure your implementation matches the test's assumptions

### Common Issues

**Compilation errors:**
- Missing imports (`use` statements)
- Type mismatches
- Missing trait implementations

**Logic errors:**
- Incorrect algorithm implementation
- Wrong test assertions
- Edge cases not handled

**Test failures:**
- Implementation doesn't match specification
- Display format differences
- Incorrect error handling

### Debugging Example

```rust
#[test]
fn test_unify_concrete_types() {
    let ty1 = InferType::Concrete(Type::Int);
    let ty2 = InferType::Concrete(Type::Int);
    
    println!("Unifying: {} with {}", ty1, ty2);
    let result = unify(&ty1, &ty2);
    println!("Result: {:?}", result);
    
    assert!(result.is_ok());
}
```

## Phase-Specific Notes

### Phase 1: Type Variables
- Focus on the newtype pattern and generator
- Ensure Display format matches exactly: `?t0`, `?t1`, etc.
- Implement Hash and Ord for use in collections

### Phase 2: Inference Types
- Start with simple variants, add complexity gradually
- Helper constructors make tests easier to write
- Display format should be readable and consistent

### Phase 3: Unification
- The core algorithm - take time to understand it
- Occurs check is crucial for correctness
- Handle all type variant combinations

### Phase 4: Substitution
- Apply substitutions recursively through type structure
- Composition is key for combining multiple substitutions
- Test transitive substitution carefully

### Phase 5: Constraints
- Collection pattern: generate constraints, then solve
- Span information enables good error messages
- Batch solving handles interdependencies

### Phase 6: Type Schemes
- Generalization identifies quantified variables
- Instantiation creates fresh variables each time
- Free variable analysis is the tricky part

### Phase 7: Inference Context
- State management for the entire inference process
- Two environments: monomorphic and polymorphic
- Automatic instantiation on lookup

### Phase 8: Integration
- Connect inference to AST walking
- Generate constraints from expressions
- Produce typed AST from results

## Success Criteria

After each phase, you should be able to:

- **Explain the concept** in your own words
- **Run all tests** successfully for that phase  
- **Understand the code** you've written
- **See how it connects** to previous phases

Complete understanding is more important than speed. Take time to internalize each concept before moving forward.

## File Organization

```
tree-walk-interpreter/
├── src/
│   ├── typeinference/
│   │   └── mod.rs           ← Your implementation
│   └── lib.rs               ← Exports for tests
└── tests/
    └── typeinference_tests.rs   ← Test structure
```

All implementation goes in `mod.rs`. Tests import from there via `lib.rs`.

## Integration Points

The inference system connects to:

- **AST** (`src/ast/mod.rs`) - source of expressions to infer
- **Types** (`src/types/mod.rs`) - concrete type definitions  
- **Typechecker** (`src/typechecker/mod.rs`) - final integration point
- **Error** (`src/error/mod.rs`) - error reporting with source locations

Phase 8 brings these together into a working type checker.