use crate::error::{MetelError, RuntimeErrorCode};

use super::display::{format_float, format_value, value_to_display_string};
use super::{RuntimeMethodKey, RuntimeRegistry, Value};

fn numeric_as_i128(v: &Value) -> Option<i128> {
    match v {
        Value::I8(n) => Some(*n as i128),
        Value::I16(n) => Some(*n as i128),
        Value::I32(n) => Some(*n as i128),
        Value::I64(n) => Some(*n as i128),
        Value::U8(n) => Some(*n as i128),
        Value::U16(n) => Some(*n as i128),
        Value::U32(n) => Some(*n as i128),
        Value::U64(n) => Some(*n as i128),
        Value::F32(f) => Some(*f as i128),
        Value::F64(f) => Some(*f as i128),
        _ => None,
    }
}

fn numeric_as_f64_val(v: &Value) -> Option<f64> {
    match v {
        Value::I8(n) => Some(*n as f64),
        Value::I16(n) => Some(*n as f64),
        Value::I32(n) => Some(*n as f64),
        Value::I64(n) => Some(*n as f64),
        Value::U8(n) => Some(*n as f64),
        Value::U16(n) => Some(*n as f64),
        Value::U32(n) => Some(*n as f64),
        Value::U64(n) => Some(*n as f64),
        Value::F32(f) => Some(*f as f64),
        Value::F64(f) => Some(*f),
        _ => None,
    }
}

/// The free-function builtin names registered by this module.
/// Must stay in sync with `StdPrelude::schemes()`. See METEL-5 / ADR-0027.
#[allow(dead_code)] // called by the parity test in typechecker::tests
pub(crate) fn free_function_names() -> std::collections::HashSet<&'static str> {
    [
        "print",
        "println",
        "string_len",
        "string_concat",
        "List::new",
        "List::from",
        "clock",
        "assert",
        "assert_msg",
        "dbg",
    ]
    .into_iter()
    .collect()
}

pub(super) fn register_builtins(runtime: &mut RuntimeRegistry) {
    macro_rules! register_global {
        ($name:expr, $value:expr) => {
            runtime.register_global($name.to_string(), $value);
        };
    }
    macro_rules! register_regular {
        ($type_name:expr, $method_name:expr, $value:expr) => {
            runtime.register_method(RuntimeMethodKey::regular($type_name, $method_name), $value);
        };
    }
    macro_rules! register_from {
        ($target:expr, $source:expr, $value:expr) => {
            runtime.register_method(RuntimeMethodKey::from_impl($target, $source), $value);
        };
    }

    // print/println dispatch through Display (to_string) for any type.
    register_global!(
        "print",
        Value::Builtin("print".to_string(), |args, span| {
            let s = match args.first() {
                Some(v) => value_to_display_string(v).ok_or_else(|| {
                    MetelError::panic(
                        RuntimeErrorCode::R0009,
                        "print: value does not implement Display",
                        span,
                    )
                })?,
                None => return Err(MetelError::internal("print: expected one argument")),
            };
            print!("{s}");
            Ok(Value::Unit)
        })
    );

    register_global!(
        "println",
        Value::Builtin("println".to_string(), |args, span| {
            let s = match args.first() {
                Some(v) => value_to_display_string(v).ok_or_else(|| {
                    MetelError::panic(
                        RuntimeErrorCode::R0009,
                        "println: value does not implement Display",
                        span,
                    )
                })?,
                None => return Err(MetelError::internal("println: expected one argument")),
            };
            println!("{s}");
            Ok(Value::Unit)
        })
    );

    // to_string() methods for built-in Display types.
    register_regular!(
        "i64",
        "to_string",
        Value::Builtin("i64::to_string".to_string(), |args, _span| {
            match args.first() {
                Some(Value::I64(n)) => Ok(Value::Str(n.to_string())),
                _ => Err(MetelError::internal("i64::to_string: expected i64")),
            }
        })
    );
    register_regular!(
        "f64",
        "to_string",
        Value::Builtin("f64::to_string".to_string(), |args, _span| {
            match args.first() {
                Some(Value::F64(f)) => Ok(Value::Str(format_float(*f))),
                _ => Err(MetelError::internal("f64::to_string: expected f64")),
            }
        })
    );
    register_regular!(
        "boolean",
        "to_string",
        Value::Builtin("boolean::to_string".to_string(), |args, _span| {
            match args.first() {
                Some(Value::Boolean(b)) => {
                    Ok(Value::Str(if *b { "true" } else { "false" }.to_string()))
                }
                _ => Err(MetelError::internal("boolean::to_string: expected boolean")),
            }
        })
    );
    register_regular!(
        "Char",
        "to_string",
        Value::Builtin("Char::to_string".to_string(), |args, _span| {
            match args.first() {
                Some(Value::Char(c)) => Ok(Value::Str(c.to_string())),
                _ => Err(MetelError::internal("Char::to_string: expected Char")),
            }
        })
    );
    register_regular!(
        "String",
        "to_string",
        Value::Builtin("String::to_string".to_string(), |args, _span| {
            match args.first() {
                Some(Value::Str(s)) => Ok(Value::Str(s.clone())),
                _ => Err(MetelError::internal("String::to_string: expected String")),
            }
        })
    );

    // Numeric From impls: full cross-product of all numeric types.
    // Fallback generic builtins (for any remaining dispatch paths).
    register_regular!(
        "i64",
        "from",
        Value::Builtin("i64::from".to_string(), |args, _span| {
            match args.first().and_then(|v| numeric_as_i128(v)) {
                Some(n) => Ok(Value::I64(n as i64)),
                None => Err(MetelError::internal("i64::from: expected numeric")),
            }
        })
    );
    register_regular!(
        "f64",
        "from",
        Value::Builtin("f64::from".to_string(), |args, _span| {
            match args.first().and_then(|v| numeric_as_f64_val(v)) {
                Some(f) => Ok(Value::F64(f)),
                None => Err(MetelError::internal("f64::from: expected numeric")),
            }
        })
    );

    // Specific-key From impls (evaluated before generic fallbacks).
    macro_rules! from_int {
        ($target:literal, $source:literal, $key:literal, $out:expr) => {
            register_from!(
                $target,
                $source,
                Value::Builtin($key.to_string(), |args, _span| {
                    match args.first().and_then(|v| numeric_as_i128(v)) {
                        Some(n) => Ok($out(n)),
                        None => Err(MetelError::internal(concat!($key, ": unexpected argument"))),
                    }
                })
            );
        };
    }
    macro_rules! from_float {
        ($target:literal, $source:literal, $key:literal, $out:expr) => {
            register_from!(
                $target,
                $source,
                Value::Builtin($key.to_string(), |args, _span| {
                    match args.first().and_then(|v| numeric_as_f64_val(v)) {
                        Some(f) => Ok($out(f)),
                        None => Err(MetelError::internal(concat!($key, ": unexpected argument"))),
                    }
                })
            );
        };
    }

    // i8
    from_int!("i8", "i16", "i8::From<i16>::from", |n: i128| Value::I8(
        n as i8
    ));
    from_int!("i8", "i32", "i8::From<i32>::from", |n: i128| Value::I8(
        n as i8
    ));
    from_int!("i8", "i64", "i8::From<i64>::from", |n: i128| Value::I8(
        n as i8
    ));
    from_int!("i8", "u8", "i8::From<u8>::from", |n: i128| Value::I8(
        n as i8
    ));
    from_int!("i8", "u16", "i8::From<u16>::from", |n: i128| Value::I8(
        n as i8
    ));
    from_int!("i8", "u32", "i8::From<u32>::from", |n: i128| Value::I8(
        n as i8
    ));
    from_int!("i8", "u64", "i8::From<u64>::from", |n: i128| Value::I8(
        n as i8
    ));
    from_int!("i8", "f32", "i8::From<f32>::from", |n: i128| Value::I8(
        n as i8
    ));
    from_int!("i8", "f64", "i8::From<f64>::from", |n: i128| Value::I8(
        n as i8
    ));
    // i16
    from_int!("i16", "i8", "i16::From<i8>::from", |n: i128| Value::I16(
        n as i16
    ));
    from_int!("i16", "i32", "i16::From<i32>::from", |n: i128| Value::I16(
        n as i16
    ));
    from_int!("i16", "i64", "i16::From<i64>::from", |n: i128| Value::I16(
        n as i16
    ));
    from_int!("i16", "u8", "i16::From<u8>::from", |n: i128| Value::I16(
        n as i16
    ));
    from_int!("i16", "u16", "i16::From<u16>::from", |n: i128| Value::I16(
        n as i16
    ));
    from_int!("i16", "u32", "i16::From<u32>::from", |n: i128| Value::I16(
        n as i16
    ));
    from_int!("i16", "u64", "i16::From<u64>::from", |n: i128| Value::I16(
        n as i16
    ));
    from_int!("i16", "f32", "i16::From<f32>::from", |n: i128| Value::I16(
        n as i16
    ));
    from_int!("i16", "f64", "i16::From<f64>::from", |n: i128| Value::I16(
        n as i16
    ));
    // i32
    from_int!("i32", "i8", "i32::From<i8>::from", |n: i128| Value::I32(
        n as i32
    ));
    from_int!("i32", "i16", "i32::From<i16>::from", |n: i128| Value::I32(
        n as i32
    ));
    from_int!("i32", "i64", "i32::From<i64>::from", |n: i128| Value::I32(
        n as i32
    ));
    from_int!("i32", "u8", "i32::From<u8>::from", |n: i128| Value::I32(
        n as i32
    ));
    from_int!("i32", "u16", "i32::From<u16>::from", |n: i128| Value::I32(
        n as i32
    ));
    from_int!("i32", "u32", "i32::From<u32>::from", |n: i128| Value::I32(
        n as i32
    ));
    from_int!("i32", "u64", "i32::From<u64>::from", |n: i128| Value::I32(
        n as i32
    ));
    from_int!("i32", "f32", "i32::From<f32>::from", |n: i128| Value::I32(
        n as i32
    ));
    from_int!("i32", "f64", "i32::From<f64>::from", |n: i128| Value::I32(
        n as i32
    ));
    // i64
    from_int!("i64", "i8", "i64::From<i8>::from", |n: i128| Value::I64(
        n as i64
    ));
    from_int!("i64", "i16", "i64::From<i16>::from", |n: i128| Value::I64(
        n as i64
    ));
    from_int!("i64", "i32", "i64::From<i32>::from", |n: i128| Value::I64(
        n as i64
    ));
    from_int!("i64", "u8", "i64::From<u8>::from", |n: i128| Value::I64(
        n as i64
    ));
    from_int!("i64", "u16", "i64::From<u16>::from", |n: i128| Value::I64(
        n as i64
    ));
    from_int!("i64", "u32", "i64::From<u32>::from", |n: i128| Value::I64(
        n as i64
    ));
    from_int!("i64", "u64", "i64::From<u64>::from", |n: i128| Value::I64(
        n as i64
    ));
    from_int!("i64", "f32", "i64::From<f32>::from", |n: i128| Value::I64(
        n as i64
    ));
    from_int!("i64", "f64", "i64::From<f64>::from", |n: i128| Value::I64(
        n as i64
    ));
    // u8
    from_int!("u8", "i8", "u8::From<i8>::from", |n: i128| Value::U8(
        n as u8
    ));
    from_int!("u8", "i16", "u8::From<i16>::from", |n: i128| Value::U8(
        n as u8
    ));
    from_int!("u8", "i32", "u8::From<i32>::from", |n: i128| Value::U8(
        n as u8
    ));
    from_int!("u8", "i64", "u8::From<i64>::from", |n: i128| Value::U8(
        n as u8
    ));
    from_int!("u8", "u16", "u8::From<u16>::from", |n: i128| Value::U8(
        n as u8
    ));
    from_int!("u8", "u32", "u8::From<u32>::from", |n: i128| Value::U8(
        n as u8
    ));
    from_int!("u8", "u64", "u8::From<u64>::from", |n: i128| Value::U8(
        n as u8
    ));
    from_int!("u8", "f32", "u8::From<f32>::from", |n: i128| Value::U8(
        n as u8
    ));
    from_int!("u8", "f64", "u8::From<f64>::from", |n: i128| Value::U8(
        n as u8
    ));
    // u16
    from_int!("u16", "i8", "u16::From<i8>::from", |n: i128| Value::U16(
        n as u16
    ));
    from_int!("u16", "i16", "u16::From<i16>::from", |n: i128| Value::U16(
        n as u16
    ));
    from_int!("u16", "i32", "u16::From<i32>::from", |n: i128| Value::U16(
        n as u16
    ));
    from_int!("u16", "i64", "u16::From<i64>::from", |n: i128| Value::U16(
        n as u16
    ));
    from_int!("u16", "u8", "u16::From<u8>::from", |n: i128| Value::U16(
        n as u16
    ));
    from_int!("u16", "u32", "u16::From<u32>::from", |n: i128| Value::U16(
        n as u16
    ));
    from_int!("u16", "u64", "u16::From<u64>::from", |n: i128| Value::U16(
        n as u16
    ));
    from_int!("u16", "f32", "u16::From<f32>::from", |n: i128| Value::U16(
        n as u16
    ));
    from_int!("u16", "f64", "u16::From<f64>::from", |n: i128| Value::U16(
        n as u16
    ));
    // u32
    from_int!("u32", "i8", "u32::From<i8>::from", |n: i128| Value::U32(
        n as u32
    ));
    from_int!("u32", "i16", "u32::From<i16>::from", |n: i128| Value::U32(
        n as u32
    ));
    from_int!("u32", "i32", "u32::From<i32>::from", |n: i128| Value::U32(
        n as u32
    ));
    from_int!("u32", "i64", "u32::From<i64>::from", |n: i128| Value::U32(
        n as u32
    ));
    from_int!("u32", "u8", "u32::From<u8>::from", |n: i128| Value::U32(
        n as u32
    ));
    from_int!("u32", "u16", "u32::From<u16>::from", |n: i128| Value::U32(
        n as u32
    ));
    from_int!("u32", "u64", "u32::From<u64>::from", |n: i128| Value::U32(
        n as u32
    ));
    from_int!("u32", "f32", "u32::From<f32>::from", |n: i128| Value::U32(
        n as u32
    ));
    from_int!("u32", "f64", "u32::From<f64>::from", |n: i128| Value::U32(
        n as u32
    ));
    // u64
    from_int!("u64", "i8", "u64::From<i8>::from", |n: i128| Value::U64(
        n as u64
    ));
    from_int!("u64", "i16", "u64::From<i16>::from", |n: i128| Value::U64(
        n as u64
    ));
    from_int!("u64", "i32", "u64::From<i32>::from", |n: i128| Value::U64(
        n as u64
    ));
    from_int!("u64", "i64", "u64::From<i64>::from", |n: i128| Value::U64(
        n as u64
    ));
    from_int!("u64", "u8", "u64::From<u8>::from", |n: i128| Value::U64(
        n as u64
    ));
    from_int!("u64", "u16", "u64::From<u16>::from", |n: i128| Value::U64(
        n as u64
    ));
    from_int!("u64", "u32", "u64::From<u32>::from", |n: i128| Value::U64(
        n as u64
    ));
    from_int!("u64", "f32", "u64::From<f32>::from", |n: i128| Value::U64(
        n as u64
    ));
    from_int!("u64", "f64", "u64::From<f64>::from", |n: i128| Value::U64(
        n as u64
    ));
    // f32
    from_float!("f32", "i8", "f32::From<i8>::from", |f: f64| Value::F32(
        f as f32
    ));
    from_float!("f32", "i16", "f32::From<i16>::from", |f: f64| Value::F32(
        f as f32
    ));
    from_float!("f32", "i32", "f32::From<i32>::from", |f: f64| Value::F32(
        f as f32
    ));
    from_float!("f32", "i64", "f32::From<i64>::from", |f: f64| Value::F32(
        f as f32
    ));
    from_float!("f32", "u8", "f32::From<u8>::from", |f: f64| Value::F32(
        f as f32
    ));
    from_float!("f32", "u16", "f32::From<u16>::from", |f: f64| Value::F32(
        f as f32
    ));
    from_float!("f32", "u32", "f32::From<u32>::from", |f: f64| Value::F32(
        f as f32
    ));
    from_float!("f32", "u64", "f32::From<u64>::from", |f: f64| Value::F32(
        f as f32
    ));
    from_float!("f32", "f64", "f32::From<f64>::from", |f: f64| Value::F32(
        f as f32
    ));
    // f64
    from_float!("f64", "i8", "f64::From<i8>::from", |f: f64| Value::F64(f));
    from_float!("f64", "i16", "f64::From<i16>::from", |f: f64| Value::F64(f));
    from_float!("f64", "i32", "f64::From<i32>::from", |f: f64| Value::F64(f));
    from_float!("f64", "i64", "f64::From<i64>::from", |f: f64| Value::F64(f));
    from_float!("f64", "u8", "f64::From<u8>::from", |f: f64| Value::F64(f));
    from_float!("f64", "u16", "f64::From<u16>::from", |f: f64| Value::F64(f));
    from_float!("f64", "u32", "f64::From<u32>::from", |f: f64| Value::F64(f));
    from_float!("f64", "u64", "f64::From<u64>::from", |f: f64| Value::F64(f));
    from_float!("f64", "f32", "f64::From<f32>::from", |f: f64| Value::F64(f));

    register_from!(
        "u32",
        "Char",
        Value::Builtin("u32::From<Char>::from".to_string(), |args, _span| {
            match args.first() {
                Some(Value::Char(c)) => Ok(Value::U32(*c as u32)),
                _ => Err(MetelError::internal("u32::From<Char>::from: expected Char")),
            }
        })
    );
    register_from!(
        "Char",
        "u32",
        Value::Builtin("Char::From<u32>::from".to_string(), |args, span| {
            match args.first() {
                Some(Value::U32(n)) => char::from_u32(*n).map(Value::Char).ok_or_else(|| {
                    MetelError::panic(
                        RuntimeErrorCode::R0009,
                        format!("u32 value {n} is not a valid Unicode scalar"),
                        span,
                    )
                }),
                _ => Err(MetelError::internal("Char::From<u32>::from: expected u32")),
            }
        })
    );

    register_global!(
        "string_len",
        Value::Builtin("string_len".to_string(), |args, _span| {
            if let Some(Value::Str(s)) = args.first() {
                Ok(Value::I64(s.chars().count() as i64))
            } else {
                Err(MetelError::internal("string_len: expected String argument"))
            }
        })
    );

    register_global!(
        "string_concat",
        Value::Builtin("string_concat".to_string(), |args, _span| {
            match (args.first(), args.get(1)) {
                (Some(Value::Str(a)), Some(Value::Str(b))) => Ok(Value::Str(a.clone() + b)),
                _ => Err(MetelError::internal(
                    "string_concat: expected two String arguments",
                )),
            }
        })
    );

    // ── List<T> constructors ──────────────────────────────────────────────────

    // Helper: build a List Value from a backing Rc array.
    // List<T> is represented as Value::Struct { name: "List", fields: { "inner": Value::Array(rc) } }

    register_global!(
        "List::new",
        Value::Builtin("List::new".to_string(), |_args, _span| {
            use std::cell::RefCell;
            use std::rc::Rc;
            let mut fields = std::collections::HashMap::new();
            fields.insert(
                "inner".to_string(),
                Value::Array(Rc::new(RefCell::new(vec![]))),
            );
            Ok(Value::Struct {
                name: "List".to_string(),
                fields,
            })
        })
    );

    register_global!(
        "List::from",
        Value::Builtin("List::from".to_string(), |args, _span| {
            use std::cell::RefCell;
            use std::rc::Rc;
            match args.first() {
                Some(Value::Array(src)) => {
                    let copy = src.borrow().clone();
                    let mut fields = std::collections::HashMap::new();
                    fields.insert(
                        "inner".to_string(),
                        Value::Array(Rc::new(RefCell::new(copy))),
                    );
                    Ok(Value::Struct {
                        name: "List".to_string(),
                        fields,
                    })
                }
                _ => Err(MetelError::internal("List::from: expected array argument")),
            }
        })
    );

    // ── List<T> instance methods (keyed as "List::method") ───────────────────

    register_regular!(
        "List",
        "push",
        Value::Builtin("List::push".to_string(), |args, _span| {
            match (args.first(), args.get(1)) {
                (Some(Value::Struct { name, fields }), Some(val)) if name == "List" => {
                    if let Some(Value::Array(arr)) = fields.get("inner") {
                        arr.borrow_mut().push(val.clone());
                        Ok(Value::Unit)
                    } else {
                        Err(MetelError::internal("List::push: missing inner field"))
                    }
                }
                _ => Err(MetelError::internal("List::push: expected (List, T)")),
            }
        })
    );

    register_regular!(
        "List",
        "pop",
        Value::Builtin("List::pop".to_string(), |args, span| {
            match args.first() {
                Some(Value::Struct { name, fields }) if name == "List" => {
                    if let Some(Value::Array(arr)) = fields.get("inner") {
                        match arr.borrow_mut().pop() {
                            Some(val) => {
                                let mut f = std::collections::HashMap::new();
                                f.insert("value".to_string(), val);
                                Ok(Value::Enum {
                                    name: "Perhaps".to_string(),
                                    variant: "Some".to_string(),
                                    fields: f,
                                })
                            }
                            None => Ok(Value::Enum {
                                name: "Perhaps".to_string(),
                                variant: "None".to_string(),
                                fields: std::collections::HashMap::new(),
                            }),
                        }
                    } else {
                        Err(MetelError::internal("List::pop: missing inner field"))
                    }
                }
                _ => Err(MetelError::panic(
                    RuntimeErrorCode::R0009,
                    "List::pop: expected List",
                    span,
                )),
            }
        })
    );

    register_regular!(
        "List",
        "len",
        Value::Builtin("List::len".to_string(), |args, _span| {
            match args.first() {
                Some(Value::Struct { name, fields }) if name == "List" => {
                    if let Some(Value::Array(arr)) = fields.get("inner") {
                        Ok(Value::I64(arr.borrow().len() as i64))
                    } else {
                        Err(MetelError::internal("List::len: missing inner field"))
                    }
                }
                _ => Err(MetelError::internal("List::len: expected List")),
            }
        })
    );

    register_regular!(
        "List",
        "get",
        Value::Builtin("List::get".to_string(), |args, _span| {
            match (args.first(), args.get(1)) {
                (Some(Value::Struct { name, fields }), Some(Value::I64(idx))) if name == "List" => {
                    if let Some(Value::Array(arr)) = fields.get("inner") {
                        match arr.borrow().get(*idx as usize).cloned() {
                            Some(val) => {
                                let mut f = std::collections::HashMap::new();
                                f.insert("value".to_string(), val);
                                Ok(Value::Enum {
                                    name: "Perhaps".to_string(),
                                    variant: "Some".to_string(),
                                    fields: f,
                                })
                            }
                            None => Ok(Value::Enum {
                                name: "Perhaps".to_string(),
                                variant: "None".to_string(),
                                fields: std::collections::HashMap::new(),
                            }),
                        }
                    } else {
                        Err(MetelError::internal("List::get: missing inner field"))
                    }
                }
                _ => Err(MetelError::internal("List::get: expected (List, i64)")),
            }
        })
    );

    register_regular!(
        "List",
        "as_slice",
        Value::Builtin("List::as_slice".to_string(), |args, _span| {
            match args.first() {
                Some(Value::Struct { name, fields }) if name == "List" => {
                    if let Some(arr) = fields.get("inner") {
                        Ok(arr.clone())
                    } else {
                        Err(MetelError::internal("List::as_slice: missing inner field"))
                    }
                }
                _ => Err(MetelError::internal("List::as_slice: expected List")),
            }
        })
    );

    register_global!(
        "clock",
        Value::Builtin("clock".to_string(), |_args, _span| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;
            Ok(Value::I64(ms))
        })
    );

    register_global!(
        "assert",
        Value::Builtin("assert".to_string(), |args, span| {
            match args.first() {
                Some(Value::Boolean(true)) => Ok(Value::Unit),
                Some(Value::Boolean(false)) => Err(MetelError::panic(
                    RuntimeErrorCode::R0013,
                    "assertion failed",
                    span,
                )),
                _ => Err(MetelError::internal("assert: expected boolean argument")),
            }
        })
    );

    register_global!(
        "assert_msg",
        Value::Builtin("assert_msg".to_string(), |args, span| {
            match (args.first(), args.get(1)) {
                (Some(Value::Boolean(true)), _) => Ok(Value::Unit),
                (Some(Value::Boolean(false)), Some(Value::Str(msg))) => Err(MetelError::panic(
                    RuntimeErrorCode::R0013,
                    msg.clone(),
                    span,
                )),
                (Some(Value::Boolean(false)), _) => Err(MetelError::panic(
                    RuntimeErrorCode::R0013,
                    "assertion failed",
                    span,
                )),
                _ => Err(MetelError::internal(
                    "assert_msg: expected (boolean, String) arguments",
                )),
            }
        })
    );

    register_global!(
        "dbg",
        Value::Builtin("dbg".to_string(), |args, _span| {
            if let Some(val) = args.first() {
                eprintln!("[dbg] {}", format_value(val));
                Ok(val.clone())
            } else {
                Err(MetelError::internal("dbg: expected one argument"))
            }
        })
    );
}

pub(super) fn runtime_registry() -> RuntimeRegistry {
    let mut runtime = RuntimeRegistry::new();
    register_builtins(&mut runtime);
    runtime
}
