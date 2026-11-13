// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

//! Turn an Android capture into an `admin/bench-report` host JSON,
//! matching `android-convert.py`.

mod jsonfmt;

use serde_json::{Map, Number, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Inputs that correspond one-to-one with the Python script's flags.
pub struct ConvertArgs {
    pub rind: PathBuf,
    pub capture: PathBuf,
    pub out: PathBuf,
    pub host: String,
    pub commit: String,
    pub rustc: String,
    pub cargo: String,
    pub target: String,
    pub cpu_model: String,
    pub logical_cpus: i64,
    pub openssl_linked: String,
}

pub struct Converted {
    pub json: String,
    pub stdout: String,
}

struct Tables {
    operations: Vec<String>,
    libraries: Map<String, Value>,
    crates: Map<String, Value>,
    required: Option<Map<String, Value>>,
    has_host_arch: bool,
    locked_versions: Map<String, Value>,
    criterion_args: Vec<Value>,
}

pub fn convert(args: &ConvertArgs) -> Result<Converted, String> {
    if args.host.chars().any(|ch| ch.is_whitespace() || ch == '`') {
        return Err(
            "android-convert: host name must be a token without spaces or backticks".into(),
        );
    }

    let linked_path = args.capture.join("openssl-linked.txt");
    let mut linked = args.openssl_linked.trim().to_string();
    if linked.is_empty() && linked_path.is_file() {
        linked = read_text(&linked_path)?;
    }
    if linked.is_empty() {
        return Err("android-convert: pass --openssl-linked or provide openssl-linked.txt".into());
    }

    let rind = args
        .rind
        .canonicalize()
        .map_err(|err| format!("android-convert: {}: {err}", args.rind.display()))?;
    let tables = load_tables(&rind)?;

    let props = getprop(&args.capture.join("getprop.txt"))?;
    let (busy0, total0) = cpu_times(&read_text(&args.capture.join("stat-before"))?)?;
    let (busy1, total1) = cpu_times(&read_text(&args.capture.join("stat-after"))?)?;
    let busy_share = (busy1 - busy0) as f64 / (total1 - total0).max(1) as f64;
    let before = dumpsys_map(&read_text(&args.capture.join("battery-before.txt"))?);
    let after = dumpsys_map(&read_text(&args.capture.join("battery-after.txt"))?);
    let cpuid_path = args.capture.join("cpuid.json");
    let rind_cpu = if cpuid_path.is_file() {
        parse_json(&cpuid_path, &read_text(&cpuid_path)?)?
    } else {
        let mut cpuid = Map::new();
        cpuid.insert("cpuid".into(), Value::Object(Map::new()));
        Value::Object(cpuid)
    };

    let mut libraries = Map::new();
    for (library, crates) in &tables.crates {
        let crates = crates
            .as_array()
            .ok_or_else(|| "android-convert: CRATES values must be arrays".to_string())?;
        let mut versions = Map::new();
        for crate_name in crates {
            let crate_name = crate_name
                .as_str()
                .ok_or_else(|| "android-convert: CRATES crate names must be strings".to_string())?;
            let ver = match tables
                .locked_versions
                .get(crate_name)
                .and_then(Value::as_array)
            {
                Some(list) if !list.is_empty() => join_strings(list)?,
                _ => "?".to_string(),
            };
            versions.insert(crate_name.to_string(), Value::String(ver));
        }
        libraries.insert(library.clone(), Value::Object(versions));
    }
    match libraries.get_mut("openssl") {
        Some(Value::Object(openssl)) => {
            openssl.insert("linked".into(), Value::String(linked));
        }
        _ => {
            return Err("android-convert: CRATES has no openssl library".into());
        }
    }

    let release = prop(&props, "ro.build.version.release");
    let sdk = prop(&props, "ro.build.version.sdk");
    let build_id = prop(&props, "ro.build.id");
    let os = format!("Android {release} (API {sdk}, build {build_id})");

    let mut cpu = Map::new();
    cpu.insert("model".into(), Value::String(args.cpu_model.clone()));
    cpu.insert("arch".into(), Value::String("aarch64".into()));
    cpu.insert("logical_cpus".into(), Value::from(args.logical_cpus));

    let mut toolchain = Map::new();
    toolchain.insert("rustc".into(), Value::String(args.rustc.clone()));
    toolchain.insert("cargo".into(), Value::String(args.cargo.clone()));
    toolchain.insert("target".into(), Value::String(args.target.clone()));

    let limit = jsonfmt::python_round_digits((0.1 * args.logical_cpus as f64).max(1.0), 2);
    let mut load = Map::new();
    load.insert("limit_cpus".into(), fnum(limit));
    load.insert(
        "system_busy_share_during_run".into(),
        fnum(jsonfmt::python_round_digits(busy_share, 3)),
    );
    load.insert(
        "note".into(),
        Value::String(
            "whole-system busy share across the run, benchmarks included; \
             not the harness's other-work count"
                .into(),
        ),
    );
    load.insert("noisy".into(), Value::Null);

    let mut temperature_c = Map::new();
    temperature_c.insert("before".into(), temperature_c_value(&before));
    temperature_c.insert("after".into(), temperature_c_value(&after));
    let mut battery = Map::new();
    battery.insert("before".into(), Value::Object(before));
    battery.insert("after".into(), Value::Object(after));
    battery.insert("temperature_c".into(), Value::Object(temperature_c));

    let results = results_of(&tables, &args.capture)?;

    let mut build_cost = Map::new();
    build_cost.insert("status".into(), Value::String("failed".into()));
    build_cost.insert("started".into(), Value::String("".into()));
    build_cost.insert(
        "reason".into(),
        Value::String("not measured: the binaries were cross-compiled on another host".into()),
    );

    let mut host = Map::new();
    host.insert("schema".into(), Value::from(1));
    host.insert("host".into(), Value::String(args.host.clone()));
    host.insert("commit".into(), Value::String(args.commit.clone()));
    host.insert(
        "started".into(),
        Value::String(read_text(&args.capture.join("started"))?),
    );
    host.insert("cpu".into(), Value::Object(cpu));
    host.insert("os".into(), Value::String(os));
    host.insert("toolchain".into(), Value::Object(toolchain));
    host.insert("libraries".into(), Value::Object(libraries));
    host.insert("rind_cpu".into(), rind_cpu);
    host.insert("status".into(), Value::String("ok".into()));
    host.insert(
        "finished".into(),
        Value::String(read_text(&args.capture.join("finished"))?),
    );
    host.insert(
        "criterion_args".into(),
        Value::Array(tables.criterion_args.clone()),
    );
    host.insert("load".into(), Value::Object(load));
    host.insert("battery".into(), Value::Object(battery));
    host.insert("results".into(), Value::Array(results.clone()));
    host.insert("build_cost".into(), Value::Object(build_cost));
    host.insert(
        "refused".into(),
        Value::Object(refused_of(&tables, &Value::Object(host.clone()))?),
    );

    let json = jsonfmt::dumps(&Value::Object(host), Some(1)) + "\n";
    let stdout = format!("{} results; busy share {:.3}\n", results.len(), busy_share);
    Ok(Converted { json, stdout })
}

pub fn write_out(args: &ConvertArgs, converted: &Converted) -> Result<(), String> {
    if let Some(parent) = args.out.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("android-convert: {}: {err}", parent.display()))?;
        }
    }
    fs::write(&args.out, &converted.json)
        .map_err(|err| format!("android-convert: {}: {err}", args.out.display()))?;
    Ok(())
}

const EXTRACT: &str = r#"
import importlib.machinery, importlib.util, json, sys
path = sys.argv[1]
loader = importlib.machinery.SourceFileLoader("bench_report", path)
spec = importlib.util.spec_from_loader("bench_report", loader)
module = importlib.util.module_from_spec(spec)
loader.exec_module(module)
for name in (
    "operation_of",
    "OPERATIONS",
    "LIBRARIES",
    "CRATES",
    "locked_versions",
    "CRITERION_ARGS",
    "operation_regex",
):
    if not hasattr(module, name):
        sys.stderr.write(f"android-convert: admin/bench-report has no {name}\n")
        sys.exit(1)
print(
    json.dumps(
        {
            "operations": list(module.OPERATIONS),
            "libraries": module.LIBRARIES,
            "crates": module.CRATES,
            "required": module.REQUIRED if hasattr(module, "REQUIRED") else None,
            "has_host_arch": hasattr(module, "host_arch"),
            "locked_versions": module.locked_versions(),
            "criterion_args": list(module.CRITERION_ARGS) + [module.operation_regex(())],
        },
        separators=(",", ":"),
    )
)
"#;

fn load_tables(rind: &Path) -> Result<Tables, String> {
    let path = rind.join("admin").join("bench-report");
    if !path.is_file() {
        return Err(format!("android-convert: no {}", path.display()));
    }
    let out = Command::new("python3")
        .arg("-c")
        .arg(EXTRACT)
        .arg(&path)
        .output()
        .map_err(|err| format!("android-convert: python3: {err}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let err = err.trim();
        if err.is_empty() {
            return Err(format!(
                "android-convert: python3 failed extracting {path}",
                path = path.display()
            ));
        }
        return Err(err.to_string());
    }
    let tables: Value = serde_json::from_slice(&out.stdout)
        .map_err(|err| format!("android-convert: python3 extract was not JSON: {err}"))?;
    let obj = tables
        .as_object()
        .ok_or_else(|| "android-convert: extract must be an object".to_string())?;
    let operations = str_list(obj.get("operations"), "operations")?;
    let libraries = obj_map(obj.get("libraries"), "libraries")?;
    let crates = obj_map(obj.get("crates"), "crates")?;
    let required = match obj.get("required") {
        Some(Value::Null) | None => None,
        Some(Value::Object(m)) => Some(m.clone()),
        Some(_) => return Err("android-convert: REQUIRED must be an object".into()),
    };
    let has_host_arch = obj
        .get("has_host_arch")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let locked_versions = obj_map(obj.get("locked_versions"), "locked_versions")?;
    let criterion_args = match obj.get("criterion_args") {
        Some(Value::Array(a)) => a.clone(),
        _ => return Err("android-convert: criterion_args must be an array".into()),
    };
    Ok(Tables {
        operations,
        libraries,
        crates,
        required,
        has_host_arch,
        locked_versions,
        criterion_args,
    })
}

fn results_of(tables: &Tables, capture: &Path) -> Result<Vec<Value>, String> {
    let mut found = Vec::new();
    let mut paths = Vec::new();
    let criterion = capture.join("criterion");
    if criterion.is_dir() {
        walk_benchmarks(&criterion, &mut paths)?;
    }
    paths.sort();
    for path in paths {
        let meta = parse_json(&path, &read_raw(&path)?)?;
        let full_id = meta
            .get("full_id")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("android-convert: {}: missing full_id", path.display()))?;
        let (operation, library) = match operation_of(full_id, &tables.operations) {
            Some(pair) => pair,
            None => continue,
        };
        let estimates_path = path
            .parent()
            .ok_or_else(|| format!("android-convert: {}", path.display()))?
            .join("estimates.json");
        let estimates = parse_json(&estimates_path, &read_raw(&estimates_path)?)?;
        let estimate = estimates
            .get("slope")
            .or_else(|| estimates.get("mean"))
            .ok_or_else(|| {
                format!(
                    "android-convert: {}: no slope or mean",
                    estimates_path.display()
                )
            })?;
        let ns = estimate.get("point_estimate").cloned().ok_or_else(|| {
            format!(
                "android-convert: {}: missing point_estimate",
                estimates_path.display()
            )
        })?;
        let ci = estimate.get("confidence_interval").ok_or_else(|| {
            format!(
                "android-convert: {}: missing confidence_interval",
                estimates_path.display()
            )
        })?;
        let library_name = tables
            .libraries
            .get(&library)
            .and_then(Value::as_str)
            .unwrap_or(library.as_str())
            .to_string();
        let mut row = Map::new();
        row.insert("operation".into(), Value::String(operation));
        row.insert("library".into(), Value::String(library_name));
        row.insert("id".into(), Value::String(full_id.to_string()));
        row.insert("ns".into(), as_float(ns)?);
        row.insert(
            "ns_low".into(),
            as_float(ci.get("lower_bound").cloned().ok_or_else(|| {
                format!(
                    "android-convert: {}: missing lower_bound",
                    estimates_path.display()
                )
            })?)?,
        );
        row.insert(
            "ns_high".into(),
            as_float(ci.get("upper_bound").cloned().ok_or_else(|| {
                format!(
                    "android-convert: {}: missing upper_bound",
                    estimates_path.display()
                )
            })?)?,
        );
        row.insert(
            "throughput".into(),
            meta.get("throughput").cloned().unwrap_or(Value::Null),
        );
        found.push(Value::Object(row));
    }
    let order: Map<String, Value> = tables
        .operations
        .iter()
        .enumerate()
        .map(|(i, op)| (op.clone(), Value::from(i as u64)))
        .collect();
    found.sort_by(|a, b| {
        let ao = order
            .get(a["operation"].as_str().unwrap_or(""))
            .and_then(Value::as_u64)
            .unwrap_or(u64::MAX);
        let bo = order
            .get(b["operation"].as_str().unwrap_or(""))
            .and_then(Value::as_u64)
            .unwrap_or(u64::MAX);
        let al = a["library"].as_str().unwrap_or("");
        let bl = b["library"].as_str().unwrap_or("");
        ao.cmp(&bo).then_with(|| al.cmp(bl))
    });
    Ok(found)
}

fn refused_of(tables: &Tables, host: &Value) -> Result<Map<String, Value>, String> {
    let required = match (tables.required.as_ref(), tables.has_host_arch) {
        (Some(required), true) => required,
        _ => return Ok(Map::new()),
    };
    let cpuid = host
        .get("rind_cpu")
        .and_then(|v| v.get("cpuid"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let arch = host_arch(host)?;
    let mut refused = Map::new();
    for (library, features) in required {
        let features = match features.get(&arch).and_then(Value::as_array) {
            Some(f) => f,
            None => continue,
        };
        let mut missing = Vec::new();
        for feature in features {
            let feature = feature
                .as_str()
                .ok_or_else(|| "android-convert: REQUIRED features must be strings".to_string())?;
            let present = cpuid.get(feature).and_then(Value::as_bool).unwrap_or(false);
            if !present {
                missing.push(Value::String(feature.to_string()));
            }
        }
        if !missing.is_empty() {
            refused.insert(library.clone(), Value::Array(missing));
        }
    }
    Ok(refused)
}

fn host_arch(host: &Value) -> Result<String, String> {
    let arch = host
        .get("cpu")
        .and_then(|c| c.get("arch"))
        .and_then(Value::as_str)
        .ok_or_else(|| "android-convert: host.cpu.arch missing".to_string())?;
    Ok(match arch {
        "arm64" => "aarch64".into(),
        "amd64" => "x86_64".into(),
        other => other.to_string(),
    })
}

/// The operation and library of a criterion ID, or `None` if it is not one.
pub fn operation_of(full_id: &str, operations: &[String]) -> Option<(String, String)> {
    for op in operations {
        if let Some(library) = match_operation(op, full_id) {
            return Some((op.clone(), library));
        }
    }
    None
}

fn match_operation(op: &str, full_id: &str) -> Option<String> {
    let (prefix, suffix) = op.split_once("{lib}")?;
    if !full_id.starts_with(prefix) || !full_id.ends_with(suffix) {
        return None;
    }
    let mid_end = full_id.len() - suffix.len();
    if mid_end < prefix.len() {
        return None;
    }
    let mid = &full_id[prefix.len()..mid_end];
    if mid.is_empty() || mid.contains('/') {
        return None;
    }
    Some(mid.to_string())
}

fn walk_benchmarks(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|err| format!("android-convert: {}: {err}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("android-convert: {}: {err}", dir.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|err| format!("android-convert: {}: {err}", path.display()))?;
        if file_type.is_dir() {
            walk_benchmarks(&path, out)?;
        } else if file_type.is_file()
            && entry.file_name() == "benchmark.json"
            && path
                .parent()
                .and_then(|p| p.file_name())
                .is_some_and(|n| n == "new")
        {
            out.push(path);
        }
    }
    Ok(())
}

fn getprop(path: &Path) -> Result<Map<String, Value>, String> {
    let text = read_text(path)?;
    let mut props = Map::new();
    for line in text.lines() {
        if let Some((key, value)) = parse_getprop_line(line) {
            props.insert(key, Value::String(value));
        }
    }
    Ok(props)
}

fn parse_getprop_line(line: &str) -> Option<(String, String)> {
    if !line.starts_with('[') {
        return None;
    }
    let rest = &line[1..];
    let key_end = rest.find("]: [")?;
    let key = rest[..key_end].to_string();
    let after = &rest[key_end + 4..];
    let value = after.strip_suffix(']')?;
    Some((key, value.to_string()))
}

fn cpu_times(text: &str) -> Result<(i64, i64), String> {
    let fields: Vec<&str> = text.split_whitespace().collect();
    if fields.len() < 9 {
        return Err("android-convert: /proc/stat line is too short".into());
    }
    let mut vals = [0i64; 8];
    for (i, part) in fields[1..9].iter().enumerate() {
        vals[i] = part
            .parse()
            .map_err(|_| format!("android-convert: /proc/stat field is not an integer: {part}"))?;
    }
    let idle = vals[3] + vals[4];
    let total: i64 = vals.iter().sum();
    Ok((total - idle, total))
}

fn dumpsys_map(text: &str) -> Map<String, Value> {
    let mut out = Map::new();
    for line in text.lines() {
        if let Some((key, value)) = line.split_once(':') {
            out.insert(
                key.trim().to_string(),
                Value::String(value.trim().to_string()),
            );
        }
    }
    out
}

fn temperature_c_value(battery: &Map<String, Value>) -> Value {
    let raw = battery
        .get("temperature")
        .and_then(Value::as_str)
        .unwrap_or("0");
    match raw.parse::<i64>() {
        Ok(t) => fnum(t as f64 / 10.0),
        Err(_) => Value::Null,
    }
}

fn read_text(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|err| format!("android-convert: {}: {err}", path.display()))?;
    Ok(String::from_utf8_lossy(&bytes).trim().to_string())
}

fn read_raw(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|err| format!("android-convert: {}: {err}", path.display()))
}

fn parse_json(path: &Path, text: &str) -> Result<Value, String> {
    serde_json::from_str(text).map_err(|err| format!("android-convert: {}: {err}", path.display()))
}

fn prop(props: &Map<String, Value>, key: &str) -> String {
    props
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("?")
        .to_string()
}

fn str_list(v: Option<&Value>, name: &str) -> Result<Vec<String>, String> {
    let arr = v
        .and_then(Value::as_array)
        .ok_or_else(|| format!("android-convert: {name} must be an array"))?;
    arr.iter()
        .map(|x| {
            x.as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("android-convert: {name} entries must be strings"))
        })
        .collect()
}

fn obj_map(v: Option<&Value>, name: &str) -> Result<Map<String, Value>, String> {
    v.and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| format!("android-convert: {name} must be an object"))
}

fn join_strings(list: &[Value]) -> Result<String, String> {
    let mut parts = Vec::new();
    for v in list {
        let s = v
            .as_str()
            .ok_or_else(|| "android-convert: locked version is not a string".to_string())?;
        parts.push(s);
    }
    Ok(parts.join(", "))
}

fn fnum(x: f64) -> Value {
    Value::Number(Number::from_f64(x).expect("finite float"))
}

fn as_float(v: Value) -> Result<Value, String> {
    match v {
        Value::Number(n) => {
            let x = n
                .as_f64()
                .ok_or_else(|| "android-convert: estimate is not a number".to_string())?;
            Ok(fnum(x))
        }
        _ => Err("android-convert: estimate is not a number".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getprop_parses_bracket_lines() {
        let (k, v) = parse_getprop_line("[ro.build.id]: [AP4A.250205.002]").unwrap();
        assert_eq!(k, "ro.build.id");
        assert_eq!(v, "AP4A.250205.002");
        assert!(parse_getprop_line("unrelated: without brackets is ignored").is_none());
    }

    #[test]
    fn operation_of_matches_lib_segment() {
        let ops = vec!["aes128-gcm/{lib}/32B".into(), "x25519-ecdh/{lib}".into()];
        assert_eq!(
            operation_of("aes128-gcm/rind/32B", &ops),
            Some(("aes128-gcm/{lib}/32B".into(), "rind".into()))
        );
        assert_eq!(
            operation_of("x25519-ecdh/openssl", &ops),
            Some(("x25519-ecdh/{lib}".into(), "openssl".into()))
        );
        assert_eq!(operation_of("not-an-operation/unknown", &ops), None);
        assert_eq!(operation_of("aes128-gcm/foo/bar/32B", &ops), None);
    }

    #[test]
    fn cpu_times_ignores_idle_and_iowait() {
        let (busy, total) = cpu_times("cpu  100 0 50 800 20 0 0 0 0 0").unwrap();
        assert_eq!(busy, 150);
        assert_eq!(total, 970);
    }
}
