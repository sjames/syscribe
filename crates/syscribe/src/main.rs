#![deny(warnings)]

mod audit;
mod aw;
mod coanalysis;
mod cyberrisk;
mod connectivity;
mod diagram;
mod discover;
mod export;
mod fmea_report;
mod help;
mod ingest;
mod lint_docs;
mod matrix;
mod metrics_cmd;
mod mgreport;
mod mv;
mod query;
mod render;
mod reviews;
mod safety_case;
mod scaffold;
mod scripting;
mod testplan;
mod vdepth;
mod spec;

use std::collections::{BTreeMap, HashMap};
use syscribe_model::{
    config::ValidateConfig,
    element::{ElementType, RawElement},
    resolver::{is_adr_id, is_req_id, is_tc_id, Resolver},
    results::ResultsData,
    validator,
    walker,
};

// ── helpers ───────────────────────────────────────────────────────────────────

fn is_native_req(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::Requirement))
        && e.frontmatter.id.as_deref().map(is_req_id).unwrap_or(false)
}

fn is_native_tc(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::TestCase))
        && e.frontmatter.id.as_deref().map(is_tc_id).unwrap_or(false)
}

fn is_native_adr(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::ADR))
        && e.frontmatter.id.as_deref().map(is_adr_id).unwrap_or(false)
}

fn is_arch_element(e: &RawElement) -> bool {
    matches!(
        e.frontmatter.element_type,
        Some(ElementType::PartDef) | Some(ElementType::Part)
    )
}

/// REQ-TRS-LINK-004 — render an element reference as a Markdown link to its
/// hosted source URL when `[links]` resolves one, else the plain `label`. The
/// URL is used verbatim (already encoded by REQ-TRS-LINK-001).
fn linked_label(label: &str, e: &RawElement, cfg: &ValidateConfig) -> String {
    match cfg.hosted_url_for(&e.file_path, &e.qualified_name, e.frontmatter.id.as_deref().unwrap_or("")) {
        Some(url) => format!("[{}]({})", label, url),
        None => label.to_string(),
    }
}

/// Count `Scenario:` lines across the doc body.
fn count_gherkin_scenarios(doc: &str) -> usize {
    doc.lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("Scenario:") || t.starts_with("Scenario Outline:")
        })
        .count()
}

/// Auto-discover the model root (REQ-TRS-CLI-004): walk upward from the current
/// working directory and return the nearest ancestor that contains a
/// `.syscribe.toml` file. Tooling locator only — it never affects model
/// semantics. `None` when no ancestor carries the marker (callers fall through
/// to the `model` default).
fn discover_model_root() -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let mut dir: &std::path::Path = cwd.as_path();
    loop {
        if dir.join(".syscribe.toml").is_file() {
            return Some(dir.to_string_lossy().into_owned());
        }
        dir = dir.parent()?;
    }
}

/// Resolve the `--config` lens for a read-only analysis command (GH #35): returns
/// the element set projected onto the configuration (only elements active per
/// `appliesWhen`), or the full set when `--config` is absent or there is no
/// feature model. Exits 1 on an unresolvable argument.
fn projected_elements(elems: &[RawElement], config: Option<&str>) -> Vec<RawElement> {
    use syscribe_model::projection::{project, resolve_selection, SelectionOutcome};
    match config {
        None => elems.to_vec(),
        Some(c) => match resolve_selection(elems, c) {
            SelectionOutcome::Dormant => elems.to_vec(),
            SelectionOutcome::Resolved(sel) => project(elems, &sel),
            SelectionOutcome::Error(m) => {
                eprintln!("Error: {m}");
                std::process::exit(1);
            }
        },
    }
}

/// Resolve the `--plan TP-X` lens (REQ-TRS-PLAN-006), composing with the
/// `--config` projection. The plan lens restricts the element set to the plan's
/// in-scope requirements ∪ effective TestCases ∪ their satisfying architecture
/// elements ∪ the plan's Configurations; `--config` then projects onto the
/// variant. Either lens may be absent. Exits 1 on an unresolvable plan id and on
/// an unresolvable configuration. Dormant-safe.
fn lensed_elements(
    elems: &[RawElement],
    plan: Option<&str>,
    config: Option<&str>,
) -> Vec<RawElement> {
    // Apply the plan lens first (it is the coarser scope), then project.
    let scoped: Vec<RawElement> = match plan {
        None => elems.to_vec(),
        Some(tp) => testplan::plan_lens(elems, tp),
    };
    projected_elements(&scoped, config)
}

/// Extract the top-level package name from `file_path`, given a model root prefix.
fn top_level_package(file_path: &str, model_root: &str) -> String {
    // Strip the model root prefix (with trailing slash) and split on '/'
    let rel = file_path
        .strip_prefix(model_root)
        .and_then(|s| s.strip_prefix('/'))
        .unwrap_or(file_path);
    let parts: Vec<&str> = rel.splitn(2, '/').collect();
    if parts.len() == 2 {
        // There is a subdirectory — the first segment is the top-level package.
        parts[0].to_string()
    } else {
        // File is directly in the model root (no subdirectory).
        "(root)".to_string()
    }
}

const AGENT_INSTRUCTIONS: &str = include_str!("../../../prompts/create-model.md");
// REQ-TRS-CLI-006: `--agent-instructions magicgrid` teaches MagicGrid modeling.
const MAGICGRID_INSTRUCTIONS: &str = include_str!("../../../prompts/create-magicgrid-model.md");

/// Parse CI severity-gating flags for the `validate` subcommand (issue #3):
///   --deny <CODES>          comma-separated warning codes to treat as gate failures
///   --max-warnings <N>      fail when warnings exceed N
///   --warnings-as-errors    promote every warning to a gate failure
fn parse_gate_options(args: &[String]) -> query::GateOptions {
    let mut gate = query::GateOptions::default();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "--warnings-as-errors" {
            gate.warnings_as_errors = true;
        } else if a == "--deny" {
            if let Some(val) = args.get(i + 1) {
                for code in val.split(',') {
                    let c = code.trim();
                    if !c.is_empty() {
                        gate.deny.insert(c.to_string());
                    }
                }
                i += 1;
            }
        } else if let Some(val) = a.strip_prefix("--deny=") {
            for code in val.split(',') {
                let c = code.trim();
                if !c.is_empty() {
                    gate.deny.insert(c.to_string());
                }
            }
        } else if a == "--max-warnings" {
            if let Some(val) = args.get(i + 1) {
                match val.parse::<usize>() {
                    Ok(n) => gate.max_warnings = Some(n),
                    Err(_) => {
                        eprintln!("Error: --max-warnings expects a non-negative integer, got '{}'", val);
                        std::process::exit(1);
                    }
                }
                i += 1;
            }
        } else if let Some(val) = a.strip_prefix("--max-warnings=") {
            match val.parse::<usize>() {
                Ok(n) => gate.max_warnings = Some(n),
                Err(_) => {
                    eprintln!("Error: --max-warnings expects a non-negative integer, got '{}'", val);
                    std::process::exit(1);
                }
            }
        }
        i += 1;
    }
    gate
}

/// Collect and parse every `--where <pred>` / `--where=<pred>` occurrence (GH #39).
/// Multiple predicates are returned in order and ANDed by the caller. An unparseable
/// predicate prints a usage error to stderr and exits non-zero.
fn parse_where_options(args: &[String]) -> Vec<query::CustomWhere> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        let raw = if a == "--where" {
            let v = args.get(i + 1).map(|s| s.as_str());
            i += 1;
            match v {
                Some(v) => Some(v),
                None => {
                    eprintln!("Error: --where expects a predicate (e.g. custom.supplier=Bosch)");
                    std::process::exit(1);
                }
            }
        } else {
            a.strip_prefix("--where=")
        };
        if let Some(raw) = raw {
            match query::parse_custom_where(raw) {
                Ok(p) => out.push(p),
                Err(msg) => {
                    eprintln!("Error: {msg}");
                    std::process::exit(1);
                }
            }
        }
        i += 1;
    }
    out
}

/// REQ-TRS-SCRIPT-005/006 — the `scripts` command family. Returns the process
/// exit code (0 clean, 1 on error/runtime failure, 2 on a tripped gate). Loads
/// the sandboxed script environment lazily; a load failure (e.g. duplicate name)
/// is reported and exits non-zero. Independent of the built-in `validate`.
fn cmd_scripts(
    elems: &[RawElement],
    vcfg: &ValidateConfig,
    sub: &str,
    rest: &[String],
) -> i32 {
    use scripting::{ScriptEnv, ScriptError};

    // `scripts` with no/unknown subcommand → usage.
    match sub {
        "list" | "run" | "validate" => {}
        "" => {
            eprintln!("Usage: syscribe -m <root> scripts <list|run|validate> [...]");
            return 1;
        }
        other => {
            eprintln!("Unknown scripts subcommand: {other}");
            eprintln!("Usage: syscribe -m <root> scripts <list|run|validate> [...]");
            return 1;
        }
    }

    let env = match ScriptEnv::load(vcfg) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Error loading extension scripts: {}", err.message());
            return 1;
        }
    };

    let json = rest.iter().any(|a| a == "--json");

    match sub {
        "list" => {
            env.list(json);
            0
        }
        "run" => {
            // First non-flag token is the command name.
            let name = rest.iter().find(|a| !a.starts_with("--")).map(|s| s.as_str());
            let name = match name {
                Some(n) => n,
                None => {
                    eprintln!("Usage: syscribe -m <root> scripts run <command> [--json]");
                    return 1;
                }
            };
            match env.run_command(elems, vcfg, name) {
                Ok(out) => {
                    if !out.is_empty() {
                        println!("{out}");
                    }
                    0
                }
                Err(err) => {
                    match err {
                        ScriptError::NotFound(m) | ScriptError::Runtime(m) | ScriptError::Load(m) => {
                            eprintln!("Error: {m}");
                        }
                    }
                    1
                }
            }
        }
        "validate" => {
            let gate = parse_gate_options(rest);
            let (findings, runtime_error) = env.run_checks(elems, vcfg);
            scripts_validate_report(&findings, &gate, runtime_error, json)
        }
        _ => unreachable!(),
    }
}

/// Render `scripts validate` findings and compute the exit code (REQ-TRS-SCRIPT-006).
/// Findings are namespaced `<check>/<code>`; the exit contract mirrors built-in
/// `validate`: 1 on any error-severity finding, 2 on a tripped gate, else 0. A
/// runtime check failure also forces a non-zero (1) exit.
fn scripts_validate_report(
    findings: &[scripting::ScriptFinding],
    gate: &query::GateOptions,
    runtime_error: bool,
    json: bool,
) -> i32 {
    let ns = |f: &scripting::ScriptFinding| format!("{}/{}", f.check, f.code);

    let errors: Vec<&scripting::ScriptFinding> =
        findings.iter().filter(|f| f.severity == "error").collect();
    let warnings: Vec<&scripting::ScriptFinding> =
        findings.iter().filter(|f| f.severity == "warning").collect();
    let infos: Vec<&scripting::ScriptFinding> =
        findings.iter().filter(|f| f.severity == "info").collect();

    // Gate evaluation over the namespaced codes (`<check>/<code>`).
    let denied: Vec<&scripting::ScriptFinding> = if gate.warnings_as_errors {
        warnings.clone()
    } else {
        warnings.iter().filter(|f| gate.deny.contains(&ns(f))).copied().collect()
    };
    let denied_infos: Vec<&scripting::ScriptFinding> =
        infos.iter().filter(|f| gate.deny.contains(&ns(f))).copied().collect();
    let over_max = gate.max_warnings.map_or(false, |m| warnings.len() > m);
    let gate_tripped = !denied.is_empty() || !denied_infos.is_empty() || over_max;

    let exit_code = if !errors.is_empty() || runtime_error {
        1
    } else if gate_tripped {
        2
    } else {
        0
    };

    if json {
        let items: Vec<serde_json::Value> = findings
            .iter()
            .map(|f| {
                serde_json::json!({
                    "code": ns(f),
                    "severity": f.severity,
                    "file": f.file,
                    "message": f.message,
                    "source": f.source,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
        return exit_code;
    }

    if findings.is_empty() {
        println!("0 errors, 0 warnings — no extension-check findings.");
        return exit_code;
    }

    println!("| Code | Severity | File | Message | Source |");
    println!("|---|---|---|---|---|");
    for f in findings {
        println!(
            "| {} | {} | {} | {} | {} |",
            ns(f),
            f.severity,
            f.file,
            f.message,
            f.source
        );
    }
    exit_code
}

/// The top-level command registry and router (REQ-TRS-CLI-008). Subcommands are derived
/// from the single embedded help-page list (`help::commands()`) so the router cannot
/// drift from the man pages. Used only to validate the command line and reject unknown
/// commands; each subcommand collects its arguments as trailing values so the existing
/// per-command handlers parse their own flags unchanged. `--help`/`--version`/`spec`/
/// `--agent-instructions` are handled before this router runs and are intentionally not
/// modelled here.
fn build_cli() -> clap::Command {
    let mut cmd = clap::Command::new("syscribe")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .subcommand_required(false)
        .arg_required_else_help(false)
        .allow_external_subcommands(false)
        .arg(
            clap::Arg::new("model")
                .short('m')
                .long("model")
                .num_args(1)
                .global(true),
        );
    for (name, about) in help::commands() {
        cmd = cmd.subcommand(
            clap::Command::new(name).about(about).arg(
                clap::Arg::new("args")
                    .num_args(0..)
                    .trailing_var_arg(true)
                    .allow_hyphen_values(true),
            ),
        );
    }
    cmd
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // REQ-TRS-CLI-007: version reporting. `--version`, `-V`, or the `version`
    // subcommand print "syscribe <semver>" to stdout and exit 0, handled before any
    // model resolution so they work from any directory with no model present.
    if args.iter().skip(1).any(|a| a == "--version" || a == "-V")
        || args.get(1).map(|a| a == "version").unwrap_or(false)
    {
        println!("syscribe {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // REQ-TRS-CLI-006: `--agent-instructions [topic]`. No topic (or `general`) →
    // the general modeling prompt; `magicgrid` → the MagicGrid modeling prompt; an
    // unknown topic exits non-zero. The token after the flag is a topic only when it
    // is not itself an option.
    if let Some(pos) = args.iter().position(|a| a == "--agent-instructions") {
        let topic = args.get(pos + 1).map(|s| s.as_str()).filter(|t| !t.starts_with('-'));
        match topic {
            None | Some("general") => print!("{}", AGENT_INSTRUCTIONS),
            Some("magicgrid") => print!("{}", MAGICGRID_INSTRUCTIONS),
            Some(other) => {
                eprintln!(
                    "Unknown --agent-instructions topic '{}'. Available topics: magicgrid (or omit for the general modeling prompt).",
                    other
                );
                std::process::exit(2);
            }
        }
        return;
    }

    // Detailed help (REQ-TRS-CLI-005), handled before model resolution so it
    // works without a model directory.
    //   syscribe help [<command>]
    //   syscribe <command> --help | -h   (also `spec --help`, etc.)
    if args.iter().skip(1).any(|a| a == "--help" || a == "-h") {
        match args.iter().skip(1).find(|a| help::is_command(a)) {
            Some(cmd) => print!("{}", help::page(cmd).unwrap_or("")),
            None => query::print_help(), // bare `--help` → command index
        }
        return;
    }
    if args.get(1).map(|a| a == "help").unwrap_or(false) {
        help::cmd_help(args.get(2).map(|s| s.as_str()));
        return;
    }

    if args.get(1).map(|a| a == "spec").unwrap_or(false) {
        let section = args.get(2).map(|s| s.as_str()).unwrap_or("toc");
        spec::cmd_spec(section);
        return;
    }

    let top_help = args.get(1).map(|a| a == "--help" || a == "-h").unwrap_or(false);
    if top_help || args.len() == 1 {
        query::print_help();
        return;
    }

    // REQ-TRS-CLI-008: validate the command line through the clap registry before any
    // model work, so an unknown command is rejected with a clear error + non-zero exit
    // from any directory. `--help`/`-h`/`help`/`--version`/`version`/`--agent-instructions`/
    // `spec` are all handled above, so clap only sees real commands here. clap is used
    // for validation/rejection only — the matched values are discarded and the existing
    // hand-rolled dispatch below runs unchanged (per-command flags pass through as
    // trailing args).
    if let Err(e) = build_cli().try_get_matches_from(&args) {
        e.exit();
    }

    // Strip --model <path> or --model=<path> from args; collect remaining args.
    let mut remaining: Vec<String> = Vec::new();
    let mut model_flag: Option<String> = None;
    {
        let mut iter = args[1..].iter();
        while let Some(a) = iter.next() {
            if a == "--model" || a == "-m" {
                model_flag = iter.next().cloned();
            } else if let Some(val) = a.strip_prefix("--model=") {
                model_flag = Some(val.to_string());
            } else {
                remaining.push(a.clone());
            }
        }
    }

    // Priority (REQ-TRS-CLI-004): --model flag > SYSCRIBE_MODEL env > walk-up to
    // the nearest ancestor `.syscribe.toml` > the literal "model" default.
    let model_root_arg = model_flag
        .or_else(|| std::env::var("SYSCRIBE_MODEL").ok())
        .or_else(discover_model_root)
        .unwrap_or_else(|| "model".to_string());

    let subcommand_args: &[String] = &remaining;

    let model_root = std::path::Path::new(&model_root_arg);
    let model_root_str = model_root.to_string_lossy().into_owned();

    if !model_root.exists() {
        eprintln!("Error: model path does not exist: {}", model_root_str);
        std::process::exit(1);
    }
    if !model_root.is_dir() {
        eprintln!("Error: model path is not a directory: {}", model_root_str);
        std::process::exit(1);
    }

    let elems = match walker::walk_model(model_root) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Error walking model: {}", err);
            std::process::exit(1);
        }
    };

    // Validation config rooted at the model directory so on-disk references
    // (e.g. `sourceFile:`) resolve correctly per spec §11.12.
    let vcfg = ValidateConfig::with_model_root(model_root);

    // ── Subcommand dispatch ───────────────────────────────────────────────────
    // `report` (and a bare invocation with no subcommand) fall through to the default
    // full validation report below (REQ-TRS-CLI-008).
    if let Some(subcmd) = subcommand_args.first().map(|s| s.as_str()).filter(|s| *s != "report") {
        let resolver = Resolver::new(&elems);
        // subcommand_args[0] = subcommand, subcommand_args[1] = key, subcommand_args[2] = scope, …
        let key = subcommand_args.get(1).map(|s| s.as_str()).unwrap_or("");
        match subcmd {
            "show" => {
                // Build a validation result with the MagicGrid index active so the
                // `actorIn` reverse index is available to surface (REQ-TRS-MG-002).
                let mut show_cfg = vcfg.clone();
                show_cfg.magicgrid = true;
                let result = validator::validate_with_config(&elems, &show_cfg);
                query::cmd_show(&elems, &resolver, &result, key);
            }
            "ls" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let wheres = parse_where_options(rest);
                // `ls` takes an optional positional parent that is not a flag/flag-value.
                let parent = if key.starts_with("--") || key == "--where" { "" } else { key };
                query::cmd_ls(&elems, parent, &wheres);
            }
            "tree" => {
                query::cmd_tree(&elems, key);
            }
            "find" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let wheres = parse_where_options(rest);
                if key.is_empty() || key.starts_with("--") {
                    eprintln!("Usage: syscribe --model <root> find <pattern> [--where custom.<key>[op<val>]]");
                    std::process::exit(1);
                }
                query::cmd_find(&elems, key, &wheres);
            }
            "extref" => {
                if key.is_empty() {
                    eprintln!("Usage: syscribe --model <root> extref <external-reference> [--json]");
                    std::process::exit(1);
                }
                let json = subcommand_args.iter().any(|a| a == "--json");
                if !query::cmd_extref(&elems, key, json) {
                    std::process::exit(1);
                }
            }
            "links" => {
                query::cmd_links(&elems, &resolver, key);
            }
            "connectivity" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let code = connectivity::cmd_connectivity(&elems, &resolver, rest);
                if code != 0 {
                    std::process::exit(code);
                }
            }
            "refs" => {
                query::cmd_refs(&elems, &resolver, key);
            }
            "render" => {
                if key.is_empty() {
                    eprintln!("Usage: syscribe --model <root> render <diagram_path>");
                    std::process::exit(1);
                }
                render::cmd_render(&elems, &resolver, key, &vcfg);
            }
            "diagram" => {
                let sub = subcommand_args.get(1).map(|s| s.as_str()).unwrap_or("");
                let rest: Vec<String> = subcommand_args.get(2..).unwrap_or(&[]).to_vec();
                diagram::cmd_diagram(&elems, &resolver, sub, &rest, &vcfg);
            }
            "validate" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let file_filter = rest.windows(2)
                    .find(|w| w[0] == "--file")
                    .map(|w| w[1].as_str());
                let gate = parse_gate_options(rest);
                // Named severity profile (issue #18): load `[profiles.<name>]` from
                // <model_root>/.syscribe.toml. An undefined name (or missing file) is
                // a usage error → exit 1.
                let profile_name = rest.windows(2)
                    .find(|w| w[0] == "--profile")
                    .map(|w| w[1].as_str());
                let profile = if let Some(name) = profile_name {
                    let profiles = syscribe_model::config::load_profiles(model_root);
                    match profiles.get(name) {
                        Some(p) => Some(p.clone()),
                        None => {
                            eprintln!(
                                "Error: profile '{}' is not defined in {}/.syscribe.toml",
                                name,
                                model_root.display()
                            );
                            std::process::exit(1);
                        }
                    }
                } else {
                    None
                };
                let profile_ref = profile.as_ref();
                // Ad-hoc results ingest for this run (does not write the sidecar).
                let results_file = rest.windows(2)
                    .find(|w| w[0] == "--results")
                    .map(|w| w[1].as_str());
                let mut vcfg_run = vcfg.clone();
                // REQ-TRS-MG-*: a magicgrid-enabled profile (`magicgrid = true`)
                // turns on the gated MagicGrid validation pass for this run.
                vcfg_run.magicgrid = profile_ref.map_or(false, |p| p.magicgrid);
                if let Some(rf) = results_file {
                    let fmt = rest.windows(2)
                        .find(|w| w[0] == "--format")
                        .map(|w| w[1].as_str());
                    let inferred = if rf.ends_with(".xml") { "junit" } else { "cargo-json" };
                    if let Some(data) = ingest::parse_file(fmt.unwrap_or(inferred), rf) {
                        vcfg_run.results = Some(data);
                    }
                }
                // Opt-in: enable the .syscribe.toml download hook for remote sourceFiles.
                if rest.iter().any(|a| a == "--fetch-remote") {
                    vcfg_run.remote_hook =
                        syscribe_model::remote::RemoteHook::load(model_root);
                    if vcfg_run.remote_hook.is_none() {
                        eprintln!("--fetch-remote: no [remote] download hook configured in .syscribe.toml");
                    }
                }
                // Configuration lens: --all-configs gate, or --config <C> projection.
                let all_configs = rest.iter().any(|a| a == "--all-configs");
                let config = rest.windows(2).find(|w| w[0] == "--config").map(|w| w[1].as_str());
                if all_configs {
                    query::cmd_validate_all_configs(&elems, &vcfg_run, json);
                } else if let Some(c) = config {
                    match syscribe_model::projection::resolve_selection(&elems, c) {
                        syscribe_model::projection::SelectionOutcome::Dormant => {
                            query::cmd_validate(&elems, &vcfg_run, &gate, profile_ref, file_filter, json)
                        }
                        syscribe_model::projection::SelectionOutcome::Resolved(sel) => {
                            query::cmd_validate_projected(&elems, &vcfg_run, &gate, json, &sel)
                        }
                        syscribe_model::projection::SelectionOutcome::Error(m) => {
                            eprintln!("{m}");
                            std::process::exit(2);
                        }
                    }
                } else {
                    query::cmd_validate(&elems, &vcfg_run, &gate, profile_ref, file_filter, json);
                }
            }
            "diff" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let cfgs: Vec<&str> = rest
                    .windows(2)
                    .filter(|w| w[0] == "--config")
                    .map(|w| w[1].as_str())
                    .collect();
                if cfgs.len() != 2 {
                    eprintln!("Usage: syscribe --model <root> diff --config <A> --config <B> [--json]");
                    std::process::exit(1);
                }
                query::cmd_diff(&elems, cfgs[0], cfgs[1], json);
            }
            "audit" => {
                // Read-only safety-readiness dashboard (GH #15 / REQ-TRS-OUT-013).
                // Reuses validate_with_config, the matrix coverage computation and
                // the issue-#18 profile loader/promotion. Exit 0 PASS · 2 FAIL.
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let profile_name = rest
                    .windows(2)
                    .find(|w| w[0] == "--profile")
                    .map(|w| w[1].as_str());
                let profile = if let Some(name) = profile_name {
                    let profiles = syscribe_model::config::load_profiles(model_root);
                    match profiles.get(name) {
                        Some(p) => Some(p.clone()),
                        None => {
                            eprintln!(
                                "Error: profile '{}' is not defined in {}/.syscribe.toml",
                                name,
                                model_root.display()
                            );
                            std::process::exit(1);
                        }
                    }
                } else {
                    None
                };
                // Configuration lens (GH #35): project the dashboard onto a variant.
                // Plan lens (GH #40): `--plan TP-X` validates the FULL model and counts
                // only findings whose element is in the plan's scope (no escaping-ref
                // artifacts); the sections are scoped to the plan but resolve refs
                // against the full model. The two lenses compose.
                let all_configs = rest.iter().any(|a| a == "--all-configs");
                let config = rest.windows(2).find(|w| w[0] == "--config").map(|w| w[1].as_str());
                let plan = rest.windows(2).find(|w| w[0] == "--plan").map(|w| w[1].as_str());
                let plan_scope: Option<std::collections::HashSet<String>> = plan.map(|tp| {
                    // plan_lens exits 1 on an unknown plan id.
                    testplan::plan_lens(&elems, tp)
                        .iter()
                        .map(|e| e.file_path.clone())
                        .collect()
                });
                let ps = plan_scope.as_ref();
                // REQ-TRS-MG-*: a magicgrid-enabled profile turns on the gated pass.
                let mut vcfg_audit = vcfg.clone();
                vcfg_audit.magicgrid = profile.as_ref().map_or(false, |p| p.magicgrid);
                let vcfg = &vcfg_audit;
                let code = if all_configs {
                    audit::cmd_audit_all_configs(&elems, vcfg, profile.as_ref(), json)
                } else if let Some(c) = config {
                    match syscribe_model::projection::resolve_selection(&elems, c) {
                        syscribe_model::projection::SelectionOutcome::Dormant => {
                            audit::cmd_audit(&elems, vcfg, model_root, profile.as_ref(), None, ps, json)
                        }
                        syscribe_model::projection::SelectionOutcome::Resolved(sel) => {
                            audit::cmd_audit(&elems, vcfg, model_root, profile.as_ref(), Some(&sel), ps, json)
                        }
                        syscribe_model::projection::SelectionOutcome::Error(m) => {
                            eprintln!("Error: {m}");
                            std::process::exit(1);
                        }
                    }
                } else {
                    audit::cmd_audit(&elems, vcfg, model_root, profile.as_ref(), None, ps, json)
                };
                if code != 0 {
                    std::process::exit(code);
                }
            }
            "ingest-results" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let format = rest.windows(2)
                    .find(|w| w[0] == "--format")
                    .map(|w| w[1].as_str());
                // The results file is the first positional arg (not a flag/flag-value).
                let mut file: Option<&str> = None;
                let mut i = 0;
                while i < rest.len() {
                    if rest[i] == "--format" { i += 2; continue; }
                    if rest[i].starts_with("--") { i += 1; continue; }
                    file = Some(rest[i].as_str());
                    break;
                }
                match file {
                    Some(f) => ingest::cmd_ingest_results(model_root, format, f),
                    None => {
                        eprintln!("Usage: syscribe --model <root> ingest-results [--format cargo-json|junit] <file>");
                        std::process::exit(1);
                    }
                }
            }
            "export" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let ndjson = rest.iter().any(|a| a == "--ndjson");
                let config = rest.windows(2).find(|w| w[0] == "--config").map(|w| w[1].as_str());
                match config {
                    None => export::cmd_export(&elems, &vcfg, ndjson),
                    Some(c) => match syscribe_model::projection::resolve_selection(&elems, c) {
                        syscribe_model::projection::SelectionOutcome::Dormant => {
                            export::cmd_export(&elems, &vcfg, ndjson)
                        }
                        syscribe_model::projection::SelectionOutcome::Resolved(sel) => {
                            let view = syscribe_model::projection::project(&elems, &sel);
                            export::cmd_export(&view, &vcfg, ndjson);
                        }
                        syscribe_model::projection::SelectionOutcome::Error(m) => {
                            eprintln!("{m}");
                            std::process::exit(1);
                        }
                    },
                }
            }
            "types" => {
                query::cmd_types(&elems);
            }
            "untyped" => {
                query::cmd_untyped(&elems);
            }
            "list" => {
                if key.is_empty() {
                    eprintln!("Usage: syscribe --model <root> list <type> [scope] [--tag <t>]... [--config <c>] [--feature <F>] [--status <s>] [--sil <v>] [--json]");
                    eprintln!("       --tag may be repeated; all tags must be present (AND)");
                    std::process::exit(1);
                }
                let rest = subcommand_args.get(2..).unwrap_or(&[]);
                let tags: Vec<&str> = rest
                    .windows(2)
                    .filter(|w| w[0] == "--tag")
                    .map(|w| w[1].as_str())
                    .collect();
                let config = rest
                    .windows(2)
                    .find(|w| w[0] == "--config")
                    .map(|w| w[1].as_str());
                let feature = rest
                    .windows(2)
                    .find(|w| w[0] == "--feature")
                    .map(|w| w[1].as_str());
                let metadata = rest
                    .windows(2)
                    .find(|w| w[0] == "--metadata")
                    .map(|w| w[1].as_str());
                let status = rest
                    .windows(2)
                    .find(|w| w[0] == "--status")
                    .map(|w| w[1].as_str());
                let sil = rest
                    .windows(2)
                    .find(|w| w[0] == "--sil")
                    .map(|w| w[1].as_str());
                let has_wcet = rest.iter().any(|a| a == "--has-wcet");
                let json = rest.iter().any(|a| a == "--json");
                let wheres = parse_where_options(rest);
                // scope = first positional argument that is not a flag or flag
                // value. Two-arg flags consume their value so it is not mistaken
                // for the positional scope.
                let mut scope = "";
                let mut i = 0;
                while i < rest.len() {
                    if matches!(
                        rest[i].as_str(),
                        "--tag" | "--config" | "--feature" | "--metadata" | "--status" | "--sil"
                            | "--where"
                    ) {
                        i += 2;
                        continue;
                    }
                    if rest[i].starts_with("--") {
                        i += 1;
                        continue;
                    }
                    scope = rest[i].as_str();
                    break;
                }
                match config {
                    None => query::cmd_list(&elems, key, scope, &tags, feature, metadata, status, sil, has_wcet, &wheres, json),
                    Some(c) => match syscribe_model::projection::resolve_selection(&elems, c) {
                        syscribe_model::projection::SelectionOutcome::Dormant => {
                            query::cmd_list(&elems, key, scope, &tags, feature, metadata, status, sil, has_wcet, &wheres, json)
                        }
                        syscribe_model::projection::SelectionOutcome::Resolved(sel) => {
                            let view = syscribe_model::projection::project(&elems, &sel);
                            query::cmd_list(&view, key, scope, &tags, feature, metadata, status, sil, has_wcet, &wheres, json);
                        }
                        syscribe_model::projection::SelectionOutcome::Error(m) => {
                            eprintln!("{m}");
                            std::process::exit(1);
                        }
                    },
                }
            }
            "matrix" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                // REQ-TRS-MG-006: `matrix --allocations` is an Allocation source ×
                // target matrix mode, distinct from the Requirement × Configuration grid.
                if rest.iter().any(|a| a == "--allocations") {
                    mgreport::cmd_matrix_allocations(&elems, json);
                    return;
                }
                let gaps_only = rest.iter().any(|a| a == "--gaps-only");
                let linked_only = rest.iter().any(|a| a == "--linked-only");
                let tag = rest
                    .windows(2)
                    .find(|w| w[0] == "--tag")
                    .map(|w| w[1].as_str());
                let status = rest
                    .windows(2)
                    .find(|w| w[0] == "--status")
                    .map(|w| w[1].as_str());
                let plan = rest.windows(2).find(|w| w[0] == "--plan").map(|w| w[1].as_str());
                let config = rest.windows(2).find(|w| w[0] == "--config").map(|w| w[1].as_str());
                // --plan lens (REQ-TRS-PLAN-006), composing with --config.
                let mut view = lensed_elements(&elems, plan, config);
                // matrix is a Requirement × Configuration grid; when --config names
                // a single stored Configuration, reduce the columns to just that
                // variant (projection alone keeps every Configuration element, so
                // the grid would otherwise still show every column — GH #38 review).
                if let Some(c) = config {
                    if let Some(keep) = elems.iter().find(|e| {
                        matches!(e.frontmatter.element_type, Some(ElementType::Configuration))
                            && (e.frontmatter.id.as_deref() == Some(c) || e.qualified_name == c)
                    }) {
                        let keep_qn = keep.qualified_name.clone();
                        view.retain(|e| {
                            !matches!(e.frontmatter.element_type, Some(ElementType::Configuration))
                                || e.qualified_name == keep_qn
                        });
                    }
                }
                if rest.iter().any(|a| a == "--features") {
                    matrix::cmd_matrix_features(&view, json);
                } else {
                    // Surface executed-evidence by default when a sidecar exists
                    // (issue #21); absent results, behaves exactly as before.
                    let results = ResultsData::load_sidecar(model_root);
                    matrix::cmd_matrix(
                        &view,
                        json,
                        tag,
                        status,
                        gaps_only,
                        results.as_ref(),
                        linked_only,
                    );
                }
            }
            "magicgrid" => {
                // REQ-TRS-MG-003: read-only B/W/S × 1-4 grid report over mg_cell.
                // REQ-TRS-MG-013: `--audit` rolls up the gated MagicGrid findings,
                // a readiness summary, and a PASS/FAIL verdict (exit 0/2).
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                if rest.iter().any(|a| a == "--audit") {
                    let mut vcfg_audit = vcfg.clone();
                    vcfg_audit.magicgrid = true;
                    let result =
                        syscribe_model::validator::validate_with_config(&elems, &vcfg_audit);
                    std::process::exit(mgreport::cmd_magicgrid_audit(&elems, &result, json));
                }
                // REQ-TRS-MG-016: `--svg [-o <file>]` renders the grid as a standalone
                // SVG, usable as a Diagram element's companion.
                if rest.iter().any(|a| a == "--svg") {
                    let out_file = rest
                        .windows(2)
                        .find(|w| w[0] == "-o" || w[0] == "--output")
                        .map(|w| w[1].as_str());
                    mgreport::cmd_magicgrid_svg(&elems, out_file);
                } else {
                    mgreport::cmd_magicgrid(&elems, json);
                }
            }
            "trade-study" => {
                // REQ-TRS-MG-007: MoE-weighted trade study scoring Configurations.
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let config_filter: Vec<String> = rest
                    .windows(2)
                    .filter(|w| w[0] == "--config")
                    .map(|w| w[1].clone())
                    .collect();
                mgreport::cmd_trade_study(&elems, json, &config_filter);
            }
            "co-analysis" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let config = rest.windows(2).find(|w| w[0] == "--config").map(|w| w[1].as_str());
                let view = projected_elements(&elems, config);
                coanalysis::cmd_coanalysis(&view, json);
            }
            "metrics" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let config = rest.windows(2).find(|w| w[0] == "--config").map(|w| w[1].as_str());
                let view = projected_elements(&elems, config);
                metrics_cmd::cmd_metrics(&view, json);
            }
            "cyber-risk" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let config = rest.windows(2).find(|w| w[0] == "--config").map(|w| w[1].as_str());
                let view = projected_elements(&elems, config);
                cyberrisk::cmd_cyber_risk(&view, json);
            }
            "safety-case" => {
                // GSN safety-argument tree (issue #20). Read-only; reuses Resolver
                // and the issue-#21 results sidecar for TestCase verdicts.
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let no_implicit = rest.iter().any(|a| a == "--no-implicit");
                let config = rest.windows(2).find(|w| w[0] == "--config").map(|w| w[1].as_str());
                // optional positional <SG-id> = first non-flag arg that is not a flag value.
                let mut goal = "";
                let mut gi = 0;
                while gi < rest.len() {
                    if rest[gi] == "--config" {
                        gi += 2;
                        continue;
                    }
                    if rest[gi].starts_with("--") {
                        gi += 1;
                        continue;
                    }
                    goal = rest[gi].as_str();
                    break;
                }
                let results = ResultsData::load_sidecar(model_root);
                let sidecar_loaded = results.is_some();
                let view = projected_elements(&elems, config);
                let view_resolver = Resolver::new(&view);
                safety_case::cmd_safety_case(&view, &view_resolver, goal, results.as_ref(), json, no_implicit, sidecar_loaded);
            }
            "testplan" => {
                // Read-only TestPlan surface (GH #38 / REQ-TRS-PLAN-005).
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let results = ResultsData::load_sidecar(model_root);
                // First non-flag positional = optional TP-id.
                let tp = rest.iter().find(|a| !a.starts_with("--")).map(|s| s.as_str());
                match tp {
                    None => testplan::cmd_testplan_list(&elems, json, results.as_ref()),
                    Some(id) => {
                        let code =
                            testplan::cmd_testplan_detail(&elems, id, json, results.as_ref());
                        if code != 0 {
                            std::process::exit(code);
                        }
                    }
                }
            }
            "reviews" => {
                // ReviewRecord surface (§19, GH #71). Read-only.
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let open_only = rest.iter().any(|a| a == "--open-only");
                let coverage = rest.iter().any(|a| a == "--coverage");
                let filter = rest.iter().find(|a| !a.starts_with("--")).map(|s| s.as_str());
                reviews::cmd_reviews(&elems, filter, open_only, coverage, json);
            }
            "review" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                match rest.iter().find(|a| !a.starts_with("--")) {
                    Some(id) => {
                        let code = reviews::cmd_review(&elems, id, json);
                        if code != 0 {
                            std::process::exit(code);
                        }
                    }
                    None => {
                        eprintln!("Usage: syscribe --model <root> review <RR-id> [--json]");
                        std::process::exit(2);
                    }
                }
            }
            "feature-check" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let deep = rest.iter().any(|a| a == "--deep");
                let count = rest.iter().any(|a| a == "--count");
                let enumerate = rest.iter().any(|a| a == "--enumerate");
                let prove = rest
                    .windows(2)
                    .find(|w| w[0] == "--prove")
                    .map(|w| w[1].as_str());
                let gate = parse_gate_options(rest);
                query::cmd_feature_check(&elems, json, deep, count, enumerate, prove, &gate);
            }
            "features" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                discover::cmd_features(&elems, json);
            }
            "feature" => {
                let rest = subcommand_args.get(2..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                if key.is_empty() || key.starts_with("--") {
                    eprintln!("Usage: syscribe --model <root> feature <qname|name> [--json]");
                    std::process::exit(1);
                }
                discover::cmd_feature(&elems, key, json);
            }
            "why-active" => {
                let rest = subcommand_args.get(2..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let config = rest
                    .windows(2)
                    .find(|w| w[0] == "--config")
                    .map(|w| w[1].as_str());
                if key.is_empty() || key.starts_with("--") {
                    eprintln!("Usage: syscribe --model <root> why-active <qname|id> --config <Configuration|features> [--json]");
                    std::process::exit(1);
                }
                discover::cmd_why_active(&elems, key, config, json);
            }
            "configure" => {
                let conf = subcommand_args.get(1).map(|s| s.as_str()).unwrap_or("");
                if conf.is_empty() || conf.starts_with("--") {
                    eprintln!("Usage: syscribe --model <root> configure <Configuration> [--json]");
                    std::process::exit(1);
                }
                let rest = subcommand_args.get(2..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                query::cmd_configure(&elems, conf, json);
            }
            "path-for" => {
                if key.is_empty() {
                    eprintln!("Usage: syscribe --model <root> path-for <qname|id>");
                    std::process::exit(1);
                }
                query::cmd_path_for(&elems, &resolver, key);
            }
            "check-ref" => {
                if key.is_empty() {
                    eprintln!("Usage: syscribe --model <root> check-ref <qname|id>");
                    std::process::exit(1);
                }
                query::cmd_check_ref(&elems, &resolver, key);
            }
            "next-id" => {
                if key.is_empty() {
                    eprintln!("Usage: syscribe --model <root> next-id <id-prefix>");
                    std::process::exit(1);
                }
                query::cmd_next_id(&elems, key);
            }
            "template" => {
                if key.is_empty() {
                    eprintln!("Usage: syscribe --model <root> template <type>");
                    std::process::exit(1);
                }
                query::cmd_template(key);
            }
            "move" => {
                let dest = subcommand_args.get(2).map(|s| s.as_str()).unwrap_or("");
                if key.is_empty() || dest.is_empty() {
                    eprintln!("Usage: syscribe --model <root> move <source-qname|id> <dest-qname> [--dry-run]");
                    std::process::exit(1);
                }
                let dry_run = subcommand_args.iter().any(|a| a == "--dry-run");
                mv::cmd_move(model_root, &elems, &resolver, key, dest, dry_run);
            }
            "scaffold-gherkin" => {
                if key.is_empty() {
                    eprintln!("Usage: syscribe --model <root> scaffold-gherkin <TC> [--fix]");
                    std::process::exit(1);
                }
                let fix = subcommand_args.iter().any(|a| a == "--fix");
                scaffold::cmd_scaffold_gherkin(&elems, &resolver, key, fix);
            }
            "applies-when" => {
                if key.is_empty() {
                    eprintln!("Usage: syscribe --model <root> applies-when <element-qname|id> (--set \"<expr>\" | --clear) [--dry-run]");
                    std::process::exit(1);
                }
                let set_expr = subcommand_args
                    .windows(2)
                    .find(|w| w[0] == "--set")
                    .map(|w| w[1].as_str());
                let clear = subcommand_args.iter().any(|a| a == "--clear");
                let dry_run = subcommand_args.iter().any(|a| a == "--dry-run");
                let json = subcommand_args.iter().any(|a| a == "--json");
                // No --set / --clear → read-only display of the gate (REQ-TRS-AW-002).
                if set_expr.is_some() && clear {
                    eprintln!("applies-when: --set and --clear are mutually exclusive");
                    std::process::exit(1);
                }
                aw::cmd_applies_when(model_root, &elems, &resolver, key, set_expr, clear, json, dry_run);
            }
            "trace" | "why" | "who-verifies" => {
                let result = validator::validate_with_config(&elems, &vcfg);
                match subcmd {
                    "trace" => {
                        let rest = subcommand_args.get(1..).unwrap_or(&[]);
                        let linked_only = rest.iter().any(|a| a == "--linked-only");
                        // Annotate verifying TestCases with ingested verdicts when a
                        // sidecar exists (issue #21).
                        let results = ResultsData::load_sidecar(model_root);
                        query::cmd_trace(
                            &elems,
                            &resolver,
                            &result,
                            key,
                            results.as_ref(),
                            linked_only,
                        );
                    }
                    "why" => query::cmd_why(&elems, &resolver, &result, key),
                    "who-verifies" => query::cmd_who_verifies(&elems, &resolver, &result, key),
                    _ => unreachable!(),
                }
            }
            "verification-depth" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let sil = rest.windows(2).find(|w| w[0] == "--sil").map(|w| w[1].as_str());
                let status = rest.windows(2).find(|w| w[0] == "--status").map(|w| w[1].as_str());
                let json = rest.iter().any(|a| a == "--json");
                let min_levels = rest
                    .windows(2)
                    .find(|w| w[0] == "--min-levels")
                    .and_then(|w| w[1].parse::<usize>().ok());
                let config = rest.windows(2).find(|w| w[0] == "--config").map(|w| w[1].as_str());
                let plan = rest.windows(2).find(|w| w[0] == "--plan").map(|w| w[1].as_str());
                // --plan lens (REQ-TRS-PLAN-006), composing with --config.
                let view = lensed_elements(&elems, plan, config);
                let view_resolver = Resolver::new(&view);
                let result = validator::validate_with_config(&view, &vcfg);
                let ok = vdepth::cmd_verification_depth(
                    &view, &view_resolver, &result, sil, status, min_levels, json,
                );
                if !ok {
                    std::process::exit(2);
                }
            }
            "scripts" => {
                // REQ-TRS-SCRIPT-005/006 — `scripts list|run|validate`. The
                // built-in `validate` never touches this path (separation).
                let sub = subcommand_args.get(1).map(|s| s.as_str()).unwrap_or("");
                let rest = subcommand_args.get(2..).unwrap_or(&[]);
                let code = cmd_scripts(&elems, &vcfg, sub, rest);
                if code != 0 {
                    std::process::exit(code);
                }
            }
            "fmea" => {
                let sub = subcommand_args.get(1).map(|s| s.as_str()).unwrap_or("");
                let rest = subcommand_args.get(2..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let sheet_filter = rest.windows(2).find(|w| w[0] == "--fmea-sheet").map(|w| w[1].as_str());
                match sub {
                    "report" => fmea_report::cmd_fmea_report(&elems, sheet_filter, json),
                    _ => {
                        eprintln!("Usage: syscribe -m <model> fmea report [--fmea-sheet <id>] [--json]");
                        std::process::exit(1);
                    }
                }
            }
            "fault-tree" => {
                let sub = subcommand_args.get(1).map(|s| s.as_str()).unwrap_or("");
                let rest = subcommand_args.get(2..).unwrap_or(&[]);
                let ft_id = rest.iter().find(|a| !a.starts_with("--")).map(|s| s.as_str()).unwrap_or("");
                match sub {
                    "render" => {
                        if ft_id.is_empty() {
                            eprintln!("Usage: syscribe -m <model> fault-tree render <FaultTree-id>");
                            std::process::exit(1);
                        }
                        fmea_report::cmd_fault_tree_render(&elems, ft_id);
                    }
                    _ => {
                        eprintln!("Usage: syscribe -m <model> fault-tree render <FaultTree-id>");
                        std::process::exit(1);
                    }
                }
            }
            "lint-docs" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let paths: Vec<&str> = rest.iter().filter(|a| !a.starts_with("--")).map(|s| s.as_str()).collect();
                if paths.is_empty() {
                    eprintln!("Usage: syscribe -m <model> lint-docs <path>... [--json]");
                    std::process::exit(1);
                }
                let code = lint_docs::cmd_lint_docs(&elems, &paths, json);
                if code != 0 {
                    std::process::exit(code);
                }
            }
            other => {
                eprintln!("Unknown command: {other}");
                eprintln!("Run `syscribe --help` for usage.");
                std::process::exit(1);
            }
        }
        return;
    }

    let result = validator::validate_with_config(&elems, &vcfg);

    let error_count = result.errors().count();
    let warning_count = result.warnings().count();
    let info_count = result.infos().count();

    // Pre-compute sets we reuse across sections
    let native_reqs: Vec<&RawElement> = elems.iter().filter(|e| is_native_req(e)).collect();
    let native_tcs: Vec<&RawElement> = elems.iter().filter(|e| is_native_tc(e)).collect();
    let native_adrs: Vec<&RawElement> = elems.iter().filter(|e| is_native_adr(e)).collect();

    let parent_ids: std::collections::HashSet<&str> = result
        .derived_children
        .keys()
        .map(|s| s.as_str())
        .collect();

    let req_count = native_reqs.len();
    let parent_req_count = native_reqs
        .iter()
        .filter(|e| {
            e.frontmatter
                .id
                .as_deref()
                .map(|id| parent_ids.contains(id))
                .unwrap_or(false)
        })
        .count();
    let leaf_req_count = req_count - parent_req_count;

    let tc_count = native_tcs.len();
    let adr_count = native_adrs.len();

    let gherkin_scenario_count: usize = native_tcs.iter().map(|e| count_gherkin_scenarios(&e.doc)).sum();

    let arch_with_satisfies = elems
        .iter()
        .filter(|e| {
            is_arch_element(e)
                && e.frontmatter.satisfies.as_ref().map_or(false, |v| !v.is_empty())
        })
        .count();
    let arch_without_satisfies = elems
        .iter()
        .filter(|e| {
            is_arch_element(e)
                && e.frontmatter.is_abstract != Some(true)
                && e.frontmatter.satisfies.as_ref().map_or(true, |v| v.is_empty())
        })
        .count();

    // ── Header ────────────────────────────────────────────────────────────────
    println!("# UAV Model Validation Report");
    println!();

    // ── Section 1: Executive Summary ─────────────────────────────────────────
    println!("## 1. Executive Summary");
    println!();
    println!("| Metric | Count |");
    println!("|---|---|");
    println!("| Total elements | {} |", elems.len());
    println!("| Errors | {} |", error_count);
    println!("| Warnings | {} |", warning_count);
    println!("| Informational | {} |", info_count);
    println!("| Requirements (total) | {} |", req_count);
    println!("| Requirements (parent) | {} |", parent_req_count);
    println!("| Requirements (leaf) | {} |", leaf_req_count);
    println!("| Test cases | {} |", tc_count);
    println!("| ADRs | {} |", adr_count);
    println!("| Gherkin scenarios | {} |", gherkin_scenario_count);
    println!("| Architecture elements with `satisfies` | {} |", arch_with_satisfies);
    println!("| Architecture elements without `satisfies` | {} |", arch_without_satisfies);
    println!();

    println!("---");
    println!();

    // ── Section 2: Validation Findings ───────────────────────────────────────
    println!("## 2. Validation Findings");
    println!();

    let errors: Vec<_> = result.errors().collect();
    let warnings: Vec<_> = result.warnings().collect();
    let infos: Vec<_> = result.infos().collect();

    if errors.is_empty() && warnings.is_empty() && infos.is_empty() {
        println!("> **All validation rules pass — 0 errors, 0 warnings.**");
    } else {
        if !errors.is_empty() {
            println!("### Errors");
            println!();
            println!("| Code | File | Message |");
            println!("|---|---|---|");
            for f in &errors {
                println!("| {} | {} | {} |", f.code, f.file, f.message);
            }
            println!();
        }
        if !warnings.is_empty() {
            println!("### Warnings");
            println!();
            println!("| Code | File | Message |");
            println!("|---|---|---|");
            for f in &warnings {
                println!("| {} | {} | {} |", f.code, f.file, f.message);
            }
            println!();
        }
        if !infos.is_empty() {
            println!("### Informational");
            println!();
            println!("| Code | File | Message |");
            println!("|---|---|---|");
            for f in &infos {
                println!("| {} | {} | {} |", f.code, f.file, f.message);
            }
            println!();
        }
    }

    println!("---");
    println!();

    // ── Section 3: Requirements ───────────────────────────────────────────────
    println!("## 3. Requirements");
    println!();

    // 3.1 Parent vs Leaf table
    println!("### 3.1 Parent vs Leaf");
    println!();
    println!("| ID | Title | Kind | Status | reqDomain | SIL | ASIL |");
    println!("|---|---|---|---|---|---|---|");
    let mut sorted_reqs = native_reqs.clone();
    sorted_reqs.sort_by_key(|e| e.frontmatter.id.as_deref().unwrap_or(""));
    for e in &sorted_reqs {
        let id = e.frontmatter.id.as_deref().unwrap_or("—");
        let title = e.frontmatter.name.as_deref().unwrap_or("—");
        let kind = if parent_ids.contains(id) { "Parent" } else { "Leaf" };
        let status = e.frontmatter.status.as_deref().unwrap_or("—");
        let req_domain = e.frontmatter.req_domain.as_deref().unwrap_or("—");
        let sil = e
            .frontmatter
            .sil_level
            .map(|v| v.to_string())
            .unwrap_or_else(|| "—".to_string());
        let asil = e.frontmatter.asil_level.as_deref().unwrap_or("—");
        println!("| {} | {} | {} | {} | {} | {} | {} |", id, title, kind, status, req_domain, sil, asil);
    }
    println!();

    // 3.2 Status progression
    println!("### 3.2 Status Progression");
    println!();
    let statuses = ["draft", "review", "approved", "implemented", "verified"];
    println!("| Status | Count | IDs |");
    println!("|---|---|---|");
    for s in &statuses {
        let ids: Vec<&str> = native_reqs
            .iter()
            .filter(|e| e.frontmatter.status.as_deref() == Some(s))
            .filter_map(|e| e.frontmatter.id.as_deref())
            .collect();
        let count = ids.len();
        let ids_str = if ids.is_empty() { "—".to_string() } else { ids.join(", ") };
        println!("| {} | {} | {} |", s, count, ids_str);
    }
    println!();

    // 3.3 Domain distribution
    println!("### 3.3 Domain Distribution");
    println!();
    println!("| reqDomain | Count | Requirement IDs |");
    println!("|---|---|---|");
    let domain_keys = ["system", "hardware", "software"];
    for d in &domain_keys {
        let ids: Vec<&str> = native_reqs
            .iter()
            .filter(|e| e.frontmatter.req_domain.as_deref() == Some(d))
            .filter_map(|e| e.frontmatter.id.as_deref())
            .collect();
        if !ids.is_empty() {
            println!("| {} | {} | {} |", d, ids.len(), ids.join(", "));
        }
    }
    // Row for no reqDomain
    let no_domain_ids: Vec<&str> = native_reqs
        .iter()
        .filter(|e| e.frontmatter.req_domain.is_none())
        .filter_map(|e| e.frontmatter.id.as_deref())
        .collect();
    if !no_domain_ids.is_empty() {
        println!("| (none) | {} | {} |", no_domain_ids.len(), no_domain_ids.join(", "));
    }
    println!();

    // 3.4 SIL/ASIL summary
    println!("### 3.4 SIL/ASIL Summary");
    println!();
    println!("| ID | SIL | ASIL |");
    println!("|---|---|---|");
    for e in &sorted_reqs {
        let id = e.frontmatter.id.as_deref().unwrap_or("—");
        let sil = e
            .frontmatter
            .sil_level
            .map(|v| v.to_string())
            .unwrap_or_else(|| "—".to_string());
        let asil = e.frontmatter.asil_level.as_deref().unwrap_or("—");
        println!("| {} | {} | {} |", id, sil, asil);
    }
    println!();

    // 3.5 Derivation tree
    println!("### 3.5 Derivation Tree");
    println!();
    println!("```");
    // Collect top-level parents (requirements that have children but no derivedFrom themselves)
    let mut top_parents: Vec<&str> = result
        .derived_children
        .keys()
        .filter(|pid| {
            // A top-level parent has no derivedFrom of its own
            native_reqs
                .iter()
                .find(|e| e.frontmatter.id.as_deref() == Some(pid.as_str()))
                .map(|e| e.frontmatter.derived_from.as_ref().map_or(true, |v| v.is_empty()))
                .unwrap_or(true)
        })
        .map(|s| s.as_str())
        .collect();
    top_parents.sort();

    fn print_tree(
        pid: &str,
        derived_children: &HashMap<String, Vec<String>>,
        indent: &str,
        native_reqs: &[&RawElement],
    ) {
        let empty = Vec::new();
        let mut children: Vec<&str> = derived_children
            .get(pid)
            .unwrap_or(&empty)
            .iter()
            .map(|s| s.as_str())
            .collect();
        children.sort();
        let last_idx = if children.is_empty() { 0 } else { children.len() - 1 };
        for (i, cid) in children.iter().enumerate() {
            let connector = if i == last_idx { "└──" } else { "├──" };
            // Find breakdownAdr for this child
            let breakdown = native_reqs
                .iter()
                .find(|e| e.frontmatter.id.as_deref() == Some(cid))
                .and_then(|e| e.frontmatter.breakdown_adr.as_deref())
                .unwrap_or("—");
            println!("{}  {} {}  (breakdownAdr: {})", indent, connector, cid, breakdown);
            // Recurse
            let child_indent = format!("{}  {}", indent, if i == last_idx { " " } else { "|" });
            print_tree(cid, derived_children, &child_indent, native_reqs);
        }
    }

    for pid in &top_parents {
        println!("{}", pid);
        print_tree(pid, &result.derived_children, "", &native_reqs);
    }
    println!("```");
    println!();

    println!("---");
    println!();

    // ── Section 4: Traceability Matrix ────────────────────────────────────────
    println!("## 4. Traceability Matrix");
    println!();

    let mut sorted_req_ids: Vec<&str> = native_reqs
        .iter()
        .filter_map(|e| e.frontmatter.id.as_deref())
        .collect();
    sorted_req_ids.sort();

    let mut sorted_tc_ids: Vec<&str> = native_tcs
        .iter()
        .filter_map(|e| e.frontmatter.id.as_deref())
        .collect();
    sorted_tc_ids.sort();

    if sorted_req_ids.is_empty() || sorted_tc_ids.is_empty() {
        println!("No requirements or test cases found.");
    } else {
        // Header row
        print!("| Requirement |");
        for tc_id in &sorted_tc_ids {
            print!(" {} |", tc_id);
        }
        println!(" Active TCs |");

        // Separator
        print!("|---|");
        for _ in &sorted_tc_ids {
            print!("---|");
        }
        println!("---|");

        // Data rows
        for req_id in &sorted_req_ids {
            print!("| {} |", req_id);
            let covering_tcs = result.verified_by.get(*req_id);
            let mut active_count = 0usize;
            for tc_id in &sorted_tc_ids {
                let covers = covering_tcs
                    .map(|tcs| tcs.iter().any(|t| t == tc_id))
                    .unwrap_or(false);
                if covers {
                    active_count += 1;
                    print!(" \u{2713} |");
                } else {
                    print!("  |");
                }
            }
            println!(" {} |", active_count);
        }
    }
    println!();

    println!("---");
    println!();

    // ── Section 5: Test Cases ─────────────────────────────────────────────────
    println!("## 5. Test Cases");
    println!();
    println!("| ID | Level | Gherkin Scenarios | Status | Verifies |");
    println!("|---|---|---|---|---|");
    let mut sorted_tcs = native_tcs.clone();
    sorted_tcs.sort_by_key(|e| e.frontmatter.id.as_deref().unwrap_or(""));
    for e in &sorted_tcs {
        let id = e.frontmatter.id.as_deref().unwrap_or("—");
        let level = e.frontmatter.test_level.as_deref().unwrap_or("—");
        let scenarios = count_gherkin_scenarios(&e.doc);
        let status = e.frontmatter.status.as_deref().unwrap_or("—");
        let verifies = e
            .frontmatter
            .verifies
            .as_ref()
            .map(|v| v.join(", "))
            .unwrap_or_else(|| "—".to_string());
        println!("| {} | {} | {} | {} | {} |", id, level, scenarios, status, verifies);
    }
    println!();

    // Level summary lines
    let l2_count = native_tcs
        .iter()
        .filter(|e| e.frontmatter.test_level.as_deref() == Some("L2"))
        .count();
    let l5_count = native_tcs
        .iter()
        .filter(|e| e.frontmatter.test_level.as_deref() == Some("L5"))
        .count();
    println!("L2 (analysis/review): {} test cases", l2_count);
    println!();
    println!("L5 (physical/HIL test): {} test cases", l5_count);
    println!();

    println!("---");
    println!();

    // ── Section 6: Architecture Decision Records ──────────────────────────────
    println!("## 6. Architecture Decision Records");
    println!();
    let mut sorted_adrs = native_adrs.clone();
    sorted_adrs.sort_by_key(|e| e.frontmatter.id.as_deref().unwrap_or(""));
    if sorted_adrs.is_empty() {
        println!("No ADRs found.");
    } else {
        println!("| ID | Status | Title |");
        println!("|---|---|---|");
        for e in &sorted_adrs {
            let id = e.frontmatter.id.as_deref().unwrap_or("—");
            let status = e.frontmatter.status.as_deref().unwrap_or("—");
            let title = e.frontmatter.name.as_deref().unwrap_or("—");
            println!("| {} | {} | {} |", id, status, title);
        }
    }
    println!();

    println!("---");
    println!();

    // ── Section 7: Satisfaction Links ─────────────────────────────────────────
    println!("## 7. Satisfaction Links");
    println!();

    // 7.1 Elements with satisfies
    println!("### 7.1 Elements with `satisfies`");
    println!();
    let with_satisfies: Vec<&RawElement> = elems
        .iter()
        .filter(|e| e.frontmatter.satisfies.as_ref().map_or(false, |v| !v.is_empty()))
        .collect();
    if with_satisfies.is_empty() {
        println!("None.");
    } else {
        println!("| Qualified Name | Domain | Satisfies |");
        println!("|---|---|---|");
        let mut sorted_ws = with_satisfies.clone();
        sorted_ws.sort_by_key(|e| e.qualified_name.as_str());
        for e in sorted_ws {
            let qn = &e.qualified_name;
            let domain = e.frontmatter.domain.as_deref().unwrap_or("—");
            let req_satisfies: Vec<&str> = e
                .frontmatter
                .satisfies
                .as_ref()
                .unwrap()
                .iter()
                .filter(|s| is_req_id(s))
                .map(|s| s.as_str())
                .collect();
            let sat_str = if req_satisfies.is_empty() {
                "—".to_string()
            } else {
                req_satisfies.join(", ")
            };
            println!("| {} | {} | {} |", linked_label(qn, e, &vcfg), domain, sat_str);
        }
    }
    println!();

    // 7.2 Elements without satisfies
    // Only flag elements with `domain` set but no `satisfies` — these represent a meaningful
    // gap where the element has been deliberately classified but has no requirement allocation.
    // Structural sub-components (Motor, Rotor, IMU, …) that carry no domain are listed
    // separately as an informational count, not as gaps.
    println!("### 7.2 Architecture Elements without `satisfies`");
    println!();
    let domain_no_satisfies: Vec<&RawElement> = elems
        .iter()
        .filter(|e| {
            is_arch_element(e)
                && e.frontmatter.is_abstract != Some(true)
                && e.frontmatter.domain.is_some()
                && e.frontmatter.satisfies.as_ref().map_or(true, |v| v.is_empty())
        })
        .collect();
    if domain_no_satisfies.is_empty() {
        println!("> All domain-classified architecture elements have at least one satisfaction link.");
    } else {
        println!("The following elements have `domain` set but no `satisfies` — requirement allocation is missing:");
        println!();
        println!("| Qualified Name | Domain |");
        println!("|---|---|");
        let mut sorted_dns = domain_no_satisfies.clone();
        sorted_dns.sort_by_key(|e| e.qualified_name.as_str());
        for e in sorted_dns {
            let domain = e.frontmatter.domain.as_deref().unwrap_or("—");
            println!("| {} | {} |", linked_label(&e.qualified_name, e, &vcfg), domain);
        }
    }
    println!();
    // Informational: count structural sub-components (no domain, no satisfies)
    let structural_count = elems
        .iter()
        .filter(|e| {
            is_arch_element(e)
                && e.frontmatter.is_abstract != Some(true)
                && e.frontmatter.domain.is_none()
                && e.frontmatter.satisfies.as_ref().map_or(true, |v| v.is_empty())
        })
        .count();
    if structural_count > 0 {
        println!(
            "> {} structural sub-component(s) carry no `domain` or `satisfies` — this is expected for \
leaf hardware parts (Motor, Rotor, IMU, etc.) that are not directly allocated a requirement.",
            structural_count
        );
        println!();
    }

    println!("---");
    println!();

    // ── Section 8: Allocation Summary ─────────────────────────────────────────
    println!("## 8. Allocation Summary");
    println!();

    let alloc_elems: Vec<&RawElement> = elems
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Allocation)))
        .collect();

    if alloc_elems.is_empty() {
        println!("No Allocation elements found.");
    } else {
        // Collect rows: (group_name, from, to)
        let mut rows: Vec<(String, String, String)> = Vec::new();
        for alloc in &alloc_elems {
            let group_name = alloc
                .frontmatter
                .name
                .clone()
                .unwrap_or_else(|| alloc.qualified_name.clone());

            // Prefer top-level allocated_from / allocated_to
            if alloc.frontmatter.allocated_from.is_some() || alloc.frontmatter.allocated_to.is_some() {
                let from = alloc.frontmatter.allocated_from.as_ref()
                    .map(|v| v.join(", ")).unwrap_or_else(|| "—".into());
                let to = alloc.frontmatter.allocated_to.as_ref()
                    .map(|v| v.join(", ")).unwrap_or_else(|| "—".into());
                rows.push((group_name, from, to));
            } else if let Some(ref features) = alloc.frontmatter.features {
                // Look for allocatedFrom / allocatedTo in inline feature maps
                for feat in features {
                    if let serde_yaml::Value::Mapping(map) = feat {
                        let from = map
                            .get(&serde_yaml::Value::String("allocatedFrom".into()))
                            .and_then(|v| v.as_str())
                            .unwrap_or("—")
                            .to_string();
                        let to = map
                            .get(&serde_yaml::Value::String("allocatedTo".into()))
                            .and_then(|v| v.as_str())
                            .unwrap_or("—")
                            .to_string();
                        rows.push((group_name.clone(), from, to));
                    }
                }
                if features.is_empty() {
                    rows.push((group_name, "—".to_string(), "—".to_string()));
                }
            } else {
                rows.push((group_name, "—".to_string(), "—".to_string()));
            }
        }

        // Group by allocation element name
        let mut grouped: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
        for (group, from, to) in rows {
            grouped.entry(group).or_default().push((from, to));
        }

        for (group, pairs) in &grouped {
            println!("**{}**", group);
            println!();
            println!("| From | To |");
            println!("|---|---|");
            for (from, to) in pairs {
                println!("| {} | {} |", from, to);
            }
            println!();
        }
    }

    println!("---");
    println!();

    // ── Section 9: Open Gaps ──────────────────────────────────────────────────
    println!("## 9. Open Gaps");
    println!();

    // Requirements still at approved
    let approved_ids: Vec<&str> = native_reqs
        .iter()
        .filter(|e| e.frontmatter.status.as_deref() == Some("approved"))
        .filter_map(|e| e.frontmatter.id.as_deref())
        .collect();
    if approved_ids.is_empty() {
        println!("- No requirements remain at `approved` status.");
    } else {
        println!(
            "- Requirements still at `approved` (none have advanced to `implemented` or `verified`): {}",
            approved_ids.join(", ")
        );
    }

    // Leaf requirements with no SIL/ASIL
    let no_sil_ids: Vec<&str> = native_reqs
        .iter()
        .filter(|e| {
            let id = e.frontmatter.id.as_deref().unwrap_or("");
            !parent_ids.contains(id) // leaf only
                && e.frontmatter.sil_level.is_none()
                && e.frontmatter.asil_level.is_none()
        })
        .filter_map(|e| e.frontmatter.id.as_deref())
        .collect();
    if no_sil_ids.is_empty() {
        println!("- All leaf requirements have SIL/ASIL classification.");
    } else {
        println!(
            "- Leaf requirements with no SIL/ASIL classification: {}",
            no_sil_ids.join(", ")
        );
    }

    // Non-abstract PartDef/Part with no satisfies and no domain
    let no_sat_no_domain: Vec<&str> = elems
        .iter()
        .filter(|e| {
            is_arch_element(e)
                && e.frontmatter.is_abstract != Some(true)
                && e.frontmatter.satisfies.as_ref().map_or(true, |v| v.is_empty())
                && e.frontmatter.domain.is_none()
        })
        .map(|e| e.qualified_name.as_str())
        .collect();
    if no_sat_no_domain.is_empty() {
        println!("- All non-abstract PartDef/Part elements have either `satisfies` or `domain` set.");
    } else {
        println!(
            "- Architecture elements (non-abstract PartDef/Part) with no `satisfies` and no `domain`: {}",
            no_sat_no_domain.join(", ")
        );
    }
    println!();

    println!("---");
    println!();

    // ── Section 10: Element Inventory by Package ──────────────────────────────
    println!("## 10. Element Inventory by Package");
    println!();

    // For each top-level package, count elements by type
    let mut pkg_map: BTreeMap<String, HashMap<String, usize>> = BTreeMap::new();
    for e in &elems {
        let pkg = top_level_package(&e.file_path, &model_root_str);
        let type_str = e
            .frontmatter
            .element_type
            .as_ref()
            .map(query::type_label)
            .unwrap_or("Unknown")
            .to_string();
        *pkg_map.entry(pkg).or_default().entry(type_str).or_insert(0) += 1;
    }

    println!("| Package | Total | Top Element Types |");
    println!("|---|---|---|");
    for (pkg, type_counts) in &pkg_map {
        let total: usize = type_counts.values().sum();
        // Sort by count descending, take top 3
        let mut types: Vec<(&String, &usize)> = type_counts.iter().collect();
        types.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        let top3: Vec<String> = types
            .iter()
            .take(3)
            .map(|(t, c)| format!("{}x{}", t, c))
            .collect();
        println!("| {} | {} | {} |", pkg, total, top3.join(", "));
    }
    println!();

    if error_count > 0 {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod cli_router_tests {
    use super::*;

    /// The clap registry is structurally valid and stays consistent with the embedded
    /// help pages: every command with a man page is a registered subcommand
    /// (REQ-TRS-CLI-008).
    #[test]
    fn clap_registry_matches_help_pages() {
        build_cli().debug_assert();
        let cli = build_cli();
        for (name, _) in help::commands() {
            assert!(
                cli.find_subcommand(name).is_some(),
                "command '{name}' has a man page but is not registered in build_cli()"
            );
        }
    }
}
