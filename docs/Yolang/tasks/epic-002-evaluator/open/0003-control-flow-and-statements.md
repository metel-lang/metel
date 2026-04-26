# Task 0003: Control Flow and Statement Execution

**Status:** open  
**Epic:** epic-002-evaluator  
**Component:** evaluator  
**Spec Link:** spec/Language Spec.md#Control-Flow  
**Blocked By:** 0002

## What

Implement **control flow** and **statements** — conditionals, loops, blocks, and their evaluation.

Statement types:
- **If/Else** — conditional execution
- **Match** — pattern matching with guards
- **While/For** — loops with break/continue
- **Return/Break/Continue** — flow control
- **Blocks** — statement sequences with tail expressions

## Design

**Block Evaluation:**
- Execute statements in sequence
- Tail expression (if present) is the block's value
- Each statement may introduce bindings

**If Expression:**
- Evaluate condition (must be bool)
- Execute then or else branch
- Both branches must have compatible types (already type-checked)

**Match Expression:**
- Evaluate scrutinee
- Try patterns in order
- Evaluate guard (if present, must be bool)
- Execute first matching arm
- Return arm value

**Loops:**
- **While:** condition evaluated each iteration
- **For:** C-style loop with init, condition, step
- **ForIn:** iterate over array elements
- **Loop:** infinite loop (must break/return to exit)

**Flow Control:**
- **Return:** exit function with value
- **Break:** exit loop, optionally with value
- **Continue:** skip to next iteration

## Acceptance Criteria

- [ ] If/else expressions evaluate correctly
- [ ] All branches type-check and evaluate
- [ ] Match expressions work with all pattern types
- [ ] Guards evaluate correctly
- [ ] While loops execute correctly
- [ ] For loops (C-style and for-in) work
- [ ] Infinite loops work with break/continue
- [ ] Return exits function with correct value
- [ ] Break/continue control loop flow
- [ ] Blocks execute statements and return tail value
- [ ] Nested control flow works
- [ ] Type errors caught at runtime
- [ ] Tests cover all control flow patterns
- [ ] All previous tests still pass

## Notes

- Pattern matching uses the Pattern type (already defined)
- Break/continue need special handling (exception-like mechanism)
- Tail expression is the block's value (if present)
- Type checker ensures type safety; evaluator assumes valid types
