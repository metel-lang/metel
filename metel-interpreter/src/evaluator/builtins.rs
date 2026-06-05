use crate::error::{MetelError, RuntimeErrorCode};

use super::{Environment, Value};
use super::display::{format_float, format_value, value_to_display_string};

/// The free-function builtin names registered by this module.
/// Must stay in sync with `StdPrelude::schemes()`. See METEL-5 / ADR-0027.
pub(crate) fn free_function_names() -> std::collections::HashSet<&'static str> {
    [
        "print", "println", "string_len", "string_concat",
        "List::new", "List::from", "clock", "assert", "assert_msg", "dbg",
    ].into_iter().collect()
}

pub(super) fn register_builtins(env: &mut Environment) {
    // print/println dispatch through Display (to_string) for any type.
    env.define("print", Value::Builtin("print".to_string(), |args, span| {
        let s = match args.first() {
            Some(v) => value_to_display_string(v).ok_or_else(|| {
                MetelError::panic(RuntimeErrorCode::R0009, "print: value does not implement Display", span)
            })?,
            None => return Err(MetelError::internal("print: expected one argument")),
        };
        print!("{s}");
        Ok(Value::Unit)
    }));

    env.define("println", Value::Builtin("println".to_string(), |args, span| {
        let s = match args.first() {
            Some(v) => value_to_display_string(v).ok_or_else(|| {
                MetelError::panic(RuntimeErrorCode::R0009, "println: value does not implement Display", span)
            })?,
            None => return Err(MetelError::internal("println: expected one argument")),
        };
        println!("{s}");
        Ok(Value::Unit)
    }));

    // to_string() methods for built-in Display types.
    env.define("i64::to_string", Value::Builtin("i64::to_string".to_string(), |args, _span| {
        match args.first() {
            Some(Value::I64(n)) => Ok(Value::Str(n.to_string())),
            _ => Err(MetelError::internal("i64::to_string: expected i64")),
        }
    }));
    env.define("f64::to_string", Value::Builtin("f64::to_string".to_string(), |args, _span| {
        match args.first() {
            Some(Value::F64(f)) => Ok(Value::Str(format_float(*f))),
            _ => Err(MetelError::internal("f64::to_string: expected f64")),
        }
    }));
    env.define("Bool::to_string", Value::Builtin("Bool::to_string".to_string(), |args, _span| {
        match args.first() {
            Some(Value::Bool(b)) => Ok(Value::Str(if *b { "true" } else { "false" }.to_string())),
            _ => Err(MetelError::internal("Bool::to_string: expected Bool")),
        }
    }));
    env.define("Char::to_string", Value::Builtin("Char::to_string".to_string(), |args, _span| {
        match args.first() {
            Some(Value::Char(c)) => Ok(Value::Str(c.to_string())),
            _ => Err(MetelError::internal("Char::to_string: expected Char")),
        }
    }));
    env.define("String::to_string", Value::Builtin("String::to_string".to_string(), |args, _span| {
        match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(s.clone())),
            _ => Err(MetelError::internal("String::to_string: expected String")),
        }
    }));

    // From impls for numeric conversions.
    env.define("i64::from", Value::Builtin("i64::from".to_string(), |args, _span| {
        match args.first() {
            Some(Value::F64(f)) => Ok(Value::I64(*f as i64)),
            Some(Value::I64(n))   => Ok(Value::I64(*n)),
            _ => Err(MetelError::internal("i64::from: expected f64")),
        }
    }));
    env.define("f64::from", Value::Builtin("f64::from".to_string(), |args, _span| {
        match args.first() {
            Some(Value::I64(n))   => Ok(Value::F64(*n as f64)),
            Some(Value::F64(f)) => Ok(Value::F64(*f)),
            _ => Err(MetelError::internal("f64::from: expected i64")),
        }
    }));

    // Sized integer / float → Int
    macro_rules! int_from {
        ($key:expr, $pat:pat => $val:expr) => {
            env.define($key, Value::Builtin($key.to_string(), |args, _span| {
                match args.first() {
                    Some($pat) => Ok(Value::I64($val)),
                    _ => Err(MetelError::internal(concat!($key, ": unexpected argument"))),
                }
            }));
        };
    }
    int_from!("i64::From<i8>::from",  Value::I8(n)  => *n as i64);
    int_from!("i64::From<i16>::from", Value::I16(n) => *n as i64);
    int_from!("i64::From<i32>::from", Value::I32(n) => *n as i64);
    int_from!("i64::From<u8>::from",  Value::U8(n)  => *n as i64);
    int_from!("i64::From<u16>::from", Value::U16(n) => *n as i64);
    int_from!("i64::From<u32>::from", Value::U32(n) => *n as i64);
    int_from!("i64::From<u64>::from", Value::U64(n) => *n as i64);
    int_from!("i64::From<f32>::from", Value::F32(f) => *f as i64);

    // Sized integer / float → Float
    macro_rules! float_from {
        ($key:expr, $pat:pat => $val:expr) => {
            env.define($key, Value::Builtin($key.to_string(), |args, _span| {
                match args.first() {
                    Some($pat) => Ok(Value::F64($val)),
                    _ => Err(MetelError::internal(concat!($key, ": unexpected argument"))),
                }
            }));
        };
    }
    float_from!("f64::From<i8>::from",  Value::I8(n)  => *n as f64);
    float_from!("f64::From<i16>::from", Value::I16(n) => *n as f64);
    float_from!("f64::From<i32>::from", Value::I32(n) => *n as f64);
    float_from!("f64::From<u8>::from",  Value::U8(n)  => *n as f64);
    float_from!("f64::From<u16>::from", Value::U16(n) => *n as f64);
    float_from!("f64::From<u32>::from", Value::U32(n) => *n as f64);
    float_from!("f64::From<u64>::from", Value::U64(n) => *n as f64);
    float_from!("f64::From<f32>::from", Value::F32(f) => *f as f64);

    // Int / Float → sized integer types (truncating / wrapping)
    macro_rules! sized_from {
        ($key:expr, Int => $cast:expr) => {
            env.define($key, Value::Builtin($key.to_string(), |args, _span| {
                match args.first() {
                    Some(Value::I64(n))   => Ok($cast(*n as i128)),
                    Some(Value::F64(f)) => Ok($cast(*f as i128)),
                    _ => Err(MetelError::internal(concat!($key, ": unexpected argument"))),
                }
            }));
        };
    }
    sized_from!("u64::From<i64>::from",   Int => |n: i128| Value::U64(n as u64));
    sized_from!("u64::From<f64>::from",   Int => |n: i128| Value::U64(n as u64));
    sized_from!("i8::From<i64>::from",    Int => |n: i128| Value::I8(n as i8));
    sized_from!("i8::From<f64>::from",    Int => |n: i128| Value::I8(n as i8));
    sized_from!("i16::From<i64>::from",   Int => |n: i128| Value::I16(n as i16));
    sized_from!("i16::From<f64>::from",   Int => |n: i128| Value::I16(n as i16));
    sized_from!("i32::From<i64>::from",   Int => |n: i128| Value::I32(n as i32));
    sized_from!("i32::From<f64>::from",   Int => |n: i128| Value::I32(n as i32));
    sized_from!("u8::From<i64>::from",    Int => |n: i128| Value::U8(n as u8));
    sized_from!("u8::From<f64>::from",    Int => |n: i128| Value::U8(n as u8));
    sized_from!("u16::From<i64>::from",   Int => |n: i128| Value::U16(n as u16));
    sized_from!("u16::From<f64>::from",   Int => |n: i128| Value::U16(n as u16));
    sized_from!("u32::From<i64>::from",   Int => |n: i128| Value::U32(n as u32));
    sized_from!("u32::From<f64>::from",   Int => |n: i128| Value::U32(n as u32));
    sized_from!("f32::From<i64>::from",   Int => |n: i128| Value::F32(n as f32));
    sized_from!("f32::From<f64>::from",   Int => |n: i128| Value::F32(n as f32));

    env.define("u32::From<Char>::from", Value::Builtin("u32::From<Char>::from".to_string(), |args, _span| {
        match args.first() {
            Some(Value::Char(c)) => Ok(Value::U32(*c as u32)),
            _ => Err(MetelError::internal("u32::From<Char>::from: expected Char")),
        }
    }));
    env.define("Char::From<u32>::from", Value::Builtin("Char::From<u32>::from".to_string(), |args, span| {
        match args.first() {
            Some(Value::U32(n)) => char::from_u32(*n).map(Value::Char).ok_or_else(|| {
                MetelError::panic(RuntimeErrorCode::R0009, format!("u32 value {n} is not a valid Unicode scalar"), span)
            }),
            _ => Err(MetelError::internal("Char::From<u32>::from: expected u32")),
        }
    }));

    env.define("string_len", Value::Builtin("string_len".to_string(), |args, _span| {
        if let Some(Value::Str(s)) = args.first() {
            Ok(Value::I64(s.chars().count() as i64))
        } else {
            Err(MetelError::internal("string_len: expected String argument"))
        }
    }));

    env.define("string_concat", Value::Builtin("string_concat".to_string(), |args, _span| {
        match (args.first(), args.get(1)) {
            (Some(Value::Str(a)), Some(Value::Str(b))) => Ok(Value::Str(a.clone() + b)),
            _ => Err(MetelError::internal("string_concat: expected two String arguments")),
        }
    }));

    // ── List<T> constructors ──────────────────────────────────────────────────

    // Helper: build a List Value from a backing Rc array.
    // List<T> is represented as Value::Struct { name: "List", fields: { "inner": Value::Array(rc) } }

    env.define("List::new", Value::Builtin("List::new".to_string(), |_args, _span| {
        use std::rc::Rc;
        use std::cell::RefCell;
        let mut fields = std::collections::HashMap::new();
        fields.insert("inner".to_string(), Value::Array(Rc::new(RefCell::new(vec![]))));
        Ok(Value::Struct { name: "List".to_string(), fields })
    }));

    env.define("List::from", Value::Builtin("List::from".to_string(), |args, _span| {
        use std::rc::Rc;
        use std::cell::RefCell;
        match args.first() {
            Some(Value::Array(src)) => {
                let copy = src.borrow().clone();
                let mut fields = std::collections::HashMap::new();
                fields.insert("inner".to_string(), Value::Array(Rc::new(RefCell::new(copy))));
                Ok(Value::Struct { name: "List".to_string(), fields })
            }
            _ => Err(MetelError::internal("List::from: expected array argument")),
        }
    }));

    // ── List<T> instance methods (keyed as "List::method") ───────────────────

    env.define("List::push", Value::Builtin("List::push".to_string(), |args, _span| {
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
    }));

    env.define("List::pop", Value::Builtin("List::pop".to_string(), |args, span| {
        match args.first() {
            Some(Value::Struct { name, fields }) if name == "List" => {
                if let Some(Value::Array(arr)) = fields.get("inner") {
                    match arr.borrow_mut().pop() {
                        Some(val) => {
                            let mut f = std::collections::HashMap::new();
                            f.insert("value".to_string(), val);
                            Ok(Value::Enum { name: "Perhaps".to_string(), variant: "Some".to_string(), fields: f })
                        }
                        None => Ok(Value::Enum { name: "Perhaps".to_string(), variant: "None".to_string(), fields: std::collections::HashMap::new() }),
                    }
                } else {
                    Err(MetelError::internal("List::pop: missing inner field"))
                }
            }
            _ => Err(MetelError::panic(RuntimeErrorCode::R0009, "List::pop: expected List", span)),
        }
    }));

    env.define("List::len", Value::Builtin("List::len".to_string(), |args, _span| {
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
    }));

    env.define("List::get", Value::Builtin("List::get".to_string(), |args, _span| {
        match (args.first(), args.get(1)) {
            (Some(Value::Struct { name, fields }), Some(Value::I64(idx))) if name == "List" => {
                if let Some(Value::Array(arr)) = fields.get("inner") {
                    match arr.borrow().get(*idx as usize).cloned() {
                        Some(val) => {
                            let mut f = std::collections::HashMap::new();
                            f.insert("value".to_string(), val);
                            Ok(Value::Enum { name: "Perhaps".to_string(), variant: "Some".to_string(), fields: f })
                        }
                        None => Ok(Value::Enum { name: "Perhaps".to_string(), variant: "None".to_string(), fields: std::collections::HashMap::new() }),
                    }
                } else {
                    Err(MetelError::internal("List::get: missing inner field"))
                }
            }
            _ => Err(MetelError::internal("List::get: expected (List, i64)")),
        }
    }));

    env.define("List::as_slice", Value::Builtin("List::as_slice".to_string(), |args, _span| {
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
    }));

    env.define("clock", Value::Builtin("clock".to_string(), |_args, _span| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        Ok(Value::I64(ms))
    }));

    env.define("assert", Value::Builtin("assert".to_string(), |args, span| {
        match args.first() {
            Some(Value::Bool(true)) => Ok(Value::Unit),
            Some(Value::Bool(false)) => Err(MetelError::panic(
                RuntimeErrorCode::R0013,
                "assertion failed",
                span,
            )),
            _ => Err(MetelError::internal("assert: expected Bool argument")),
        }
    }));

    env.define("assert_msg", Value::Builtin("assert_msg".to_string(), |args, span| {
        match (args.first(), args.get(1)) {
            (Some(Value::Bool(true)), _) => Ok(Value::Unit),
            (Some(Value::Bool(false)), Some(Value::Str(msg))) => Err(MetelError::panic(
                RuntimeErrorCode::R0013,
                msg.clone(),
                span,
            )),
            (Some(Value::Bool(false)), _) => Err(MetelError::panic(
                RuntimeErrorCode::R0013,
                "assertion failed",
                span,
            )),
            _ => Err(MetelError::internal("assert_msg: expected (Bool, String) arguments")),
        }
    }));

    env.define("dbg", Value::Builtin("dbg".to_string(), |args, _span| {
        if let Some(val) = args.first() {
            eprintln!("[dbg] {}", format_value(val));
            Ok(val.clone())
        } else {
            Err(MetelError::internal("dbg: expected one argument"))
        }
    }));
}
