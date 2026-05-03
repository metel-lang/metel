# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Yolang is a Rust-inspired programming language with a tree-walk interpreter written in Rust. The project implements a statically typed, expression-oriented language with features like type inference, pattern matching, and generics.

## Common Development Commands

### Building and Running
```bash
# Build the interpreter
cd tree-walk-interpreter
cargo build --release

# Run a Yolang program
cargo run -- path/to/program.yolo

# Run in debug mode with output
cargo run -- --debug path/to/program.yolo
```

### Testing
```bash
# Run all tests
cargo test

# Run type inference tests specifically
cargo test --test typeinference_tests

# Run specific test phase with output
cargo test --test typeinference_tests phase_2 -- --nocapture

# Run specific test by name
cargo test test_name -- --nocapture
```

### Development Workflow
```bash
# Lint and format (if available)
cargo clippy
cargo fmt

# Build and test together
cargo build && cargo test
```

## Project Architecture

The interpreter follows a multi-stage pipeline:

```
.yolo source → Parser (pest) → AST Builder → Type Checker → Evaluator
```

### Key Components

- **Parser** (`src/parser/`): Uses pest PEG grammar (`src/grammar.pest`) to generate CST, then builds untyped AST
- **AST** (`src/ast/`): Untyped abstract syntax tree definitions
- **Type System** (`src/types/`, `src/typeinference/`, `src/typechecker/`): Type representation, inference engine, and validation
- **Typed AST** (`src/typed_ast/`): AST nodes that carry type information
- **Evaluator** (`src/evaluator/`): Tree-walking interpreter for executing typed programs
- **Error Handling** (`src/error/`): Comprehensive error types with source location tracking

### Working Directory

All Rust development happens in the `tree-walk-interpreter/` subdirectory. Always `cd` there first:

```bash
cd tree-walk-interpreter
# Then run cargo commands
```

## Documentation Structure

The project follows a strict documentation hierarchy in `docs/`:

- **00-PROCESS/**: Development workflow and task management conventions
- **01-SPEC/**: Language specification (authoritative) and feature backlog
- **02-ARCHITECTURE/**: Design decisions and architectural documentation
- **03-COMPONENTS/**: Implementation guides for specific components
- **04-PLANNING/**: Strategic roadmaps and medium-term plans
- **05-TASKS/**: Epic-based task organization with status tracking

### Key Files

- `docs/01-SPEC/LANGUAGE-SPEC.md`: Complete language specification (source of truth)
- `docs/01-SPEC/BACKLOG.md`: Features not yet implemented
- `docs/02-ARCHITECTURE/INTERPRETER-DESIGN.md`: Overall system design
- `docs/03-COMPONENTS/typeinference/`: Type inference implementation guide
- `docs/00-PROCESS/TASK-CONVENTION.md`: Task management workflow

## Development Principles

### Spec-First Development
- The language specification in `docs/01-SPEC/LANGUAGE-SPEC.md` is authoritative
- Implementation reveals spec ambiguities - resolve in spec first, then implement
- Never implement behavior not specified in the spec
- Tag spec sections when interpreter-validated: `> ✓ Interpreter-validated (v0.1)`

### Task Management
- Use epic-based organization under `docs/05-TASKS/`
- Tasks have clear status: `open`, `in-progress`, `done`, `blocked`
- Every task links to relevant spec section or backlog item
- Move task files between status folders to reflect current state

### Three-Stage Validation
1. **Designed**: Written in spec, not yet implemented
2. **Interpreter-validated**: Implemented and tested in tree-walk interpreter
3. **Compiler-validated**: Future compiler implementation (not current focus)

## Type Inference Implementation

The type inference system is built incrementally with comprehensive test coverage:

### Test-Driven Development
```bash
# Check current phase status
cargo test --test typeinference_tests phase_1

# Work on specific phase
cargo test --test typeinference_tests phase_2 -- --nocapture
```

### Key Files
- `src/typeinference/mod.rs`: Core inference engine
- `src/types/mod.rs`: Type representation
- `tests/typeinference_tests.rs`: Phase-based test suite
- `docs/03-COMPONENTS/typeinference/ROADMAP.md`: Implementation roadmap

## Current Development Focus

### Epic 001: Typechecker (Foundation)
- Typed AST representation
- Type inference engine with let-polymorphism
- Type checker validation pass
- Basic type system (int, float, bool, string, array, unit, tuple)

### Epic 002: Evaluator (Runtime)
- Expression evaluation for all 20+ expression types
- Statement execution and control flow
- Function calls and closures
- Built-in function support

### Epic 003: Generics (Advanced Features)
- Type variables and constraints
- Generic instantiation
- Monomorphization at compile-time
- Recursive and nested generics

## Error Handling

Uses miette for rich error reporting with source context. Error types are defined in `src/error/mod.rs` with proper source location tracking.

## Dependencies

- **pest**: PEG parser generator (grammar in `src/grammar.pest`)
- **miette**: Rich error reporting with source context
- **thiserror**: Error derive macros
- **clap**: CLI argument parsing

## Testing Strategy

- Phase-based test development for type inference
- Integration tests in `tests/test_programs/` with `.yolo` files
- Unit tests within component modules
- Test program examples covering language features 01-10