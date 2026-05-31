---
id: spec
title: "Metel Language Specification"
type: spec
version: v0.6.4
created_date: '2026-05-16'
---

> **Status:** Active. This document is the single source of truth for the Metel language.
> Features not described here are not part of the language.

Source files use the \`.mln\` extension.

---

## Overview

Metel is a statically typed, expression-oriented language with a Rust-inspired syntax.
This specification describes the language accepted by the current interpreter.

The language's core design principles are:

- **Strong static typing** with full Hindley-Milner type inference
- **No classes** — data and behaviour are defined separately via structs, enums, and aspects
- **Algebraic data types** — enums with data-carrying variants and exhaustive pattern matching
- **Explicit nullability** — absence of a value is represented by `Perhaps<T>`, never by null
- **Explicit error handling** — errors are values, represented as `Result<T, E>`
- **Safe memory by default** — reference counting, no ownership semantics required

---

## Contents

| File | Contents |
|---|---|
| [Lexical Structure](spec/lexical.md) | Comments, identifiers, keywords, literals, operators |
| [Modules](spec/modules.md) | Files, modules, imports, path roots, visibility, re-exports |
| [Type System](spec/types.md) | Primitive types, inference, tuples, arrays, casting, generics, Never, `Perhaps<T>`, `Result<T,E>` |
| [Declarations](spec/declarations.md) | Variables, structs, enums, aspects |
| [Functions](spec/functions.md) | Functions, closures, the `?` operator |
| [Expressions](spec/expressions.md) | Pattern matching, control flow |
| [Runtime](spec/runtime.md) | Panics, built-in functions |
| [Grammar](spec/grammar.md) | Formal grammar |

See [Changelog](changelog.md) for version history.
