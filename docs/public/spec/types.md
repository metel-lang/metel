# Type System

Gust is statically and strongly typed. Types are checked at compile time. There are no implicit conversions.

## Primitive Types

| Type     | Description               | Example   |
|----------|---------------------------|-----------|
| `Int`    | 64-bit signed integer     | `42`      |
| `Float`  | 64-bit floating point     | `3.14`    |
| `Bool`   | Boolean                   | `true`    |
| `String` | UTF-8 string              | `"hello"` |
| `()`     | Unit — represents no value | `()`     |

The unit type `()` is only written explicitly when needed as a type parameter (e.g. `Result<(), Error>`). Functions that return nothing omit the `->` annotation entirely.

## Type Inference

Types are inferred using the Hindley-Milner algorithm with let-polymorphism. Annotations are optional for all bindings, including function parameters and return types. They may be written explicitly for documentation or to restrict a binding to a less general type.

Annotations are required only where there is no expression to infer from:
- Struct and enum field types
- Trait method signatures

```gust
let x = 42;           // inferred: Int
let name = "Vlad";    // inferred: String
let y: Float = 3.14;  // explicit annotation (optional here)

fun add(a: Int, b: Int) -> Int { a + b }   // annotated
fun add(a, b) { a + b }                    // also valid; inferred from use
```

## Tuples

Tuples are lightweight anonymous product types.

```gust
let coord: (Int, Int) = (10, 20);
let triple: (String, Int, Bool) = ("yes", 42, true);
```

Positional field access uses `.0`, `.1`, etc.:

```gust
let x = coord.0;   // 10
let y = coord.1;   // 20
```

`()` is the zero-element tuple (unit type).

Tuples can be destructured in `match`:

```gust
match coord {
    (0, y) => println("on y-axis"),
    (x, 0) => println("on x-axis"),
    (x, y) => println("elsewhere"),
}
```

## Arrays

`Array<T>` is the built-in ordered sequence type. The shorthand `T[]` is preferred.

```gust
let nums: Int[] = [1, 2, 3];
let names: Array<String> = ["alice", "bob"];
```

Index access uses `[]` with an `Int` index. Out-of-bounds access causes a panic.

```gust
let first = nums[0];
```

Arrays are usable in `for-in` loops. `List<T>` is not available in v0.1; `T[]` is the only sequence type.

## Type Ascription

The `:` operator asserts that an expression has a given type without performing any runtime conversion. It is a pure type-inference hint — no code is emitted at runtime.

```gust
// Resolve the element type of an empty array literal.
let xs = [] : Int[];

// Assert that a variable has the expected type.
let n: Int = some_expr : Int;

// Resolve an ambiguous argument.
take_array([] : Float[]);
```

Ascription fails at compile time if the expression's inferred type is incompatible with the given type. Use `as` to convert between types; use `:` only when the value already has the target type and you want to make it explicit to the type checker.

## Type Casting

The `as` operator casts between numeric primitive types. It desugars to a call to the `From` trait and is infallible — the result is the target type directly.

```gust
let x: Int = 42;
let f: Float = x as Float;

let f2: Float = 3.99;
let i: Int = f2 as Int;   // truncates toward zero
```

Allowed primitive casts: `Int` ↔ `Float`.

Because `as` desugars to `From`, user-defined types become castable by implementing `From<SourceType>` for the target type.

## Generics

> **v0.2 feature.** User-defined generic functions and types are not available in v0.1.
> Built-in generic types (`Perhaps<T>`, `Result<T, E>`, `T[]`) are supported in v0.1 as
> special cases in the type system.

Types and functions can be parameterized with `<T>` syntax.

```gust
struct Stack<T> {
    items: T[],
}

fun first<T>(arr: T[]) -> Perhaps<T> { ... }
```

Constraints are expressed with `where` clauses or inline bounds:

```gust
fun largest<T>(a: T, b: T) -> T where T: Comparable { ... }

fun largest<T: Comparable>(a: T, b: T) -> T { ... }  // inline form
```

## Never Type

`!` (Never) is the bottom type — the type of an expression that never produces a value because it diverges (runs forever, panics, or exits). A `loop` with no reachable `break` has type `!`:

```gust
let x: ! = loop { };         // runs forever — type is !
let y: ! = loop { panic!(); };
```

Because `!` coerces to every type, it can appear where any type is expected:

```gust
let result: Int = loop { break 42; };  // break gives the loop type Int
let diverge: Int = loop { };           // ! coerces to Int — dead code after
```

`!` is not a type users write in practice; it appears as an inferred type when the typechecker determines a branch or expression cannot return. It is the type of `return` and `panic!` expressions as well.

## Perhaps<T>

`Perhaps<T>` is the built-in optional type. There is no null — all absence is expressed via `Perhaps<T>`.

The type of `nope` is `Perhaps<T>` for some `T` that must be determinable from context. If no context constrains `T` — for example, a bare `let x = nope` with no annotation and no subsequent use that pins the element type — the program is a type error. An explicit annotation is required in that case:

```gust
let x = nope;              // ERROR: cannot infer type of `nope`
let x: Perhaps<Int> = nope; // OK
```

```gust
let result: Perhaps<Int> = nope;
let value: Perhaps<Int> = 42;
```

Use `match` to unwrap safely:

```gust
match find_user(1) {
    Perhaps::Some { value } => println(value.name),
    Perhaps::Nope => println("not found"),
}
```

`.yolo()` unwraps, panicking if the value is `nope`:

```gust
let user = find_user(1).yolo();
```

## Result<T, E>

`Result<T, E>` represents the outcome of a fallible operation:

```gust
fun divide(a: Float, b: Float) -> Result<Float, String> {
    if (b == 0.0) {
        return Result::Err { error: "division by zero" };
    }
    return Result::Ok { value: a / b };
}
```

Use `match` to handle both cases, or `?` to propagate errors.

`.yolo()` also works on `Result<T, E>`, panicking on `Err`.
