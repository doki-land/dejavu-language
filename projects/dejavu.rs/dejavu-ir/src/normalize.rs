use serde_json::{Map, Value};

/// Normalize an IR JSON value for semantic equality.
pub fn normalize_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                if k == "span" {
                    continue;
                }
                let nv = normalize_value(v);
                if k == "raw" && nv == Value::Bool(false) {
                    continue;
                }
                if k == "type" {
                    // keep type
                }
                if let (Some(Value::String(t)), Value::String(text)) =
                    (out.get("type").cloned().or_else(|| if k == "type" { Some(nv.clone()) } else { None }), &nv)
                {
                    if t == "Text" && k == "value" && text.is_empty() {
                        return Value::Null;
                    }
                }
                out.insert(k, nv);
            }
            // Drop empty Text objects
            if out.get("type") == Some(&Value::String("Text".into())) && out.get("value") == Some(&Value::String(String::new()))
            {
                return Value::Null;
            }
            Value::Object(sort_keys(out))
        }
        Value::Array(items) => Value::Array(items.into_iter().map(normalize_value).filter(|v| !v.is_null()).collect()),
        other => other,
    }
}

fn sort_keys(map: Map<String, Value>) -> Map<String, Value> {
    let mut keys: Vec<_> = map.keys().cloned().collect();
    keys.sort();
    let mut sorted = Map::new();
    for k in keys {
        if let Some(v) = map.get(&k) {
            sorted.insert(k, v.clone());
        }
    }
    sorted
}
