# Task 0002: Expression Evaluation

**Status:** open  
**Epic:** epic-002-evaluator  
**Component:** evaluator  
**Spec Link:** spec/Language Spec.md#Expression-Evaluation  
**Blocked By:** 0001

## What

Implement evaluation for **all 20 TypedExpr variants**:

Basic:
- Literal, Ident, Path, Tuple, Array
- BinOp, UnaryOp, Cast

Access:
- FieldAccess, TupleAccess, Index

Operations:
- Assign, Call, MethodCall
- StructLiteral

Control Flow:
- If, Match, Loop

Functions:
- Closure, Call

Error:
- PropagateError

## Design

**Expression Evaluator:**
- Recursive evaluation function: `eval_expr(TypedExpr, env) -> Result<Value, Error>`
- Evaluates children, combines results
- Uses type information for validation

**Binary Operations:**
- Arithmetic: `+`, `-`, `*`, `/`, `%` (on numeric types)
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logical: `&&`, `||`
- Use type info to select correct operation

**Array/Tuple Operations:**
- Indexing: `arr[i]` returns element
- Field access: `tuple.0`, `tuple.1`
- Length: built-in function

**Assignment:**
- Update variable in environment
- Return assigned value

## Acceptance Criteria

- [ ] All 20 TypedExpr variants evaluate correctly
- [ ] Binary operations work for all valid type combinations
- [ ] Array/tuple indexing and access work
- [ ] Assignment updates environment
- [ ] Type information used to validate operations
- [ ] Error messages for invalid operations (e.g., indexing non-array)
- [ ] Recursive expression evaluation works
- [ ] Complex expressions nested correctly
- [ ] Tests cover each variant and common combinations
- [ ] All previous tests still pass

## Notes

- Control flow (If, Match, Loop) can be stubbed here; full implementation in task 0003
- Function calls stubbed; full implementation in task 0003
- Keep operation logic separate for readability (operations module?)
- Use type info for runtime assertions/validation
