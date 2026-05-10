# Agent Guide: Working on Yoloscript

This document explains how to work on the Yoloscript project as an AI agent (Claude, etc.).

## Quick Summary

1. **Always check the spec first** - `docs/01-SPEC/LANGUAGE-SPEC.md` is the source of truth
2. **Follow task conventions** - See `TASK-CONVENTION.md` for how to organize work
3. **Use the documentation structure** - Everything is in `docs/Yoloscript/`
4. **Write tests first** - All implementation has tests
5. **Update references** - When moving/creating files, update cross-references
6. **Link to documentation** - Changes should link to relevant spec sections

---

## Project Navigation

### Core Locations

```
tree-walk-interpreter/          # Rust implementation
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library exports (required for tests)
│   ├── parser/mod.rs           # Parsing
│   ├── ast/mod.rs              # AST definition
│   ├── typeinference/mod.rs    # Type inference
│   ├── typechecker/mod.rs      # Type checking
│   ├── evaluator/mod.rs        # Runtime evaluation
│   ├── error/mod.rs            # Error types
│   └── types/mod.rs            # Type system
├── tests/
│   └── typeinference_tests.rs  # Type inference test suite
└── Cargo.toml

docs/                                # All documentation
├── 00-PROCESS/                 # How to work (this folder)
├── 01-SPEC/                    # What is Yoloscript
├── 02-ARCHITECTURE/            # Architecture & design
├── 03-COMPONENTS/              # Implementation guides
├── 04-PLANNING/                # Roadmaps
├── 05-TASKS/                   # Current work
└── 06-DECISIONS/               # ADRs — why non-obvious choices were made
```

---

## Before You Start Work

### 1. Understand the Documentation Structure

Read this in order:
1. `README.md` (root) - Project overview
2. `docs/00-PROCESS/PROCESS.md` - Documentation philosophy
3. `docs/01-SPEC/LANGUAGE-SPEC.md` - Language definition (reference as needed)
4. Relevant component guide (e.g., `docs/03-COMPONENTS/typeinference/`)

### 2. Check Current Tasks

Open `docs/05-TASKS/` to see:
- Open issues (in `open/` folders)
- In-progress work (in `in-progress/` folders)
- Completed work (in `done/` folders)
- Blocked work (in `blocked/` folders)

### 3. Find Your Task

Get the exact task ID and path. Example:
```
docs/05-TASKS/epic-001-typechecker/open/0002-type-inference.md
```

---

## Working on Implementation

### Phase 1: Read the Task

Task files have this structure:

```markdown
# Task NNNN: Brief Title

**Status:**      open | in-progress | done | blocked
**Epic:**        epic-001-typechecker
**Component:**   interpreter | parser | typechecker | evaluator
**Spec Link:**   docs/01-SPEC/LANGUAGE-SPEC.md#Section-Name
**Blocked By:**  task IDs or "none"

## What
What needs doing and why.

## Acceptance Criteria
- [ ] Testable outcome 1
- [ ] Testable outcome 2
- [ ] No regressions

## Notes
Progress and discoveries
```

**Always check Spec Link** - That's what you're implementing.

### Phase 2: Update Task Status

When starting work:
1. Change `**Status:**` from `open` to `in-progress`
2. Move the file from `open/` to `in-progress/`
3. Add notes about what you're doing

### Phase 3: Write Tests First

Before implementing:
1. Open the test file relevant to your component
   - Example: `tests/typeinference_tests.rs` for type inference
2. Find the test stubs (marked with `todo!()`)
3. Write the test cases based on the task's acceptance criteria
4. Leave tests commented out or stubbed for now

### Phase 4: Implement

Follow these rules:

#### a) Implement What's Specified
- **Only implement what's in the spec** (`docs/01-SPEC/LANGUAGE-SPEC.md`)
- If something isn't speced, check `docs/01-SPEC/BACKLOG.md`
- If it's in the backlog as "open", ask for clarification
- If it's "deferred", don't implement it

#### b) Follow Rust Conventions
- Use `cargo fmt` before committing
- Use `cargo clippy` to check for issues
- Write clear variable and function names
- Add doc comments for public APIs

#### c) Link to Documentation
Every implementation should have a clear path back to the spec:
```rust
// Implements §3.6 of Language Spec: Generics
pub struct TypeScheme { ... }
```

#### d) Keep It Simple
- YOLO: You Only Live Once (make decisions, move forward)
- Don't over-engineer for future features
- Make it clear and correct first, fast later

### Phase 5: Write Real Tests

Now replace `todo!()` with real test code:

```rust
#[test]
fn test_unify_var_with_concrete() {
    let mut gen = TypeVarGenerator::new();
    let var = gen.fresh();
    
    let ty1 = InferType::Var(var);
    let ty2 = InferType::Concrete(Type::Int);
    
    let subst = unify(&ty1, &ty2, &Substitution::new()).unwrap();
    assert_eq!(subst.apply(&ty1), ty2);
}
```

**Test requirements:**
- At least one test per acceptance criterion
- Tests should verify the behavior described in the spec
- Tests should be repeatable and deterministic
- Test names should describe what they test

### Phase 6: Run Tests

```bash
cd tree-walk-interpreter

# All tests
cargo test

# Specific test file
cargo test --test typeinference_tests

# Specific test
cargo test --test typeinference_tests phase_2::test_infer_type_concrete

# With output
cargo test --test typeinference_tests -- --nocapture
```

**All tests must pass before marking the task done.**

### Phase 7: Update Documentation

If you added new concepts or changed behavior:
1. Update the relevant spec section
2. Update component guides
3. Update cross-references

### Phase 8: Mark Task Done

When all acceptance criteria are met:
1. Update `**Status:**` to `done`
2. Move file from `in-progress/` to `done/`
3. Add final notes

---

## Key Principles

### The Spec is Sacred

- `docs/01-SPEC/LANGUAGE-SPEC.md` is the source of truth
- Implementation must match the spec exactly
- If you discover the spec is unclear, update it
- If you find the spec is wrong, fix it and document why

### Tasks Are Atomic

- Each task has specific acceptance criteria
- You're done when ALL criteria are met
- Tests prove the criteria are met
- No "almost done" or "mostly works"

### Documentation is Code

- Update docs when you change behavior
- Links must be current (not broken)
- Examples in docs must actually work
- Spec clarity is a feature

### Testing is Non-Negotiable

- Every feature needs tests
- Tests should be comprehensive
- Tests are your specification
- Passing tests = done

---

## Common Tasks

### Adding a New Type Variant

1. Add to `Type` enum in `src/types/mod.rs`
2. Add to `InferType` enum in `src/typeinference/mod.rs`
3. Update `Display` implementations
4. Add test cases
5. Update spec if needed
6. Update component guides

### Adding a New Expression Form

1. Add to `Expr` enum in `src/ast/mod.rs`
2. Update parser in `src/parser/mod.rs`
3. Add type checking in `src/typechecker/mod.rs`
4. Add evaluation in `src/evaluator/mod.rs`
5. Write tests for each phase
6. Update language spec

### Fixing a Bug

1. Create a minimal test that reproduces the bug
2. Fix the implementation
3. Verify the test passes
4. Check no other tests broke (`cargo test`)
5. Update relevant documentation if needed

### Refactoring Code

1. Ensure all tests pass before starting
2. Make small changes incrementally
3. Run tests after each change
4. Don't change behavior, only structure
5. Update comments/docs if structure changed

---

## Error Messages

When something goes wrong, check:

### Compilation Error
```bash
cargo build
```
Fix the error by looking at the line number and message.

### Test Failure
```bash
cargo test --test typeinference_tests -- --nocapture
```
Tests show exactly what failed. Fix the code to match test expectations.

### Reference Error
If a file reference is broken:
1. Find the file's new location
2. Update the reference everywhere
3. Verify the link is correct with `ls` or `find`

---

## File Editing

### When Reading Files

Use the `Read` tool to understand current state before editing.

```bash
# Read a full file
Read(file_path)

# Read specific lines
Read(file_path, limit=50)
Read(file_path, offset=100, limit=50)
```

### When Writing Files

Use the `Write` tool for new files and the `Edit` tool for modifications.

```bash
# New file
Write(file_path, content)

# Modify existing file (must be read first)
Edit(file_path, old_string, new_string)
```

### When Updating References

Use `replace_all: true` to update all occurrences:

```bash
Edit(
  file_path,
  old_string="old/path/file.md",
  new_string="new/path/file.md",
  replace_all=true
)
```

---

## Testing Strategy

### For Type Inference

1. **Unit tests** - Test individual functions (unify, apply, etc.)
2. **Integration tests** - Test full pipelines (infer → solve → resolve)
3. **Acceptance tests** - Test that spec requirements are met
4. **Regression tests** - Ensure previous features still work

Example test structure:

```rust
#[cfg(test)]
mod phase_3_unification {
    use yoloscript::typeinference::*;

    #[test]
    fn test_unify_concrete_same() {
        // Test unifying identical concrete types
    }

    #[test]
    fn test_unify_concrete_different() {
        // Test unifying different concrete types (should fail)
    }

    #[test]
    fn test_unify_var_with_concrete() {
        // Test binding a variable to concrete type
    }
    
    // etc.
}
```

---

## Documentation Standards

### Code Comments

For complex logic:
```rust
// Implements constraint solving using unification.
// See Language Spec §3.2 for the algorithm.
pub fn solve_constraints(constraints: Vec<Constraint>) -> Result<Substitution, YoloscriptError> {
    // ...
}
```

### Commit Messages (if applicable)

```
[TASK-0002] Implement type variable generation

- TypeVar struct with unique IDs
- TypeVarGenerator for creating fresh variables
- Tests for ordering and hashing behavior

Closes: epic-001-typechecker/open/0002-type-inference
```

### Task Notes

Update the task file with progress:

```markdown
## Notes

### Session 1
- Implemented TypeVar struct
- Added Display trait
- Tests for basic creation passing

### Session 2
- Implemented TypeVarGenerator
- Fresh variable generation working
- All Phase 1 tests passing
```

---

## Getting Help

### If Stuck on Implementation
1. Re-read the spec section
2. Look at test expectations
3. Check related code for patterns
4. Look at error messages carefully

### If Spec is Unclear
1. Check `BACKLOG.md` for open questions
2. Look at decision records in `docs/06-DECISIONS/closed/` (accepted) or `docs/06-DECISIONS/open/` (pending)
3. If still unclear, mark it in task notes

### If Tests Keep Failing
1. Ensure you understand what the test expects
2. Print intermediate values (`println!` debugging)
3. Break the problem into smaller pieces
4. Write simpler tests first

---

## Quick Checklist

Before marking a task complete:

- [ ] All acceptance criteria met
- [ ] All tests passing (`cargo test`)
- [ ] No compiler warnings (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Documentation updated (if needed)
- [ ] Spec links are correct
- [ ] Task status moved to `done/`
- [ ] Task notes updated with completion info

---

## Example: Implementing a Phase

### Scenario: Implement Phase 2 (InferType)

1. **Read task**: `docs/05-TASKS/epic-001-typechecker/open/0002-type-inference.md`
2. **Check spec**: `docs/01-SPEC/LANGUAGE-SPEC.md` (Type System section)
3. **Move to in-progress**: Update status and file location
4. **Write tests**: Add test cases in `tests/typeinference_tests.rs`
5. **Implement**: Add `InferType` enum in `src/typeinference/mod.rs`
6. **Run tests**: `cargo test --test typeinference_tests phase_2`
7. **Fix failures**: Update implementation until tests pass
8. **Update docs**: If the enum is public API, add doc comments
9. **Mark done**: Move file to `done/` and update status

---

## Important Remember

- **The spec is law** - implement what's specified, not what you think is good
- **Tests are truth** - if tests pass, the code is correct
- **Documentation matters** - future developers (or you later) will thank you
- **Small steps** - break big tasks into small ones
- **Progress over perfection** - it's okay to complete something simple first

Good luck! 🚀
