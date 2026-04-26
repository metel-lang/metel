# Epic 002: Evaluator

**Status:** open  
**Started:** 2026-04-26  
**Depends On:** Epic 001 (Typechecker and Typed AST)

## Overview

Implement the **evaluator** — the runtime engine that executes fully typed programs. Transforms the TypedAST into running code with full type information available.

This epic completes the core interpreter pipeline: Parse → Type Check → **Evaluate** → Output

## Goals

1. **Expression Evaluation** — Execute all 20 TypedExpr variants correctly
2. **Block & Statement Execution** — Handle scoping, control flow, loops
3. **Function Calls** — User-defined and built-in functions with proper calling conventions
4. **Type-Safe Operations** — Use type info for runtime validation and optimization
5. **Runtime Values** — Represent values with their types (Value enum)
6. **Error Propagation** — Handle errors with type information
7. **Built-in Functions** — Standard library operations (print, array ops, etc.)

## Why This Epic?

The evaluator is the final piece of the core interpreter:
- **Validates the design** — TypedAST and type system are proven correct
- **Enables full testing** — Test parsing, type checking, AND execution together
- **Provides user value** — Can actually run Yolang programs
- **Foundation for optimization** — Type info enables runtime optimizations
- **Sets up generics** — Monomorphization (Epic 003) feeds specialized code to the evaluator

## Architecture

```
TypedProgram (from type checker)
    ↓
Environment Setup (globals, functions, types)
    ↓
Function Execution (with TypedExpr bodies)
    ↓
Value Computation (Int, Bool, String, Array, Tuple, etc.)
    ↓
Output / Results
```

## Dependencies

- **Epic 001:** Type checker produces TypedProgram
- **Types module:** Type enum for runtime type information
- **TypedAST module:** TypedExpr, TypedBlock, TypedDecl, etc. (✅ already defined)

## Out of Scope (for Epic 002)

- Generics execution (Epic 003 - monomorphization handles this)
- Advanced memory management / GC
- Debugger / REPL (future)
- Optimization passes
- Async/concurrency

## Success Criteria

When this epic is done:

- [ ] All TypedExpr variants evaluate correctly
- [ ] Variable binding and scoping work correctly
- [ ] Function definitions and calls work
- [ ] All basic types work (Int, Float, Bool, String, Array, Unit, Tuple)
- [ ] Control flow works (if/else, match, loops, while, for)
- [ ] Type-safe operations (using type info at runtime)
- [ ] Built-in functions available (print, len, indexing, etc.)
- [ ] Error messages report type-related issues clearly
- [ ] All Epic 001 type checking still works
- [ ] Comprehensive test suite (parse + type + eval)
- [ ] Can run non-generic Yolang programs end-to-end

## Related Issues/Tasks

- REPL implementation (after evaluator)
- Performance optimization (post-MVP)
- Standard library (parallel effort)

## Notes

- Evaluation is straightforward once types are resolved
- Use Rust's type system to represent Values
- Keep evaluation separate from type checking for clarity
- TypedExpr carries all information needed; no extra passes
- Each expression evaluation returns a Value with its type
