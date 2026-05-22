# Runtime

## Panics

A panic is a hard, unrecoverable runtime error. It prints a message and exits the process with a non-zero status. Panics cannot be caught.

Panics are triggered by:
- `.yolo()` on `nope` or a `Result::Err`
- Out-of-bounds array access
- Integer division by zero

## Built-in Functions

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
