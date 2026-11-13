// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

//! `json.dumps` matching CPython (ensure_ascii, indent=1 or compact).

use std::fmt::Write as _;

use serde_json::Value;

pub fn dumps_compact(value: &Value) -> Result<String, String> {
    let mut out = String::new();
    write_value(&mut out, value, None, 0)?;
    Ok(out)
}

pub fn dumps_pretty(value: &Value) -> Result<String, String> {
    let mut out = String::new();
    write_value(&mut out, value, Some(1), 0)?;
    out.push('\n');
    Ok(out)
}

fn write_value(
    out: &mut String,
    value: &Value,
    indent: Option<usize>,
    level: usize,
) -> Result<(), String> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => write_number(out, n)?,
        Value::String(s) => write_string(out, s),
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return Ok(());
            }
            match indent {
                None => {
                    out.push('[');
                    for (i, item) in items.iter().enumerate() {
                        if i > 0 {
                            out.push_str(", ");
                        }
                        write_value(out, item, None, 0)?;
                    }
                    out.push(']');
                }
                Some(step) => {
                    out.push_str("[\n");
                    for (i, item) in items.iter().enumerate() {
                        push_indent(out, (level + 1) * step);
                        write_value(out, item, indent, level + 1)?;
                        if i + 1 != items.len() {
                            out.push(',');
                        }
                        out.push('\n');
                    }
                    push_indent(out, level * step);
                    out.push(']');
                }
            }
        }
        Value::Object(map) => {
            if map.is_empty() {
                out.push_str("{}");
                return Ok(());
            }
            match indent {
                None => {
                    out.push('{');
                    for (i, (k, v)) in map.iter().enumerate() {
                        if i > 0 {
                            out.push_str(", ");
                        }
                        write_string(out, k);
                        out.push_str(": ");
                        write_value(out, v, None, 0)?;
                    }
                    out.push('}');
                }
                Some(step) => {
                    out.push_str("{\n");
                    for (i, (k, v)) in map.iter().enumerate() {
                        push_indent(out, (level + 1) * step);
                        write_string(out, k);
                        out.push_str(": ");
                        write_value(out, v, indent, level + 1)?;
                        if i + 1 != map.len() {
                            out.push(',');
                        }
                        out.push('\n');
                    }
                    push_indent(out, level * step);
                    out.push('}');
                }
            }
        }
    }
    Ok(())
}

fn push_indent(out: &mut String, n: usize) {
    for _ in 0..n {
        out.push(' ');
    }
}

fn write_number(out: &mut String, n: &serde_json::Number) -> Result<(), String> {
    if let Some(i) = n.as_i64() {
        let _ = write!(out, "{i}");
        return Ok(());
    }
    if let Some(u) = n.as_u64() {
        let _ = write!(out, "{u}");
        return Ok(());
    }
    if let Some(f) = n.as_f64() {
        out.push_str(&format_f64(f)?);
        return Ok(());
    }
    Err(format!("unsupported number {n}"))
}

pub fn format_f64(f: f64) -> Result<String, String> {
    if !f.is_finite() {
        return Err("Out of range float values are not JSON compliant".into());
    }
    if f == 0.0 {
        return Ok(if f.is_sign_negative() {
            "-0.0".into()
        } else {
            "0.0".into()
        });
    }
    let abs = f.abs();
    // CPython json uses a shortest round-trip, scientific when abs < 1e-4
    // or >= 1e16, a two-digit signed exponent, and a trailing .0 on whole
    // decimal values.
    if abs < 1e-4 || abs >= 1e16 {
        Ok(normalize_sci(&format!("{f:e}")))
    } else {
        let mut s = format!("{f}");
        if !s.contains('.') && !s.contains('e') && !s.contains('E') {
            s.push_str(".0");
        }
        Ok(s)
    }
}

fn normalize_sci(s: &str) -> String {
    let (sign, rest) = match s.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", s),
    };
    let Some((mant, exp)) = rest.split_once('e').or_else(|| rest.split_once('E')) else {
        return s.to_string();
    };
    let e: i32 = match exp.parse() {
        Ok(e) => e,
        Err(_) => return s.to_string(),
    };
    format!("{sign}{mant}e{e:+03}")
}

fn write_string(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || (c as u32) >= 0x7f => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    let _ = write!(out, "\\u{unit:04x}");
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::format_f64;

    #[test]
    fn float_examples() {
        for (f, expected) in [
            (0.0, "0.0"),
            (-0.0, "-0.0"),
            (1.0, "1.0"),
            (100.0, "100.0"),
            (110.5, "110.5"),
            (0.1, "0.1"),
            (0.2, "0.2"),
            (1e-7, "1e-07"),
            (1e-4, "0.0001"),
            (1e15, "1000000000000000.0"),
            (1e16, "1e+16"),
            (2.5e-8, "2.5e-08"),
        ] {
            assert_eq!(format_f64(f).unwrap(), expected, "f={f}");
        }
    }
}
