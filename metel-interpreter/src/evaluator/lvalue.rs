use std::collections::HashMap;

use crate::ast::{BinOp, Span};
use crate::error::{MetelError, RuntimeErrorCode};
use crate::typed_ast::TypedPlace;

use super::{eval_expr, Environment, Signal, Value};


pub(super) fn extract_lvalue_path<'a>(
    expr: &'a crate::ast::Expr,
    span: &Span,
) -> Result<(&'a str, Vec<&'a str>), MetelError> {
    use crate::ast::Expr;
    fn walk<'a>(expr: &'a Expr, path: &mut Vec<&'a str>, span: &Span) -> Result<&'a str, MetelError> {
        match expr {
            Expr::Ident(name, _) => Ok(name.as_str()),
            Expr::FieldAccess { object, field, .. } => {
                let root = walk(object, path, span)?;
                path.push(field.as_str());
                Ok(root)
            }
            _ => Err(MetelError::panic(
                RuntimeErrorCode::R0003,
                "field assign: receiver must be a variable or field access chain",
                span,
            )),
        }
    }
    let mut path = Vec::new();
    let root = walk(expr, &mut path, span)?;
    Ok((root, path))
}

/// Resolve the root `Rc<RefCell<Value>>` for a field-assign target and collect
/// the full field path (including `final_field`) to navigate within it.
///
/// Handles three root forms:
/// - `Ident(x)` — looks up `x` in the environment; auto-derefs one `*mut` level.
/// - `Deref(ptr_expr)` — evaluates `ptr_expr` and extracts the `MutPointer` inner Rc.
/// - `Field { object, field }` — recurses into `object`, then appends `field`.
pub(super) fn resolve_field_assign_root<'a>(
    place: &'a TypedPlace,
    final_field: &'a str,
    env: &mut Environment,
    span: &Span,
) -> Result<(std::rc::Rc<std::cell::RefCell<Value>>, Vec<&'a str>), MetelError> {
    fn walk<'a>(
        place: &'a TypedPlace,
        path: &mut Vec<&'a str>,
        env: &mut Environment,
        span: &Span,
    ) -> Result<std::rc::Rc<std::cell::RefCell<Value>>, MetelError> {
        match place {
            TypedPlace::Ident(name, _) => {
                let rc = env.get_rc(name).ok_or_else(|| {
                    MetelError::panic(RuntimeErrorCode::R0003, format!("assign: `{name}` not found"), span)
                })?;
                // Auto-deref: if the binding holds a *mut pointer, follow it.
                let inner = {
                    let v = rc.borrow();
                    if let Value::MutPointer(inner_rc) = &*v { Some(inner_rc.clone()) } else { None }
                };
                Ok(inner.unwrap_or(rc))
            }
            TypedPlace::Deref { object, span: tspan } => {
                let ptr = eval_expr(object, env)?.into_value();
                match ptr {
                    Value::MutPointer(inner_rc) => Ok(inner_rc),
                    _ => Err(MetelError::panic(RuntimeErrorCode::R0003, "field assign: not a *mut pointer", tspan)),
                }
            }
            TypedPlace::Field { object, field, .. } => {
                let root_rc = walk(object, path, env, span)?;
                path.push(field.as_str());
                Ok(root_rc)
            }
            _ => Err(MetelError::panic(
                RuntimeErrorCode::R0003,
                "field assign: unsupported receiver form",
                span,
            )),
        }
    }
    let mut path = Vec::new();
    let root_rc = walk(place, &mut path, env, span)?;
    path.push(final_field);
    Ok((root_rc, path))
}

/// Evaluate a `TypedPlace` to the `Value` it currently holds.
/// For arrays the returned `Value::Array(rc)` shares the same `Rc` as the binding,
/// so callers can mutate through it without a round-trip through the environment.
pub(super) fn eval_typed_place_value(
    place: &TypedPlace,
    env: &mut Environment,
    span: &Span,
) -> Result<Value, MetelError> {
    match place {
        TypedPlace::Ident(name, _) =>
            env.get(name).ok_or_else(|| {
                MetelError::panic(RuntimeErrorCode::R0003, format!("assign: `{name}` not found"), span)
            }),
        TypedPlace::Deref { object, span: tspan } => {
            let ptr = eval_expr(object, env)?.into_value();
            match ptr {
                Value::Pointer(rc) | Value::MutPointer(rc) => Ok(rc.borrow().clone()),
                _ => Err(MetelError::panic(RuntimeErrorCode::R0003, "assign: not a pointer", tspan)),
            }
        }
        TypedPlace::Field { object, field, span: tspan } => {
            let parent = eval_typed_place_value(object, env, tspan)?;
            match parent {
                Value::Struct { fields, .. } | Value::Enum { fields, .. } =>
                    fields.get(field).cloned().ok_or_else(|| {
                        MetelError::panic(RuntimeErrorCode::R0008, format!("field access: no field `{field}`"), tspan)
                    }),
                _ => Err(MetelError::internal(format!("field `{field}`: receiver is not a struct/enum"))),
            }
        }
        TypedPlace::Index { object, index, span: tspan } => {
            let arr = eval_typed_place_value(object, env, tspan)?;
            let idx = eval_expr(index, env)?.into_value();
            let i = match idx {
                Value::Int(n) => n,
                _ => return Err(MetelError::internal("index expression must be an integer")),
            };
            match arr {
                Value::Array(rc) => {
                    let len = rc.borrow().len() as i64;
                    if i < 0 || i >= len {
                        return Err(MetelError::panic(RuntimeErrorCode::R0004,
                            format!("index {i} out of bounds (len {len})"), tspan));
                    }
                    Ok(rc.borrow()[i as usize].clone())
                }
                _ => Err(MetelError::internal("index: receiver is not an Array")),
            }
        }
    }
}

pub(super) fn apply_assign_op(
    op: &crate::ast::AssignOp,
    cur: Value,
    rhs: Value,
    span: &Span,
) -> Result<Value, MetelError> {
    use crate::ast::AssignOp;
    let fake_binop = match op {
        AssignOp::AddAssign => BinOp::Add,
        AssignOp::SubAssign => BinOp::Sub,
        AssignOp::MulAssign => BinOp::Mul,
        AssignOp::DivAssign => BinOp::Div,
        AssignOp::RemAssign => BinOp::Rem,
        AssignOp::Assign    => unreachable!("plain Assign handled before apply_assign_op"),
    };
    eval_binop(&fake_binop, cur, rhs, span).map(Signal::into_value)
}

pub(super) fn eval_binop(op: &BinOp, lv: Value, rv: Value, span: &Span) -> Result<Signal, MetelError> {
    let result = match (op, lv, rv) {
        // Int arithmetic
        (BinOp::Add, Value::Int(a), Value::Int(b)) => Value::Int(a + b),
        (BinOp::Sub, Value::Int(a), Value::Int(b)) => Value::Int(a - b),
        (BinOp::Mul, Value::Int(a), Value::Int(b)) => Value::Int(a * b),
        (BinOp::Div, Value::Int(a), Value::Int(b)) => {
            if b == 0 { return Err(MetelError::panic(RuntimeErrorCode::R0007, "division by zero", span)); }
            Value::Int(a / b)
        }
        (BinOp::Rem, Value::Int(a), Value::Int(b)) => {
            if b == 0 { return Err(MetelError::panic(RuntimeErrorCode::R0007, "remainder by zero", span)); }
            Value::Int(a % b)
        }

        // Float arithmetic
        (BinOp::Add, Value::Float(a), Value::Float(b)) => Value::Float(a + b),
        (BinOp::Sub, Value::Float(a), Value::Float(b)) => Value::Float(a - b),
        (BinOp::Mul, Value::Float(a), Value::Float(b)) => Value::Float(a * b),
        (BinOp::Div, Value::Float(a), Value::Float(b)) => Value::Float(a / b),
        (BinOp::Rem, Value::Float(a), Value::Float(b)) => Value::Float(a % b),

        // String concatenation
        (BinOp::Add, Value::Str(a), Value::Str(b)) => Value::Str(a + &b),

        // Int comparison
        (BinOp::Eq, Value::Int(a), Value::Int(b)) => Value::Bool(a == b),
        (BinOp::Ne, Value::Int(a), Value::Int(b)) => Value::Bool(a != b),
        (BinOp::Lt, Value::Int(a), Value::Int(b)) => Value::Bool(a <  b),
        (BinOp::Le, Value::Int(a), Value::Int(b)) => Value::Bool(a <= b),
        (BinOp::Gt, Value::Int(a), Value::Int(b)) => Value::Bool(a >  b),
        (BinOp::Ge, Value::Int(a), Value::Int(b)) => Value::Bool(a >= b),

        // Float comparison
        (BinOp::Eq, Value::Float(a), Value::Float(b)) => Value::Bool(a == b),
        (BinOp::Ne, Value::Float(a), Value::Float(b)) => Value::Bool(a != b),
        (BinOp::Lt, Value::Float(a), Value::Float(b)) => Value::Bool(a <  b),
        (BinOp::Le, Value::Float(a), Value::Float(b)) => Value::Bool(a <= b),
        (BinOp::Gt, Value::Float(a), Value::Float(b)) => Value::Bool(a >  b),
        (BinOp::Ge, Value::Float(a), Value::Float(b)) => Value::Bool(a >= b),

        // Bool equality
        (BinOp::Eq, Value::Bool(a), Value::Bool(b)) => Value::Bool(a == b),
        (BinOp::Ne, Value::Bool(a), Value::Bool(b)) => Value::Bool(a != b),

        // String equality
        (BinOp::Eq, Value::Str(a), Value::Str(b)) => Value::Bool(a == b),
        (BinOp::Ne, Value::Str(a), Value::Str(b)) => Value::Bool(a != b),

        // Range — produce a Struct value understood by for-in (issue #55)
        (BinOp::Range, Value::Int(a), Value::Int(b)) => Value::Struct {
            name: "Range".to_string(),
            fields: {
                let mut m = HashMap::new();
                m.insert("start".to_string(), Value::Int(a));
                m.insert("end".to_string(),   Value::Int(b));
                m
            },
        },
        (BinOp::RangeInclusive, Value::Int(a), Value::Int(b)) => Value::Struct {
            name: "RangeInclusive".to_string(),
            fields: {
                let mut m = HashMap::new();
                m.insert("start".to_string(), Value::Int(a));
                m.insert("end".to_string(),   Value::Int(b));
                m
            },
        },

        (_, lv, rv) => return Err(MetelError::internal(
            format!("binop: unsupported operand types ({lv:?}, {rv:?}) (typechecker should have caught this)"),
        )),
    };
    Ok(Signal::Value(result))
}
