# Type System

Metel is statically and strongly typed. Types are checked at compile time. There are no implicit conversions.

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
- Aspect method signatures

```metel
fun add_annotated(a: Int, b: Int) -> Int { a + b }
fun add_inferred(a, b) { a + b }

fun main() -> Int {
    let x = 42;           // inferred: Int
    let name = "Vlad";    // inferred: String
    let y: Float = 3.14;  // explicit annotation (optional here)
    let total = add_annotated(x, 1) + add_inferred(2, 3);
    if (name == "Vlad") { total + (y as Int) } else { 0 }
}
```

## Tuples

Tuples are lightweight anonymous product types.

```metel
fun main() -> Int {
    let coord: (Int, Int) = (10, 20);
    let triple: (String, Int, Bool) = ("yes", 42, true);
    return coord.0 + triple.1;
}
```

Positional field access uses `.0`, `.1`, etc.:

```metel
fun main() -> Int {
    let coord: (Int, Int) = (10, 20);
    let x = coord.0;
    let y = coord.1;
    return x + y;
}
```

`()` is the zero-element tuple (unit type).

Tuples can be destructured in `match`:

```metel
fun main() -> Int {
    let coord: (Int, Int) = (10, 0);
    match coord {
        (0, y) => y,
        (x, 0) => x,
        (x, y) => x + y,
    }
}
```

## Arrays

`Array<T>` is the built-in ordered sequence type. The shorthand `T[]` is preferred.

```metel
fun main() -> Int {
    let nums: Int[] = [1, 2, 3];
    let names: Array<String> = ["alice", "bob"];
    if (array_len(names) == 2) { nums[0] } else { 0 }
}
```

Index access uses `[]` with an `Int` index. Out-of-bounds access causes a panic.

```metel
fun main() -> Int {
    let nums: Int[] = [1, 2, 3];
    let first = nums[0];
    return first;
}
```

Arrays are usable in `for-in` loops.

## Type Ascription

> **Availability:** Since v0.2.0.

The `:` operator asserts that an expression has a given type without performing any runtime conversion. It is a pure type-inference hint — no code is emitted at runtime.

Type ascription is mainly an ergonomics feature. Most code should type-check from
its surrounding context alone; `:` is for the cases where spelling out the intended
type inline is clearer than introducing a separate annotated binding.

```metel
fun main() -> Int {
    let xs = [] : Int[];
    let x  = 1 : Int;
    if (array_len(xs) == 0) { x } else { 0 }
}
```

Ascription fails at compile time if the inferred type of the sub-expression cannot be unified with the ascribed type. For example, `1 : String` is invalid. Use `as` to convert between types; use `:` only when the value already has the target type.

```metel
fun main() -> Int {
    let y = 1 : String;
    return 0;
}
```

### When ascription helps

Type inference uses surrounding expected types. That expected type can come from a `let` annotation, a function return type, a callee's parameter types, or the surrounding expression context.

Because of that, ambiguous literals like `[]` and `None` often type-check without explicit ascription when the context already determines their type:

```metel
fun zip_lengths(a: Int[], b: String[]) -> Int {
    return array_len(a) + array_len(b);
}

fun make_row(use_default: Bool, fallback: Int[]) -> Int[] {
    return match use_default {
        true  => [],
        false => fallback,
    };
}

fun first_or_default(items: Int[], fallback: Perhaps<Int>) -> Int {
    return match fallback {
        Perhaps::Some { value } => value,
        None => if (array_len(items) > 0) { items[0] } else { 0 },
    };
}

fun main() -> Int {
    let total = zip_lengths([], ["a", "b"]);
    let row = make_row(true, [1, 2, 3]);
    let first = first_or_default([1, 2, 3], None);
    return total + array_len(row) + first;
}
```

Ascription is still useful when no surrounding context fixes the type:

```metel
fun main() -> Int {
    let arr = [] : Int[];
    let value = None : Perhaps<Int>;
    match value {
        Perhaps::Some { value } => value + array_len(arr),
        Perhaps::None => array_len(arr),
    }
}
```

Without such context, ambiguous literals remain a type error. For example, `let x = None;` does not provide enough information to infer the element type.

```metel
fun main() -> Int {
    let x = None;
    return 0;
}
```

## Type Casting

The `as` operator casts between numeric primitive types. It desugars to a call to the `From` aspect and is infallible — the result is the target type directly.

```metel
fun main() -> Int {
    let x: Int = 42;
    let f: Float = x as Float;
    let f2: Float = 3.99;
    let i: Int = f2 as Int;
    return i + (f as Int);
}
```

Allowed primitive casts: `Int` ↔ `Float`.

Because `as` desugars to `From`, user-defined types become castable by implementing `From<SourceType>` for the target type.

## Generics

> **Availability:**
> User-defined generic functions and types: since v0.3.0.
> Built-in generic types (`Perhaps<T>`, `Result<T, E>`, `T[]`): since v0.1.0.

Types and functions can be parameterized with `<T>` syntax.

```metel
struct Stack<T> {
    items: T[],
}

fun first<T>(arr: T[]) -> Perhaps<T> {
    if (array_len(arr) == 0) {
        return None;
    }
    return Perhaps::Some { value: arr[0] };
}

fun main() -> Int {
    let stack = Stack { items: [1, 2, 3] };
    match first(stack.items) {
        Perhaps::Some { value } => value,
        Perhaps::None => 0,
    }
}
```

## Never Type

`!` (Never) is the bottom type — the type of an expression that never produces a value because it diverges (runs forever, panics, or exits). A `loop` with no reachable `break` has type `!`:

```metel
fun main() -> Int {
    let result: Int = loop { break 42; };
    return result;
}
```

`!` is not a type users write in practice; it appears as an inferred type when the typechecker determines a branch or expression cannot return. It is the type of `return`, `panic!`, and `loop { }` with no reachable `break`.

## `Perhaps<T>`

`Perhaps<T>` is the built-in optional type. There is no null — all absence is expressed via `Perhaps<T>`.

The type of `None` is `Perhaps<T>` for some `T` that must be determinable from context. If no context constrains `T` — for example, a bare `let x = None` with no annotation and no subsequent use that pins the element type — the program is a type error. An explicit annotation is required in that case:

```metel
fun main() -> Int {
    let x: Perhaps<Int> = None;
    match x {
        Perhaps::Some { value } => value,
        Perhaps::None => 0,
    }
}
```

```metel
fun main() -> Int {
    let result: Perhaps<Int> = None;
    let value: Perhaps<Int> = 42;
    match value {
        Perhaps::Some { value } => value,
        Perhaps::None => match result {
            Perhaps::Some { value } => value,
            Perhaps::None => 0,
        },
    }
}
```

Use `match` to unwrap safely:

```metel
struct User {
    id: Int,
}

fun find_user(id: Int) -> Perhaps<User> {
    if (id == 1) {
        return Perhaps::Some { value: User { id: 1 } };
    }
    return None;
}

fun main() -> Int {
    match find_user(1) {
        Perhaps::Some { value } => value.id,
        Perhaps::None => 0,
    }
}
```

`.yolo()` unwraps, panicking if the value is `None`:

```metel
struct User {
    id: Int,
}

fun find_user(id: Int) -> Perhaps<User> {
    if (id == 1) {
        return Perhaps::Some { value: User { id: 1 } };
    }
    return None;
}

fun main() -> Int {
    let user = find_user(1).yolo();
    return user.id;
}
```

## `Result<T, E>`

`Result<T, E>` represents the outcome of a fallible operation:

```metel
fun divide(a: Float, b: Float) -> Result<Float, String> {
    if (b == 0.0) {
        return Result::Err { error: "division by zero" };
    }
    return Result::Ok { value: a / b };
}

fun main() -> Int {
    match divide(8.0, 2.0) {
        Result::Ok { value } => value as Int,
        Result::Err { error } => 0,
    }
}
```

Use `match` to handle both cases, or `?` to propagate errors.

`.yolo()` also works on `Result<T, E>`, panicking on `Err`.
