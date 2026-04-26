# Type Checker & Typed AST Implementation Tasks

## Objective
Implement a proof-of-concept type checker and typed AST to enable the evaluator to process and execute all existing Yolang tests (01-10).

## Current State
- **Parser**: Generates untyped AST from Yolang source code
- **Type Checker**: Stub with TODO comments outlining 5 phases
- **Typed AST**: Placeholder implementation with only `_Placeholder` variant
- **Types**: Enum and Display trait already defined
- **Evaluator**: Ready to consume typed AST (implementation pending)
- **Tests**: 10 test files covering core language features (literals, control flow, functions, structs, enums, traits, arrays, error handling, casting, and a comprehensive integration test)

## Test Coverage Target
The type checker must handle all features tested in:
1. `01_literals_and_variables.yolo` — Int, Float, Bool, Str, Unit, type inference, shadowing
2. `02_control_flow.yolo` — if/else, while, for, for-in, loop, break, continue
3. `03_functions_and_closures.yolo` — named functions, closures, generics, higher-order functions
4. `04_structs_and_impl.yolo` — struct definitions, field access, methods, constructors
5. `05_enums_and_match.yolo` — enum definitions, match expressions, Perhaps<T>, Result<T, E>
6. `06_traits.yolo` — trait definitions, trait implementations, trait bounds
7. `07_arrays_and_tuples.yolo` — Array<T>, T[], tuple literals and destructuring
8. `08_error_handling.yolo` — Result<T, E>, ? operator, error propagation
9. `09_casting_and_generics.yolo` — type casting, generic instantiation
10. `10_comprehensive.yolo` — integration test combining most features

---

## Phase 1: Extend Typed AST to Mirror Untyped AST

### Task 1.1: Create typed declaration enum
**What**: Replace the `_Placeholder` in `typed_ast::TypedDecl` with proper variants that mirror `ast::Decl`.

**Subtasks**:
- [ ] Add `LetDecl(TypedLetDecl)` variant
- [ ] Add `MutDecl(TypedMutDecl)` variant
- [ ] Add `FunDecl(TypedFunDecl)` variant
- [ ] Add `StructDecl(TypedStructDecl)` variant
- [ ] Add `EnumDecl(TypedEnumDecl)` variant
- [ ] Add `ImplBlock(TypedImplBlock)` variant
- [ ] Add `TraitDecl(TypedTraitDecl)` variant
- [ ] Add `Stmt(TypedStmt)` variant
- [ ] Remove `_Placeholder` variant

**Notes**:
- Each typed declaration must carry type information for all contained expressions
- Use `Span` from error module for source location tracking
- Reference the untyped AST structure as a template

---

### Task 1.2: Create typed statement enum
**What**: Define `TypedStmt` enum with typed variants of all statement types.

**Subtasks**:
- [ ] Define `TypedIfStmt` with typed `condition` and `branches`
- [ ] Define `TypedWhileStmt` with typed `condition` and `body`
- [ ] Define `TypedForStmt` with typed `init`, `condition`, `step`, and `body`
- [ ] Define `TypedForInStmt` with typed `iterable` and `body`
- [ ] Define `TypedLoopStmt` with typed `body`
- [ ] Define `TypedMatchExpr` with typed `scrutinee` and `arms`
- [ ] Define `TypedReturnStmt` with optional typed `value`
- [ ] Define `TypedBreakStmt` with optional typed `value`
- [ ] Add `Continue` variant for continue statements

**Notes**:
- Every statement that contains expressions must carry the type of those expressions
- Match arms must carry types for their patterns and guard expressions

---

### Task 1.3: Create typed expression enum
**What**: Define `TypedExpr` enum where every variant carries a resolved `Type`.

**Subtasks**:
- [ ] Add `Type` field to every expression variant OR use a wrapper struct `Expr { ty: Type, kind: ExprKind }`
- [ ] Create variants for: Literal, Ident, Path, Tuple, Array, BinOp, UnaryOp, Assign, Call, MethodCall, FieldAccess, TupleAccess, Index, Cast, Match, If, Loop, Closure, StructLiteral, PropagateError
- [ ] Implement `span()` method on `TypedExpr` to extract source location

**Design Choice**:
Consider whether to use:
- **Option A**: `enum TypedExpr { Literal(Type, Literal, Span), ... }` — type at the front
- **Option B**: `struct TypedExpr { expr: ExprKind, ty: Type, span: Span }` — wrapping struct
- **Recommendation**: Option B is cleaner and avoids duplication; type information is uniform

---

### Task 1.4: Create typed block structure
**What**: Define `TypedBlock` to hold a list of typed declarations and an optional tail expression.

**Subtasks**:
- [ ] Define `TypedBlock { stmts: Vec<TypedDecl>, tail: Option<Box<TypedExpr>>, span: Span }`
- [ ] Ensure tail expression type is accessible for block type inference

**Notes**:
- Block type is determined by the tail expression or `Unit` if no tail

---

### Task 1.5: Create support structures for type information
**What**: Define typed versions of parameter, field, variant, and method structures.

**Subtasks**:
- [ ] `TypedParam { mutable: bool, name: String, ty: Type, span: Span }`
- [ ] `TypedFieldDef { name: String, ty: Type, span: Span }`
- [ ] `TypedVariantDef { name: String, fields: Vec<TypedFieldDef>, span: Span }`
- [ ] `TypedTraitMethod { name: String, params: Vec<TypedParam>, return_type: Type, default_body: Option<TypedBlock>, span: Span }`

---

## Phase 2: Declaration Collection & Type Resolution

### Task 2.1: Build declaration table
**What**: Implement the first phase of the type checker: collect all top-level declarations (functions, structs, enums, traits) and build a symbol table.

**Subtasks**:
- [ ] Create `struct DeclarationTable { functions: Map<String, FunDecl>, structs: Map<String, StructDecl>, enums: Map<String, EnumDecl>, traits: Map<String, TraitDecl>, ... }`
- [ ] Walk through `Program.decls` and populate the table
- [ ] Detect and report duplicate declarations
- [ ] Handle generic declarations separately (they are templates, not concrete types)

**Notes**:
- This allows forward references (calling a function declared later)
- Should be done once, at the start of type checking

---

### Task 2.2: Resolve type annotations
**What**: Implement type resolution: convert `TypeExpr` (syntactic type expressions) to `Type` (semantic types).

**Subtasks**:
- [ ] Create `fn resolve_type_expr(expr: &TypeExpr, table: &DeclarationTable) -> Result<Type, YolangError>`
- [ ] Handle built-in types: Int, Float, Bool, Str, Unit
- [ ] Handle named types: Look up structs/enums in the declaration table
- [ ] Handle parametric types: `Perhaps<T>`, `Result<T, E>`, generic structs
- [ ] Handle array types: `T[]` and `Array<T>`
- [ ] Handle function types: `fun(T, U) -> V`
- [ ] Handle tuple types: `(T, U, V)`
- [ ] Report errors for undefined types

---

### Task 2.3: Check struct field types
**What**: Resolve all field type annotations in struct declarations.

**Subtasks**:
- [ ] For each `StructDecl` in the table, resolve all `FieldDef.type_ann` to concrete `Type`
- [ ] Store resolved types in `TypedStructDecl`
- [ ] Detect duplicate field names within a struct
- [ ] Report errors for invalid type expressions

---

### Task 2.4: Check enum variant types
**What**: Resolve all variant field types in enum declarations.

**Subtasks**:
- [ ] For each `EnumDecl`, resolve all field types in all variants
- [ ] Store resolved types in `TypedEnumDecl`
- [ ] Detect duplicate variant names within an enum
- [ ] Detect duplicate field names within a variant

---

### Task 2.5: Check trait method signatures
**What**: Resolve parameter and return types for all trait methods.

**Subtasks**:
- [ ] For each `TraitDecl`, resolve parameter and return types
- [ ] Store resolved signatures in `TypedTraitDecl`
- [ ] Detect duplicate method names within a trait

---

## Phase 3: Type Inference & Checking for Function Bodies

### Task 3.1: Implement type inference context
**What**: Create a context structure to track variable bindings and types during inference.

**Subtasks**:
- [ ] `struct TypeCheckContext { var_types: Map<String, Type>, current_function: Option<FunDecl>, ... }`
- [ ] Methods to bind/lookup variables
- [ ] Stack-based scoping for nested blocks
- [ ] Track mutable vs immutable bindings

---

### Task 3.2: Implement expression type inference
**What**: Implement `fn infer_expr(expr: &Expr, ctx: &TypeCheckContext) -> Result<(TypedExpr, Type), YolangError>`.

**Subtasks**:
- [ ] **Literals**: Int/Float/Bool/Str/Nope → their respective types
- [ ] **Identifiers**: Look up in context; error if not found
- [ ] **Paths**: Resolve enum variants, struct constructors, etc.
- [ ] **Tuples**: Infer type of each element; tuple type is `Tuple(types)`
- [ ] **Arrays**: Infer element types; ensure all elements have same type; array type is `Array(T)`
- [ ] **Binary operations**: Check operand types; return operation result type
- [ ] **Unary operations**: Check operand type; return result type
- [ ] **Function calls**: Resolve callee type; check argument count/types; return function return type
- [ ] **Method calls**: Resolve receiver type; look up method in impl blocks; check argument types
- [ ] **Field access**: Resolve object type; check field exists; return field type
- [ ] **Tuple access**: Resolve object type; check index is in range; return element type
- [ ] **Index access**: Resolve object type (must be Array); resolve index type (must be Int); return element type
- [ ] **Cast expressions**: Resolve target type; check cast is valid (Int↔Float)
- [ ] **Match expressions**: Type scrutinee; type each arm; ensure all arms have same type
- [ ] **If expressions**: Type condition (must be Bool); type branches; ensure equal types
- [ ] **Loops**: Type body; loop expression type is unit or break value type
- [ ] **Closures**: Infer parameter and return types; closure type is `Fun(param_types, return_type)`
- [ ] **Struct literals**: Resolve struct type; check field names and types
- [ ] **Error propagation (`?`)**: Unwrap Result or Perhaps; propagate error type

---

### Task 3.3: Implement statement type inference
**What**: Implement `fn infer_stmt(stmt: &Stmt, ctx: &mut TypeCheckContext) -> Result<TypedStmt, YolangError>`.

**Subtasks**:
- [ ] **Let bindings**: Infer value type; bind variable name with inferred type; error if shadowing rules violated
- [ ] **Mut bindings**: Same as let, but mark binding as mutable
- [ ] **If statements**: Type condition (must be Bool); type branches
- [ ] **While statements**: Type condition (must be Bool); type body; check break values
- [ ] **For loops**: Type init; type condition; type step; type body
- [ ] **For-in loops**: Type iterable (must be Array or have iterator); bind loop variable; type body
- [ ] **Loop statements**: Type body; check break values; loop value is break value or unit
- [ ] **Match statements**: Type scrutinee; type all arms
- [ ] **Return statements**: Check return type matches function signature
- [ ] **Break statements**: Ensure in a loop; check break value type if present
- [ ] **Continue statements**: Ensure in a loop

---

### Task 3.4: Implement block type inference
**What**: Infer type of a block: `Unit` if no tail, or type of tail expression.

**Subtasks**:
- [ ] Process declarations in sequence, updating context
- [ ] If tail expression exists, infer its type; block type = tail type
- [ ] If no tail, block type = Unit

---

### Task 3.5: Type-check function declarations
**What**: Infer types in function bodies and check against declared return type.

**Subtasks**:
- [ ] For each `FunDecl`: Create fresh context with function parameters bound
- [ ] Infer block type; compare against declared return type (if present)
- [ ] If return type was omitted, infer it from block
- [ ] Check all code paths have consistent return type
- [ ] Store typed function in result

---

### Task 3.6: Type-check impl blocks
**What**: Infer types in impl block methods.

**Subtasks**:
- [ ] For each method in an impl block: type-check as a function
- [ ] Add `self` binding with the target type
- [ ] For trait impls, check all trait methods are implemented
- [ ] Check method signatures match trait (if trait impl)

---

## Phase 4: Monomorphisation of Generics

### Task 4.1: Identify generic instantiations
**What**: Collect all places where generic functions, structs, or enums are used with concrete type arguments.

**Subtasks**:
- [ ] Walk typed AST and collect all calls/instantiations with their type arguments
- [ ] Build a set of `(generic_name, type_arguments)` pairs to be monomorphised

---

### Task 4.2: Monomorphise generic functions
**What**: For each generic function instantiation, create a concrete copy with type arguments substituted.

**Subtasks**:
- [ ] `fn monomorphise_function(generic: &TypedFunDecl, type_args: &[Type]) -> TypedFunDecl`
- [ ] Substitute type variables with concrete types throughout function body
- [ ] Create unique name for monomorphised version (e.g., `foo<Int, Float>` → `foo_Int_Float`)
- [ ] Add to declaration table

---

### Task 4.3: Monomorphise generic structs
**What**: For each generic struct instantiation, create a concrete copy.

**Subtasks**:
- [ ] `fn monomorphise_struct(generic: &TypedStructDecl, type_args: &[Type]) -> TypedStructDecl`
- [ ] Substitute type variables in field types
- [ ] Create unique name for monomorphised version
- [ ] Add to declaration table

---

### Task 4.4: Monomorphise generic enums
**What**: For each generic enum instantiation, create a concrete copy.

**Subtasks**:
- [ ] `fn monomorphise_enum(generic: &TypedEnumDecl, type_args: &[Type]) -> TypedEnumDecl`
- [ ] Substitute type variables in variant field types
- [ ] Create unique name for monomorphised version
- [ ] Add to declaration table

---

## Phase 5: Match Exhaustiveness Checking

### Task 5.1: Implement pattern coverage analysis
**What**: Check that match expressions cover all possible values.

**Subtasks**:
- [ ] `fn check_match_exhaustive(scrutinee_type: &Type, patterns: &[Pattern]) -> Result<(), YolangError>`
- [ ] For enums: all variants must be covered (or a wildcard must exist)
- [ ] For booleans: both true and false must be covered
- [ ] For other types: wildcard or literal must exist
- [ ] Allow `_` wildcard to match any value

---

### Task 5.2: Check pattern validity
**What**: Ensure patterns match the scrutinee type.

**Subtasks**:
- [ ] Enum variant patterns must match scrutinee type
- [ ] Literal patterns must match scrutinee type
- [ ] Tuple patterns must match tuple type
- [ ] Binding patterns always match
- [ ] Report errors for type mismatches

---

## Integration & Testing

### Task 6.1: Integrate type checker into pipeline
**What**: Update `main.rs` or runner to invoke type checker after parsing.

**Subtasks**:
- [ ] Parse source → untyped AST
- [ ] Run type checker → typed AST (or error)
- [ ] Pass typed AST to evaluator

---

### Task 6.2: Test on 01_literals_and_variables.yolo
**Subtasks**:
- [ ] Run type checker on test file
- [ ] Verify all variable types are inferred correctly
- [ ] Check no type errors are reported
- [ ] Verify typed AST is produced

---

### Task 6.3: Test on 02_control_flow.yolo
**Subtasks**:
- [ ] Verify condition types are Bool
- [ ] Verify loop body types
- [ ] Check break values have consistent type

---

### Task 6.4: Test on 03_functions_and_closures.yolo
**Subtasks**:
- [ ] Verify function parameter/return types
- [ ] Verify closure types
- [ ] Check higher-order function calls

---

### Task 6.5: Test on 04_structs_and_impl.yolo, 05_enums_and_match.yolo, 06_traits.yolo
**Subtasks**:
- [ ] Verify struct field access types
- [ ] Verify method resolution
- [ ] Verify enum variant pattern matching
- [ ] Verify trait implementations

---

### Task 6.6: Test on 07_arrays_and_tuples.yolo, 08_error_handling.yolo, 09_casting_and_generics.yolo
**Subtasks**:
- [ ] Verify array element types
- [ ] Verify tuple access types
- [ ] Verify Result/Perhaps handling
- [ ] Verify type casts
- [ ] Verify generic instantiation

---

### Task 6.7: Test on 10_comprehensive.yolo
**Subtasks**:
- [ ] Full integration test
- [ ] Verify all features work together
- [ ] Check no type errors

---

## Implementation Order Recommendation

1. **Phase 1** (1.1–1.5): Define all typed AST structures → enables passing around typed information
2. **Phase 2** (2.1–2.5): Build declaration table and resolve types → foundation for type checking
3. **Phase 3** (3.1–3.6): Implement expression and statement type inference → core type checking logic
4. **Phase 4** (4.1–4.4): Monomorphisation → handle generics
5. **Phase 5** (5.1–5.2): Match exhaustiveness → ensure correctness
6. **Integration** (6.1–6.7): Wire up and test → validate against all test files

---

## Notes for PoC Implementation

- **Simplicity over perfection**: Focus on correctness, not optimization
- **Clarity**: Write code to be easy to modify as the language spec evolves
- **Error messages**: Provide helpful error messages including source locations
- **Iterative development**: Test on simpler features (tests 01–02) before complex ones (tests 08–10)
- **Expect rewrites**: This implementation will be rewritten once the language spec stabilizes; don't over-engineer

