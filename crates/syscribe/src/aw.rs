//! `syscribe applies-when <element> --set "<expr>" | --clear` — author the
//! `appliesWhen:` gate on a model element (REQ-TRS-AW-001).
//!
//! On `--set` the expression is parsed and then applied to an **in-memory** copy of
//! the model, which is validated: if the edit would raise `E209` (malformed /
//! unresolved-operand) or `E228` (forbidden placement) on the target, the edit is
//! refused and nothing is written. On success the `appliesWhen:` key is written into
//! the element's frontmatter — byte-preserving for everything else — and the
//! feature-model bad-configuration analysis (`feature-check --deep`) is run over the
//! result, whose exit code becomes the command's.

use std::path::Path;

use syscribe_model::config::ValidateConfig;
use syscribe_model::element::RawElement;
use syscribe_model::frontmatter::split_frontmatter;
use syscribe_model::resolver::Resolver;
use syscribe_model::{validator, variability};

use crate::query::{self, GateOptions};

/// Render an `appliesWhen:` value (string or AND-list sequence) as a readable
/// expression string.
fn render_aw_value(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Sequence(seq) => seq
            .iter()
            .map(render_aw_value)
            .collect::<Vec<_>>()
            .join(" and "),
        other => serde_yaml::to_string(other)
            .unwrap_or_default()
            .trim()
            .to_string(),
    }
}

/// Read-only display of an element's own and effective `appliesWhen` (REQ-TRS-AW-002).
fn show_gate(elements: &[RawElement], target: &RawElement, json: bool) -> ! {
    let pkg = variability::package_conditions(elements);
    let own = target
        .frontmatter
        .applies_when
        .as_ref()
        .map(render_aw_value);
    let eff = variability::effective_applies_when(target, &pkg);
    let (eff_str, inherited): (Option<String>, Option<String>) = match &eff {
        Some((v, src)) => (Some(render_aw_value(v)), src.clone()),
        None => (None, None),
    };

    if json {
        let doc = serde_json::json!({
            "element": target.qualified_name,
            "own": own,
            "effective": eff_str,
            "inheritedFrom": inherited,
        });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap_or_default());
        std::process::exit(0);
    }

    println!("{}", target.qualified_name);
    println!("  own:       {}", own.as_deref().unwrap_or("(none)"));
    match (&eff_str, &inherited) {
        (Some(e), Some(pkg)) => println!("  effective: {e}   (inherited from package {pkg})"),
        (Some(e), None) => println!("  effective: {e}"),
        (None, _) => println!(
            "  effective: always applies (no appliesWhen on it or any ancestor package)"
        ),
    }
    std::process::exit(0);
}

/// Format an appliesWhen value as a YAML scalar: a single bare token stays plain;
/// anything with spaces / operators / parentheses is double-quoted.
fn yaml_scalar(expr: &str) -> String {
    let plain = !expr.is_empty()
        && expr
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '.' | '-'));
    if plain {
        expr.to_string()
    } else {
        format!("\"{}\"", expr.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// Return the YAML lines with any top-level `appliesWhen:` key (and its indented
/// continuation lines) removed.
fn strip_applies_when(yaml: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut it = yaml.lines().peekable();
    while let Some(line) = it.next() {
        if line.starts_with("appliesWhen:") {
            while let Some(next) = it.peek() {
                if !next.is_empty() && next.starts_with(|c: char| c.is_whitespace()) {
                    it.next();
                } else {
                    break;
                }
            }
            continue;
        }
        out.push(line.to_string());
    }
    out
}

/// Splice `new_fm` into `content` in place of the borrowed `yaml` region, leaving
/// the delimiters and body byte-identical.
fn splice_frontmatter(content: &str, yaml: &str, new_fm: &str) -> String {
    let base = content.as_ptr() as usize;
    let start = yaml.as_ptr() as usize - base;
    let end = start + yaml.len();
    format!("{}{}{}", &content[..start], new_fm, &content[end..])
}

/// Run the deep feature-model bad-configuration analysis and exit with its code
/// (non-zero on void / dead / invalid configurations). Exits `0` when there is no
/// feature model to check.
fn feature_check_and_exit(elements: &[RawElement]) -> ! {
    let gate = GateOptions::default();
    query::cmd_feature_check(elements, false, true, false, false, None, &gate);
    std::process::exit(0);
}

/// `applies-when` subcommand entry point.
#[allow(clippy::too_many_arguments)]
pub fn cmd_applies_when(
    model_root: &Path,
    elements: &[RawElement],
    resolver: &Resolver,
    target_key: &str,
    set_expr: Option<&str>,
    clear: bool,
    json: bool,
    dry_run: bool,
) -> ! {
    let target = match resolver.resolve_ref(elements, target_key) {
        Some(e) => e,
        None => {
            eprintln!("Element not found: {target_key}");
            std::process::exit(1);
        }
    };

    // ── read mode: no --set / --clear → display the gate (REQ-TRS-AW-002) ──────
    if set_expr.is_none() && !clear {
        show_gate(elements, target, json);
    }

    let file_path = target.file_path.clone();
    let qname = target.qualified_name.clone();

    let content = match std::fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cannot read {file_path}: {e}");
            std::process::exit(1);
        }
    };
    let (yaml_opt, _body) = split_frontmatter(&content);
    let yaml = match yaml_opt {
        Some(y) => y,
        None => {
            eprintln!("{file_path} has no YAML frontmatter to edit.");
            std::process::exit(1);
        }
    };

    // ── --clear ───────────────────────────────────────────────────────────────
    if clear {
        if !yaml.lines().any(|l| l.starts_with("appliesWhen:")) {
            println!("{qname} has no appliesWhen — nothing to clear.");
            std::process::exit(0);
        }
        let new_fm = strip_applies_when(yaml).join("\n");
        if dry_run {
            println!("[dry-run] would clear appliesWhen on {qname} ({file_path})");
            std::process::exit(0);
        }
        let new_content = splice_frontmatter(&content, yaml, &new_fm);
        if let Err(e) = std::fs::write(&file_path, new_content) {
            eprintln!("Write failed for {file_path}: {e}");
            std::process::exit(1);
        }
        println!("Cleared appliesWhen on {qname} ({file_path})");
        std::process::exit(0);
    }

    // ── --set ─────────────────────────────────────────────────────────────────
    let expr = match set_expr {
        Some(e) => e.trim(),
        None => {
            eprintln!("applies-when requires --set \"<expr>\" or --clear");
            std::process::exit(1);
        }
    };

    // (1) parse with the appliesWhen boolean grammar
    if let Err(e) = syscribe_model::variability::parse(expr) {
        eprintln!("Invalid appliesWhen expression '{expr}': {e}");
        std::process::exit(1);
    }

    // (2) apply to an in-memory copy and validate — refuse on E209 / E228 for the
    //     target (reuses the full operand-resolution + placement validator).
    let mut clones: Vec<RawElement> = elements.to_vec();
    for e in clones.iter_mut() {
        if e.qualified_name == qname {
            e.frontmatter.applies_when = Some(serde_yaml::Value::String(expr.to_string()));
        }
    }
    let vcfg = ValidateConfig::with_model_root(model_root);
    let result = validator::validate_with_config(&clones, &vcfg);
    let blockers: Vec<_> = result
        .errors()
        .filter(|f| f.file == file_path && matches!(f.code, "E209" | "E228"))
        .collect();
    if !blockers.is_empty() {
        eprintln!("Refusing to set appliesWhen on {qname} — the gate would be invalid:");
        for f in blockers {
            eprintln!("  {} {}", f.code, f.message);
        }
        std::process::exit(1);
    }

    // (3) write the field, byte-preserving for everything else
    let mut lines = strip_applies_when(yaml);
    lines.push(format!("appliesWhen: {}", yaml_scalar(expr)));
    let new_fm = lines.join("\n");

    if dry_run {
        println!("[dry-run] would set on {qname} ({file_path}):  appliesWhen: {}", yaml_scalar(expr));
        println!("[dry-run] feature-model bad-configuration check on the result:");
        println!();
        feature_check_and_exit(&clones);
    }

    let new_content = splice_frontmatter(&content, yaml, &new_fm);
    if let Err(e) = std::fs::write(&file_path, new_content) {
        eprintln!("Write failed for {file_path}: {e}");
        std::process::exit(1);
    }
    println!("Set appliesWhen: {} on {qname} ({file_path})", yaml_scalar(expr));
    println!();
    println!("Validating the feature model for bad configurations…");
    println!();
    feature_check_and_exit(&clones);
}
