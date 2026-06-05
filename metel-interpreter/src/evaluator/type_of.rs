use crate::types::Type;
use super::Value;

/// Derive a concrete `Type` from a runtime `Value`.
///
/// Used during construction-at-call-time for generic function bodies: the caller
/// maps each argument value to its type, then uses those types to instantiate the
/// function's `TypeScheme` and build the `Substitution` for `ConstructCtx`.
///
/// Limitations:
/// - Generic structs/enums: type parameters are not recoverable from runtime values,
///   so `Named(name, [])` is returned. The construction pass must unify against the
///   scheme and fill in the parameters from context.
/// - Closures: the concrete function type is not stored in `Value::Closure`, so
///   `Fun([], Box::new(Unit))` is returned as a placeholder.
pub(super) fn value_to_type(value: &Value) -> Type {
    match value {
        Value::Int(_)   => Type::Int,
        Value::Float(_) => Type::Float,
        Value::Bool(_)  => Type::Bool,
        Value::Str(_)   => Type::Str,
        Value::Unit     => Type::Unit,
        Value::I8(_)    => Type::I8,
        Value::I16(_)   => Type::I16,
        Value::I32(_)   => Type::I32,
        Value::U8(_)    => Type::U8,
        Value::U16(_)   => Type::U16,
        Value::U32(_)   => Type::U32,
        Value::U64(_)   => Type::U64,
        Value::F32(_)   => Type::F32,
        Value::Tuple(elems) => {
            Type::Tuple(elems.iter().map(value_to_type).collect())
        }
        Value::Array(rc) => {
            let borrowed = rc.borrow();
            let elem_ty = borrowed.first()
                .map(value_to_type)
                .unwrap_or(Type::Unit);
            Type::Array(Box::new(elem_ty))
        }
        Value::Struct { name, .. } => Type::Named(name.clone(), vec![]),
        Value::Enum   { name, .. } => Type::Named(name.clone(), vec![]),
        Value::Closure(rc) => {
            rc.fun_type.clone().unwrap_or_else(|| Type::Fun(vec![], Box::new(Type::Unit)))
        }
        Value::Builtin(_, _) => {
            Type::Fun(vec![], Box::new(Type::Unit))
        }
        Value::Pointer(rc)    => Type::Pointer(Box::new(value_to_type(&rc.borrow()))),
        Value::MutPointer(rc) => Type::MutPointer(Box::new(value_to_type(&rc.borrow()))),
    }
}
