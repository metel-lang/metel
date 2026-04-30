# Type Inference Implementation Guide

## Fresh Start

You've chosen to build the type inference system step-by-step with tests. This is the right approach—it ensures you understand each piece deeply.

---

## 📁 What's Been Created

### Minimal Foundation
- ✅ `src/typeinference/mod.rs` - Phase 1 only (TypeVar & TypeVarGenerator)
- ✅ `src/lib.rs` - NEW: Makes modules testable
- ✅ `tests/typeinference_tests.rs` - Complete test structure (618 lines)

### Documentation
- ✅ `TYPEINFERENCE_ROADMAP.md` - Full 8-phase breakdown with all details
- ✅ `TYPEINFERENCE_SETUP.md` - Quick start guide
- ✅ `IMPLEMENTATION_GUIDE.md` - This file
- 📚 `TYPE_SCHEMES_DESIGN.md` - Reference material (not needed yet)

### Tasks Created
- ✅ Task #3: Master task "Build Type Inference System Step-by-Step"
- ✅ Task #4: Phase 2 - InferType enum
- ✅ Task #5: Phase 3 - Unification algorithm
- ✅ Task #6: Phase 4 - Substitution

---

## 🚀 Quick Start

### 1. Understand the Full Picture
```bash
# Read this file (you are here)
# Then read TYPEINFERENCE_ROADMAP.md (5-10 min)
```

### 2. Check Phase 1 Tests Pass
```bash
cd tree-walk-interpreter
cargo test --test typeinference_tests phase_1
```

Expected output:
```
test phase_1_type_variables::test_type_var_creation ... ok
test phase_1_type_variables::test_type_var_display ... ok
test phase_1_type_variables::test_type_var_generator_fresh ... ok
test phase_1_type_variables::test_type_var_generator_counter ... ok
test phase_1_type_variables::test_type_var_ordering ... ok
test phase_1_type_variables::test_type_var_hashable ... ok

test result: ok. 6 passed
```

### 3. Start Phase 2
Open `TYPEINFERENCE_ROADMAP.md` and follow Phase 2 section.

---

## 📝 Implementation Workflow for Each Phase

### Template: How to Do a Phase

**1. Read the roadmap**
```
→ Open TYPEINFERENCE_ROADMAP.md
→ Find "Phase N: ___"
→ Read "What", "Definition", "Tasks"
```

**2. Examine test stubs**
```
→ Open tests/typeinference_tests.rs
→ Go to phase_N_xxx section
→ Look at test names and comments
```

**3. Implement in src/typeinference/mod.rs**
```rust
// Add your code here
pub struct MyNewType { ... }
impl Display for MyNewType { ... }
```

**4. Implement test assertions**
```rust
// Replace todo!() with actual test code
#[test]
fn test_something() {
    let result = my_function();
    assert_eq!(result, expected);
}
```

**5. Run tests**
```bash
cargo test --test typeinference_tests phase_N
```

**6. Fix any failures**
- Debug implementation
- Update test if needed
- Re-run tests

**7. Celebrate** ✅
Move to next phase

---

## 🧪 Test Structure

All tests are in `tests/typeinference_tests.rs` organized by phase:

```rust
#[cfg(test)]
mod phase_1_type_variables { ... }   // ✅ Complete

#[cfg(test)]
mod phase_2_infer_types { ... }       // Next: Add your impl here

#[cfg(test)]
mod phase_3_unification { ... }

#[cfg(test)]
mod phase_4_substitution { ... }

#[cfg(test)]
mod phase_5_constraints { ... }

#[cfg(test)]
mod phase_6_type_schemes { ... }

#[cfg(test)]
mod phase_7_inference_context { ... }

#[cfg(test)]
mod phase_6_integration { ... }
```

### Running Tests

**All tests**:
```bash
cargo test --test typeinference_tests
```

**Specific phase**:
```bash
cargo test --test typeinference_tests phase_2
```

**Specific test**:
```bash
cargo test --test typeinference_tests phase_2::test_infer_type_concrete
```

**With output**:
```bash
cargo test --test typeinference_tests phase_2 -- --nocapture
```

---

## 🔗 The 8 Phases at a Glance

| Phase | What | Key Type | Lines of Code |
|-------|------|----------|---------------|
| 1 | Type variables | `TypeVar` | ~50 |
| 2 | Inference types | `InferType` | ~100 |
| 3 | Unification | `unify()` fn | ~150 |
| 4 | Substitution | `Substitution` | ~100 |
| 5 | Constraints | `Constraint` | ~80 |
| 6 | Type schemes | `TypeScheme` | ~150 |
| 7 | Context | `InferContext` | ~200 |
| 8 | Integration | expression walk | ~300 |

**Total**: ~1,130 lines of implementation + tests

---

## 🎯 Success Criteria

After completing each phase, you should be able to answer:

**Phase 1**: 
- How do you generate fresh type variables?
- Why do type variables need unique IDs?

**Phase 2**:
- What's the difference between `Type` (concrete) and `InferType` (with variables)?
- Why do you need both?

**Phase 3**:
- How does unification work at a high level?
- What's the occurs check and why does it matter?

**Phase 4**:
- What's a substitution and when do you apply it?
- How do you compose two substitutions?

**Phase 5**:
- What's a constraint and why collect them separately?
- How do you solve multiple constraints together?

**Phase 6**:
- What's generalization and when does it happen?
- What's instantiation and when does it happen?

**Phase 7**:
- What state does the inference context maintain?
- How do monomorphic and polymorphic environments differ?

**Phase 8**:
- How does the expression walker generate constraints?
- How does constraint solving produce a typed AST?

---

## 💡 Tips for Understanding

### When You Feel Lost
1. Re-read the "What" section in TYPEINFERENCE_ROADMAP.md
2. Look at the examples in the roadmap
3. Study the test cases—they show expected behavior
4. Run tests with `--nocapture` to see error messages

### When Tests Fail
1. Read the error message carefully
2. Check if your implementation matches the test's expectations
3. Add `println!()` statements to debug
4. Run a single test to focus: `cargo test phase_2::test_x -- --nocapture`

### When You Have Questions
- Look for "Key Concept" sections in TYPEINFERENCE_ROADMAP.md
- Check TYPE_SCHEMES_DESIGN.md for deep explanation of concepts
- Read test comments—they often explain the intent

### Building Intuition
After each phase, think about:
- **What problem does this solve?** (e.g., Phase 3 unification solves "how do I determine if ?t0 must be Int?")
- **When would you use it?** (e.g., Substitution is applied after unification determines bindings)
- **How does it connect to previous phases?**

---

## 📚 Reference Documents

### Primary
- **TYPEINFERENCE_ROADMAP.md** - Complete phase-by-phase specification
- **tests/typeinference_tests.rs** - Executable examples of expected behavior

### Secondary
- **TYPE_SCHEMES_DESIGN.md** - Deep dive on let-polymorphism (for Phase 6)
- **TYPEINFERENCE_SETUP.md** - Quick reference guide

### Code
- **src/typeinference/mod.rs** - Your implementation file
- **src/lib.rs** - Module exports for tests
- **src/types/mod.rs** - The concrete `Type` enum (for reference)
- **src/error/mod.rs** - The `YolangError` type (for error handling)

---

## 🐛 Debugging Checklist

- [ ] Does the code compile? (`cargo build`)
- [ ] Do the tests compile? (`cargo test --test typeinference_tests --no-run`)
- [ ] Does one test pass? (Pick the simplest test)
- [ ] Do all tests for this phase pass?
- [ ] Does the next phase compile?

---

## 🎓 Learning Outcomes

After completing this, you will understand:

✅ How type inference works at each step  
✅ The relationship between unification, substitution, and constraints  
✅ How type variables represent unknowns  
✅ How polymorphic types enable code reuse  
✅ The Hindley-Milner algorithm basics  
✅ How to test type system code  

---

## 🚦 Current Status

**Completed**:
- ✅ Phase 1: Type Variables (6 tests passing)

**Ready to start**:
- ⏳ Phase 2: InferType enum

**Next**:
- ⏳ Phase 3-8: Unification → Integration

---

## What to Do Now

1. **Read** `TYPEINFERENCE_ROADMAP.md` completely (understand all 8 phases)
2. **Look at** the Phase 2 test stubs in `tests/typeinference_tests.rs`
3. **Implement** Phase 2 in `src/typeinference/mod.rs`
4. **Run** `cargo test --test typeinference_tests phase_2`
5. **Verify** all tests pass
6. **Repeat** for Phases 3-8

**Time estimate**: 1 week for thorough understanding, 2-3 days for implementation

---

**Good luck! You've got a solid plan. Let's build this properly.** 🚀
