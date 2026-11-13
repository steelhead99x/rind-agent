// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

//! Python `repr(float)` and `json.dumps(..., indent=...)`.

use serde_json::{Map, Value};

/// `repr(float)`: the shortest digits that round-trip, positional from 1e-4
/// up to 1e16 and scientific (`1e-05`, `1.5e+16`) outside that.
pub fn repr(x: f64) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Infinity" } else { "-Infinity" }.into();
    }
    if x == 0.0 {
        return if x.is_sign_negative() {
            "-0.0".into()
        } else {
            "0.0".into()
        };
    }
    let s = format!("{:e}", x.abs());
    let (mant, exp) = s.split_once('e').expect("{:e} has an exponent");
    let exp: i32 = exp.parse().expect("exponent");
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    let sign = if x < 0.0 { "-" } else { "" };
    let n = digits.len() as i32;
    let decpt = exp + 1;
    let body = if (-4..16).contains(&exp) {
        if decpt <= 0 {
            format!("0.{}{}", "0".repeat((-decpt) as usize), digits)
        } else if decpt >= n {
            format!("{}{}.0", digits, "0".repeat((decpt - n) as usize))
        } else {
            format!(
                "{}.{}",
                &digits[..decpt as usize],
                &digits[decpt as usize..]
            )
        }
    } else {
        let rest = &digits[1..];
        let e = format!("{}{:02}", if exp < 0 { '-' } else { '+' }, exp.abs());
        if rest.is_empty() {
            format!("{}e{}", &digits[..1], e)
        } else {
            format!("{}.{}e{}", &digits[..1], rest, e)
        }
    };
    format!("{sign}{body}")
}

/// Python 3 `round(x, ndigits)`, half to even.
pub fn python_round_digits(x: f64, ndigits: i32) -> f64 {
    if !x.is_finite() {
        return x;
    }
    let factor = 10f64.powi(ndigits);
    let y = x * factor;
    let floor = y.floor();
    let diff = y - floor;
    let rounded = if diff > 0.5 || (diff == 0.5 && (floor as i64) % 2 != 0) {
        floor + 1.0
    } else {
        floor
    };
    rounded / factor
}

/// `json.dumps(v, indent=indent)` with `ensure_ascii` on.
pub fn dumps(v: &Value, indent: Option<usize>) -> String {
    let mut out = String::new();
    write_value(&mut out, v, indent, 0);
    out
}

fn write_value(out: &mut String, v: &Value, indent: Option<usize>, level: usize) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            if n.is_f64() {
                out.push_str(&repr(n.as_f64().unwrap()));
            } else {
                out.push_str(&n.to_string());
            }
        }
        Value::String(s) => write_string(out, s),
        Value::Array(a) => {
            if a.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                sep(out, i, indent, level + 1);
                write_value(out, x, indent, level + 1);
            }
            close(out, indent, level);
            out.push(']');
        }
        Value::Object(m) => write_object(out, m, indent, level),
    }
}

fn write_object(out: &mut String, m: &Map<String, Value>, indent: Option<usize>, level: usize) {
    if m.is_empty() {
        out.push_str("{}");
        return;
    }
    out.push('{');
    for (i, (k, x)) in m.iter().enumerate() {
        sep(out, i, indent, level + 1);
        write_string(out, k);
        out.push_str(": ");
        write_value(out, x, indent, level + 1);
    }
    close(out, indent, level);
    out.push('}');
}

fn sep(out: &mut String, i: usize, indent: Option<usize>, level: usize) {
    match indent {
        Some(n) => {
            if i > 0 {
                out.push(',');
            }
            out.push('\n');
            out.push_str(&" ".repeat(n * level));
        }
        None => {
            if i > 0 {
                out.push_str(", ");
            }
        }
    }
}

fn close(out: &mut String, indent: Option<usize>, level: usize) {
    if let Some(n) = indent {
        out.push('\n');
        out.push_str(&" ".repeat(n * level));
    }
}

fn write_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7e => {
                let mut buf = [0u16; 2];
                for u in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{:04x}", u));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floats_match_python_repr() {
        for (x, s) in [
            (1e-5, "1e-05"),
            (0.0001, "0.0001"),
            (1.5e16, "1.5e+16"),
            (1e16, "1e+16"),
            (1234567890123456.0, "1234567890123456.0"),
            (17.0, "17.0"),
            (0.1, "0.1"),
            (70.841, "70.841"),
            (-2.5, "-2.5"),
            (239999431.6, "239999431.6"),
            (1.0 / 3.0, "0.3333333333333333"),
            (100.25, "100.25"),
            (123.5, "123.5"),
            (48000.125, "48000.125"),
            (50000.0, "50000.0"),
            (12345.6789, "12345.6789"),
            (24.7, "24.7"),
            (0.577, "0.577"),
            (1.0, "1.0"),
        ] {
            assert_eq!(repr(x), s, "repr({x})");
        }
    }

    #[test]
    fn python_rounds_half_to_even() {
        assert_eq!(python_round_digits(2.5, 0), 2.0);
        assert_eq!(python_round_digits(3.5, 0), 4.0);
        assert_eq!(python_round_digits(0.576923076923077, 3), 0.577);
        assert_eq!(python_round_digits(1.0, 2), 1.0);
    }
}
