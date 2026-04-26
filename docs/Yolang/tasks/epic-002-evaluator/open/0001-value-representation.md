# Task 0001: Value Representation and Basic Evaluation

**Status:** open  
**Epic:** epic-002-evaluator  
**Component:** evaluator  
**Spec Link:** spec/Language Spec.md#Runtime-Values  
**Blocked By:** epic-001 task 0004 (basic types)

## What

Define how **runtime values** are represented in the interpreter and implement basic expression evaluation for literals and simple operations.

A Value is what the interpreter produces when evaluating code. Each value carries its type information.

Example:
```rust
enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Unit,
    Function(...),
}
```

## Design

**Value Type:**
- Each variant corresponds to a type from the type system
- Values carry enough info for further operations
- Type information available for runtime checks

**Basic Literal Evaluation:**
- `Literal(Int(42))` → `Value::Int(42)`
- `Literal(Bool(true))` → `Value::Bool(true)`
- `Literal(Str("hi"))` → `Value::String("hi")`
- Empty/unit literals

**Environment/Context:**
- Variable bindings: `Map<String, Value>`
- Function definitions: `Map<String, Function>`
- Scoping (nested environments)

## Acceptance Criteria

- [ ] Value enum defined with all basic types
- [ ] Literal expressions evaluate to values
- [ ] Variables can be looked up in environment
- [ ] Basic scoping with nested environments works
- [ ] Type information preserved in values
- [ ] Error handling for undefined variables
- [ ] Tests verify literal evaluation works
- [ ] All Epic 001 tests still pass

## Notes

- Keep Value simple; complexity comes in task 0002 (expression evaluation)
- Functions are complex; stub them for now (task 0003)
- Don't implement full garbage collection yet
- Use Rust's String and Vec types for dynamic data
