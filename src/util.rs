//! Shared helpers: repository root, design tokens, loose JSON accessors, number formatting.
use crate::error::{err, Error, Result};
use serde_json::{Map, Number, Value};
use std::path::{Path, PathBuf};

/// Repository root: `ARCHITECT_DESIGN_ROOT`, else the nearest ancestor of the working
/// directory holding `assets/design-tokens.json`, else the crate's own directory.
pub fn find_root() -> PathBuf {
    if let Some(root) = std::env::var_os("ARCHITECT_DESIGN_ROOT") {
        return PathBuf::from(root);
    }
    if let Ok(mut dir) = std::env::current_dir() {
        loop {
            if dir.join("assets/design-tokens.json").is_file() {
                return dir;
            }
            if !dir.pop() {
                break;
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn read_json(path: &Path) -> Result<Value> {
    let text = std::fs::read_to_string(path).map_err(|e| Error(format!("{}: {e}", path.display())))?;
    serde_json::from_str(&text).map_err(|e| Error(format!("{}: {e}", path.display())))
}

pub fn tokens(root: &Path) -> Result<Value> {
    read_json(&root.join("assets/design-tokens.json"))
}

/// `dict.get(key)` where JSON `null` counts as absent.
pub fn opt<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.get(key).filter(|x| !x.is_null())
}

/// Python truthiness for JSON values.
pub fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) | Some(Value::Bool(false)) => false,
        Some(Value::Number(n)) => n.as_f64().map_or(true, |f| f != 0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
        Some(Value::Bool(true)) => true,
    }
}

pub fn str_of<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    opt(v, key).and_then(Value::as_str)
}

pub fn req_str<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    match v.get(key).and_then(Value::as_str) {
        Some(s) => Ok(s),
        None => err(format!("missing or non-string field: {key}")),
    }
}

pub fn num_of(v: &Value, key: &str, default: f64) -> f64 {
    opt(v, key).and_then(Value::as_f64).unwrap_or(default)
}

pub fn req_num(v: &Value, key: &str) -> Result<f64> {
    match v.get(key).and_then(Value::as_f64) {
        Some(n) => Ok(n),
        None => err(format!("missing or non-numeric field: {key}")),
    }
}

pub fn items<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[])
}

/// Four finite numbers `[x, y, w, h]`, or `None` when malformed.
pub fn rect_of(v: Option<&Value>) -> Option<[f64; 4]> {
    let a = v?.as_array()?;
    if a.len() != 4 {
        return None;
    }
    let mut out = [0.0; 4];
    for (slot, n) in out.iter_mut().zip(a) {
        let f = n.as_f64()?;
        if !f.is_finite() {
            return None;
        }
        *slot = f;
    }
    Some(out)
}

pub fn point_of(v: &Value) -> Option<(f64, f64)> {
    let a = v.as_array()?;
    Some((a.first()?.as_f64()?, a.get(1)?.as_f64()?))
}

/// JSON number from f64, keeping integral values as integers (as Python's `int` arithmetic does).
pub fn num_value(f: f64) -> Value {
    if f.fract() == 0.0 && f.abs() < 9.0e15 {
        Value::Number(Number::from(f as i64))
    } else {
        Number::from_f64(f).map(Value::Number).unwrap_or(Value::Null)
    }
}

/// Number as text in SVG attributes: integral values without a decimal point.
pub fn fmt_num(f: f64) -> String {
    if f.fract() == 0.0 && f.abs() < 9.0e15 {
        format!("{}", f as i64)
    } else {
        format!("{f}")
    }
}

/// `json.dumps(value, ensure_ascii=False)` formatting (`, ` and `: ` separators).
pub fn py_json(v: &Value) -> String {
    let mut out = String::new();
    write_py_json(v, &mut out);
    out
}

fn write_py_json(v: &Value, out: &mut String) {
    match v {
        Value::Number(n) => match (n.as_i64(), n.as_f64()) {
            (Some(i), _) => out.push_str(&i.to_string()),
            (None, Some(f)) => out.push_str(&fmt_num(f)),
            _ => out.push_str(&n.to_string()),
        },
        Value::String(s) => out.push_str(&serde_json::to_string(s).unwrap_or_default()),
        Value::Array(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_py_json(x, out);
            }
            out.push(']');
        }
        Value::Object(o) => {
            out.push('{');
            for (i, (k, x)) in o.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&serde_json::to_string(k).unwrap_or_default());
                out.push_str(": ");
                write_py_json(x, out);
            }
            out.push('}');
        }
        other => out.push_str(&other.to_string()),
    }
}

/// Python `str(value)` for scalar JSON values.
pub fn py_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "None".into(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        other => py_json(other),
    }
}

pub fn object() -> Map<String, Value> {
    Map::new()
}

pub fn contains(outer: [f64; 4], inner: [f64; 4], tolerance: f64) -> bool {
    let [x, y, w, h] = outer;
    let [a, b, c, d] = inner;
    x - tolerance <= a && y - tolerance <= b && a + c <= x + w + tolerance && b + d <= y + h + tolerance
}

pub fn intersect(a: [f64; 4], b: [f64; 4], tolerance: f64) -> bool {
    (a[0] + a[2]).min(b[0] + b[2]) - a[0].max(b[0]) > tolerance
        && (a[1] + a[3]).min(b[1] + b[3]) - a[1].max(b[1]) > tolerance
}
