use serde_json::Value;
use crate::services::models::category::Category;

pub fn val_to_opt_u64(v: &Value) -> Option<u64> {
    match v {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}

pub fn val_to_bool_default(v: &Value, default: bool) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::String(s) => match s.as_str() {
            "true" | "1" => true,
            "false" | "0" => false,
            _ => default,
        },
        Value::Number(n) => n.as_u64().map(|x| x != 0).unwrap_or(default),
        _ => default,
    }
}

pub fn value_to_category(v: &Value) -> Option<Category> {
    let id = v.get("id").and_then(val_to_opt_u64)?;
    let name = v
        .get("name")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let parent = v.get("parent").and_then(val_to_opt_u64);
    let available = v
        .get("available")
        .map(|x| val_to_bool_default(x, false))
        .unwrap_or(false);
    Some(Category {
        id,
        name,
        parent,
        available,
    })
}