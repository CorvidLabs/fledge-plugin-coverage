// fledge-plugin-coverage
//
// Run language-native test coverage tools and report the result. With
// `--threshold N`, exit non-zero when the parsed percentage falls below N.
//
// Communicates with fledge via the fledge-v1 plugin protocol over stdio:
// reads init from stdin, sends an `exec` request to run the right tool for
// the project's language, parses the percentage out of stdout, and emits
// either a friendly summary or a JSON envelope.

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::exit;

#[derive(Deserialize)]
struct InitMessage {
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    project: Option<ProjectInfo>,
    #[serde(default)]
    capabilities: Capabilities,
}

#[derive(Deserialize)]
struct ProjectInfo {
    #[serde(default)]
    root: Option<String>,
    #[serde(default)]
    language: Option<String>,
}

#[derive(Default, Deserialize)]
struct Capabilities {
    #[serde(default)]
    exec: bool,
}

#[derive(Deserialize)]
struct ExecResult {
    #[serde(default)]
    code: i32,
    #[serde(default)]
    stdout: String,
    #[serde(default)]
    stderr: String,
}

#[derive(Serialize)]
struct Envelope<'a> {
    schema_version: u32,
    action: &'a str,
    language: &'a str,
    command: &'a str,
    exit_code: i32,
    coverage_percent: Option<f64>,
    threshold: Option<f64>,
    gate_failed: bool,
    ok: bool,
}

struct Options {
    json: bool,
    threshold: Option<f64>,
    lang: Option<String>,
}

fn main() {
    let init = read_init();
    let opts = parse_args(&init.args);

    if !init.capabilities.exec {
        log_err("exec capability not granted; cannot run coverage tools");
        exit(126);
    }

    let project = init.project;
    let root = project.as_ref().and_then(|p| p.root.clone()).unwrap_or_else(|| ".".into());
    let project_lang = project.as_ref().and_then(|p| p.language.clone());

    let lang = opts.lang.clone()
        .or(project_lang)
        .or_else(|| detect_lang(&root));

    let lang = match lang.as_deref() {
        Some(l) if COMMANDS.iter().any(|(name, _, _)| *name == l) => l.to_string(),
        _ => {
            let msg = format!("Could not detect a supported language in {} (try --lang)", root);
            if opts.json {
                emit_json_error(&msg);
            } else {
                log_err(&msg);
            }
            exit(2);
        }
    };

    let (_, command, pattern) = COMMANDS.iter().find(|(n, _, _)| **n == *lang).unwrap();

    progress(&format!("Running coverage ({})", lang));
    let result = exec(command, 600);
    progress_done();

    let pct = extract_percent(&result.stdout, pattern)
        .or_else(|| extract_percent(&result.stderr, pattern));

    let gate_failed = matches!((opts.threshold, pct), (Some(t), Some(p)) if p < t);
    let ok = result.code == 0 && pct.is_some() && !gate_failed;

    if opts.json {
        let env = Envelope {
            schema_version: 1,
            action: "coverage",
            language: &lang,
            command,
            exit_code: result.code,
            coverage_percent: pct,
            threshold: opts.threshold,
            gate_failed,
            ok,
        };
        output(&serde_json::to_string(&env).unwrap());
        output("\n");
    } else {
        if let Some(p) = pct {
            let mut line = format!("Coverage: {:.1}%", p);
            if let Some(t) = opts.threshold {
                line.push_str(&format!(" (threshold: {:.1}%)", t));
            }
            output(&line);
            output("\n");
        } else {
            log_warn("Could not parse coverage from tool output");
        }
        if result.code != 0 {
            output(&result.stdout);
            output(&result.stderr);
        }
    }

    if !ok {
        exit(1);
    }
}

const COMMANDS: &[(&str, &str, &str)] = &[
    (
        "rust",
        "cargo llvm-cov --summary-only --workspace",
        // Match the lines column on the TOTAL row: ... Cover ... Cover NN.NN%
        r"TOTAL\s+\d+\s+\d+\s+[\d\.]+%\s+\d+\s+\d+\s+[\d\.]+%\s+\d+\s+\d+\s+([\d\.]+)%",
    ),
    ("python", "pytest --cov=. --cov-report=term", r"TOTAL\s+\d+\s+\d+\s+([\d\.]+)%"),
    ("bun", "bun test --coverage", r"All files\s*\|\s*([\d\.]+)"),
    ("node", "npx jest --coverage --coverageReporters=text", r"All files\s*\|\s*([\d\.]+)"),
    ("go", "go test -cover ./...", r"coverage:\s+([\d\.]+)%"),
];

fn detect_lang(root: &str) -> Option<String> {
    let r = Path::new(root);
    let markers: &[(&str, &[&str])] = &[
        ("rust", &["Cargo.toml"]),
        ("python", &["pyproject.toml", "setup.py", "setup.cfg"]),
        ("bun", &["bun.lockb", "bun.lock"]),
        ("node", &["package.json"]),
        ("go", &["go.mod"]),
    ];
    for (lang, files) in markers {
        if files.iter().any(|f| r.join(f).exists()) {
            return Some((*lang).to_string());
        }
    }
    None
}

fn extract_percent(text: &str, pattern: &str) -> Option<f64> {
    let re = Regex::new(pattern).ok()?;
    for cap in re.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            if let Ok(p) = m.as_str().parse::<f64>() {
                return Some(p);
            }
        }
    }
    None
}

fn parse_args(args: &[String]) -> Options {
    let mut opts = Options { json: false, threshold: None, lang: None };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => opts.json = true,
            "--threshold" => {
                if let Some(v) = args.get(i + 1).and_then(|s| s.parse().ok()) {
                    opts.threshold = Some(v);
                    i += 1;
                }
            }
            "--lang" => {
                if let Some(v) = args.get(i + 1) {
                    opts.lang = Some(v.clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    opts
}

// ---- protocol I/O -----------------------------------------------------------

fn send(value: &Value) {
    println!("{}", value);
    io::stdout().flush().ok();
}

fn recv() -> Value {
    let mut line = String::new();
    let stdin = io::stdin();
    stdin.lock().read_line(&mut line).ok();
    if line.trim().is_empty() {
        exit(0);
    }
    serde_json::from_str(&line).unwrap_or(Value::Null)
}

fn read_init() -> InitMessage {
    let v = recv();
    serde_json::from_value(v).unwrap_or_else(|e| {
        log_err(&format!("malformed init: {}", e));
        exit(1);
    })
}

fn exec(command: &str, timeout: u64) -> ExecResult {
    send(&json!({"type": "exec", "id": "1", "command": command, "timeout": timeout}));
    let v = recv();
    let value = v.get("value").cloned().unwrap_or(Value::Null);
    serde_json::from_value(value).unwrap_or(ExecResult {
        code: -1,
        stdout: String::new(),
        stderr: String::new(),
    })
}

fn output(text: &str) {
    send(&json!({"type": "output", "text": text}));
}

fn progress(message: &str) {
    send(&json!({"type": "progress", "message": message}));
}

fn progress_done() {
    send(&json!({"type": "progress", "done": true}));
}

fn log_err(message: &str) {
    send(&json!({"type": "log", "level": "error", "message": message}));
}

fn log_warn(message: &str) {
    send(&json!({"type": "log", "level": "warn", "message": message}));
}

fn emit_json_error(msg: &str) {
    output(&serde_json::to_string(&json!({
        "schema_version": 1,
        "action": "coverage",
        "ok": false,
        "error": msg,
    })).unwrap());
    output("\n");
}
