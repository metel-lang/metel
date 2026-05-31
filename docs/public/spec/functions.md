# Functions

```metel
fun add(a: Int, b: Int) -> Int {
    return a + b;
}

fun main() -> Int {
    return add(2, 3);
}
```

Parameter type annotations are optional when types can be inferred from context. The return type follows `->` and is also optional — a function with no return annotation and no `return expr;` returns `()`. `return expr;` and bare `return;` are both valid.

## Associated Functions

`impl` blocks may contain functions with no `self` parameter. These are called on the type via `::` syntax and serve as the canonical constructor pattern:

```metel
struct Point {
    x: Float,
    y: Float,
}

impl Point {
    fun new(x: Float, y: Float) -> Point {
        return Point { x: x, y: y };
    }
}

fun main() -> Int {
    let p = Point::new(1.0, 2.0);
    return p.x as Int;
}
```

## First-Class Functions

Functions are first-class values and can be assigned, passed, and returned:

```metel
fun add(a: Int, b: Int) -> Int {
    return a + b;
}

fun apply(f: fun(Int) -> Int, x: Int) -> Int {
    return f(x);
}

fun main() -> Int {
    let f = add;
    let inc = fun(x: Int) -> Int { return x + 1; };
    return f(1, 2) + apply(inc, 4);
}
```

The type of a function or closure is written as `fun(ParamTypes) -> ReturnType`.

## Closures

Anonymous functions are written with `fun` in expression position:

```metel
fun main() -> Int {
    let double = fun(x: Int) -> Int { return x * 2; };
    return double(5);
}
```

Closures capture variables from their enclosing scope. Captured `mut` variables are shared — mutations are visible in the outer scope:

```metel
fun main() -> Int {
    mut count = 0;
    let inc = fun() { count += 1; };
    inc();
    inc();
    return count;
}
```

## The ? Operator

> **Availability:**
> `?` with matching error types: since v0.1.0.
> `?` with `From`-based error coercion: since v0.4.0.

Inside a function returning `Result<T, E>`, `?` propagates errors early:

```metel
fun parse_int(s: String) -> Result<Int, String> {
    if (s == "21") {
        return Result::Ok { value: 21 };
    }
    return Result::Err { error: "not a number" };
}

fun parse_and_double(s: String) -> Result<Int, String> {
    let n = parse_int(s)?;   // returns Err early if parse_int fails
    return Result::Ok { value: n * 2 };
}

fun main() -> Int {
    match parse_and_double("21") {
        Result::Ok { value } => value,
        Result::Err { error } => 0,
    }
}
```

`?` desugars to: if the expression is `Err(e)`, return `Err(E2::from(e))` immediately (where `E2` is the enclosing function's error type); otherwise unwrap to the `Ok` value.

The inner expression's error type `E1` and the function's return error type `E2` must satisfy `E2: From<E1>`. When `E1 == E2` no conversion is performed. When they differ, `From::from` is called automatically on the error value before re-wrapping in `Err`.
