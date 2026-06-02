use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::Span;
use crate::error::{MetelError, RuntimeErrorCode};

use super::{ClosureBody, Signal, Value, attach_stack, eval_block, eval_untyped_block, pop_frame, push_frame};

pub(super) enum ReceiverBinding {
    Value(Value),
    Shared(Rc<RefCell<Value>>),
}

/// Dispatch a function call to a `Value::Builtin` or `Value::Closure`.
/// Converts `Signal::Return` at the function boundary.
pub(super) fn call_function(func: Value, args: Vec<Value>, span: &Span) -> Result<Signal, MetelError> {
    // Auto-deref: calling through a function pointer transparently unwraps one pointer layer.
    let func = match func {
        Value::Pointer(rc) | Value::MutPointer(rc) => rc.borrow().clone(),
        other => other,
    };
    match func {
        Value::Builtin(_, f) => f(args, span).map(Signal::Value).map_err(attach_stack),

        Value::Closure(rc) => {
            let closure = (*rc).clone();
            let fn_name = closure.name.clone().unwrap_or_else(|| "<closure>".to_string());
            push_frame(fn_name, span.clone());
            let mut call_env = closure.captured.clone();
            call_env.push_scope();
            for (param, val) in closure.params.iter().zip(args.iter()) {
                call_env.define(&param.name, val.clone());
            }
            let result = match &closure.body {
                ClosureBody::Typed(b)   => eval_block(b, &mut call_env),
                ClosureBody::Untyped(b) => eval_untyped_block(b, &mut call_env),
            };
            let result = result.map_err(attach_stack);
            pop_frame();
            let sig = result?;
            Ok(match sig {
                Signal::Return(v) => Signal::Value(v),
                other => other,
            })
        }

        Value::Unit =>
            Err(attach_stack(MetelError::panic(RuntimeErrorCode::R0002, "call: target is Unit, not a function", span))),

        other => Err(attach_stack(MetelError::panic(
            RuntimeErrorCode::R0010,
            format!("call: expected a closure or builtin, got {:?}", std::mem::discriminant(&other)),
            span,
        ))),
    }
}

pub(super) fn call_method_function(
    func: Value,
    receiver: ReceiverBinding,
    args: Vec<Value>,
    span: &Span,
) -> Result<Signal, MetelError> {
    match func {
        Value::Closure(rc) => {
            let closure = (*rc).clone();
            let fn_name = closure.name.clone().unwrap_or_else(|| "<closure>".to_string());
            push_frame(fn_name, span.clone());
            let mut call_env = closure.captured.clone();
            call_env.push_scope();
            if let Some(param) = closure.params.first() {
                match receiver {
                    ReceiverBinding::Value(value) => call_env.define(&param.name, value),
                    ReceiverBinding::Shared(cell) => call_env.define_rc(&param.name, cell),
                }
            }
            for (param, val) in closure.params.iter().skip(1).zip(args.iter()) {
                call_env.define(&param.name, val.clone());
            }
            let result = match &closure.body {
                ClosureBody::Typed(b)   => eval_block(b, &mut call_env),
                ClosureBody::Untyped(b) => eval_untyped_block(b, &mut call_env),
            };
            let result = result.map_err(attach_stack);
            pop_frame();
            let sig = result?;
            Ok(match sig {
                Signal::Return(v) => Signal::Value(v),
                other => other,
            })
        }
        other => call_function(other, args, span),
    }
}
