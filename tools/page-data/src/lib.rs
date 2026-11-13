// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

//! Compact page JSON from a benchmark summary, matching `page-data.py`.

use serde_json::{Map, Value};

/// Build the compact page object as a JSON string (one trailing newline).
pub fn page_from_summary(summary: &Value) -> Result<String, String> {
    let obj = summary
        .as_object()
        .ok_or_else(|| "summary must be a JSON object".to_string())?;
    let order = obj
        .get("order")
        .and_then(Value::as_array)
        .ok_or_else(|| "summary.order must be an array".to_string())?;
    let columns = obj
        .get("columns")
        .and_then(Value::as_object)
        .ok_or_else(|| "summary.columns must be an object".to_string())?;
    let hosts = obj
        .get("hosts")
        .and_then(Value::as_object)
        .ok_or_else(|| "summary.hosts must be an object".to_string())?;

    let mut labels: Map<String, Value> = Map::new();
    for host in hosts.values() {
        let rows = host
            .get("rows")
            .and_then(Value::as_object)
            .ok_or_else(|| "host.rows must be an object".to_string())?;
        for (row, value) in rows {
            if let Some(label) = value.get("label") {
                labels.insert(row.clone(), label.clone());
            }
        }
    }

    let mut rows: Vec<(String, String)> = Vec::new();
    for row in order {
        let row = row
            .as_str()
            .ok_or_else(|| "summary.order entries must be strings".to_string())?;
        if let Some(label) = labels.get(row).and_then(Value::as_str) {
            rows.push((row.to_string(), label.to_string()));
        }
    }

    let col_keys: Vec<String> = columns.keys().cloned().collect();

    let mut out = String::from("{\"rows\":[");
    for (i, (row, label)) in rows.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("[{},{}]", json_string(row), json_string(label)));
    }
    out.push_str("],\"cols\":[");
    for (i, key) in col_keys.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let title = columns
            .get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("columns.{key} must be a string"))?;
        out.push_str(&format!("[{},{}]", json_string(key), json_string(title)));
    }
    out.push_str("],\"hosts\":[");

    for (hi, (name, host)) in hosts.iter().enumerate() {
        if hi > 0 {
            out.push(',');
        }
        out.push_str(&host_json(name, host, &rows, &col_keys)?);
    }
    out.push_str("]}\n");
    Ok(out)
}

fn host_json(
    name: &str,
    host: &Value,
    rows: &[(String, String)],
    col_keys: &[String],
) -> Result<String, String> {
    let host_rows = host
        .get("rows")
        .and_then(Value::as_object)
        .ok_or_else(|| "host.rows must be an object".to_string())?;
    let cpu = host
        .get("cpu")
        .and_then(Value::as_object)
        .ok_or_else(|| "host.cpu must be an object".to_string())?;
    let rustc = host
        .get("rustc")
        .and_then(Value::as_str)
        .ok_or_else(|| "host.rustc must be a string".to_string())?;
    let rustc = rustc.split(" (").next().unwrap_or(rustc);
    let commit = host
        .get("commit")
        .and_then(Value::as_str)
        .ok_or_else(|| "host.commit must be a string".to_string())?;
    let commit = if commit.len() >= 8 {
        &commit[..8]
    } else {
        commit
    };
    let started = host
        .get("started")
        .and_then(Value::as_str)
        .ok_or_else(|| "host.started must be a string".to_string())?;
    let date = if started.len() >= 10 {
        &started[..10]
    } else {
        started
    };
    let ours = host.get("source").and_then(Value::as_str) == Some("this run");
    let load = host.get("load").and_then(Value::as_object);
    let noisy = load
        .and_then(|l| l.get("noisy"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let busy = load.and_then(|l| l.get("other_busy_cpus_during_run"));

    let mut times = String::from("[");
    for (i, (row, _)) in rows.iter().enumerate() {
        if i > 0 {
            times.push(',');
        }
        times.push('[');
        let value = host_rows.get(row);
        for (j, column) in col_keys.iter().enumerate() {
            if j > 0 {
                times.push(',');
            }
            match value
                .and_then(|v| v.get("times"))
                .and_then(Value::as_object)
            {
                Some(t) if t.contains_key(column) => {
                    let x = t[column]
                        .as_f64()
                        .ok_or_else(|| format!("times.{column} must be a number"))?;
                    times.push_str(&json_dumps_float(python_g4_float(x)));
                }
                _ => times.push_str("null"),
            }
        }
        times.push(']');
    }
    times.push(']');

    let cost = host.get("build_cost").and_then(Value::as_object);
    let (builds, bcfail) = match cost {
        Some(c) if c.get("status").and_then(Value::as_str) == Some("ok") => {
            let mut b = String::from("[");
            if let Some(list) = c.get("builds").and_then(Value::as_array) {
                for (i, build) in list.iter().enumerate() {
                    if i > 0 {
                        b.push(',');
                    }
                    b.push_str(&build_row(build)?);
                }
            }
            b.push(']');
            (b, "null".to_string())
        }
        _ => {
            let reason = cost
                .and_then(|c| c.get("reason"))
                .and_then(Value::as_str)
                .map(json_string)
                .unwrap_or_else(|| "null".to_string());
            ("[]".to_string(), reason)
        }
    };

    let os = host
        .get("os")
        .and_then(Value::as_str)
        .ok_or_else(|| "host.os must be a string".to_string())?;
    let cpu_model = cpu
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| "host.cpu.model must be a string".to_string())?;
    let arch = cpu
        .get("arch")
        .and_then(Value::as_str)
        .ok_or_else(|| "host.cpu.arch must be a string".to_string())?;
    let lead = host
        .get("lead")
        .ok_or_else(|| "host.lead is required".to_string())?;
    let timed = host
        .get("timed")
        .ok_or_else(|| "host.timed is required".to_string())?;

    Ok(format!(
        "{{\"id\":{},\"ours\":{},\"cpu\":{},\"arch\":{},\"os\":{},\"rustc\":{},\"commit\":{},\"date\":{},\"noisy\":{},\"busy\":{},\"lead\":{},\"timed\":{},\"t\":{},\"builds\":{},\"bcfail\":{}}}",
        json_string(name),
        if ours { "true" } else { "false" },
        json_string(cpu_model),
        json_string(arch),
        json_string(os),
        json_string(rustc),
        json_string(commit),
        json_string(date),
        if noisy { "true" } else { "false" },
        match busy {
            Some(v) => json_value(v)?,
            None => "null".to_string(),
        },
        json_value(lead)?,
        json_value(timed)?,
        times,
        builds,
        bcfail,
    ))
}

fn build_row(build: &Value) -> Result<String, String> {
    let b = build
        .as_object()
        .ok_or_else(|| "build_cost.builds entries must be objects".to_string())?;
    let name = b
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "build.name must be a string".to_string())?;
    let median_seconds = b
        .get("median_seconds")
        .and_then(Value::as_f64)
        .ok_or_else(|| "build.median_seconds must be a number".to_string())?;
    let median_cpu_seconds = b
        .get("median_cpu_seconds")
        .and_then(Value::as_f64)
        .ok_or_else(|| "build.median_cpu_seconds must be a number".to_string())?;
    let stripped_bytes = b
        .get("stripped_bytes")
        .and_then(Value::as_f64)
        .ok_or_else(|| "build.stripped_bytes must be a number".to_string())?;
    let packages = b
        .get("packages")
        .and_then(Value::as_array)
        .ok_or_else(|| "build.packages must be an array".to_string())?;
    Ok(format!(
        "[{},{},{},{},{}]",
        json_string(name),
        json_dumps_float(median_seconds),
        python_round_int(median_cpu_seconds),
        json_dumps_float(python_round_digits(stripped_bytes / 1e6, 2)),
        packages.len()
    ))
}

fn json_value(v: &Value) -> Result<String, String> {
    match v {
        Value::Null => Ok("null".into()),
        Value::Bool(b) => Ok(if *b { "true" } else { "false" }.into()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.to_string())
            } else if let Some(u) = n.as_u64() {
                Ok(u.to_string())
            } else if let Some(f) = n.as_f64() {
                Ok(json_dumps_float(f))
            } else {
                Err("unsupported number".into())
            }
        }
        Value::String(s) => Ok(json_string(s)),
        _ => Err("lead/timed/busy must be a scalar".into()),
    }
}

fn json_string(s: &str) -> String {
    serde_json::to_string(s).expect("string is valid JSON")
}

/// `float(f"{x:.4g}")`.
pub fn python_g4_float(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    format_g(x, 4).parse().unwrap_or(x)
}

/// Python 3 `round(x)` (ndigits omitted): half to even, returns an integer.
pub fn python_round_int(x: f64) -> i64 {
    python_round_digits(x, 0) as i64
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
    let rounded = if diff > 0.5 {
        floor + 1.0
    } else if diff < 0.5 {
        floor
    } else {
        // exactly halfway
        if (floor as i64) % 2 == 0 {
            floor
        } else {
            floor + 1.0
        }
    };
    rounded / factor
}

/// Python `format(x, ".{prec}g")`.
pub fn format_g(x: f64, prec: usize) -> String {
    if x == 0.0 {
        return if x.is_sign_negative() {
            "-0".into()
        } else {
            "0".into()
        };
    }
    if !x.is_finite() {
        return if x.is_nan() {
            "nan".into()
        } else if x > 0.0 {
            "inf".into()
        } else {
            "-inf".into()
        };
    }
    let prec = prec.max(1);
    let sign = if x < 0.0 { "-" } else { "" };
    let x = x.abs();
    // Round to `prec` significant digits via scientific form.
    let exp = x.log10().floor() as i32;
    let scale = 10f64.powi(exp - prec as i32 + 1);
    let mut mant = python_round_digits(x / scale, 0);
    let mut exp = exp;
    let upper = 10f64.powi(prec as i32);
    if mant >= upper {
        mant /= 10.0;
        exp += 1;
    }
    // Integer digits of the rounded significand.
    let digits = format!("{:0>width$}", mant as u64, width = prec);
    let mut digits = digits.trim_end_matches('0').to_string();
    if digits.is_empty() {
        digits.push('0');
    }
    // Use fixed point when -4 <= exp < prec, else scientific.
    if exp >= -4 && exp < prec as i32 {
        let mut body = if exp < 0 {
            let zeros = (-exp - 1) as usize;
            format!("0.{}{}", "0".repeat(zeros), digits)
        } else {
            let split = exp as usize + 1;
            if digits.len() <= split {
                format!("{}{}", digits, "0".repeat(split - digits.len()))
            } else {
                format!("{}.{}", &digits[..split], &digits[split..])
            }
        };
        // format g strips a trailing decimal point, not needed here.
        let _ = &mut body;
        format!("{sign}{body}")
    } else {
        let (lead, rest) = digits.split_at(1);
        let exp_str = format!("{}{:02}", if exp < 0 { '-' } else { '+' }, exp.abs());
        if rest.is_empty() {
            format!("{sign}{lead}e{exp_str}")
        } else {
            format!("{sign}{lead}.{rest}e{exp_str}")
        }
    }
}

/// CPython `json.dumps` for a finite float (and the same on the `.4g` result).
pub fn json_dumps_float(x: f64) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_g4_and_json_match_cpython_vectors() {
        let cases: &[(f64, &str, &str)] = &[
            (0.0, "0", "0.0"),
            (1.0, "1", "1.0"),
            (12.0, "12", "12.0"),
            (0.5, "0.5", "0.5"),
            (0.75, "0.75", "0.75"),
            (0.25, "0.25", "0.25"),
            (1.25, "1.25", "1.25"),
            (1.23, "1.23", "1.23"),
            (1.235, "1.235", "1.235"),
            (1.23456, "1.235", "1.235"),
            (1.234e-5, "1.234e-05", "1.234e-05"),
            (0.0001234, "0.0001234", "0.0001234"),
            (0.0001, "0.0001", "0.0001"),
            (9.999e-5, "9.999e-05", "9.999e-05"),
            (1e16, "1e+16", "1e+16"),
            (1e15, "1e+15", "1000000000000000.0"),
            (12345.6, "1.235e+04", "12350.0"),
            (2.5, "2.5", "2.5"),
            (99.99, "99.99", "99.99"),
            (100.0, "100", "100.0"),
            (-1.23456, "-1.235", "-1.235"),
        ];
        for &(x, g, json) in cases {
            assert_eq!(format_g(x, 4), g, "format_g({x})");
            assert_eq!(json_dumps_float(python_g4_float(x)), json, "json({x})");
        }
    }

    #[test]
    fn python_rounds_half_to_even() {
        assert_eq!(python_round_int(2.5), 2);
        assert_eq!(python_round_int(3.5), 4);
        assert_eq!(python_round_int(1.5), 2);
        assert_eq!(python_round_int(0.5), 0);
        assert_eq!(python_round_digits(1234567.0 / 1e6, 2), 1.23);
        assert_eq!(python_round_digits(1.225, 2), 1.23);
        assert_eq!(python_round_digits(1.235, 2), 1.24);
    }
}
