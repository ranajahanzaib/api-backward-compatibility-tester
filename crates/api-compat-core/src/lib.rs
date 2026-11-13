//! Reusable API contract comparison and recorded-scenario replay primitives.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    #[serde(default)]
    pub classification: String,
    pub severity: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub consumer: Option<String>,
    pub evidence: String,
    pub location: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    #[serde(alias = "id")]
    pub name: String,
    #[serde(default)]
    pub consumer: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub query: BTreeMap<String, String>,
    pub body: Option<Value>,
    #[serde(default)]
    pub expect_status: Option<u16>,
    #[serde(default)]
    pub allowed_statuses: Vec<u16>,
    #[serde(default)]
    pub expect_content_type: Option<String>,
    #[serde(default)]
    pub assertions: Vec<Assertion>,
    #[serde(default)]
    pub scenario_id: Option<String>,
    #[serde(default)]
    pub step_id: Option<String>,
    #[serde(default)]
    pub captures: BTreeMap<String, String>,
    #[serde(default)]
    pub request_body_redacted: bool,
    #[serde(default)]
    pub steps: Vec<ScenarioStep>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScenarioStep {
    pub id: String,
    #[serde(default)]
    pub request: StepRequest,
    #[serde(default)]
    pub expect: StepExpect,
    #[serde(default)]
    pub capture: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepRequest {
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub query: BTreeMap<String, String>,
    #[serde(default)]
    pub body: Option<Value>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepExpect {
    pub status: Option<u16>,
    #[serde(default)]
    pub allowed_statuses: Vec<u16>,
    pub content_type: Option<String>,
    #[serde(default)]
    pub json: Vec<Assertion>,
}
fn default_method() -> String {
    "GET".into()
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assertion {
    #[serde(default)]
    pub pointer: String,
    #[serde(default)]
    pub equals: Option<Value>,
    #[serde(rename = "type", default)]
    pub value_type: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default, alias = "enum")]
    pub enum_values: Vec<Value>,
    #[serde(default)]
    pub min_items: Option<usize>,
    #[serde(default)]
    pub max_items: Option<usize>,
    #[serde(default)]
    pub header: Option<String>,
    #[serde(default)]
    pub header_equals: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub name: String,
    #[serde(default)]
    pub consumer: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub scenario_id: Option<String>,
    #[serde(default)]
    pub step_id: Option<String>,
    #[serde(default)]
    pub captures: BTreeMap<String, String>,
    pub method: String,
    pub path: String,
    pub headers: BTreeMap<String, String>,
    pub query: BTreeMap<String, String>,
    pub body: Option<Value>,
    pub status: u16,
    #[serde(default)]
    pub allowed_statuses: Vec<u16>,
    #[serde(default)]
    pub expect_content_type: Option<String>,
    pub response: Value,
    #[serde(default)]
    pub request_body_redacted: bool,
    #[serde(default)]
    pub response_headers: BTreeMap<String, String>,
    #[serde(default)]
    pub elapsed_ms: u128,
    pub assertions: Vec<Assertion>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingBundle {
    pub recording_version: u32,
    pub baseline: BaselineMetadata,
    pub scenarios: Vec<Recording>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetadata {
    pub openapi_fingerprint: String,
    pub captured_at_unix: u64,
    pub scenario_count: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub name: String,
    #[serde(default)]
    pub consumer: Option<String>,
    #[serde(default)]
    pub step_id: Option<String>,
    pub passed: bool,
    #[serde(default = "default_executed")]
    pub executed: bool,
    pub status: u16,
    pub findings: Vec<Finding>,
}
fn default_executed() -> bool {
    true
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    #[serde(default = "report_schema_version")]
    pub schema_version: u32,
    pub findings: Vec<Finding>,
    pub scenarios: Vec<ScenarioResult>,
    pub passed: bool,
    #[serde(default)]
    pub consumers: Vec<ConsumerImpact>,
    #[serde(default)]
    pub summary: ReportSummary,
    #[serde(default)]
    pub tool_version: String,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub contracts: BTreeMap<String, String>,
    #[serde(default)]
    pub coverage: ReportCoverage,
    #[serde(default)]
    pub policy: GatePolicy,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportCoverage {
    pub operations_total: usize,
    pub operations_with_scenarios: usize,
    pub operations_without_scenarios: usize,
    pub inconclusive_checks: usize,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsumerImpact {
    pub consumer: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportSummary {
    pub decision: String,
    pub scenarios_total: usize,
    pub scenarios_passed: usize,
    pub scenarios_failed: usize,
    pub scenarios_skipped: usize,
    pub confirmed_breaks: usize,
    pub static_breaks: usize,
    pub potential_breaks: usize,
    pub inconclusive: usize,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GatePolicy {
    pub fail_on: Vec<String>,
    pub fail_on_inconclusive: bool,
}
fn report_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone)]
pub struct HttpOptions {
    pub allow_state_changing: bool,
    pub allow_remote: bool,
    pub timeout_seconds: u64,
    pub max_response_bytes: usize,
    pub max_request_bytes: usize,
    pub redacted_headers: Vec<String>,
    pub redacted_json_pointers: Vec<String>,
    pub ignore_json_pointers: Vec<String>,
    pub ignore_headers: Vec<String>,
    pub ignore_array_order: Vec<String>,
}
type RequestParts = (
    String,
    BTreeMap<String, String>,
    BTreeMap<String, String>,
    Option<Value>,
);
impl Default for HttpOptions {
    fn default() -> Self {
        Self {
            allow_state_changing: false,
            allow_remote: false,
            timeout_seconds: 10,
            max_response_bytes: 2 * 1024 * 1024,
            max_request_bytes: 1024 * 1024,
            redacted_headers: vec!["authorization".into(), "cookie".into(), "set-cookie".into()],
            redacted_json_pointers: vec![],
            ignore_json_pointers: vec![],
            ignore_headers: vec!["date".into(), "server".into()],
            ignore_array_order: vec![],
        }
    }
}

pub fn read_document(path: impl AsRef<Path>) -> Result<Value> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value = if path
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.eq_ignore_ascii_case("json"))
    {
        serde_json::from_str(&raw).with_context(|| format!("parse JSON {}", path.display()))?
    } else {
        serde_yaml::from_str(&raw).with_context(|| format!("parse YAML {}", path.display()))?
    };
    let root = path
        .parent()
        .unwrap_or(Path::new("."))
        .canonicalize()
        .with_context(|| format!("resolve contract directory {}", path.display()))?;
    let source = path
        .canonicalize()
        .with_context(|| format!("resolve contract path {}", path.display()))?;
    resolve_local_references(value, &source, &root, 0, &mut Vec::new())
}

fn resolve_local_references(
    value: Value,
    source: &Path,
    root: &Path,
    depth: usize,
    active: &mut Vec<String>,
) -> Result<Value> {
    if depth > 64 {
        anyhow::bail!(
            "OpenAPI reference depth exceeded 64 at {}",
            source.display()
        );
    }
    match value {
        Value::Object(object) => {
            if let Some(reference) = object
                .get("$ref")
                .and_then(Value::as_str)
                .map(str::to_owned)
            {
                let (file, pointer) = reference.split_once('#').unwrap_or((&reference, ""));
                if file.starts_with("http:") || file.starts_with("https:") {
                    return Ok(Value::Object(object));
                }
                let target_file = if file.is_empty() {
                    source.to_path_buf()
                } else {
                    source.parent().unwrap_or(Path::new(".")).join(file)
                };
                let target_file = target_file
                    .canonicalize()
                    .with_context(|| format!("resolve local reference {reference}"))?;
                if !target_file.starts_with(root) {
                    anyhow::bail!(
                        "local reference {reference} escapes the OpenAPI contract directory"
                    );
                }
                let key = format!("{}#{}", target_file.display(), pointer);
                if active.contains(&key) {
                    return Ok(Value::Object(object));
                }
                let target = load_document_unresolved(&target_file)?;
                let pointer = pointer.strip_prefix('#').unwrap_or(pointer);
                let selected = if pointer.is_empty() {
                    Some(target.clone())
                } else {
                    target.pointer(pointer).cloned()
                };
                let Some(selected) = selected else {
                    anyhow::bail!(
                        "unresolved local reference {reference} in {}",
                        source.display()
                    );
                };
                active.push(key);
                let resolved =
                    resolve_local_references(selected, &target_file, root, depth + 1, active)?;
                active.pop();
                // OpenAPI 3.1 permits siblings beside $ref. Preserve them as overlays.
                let mut merged = resolved.as_object().cloned().unwrap_or_default();
                for (key, value) in object {
                    if key != "$ref" {
                        merged.insert(
                            key,
                            resolve_local_references(value, source, root, depth + 1, active)?,
                        );
                    }
                }
                return Ok(Value::Object(merged));
            }
            let mut resolved = serde_json::Map::new();
            for (key, child) in object {
                resolved.insert(
                    key,
                    resolve_local_references(child, source, root, depth + 1, active)?,
                );
            }
            Ok(Value::Object(resolved))
        }
        Value::Array(items) => Ok(Value::Array(
            items
                .into_iter()
                .map(|item| resolve_local_references(item, source, root, depth + 1, active))
                .collect::<Result<Vec<_>>>()?,
        )),
        other => Ok(other),
    }
}

fn load_document_unresolved(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read referenced OpenAPI file {}", path.display()))?;
    if path
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.eq_ignore_ascii_case("json"))
    {
        Ok(serde_json::from_str(&raw)?)
    } else {
        Ok(serde_yaml::from_str(&raw)?)
    }
}

/// Compare the consumer-facing behavior of two OpenAPI documents. Unsupported schema
/// constructs are reported as inconclusive instead of being silently treated as compatible.
pub fn diff(old: &Value, new: &Value) -> Vec<Finding> {
    let mut out = Vec::new();
    collect_unresolved_refs(old, "old contract", &mut out);
    collect_unresolved_refs(new, "new contract", &mut out);
    let supported = |v: &Value| {
        v.get("openapi")
            .and_then(Value::as_str)
            .is_some_and(|version| {
                version.starts_with("3.0.")
                    || version.starts_with("3.1.")
                    || version.starts_with("3.2.")
            })
    };
    if !supported(old) || !supported(new) {
        out.push(finding(
            "openapi-version-unsupported",
            "warning",
            "inconclusive",
            "openapi",
            "Both contracts must declare a supported OpenAPI 3.0, 3.1, or 3.2 version",
        ));
        return out;
    }
    let (Some(op), Some(np)) = (
        old.get("paths").and_then(Value::as_object),
        new.get("paths").and_then(Value::as_object),
    ) else {
        return vec![finding(
            "invalid-contract",
            "error",
            "inconclusive",
            "paths",
            "OpenAPI document must define an object-valued paths field",
        )];
    };
    for (path, methods) in op {
        let Some((_, new_item)) = np
            .iter()
            .find(|(new_path, _)| normalize_path(new_path) == normalize_path(path))
        else {
            out.push(finding(
                "operation-removed",
                "error",
                "static_break",
                path,
                "Endpoint was removed",
            ));
            continue;
        };
        let Some(new_methods) = new_item.as_object() else {
            continue;
        };
        let Some(old_methods) = methods.as_object() else {
            continue;
        };
        for (method, old_op) in old_methods {
            if !is_method(method) {
                continue;
            }
            let loc = format!("{} {}", method.to_uppercase(), path);
            let Some(new_op) = new_methods.get(method) else {
                out.push(finding(
                    "operation-removed",
                    "error",
                    "static_break",
                    &loc,
                    "HTTP operation was removed",
                ));
                continue;
            };
            compare_parameters(
                old_op
                    .get("parameters")
                    .or_else(|| methods.get("parameters")),
                new_op
                    .get("parameters")
                    .or_else(|| new_item.get("parameters")),
                &loc,
                &mut out,
            );
            compare_security(
                old_op.get("security").or(old.get("security")),
                new_op.get("security").or(new.get("security")),
                &loc,
                &mut out,
            );
            compare_media(
                old_op.get("requestBody"),
                new_op.get("requestBody"),
                &loc,
                "request",
                &mut out,
            );
            compare_responses(
                old_op.get("responses"),
                new_op.get("responses"),
                &loc,
                &mut out,
            );
        }
    }
    out.sort_by(|a, b| (&a.location, &a.id).cmp(&(&b.location, &b.id)));
    out
}
fn normalize_path(path: &str) -> String {
    path.split('/')
        .map(|part| {
            if part.starts_with('{') && part.ends_with('}') {
                "{}"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}
fn compare_security(
    old: Option<&Value>,
    new: Option<&Value>,
    location: &str,
    out: &mut Vec<Finding>,
) {
    let old = old.and_then(Value::as_array).cloned().unwrap_or_default();
    let new = new.and_then(Value::as_array).cloned().unwrap_or_default();
    if old.is_empty() && !new.is_empty() {
        out.push(finding("security-tightened","error","static_break",location,"Operation now requires authentication where the old contract allowed anonymous requests"));
        return;
    }
    if new.is_empty() {
        return;
    }
    let all_old_options_preserved = old.iter().all(|old_requirement| {
        new.iter().any(|new_requirement| {
            security_requirement_is_no_stricter(old_requirement, new_requirement)
        })
    });
    if !all_old_options_preserved {
        out.push(finding("security-tightened","error","static_break",location,"New security requirements remove a previously accepted authentication option or add required scopes"));
    }
}
fn security_requirement_is_no_stricter(old: &Value, new: &Value) -> bool {
    let Some(old) = old.as_object() else {
        return false;
    };
    let Some(new) = new.as_object() else {
        return false;
    };
    new.iter().all(|(scheme, new_scopes)| {
        let Some(old_scopes) = old.get(scheme).and_then(Value::as_array) else {
            return false;
        };
        let Some(new_scopes) = new_scopes.as_array() else {
            return false;
        };
        new_scopes.iter().all(|scope| old_scopes.contains(scope))
    })
}
fn collect_unresolved_refs(value: &Value, location: &str, out: &mut Vec<Finding>) {
    match value {
        Value::Object(map) => {
            if let Some(reference) = map.get("$ref").and_then(Value::as_str) {
                out.push(finding(
                    "reference-inconclusive",
                    "warning",
                    "inconclusive",
                    location,
                    &format!("Reference '{reference}' could not be resolved safely"),
                ));
            }
            for (key, child) in map {
                collect_unresolved_refs(child, &format!("{location}/{key}"), out);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_unresolved_refs(child, &format!("{location}/{index}"), out);
            }
        }
        _ => {}
    }
}
fn is_method(s: &str) -> bool {
    matches!(
        s,
        "get" | "put" | "post" | "delete" | "options" | "head" | "patch" | "trace"
    )
}
fn finding(id: &str, severity: &str, evidence: &str, location: &str, message: &str) -> Finding {
    Finding {
        id: id.into(),
        classification: evidence.into(),
        severity: match severity {
            "error" if evidence == "confirmed_break" => "critical",
            "error" => "high",
            "warning" => "medium",
            other => other,
        }
        .into(),
        source: if matches!(
            evidence,
            "static_break" | "potential_break" | "inconclusive"
        ) {
            "static"
        } else {
            "replay"
        }
        .into(),
        consumer: None,
        evidence: evidence.into(),
        location: location.into(),
        message: message.into(),
    }
}
fn compare_parameters(old: Option<&Value>, new: Option<&Value>, loc: &str, out: &mut Vec<Finding>) {
    let arr = |v: Option<&Value>| v.and_then(Value::as_array).cloned().unwrap_or_default();
    let old = arr(old);
    let new = arr(new);
    for n in &new {
        if n.get("required").and_then(Value::as_bool) == Some(true) {
            let key = parameter_key(n, &new);
            let existed = old.iter().any(|o| parameter_key(o, &old) == key);
            if !existed {
                out.push(finding(
                    "required-parameter-added",
                    "error",
                    "static_break",
                    loc,
                    &format!(
                        "New required {} parameter '{}' rejects previously valid requests",
                        key.0, key.1
                    ),
                ));
            }
        }
    }
    for o in &old {
        let key = parameter_key(o, &old);
        if let Some(n) = new.iter().find(|n| parameter_key(n, &new) == key) {
            if o.get("required").and_then(Value::as_bool) == Some(false)
                && n.get("required").and_then(Value::as_bool) == Some(true)
            {
                out.push(finding(
                    "parameter-now-required",
                    "error",
                    "static_break",
                    loc,
                    &format!("Parameter '{}' is now required", key.1),
                ));
            }
            if o.get("style") != n.get("style")
                || o.get("explode") != n.get("explode")
                || o.get("allowReserved") != n.get("allowReserved")
            {
                out.push(finding(
                    "parameter-serialization-changed",
                    "error",
                    "static_break",
                    loc,
                    &format!("Serialization behavior changed for parameter '{}'", key.1),
                ));
            }
            compare_schema(
                o.get("schema"),
                n.get("schema"),
                true,
                &format!("{loc} parameter {}", key.1),
                out,
            );
        } else if matches!(
            o.get("in").and_then(Value::as_str),
            Some("query" | "header" | "cookie")
        ) {
            out.push(finding(
                "parameter-removed",
                "warning",
                "potential_break",
                loc,
                &format!(
                    "Previously accepted parameter '{}' is no longer documented",
                    key.1
                ),
            ));
        }
    }
}
fn parameter_key(parameter: &Value, all: &[Value]) -> (String, String) {
    let location = parameter
        .get("in")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_owned();
    if location == "path" {
        let index = all
            .iter()
            .filter(|p| p.get("in").and_then(Value::as_str) == Some("path"))
            .position(|p| std::ptr::eq(p, parameter))
            .unwrap_or(0);
        (location, format!("path parameter {index}"))
    } else {
        (
            location,
            parameter
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_owned(),
        )
    }
}
fn compare_media(
    old: Option<&Value>,
    new: Option<&Value>,
    loc: &str,
    kind: &str,
    out: &mut Vec<Finding>,
) {
    if kind == "request" {
        let was_required =
            old.and_then(|v| v.get("required")).and_then(Value::as_bool) == Some(true);
        let now_required =
            new.and_then(|v| v.get("required")).and_then(Value::as_bool) == Some(true);
        if !was_required && now_required {
            out.push(finding(
                "request-body-now-required",
                "error",
                "static_break",
                loc,
                "Request body is now required",
            ));
        }
    }
    let Some(content) = old
        .and_then(|v| v.get("content"))
        .and_then(Value::as_object)
    else {
        return;
    };
    let fresh = new
        .and_then(|v| v.get("content"))
        .and_then(Value::as_object);
    for media in content.keys() {
        if !fresh.is_some_and(|f| f.contains_key(media)) {
            out.push(finding(
                "media-type-removed",
                "error",
                "static_break",
                loc,
                &format!("{kind} media type {media} was removed"),
            ));
        } else if kind == "request" {
            if let Some(new_media) = fresh.and_then(|f| f.get(media)) {
                compare_schema(
                    content.get(media).and_then(|m| m.get("schema")),
                    new_media.get("schema"),
                    true,
                    &format!("{loc} request body"),
                    out,
                );
            }
        }
    }
}
fn compare_responses(old: Option<&Value>, new: Option<&Value>, loc: &str, out: &mut Vec<Finding>) {
    let Some(old) = old.and_then(Value::as_object) else {
        return;
    };
    let new = new.and_then(Value::as_object);
    for (status, r) in old {
        let Some(n) = new.and_then(|m| m.get(status)) else {
            out.push(finding(
                "response-removed",
                "error",
                "static_break",
                loc,
                &format!("Previously documented response status {status} was removed"),
            ));
            continue;
        };
        let oc = r.get("content").and_then(Value::as_object);
        let nc = n.get("content").and_then(Value::as_object);
        if let Some(oc) = oc {
            for (media, m) in oc {
                if let Some(nm) = nc.and_then(|x| x.get(media)) {
                    compare_schema(
                        m.get("schema"),
                        nm.get("schema"),
                        false,
                        &format!("{loc} response {status}"),
                        out,
                    );
                } else {
                    out.push(finding(
                        "media-type-removed",
                        "error",
                        "static_break",
                        loc,
                        &format!("Response media type {media} was removed from status {status}"),
                    ));
                }
            }
        }
        let old_headers = r.get("headers").and_then(Value::as_object);
        let new_headers = n.get("headers").and_then(Value::as_object);
        if let Some(headers) = old_headers {
            for name in headers.keys() {
                if !new_headers.is_some_and(|h| h.contains_key(name)) {
                    out.push(finding(
                        "response-header-removed",
                        "error",
                        "static_break",
                        loc,
                        &format!("Previously documented response header {name} was removed"),
                    ));
                }
            }
        }
    }
}
fn compare_schema(
    old: Option<&Value>,
    new: Option<&Value>,
    request: bool,
    loc: &str,
    out: &mut Vec<Finding>,
) {
    let (Some(o), Some(n)) = (old, new) else {
        return;
    };
    let old_types = schema_types(o);
    let new_types = schema_types(n);
    if let (Some(old_types), Some(new_types)) = (&old_types, &new_types) {
        let incompatible = if request {
            old_types.iter().any(|ty| !new_types.contains(ty))
        } else {
            new_types.iter().any(|ty| !old_types.contains(ty))
        };
        if incompatible {
            out.push(finding(
                "schema-type-changed",
                "error",
                "static_break",
                loc,
                &format!(
                    "{} schema allowed types changed from {:?} to {:?}",
                    if request { "Request" } else { "Response" },
                    old_types,
                    new_types
                ),
            ));
        }
    }
    let old_nullable = allows_null(o);
    let new_nullable = allows_null(n);
    if (request && old_nullable && !new_nullable) || (!request && !old_nullable && new_nullable) {
        out.push(finding(
            "schema-nullability-changed",
            "error",
            "static_break",
            loc,
            if request {
                "Request schema no longer accepts null"
            } else {
                "Response schema may now return null"
            },
        ));
    }
    compare_bounds(o, n, request, loc, out);
    let old_enum = o.get("enum").and_then(Value::as_array);
    let new_enum = n.get("enum").and_then(Value::as_array);
    if request && old_enum.is_none() && new_enum.is_some() {
        out.push(finding(
            "request-enum-narrowed",
            "error",
            "static_break",
            loc,
            "Request schema now restricts previously unrestricted values to an enum",
        ));
    } else if request
        && old_enum
            .zip(new_enum)
            .is_some_and(|(a, b)| a.iter().any(|v| !b.contains(v)))
    {
        let removed = true;
        if removed {
            out.push(finding(
                "request-enum-narrowed",
                "error",
                "static_break",
                loc,
                "Request enum no longer accepts every previously valid value",
            ));
        }
    }
    if !request && old_enum.is_some() && new_enum.is_none() {
        out.push(finding(
            "response-enum-expanded",
            "error",
            "static_break",
            loc,
            "Response enum restriction was removed; the old client may receive an unknown value",
        ));
    } else if !request
        && old_enum
            .zip(new_enum)
            .is_some_and(|(a, b)| b.iter().any(|v| !a.contains(v)))
    {
        out.push(finding(
            "response-enum-expanded",
            "error",
            "static_break",
            loc,
            "Response enum now permits values excluded by the old client contract",
        ));
    }
    let op = o.get("properties").and_then(Value::as_object);
    let np = n.get("properties").and_then(Value::as_object);
    if let (Some(op), Some(np)) = (op, np) {
        let old_req = o.get("required").and_then(Value::as_array);
        let new_req = n.get("required").and_then(Value::as_array);
        for (name, os) in op {
            if let Some(ns) = np.get(name) {
                let was_required =
                    old_req.is_some_and(|r| r.contains(&Value::String(name.clone())));
                let remains_required =
                    new_req.is_some_and(|r| r.contains(&Value::String(name.clone())));
                if !request && was_required && !remains_required {
                    out.push(finding(
                        "response-property-now-optional",
                        "error",
                        "static_break",
                        &format!("{loc}.{name}"),
                        "Response property is no longer guaranteed to be present",
                    ));
                }
                compare_schema(Some(os), Some(ns), request, &format!("{loc}.{name}"), out);
            } else if !request && old_req.is_some_and(|r| r.contains(&Value::String(name.clone())))
            {
                out.push(finding(
                    "response-property-removed",
                    "error",
                    "static_break",
                    &format!("{loc}.{name}"),
                    "Previously required response property was removed",
                ));
            } else if !request {
                out.push(finding(
                    "optional-response-property-removed",
                    "warning",
                    "potential_break",
                    &format!("{loc}.{name}"),
                    "Response property was removed; impact depends on consumer expectations",
                ));
            } else if n.get("additionalProperties").and_then(Value::as_bool) == Some(false) {
                out.push(finding(
                    "request-property-removed",
                    "error",
                    "static_break",
                    &format!("{loc}.{name}"),
                    "Previously accepted request property is now disallowed",
                ));
            }
        }
        if request {
            for name in new_req.into_iter().flatten().filter_map(Value::as_str) {
                if !op.contains_key(name) {
                    out.push(finding(
                        "required-property-added",
                        "error",
                        "static_break",
                        &format!("{loc}.{name}"),
                        "Request property is newly required",
                    ));
                }
            }
        }
    }
    if let (Some(old_items), Some(new_items)) = (o.get("items"), n.get("items")) {
        compare_schema(
            Some(old_items),
            Some(new_items),
            request,
            &format!("{loc}/*"),
            out,
        );
    }
    if request
        && o.get("additionalProperties").and_then(Value::as_bool) != Some(false)
        && n.get("additionalProperties").and_then(Value::as_bool) == Some(false)
    {
        out.push(finding(
            "request-properties-restricted",
            "error",
            "static_break",
            loc,
            "Request no longer permits additional properties",
        ));
    }
    if o.get("$ref").is_some()
        || n.get("$ref").is_some()
        || o.get("allOf").is_some()
        || n.get("allOf").is_some()
        || o.get("oneOf").is_some()
        || n.get("oneOf").is_some()
    {
        out.push(finding(
            "schema-inconclusive",
            "warning",
            "inconclusive",
            loc,
            "Schema reference or composition requires resolution beyond this structural comparison",
        ));
    }
}

fn schema_types(schema: &Value) -> Option<Vec<String>> {
    let mut types = match schema.get("type")? {
        Value::String(s) => vec![s.clone()],
        Value::Array(items) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
        _ => return None,
    };
    if schema.get("nullable").and_then(Value::as_bool) == Some(true)
        && !types.iter().any(|t| t == "null")
    {
        types.push("null".into());
    }
    types.sort();
    types.dedup();
    Some(types)
}
fn allows_null(schema: &Value) -> bool {
    schema.get("nullable").and_then(Value::as_bool) == Some(true)
        || schema
            .get("type")
            .and_then(Value::as_array)
            .is_some_and(|types| types.iter().any(|t| t.as_str() == Some("null")))
}
fn compare_bounds(old: &Value, new: &Value, request: bool, location: &str, out: &mut Vec<Finding>) {
    for name in [
        "minimum",
        "exclusiveMinimum",
        "maximum",
        "exclusiveMaximum",
        "minLength",
        "maxLength",
        "minItems",
        "maxItems",
    ] {
        let old_bound = old.get(name).and_then(Value::as_f64);
        let new_bound = new.get(name).and_then(Value::as_f64);
        let lower = matches!(
            name,
            "minimum" | "exclusiveMinimum" | "minLength" | "minItems"
        );
        let absent_breaks = if request {
            old_bound.is_none() && new_bound.is_some()
        } else {
            old_bound.is_some() && new_bound.is_none()
        };
        if absent_breaks {
            out.push(finding("schema-constraint-tightened","error","static_break",location,&format!("{name} constraint was introduced or removed in a direction incompatible with the old {} contract",if request{"request"}else{"response"})));
        } else if let (Some(a), Some(b)) = (old_bound, new_bound) {
            let more_restrictive = match name {
                _ if lower => b > a,
                _ => b < a,
            };
            let breaking = if request {
                more_restrictive
            } else {
                !more_restrictive
            } && a != b;
            if breaking {
                out.push(finding("schema-constraint-tightened", "error", "static_break", location, &format!("{name} changed from {a} to {b} in a direction incompatible with the old {} contract", if request{"request"}else{"response"})));
            }
        }
    }
}

fn validate_target(base_url: &str, allow_remote: bool) -> Result<()> {
    let url = reqwest::Url::parse(base_url).context("candidate base URL must be absolute")?;
    if !matches!(url.scheme(), "http" | "https") {
        anyhow::bail!("only HTTP and HTTPS targets are supported");
    }
    let host = url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("target URL has no host"))?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        anyhow::bail!("base URL must not contain credentials, a query, or a fragment");
    }
    let is_loopback = host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback());
    if !is_loopback && !allow_remote {
        anyhow::bail!("remote target '{host}' is blocked by default; pass --allow-remote only for a trusted environment");
    }
    Ok(())
}

fn interpolate_with_vars(input: &str, variables: &BTreeMap<String, Value>) -> Result<String> {
    let mut output = String::new();
    let mut rest = input;
    while let Some(start) = rest.find("${") {
        output.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find('}') else {
            anyhow::bail!("unterminated environment placeholder in request");
        };
        let key = &after[..end];
        if let Some(variable) = key.strip_prefix("steps.") {
            let value = variables
                .get(variable)
                .ok_or_else(|| anyhow::anyhow!("required scenario variable is unavailable"))?;
            if let Some(string) = value.as_str() {
                output.push_str(string);
            } else {
                output.push_str(&value.to_string());
            }
        } else if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            anyhow::bail!("invalid environment variable name in request placeholder");
        } else {
            let value = std::env::var(key)
                .with_context(|| format!("required environment variable {key} is not set"))?;
            output.push_str(&value);
        }
        rest = &after[end + 1..];
    }
    output.push_str(rest);
    Ok(output)
}
fn interpolate_value(value: &Value, variables: &BTreeMap<String, Value>) -> Result<Value> {
    match value {
        Value::String(s) => Ok(Value::String(interpolate_with_vars(s, variables)?)),
        Value::Array(a) => Ok(Value::Array(
            a.iter()
                .map(|v| interpolate_value(v, variables))
                .collect::<Result<Vec<_>>>()?,
        )),
        Value::Object(o) => Ok(Value::Object(
            o.iter()
                .map(|(k, v)| Ok((k.clone(), interpolate_value(v, variables)?)))
                .collect::<Result<_>>()?,
        )),
        _ => Ok(value.clone()),
    }
}
fn redact_headers(
    headers: &BTreeMap<String, String>,
    names: &[String],
) -> BTreeMap<String, String> {
    headers
        .iter()
        .map(|(key, value)| {
            let sensitive = names.iter().any(|name| name.eq_ignore_ascii_case(key));
            let placeholder = value.contains("${") && value.contains('}');
            (
                key.clone(),
                if sensitive && !placeholder {
                    "[REDACTED]".into()
                } else {
                    value.clone()
                },
            )
        })
        .collect()
}
fn redact_json(mut value: Value, pointers: &[String]) -> Value {
    for pointer in pointers {
        if let Some(target) = value.pointer_mut(pointer) {
            let has_template = target
                .as_str()
                .is_some_and(|v| v.contains("${") && v.contains('}'));
            if !has_template {
                *target = Value::String("[REDACTED]".into());
            }
        }
    }
    value
}
fn has_redacted_pointer(value: &Value, pointers: &[String]) -> bool {
    pointers.iter().any(|pointer| {
        value.pointer(pointer).is_some_and(|v| {
            !v.as_str()
                .is_some_and(|s| s.contains("${") && s.contains('}'))
        })
    })
}
fn normalize_response(mut value: Value, ignored: &[String], unordered: &[String]) -> Value {
    for pointer in ignored {
        remove_pointer(&mut value, pointer);
    }
    for pointer in unordered {
        if let Some(Value::Array(items)) = value.pointer_mut(pointer) {
            items.sort_by_key(|v| serde_json::to_string(v).unwrap_or_default());
        }
    }
    value
}
fn remove_pointer(value: &mut Value, pointer: &str) {
    let Some((parent, last)) = pointer.rsplit_once('/') else {
        return;
    };
    let parent = if parent.is_empty() { "" } else { parent };
    if let Some(Value::Object(map)) = value.pointer_mut(parent) {
        map.remove(&last.replace("~1", "/").replace("~0", "~"));
    } else if let Some(Value::Array(items)) = value.pointer_mut(parent) {
        if let Ok(index) = last.parse::<usize>() {
            if index < items.len() {
                items.remove(index);
            }
        }
    }
}
fn normalize_headers(
    headers: &BTreeMap<String, String>,
    ignored: &[String],
) -> BTreeMap<String, String> {
    headers
        .iter()
        .filter(|(key, _)| !ignored.iter().any(|name| name.eq_ignore_ascii_case(key)))
        .map(|(k, v)| (k.to_ascii_lowercase(), v.clone()))
        .collect()
}
fn json_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                "integer"
            } else {
                "number"
            }
        }
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
fn type_matches(value: &Value, expected: &str) -> bool {
    let actual = json_type(value);
    actual == expected || (expected == "number" && actual == "integer")
}
fn assertion_message(
    a: &Assertion,
    actual: Option<&Value>,
    missing: bool,
    wrong_type: bool,
    wrong_value: bool,
    wrong_enum: bool,
    wrong_length: bool,
) -> String {
    if missing {
        return format!("Required field {} is missing", a.pointer);
    }
    if wrong_type {
        return format!(
            "Expected {} at {}, received {}",
            a.value_type.as_deref().unwrap_or("declared type"),
            a.pointer,
            actual.map(json_type).unwrap_or("missing")
        );
    }
    if wrong_value {
        return format!(
            "Value at {} did not match the declared exact value",
            a.pointer
        );
    }
    if wrong_enum {
        return format!(
            "Value at {} is outside the declared allowed values",
            a.pointer
        );
    }
    if wrong_length {
        return format!(
            "Array at {} is outside the declared length bounds",
            a.pointer
        );
    }
    "Response did not satisfy the declared consumer expectation".into()
}

pub async fn capture(
    base_url: &str,
    scenarios: &[Scenario],
    allow_writes: bool,
) -> Result<Vec<Recording>> {
    let options = HttpOptions {
        allow_state_changing: allow_writes,
        ..HttpOptions::default()
    };
    capture_with_options(base_url, scenarios, &options).await
}
pub async fn capture_with_options(
    base_url: &str,
    scenarios: &[Scenario],
    options: &HttpOptions,
) -> Result<Vec<Recording>> {
    let expanded = expand_scenarios(scenarios);
    let (records, results) = execute(base_url, &expanded, options, None, None).await?;
    if results.iter().any(|r| !r.passed) {
        anyhow::bail!("baseline failed one or more declared expectations; inspect the API and scenarios before recording");
    }
    Ok(records)
}
pub async fn capture_with_contract(
    base_url: &str,
    scenarios: &[Scenario],
    options: &HttpOptions,
    contract: &Value,
) -> Result<Vec<Recording>> {
    let recordings = capture_with_options(base_url, scenarios, options).await?;
    for record in &recordings {
        let scenario = Scenario {
            name: record.name.clone(),
            consumer: record.consumer.clone(),
            description: record.description.clone(),
            method: record.method.clone(),
            path: record.path.clone(),
            headers: record.headers.clone(),
            query: record.query.clone(),
            body: record.body.clone(),
            expect_status: Some(record.status),
            allowed_statuses: record.allowed_statuses.clone(),
            expect_content_type: record.expect_content_type.clone(),
            assertions: record.assertions.clone(),
            scenario_id: record.scenario_id.clone(),
            step_id: record.step_id.clone(),
            captures: record.captures.clone(),
            request_body_redacted: record.request_body_redacted,
            steps: Vec::new(),
        };
        let violations = response_contract_violations(
            contract,
            &scenario,
            &record.path,
            record.status,
            &record.response,
            &record.response_headers,
        );
        if !violations.is_empty() {
            anyhow::bail!("baseline response for scenario '{}' did not satisfy the declared OpenAPI response contract at {}",record.name,violations[0].1);
        }
    }
    Ok(recordings)
}
pub async fn replay(
    base_url: &str,
    recordings: &[Recording],
    allow_writes: bool,
) -> Result<Vec<ScenarioResult>> {
    let options = HttpOptions {
        allow_state_changing: allow_writes,
        ..HttpOptions::default()
    };
    replay_with_options(base_url, recordings, &options).await
}
pub async fn replay_with_options(
    base_url: &str,
    recordings: &[Recording],
    options: &HttpOptions,
) -> Result<Vec<ScenarioResult>> {
    replay_with_contracts(base_url, recordings, options, None, None).await
}
pub async fn replay_with_contracts(
    base_url: &str,
    recordings: &[Recording],
    options: &HttpOptions,
    old_contract: Option<&Value>,
    new_contract: Option<&Value>,
) -> Result<Vec<ScenarioResult>> {
    let scenarios: Vec<_> = recordings
        .iter()
        .map(|r| Scenario {
            name: r.name.clone(),
            consumer: r.consumer.clone(),
            description: r.description.clone(),
            method: r.method.clone(),
            path: r.path.clone(),
            headers: r.headers.clone(),
            query: r.query.clone(),
            body: r.body.clone(),
            expect_status: Some(r.status),
            allowed_statuses: r.allowed_statuses.clone(),
            expect_content_type: r.expect_content_type.clone(),
            assertions: r.assertions.clone(),
            scenario_id: r.scenario_id.clone(),
            step_id: r.step_id.clone(),
            captures: r.captures.clone(),
            request_body_redacted: r.request_body_redacted,
            steps: Vec::new(),
        })
        .collect();
    execute(
        base_url,
        &scenarios,
        options,
        Some(recordings),
        Some((old_contract, new_contract)),
    )
    .await
    .map(|v| v.1)
}

fn expand_scenarios(scenarios: &[Scenario]) -> Vec<Scenario> {
    let mut expanded = Vec::new();
    for scenario in scenarios {
        if scenario.steps.is_empty() {
            expanded.push(scenario.clone());
            continue;
        }
        for step in &scenario.steps {
            expanded.push(Scenario {
                name: scenario.name.clone(),
                consumer: scenario.consumer.clone(),
                description: scenario.description.clone(),
                method: step.request.method.clone(),
                path: step.request.path.clone(),
                headers: step.request.headers.clone(),
                query: step.request.query.clone(),
                body: step.request.body.clone(),
                expect_status: step.expect.status,
                allowed_statuses: step.expect.allowed_statuses.clone(),
                expect_content_type: step.expect.content_type.clone(),
                assertions: step.expect.json.clone(),
                scenario_id: Some(scenario.name.clone()),
                step_id: Some(step.id.clone()),
                captures: step.capture.clone(),
                request_body_redacted: false,
                steps: Vec::new(),
            });
        }
    }
    expanded
}
async fn execute(
    base_url: &str,
    scenarios: &[Scenario],
    options: &HttpOptions,
    baseline: Option<&[Recording]>,
    contracts: Option<(Option<&Value>, Option<&Value>)>,
) -> Result<(Vec<Recording>, Vec<ScenarioResult>)> {
    validate_target(base_url, options.allow_remote)?;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(options.timeout_seconds))
        .build()?;
    let mut recordings = Vec::new();
    let mut results = Vec::new();
    let mut variables = BTreeMap::<String, Value>::new();
    let mut active_scenario = String::new();
    for s in scenarios {
        let scenario_key = s.scenario_id.as_deref().unwrap_or(&s.name).to_owned();
        if active_scenario != scenario_key {
            variables.clear();
            active_scenario = scenario_key.clone();
        }
        if s.request_body_redacted {
            let issue=finding("redacted-request-body","warning","inconclusive",&s.name,"Request body contains redacted baseline data and cannot be safely replayed; use environment placeholders for replayable secrets");
            results.push(ScenarioResult {
                name: s.name.clone(),
                consumer: s.consumer.clone(),
                step_id: s.step_id.clone(),
                passed: false,
                executed: false,
                status: 0,
                findings: vec![issue],
            });
            continue;
        }
        let method = reqwest::Method::from_bytes(s.method.to_uppercase().as_bytes())?;
        if !matches!(
            method,
            reqwest::Method::GET | reqwest::Method::HEAD | reqwest::Method::OPTIONS
        ) && !options.allow_state_changing
        {
            anyhow::bail!("scenario '{}' uses state-changing method {}; pass --allow-state-changing only for a controlled target",s.name,method);
        }
        let request_parts = (|| -> Result<RequestParts> {
            let path = interpolate_with_vars(&s.path, &variables)?;
            let mut query = BTreeMap::new();
            for (k, v) in &s.query {
                query.insert(k.clone(), interpolate_with_vars(v, &variables)?);
            }
            let mut headers = BTreeMap::new();
            for (k, v) in &s.headers {
                headers.insert(k.clone(), interpolate_with_vars(v, &variables)?);
            }
            let body = s
                .body
                .as_ref()
                .map(|v| interpolate_value(v, &variables))
                .transpose()?;
            Ok((path, query, headers, body))
        })();
        let (path, query, headers, body) = match request_parts {
            Ok(parts) => parts,
            Err(_) => {
                let finding = finding(if s.step_id.is_some(){"step-dependency-blocked"}else{"request-interpolation-failed"}, "warning", "inconclusive", &s.name, "Request could not be prepared because a required variable is missing or invalid; sensitive details are omitted");
                results.push(ScenarioResult {
                    name: s.name.clone(),
                    consumer: s.consumer.clone(),
                    step_id: s.step_id.clone(),
                    passed: false,
                    executed: false,
                    status: 0,
                    findings: vec![finding],
                });
                continue;
            }
        };
        let mut req = client.request(
            method.clone(),
            format!("{}{}", base_url.trim_end_matches('/'), path),
        );
        req = req.query(&query);
        for (k, v) in &headers {
            req = req.header(k, v);
        }
        if let Some(body) = &body {
            let body = serde_json::to_vec(body)?;
            if body.len() > options.max_request_bytes {
                anyhow::bail!(
                    "request body for scenario '{}' exceeds {} byte limit",
                    s.name,
                    options.max_request_bytes
                );
            }
            req = req
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body);
        }
        let started = std::time::Instant::now();
        let response = match req.send().await {
            Ok(response) => response,
            Err(_) => {
                let finding=finding("http-request-failed","warning","inconclusive",&s.name,"HTTP request failed (timeout, connection, or TLS); request details were omitted");
                results.push(ScenarioResult {
                    name: s.name.clone(),
                    consumer: s.consumer.clone(),
                    step_id: s.step_id.clone(),
                    passed: false,
                    executed: true,
                    status: 0,
                    findings: vec![finding],
                });
                continue;
            }
        };
        let status = response.status().as_u16();
        let response_headers = response
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_owned(),
                    v.to_str().unwrap_or("[non-text]").to_owned(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        if response
            .content_length()
            .is_some_and(|n| n > options.max_response_bytes as u64)
        {
            anyhow::bail!(
                "response for scenario '{}' exceeds {} byte limit",
                s.name,
                options.max_response_bytes
            );
        }
        let mut response = response;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if bytes.len().saturating_add(chunk.len()) > options.max_response_bytes {
                anyhow::bail!(
                    "response for scenario '{}' exceeds {} byte limit",
                    s.name,
                    options.max_response_bytes
                );
            }
            bytes.extend_from_slice(&chunk);
        }
        let elapsed_ms = started.elapsed().as_millis();
        let value = serde_json::from_slice::<Value>(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
        let mut findings = Vec::new();
        if let Some((old_contract, new_contract)) = contracts {
            if let Some(old_contract) = old_contract {
                for (classification, violation) in response_contract_violations(
                    old_contract,
                    s,
                    &path,
                    status,
                    &value,
                    &response_headers,
                ) {
                    findings.push(finding(
                        "response-violates-old-contract",
                        "error",
                        &classification,
                        &format!("{} {violation}", s.name),
                        "Candidate response violates the previously published response contract",
                    ));
                }
            }
            if let Some(new_contract) = new_contract {
                for (classification, violation) in response_contract_violations(
                    new_contract,
                    s,
                    &path,
                    status,
                    &value,
                    &response_headers,
                ) {
                    findings.push(finding(
                        "candidate-contract-mismatch",
                        "warning",
                        &classification,
                        &format!("{} {violation}", s.name),
                        "Candidate response does not match its published OpenAPI response contract",
                    ));
                }
            }
        }
        if let Some(expected) = s.expect_status {
            if status != expected && !s.allowed_statuses.contains(&status) {
                findings.push(finding(
                    "status-mismatch",
                    "error",
                    "confirmed_break",
                    &s.name,
                    &format!("Expected HTTP {expected}, received {status}"),
                ));
            }
        }
        if !s.allowed_statuses.is_empty()
            && !s.allowed_statuses.contains(&status)
            && s.expect_status.is_none()
        {
            findings.push(finding(
                "status-not-allowed",
                "error",
                "confirmed_break",
                &s.name,
                "Response status is outside the explicitly allowed set",
            ));
        }
        if let Some(expected) = &s.expect_content_type {
            let actual = response_headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
                .map(|(_, v)| v.as_str())
                .unwrap_or("");
            if !actual
                .to_ascii_lowercase()
                .starts_with(&expected.to_ascii_lowercase())
            {
                findings.push(finding(
                    "content-type-mismatch",
                    "error",
                    "confirmed_break",
                    &s.name,
                    &format!(
                        "Expected content type {expected}, received {}",
                        actual.split(';').next().unwrap_or("missing")
                    ),
                ));
            }
        }
        for a in &s.assertions {
            if !a.pointer.is_empty() && !a.pointer.starts_with('/') {
                findings.push(finding(
                    "invalid-json-pointer",
                    "error",
                    "inconclusive",
                    &s.name,
                    "Assertion JSON Pointer must be empty or begin with '/'",
                ));
                continue;
            }
            if let Some(header) = &a.header {
                let actual = response_headers
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(header))
                    .map(|(_, v)| v);
                let failed = a.required && actual.is_none()
                    || a.header_equals
                        .as_ref()
                        .is_some_and(|expected| actual != Some(expected));
                if failed {
                    findings.push(finding(
                        "header-assertion-failed",
                        "error",
                        "confirmed_break",
                        &format!("{} header {}", s.name, header),
                        "Response header did not satisfy the declared expectation",
                    ));
                }
                continue;
            }
            let actual = value.pointer(&a.pointer);
            let wrong_type = a
                .value_type
                .as_ref()
                .is_some_and(|expected| actual.is_none_or(|v| !type_matches(v, expected)));
            let wrong_value = a
                .equals
                .as_ref()
                .is_some_and(|expected| actual != Some(expected));
            let wrong_enum =
                !a.enum_values.is_empty() && actual.is_some_and(|v| !a.enum_values.contains(v));
            let wrong_length = actual.and_then(Value::as_array).is_some_and(|array| {
                a.min_items.is_some_and(|n| array.len() < n)
                    || a.max_items.is_some_and(|n| array.len() > n)
            });
            let missing = a.required && actual.is_none();
            if wrong_type || wrong_value || wrong_enum || wrong_length || missing {
                findings.push(finding(
                    "assertion-failed",
                    "error",
                    "confirmed_break",
                    &format!("{} {}", s.name, a.pointer),
                    &assertion_message(
                        a,
                        actual,
                        missing,
                        wrong_type,
                        wrong_value,
                        wrong_enum,
                        wrong_length,
                    ),
                ));
            }
        }
        for (name, pointer) in &s.captures {
            if findings
                .iter()
                .any(|f| f.classification == "confirmed_break")
            {
                break;
            }
            if let Some(captured) = value.pointer(pointer) {
                variables.insert(
                    format!("{}.{}", s.step_id.as_deref().unwrap_or("step"), name),
                    captured.clone(),
                );
            } else {
                findings.push(finding(
                    "variable-extraction-failed",
                    "error",
                    "confirmed_break",
                    &format!("{} {pointer}", s.name),
                    "A required scenario variable could not be extracted from the response",
                ));
            }
        }
        if let Some(original) = baseline.and_then(|all| {
            all.iter()
                .find(|r| r.name == s.name && r.step_id == s.step_id)
        }) {
            if normalize_response(
                original.response.clone(),
                &options.ignore_json_pointers,
                &options.ignore_array_order,
            ) != normalize_response(
                redact_json(value.clone(), &options.redacted_json_pointers),
                &options.ignore_json_pointers,
                &options.ignore_array_order,
            ) {
                findings.push(finding("response-drift", "info", "non_breaking", &s.name, "Response content changed; review contract and consumer assertions for significance"));
            }
            if normalize_headers(&original.response_headers, &options.ignore_headers)
                != normalize_headers(&response_headers, &options.ignore_headers)
            {
                findings.push(finding("response-header-drift", "info", "non_breaking", &s.name, "Response headers changed; review explicit header assertions for compatibility impact"));
            }
        }
        let passed = findings.iter().all(|f| f.classification == "non_breaking");
        if baseline.is_none() {
            recordings.push(Recording {
                name: s.name.clone(),
                consumer: s.consumer.clone(),
                description: s.description.clone(),
                scenario_id: s.scenario_id.clone(),
                step_id: s.step_id.clone(),
                captures: s.captures.clone(),
                method: s.method.to_uppercase(),
                path: s.path.clone(),
                headers: redact_headers(&s.headers, &options.redacted_headers),
                query: s.query.clone(),
                body: s
                    .body
                    .as_ref()
                    .map(|v| redact_json(v.clone(), &options.redacted_json_pointers)),
                request_body_redacted: s
                    .body
                    .as_ref()
                    .is_some_and(|v| has_redacted_pointer(v, &options.redacted_json_pointers)),
                status,
                allowed_statuses: s.allowed_statuses.clone(),
                expect_content_type: s.expect_content_type.clone(),
                response: redact_json(value, &options.redacted_json_pointers),
                response_headers: redact_headers(&response_headers, &options.redacted_headers),
                elapsed_ms,
                assertions: s.assertions.clone(),
            });
        }
        results.push(ScenarioResult {
            name: s.name.clone(),
            consumer: s.consumer.clone(),
            step_id: s.step_id.clone(),
            passed,
            executed: true,
            status,
            findings,
        });
    }
    Ok((recordings, results))
}

fn response_contract_violations(
    contract: &Value,
    scenario: &Scenario,
    actual_path: &str,
    status: u16,
    body: &Value,
    headers: &BTreeMap<String, String>,
) -> Vec<(String, String)> {
    let method = scenario.method.to_ascii_lowercase();
    let Some(paths) = contract.get("paths").and_then(Value::as_object) else {
        return vec![(
            "inconclusive".into(),
            "paths: contract has no paths object".into(),
        )];
    };
    let operation = paths
        .iter()
        .find(|(template, _)| path_matches(template, actual_path))
        .and_then(|(_, item)| item.get(&method));
    let Some(operation) = operation else {
        return vec![];
    };
    let Some(responses) = operation.get("responses").and_then(Value::as_object) else {
        return vec![];
    };
    let response = responses
        .get(&status.to_string())
        .or_else(|| responses.get("default"))
        .or_else(|| {
            responses
                .iter()
                .find(|(key, _)| {
                    key.len() == 3
                        && key.as_bytes()[0].is_ascii_digit()
                        && key[1..].eq_ignore_ascii_case("XX")
                        && key[..1].parse::<u16>().ok() == Some(status / 100)
                })
                .map(|(_, v)| v)
        });
    let Some(response) = response else {
        return vec![];
    };
    let Some(content) = response.get("content").and_then(Value::as_object) else {
        return vec![];
    };
    let content_type = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        .map(|(_, v)| {
            v.split(';')
                .next()
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase()
        })
        .unwrap_or_default();
    let media = content.get(&content_type).or_else(|| {
        content
            .iter()
            .find(|(key, _)| {
                (key.ends_with("/*") && content_type.starts_with(key.trim_end_matches('*')))
                    || key.as_str() == "*/*"
            })
            .map(|(_, v)| v)
    });
    let Some(media) = media else {
        return if content.is_empty() {
            vec![]
        } else {
            vec![(
                "confirmed_break".into(),
                format!("/content-type: unexpected response media type {content_type}"),
            )]
        };
    };
    let Some(schema) = media.get("schema") else {
        return vec![];
    };
    let mut violations = Vec::new();
    validate_json_schema(schema, body, "", 0, &mut violations);
    violations
        .into_iter()
        .map(|(path, reason)| {
            let classification = if reason.starts_with("unsupported:") {
                "inconclusive"
            } else {
                "confirmed_break"
            };
            (classification.into(), format!("{path}: {reason}"))
        })
        .collect()
}
fn path_matches(template: &str, actual: &str) -> bool {
    let template = template.trim_matches('/').split('/').collect::<Vec<_>>();
    let actual = actual
        .split('?')
        .next()
        .unwrap_or(actual)
        .trim_matches('/')
        .split('/')
        .collect::<Vec<_>>();
    template.len() == actual.len()
        && template
            .iter()
            .zip(actual)
            .all(|(a, b)| a.starts_with('{') && a.ends_with('}') || a == &b)
}
fn validate_json_schema(
    schema: &Value,
    value: &Value,
    path: &str,
    depth: usize,
    out: &mut Vec<(String, String)>,
) {
    if depth > 64 {
        out.push((path.into(), "unsupported: validation depth exceeded".into()));
        return;
    }
    if schema.get("$ref").is_some()
        || schema.get("allOf").is_some()
        || schema.get("oneOf").is_some()
        || schema.get("anyOf").is_some()
        || schema.get("not").is_some()
    {
        out.push((
            path.into(),
            "unsupported: schema reference or composition".into(),
        ));
        return;
    }
    let types = schema_types(schema);
    if let Some(types) = types {
        let actual = json_type(value);
        if !types
            .iter()
            .any(|t| t == actual || t == "number" && actual == "integer")
        {
            out.push((
                path.into(),
                format!("expected type {:?}, received {actual}", types),
            ));
            return;
        }
    }
    if allows_null(schema) && value.is_null() {
        return;
    }
    if let Some(enumeration) = schema.get("enum").and_then(Value::as_array) {
        if !enumeration.contains(value) {
            out.push((path.into(), "value is outside the declared enum".into()));
        }
    }
    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for name in required.iter().filter_map(Value::as_str) {
            if value.get(name).is_none() {
                out.push((
                    format!("{path}/{name}"),
                    "required response property is missing".into(),
                ));
            }
        }
    }
    if let (Some(properties), Some(object)) = (
        schema.get("properties").and_then(Value::as_object),
        value.as_object(),
    ) {
        for (name, child_schema) in properties {
            if let Some(child) = object.get(name) {
                validate_json_schema(
                    child_schema,
                    child,
                    &format!("{path}/{name}"),
                    depth + 1,
                    out,
                );
            }
        }
    }
    if let (Some(items_schema), Some(items)) = (schema.get("items"), value.as_array()) {
        for (index, item) in items.iter().enumerate() {
            validate_json_schema(
                items_schema,
                item,
                &format!("{path}/{index}"),
                depth + 1,
                out,
            );
        }
    }
    if schema.get("additionalProperties").and_then(Value::as_bool) == Some(false) {
        if let (Some(properties), Some(object)) = (
            schema.get("properties").and_then(Value::as_object),
            value.as_object(),
        ) {
            for key in object.keys() {
                if !properties.contains_key(key) {
                    out.push((
                        format!("{path}/{key}"),
                        "additional response property is not allowed by the contract".into(),
                    ));
                }
            }
        }
    }
    for (constraint, actual) in [
        (
            "minLength",
            value.as_str().map(|v| v.chars().count() as f64),
        ),
        ("minItems", value.as_array().map(|v| v.len() as f64)),
        ("minimum", value.as_f64()),
    ] {
        if let (Some(expected), Some(actual)) =
            (schema.get(constraint).and_then(Value::as_f64), actual)
        {
            if actual < expected {
                out.push((
                    path.into(),
                    format!("value is below {constraint}={expected}"),
                ));
            }
        }
    }
    for (constraint, actual) in [
        (
            "maxLength",
            value.as_str().map(|v| v.chars().count() as f64),
        ),
        ("maxItems", value.as_array().map(|v| v.len() as f64)),
        ("maximum", value.as_f64()),
    ] {
        if let (Some(expected), Some(actual)) =
            (schema.get(constraint).and_then(Value::as_f64), actual)
        {
            if actual > expected {
                out.push((
                    path.into(),
                    format!("value exceeds {constraint}={expected}"),
                ));
            }
        }
    }
}

pub fn build_report(findings: Vec<Finding>, scenarios: Vec<ScenarioResult>) -> Report {
    build_report_with_policy(
        findings,
        scenarios,
        &["confirmed_break".into(), "static_break".into()],
        true,
    )
}
pub fn build_report_with_policy(
    findings: Vec<Finding>,
    scenarios: Vec<ScenarioResult>,
    fail_on: &[String],
    fail_on_inconclusive: bool,
) -> Report {
    let blocked = findings.iter().any(|f| {
        fail_on.iter().any(|evidence| evidence == &f.classification)
            || fail_on_inconclusive && f.classification == "inconclusive"
    }) || scenarios.iter().any(|s| {
        s.findings
            .iter()
            .any(|f| fail_on.iter().any(|evidence| evidence == &f.classification))
    });
    let passed = !blocked;
    let mut consumers = BTreeMap::<String, ConsumerImpact>::new();
    for scenario in &scenarios {
        let Some(name) = &scenario.consumer else {
            continue;
        };
        let entry = consumers
            .entry(name.clone())
            .or_insert_with(|| ConsumerImpact {
                consumer: name.clone(),
                ..ConsumerImpact::default()
            });
        entry.total += 1;
        if scenario.passed {
            entry.passed += 1;
        } else {
            entry.failed += 1;
        }
    }
    let summary = ReportSummary {
        decision: if passed { "passed" } else { "blocked" }.into(),
        scenarios_total: scenarios.len(),
        scenarios_passed: scenarios.iter().filter(|s| s.passed).count(),
        scenarios_failed: scenarios.iter().filter(|s| !s.passed).count(),
        scenarios_skipped: scenarios.iter().filter(|s| !s.executed).count(),
        confirmed_breaks: findings
            .iter()
            .filter(|f| f.classification == "confirmed_break")
            .count(),
        static_breaks: findings
            .iter()
            .filter(|f| f.classification == "static_break")
            .count(),
        potential_breaks: findings
            .iter()
            .filter(|f| f.classification == "potential_break")
            .count(),
        inconclusive: findings
            .iter()
            .filter(|f| f.classification == "inconclusive")
            .count(),
    };
    let inconclusive_checks = findings
        .iter()
        .filter(|f| f.classification == "inconclusive")
        .count();
    Report {
        schema_version: 1,
        findings,
        scenarios,
        passed,
        consumers: consumers.into_values().collect(),
        summary,
        tool_version: env!("CARGO_PKG_VERSION").into(),
        project: None,
        contracts: BTreeMap::new(),
        coverage: ReportCoverage {
            operations_total: 0,
            operations_with_scenarios: 0,
            operations_without_scenarios: 0,
            inconclusive_checks,
        },
        policy: GatePolicy {
            fail_on: fail_on.to_vec(),
            fail_on_inconclusive,
        },
    }
}

pub fn operation_keys(contract: &Value) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(paths) = contract.get("paths").and_then(Value::as_object) {
        for (path, item) in paths {
            if let Some(methods) = item.as_object() {
                for (method, _) in methods {
                    if is_method(method) {
                        keys.push(format!("{} {}", method.to_ascii_uppercase(), path));
                    }
                }
            }
        }
    }
    keys.sort();
    keys
}
pub fn contract_fingerprint(contract: &Value) -> String {
    // Stable FNV-1a is a reproducibility checksum, not a cryptographic integrity guarantee.
    let bytes = serde_json::to_vec(contract).unwrap_or_default();
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}
pub fn covered_operation_keys(contract: &Value, recordings: &[Recording]) -> Vec<String> {
    let keys = operation_keys(contract);
    let mut covered = std::collections::BTreeSet::new();
    for record in recordings {
        for key in &keys {
            let Some((method, path)) = key.split_once(' ') else {
                continue;
            };
            if method.eq_ignore_ascii_case(&record.method) && path_matches(path, &record.path) {
                covered.insert(key.clone());
            }
        }
    }
    covered.into_iter().collect()
}
pub fn html_report(report: &Report) -> String {
    let finding_rows = report
        .findings
        .iter()
        .map(|f| {
            format!(
                "<tr><td>{}</td><td>{}</td><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
                html_escape(&f.severity),
                html_escape(&f.classification),
                html_escape(&f.location),
                html_escape(&f.message),
                html_escape(&f.id)
            )
        })
        .collect::<String>();
    let scenario_rows = report
        .scenarios
        .iter()
        .map(|s| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                html_escape(s.consumer.as_deref().unwrap_or("unassigned")),
                html_escape(&s.name),
                html_escape(s.step_id.as_deref().unwrap_or("")),
                s.status,
                if s.passed { "Passed" } else { "Failed" }
            )
        })
        .collect::<String>();
    let consumer_rows = report
        .consumers
        .iter()
        .map(|c| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                html_escape(&c.consumer),
                c.total,
                c.passed,
                c.failed
            )
        })
        .collect::<String>();
    let status = if report.passed { "passed" } else { "blocked" };
    format!("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; img-src data:\"><title>API compatibility report</title><style>body{{font:16px system-ui,sans-serif;max-width:1100px;margin:2.5rem auto;padding:0 1.2rem;color:#172033}}h1{{margin-bottom:.2rem}}.sub{{color:#526078}}.decision{{padding:.8rem 1rem;border:1px solid #b8c3d2;border-left:5px solid {};border-radius:6px;font-weight:700}}.cards{{display:grid;grid-template-columns:repeat(auto-fit,minmax(145px,1fr));gap:.8rem;margin:1.4rem 0}}.card{{border:1px solid #d8deea;border-radius:8px;padding:1rem}}.card strong{{display:block;font-size:1.5rem}}table{{width:100%;border-collapse:collapse;margin:1rem 0 2rem}}th,td{{text-align:left;padding:.65rem;border-bottom:1px solid #e0e5ed;vertical-align:top}}th{{background:#f5f7fa}}code{{font-family:ui-monospace,monospace;overflow-wrap:anywhere}}.pass{{color:#087443}}.fail{{color:#b42318}}.muted{{color:#64748b}}@media(max-width:650px){{body{{margin:1rem auto}}table{{font-size:.9rem}}}}</style></head><body><header><h1>API compatibility report</h1><p class=\"sub\">{} · report schema {} · {}</p></header><p class=\"decision {}\">Release {}</p><section class=\"cards\"><div class=\"card\"><span>Scenarios</span><strong>{}</strong></div><div class=\"card\"><span>Passed</span><strong>{}</strong></div><div class=\"card\"><span>Failed</span><strong>{}</strong></div><div class=\"card\"><span>Static breaks</span><strong>{}</strong></div><div class=\"card\"><span>Confirmed breaks</span><strong>{}</strong></div><div class=\"card\"><span>Inconclusive</span><strong>{}</strong></div></section><h2>Findings</h2><table><thead><tr><th>Severity</th><th>Classification</th><th>Location</th><th>Finding</th><th>Rule</th></tr></thead><tbody>{}</tbody></table><h2>Scenario outcomes</h2><table><thead><tr><th>Consumer</th><th>Scenario</th><th>Step</th><th>Status</th><th>Outcome</th></tr></thead><tbody>{}</tbody></table><h2>Consumer impact</h2><table><thead><tr><th>Consumer</th><th>Total</th><th>Passed</th><th>Failed</th></tr></thead><tbody>{}</tbody></table><h2>Coverage</h2><p>{} of {} baseline operations have at least one recorded scenario. This measures operation representation, not coverage of all behavior.</p><p class=\"muted\">Checks cover only the supplied scenarios and implemented OpenAPI rules. A pass does not prove complete semantic compatibility.</p></body></html>",if report.passed{"#168452"}else{"#b42318"},html_escape(report.project.as_deref().unwrap_or("API compatibility")),report.schema_version,html_escape(&report.tool_version),status,if report.passed{"PASSED"}else{"BLOCKED"},report.summary.scenarios_total,report.summary.scenarios_passed,report.summary.scenarios_failed,report.summary.static_breaks,report.summary.confirmed_breaks,report.summary.inconclusive,finding_rows,scenario_rows,consumer_rows,report.coverage.operations_with_scenarios,report.coverage.operations_total)
}
fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_added_required_param() {
        let a = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"get":{"parameters":[]}}}});
        let b = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"get":{"parameters":[{"name":"region","in":"query","required":true}]}}}});
        assert!(diff(&a, &b)
            .iter()
            .any(|f| f.id == "required-parameter-added"));
    }
    #[test]
    fn added_endpoint_is_safe() {
        let a = serde_json::json!({"openapi":"3.1.0","paths":{}});
        let b = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"get":{}}}});
        assert!(diff(&a, &b).is_empty());
    }
    #[test]
    fn response_type_change_is_break() {
        let a = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"get":{"responses":{"200":{"content":{"application/json":{"schema":{"type":"string"}}}}}}}}});
        let b = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"get":{"responses":{"200":{"content":{"application/json":{"schema":{"type":"object"}}}}}}}}});
        assert!(diff(&a, &b).iter().any(|f| f.id == "schema-type-changed"));
    }

    #[test]
    fn request_enum_narrowing_is_breaking() {
        let a = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"post":{"requestBody":{"content":{"application/json":{"schema":{"type":"string","enum":["a","b"]}}}},"responses":{"204":{"description":"ok"}}}}}});
        let b = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"post":{"requestBody":{"content":{"application/json":{"schema":{"type":"string","enum":["a"]}}}},"responses":{"204":{"description":"ok"}}}}}});
        assert!(diff(&a, &b).iter().any(|f| f.id == "request-enum-narrowed"));
    }

    #[test]
    fn response_property_becoming_optional_is_breaking() {
        let a = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"get":{"responses":{"200":{"content":{"application/json":{"schema":{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}}}}}}}}});
        let b = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"get":{"responses":{"200":{"content":{"application/json":{"schema":{"type":"object","properties":{"id":{"type":"string"}}}}}}}}}}});
        assert!(diff(&a, &b)
            .iter()
            .any(|f| f.id == "response-property-now-optional"));
    }

    #[test]
    fn resolves_local_and_external_refs_and_stops_cycles() {
        let root = std::env::temp_dir().join(format!("api-compat-ref-test-{}", std::process::id()));
        let _ = fs::create_dir_all(&root);
        fs::write(root.join("part.yaml"), "components:\n  schemas:\n    User:\n      type: object\n      properties:\n        manager:\n          $ref: '#/components/schemas/User'\n").unwrap();
        fs::write(root.join("openapi.yaml"), "openapi: 3.1.0\npaths: {}\ncomponents:\n  schemas:\n    Customer:\n      $ref: 'part.yaml#/components/schemas/User'\n").unwrap();
        let resolved = read_document(root.join("openapi.yaml")).unwrap();
        assert_eq!(
            resolved
                .pointer("/components/schemas/Customer/type")
                .and_then(Value::as_str),
            Some("object")
        );
        assert!(resolved
            .pointer("/components/schemas/Customer/properties/manager/$ref")
            .is_some());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn equivalent_nullable_syntax_is_not_a_break() {
        let old = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"get":{"responses":{"200":{"content":{"application/json":{"schema":{"type":"string","nullable":true}}}}}}}}});
        let new = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"get":{"responses":{"200":{"content":{"application/json":{"schema":{"type":["string","null"]}}}}}}}}});
        assert!(diff(&old, &new)
            .iter()
            .all(|f| f.id != "schema-type-changed" && f.id != "schema-nullability-changed"));
    }

    #[test]
    fn response_nullability_expansion_is_breaking() {
        let old = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"get":{"responses":{"200":{"content":{"application/json":{"schema":{"type":"string"}}}}}}}}});
        let new = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"get":{"responses":{"200":{"content":{"application/json":{"schema":{"type":["string","null"]}}}}}}}}});
        assert!(diff(&old, &new)
            .iter()
            .any(|f| f.id == "schema-nullability-changed"));
    }

    #[test]
    fn request_constraint_addition_is_breaking_but_widening_is_safe() {
        let old = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"post":{"requestBody":{"content":{"application/json":{"schema":{"type":"string"}}}},"responses":{"204":{"description":"ok"}}}}}});
        let stricter = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"post":{"requestBody":{"content":{"application/json":{"schema":{"type":"string","minLength":4}}}},"responses":{"204":{"description":"ok"}}}}}});
        let old_with_bound = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"post":{"requestBody":{"content":{"application/json":{"schema":{"type":"string","maxLength":50}}}},"responses":{"204":{"description":"ok"}}}}}});
        let wider = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"post":{"requestBody":{"content":{"application/json":{"schema":{"type":"string","maxLength":100}}}},"responses":{"204":{"description":"ok"}}}}}});
        assert!(diff(&old, &stricter)
            .iter()
            .any(|f| f.id == "schema-constraint-tightened"));
        assert!(!diff(&old_with_bound, &wider)
            .iter()
            .any(|f| f.classification == "static_break"));
    }

    #[test]
    fn renamed_path_parameter_keeps_route_identity() {
        let old = serde_json::json!({"openapi":"3.1.0","paths":{"/customers/{id}":{"get":{"parameters":[{"name":"id","in":"path","required":true,"schema":{"type":"string"}}],"responses":{"200":{"description":"ok"}}}}}});
        let new = serde_json::json!({"openapi":"3.1.0","paths":{"/customers/{customerId}":{"get":{"parameters":[{"name":"customerId","in":"path","required":true,"schema":{"type":"string"}}],"responses":{"200":{"description":"ok"}}}}}});
        assert!(diff(&old, &new).is_empty());
    }

    #[test]
    fn new_authentication_requirement_is_a_break() {
        let old = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"get":{"responses":{"200":{"description":"ok"}}}}}});
        let new = serde_json::json!({"openapi":"3.1.0","paths":{"/x":{"get":{"security":[{"Bearer":[]}],"responses":{"200":{"description":"ok"}}}}}});
        assert!(diff(&old, &new)
            .iter()
            .any(|f| f.id == "security-tightened"));
    }

    #[test]
    fn redaction_preserves_templates_and_masks_literals() {
        let value = serde_json::json!({"token":"${API_TOKEN}","email":"person@example.test"});
        let redacted = redact_json(value, &["/token".into(), "/email".into()]);
        assert_eq!(redacted["token"], "${API_TOKEN}");
        assert_eq!(redacted["email"], "[REDACTED]");
        assert!(has_redacted_pointer(&redacted, &["/email".into()]));
    }

    #[test]
    fn step_variables_interpolate_without_reusing_other_scenarios() {
        let variables = BTreeMap::from([(
            "first-page.next_cursor".into(),
            Value::String("cursor-2".into()),
        )]);
        assert_eq!(
            interpolate_with_vars("${steps.first-page.next_cursor}", &variables).unwrap(),
            "cursor-2"
        );
        assert!(interpolate_with_vars("${steps.first-page.missing}", &variables).is_err());
    }

    #[test]
    fn html_report_escapes_untrusted_finding_content() {
        let report = build_report(
            vec![finding(
                "sample",
                "error",
                "confirmed_break",
                "<img>",
                "<script>alert(1)</script>",
            )],
            vec![],
        );
        let html = html_report(&report);
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[tokio::test]
    async fn captures_and_replays_a_real_http_request() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                eprintln!("skipping loopback replay check: sandbox denies local sockets");
                return;
            }
            Err(error) => panic!("could not bind local test server: {error}"),
        };
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0_u8; 2048];
                let _ = stream.read(&mut request).unwrap();
                let body = br#"{"customer_id":"cust_123"}"#;
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
                stream.write_all(body).unwrap();
            }
        });
        let scenario = Scenario {
            name: "customer".into(),
            consumer: Some("test-client".into()),
            description: None,
            method: "GET".into(),
            path: "/customers/cust_123".into(),
            headers: BTreeMap::new(),
            query: BTreeMap::new(),
            body: None,
            expect_status: Some(200),
            allowed_statuses: vec![],
            expect_content_type: None,
            scenario_id: None,
            step_id: None,
            captures: BTreeMap::new(),
            request_body_redacted: false,
            steps: vec![],
            assertions: vec![Assertion {
                pointer: "/customer_id".into(),
                equals: Some(Value::String("cust_123".into())),
                value_type: None,
                required: false,
                enum_values: Vec::new(),
                min_items: None,
                max_items: None,
                header: None,
                header_equals: None,
            }],
        };
        let base = format!("http://{address}");
        let saved = capture(&base, &[scenario], false).await.unwrap();
        assert_eq!(saved[0].response["customer_id"], "cust_123");
        let results = replay(&base, &saved, false).await.unwrap();
        assert!(results[0].passed);
        server.join().unwrap();
    }
}
