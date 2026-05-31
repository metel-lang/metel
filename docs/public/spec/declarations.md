# Declarations

`pub` may be prefixed to any top-level `fun`, `struct`, `enum`, or `aspect` declaration to mark it as accessible from other modules. See [Modules — Visibility](modules.md#visibility) for the full rules.

## Variables

### Immutable Bindings

```metel
fun main() -> Int {
    let x = 42;
    let name: String = "Vlad";
    if (name == "Vlad") { return x; }
    return 0;
}
```

`let` bindings cannot be reassigned and must always be initialized.

### Mutable Bindings

```metel
fun main() -> Int {
    mut counter = 0;
    counter = counter + 1;
    counter += 1;
    return counter;
}
```

`mut` bindings can be reassigned and also must be initialized at declaration. Compound assignment operators `+=`, `-=`, `*=`, `/=`, `%=` are supported.

### Scoping and Shadowing

Variables are lexically scoped. Each block `{ }` introduces a new scope. Inner scopes can shadow outer variables.

`let` and `mut` declarations are sequential — a binding is visible only from its declaration point to the end of its containing block.

`fun` declarations are hoisted to the top of their containing block. All `fun` declarations in a block are mutually visible to each other and to all other statements in that block, regardless of declaration order. This enables forward references and mutual recursion at any nesting level.

Hoisting is block-local: a `fun` declared in an inner block is not visible in the outer block. Normal lexical scoping applies across block boundaries — inner blocks see outer declarations, outer blocks do not see inner declarations.

```metel
fun is_even(n: Int) -> Bool {
    if (n == 0) { return true; }
    return is_odd(n - 1);
}

fun is_odd(n: Int) -> Bool {
    if (n == 0) { return false; }
    return is_even(n - 1);
}

fun outer() -> Int {
    inner();

    fun inner() {
        helper();
        fun helper() { }
    }

    return 1;
}

fun main() -> Int {
    if (is_odd(3)) { return outer(); }
    return 0;
}
```

An inner function remains scoped to its own block. For example, `helper();` is valid inside `inner()`, but calling `helper();` from `outer()` is a type error.

```metel
fun outer() {
    fun inner() {
        fun helper() { }
        helper();
    }

    helper();
}

fun main() {
    outer();
}
```

Top-level `struct` and `enum` declarations are hoisted to program scope — they may be
referenced before their declaration appears in the source.

Types declared inside a function body are local to that body from their declaration
point onward; they are not visible from other functions.

```metel
fun make_point() -> Point {
    return Point { x: 1.0, y: 2.0 };   // OK — Point is globally visible
}

struct Point {
    x: Float,
    y: Float,
}

fun inner() {
    struct LocalPoint {
        x: Float,
        y: Float,
    }
    let p = LocalPoint { x: 1.0, y: 2.0 };
}

fun main() -> Int {
    inner();
    let p = make_point();
    return p.x as Int;
}
```

Top-level `impl` blocks follow the same declaration-order rule as the types they extend.

---

## Structs

```metel
struct Point {
    x: Float,
    y: Float,
}

fun main() -> Int {
    let p = Point { x: 1.0, y: 2.0 };
    return p.y as Int;
}
```

### Instantiation and Field Access

```metel
struct Point {
    x: Float,
    y: Float,
}

fun main() -> Int {
    let p = Point { x: 1.0, y: 2.0 };
    let x = p.x;
    return x as Int;
}
```

When a local variable has the same name as a field, the `: value` part can be omitted (**shorthand field init**):

```metel
struct Point {
    x: Float,
    y: Float,
}

fun main() -> Int {
    let x = 1.0;
    let y = 2.0;
    let p = Point { x, y };
    return p.x as Int;
}
```

Shorthand and explicit fields may be mixed freely within one literal.

### Methods

```metel
struct Point {
    x: Float,
    y: Float,
}

impl Point {
    fun distance(self, other: Point) -> Float {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        return dx * dx + dy * dy;   // squared distance
    }
}

fun main() -> Int {
    let p = Point { x: 1.0, y: 2.0 };
    let q = Point { x: 4.0, y: 6.0 };
    let d = p.distance(q);
    return d as Int;
}
```

`self` refers to the receiver. Methods are called with dot syntax.

### Mutable Receiver

> **Availability:** Since v0.1.0.

Methods that mutate the receiver declare `mut self`.

`mut self` gives the method a mutable local receiver value, but method calls do not
update the caller's binding in place.

```metel
struct Counter {
    value: Int,
}

impl Counter {
    fun increment(mut self) {
        self.value += 1;
    }
}

fun main() -> Int {
    let c = Counter { value: 1 };
    c.increment();
    return c.value;
}
```

### Generic Structs

```metel
struct Pair<A, B> {
    first: A,
    second: B,
}

fun main() -> Int {
    let p = Pair { first: 1, second: true };
    return p.first;
}
```

---

## Enums

```metel
enum Direction { North, South, East, West }

enum Shape {
    Circle { radius: Float },
    Rectangle { width: Float, height: Float },
}

fun main() -> Int {
    let dir = Direction::North;
    let s = Shape::Circle { radius: 5.0 };
    match dir {
        Direction::North => s.radius as Int,
        Direction::South => 0,
        Direction::East => 0,
        Direction::West => 0,
    }
}
```

Variants may be unit (no data) or struct-like (named fields).

### Instantiation

```metel
enum Direction { North, South, East, West }

enum Shape {
    Circle { radius: Float },
    Rectangle { width: Float, height: Float },
}

fun main() -> Int {
    let dir = Direction::North;
    let s = Shape::Circle { radius: 5.0 };
    match dir {
        Direction::North => s.radius as Int,
        Direction::South => 0,
        Direction::East => 0,
        Direction::West => 0,
    }
}
```

### Methods on Enums

`impl` blocks on enums follow the same syntax as structs:

```metel
enum Shape {
    Circle { radius: Float },
    Rectangle { width: Float, height: Float },
}

impl Shape {
    fun area(self) -> Float {
        match self {
            Shape::Circle { radius } => 3.14159 * radius * radius,
            Shape::Rectangle { width, height } => width * height,
        }
    }
}

fun main() -> Int {
    let s = Shape::Circle { radius: 5.0 };
    return s.area() as Int;
}
```

---

## Aspects

> **Availability:** Since v0.4.0.

```metel
aspect Printable {
    fun print(self);
}

aspect Comparable {
    fun compare(self, other: Self) -> Int;
}

fun main() -> Int {
    return 0;
}
```

### Implementing a Aspect

```metel
struct Point {
    x: Float,
    y: Float,
}

aspect Printable {
    fun print(self);
}

impl Printable for Point {
    fun print(self) {
        print("(");
        print(self.x.to_string());
        print(", ");
        print(self.y.to_string());
        println(")");
    }
}

fun main() {
    let p = Point { x: 1.0, y: 2.0 };
    p.print();
}
```

### The Self Type

`Self` inside a aspect definition refers to the concrete implementing type:

```metel
aspect Comparable {
    fun compare(self, other: Self) -> Int;
}

fun main() -> Int {
    return 0;
}
```

### Static Dispatch Only

Aspect objects (`dyn Aspect`) are not part of the language. All polymorphism is via generics (static dispatch).
