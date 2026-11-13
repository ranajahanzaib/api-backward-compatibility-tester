use anyhow::{Context, Result};
use api_compat_core::{
    contract_fingerprint, covered_operation_keys, diff, html_report, operation_keys, read_document,
    BaselineMetadata, Finding, Recording, RecordingBundle, Scenario,
};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(
    name = "api-compat",
    version,
    about = "Catch API changes that break existing clients"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[arg(long, global = true)]
    timeout: Option<u64>,
    #[arg(long, global = true)]
    strict: bool,
    #[arg(long, global = true)]
    quiet: bool,
    #[arg(short, long, global = true)]
    verbose: bool,
}
#[derive(Subcommand)]
enum Command {
    Init {
        #[arg(long)]
        force: bool,
    },
    Spec {
        #[command(subcommand)]
        command: SpecCommand,
    },
    Capture {
        #[arg(long)]
        base_url: Option<String>,
        #[arg(long)]
        cases: Option<PathBuf>,
        #[arg(long)]
        openapi: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        allow_state_changing: bool,
        #[arg(long, default_value_t = false)]
        allow_remote: bool,
    },
    Check {
        #[arg(long)]
        baseline: Option<PathBuf>,
        #[arg(long)]
        candidate_url: Option<String>,
        #[arg(long)]
        old_openapi: Option<PathBuf>,
        #[arg(long)]
        new_openapi: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        allow_state_changing: bool,
        #[arg(long)]
        fail_on: Option<String>,
        #[arg(long, default_value_t = false)]
        allow_remote: bool,
    },
    Report {
        #[arg(long)]
        input: PathBuf,
        #[arg(long, default_value = "html")]
        format: String,
        #[arg(long)]
        output: PathBuf,
    },
    Demo,
}
#[derive(Subcommand)]
enum SpecCommand {
    Diff {
        #[arg(long)]
        old: PathBuf,
        #[arg(long)]
        new: PathBuf,
        #[arg(long, default_value = "terminal")]
        format: String,
    },
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Config {
    version: Option<u32>,
    project: Option<ProjectConfig>,
    contracts: ContractsConfig,
    targets: TargetsConfig,
    scenarios: ScenariosConfig,
    replay: ReplayConfig,
    normalization: NormalizationConfig,
    redaction: RedactionConfig,
    policy: PolicyConfig,
    reports: ReportsConfig,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ProjectConfig {
    name: Option<String>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ContractsConfig {
    baseline: Option<PathBuf>,
    candidate: Option<PathBuf>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct TargetsConfig {
    baseline: TargetConfig,
    candidate: TargetConfig,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct TargetConfig {
    base_url: Option<String>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ScenariosConfig {
    directory: Option<PathBuf>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ReplayConfig {
    timeout_seconds: Option<u64>,
    allow_state_changing: Option<bool>,
    allow_remote: Option<bool>,
    max_response_bytes: Option<usize>,
    max_request_bytes: Option<usize>,
    max_concurrency: Option<usize>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct NormalizationConfig {
    ignore_json_pointers: Option<Vec<String>>,
    ignore_headers: Option<Vec<String>>,
    ignore_array_order: Option<Vec<String>>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RedactionConfig {
    headers: Option<Vec<String>>,
    json_pointers: Option<Vec<String>>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct PolicyConfig {
    fail_on: Option<Vec<String>>,
    fail_on_inconclusive: Option<bool>,
    require_successful_replay: Option<bool>,
    require_nonempty_scenarios: Option<bool>,
    require_valid_baseline: Option<bool>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ReportsConfig {
    directory: Option<PathBuf>,
    formats: Option<Vec<String>>,
}
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = cli.config.clone().or_else(|| {
        PathBuf::from("api-compat.yaml")
            .exists()
            .then(|| PathBuf::from("api-compat.yaml"))
    });
    let config = if matches!(&cli.command, Command::Init { .. }) {
        Config::default()
    } else {
        load_config(config_path.as_deref())?
    };
    if !matches!(&cli.command, Command::Init { .. })
        && config_path.is_some()
        && config.version != Some(1)
    {
        anyhow::bail!("unsupported configuration version; expected version: 1");
    }
    let config_dir = config_path
        .as_deref()
        .and_then(Path::parent)
        .unwrap_or(Path::new("."));
    let timeout = cli
        .timeout
        .or_else(|| {
            std::env::var("API_COMPAT_TIMEOUT_SECONDS")
                .ok()
                .and_then(|x| x.parse().ok())
        })
        .or(config.replay.timeout_seconds)
        .unwrap_or(10);
    let mut http = api_compat_core::HttpOptions {
        timeout_seconds: timeout,
        ..Default::default()
    };
    if timeout == 0 || timeout > 300 {
        anyhow::bail!("timeout must be between 1 and 300 seconds");
    }
    http.max_response_bytes = config
        .replay
        .max_response_bytes
        .unwrap_or(http.max_response_bytes);
    http.max_request_bytes = config
        .replay
        .max_request_bytes
        .unwrap_or(http.max_request_bytes);
    if let Some(headers) = config.redaction.headers.clone() {
        for header in headers {
            if !http
                .redacted_headers
                .iter()
                .any(|current| current.eq_ignore_ascii_case(&header))
            {
                http.redacted_headers.push(header);
            }
        }
    }
    http.redacted_json_pointers = config.redaction.json_pointers.clone().unwrap_or_default();
    http.ignore_json_pointers = config
        .normalization
        .ignore_json_pointers
        .clone()
        .unwrap_or_default();
    http.ignore_headers = config
        .normalization
        .ignore_headers
        .clone()
        .unwrap_or(http.ignore_headers);
    http.ignore_array_order = config
        .normalization
        .ignore_array_order
        .clone()
        .unwrap_or_default();
    if config.replay.max_concurrency.is_some_and(|n| n != 1) {
        anyhow::bail!(
            "this version executes scenarios sequentially; replay.max_concurrency must be 1"
        );
    }
    match cli.command {
        Command::Init { force } => init(force, cli.config.as_deref())?,
        Command::Spec {
            command: SpecCommand::Diff { old, new, format },
        } => {
            let findings = diff(&read_document(old)?, &read_document(new)?);
            if !matches!(format.as_str(), "terminal" | "json") {
                anyhow::bail!("spec diff format must be terminal or json");
            }
            if !cli.quiet {
                print_findings(&findings, &format)?;
            }
            if findings.iter().any(|f| {
                matches!(
                    f.classification.as_str(),
                    "confirmed_break" | "static_break" | "inconclusive"
                )
            }) {
                let code = if findings.iter().any(|f| f.classification == "inconclusive") {
                    3
                } else {
                    2
                };
                std::process::exit(code)
            }
        }
        Command::Capture {
            base_url,
            cases,
            openapi,
            out,
            allow_state_changing,
            allow_remote,
        } => {
            let base_url = base_url
                .or_else(|| std::env::var("API_COMPAT_BASE_URL").ok())
                .or(config.targets.baseline.base_url)
                .ok_or_else(|| {
                    anyhow::anyhow!("set --base-url or targets.baseline.base_url in config")
                })?;
            let cases = cases
                .or(config.scenarios.directory)
                .unwrap_or_else(|| PathBuf::from("cases"));
            let cases = resolve_path(config_dir, cases);
            let openapi = resolve_path(
                config_dir,
                openapi
                    .or(config.contracts.baseline.clone())
                    .ok_or_else(|| {
                        anyhow::anyhow!("set --openapi or contracts.baseline in config")
                    })?,
            );
            let out = resolve_path(
                config_dir,
                out.unwrap_or_else(|| PathBuf::from(".api-compat/baseline.json")),
            );
            let scenarios = load_scenarios(&cases)?;
            http.allow_state_changing =
                allow_state_changing || config.replay.allow_state_changing.unwrap_or(false);
            http.allow_remote = allow_remote || config.replay.allow_remote.unwrap_or(false);
            if http.allow_state_changing {
                eprintln!(
                    "WARNING: state-changing requests are enabled; use only a controlled target."
                );
            }
            if http.allow_remote {
                eprintln!("WARNING: remote HTTP target enabled; requests will be sent to the configured host.");
            }
            let contract = read_document(&openapi)?;
            let records = match api_compat_core::capture_with_contract(
                &base_url, &scenarios, &http, &contract,
            )
            .await
            {
                Ok(records) => records,
                Err(error) => {
                    eprintln!("Baseline capture could not be completed: {error:#}");
                    std::process::exit(3)
                }
            };
            let captured_at_unix = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let bundle = RecordingBundle {
                recording_version: 1,
                baseline: BaselineMetadata {
                    openapi_fingerprint: contract_fingerprint(&contract),
                    captured_at_unix,
                    scenario_count: records.len(),
                },
                scenarios: records,
            };
            write_json(&out, &bundle)?;
            if !cli.quiet {
                println!(
                    "Captured {} scenarios to {}",
                    bundle.scenarios.len(),
                    out.display()
                );
            }
        }
        Command::Check {
            baseline,
            candidate_url,
            old_openapi,
            new_openapi,
            out,
            allow_state_changing,
            fail_on,
            allow_remote,
        } => {
            let baseline = resolve_path(
                config_dir,
                baseline.unwrap_or_else(|| PathBuf::from(".api-compat/baseline.json")),
            );
            let candidate_url = candidate_url
                .or_else(|| std::env::var("API_COMPAT_CANDIDATE_URL").ok())
                .or(config.targets.candidate.base_url)
                .ok_or_else(|| {
                    anyhow::anyhow!("set --candidate-url or targets.candidate.base_url in config")
                })?;
            let old_openapi = resolve_path(
                config_dir,
                old_openapi
                    .or(config.contracts.baseline)
                    .unwrap_or_else(|| PathBuf::from("openapi-old.yaml")),
            );
            let new_openapi = resolve_path(
                config_dir,
                new_openapi
                    .or(config.contracts.candidate)
                    .unwrap_or_else(|| PathBuf::from("openapi-new.yaml")),
            );
            let out = resolve_path(
                config_dir,
                out.or(config.reports.directory)
                    .unwrap_or_else(|| PathBuf::from("reports")),
            );
            let old_contract = read_document(&old_openapi)?;
            let new_contract = read_document(&new_openapi)?;
            let mut findings = diff(&old_contract, &new_contract);
            let raw_recording: serde_json::Value = serde_json::from_slice(
                &fs::read(&baseline).with_context(|| format!("read {}", baseline.display()))?,
            )?;
            let mut records: Vec<Recording>;
            let require_valid_baseline = config.policy.require_valid_baseline.unwrap_or(true);
            let mut baseline_verified = true;
            if raw_recording.is_array() {
                records = serde_json::from_value(raw_recording)?;
                baseline_verified = false;
                let classification = if require_valid_baseline {
                    "inconclusive"
                } else {
                    "potential_break"
                };
                findings.push(Finding{id:"legacy-recording-unverified".into(),classification:classification.into(),severity:"medium".into(),source:"recording".into(),consumer:None,evidence:classification.into(),location:"recording".into(),message:"Legacy recording has no version or OpenAPI fingerprint; recapture the baseline to verify contract identity".into()});
                if require_valid_baseline {
                    records.clear();
                }
            } else {
                let bundle: RecordingBundle = serde_json::from_value(raw_recording)?;
                if bundle.recording_version != 1 {
                    anyhow::bail!(
                        "unsupported recording version {}; expected 1",
                        bundle.recording_version
                    );
                }
                records = bundle.scenarios;
                if bundle.baseline.openapi_fingerprint != contract_fingerprint(&old_contract) {
                    baseline_verified = false;
                    let classification = if require_valid_baseline {
                        "inconclusive"
                    } else {
                        "potential_break"
                    };
                    findings.push(Finding{id:"recording-contract-mismatch".into(),classification:classification.into(),severity:"high".into(),source:"recording".into(),consumer:None,evidence:classification.into(),location:"recording.baseline.openapi_fingerprint".into(),message:"Baseline recording was captured against a different OpenAPI contract; recapture before checking".into()});
                    if require_valid_baseline {
                        records.clear();
                    }
                }
            }
            if records.is_empty()
                && (cli.strict || config.policy.require_nonempty_scenarios.unwrap_or(true))
                && baseline_verified
            {
                anyhow::bail!("baseline recording is empty; capture at least one scenario");
            }
            http.allow_state_changing =
                allow_state_changing || config.replay.allow_state_changing.unwrap_or(false);
            http.allow_remote = allow_remote || config.replay.allow_remote.unwrap_or(false);
            if http.allow_state_changing {
                eprintln!(
                    "WARNING: state-changing requests are enabled; use only a controlled target."
                );
            }
            if http.allow_remote {
                eprintln!("WARNING: remote HTTP target enabled; requests will be sent to the configured host.");
            }
            let scenarios = if records.is_empty() {
                vec![]
            } else {
                api_compat_core::replay_with_contracts(
                    &candidate_url,
                    &records,
                    &http,
                    if baseline_verified {
                        Some(&old_contract)
                    } else {
                        None
                    },
                    Some(&new_contract),
                )
                .await?
            };
            for s in &scenarios {
                findings.extend(s.findings.clone());
            }
            let fail_on = fail_on
                .or_else(|| config.policy.fail_on.as_ref().map(|v| v.join(",")))
                .unwrap_or_else(|| "confirmed_break,static_break".into());
            let fail_on: Vec<String> = if fail_on == "none" {
                vec![]
            } else {
                fail_on
                    .split(',')
                    .map(str::trim)
                    .map(str::to_owned)
                    .collect()
            };
            let valid_evidence = [
                "confirmed_break",
                "static_break",
                "potential_break",
                "non_breaking",
                "inconclusive",
            ];
            if fail_on
                .iter()
                .any(|item| !valid_evidence.contains(&item.as_str()))
            {
                anyhow::bail!(
                    "--fail-on accepts only finding classifications: {} or none",
                    valid_evidence.join(", ")
                );
            }
            let fail_on_inconclusive = cli.strict
                || (config.policy.require_successful_replay.unwrap_or(true)
                    && config.policy.fail_on_inconclusive.unwrap_or(true));
            let mut report = api_compat_core::build_report_with_policy(
                findings,
                scenarios,
                &fail_on,
                fail_on_inconclusive,
            );
            report.project = config.project.as_ref().and_then(|p| p.name.clone());
            report
                .contracts
                .insert("baseline".into(), contract_fingerprint(&old_contract));
            report
                .contracts
                .insert("candidate".into(), contract_fingerprint(&new_contract));
            let all_operations = operation_keys(&old_contract);
            let covered = covered_operation_keys(&old_contract, &records);
            report.coverage.operations_total = all_operations.len();
            report.coverage.operations_with_scenarios = covered.len();
            report.coverage.operations_without_scenarios =
                all_operations.len().saturating_sub(covered.len());
            fs::create_dir_all(&out)?;
            let formats = config
                .reports
                .formats
                .unwrap_or_else(|| vec!["json".into(), "html".into()]);
            if formats
                .iter()
                .any(|f| !matches!(f.as_str(), "json" | "html"))
            {
                anyhow::bail!("reports.formats supports only json and html");
            }
            if formats.iter().any(|f| f == "json") {
                write_json(&out.join("report.json"), &report)?;
            }
            if formats.iter().any(|f| f == "html") {
                fs::write(out.join("index.html"), html_report(&report))?;
            }
            if !cli.quiet {
                print_report(&report);
            }
            if !report.passed {
                let exit = if report
                    .findings
                    .iter()
                    .any(|f| f.classification == "inconclusive")
                    && fail_on_inconclusive
                {
                    3
                } else {
                    2
                };
                std::process::exit(exit)
            }
        }
        Command::Report {
            input,
            format,
            output,
        } => {
            let raw = fs::read(&input)?;
            if format == "html" {
                let report: api_compat_core::Report = serde_json::from_slice(&raw)?;
                fs::write(output, html_report(&report))?;
            } else if format == "json" {
                fs::write(output, raw)?;
            } else {
                anyhow::bail!("format must be json or html");
            }
        }
        Command::Demo => {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(Path::parent)
                .ok_or_else(|| anyhow::anyhow!("could not locate repository root"))?;
            let status = std::process::Command::new("sh")
                .arg(root.join("scripts/demo.sh"))
                .current_dir(root)
                .status()
                .context("start demo script")?;
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
    }
    if cli.verbose {
        eprintln!("api-compat completed");
    }
    Ok(())
}

fn load_config(path: Option<&Path>) -> Result<Config> {
    let Some(path) = path else {
        return Ok(Config::default());
    };
    let value =
        read_document(path).with_context(|| format!("load configuration {}", path.display()))?;
    serde_json::from_value(value).with_context(|| {
        format!(
            "validate configuration {} (check unknown or misspelled keys)",
            path.display()
        )
    })
}
fn resolve_path(base: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

fn init(force: bool, config_path: Option<&Path>) -> Result<()> {
    let config = config_path.unwrap_or_else(|| Path::new("api-compat.yaml"));
    let case = config
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("cases/example.yaml");
    if !force && (config.exists() || case.exists()) {
        anyhow::bail!(
            "{} or {} already exists; use --force to overwrite",
            config.display(),
            case.display()
        );
    }
    fs::create_dir_all(case.parent().unwrap_or_else(|| Path::new(".")))?;
    fs::write(config,"# CLI flags override these project defaults.\nversion: 1\nproject:\n  name: example-api\ncontracts:\n  baseline: openapi-old.yaml\n  candidate: openapi-new.yaml\ntargets:\n  baseline:\n    base_url: http://127.0.0.1:3001\n  candidate:\n    base_url: http://127.0.0.1:3002\nscenarios:\n  directory: cases/\nreplay:\n  timeout_seconds: 10\n  max_concurrency: 1\n  max_request_bytes: 1048576\n  max_response_bytes: 2097152\n  allow_state_changing: false\n  allow_remote: false\nnormalization:\n  ignore_json_pointers: []\n  ignore_headers: [date, server]\nredaction:\n  headers: [authorization, cookie, set-cookie]\n  json_pointers: []\npolicy:\n  fail_on: [confirmed_break, static_break]\n  fail_on_inconclusive: true\n  require_successful_replay: true\n  require_nonempty_scenarios: true\n  require_valid_baseline: true\nreports:\n  directory: reports/\n  formats: [json, html]\n")?;
    fs::write(
        &case,
        "name: health check\nmethod: GET\npath: /health\nexpect_status: 200\n",
    )?;
    println!("Created {} and {}", config.display(), case.display());
    Ok(())
}
fn load_scenarios(path: &Path) -> Result<Vec<Scenario>> {
    let mut files = Vec::new();
    if path.is_dir() {
        for e in fs::read_dir(path)? {
            let p = e?.path();
            if p.extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| x == "yaml" || x == "yml" || x == "json")
            {
                files.push(p)
            }
        }
        files.sort();
    } else {
        files.push(path.to_path_buf())
    }
    let mut out = Vec::new();
    for p in files {
        let v = read_document(&p)?;
        if let Some(a) = v.as_array() {
            out.extend(serde_json::from_value::<Vec<Scenario>>(
                ValueCompat::array(a.clone()),
            )?)
        } else if let Some(a) = v.get("scenarios").and_then(serde_json::Value::as_array) {
            out.extend(serde_json::from_value::<Vec<Scenario>>(
                serde_json::Value::Array(a.clone()),
            )?)
        } else {
            out.push(serde_json::from_value(v)?);
        }
    }
    if out.is_empty() {
        anyhow::bail!("no scenario files found in {}", path.display())
    }
    Ok(out)
}
struct ValueCompat;
impl ValueCompat {
    fn array(a: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::Value::Array(a)
    }
}
fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?
        }
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn print_findings(findings: &[Finding], format: &str) -> Result<()> {
    if format == "json" {
        println!("{}", serde_json::to_string_pretty(findings)?);
    } else {
        if findings.is_empty() {
            println!("No breaking changes detected.")
        }
        for f in findings {
            println!(
                "[{}] {} — {} ({})",
                f.severity.to_uppercase(),
                f.location,
                f.message,
                f.evidence
            );
        }
    }
    Ok(())
}

fn print_report(report: &api_compat_core::Report) {
    println!("API BACKWARD COMPATIBILITY CHECK");
    println!(
        "Decision: {}",
        if report.passed { "PASSED" } else { "BLOCKED" }
    );
    if let Some(project) = &report.project {
        println!("Project: {project}");
    }
    println!(
        "Static breaks: {} | Confirmed breaks: {} | Potential: {} | Inconclusive: {}",
        report.summary.static_breaks,
        report.summary.confirmed_breaks,
        report.summary.potential_breaks,
        report.summary.inconclusive
    );
    println!(
        "Scenarios: {} total | {} passed | {} failed",
        report.summary.scenarios_total,
        report.summary.scenarios_passed,
        report.summary.scenarios_failed
    );
    if !report.findings.is_empty() {
        println!("\nFINDINGS");
        for finding in &report.findings {
            println!(
                "[{} / {}] {} — {}",
                finding.severity.to_uppercase(),
                finding.classification,
                finding.location,
                finding.message
            );
            if let Some(consumer) = &finding.consumer {
                println!("  Consumer: {consumer}");
            }
        }
    }
    if !report.scenarios.is_empty() {
        println!("\nSCENARIOS");
        for scenario in &report.scenarios {
            let step = scenario
                .step_id
                .as_ref()
                .map(|name| format!(" / {name}"))
                .unwrap_or_default();
            println!(
                "[{}] {} / {}{} — {}",
                if scenario.passed { "PASS" } else { "FAIL" },
                scenario.consumer.as_deref().unwrap_or("unassigned"),
                scenario.name,
                step,
                if scenario.executed {
                    format!("HTTP {}", scenario.status)
                } else {
                    "skipped".into()
                }
            );
        }
    }
    println!(
        "\nRELEASE {}",
        if report.passed { "PASSED" } else { "BLOCKED" }
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_demo_consumer_scenarios_including_pagination() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let scenarios = load_scenarios(&root.join("examples/ecommerce-api/cases")).unwrap();
        assert_eq!(scenarios.len(), 3);
        assert_eq!(
            scenarios
                .iter()
                .find(|s| s.name == "customer-pagination")
                .unwrap()
                .steps
                .len(),
            2
        );
    }

    #[test]
    fn rejects_unknown_project_configuration_keys() {
        let value = serde_json::json!({"version":1,"replay":{"timeout_second":8}});
        assert!(serde_json::from_value::<Config>(value).is_err());
    }
}
