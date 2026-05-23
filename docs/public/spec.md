---
id: doc-2
title: "Gust Language Specification"
type: spec
version: v0.2
created_date: '2026-05-16'
---

> **Status:** Active. This document is the single source of truth for the Gust language.
> Features not described here are not part of the language.

Source files use the `.gust` extension.

---

## Overview

Gust is a strongly typed, compiled language with a Rust-inspired type system. Its core design principles are:

- **Strong static typing** with full Hindley-Milner type inference
- **No classes** — data and behaviour are defined separately via structs, enums, and traits
- **Algebraic data types** — enums with data-carrying variants and exhaustive pattern matching
- **Explicit nullability** — absence of a value is represented by `Perhaps<T>` / `nope`, never by null
- **Explicit error handling** — errors are values, represented as `Result<T, E>`
- **Memory managed by the runtime** — reference counting, no ownership semantics in the language

---

## Contents

| File | Contents |
|---|---|
| [Lexical Structure](spec/lexical.md) | Comments, identifiers, keywords, literals, operators |
| [Type System](spec/types.md) | Primitive types, inference, tuples, arrays, casting, generics, Never, `Perhaps<T>`, `Result<T,E>` |
| [Declarations](spec/declarations.md) | Variables, structs, enums, traits |
| [Functions](spec/functions.md) | Functions, closures, the `?` operator |
| [Expressions](spec/expressions.md) | Pattern matching, control flow |
| [Runtime](spec/runtime.md) | Panics, built-in functions |
| [Grammar](spec/grammar.md) | Formal grammar |

See [Changelog](../changelog.md) for version history.
