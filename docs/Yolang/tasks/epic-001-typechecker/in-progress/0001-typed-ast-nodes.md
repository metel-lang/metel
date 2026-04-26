# Task 0001: Design and Implement Typed AST Nodes

**Status:** open  
**Epic:** epic-001-typechecker  
**Component:** parser, interpreter  
**Spec Link:** spec/Language Spec.md#Type-System (or create)  
**Blocked By:** none

## What

Create a mirrored `TypedExpr` enum that parallels the untyped `Expr` enum but includes type information on every variant. This establishes the foundation for type inference and checking.

**Current flow:**
```
Parser → Expr (untyped) → Evaluator
```

**After this task:**
```
Parser → Expr (untyped) → Type Checker/Inference → TypedExpr → Evaluator
```

The type checker will convert `Expr` to `TypedExpr`, annotating every subexpression with its inferred/resolved type.

## Design: Mirrored TypedExpr

Mirror **every** `Expr` variant from `src/ast/mod.rs`, adding a `Type` field to each. Current `Expr` has these 20 variants:

```
Literal, Ident, Path, Tuple, Array, BinOp, UnaryOp, Assign, Call, MethodCall,
FieldAccess, TupleAccess, Index, Cast, Match, If, Loop, Closure, StructLiteral, PropagateError
```

**Example structure** (from `tree-walk-interpreter/src/typed_ast/mod.rs`):

```rust
use crate::ast::{Literal, BinOp, UnaryOp, AssignTarget, AssignOp, Span, Param, TypeExpr};
use crate::types::Type;

#[derive(Debug, Clone)]
pub enum TypedExpr {
    Literal(Literal, Type, Span),
    Ident(String, Type, Span),
    Path(Vec<String>, Type, Span),
    Tuple(Vec<TypedExpr>, Type, Span),
    Array(Vec<TypedExpr>, Type, Span),
    BinOp(Box<TypedExpr>, BinOp, Box<TypedExpr>, Type, Span),
    UnaryOp(UnaryOp, Box<TypedExpr>, Type, Span),
    Assign { target: AssignTarget, op: AssignOp, value: Box<TypedExpr>, ty: Type, span: Span },
    Call { callee: Box<TypedExpr>, args: Vec<TypedExpr>, ty: Type, span: Span },
    MethodCall { receiver: Box<TypedExpr>, method: String, args: Vec<TypedExpr>, ty: Type, span: Span },
    FieldAccess { object: Box<TypedExpr>, field: String, ty: Type, span: Span },
    TupleAccess { object: Box<TypedExpr>, index: usize, ty: Type, span: Span },
    Index { object: Box<TypedExpr>, index: Box<TypedExpr>, ty: Type, span: Span },
    Cast { expr: Box<TypedExpr>, target_type: TypeExpr, ty: Type, span: Span },
    Match(TypedMatchExpr),
    If { condition: Box<TypedExpr>, then_branch: TypedBlock, else_branch: TypedBlock, ty: Type, span: Span },
    Loop { body: TypedBlock, ty: Type, span: Span },
    Closure { params: Vec<Param>, return_type: Option<TypeExpr>, body: TypedBlock, ty: Type, span: Span },
    StructLiteral { path: Vec<String>, fields: Vec<(String, TypedExpr)>, ty: Type, span: Span },
    PropagateError { expr: Box<TypedExpr>, ty: Type, span: Span },
}
```

## Key Mirroring Rules

1. **Child expressions:** `Expr` → `TypedExpr`, `Vec<Expr>` → `Vec<TypedExpr>`
2. **Type field naming:**
   - Tuple variants: `Type` as positional arg (e.g., `Literal(Literal, Type, Span)`)
   - Struct variants: named field `ty` (e.g., `Assign { ..., ty: Type, ... }`)
3. **Nested structures:** Also mirror `Block` → `TypedBlock` and `MatchExpr` → `TypedMatchExpr`

## Supporting Types to Create

Mirror these structs where they contain `Expr`:

```rust
#[derive(Debug, Clone)]
pub struct TypedBlock {
    pub stmts: Vec<TypedDecl>,
    pub tail:  Option<Box<TypedExpr>>,
    pub span:  Span,
}

#[derive(Debug, Clone)]
pub struct TypedMatchExpr {
    pub scrutinee: Box<TypedExpr>,
    pub arms:      Vec<TypedMatchArm>,
    pub span:      Span,
}

#[derive(Debug, Clone)]
pub struct TypedMatchArm {
    pub pattern: Pattern,  // Patterns don't contain exprs, reuse as-is
    pub guard:   Option<TypedExpr>,
    pub body:    TypedExpr,
    pub span:    Span,
}
```

Also mirror `TypedDecl` (similar structure to `ast::Decl` but with TypedExpr in let/mut values).

## Implementation Notes

1. **Location:** Refactor `/tree-walk-interpreter/src/typed_ast/mod.rs`
2. **Parser unchanged:** Parser still outputs `Expr`. Type checker implements `Expr` → `TypedExpr` conversion
3. **Recursive mirroring:** Any `Box<Expr>` or `Vec<Expr>` becomes `Box<TypedExpr>` or `Vec<TypedExpr>`
4. **Reuse untyped AST:** Import `Literal`, `BinOp`, `Span`, `Pattern`, etc. from `ast` module

## Acceptance Criteria

- [ ] `TypedExpr` enum defined with all 20 variants matching `ast::Expr`
- [ ] `TypedBlock` struct defined (mirrors `Block`)
- [ ] `TypedMatchExpr` and `TypedMatchArm` structs defined
- [ ] `TypedDecl` enum defined (mirrors `Decl` with typed expressions)
- [ ] All Expr → TypedExpr field conversions are consistent
- [ ] Code compiles without errors
- [ ] Parser still outputs `Expr` (no parser changes yet)
- [ ] Evaluator still works with `Expr` (no evaluator changes yet)
- [ ] All existing tests pass
- [ ] Create minimal unit tests: construct a few `TypedExpr` values manually to verify structure

## Notes

- This is **structural** work only — define the enum, don't implement logic yet
- Type fields will initially be `Type::Unknown` (or inferred based on literals); task 0002 does the inference
- Consider implementing `impl TypedExpr { pub fn ty(&self) -> &Type }` for convenient type access
- Do NOT implement conversion logic (`Expr` → `TypedExpr`) yet — that's task 0002
- Keep the old `ast::Expr` unchanged; this is purely additive
