use super::Value;

pub(super) fn format_float(f: f64) -> String {
    if f.fract() == 0.0 && f.is_finite() {
        format!("{}", f as i64)
    } else {
        f.to_string()
    }
}

pub(super) fn value_to_display_string(v: &Value) -> Option<String> {
    match v {
        Value::Int(n)   => Some(n.to_string()),
        Value::Float(f) => Some(format_float(*f)),
        Value::Bool(b)  => Some(if *b { "true" } else { "false" }.to_string()),
        Value::Str(s)   => Some(s.clone()),
        _ => None,
    }
}

pub(super) fn format_value(val: &Value) -> String {
    match val {
        Value::Int(n)   => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b)  => b.to_string(),
        Value::Str(s)   => format!("{:?}", s),
        Value::Unit     => "()".to_string(),
        Value::Tuple(items) => {
            let inner = items.iter().map(format_value).collect::<Vec<_>>().join(", ");
            format!("({})", inner)
        }
        Value::Array(arr) => {
            let inner = arr.borrow().iter().map(format_value).collect::<Vec<_>>().join(", ");
            format!("[{}]", inner)
        }
        Value::Struct { name, fields } => {
            let mut pairs: Vec<_> = fields.iter().collect();
            pairs.sort_by_key(|(k, _)| k.as_str());
            let inner = pairs.iter().map(|(k, v)| format!("{}: {}", k, format_value(v))).collect::<Vec<_>>().join(", ");
            format!("{} {{ {} }}", name, inner)
        }
        // Perhaps and Result use familiar Rust-style display rather than the generic enum format.
        // These are the only two enum names singled out by display — all others use the generic arm.
        // See ADR-0028 for why Perhaps/Result are represented as Value::Enum despite special display.
        Value::Enum { name, variant, fields } if name == "Perhaps" => {
            match (variant.as_str(), fields.get("value")) {
                ("Some", Some(v)) => format!("Some({})", format_value(v)),
                _                 => "None".to_string(),
            }
        }
        Value::Enum { name, variant, fields } if name == "Result" => {
            match variant.as_str() {
                "Ok"  => format!("Ok({})",  format_value(fields.get("value").unwrap_or(&Value::Unit))),
                _     => format!("Err({})", format_value(fields.get("error").unwrap_or(&Value::Unit))),
            }
        }
        Value::Enum { name, variant, fields } => {
            if fields.is_empty() {
                format!("{}::{}", name, variant)
            } else {
                let mut pairs: Vec<_> = fields.iter().collect();
                pairs.sort_by_key(|(k, _)| k.as_str());
                let inner = pairs.iter().map(|(k, v)| format!("{}: {}", k, format_value(v))).collect::<Vec<_>>().join(", ");
                format!("{}::{}{{ {} }}", name, variant, inner)
            }
        }
        Value::Closure(_) => "<closure>".to_string(),
        Value::Builtin(name, _) => format!("<builtin:{}>", name),
        // RFC-0001 (pointer syntax) placeholder variants — not constructed until that RFC is implemented.
        Value::Pointer(_) | Value::MutPointer(_) => unreachable!("pointer values not constructed until RFC-0001 is implemented"),
    }
}
