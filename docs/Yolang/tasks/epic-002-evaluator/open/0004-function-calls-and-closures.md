# Task 0004: Function Calls and Closures

**Status:** open  
**Epic:** epic-002-evaluator  
**Component:** evaluator  
**Spec Link:** spec/Language Spec.md#Functions  
**Blocked By:** 0003

## What

Implement **function definition, calling, and closures**. Functions are first-class values that can be defined, passed, and called.

Features:
- **User-defined functions** — declared with `fn name<params> -> type { body }`
- **Function values** — can be assigned, passed, returned
- **Function calls** — apply arguments, execute body, return result
- **Closures** — capture variables from enclosing scope
- **Built-in functions** — print, len, etc.

## Design

**Function Representation:**
- Store function definition: parameters, return type, body
- Capture environment for closures
- Callable from expressions and statements

**Function Calls:**
1. Evaluate arguments
2. Bind parameters to argument values
3. Create new scope (with captured environment)
4. Execute body
5. Return result (or unit if no return)

**Built-in Functions:**
- `print(val)` — output value
- `len(arr)` — array/string length
- `push(arr, val)` — append to array
- Array indexing, type conversions, etc.

**Closures:**
- Capture variables from enclosing scope
- Create new scope with captured bindings
- Can be called later in different context

## Acceptance Criteria

- [ ] Function definitions evaluated correctly
- [ ] Function parameters bound to arguments
- [ ] Function bodies execute with correct scope
- [ ] Return values propagated correctly
- [ ] Function values can be assigned and passed
- [ ] Closures capture variables correctly
- [ ] Built-in functions work (print, len, etc.)
- [ ] Recursive functions work
- [ ] Higher-order functions work (functions taking/returning functions)
- [ ] Error handling for wrong argument count/types
- [ ] Type information available during execution
- [ ] Tests cover functions, closures, recursion, higher-order
- [ ] All previous tests still pass

## Notes

- Closure capture: copy values or reference them?
- Stack-based evaluation may be simpler than env-passing
- Built-in functions are special-cased initially
- Function values need to carry their closure environment
- Recursive calls need cycle detection or stack limit
