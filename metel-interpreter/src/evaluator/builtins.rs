use crate::error::{MetelError, RuntimeErrorCode};

use super::{Environment, Value};
use super::display::{format_float, format_value, value_to_display_string};

/// The free-function builtin names registered by this module.
/// Must stay in sync with `StdPrelude::schemes()`. See METEL-5 / ADR-0027.
pub(crate) fn free_function_names() -> std::collections::HashSet<&'static str> {
    [
        "print", "println", "string_len", "string_concat",
        "array_push", "array_len", "clock", "assert", "assert_msg", "dbg",
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
    env.define("Int::to_string", Value::Builtin("Int::to_string".to_string(), |args, _span| {
        match args.first() {
            Some(Value::Int(n)) => Ok(Value::Str(n.to_string())),
            _ => Err(MetelError::internal("Int::to_string: expected Int")),
        }
    }));
    env.define("Float::to_string", Value::Builtin("Float::to_string".to_string(), |args, _span| {
        match args.first() {
            Some(Value::Float(f)) => Ok(Value::Str(format_float(*f))),
            _ => Err(MetelError::internal("Float::to_string: expected Float")),
        }
    }));
    env.define("Bool::to_string", Value::Builtin("Bool::to_string".to_string(), |args, _span| {
        match args.first() {
            Some(Value::Bool(b)) => Ok(Value::Str(if *b { "true" } else { "false" }.to_string())),
            _ => Err(MetelError::internal("Bool::to_string: expected Bool")),
        }
    }));
    env.define("String::to_string", Value::Builtin("String::to_string".to_string(), |args, _span| {
        match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(s.clone())),
            _ => Err(MetelError::internal("String::to_string: expected String")),
        }
    }));

    // From impls for numeric conversions.
    env.define("Int::from", Value::Builtin("Int::from".to_string(), |args, _span| {
        match args.first() {
            Some(Value::Float(f)) => Ok(Value::Int(*f as i64)),
            Some(Value::Int(n))   => Ok(Value::Int(*n)),
            _ => Err(MetelError::internal("Int::from: expected Float")),
        }
    }));
    env.define("Float::from", Value::Builtin("Float::from".to_string(), |args, _span| {
        match args.first() {
            Some(Value::Int(n))   => Ok(Value::Float(*n as f64)),
            Some(Value::Float(f)) => Ok(Value::Float(*f)),
            _ => Err(MetelError::internal("Float::from: expected Int")),
        }
    }));

    // Sized integer / float → Int
    macro_rules! int_from {
        ($key:expr, $pat:pat => $val:expr) => {
            env.define($key, Value::Builtin($key.to_string(), |args, _span| {
                match args.first() {
                    Some($pat) => Ok(Value::Int($val)),
                    _ => Err(MetelError::internal(concat!($key, ": unexpected argument"))),
                }
            }));
        };
    }
    int_from!("Int::From<i8>::from",  Value::I8(n)  => *n as i64);
    int_from!("Int::From<i16>::from", Value::I16(n) => *n as i64);
    int_from!("Int::From<i32>::from", Value::I32(n) => *n as i64);
    int_from!("Int::From<u8>::from",  Value::U8(n)  => *n as i64);
    int_from!("Int::From<u16>::from", Value::U16(n) => *n as i64);
    int_from!("Int::From<u32>::from", Value::U32(n) => *n as i64);
    int_from!("Int::From<u64>::from", Value::U64(n) => *n as i64);
    int_from!("Int::From<f32>::from", Value::F32(f) => *f as i64);

    // Sized integer / float → Float
    macro_rules! float_from {
        ($key:expr, $pat:pat => $val:expr) => {
            env.define($key, Value::Builtin($key.to_string(), |args, _span| {
                match args.first() {
                    Some($pat) => Ok(Value::Float($val)),
                    _ => Err(MetelError::internal(concat!($key, ": unexpected argument"))),
                }
            }));
        };
    }
    float_from!("Float::From<i8>::from",  Value::I8(n)  => *n as f64);
    float_from!("Float::From<i16>::from", Value::I16(n) => *n as f64);
    float_from!("Float::From<i32>::from", Value::I32(n) => *n as f64);
    float_from!("Float::From<u8>::from",  Value::U8(n)  => *n as f64);
    float_from!("Float::From<u16>::from", Value::U16(n) => *n as f64);
    float_from!("Float::From<u32>::from", Value::U32(n) => *n as f64);
    float_from!("Float::From<u64>::from", Value::U64(n) => *n as f64);
    float_from!("Float::From<f32>::from", Value::F32(f) => *f as f64);

    // Int / Float → sized integer types (truncating / wrapping)
    macro_rules! sized_from {
        ($key:expr, Int => $cast:expr) => {
            env.define($key, Value::Builtin($key.to_string(), |args, _span| {
                match args.first() {
                    Some(Value::Int(n))   => Ok($cast(*n as i128)),
                    Some(Value::Float(f)) => Ok($cast(*f as i128)),
                    _ => Err(MetelError::internal(concat!($key, ": unexpected argument"))),
                }
            }));
        };
    }
    sized_from!("u64::From<Int>::from",   Int => |n: i128| Value::U64(n as u64));
    sized_from!("u64::From<Float>::from", Int => |n: i128| Value::U64(n as u64));
    sized_from!("i8::From<Int>::from",    Int => |n: i128| Value::I8(n as i8));
    sized_from!("i8::From<Float>::from",  Int => |n: i128| Value::I8(n as i8));
    sized_from!("i16::From<Int>::from",   Int => |n: i128| Value::I16(n as i16));
    sized_from!("i16::From<Float>::from", Int => |n: i128| Value::I16(n as i16));
    sized_from!("i32::From<Int>::from",   Int => |n: i128| Value::I32(n as i32));
    sized_from!("i32::From<Float>::from", Int => |n: i128| Value::I32(n as i32));
    sized_from!("u8::From<Int>::from",    Int => |n: i128| Value::U8(n as u8));
    sized_from!("u8::From<Float>::from",  Int => |n: i128| Value::U8(n as u8));
    sized_from!("u16::From<Int>::from",   Int => |n: i128| Value::U16(n as u16));
    sized_from!("u16::From<Float>::from", Int => |n: i128| Value::U16(n as u16));
    sized_from!("u32::From<Int>::from",   Int => |n: i128| Value::U32(n as u32));
    sized_from!("u32::From<Float>::from", Int => |n: i128| Value::U32(n as u32));
    sized_from!("f32::From<Int>::from",   Int => |n: i128| Value::F32(n as f32));
    sized_from!("f32::From<Float>::from", Int => |n: i128| Value::F32(n as f32));

    env.define("string_len", Value::Builtin("string_len".to_string(), |args, _span| {
        if let Some(Value::Str(s)) = args.first() {
            Ok(Value::Int(s.chars().count() as i64))
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

    env.define("array_push", Value::Builtin("array_push".to_string(), |args, _span| {
        if let Some(Value::Array(arr)) = args.first() {
            if let Some(val) = args.get(1) {
                arr.borrow_mut().push(val.clone());
                Ok(Value::Unit)
            } else {
                Err(MetelError::internal("array_push: missing value argument"))
            }
        } else {
            Err(MetelError::internal("array_push: expected Array as first argument"))
        }
    }));

    env.define("array_len", Value::Builtin("array_len".to_string(), |args, _span| {
        if let Some(Value::Array(arr)) = args.first() {
            Ok(Value::Int(arr.borrow().len() as i64))
        } else {
            Err(MetelError::internal("array_len: expected Array argument"))
        }
    }));

    env.define("clock", Value::Builtin("clock".to_string(), |_args, _span| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        Ok(Value::Int(ms))
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
