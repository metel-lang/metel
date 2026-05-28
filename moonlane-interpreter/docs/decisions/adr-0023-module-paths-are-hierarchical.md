# ADR-0023: Module Paths Are Hierarchical (Absolute from Root)

**Status:** Accepted  
**Date:** 2026-05-28

---

## Context

The module loader assigns a `module_path: Vec<String>` to each loaded module. This path is used as the key in `GlobalExports`, `ResolvedNames.scopes`, `ResolvedNames.pub_surface`, and in scope binding lookups.

---

## Decision

Module paths are **absolute from the project root**, constructed by concatenating the parent module's path with the new module's name segments:

```
root.mln          →  module_path = []
root imports parser → module_path = ["parser"]
parser imports lexer → module_path = ["parser", "lexer"]
```

This is enforced in `module_loader.rs`:
```rust
let mut child_path = module_path.clone();  // parent's path
child_path.extend(mod_segs);               // append new name
self.load_module(child, child_path)?;
```

The `name_resolver::absolute_base` function produces the canonical key for a module referenced in an import path. For `PathRoot::Name(n)` from a module at `current`:

```rust
PathRoot::Name(n) => {
    let mut path = current.to_vec();
    path.push(n.clone());
    path
}
```

This ensures `import lexer::*` from `parser.mln` (path `["parser"]`) resolves to `["parser", "lexer"]`, matching lexer's actual `module_path`.

---

## Invariant

The `GlobalExports` key for a module **must equal** that module's `module_path` from the loader. Any code that computes a module identifier for lookup in `GlobalExports` or `ResolvedNames` must use the full hierarchical path, not just the last segment.

---

## Why Not Flat Paths

An earlier implementation used `PathRoot::Name(n) => vec![n.clone()]` (flat, single-segment). This worked for depth-1 modules (direct imports from root) but broke for deeper transitive imports: `parser/lexer` would get `module_path = ["parser", "lexer"]` but `absolute_base` would produce `["lexer"]`, causing `GlobalExports` and scope lookups to miss the module entirely.

The fix was discovered when the `transitive_dependency_via_graph_pipeline` integration test failed. The flat-path behavior was retained in unit tests that used manually-constructed graphs with non-hierarchical paths — those tests were updated to use hierarchical paths to match the loader's actual behavior.
