# Task 0004: Implement Basic Type System (Int, Bool, String, List)

**Status:** open  
**Epic:** epic-001-typechecker  
**Component:** typechecker, interpreter  
**Spec Link:** spec/Language Spec.md#Basic-Types  
**Blocked By:** 0001

## What

Define and implement the basic type system:

- **Int** — signed or unsigned integers (32 or 64-bit)
- **Bool** — true/false values
- **String** — UTF-8 text
- **List** — homogeneous collections of one type

Include type coercion rules and operation compatibility (e.g., can you add an int to a bool?).

## Design Decisions Needed

1. Should string concatenation coerce numbers to strings, or require explicit conversion?
2. Are lists `List<T>` (parametric) or just `List`?
3. How does type checking handle empty lists `[]`?

## Acceptance Criteria

- [ ] Type system defines Int, Bool, String, List types
- [ ] Coercion rules are documented in spec
- [ ] Binary operations validate operand types (e.g., `+` requires numeric types)
- [ ] List operations type-check correctly
- [ ] Type inference handles type annotations like `x: int = 42`
- [ ] Evaluator enforces type constraints (e.g., can't add string to int without coercion)
- [ ] Tests cover basic operations for each type
- [ ] No regressions

## Notes

- Lists should probably be parametric (`List<T>`) even if we don't support full generics yet
- Consider adding type conversion functions early (int_to_string, etc.)
- This task works in parallel with 0001-0003; can start once 0001 is underway
