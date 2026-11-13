// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

use serde_json::{Map, Value};

/// Python `str()` of a JSON value, used in NOTES.md lines.
pub fn py_str(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "None".into(),
        Some(Value::Bool(true)) => "True".into(),
        Some(Value::Bool(false)) => "False".into(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(items)) => py_repr_array(items),
        Some(Value::Object(map)) => py_repr_object(map),
    }
}

fn py_repr(value: &Value) -> String {
    match value {
        Value::Null => "None".into(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => py_repr_str(s),
        Value::Array(items) => py_repr_array(items),
        Value::Object(map) => py_repr_object(map),
    }
}

fn py_repr_array(items: &[Value]) -> String {
    let mut out = String::from("[");
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&py_repr(item));
    }
    out.push(']');
    out
}

fn py_repr_object(map: &Map<String, Value>) -> String {
    let mut out = String::from("{");
    for (i, (k, v)) in map.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&py_repr_str(k));
        out.push_str(": ");
        out.push_str(&py_repr(v));
    }
    out.push('}');
    out
}

/// Python `repr()` for a string, used in error messages and dict/list notes.
pub fn py_repr_str(s: &str) -> String {
    let use_double = s.contains('\'') && !s.contains('"');
    let quote = if use_double { '"' } else { '\'' };
    let mut out = String::from(if use_double { "\"" } else { "'" });
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' if !use_double => out.push_str("\\'"),
            '"' if use_double => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let code = c as u32;
                if code <= 0xff {
                    out.push_str(&format!("\\x{code:02x}"));
                } else if code <= 0xffff {
                    out.push_str(&format!("\\u{code:04x}"));
                } else {
                    out.push_str(&format!("\\U{code:08x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

fn map<'a>(host: &'a Value, key: &str) -> Option<&'a Map<String, Value>> {
    host.get(key).and_then(Value::as_object)
}

/// `notes_for` in stage-repo.py.
pub fn notes_for(host: &Value) -> String {
    let load = map(host, "load").cloned().unwrap_or_default();
    let cost = map(host, "build_cost").cloned().unwrap_or_default();
    let openssl = map(host, "libraries")
        .and_then(|m| m.get("openssl"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let mut cost_line = format!("- Build cost: {}", py_str(cost.get("status")));
    if cost.get("status").and_then(Value::as_str) == Some("ok") {
        let tool = cost
            .get("host")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        cost_line.push_str(&format!(
            ", {}, {} jobs",
            py_str(tool.get("cc")),
            py_str(tool.get("jobs"))
        ));
    } else if truthy(cost.get("reason")) {
        cost_line.push_str(&format!(" ({})", py_str(cost.get("reason"))));
    }

    let results = host.get("results").and_then(Value::as_array);
    let timed = {
        let mut ops = Vec::new();
        if let Some(rows) = results {
            for row in rows {
                if let Some(op) = row.get("operation").and_then(Value::as_str) {
                    if !ops.contains(&op) {
                        ops.push(op);
                    }
                }
            }
        }
        ops.len()
    };
    let result_count = results.map(Vec::len).unwrap_or(0);

    let cpu = map(host, "cpu").cloned().unwrap_or_default();
    let toolchain = map(host, "toolchain").cloned().unwrap_or_default();

    // `refused or 'none'`: empty objects/arrays and other falsy values become "none".
    let refused = match host.get("refused") {
        Some(v) if truthy(Some(v)) => py_str(Some(v)),
        _ => "none".into(),
    };

    let lines = [
        format!("# `{}`", py_str(host.get("host"))),
        String::new(),
        format!(
            "- CPU: {} ({}, {} logical CPUs)",
            py_str(cpu.get("model")),
            py_str(cpu.get("arch")),
            py_str(cpu.get("logical_cpus"))
        ),
        format!("- OS: {}", py_str(host.get("os"))),
        format!(
            "- Toolchain: {}; {}",
            py_str(toolchain.get("rustc")),
            py_str(toolchain.get("cargo"))
        ),
        format!("- Commit: {}", py_str(host.get("commit"))),
        format!(
            "- Started / finished (UTC): {} / {}",
            py_str(host.get("started")),
            py_str(host.get("finished"))
        ),
        format!(
            "- Busy CPUs before build {}, before run {}, during run {} (harness limit {})",
            py_str(load.get("busy_cpus_before_build")),
            py_str(load.get("busy_cpus_before_run")),
            py_str(load.get("other_busy_cpus_during_run")),
            py_str(load.get("limit_cpus"))
        ),
        format!("- Noisy flag (harness): {}", py_str(load.get("noisy"))),
        format!("- Libraries refused for missing CPU features: {refused}"),
        format!("- OpenSSL linked: {}", py_str(openssl.get("linked"))),
        cost_line,
        format!("- Operations timed: {timed}; results: {result_count}"),
        String::new(),
    ];
    lines.join("\n")
}

/// Python truthiness for the values `or` sees.
fn truthy(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(m)) => !m.is_empty(),
    }
}

const LOG_KEEP: &str = r"^(Benchmarking \S+$|\s+time:|\s+thrpt:|host-run:|Warning:|error|bench-report|start |end |WATCHDOG|ABORT|libssl|.*Finished|.*failed|.*panicked)";

pub fn trim_log(text: &str) -> String {
    let keep = RegexHold::get();
    let mut kept: Vec<String> = Vec::new();
    for line in text.split('\n') {
        // split('\n') keeps a trailing empty from a final newline as an empty
        // line, which will not match LOG_KEEP. Python splitlines() drops the
        // final newline and does not yield a trailing empty. Equivalent for
        // matching purposes.
        let line = line.strip_suffix('\r').unwrap_or(line);
        if !keep.is_match(line) {
            continue;
        }
        if line.starts_with("Benchmarking ")
            && kept
                .last()
                .is_some_and(|prev| prev.starts_with("Benchmarking "))
        {
            *kept.last_mut().unwrap() = line.to_string();
        } else {
            kept.push(line.to_string());
        }
    }
    if kept.is_empty() {
        String::new()
    } else {
        let mut out = kept.join("\n");
        out.push('\n');
        out
    }
}

struct RegexHold;

impl RegexHold {
    fn get() -> &'static regex::Regex {
        use std::sync::OnceLock;
        static RE: OnceLock<regex::Regex> = OnceLock::new();
        RE.get_or_init(|| regex::Regex::new(LOG_KEEP).expect("LOG_KEEP"))
    }
}
