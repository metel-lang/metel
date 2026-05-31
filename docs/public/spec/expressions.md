# Expressions

## Pattern Matching

`match` performs exhaustive pattern matching. All cases must be covered.

```metel
fun main() -> Int {
    let value = 1;
    match value {
        1 => 10,
        _ => 0,
    }
}
```

Each arm body can be an expression, a `return`/`break` statement, or a block:

```metel
// Match arm body forms start here.
fun classify(value: Int) -> Int {
    loop {
        break match value {
            0 => 0,
            1 => return 10,
            _ => { 20 },
        };
    }
}

fun main() -> Int {
    return classify(0);
}
```

`match` is an expression — all arms must produce the same type:

```metel
fun main() -> Int {
    let x = 1;
    let label = match x {
        0 => "zero",
        1 => "one",
        _ => "other",
    };
    return label.len();
}
```

Arms with blocks follow the same rules as function bodies: the block's tail expression (if present) is the arm's value; a block with no tail produces `Unit`.

```metel
enum Shape {
    Circle { radius: Float },
    Rectangle { width: Float, height: Float },
}

fun main() -> Int {
    let shape = Shape::Circle { radius: 3.0 };
    let desc: String = match shape {
        Shape::Circle { radius } => {
            let area = radius * radius;
            (area as Int).to_string()
        },
        Shape::Rectangle { width, height } => "rectangle",
    };
    return desc.len();
}
```

### Pattern Kinds

| Pattern | Example | Matches |
|---------|---------|---------|
| Wildcard | `_` | anything, binds nothing |
| Binding | `n` | anything, binds to `n` |
| Literal | `0`, `"hi"`, `true`, `None` | exact value |
| Enum variant | `Direction::North` | unit variant |
| Enum with fields | `Shape::Circle { radius }` | variant, binds fields |
| Tuple | `(a, b)` | tuple, binds elements |
| Guard | `n if n < 0` | binding + boolean condition |

### Examples

```metel
// Pattern examples start here.
enum Shape {
    Circle { radius: Float },
    Rectangle { width: Float, height: Float },
}

fun main() -> Int {
    let shape = Shape::Rectangle { width: 4.0, height: 2.0 };
    let x = -3;
    let point: (Int, Int) = (0, 7);

    let a = match shape {
        Shape::Circle { radius } => radius as Int,
        Shape::Rectangle { width, height } => width as Int,
    };

    let b = match x {
        0          => 0,
        n if n < 0 => 1,
        _          => 2,
    };

    let c = match point {
        (0, 0) => 0,
        (x, 0) => x,
        (0, y) => y,
        (x, y) => x + y,
    };

    return a + b + c;
}
```

---

## Control Flow

### If / Else

```metel
fun main() -> Int {
    let condition = false;
    let other = true;
    if (condition) {
        return 1;
    } else if (other) {
        return 2;
    } else {
        return 3;
    }
}
```

`if` is also an expression (both branches must produce the same type):

```metel
fun main() -> Int {
    let x = 1;
    let label = if (x > 0) { "positive" } else { "non-positive" };
    return label.len();
}
```

**Braceless bodies.** A single expression may be used as the branch body without braces:

```metel
fun print_state() { }

fun main() -> Int {
    let debug = true;
    let flag = false;
    let value_a = 10;
    let value_b = 20;
    if (debug) print_state();
    let x = if (flag) value_a else value_b;
    return x;
}
```

The braceless form desugars to a single-expression block. Three restrictions apply:

1. **Arm style must be consistent.** Both the `then` and `else` arms must use the same style — either both braced or both braceless. Mixing is a parse error.
2. **Dangling-else is forbidden.** If the outer body is braceless, the body expression must not itself be an `if–else`. Use braces on the outer body to resolve the ambiguity.
   ```metel
   fun main() -> Int {
       let a = true;
       let b = false;
       if (a) if (b) { return 1; }
       if (a) { if (b) { return 2; } else { return 3; } }
       return 4;
   }
   ```
   ```metel
   fun main() {
       let a = true;
       let b = false;
       if (a) if (b) { return; } else { return; }
   }
   ```
3. **No semicolon between braceless arms.** Write `if (c) a else b;`, not `if (c) a; else b;` — the `;` terminates the statement before the `else`.

### While

```metel
fun main() -> Int {
    mut n = 3;
    mut total = 0;
    while (n > 0) {
        total += n;
        n -= 1;
    }
    return total;
}
```

### For

```metel
fun main() -> Int {
    mut total = 0;
    for (mut i = 0; i < 4; i += 1) {
        total += i;
    }
    return total;
}
```

### For-In

> **Availability:**
> Array and range iteration: since v0.1.0.
> User-defined `Iterable<T>` implementations: since v0.4.0.

`for-in` works on any type implementing the `Iterable<T>` aspect. The loop variable
receives type `T`. `T[]` (array) and `Range` (produced by `..` and `..=`) implement
`Iterable<T>` by default. User-defined types can be made iterable by implementing
`Iterable<T>`:

```metel
aspect Iterable<T> {
    fun next(mut self) -> Perhaps<T>;
}

fun main() -> Int {
    return 0;
}
```

```metel
fun main() -> Int {
    let collection = [1, 2, 3];
    mut total = 0;
    for (let item in collection) { total += item; }
    for (let i in 0..10) { total += i; }
    for (let i in 0..=10) { total += i; }
    return total;
}
```

### Loop

`loop` creates an infinite loop. It is the only loop form that can produce a value:

```metel
fun main() -> Int {
    let result = loop {
        break 42;
    };
    return result;
}
```

**Typing rules:**

- `loop { break expr; }` has type `T` where `expr: T`. All `break` arms must produce the same type.
- `loop { }` — a loop with no reachable `break` — has type `!` (Never). See [Never Type](types.md#never-type).

### Break and Continue

`break` exits the innermost loop. `break expr` exits a `loop` and produces `expr` as the loop's value.

`continue` skips to the next iteration of the innermost loop.

### Return

```metel
fun returns_unit() {
    return;
}

fun returns_value() -> Int {
    return 42;
}

fun main() -> Int {
    returns_unit();
    return returns_value();
}
```
