# Yolang Language Specification

> **Status:** Active. This document is the single source of truth for the Yolang language.
> Features not described here are not part of the language.
> Open design questions and deferred features are tracked in `Backlog.md`.

Source files use the `.yolo` extension.

---

## 1. Overview

Yolang is a strongly typed, compiled language with a Rust-inspired type system. Its core design principles are:

- **Strong static typing** with full Hindley-Milner type inference
- **No classes** — data and behaviour are defined separately via structs, enums, and traits
- **Algebraic data types** — enums with data-carrying variants and exhaustive pattern matching
- **Explicit nullability** — absence of a value is represented by `Perhaps<T>` / `nope`, never by null
- **Explicit error handling** — errors are values, represented as `Result<T, E>`
- **Memory managed by the runtime** — reference counting, no ownership semantics in the language

---

## 2. Lexical Structure

### 2.1 Comments

```yolo
// Single-line comment

/* Multi-line
   comment */
```

Multi-line comments do not nest.

### 2.2 Identifiers

Identifiers start with a letter (`a–z`, `A–Z`) or underscore, followed by any combination of letters, digits, or underscores.

```
identifier := [a-zA-Z_][a-zA-Z0-9_]*
```

By convention:
- Types, structs, enums, and traits use `PascalCase`
- Variables, functions, and fields use `snake_case`

### 2.3 Keywords

```
and       as        break     continue  else      enum      false
for       fun       if        impl      let       loop      match
mut       nope      or        return    struct    trait     true
use       where     while
```

### 2.4 Literals

**Integers** — decimal, with optional `_` separators:
```yolo
42
1_000_000
```

**Floats:**
```yolo
3.14
2.0
```

Integer and float are distinct types and do not implicitly coerce.

**Strings** — double-quoted UTF-8:

| Sequence | Meaning         |
|----------|-----------------|
| `\n`     | Newline         |
| `\t`     | Tab             |
| `\\`     | Backslash       |
| `\"`     | Double quote    |
| `\r`     | Carriage return |

**Booleans:** `true`, `false`

**Absence literal:** `nope`

### 2.5 Operators

| Category        | Operators                                     |
|-----------------|-----------------------------------------------|
| Arithmetic      | `+`  `-`  `*`  `/`  `%`                       |
| Compound assign | `+=`  `-=`  `*=`  `/=`  `%=`                  |
| Comparison      | `==`  `!=`  `<`  `<=`  `>`  `>=`              |
| Logical         | `&&`  `\|\|`  `!`  (`and` / `or` as aliases)  |
| Assignment      | `=`                                           |
| Error prop      | `?`                                           |
| Type cast       | `as`                                          |
| Path            | `::`                                          |
| Range           | `..`  `..=`  (for use in `for-in` only)       |

---

## 3. Type System

Yolang is statically and strongly typed. Types are checked at compile time. There are no implicit conversions.

### 3.1 Primitive types

| Type     | Description               | Example   |
|----------|---------------------------|-----------|
| `Int`    | 64-bit signed integer     | `42`      |
| `Float`  | 64-bit floating point     | `3.14`    |
| `Bool`   | Boolean                   | `true`    |
| `String` | UTF-8 string              | `"hello"` |
| `()`     | Unit — represents no value | `()`     |

The unit type `()` is only written explicitly when needed as a type parameter (e.g. `Result<(), Error>`). Functions that return nothing omit the `->` annotation entirely.

### 3.2 Type inference

Types are inferred using the Hindley-Milner algorithm with let-polymorphism. Annotations are optional for all bindings, including function parameters and return types. They may be written explicitly for documentation or to restrict a binding to a less general type.

Annotations are required only where there is no expression to infer from:
- Struct and enum field types
- Trait method signatures

```yolo
let x = 42;           // inferred: Int
let name = "Vlad";    // inferred: String
let y: Float = 3.14;  // explicit annotation (optional here)

fun add(a: Int, b: Int) -> Int { a + b }   // annotated
fun add(a, b) { a + b }                    // also valid; inferred from use
```

### 3.3 Tuples

Tuples are lightweight anonymous product types.

```yolo
let coord: (Int, Int) = (10, 20);
let triple: (String, Int, Bool) = ("yes", 42, true);
```

Positional field access uses `.0`, `.1`, etc.:

```yolo
let x = coord.0;   // 10
let y = coord.1;   // 20
```

`()` is the zero-element tuple (unit type).

Tuples can be destructured in `match`:

```yolo
match coord {
    (0, y) => println("on y-axis"),
    (x, 0) => println("on x-axis"),
    (x, y) => println("elsewhere"),
}
```

### 3.4 Arrays

`Array<T>` is the built-in ordered sequence type. The shorthand `T[]` is preferred.

```yolo
let nums: Int[] = [1, 2, 3];
let names: Array<String> = ["alice", "bob"];
```

Index access uses `[]` with an `Int` index. Out-of-bounds access causes a panic.

```yolo
let first = nums[0];
```

Arrays are usable in `for-in` loops. `List<T>` is not available in v0.1; `T[]` is the only sequence type.

### 3.5 Type casting (`as`)

The `as` operator casts between numeric primitive types. It desugars to a call to the `From` trait and is infallible — the result is the target type directly.

```yolo
let x: Int = 42;
let f: Float = x as Float;

let f2: Float = 3.99;
let i: Int = f2 as Int;   // truncates toward zero
```

Allowed primitive casts: `Int` ↔ `Float`.

Because `as` desugars to `From`, user-defined types become castable by implementing `From<SourceType>` for the target type.

### 3.6 Generics

Types and functions can be parameterized with `<T>` syntax.

```yolo
struct Stack<T> {
    items: T[],
}

fun first<T>(arr: T[]) -> Perhaps<T> { ... }
```

Constraints are expressed with `where` clauses or inline bounds:

```yolo
fun largest<T>(a: T, b: T) -> T where T: Comparable { ... }

fun largest<T: Comparable>(a: T, b: T) -> T { ... }  // inline form
```

### 3.7 The Never type (`!`)

`!` (Never) is the bottom type — the type of an expression that never produces a value because it diverges (runs forever, panics, or exits). A `loop` with no reachable `break` has type `!`:

```yolo
let x: ! = loop { };         // runs forever — type is !
let y: ! = loop { panic!(); };
```

Because `!` coerces to every type, it can appear where any type is expected:

```yolo
let result: Int = loop { break 42; };  // break gives the loop type Int
let diverge: Int = loop { };           // ! coerces to Int — dead code after
```

`!` is not a type users write in practice; it appears as an inferred type when the typechecker determines a branch or expression cannot return. It is the type of `return` and `panic!` expressions as well.

---

## 4. Variables

### 4.1 Immutable bindings (`let`)

```yolo
let x = 42;
let name: String = "Vlad";
```

`let` bindings cannot be reassigned and must always be initialized.

### 4.2 Mutable bindings (`mut`)

```yolo
mut counter = 0;
counter = counter + 1;
counter += 1;   // compound assignment
```

`mut` bindings can be reassigned and also must be initialized at declaration. Compound assignment operators `+=`, `-=`, `*=`, `/=`, `%=` are supported.

### 4.3 Scoping and shadowing

Variables are lexically scoped. Each block `{ }` introduces a new scope. Inner scopes can shadow outer variables.

---

## 5. Functions

```yolo
fun add(a: Int, b: Int) -> Int {
    return a + b;
}
```

Parameter type annotations are optional when types can be inferred from context. The return type follows `->` and is also optional — a function with no return annotation and no `return expr;` returns `()`. `return expr;` and bare `return;` are both valid.

### 5.1 Associated functions

`impl` blocks may contain functions with no `self` parameter. These are called on the type via `::` syntax and serve as the canonical constructor pattern:

```yolo
impl Point {
    fun new(x: Float, y: Float) -> Point {
        return Point { x: x, y: y };
    }
}

let p = Point::new(1.0, 2.0);
```

### 5.2 First-class functions

Functions are first-class values and can be assigned, passed, and returned:

```yolo
let f = add;
f(1, 2);   // 3

fun apply(f: fun(Int) -> Int, x: Int) -> Int {
    return f(x);
}
```

The type of a function or closure is written as `fun(ParamTypes) -> ReturnType`.

### 5.3 Closures

Anonymous functions are written with `fun` in expression position:

```yolo
let double = fun(x: Int) -> Int { return x * 2; };
double(5);   // 10
```

Closures capture variables from their enclosing scope. Captured `mut` variables are shared — mutations are visible in the outer scope:

```yolo
mut count = 0;
let inc = fun() { count += 1; };
inc();
inc();
// count == 2
```

### 5.4 The `?` operator

Inside a function returning `Result<T, E>`, `?` propagates errors early:

```yolo
fun parse_and_double(s: String) -> Result<Int, String> {
    let n = parse_int(s)?;   // returns Err early if parse_int fails
    return Result::Ok { value: n * 2 };
}
```

`?` desugars to: if the expression is `Err(e)`, return `Err(e)` immediately; otherwise unwrap to the `Ok` value. Error types must match exactly — no implicit coercion.

---

## 6. Structs

```yolo
struct Point {
    x: Float,
    y: Float,
}
```

### 6.1 Instantiation and field access

```yolo
let p = Point { x: 1.0, y: 2.0 };
let x = p.x;
```

### 6.2 Methods (`impl`)

```yolo
impl Point {
    fun distance(self, other: Point) -> Float {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        return dx * dx + dy * dy;   // squared distance
    }
}

let d = p.distance(q);
```

`self` refers to the receiver. Methods are called with dot syntax.

### 6.3 Mutable receiver (`mut self`)

Methods that mutate the receiver declare `mut self`. Mutation happens in place:

```yolo
impl Counter {
    fun increment(mut self) {
        self.value += 1;
    }
}
```

### 6.4 Generic structs

```yolo
struct Pair<A, B> {
    first: A,
    second: B,
}
```

---

## 7. Enums

```yolo
enum Direction { North, South, East, West }

enum Shape {
    Circle { radius: Float },
    Rectangle { width: Float, height: Float },
}
```

Variants may be unit (no data) or struct-like (named fields).

### 7.1 Instantiation

```yolo
let dir = Direction::North;
let s = Shape::Circle { radius: 5.0 };
```

### 7.2 Methods on enums

`impl` blocks on enums follow the same syntax as structs:

```yolo
impl Shape {
    fun area(self) -> Float {
        match self {
            Shape::Circle { radius } => 3.14159 * radius * radius,
            Shape::Rectangle { width, height } => width * height,
        }
    }
}
```

---

## 8. `Perhaps<T>` and `nope`

`Perhaps<T>` is the built-in optional type. There is no null — all absence is expressed via `Perhaps<T>`.

```yolo
let result: Perhaps<Int> = nope;
let value: Perhaps<Int> = 42;
```

Use `match` to unwrap safely:

```yolo
match find_user(1) {
    Perhaps::Some { value } => println(value.name),
    Perhaps::Nope => println("not found"),
}
```

`.yolo()` unwraps, panicking if the value is `nope`:

```yolo
let user = find_user(1).yolo();
```

---

## 9. `Result<T, E>`

`Result<T, E>` represents the outcome of a fallible operation:

```yolo
fun divide(a: Float, b: Float) -> Result<Float, String> {
    if (b == 0.0) {
        return Result::Err { error: "division by zero" };
    }
    return Result::Ok { value: a / b };
}
```

Use `match` to handle both cases, or `?` to propagate errors.

`.yolo()` also works on `Result<T, E>`, panicking on `Err`.

---

## 10. Traits

```yolo
trait Printable {
    fun print(self);
}

trait Comparable {
    fun compare(self, other: Self) -> Int;
}
```

### 10.1 Implementing a trait

```yolo
impl Printable for Point {
    fun print(self) {
        println("(" + float_to_string(self.x) + ", " + float_to_string(self.y) + ")");
    }
}
```

### 10.2 Trait bounds in generics

```yolo
fun print_all<T: Printable>(items: T[]) {
    for (let item in items) {
        item.print();
    }
}
```

### 10.3 Default method implementations

```yolo
trait Greet {
    fun name(self) -> String;

    fun greet(self) {                          // default implementation
        println("Hello, " + self.name() + "!");
    }
}
```

### 10.4 The `Self` type

`Self` inside a trait definition refers to the concrete implementing type:

```yolo
trait Comparable {
    fun compare(self, other: Self) -> Int;
}
```

### 10.5 Static dispatch only

Trait objects (`dyn Trait`) are not available in v0.1. All polymorphism is via generics (static dispatch).

---

## 11. Pattern Matching

`match` performs exhaustive pattern matching. All cases must be covered.

```yolo
match value {
    pattern => expression,
    _       => expression,   // catch-all
}
```

`match` is an expression — all arms must produce the same type:

```yolo
let label = match x {
    0 => "zero",
    1 => "one",
    _ => "other",
};
```

### 11.1 Pattern kinds

| Pattern | Example | Matches |
|---------|---------|---------|
| Wildcard | `_` | anything, binds nothing |
| Binding | `n` | anything, binds to `n` |
| Literal | `0`, `"hi"`, `true`, `nope` | exact value |
| Enum variant | `Direction::North` | unit variant |
| Enum with fields | `Shape::Circle { radius }` | variant, binds fields |
| Tuple | `(a, b)` | tuple, binds elements |
| Guard | `n if n < 0` | binding + boolean condition |

### 11.2 Examples

```yolo
// enum destructuring
match shape {
    Shape::Circle { radius } => println(float_to_string(radius)),
    Shape::Rectangle { width, height } => println(float_to_string(width)),
}

// literal and guard
match x {
    0           => println("zero"),
    n if n < 0  => println("negative"),
    _           => println("positive"),
}

// tuple destructuring
match point {
    (0, 0) => println("origin"),
    (x, 0) => println("on x-axis"),
    (0, y) => println("on y-axis"),
    (x, y) => println("elsewhere"),
}
```

---

## 12. Control Flow

### 12.1 If / else

```yolo
if (condition) {
    // ...
} else if (other) {
    // ...
} else {
    // ...
}
```

`if` is also an expression (both branches must produce the same type):

```yolo
let label = if (x > 0) { "positive" } else { "non-positive" };
```

### 12.2 While

```yolo
while (condition) {
    // ...
}
```

### 12.3 C-style for

```yolo
for (mut i = 0; i < 10; i += 1) {
    // ...
}
```

### 12.4 For-in

Iterates over any array or range:

```yolo
for (let item in collection) { ... }
for (let i in 0..10) { ... }    // 0, 1, ..., 9
for (let i in 0..=10) { ... }   // 0, 1, ..., 10
```

### 12.5 Loop

`loop` creates an infinite loop. It is the only loop form that can produce a value:

```yolo
loop {
    // runs forever unless break is used
}

let result = loop {
    if (condition) { break value; }
};
```

**Typing rules:**

- `loop { break expr; }` has type `T` where `expr: T`. All `break` arms must produce the same type.
- `loop { }` — a loop with no reachable `break` — has type `!` (Never). See [§3.7](#37-the-never-type-).

### 12.6 Break and continue

`break` exits the innermost loop. `break expr` exits a `loop` and produces `expr` as the loop's value.

`continue` skips to the next iteration of the innermost loop.

### 12.7 Return

```yolo
return;         // from a function returning ()
return value;   // from a typed function
```

---

## 13. Panics

A panic is a hard, unrecoverable runtime error. It prints a message and exits the process with a non-zero status. Panics cannot be caught.

Panics are triggered by:
- `.yolo()` on `nope` or a `Result::Err`
- Out-of-bounds array access
- Integer division by zero

---

## 14. Grammar

```
Program            → Declaration* EOF

Declaration        → LetDeclaration
                   | MutDeclaration
                   | FunDeclaration
                   | StructDeclaration
                   | EnumDeclaration
                   | ImplBlock
                   | TraitDeclaration
                   | Statement

LetDeclaration     → "let" IDENTIFIER ( ":" Type )? "=" Expression ";"
MutDeclaration     → "mut" IDENTIFIER ( ":" Type )? "=" Expression ";"
FunDeclaration     → "fun" IDENTIFIER GenericParams? "(" Params? ")" ( "->" Type )? Block
StructDeclaration  → "struct" IDENTIFIER GenericParams? "{" StructFields "}"
EnumDeclaration    → "enum" IDENTIFIER GenericParams? "{" EnumVariants "}"
ImplBlock          → "impl" ( Type "for" )? Type "{" FunDeclaration* "}"
TraitDeclaration   → "trait" IDENTIFIER "{" TraitMethod* "}"
TraitMethod        → "fun" IDENTIFIER "(" Params? ")" ( "->" Type )? ( Block | ";" )

Params             → Param ( "," Param )*
Param              → ( "mut" )? "self" | IDENTIFIER ( ":" Type )?
StructFields       → StructField ( "," StructField )* ","?
StructField        → IDENTIFIER ":" Type
EnumVariants       → EnumVariant ( "," EnumVariant )* ","?
EnumVariant        → IDENTIFIER ( "{" StructFields "}" )?
GenericParams      → "<" GenericParam ( "," GenericParam )* ">"
GenericParam       → IDENTIFIER ( ":" Type )?

Statement          → ExpressionStatement
                   | Block
                   | IfStatement
                   | WhileStatement
                   | ForStatement
                   | LoopStatement
                   | ReturnStatement
                   | BreakStatement
                   | ContinueStatement

ExpressionStatement → Expression ";"
Block               → "{" Declaration* "}"
IfStatement         → "if" "(" Expression ")" Block ( "else" ( IfStatement | Block ) )?
WhileStatement      → "while" "(" Expression ")" Block
ForStatement        → "for" "(" ForInit Expression? ";" Expression? ")" Block
                    | "for" "(" "let" IDENTIFIER "in" Expression ")" Block
ForInit             → MutDeclaration | ExpressionStatement | ";"
LoopStatement       → "loop" Block
ReturnStatement     → "return" Expression? ";"
BreakStatement      → "break" Expression? ";"
ContinueStatement   → "continue" ";"

Expression              → AssignmentExpression
AssignmentExpression    → LValue AssignOp AssignmentExpression | LogicalOrExpression
LValue                  → IDENTIFIER | CallExpression "." IDENTIFIER | CallExpression "[" Expression "]"
AssignOp                → "=" | "+=" | "-=" | "*=" | "/=" | "%="
LogicalOrExpression     → LogicalAndExpression ( "||" LogicalAndExpression )*
LogicalAndExpression    → ComparisonExpression ( "&&" ComparisonExpression )*
ComparisonExpression    → TermExpression ( ( ">" | ">=" | "<" | "<=" | "!=" | "==" ) TermExpression )?
TermExpression          → FactorExpression ( ( "+" | "-" ) FactorExpression )*
FactorExpression        → CastExpression ( ( "*" | "/" | "%" ) CastExpression )*
CastExpression          → UnaryExpression ( "as" Type )*
UnaryExpression         → ( "!" | "-" ) UnaryExpression | PostfixExpression
PostfixExpression       → PrimaryExpression ( "(" Arguments? ")" | "." IDENTIFIER | "[" Expression "]" | "?" )*
Arguments               → Expression ( "," Expression )*

PrimaryExpression  → INT | FLOAT | STRING | "true" | "false" | "nope" | "()"
                   | "(" Expression ( "," Expression )+ ")"   // tuple
                   | "(" Expression ")"
                   | "[" ( Expression ( "," Expression )* ","? )? "]"  // array literal
                   | IDENTIFIER ( "::" IDENTIFIER )*
                   | StructLiteral
                   | MatchExpression
                   | IfExpression
                   | LoopExpression
                   | ClosureExpression

StructLiteral      → IDENTIFIER ( "::" IDENTIFIER )* "{" FieldInit ( "," FieldInit )* ","? "}"
FieldInit          → IDENTIFIER ":" Expression

MatchExpression    → "match" Expression "{" MatchArm ( "," MatchArm )* ","? "}"
MatchArm           → Pattern ( "if" Expression )? "=>" Expression
IfExpression       → "if" "(" Expression ")" Block "else" Block
LoopExpression     → "loop" Block
ClosureExpression  → "fun" "(" Params? ")" ( "->" Type )? Block

Pattern            → "_"
                   | "nope"
                   | IDENTIFIER
                   | "(" Pattern ( "," Pattern )* ")"          // tuple pattern
                   | IDENTIFIER "::" IDENTIFIER ( "{" PatternFields "}" )?
                   | INT | FLOAT | STRING | "true" | "false"
PatternFields      → IDENTIFIER ( "," IDENTIFIER )*

Type               → IDENTIFIER ( "<" TypeArgs ">" )?
                   | "()"
                   | "(" Type ( "," Type )+ ")"                // tuple type
                   | Type "[]"                                  // array shorthand
                   | "fun" "(" TypeList? ")" ( "->" Type )?    // function type
TypeArgs           → Type ( "," Type )*
TypeList           → Type ( "," Type )*
```

---

## 15. Built-in functions

These are available globally without any `use` declaration:

| Name              | Signature                           | Description                              |
|-------------------|-------------------------------------|------------------------------------------|
| `print`           | `(s: String)`                       | Print to stdout, no newline              |
| `println`         | `(s: String)`                       | Print to stdout with newline             |
| `int_to_string`   | `(n: Int) -> String`                | Decimal string representation of an Int |
| `float_to_string` | `(n: Float) -> String`              | String representation of a Float        |
| `bool_to_string`  | `(b: Bool) -> String`               | `"true"` or `"false"`                   |
| `string_len`      | `(s: String) -> Int`                | Number of characters in a string        |
| `string_concat`   | `(a: String, b: String) -> String`  | Concatenate two strings (also via `+`)  |
| `array_push`      | `(arr: T[], value: T)`              | Append a value (mutates the array)      |
| `array_len`       | `(arr: T[]) -> Int`                 | Number of elements in an array          |
| `clock`           | `() -> Int`                         | Unix timestamp in milliseconds          |
