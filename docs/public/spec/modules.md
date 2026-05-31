# Modules

## Files and Modules

> **Availability:** Since v0.5.0.

Every `.mln` source file is a module. There is no `mod` declaration — the module graph is built entirely from `import` declarations.

The root file passed to the toolchain is the root module:

```bash
metel src/main.mln
```

In that example, `root::` refers to `src/main.mln`.

## File-to-Module Mapping

`::` maps directly to `/` in the filesystem. There is no special directory module file.

| Import | File resolved |
|---|---|
| `import parser::Ast;` | `parser.mln` |
| `import parser::ast::Ast;` | `parser/ast.mln` |
| `import root::a::b::c::T;` | `a/b/c.mln` relative to the root file |

A directory module with a public facade is expressed by placing `name.mln` alongside the `name/` directory. The two coexist without ambiguity — they are different paths:

```
src/
  main.mln            ← import parser::Ast; import parser::lexer::Token;
  parser.mln          ← export ast::Ast; export lexer::Token;
  parser/
    ast.mln           ← pub struct Ast { ... }
    lexer.mln         ← pub struct Token { ... }
```

`parser.mln` is the facade. Files in `parser/` form the namespace. There is no `name/mod.mln` convention.

## File Header Ordering

At file scope, `import` and `export` declarations must precede all other declarations:

```
(import | export)* declaration*
```

`import` and `export` are not valid inside blocks.

## Paths

Paths use `::` separators.

Path roots are:

| Root | Meaning |
|---|---|
| `root::` | The selected root module for the current program |
| `std::` | The bundled standard library root; `std::core` is always available |
| `self::` | The current module |
| `super::` | The parent module; invalid from the root module |
| imported module handle | A module brought into scope by `import path::module;` |

Fully-qualified paths are valid anywhere a name is expected:

```metel
// src/main.mln
import root::parser::Token;

fun main() -> Int {
    let token: root::parser::Token = root::parser::Token { value: 42 };
    return token.value;
}

// src/parser.mln
pub struct Token {
    value: Int,
}
```

## Imports

`import` loads the referenced module file and declares which names from it are in scope for the current module:

```metel
// src/main.mln
import parser::{Ast, Token};
import root::lexer::Token as Tok;
import parser::*;
import std::core;

fun main() -> Int {
    let ast = Ast { token: Token { value: 1 } };
    let tok: Tok = core::dbg(Token { value: 2 });
    return ast.token.value + tok.value + parse(ast.token);
}

// src/parser.mln
export ast::Ast;
export ast::parse;
export lexer::Token;

// src/parser/ast.mln
import super::lexer::Token;
pub struct Ast { token: Token }
pub fun parse(token: Token) -> Int { token.value }

// src/lexer.mln
pub struct Token { value: Int }
```

Import forms:

| Form | Effect |
|---|---|
| `import path::Name;` | imports `Name` |
| `import path::Name as Alias;` | imports `Name` under `Alias` |
| `import path::{A, B, C};` | imports multiple names from one path |
| `import path::{A as X, B};` | imports with per-item aliases |
| `import path::*;` | imports all public names from the module |
| `import path::module;` | imports `module` as a module handle; `module::item` is then valid |

## Re-exports

`export` re-exports names from submodules into the current module's public API:

```metel
// parser.mln — facade module for the parser namespace
export ast::Ast;
export lexer::{Token, Span};
export ast::ParseError as Error;

fun main() -> Int {
    return 0;
}
```

`export` and `import` share the same path and tree syntax. Re-exported names are indistinguishable from names defined directly in the re-exporting module.

`pub` and `export` serve different roles:

| Keyword | Purpose |
|---|---|
| `pub` | Marks a declaration in this file as externally accessible |
| `export path::Name;` | Re-exports a name from a submodule into this module's public API |

`export` declarations are processed after the module graph is fully loaded; they do not affect which files are loaded.

## std::core Auto-Import

> **Availability:** Since v0.6.1.

Every module automatically has `std::core` glob-imported at the lowest priority tier. This means `Perhaps`, `Result`, `Display`, `Iterable`, `From`, and all built-in functions are available in every module without any explicit import statement.

```metel
// No import needed — Perhaps and Result are always in scope
fun maybe_parse(s: String) -> Perhaps<Int> {
    if (s == "1") { return Perhaps::Some { value: 1 }; }
    return None;
}

fun main() -> Int {
    match maybe_parse("1") {
        Perhaps::Some { value } => value,
        Perhaps::None => 0,
    }
}
```

You can still write `import std::core::Perhaps;` or `import std::core::*;` explicitly — the result is the same. If a local declaration or explicit import shadows a `std::core` name, the local binding wins silently.

`std::core` is a **virtual module** — it has no physical `.mln` file and cannot be listed or enumerated. Its contents are seeded by the runtime.

## Import Conflicts

Two explicit imports that bind the same local name in the same module are a compile-time error at the second import.

Glob imports use a priority tier system:

| Tier | Source | Priority |
|------|--------|----------|
| `Std` | Auto-inserted by the runtime (e.g. `std::core`) | Lowest |
| `User` | Explicit `import path::*` in source | Higher |

Conflict rules:
- Local declarations beat all glob imports.
- Explicit imports beat all glob imports.
- A `User` glob silently wins over a `Std` glob for the same name (no error).
- Two `User` globs exporting the same name are a conflict error (T0011) only if that name is actually referenced.

## Visibility

Declarations are module-private by default. A declaration is accessible from outside its module only if it is annotated with `pub`.

```metel
pub struct Token { kind: Int, span: Int }
struct InternalState { count: Int }

pub fun parse(tokens: Token[]) -> Int { return array_len(tokens); }
fun helper(token: Token) -> Bool { return token.kind == 0; }

fun main() -> Int {
    let token = Token { kind: 0, span: 1 };
    let state = InternalState { count: 2 };
    if (helper(token)) { return parse([token]) + state.count; }
    return 0;
}
```

`pub` is valid on `struct`, `enum`, `fun`, `aspect`, and top-level `let`/`mut` bindings.

Fields of a `pub struct` are public. Fields of a private struct are private because the struct itself is not externally nameable.

Within a module, all names defined in that module are accessible without qualification, including private names.

Modules do not have their own visibility annotation. Module-level access control is handled entirely by `pub` on individual items.

## Circular Imports

Circular imports are a compile error. The error message includes the full import chain.

## Module Graph Loading

The module graph is built from `import` declarations:

1. The root file is parsed.
2. All `import` declarations are collected; each is resolved to a file path via the `::` → `/` mapping.
3. Each referenced file is loaded recursively; cycles are detected and rejected.
4. Only files reachable via at least one `import` declaration are loaded.

`export` declarations do not affect which files are loaded.

## Single-File Compatibility

A `.mln` file with no `import` or `export` declarations is a complete program. Existing single-file programs remain valid without modification.
