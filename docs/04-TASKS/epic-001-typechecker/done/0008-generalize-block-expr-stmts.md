# Task 0008: Generalize Block-Expression-Statements to Cover `match` and `loop`

**Status:** done
**Epic:** epic-001-typechecker
**Component:** parser
**Spec Link:** `../01-SPEC/LANGUAGE-SPEC.md` §4 Statements, §5 Expressions
**Blocked By:** 0007 (done)
**Decisions:** [ADR-0005](../../05-DECISIONS/closed/ADR-0005-if-expression-vs-statement.md)

## What

Task 0007 unified `if` into a single expression form and introduced the
`if_stmt_item = { if_expr ~ !"}" }` grammar rule to allow `if` in statement
position without a trailing semicolon. However, `match` and `loop` have the
identical split:

| Grammar rule | AST node | Problem |
|---|---|---|
| `match_stmt` (in `stmt`) | `Stmt::Match` | match can never be a block tail |
| `loop_stmt` (in `stmt`) | `Stmt::Loop` | loop can never be a block tail |

These are in the exact same position as the old `if_stmt` before task 0007:
because `decl*` is greedy, a `match` or `loop` at the end of a block is always
consumed as a statement and can never reach the tail `expr?` slot.

The fix is to generalize `if_stmt_item` into a single `block_expr_stmt` rule
that covers all three block-ending constructs:

```pest
block_item      = { block_expr_stmt | decl }
block_expr_stmt = { (if_expr | match_expr | loop_expr) ~ !"}" }
```

`match_stmt` and `loop_stmt` are removed from the grammar and AST entirely.
In statement position, `match` and `loop` become `Stmt::Expr(Expr::Match(…))`
and `Stmt::Expr(Expr::Loop(…))` — the same pattern as `if`.

## Acceptance Criteria

- [x] `grammar.pest`: `match_stmt` and `loop_stmt` removed from `stmt`; `if_stmt_item`
  renamed to `block_expr_stmt` and expanded to `(if_expr | match_expr | loop_expr) ~ !"}"`;
  `block_item` updated accordingly
- [x] `ast/mod.rs`: `Stmt::Match` and `Stmt::Loop` removed; `LoopStmt` struct removed
- [x] `typed_ast/mod.rs`: `TypedStmt::Match` and `TypedStmt::Loop` removed;
  `TypedLoopStmt` struct removed
- [x] `parser/mod.rs`: `Rule::match_stmt` and `Rule::loop_stmt` removed from `parse_stmt`;
  `parse_loop_stmt` removed; `parse_block` updated to handle `Rule::block_expr_stmt`
  (dispatching to `parse_match_expr` or `parse_loop_expr` or `parse_if_expr`)
- [x] Parsing tests: `match` and `loop` used directly as block tail expressions parse correctly
  (`tests/parsing/sources/11_block_expr_stmts.yolo`)
- [x] All 126 tests pass (125 existing + 1 new)
- [x] No regressions in statement-position `match` and `loop` (no semicolon required)
