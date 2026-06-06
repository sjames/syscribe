#![deny(warnings)]

mod diagram;
mod export;
mod ingest;
mod matrix;
mod mv;
mod query;
mod render;
mod scaffold;
mod spec;

use std::collections::{BTreeMap, HashMap};
use syscribe_model::{
    config::ValidateConfig,
    element::{ElementType, RawElement},
    resolver::{is_adr_id, is_req_id, is_tc_id, Resolver},
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

/// Count `Scenario:` lines across the doc body.
fn count_gherkin_scenarios(doc: &str) -> usize {
    doc.lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("Scenario:") || t.starts_with("Scenario Outline:")
        })
        .count()
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

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--agent-instructions") {
        print!("{}", AGENT_INSTRUCTIONS);
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

    // Priority: --model flag > SYSCRIBE_MODEL env var > "model" default.
    let model_root_arg = model_flag
        .or_else(|| std::env::var("SYSCRIBE_MODEL").ok())
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
    if let Some(subcmd) = subcommand_args.first().map(|s| s.as_str()) {
        let resolver = Resolver::new(&elems);
        // subcommand_args[0] = subcommand, subcommand_args[1] = key, subcommand_args[2] = scope, …
        let key = subcommand_args.get(1).map(|s| s.as_str()).unwrap_or("");
        match subcmd {
            "show" => {
                query::cmd_show(&elems, &resolver, key);
            }
            "ls" => {
                query::cmd_ls(&elems, key);
            }
            "tree" => {
                query::cmd_tree(&elems, key);
            }
            "find" => {
                if key.is_empty() {
                    eprintln!("Usage: syscribe --model <root> find <pattern>");
                    std::process::exit(1);
                }
                query::cmd_find(&elems, key);
            }
            "links" => {
                query::cmd_links(&elems, &resolver, key);
            }
            "refs" => {
                query::cmd_refs(&elems, &resolver, key);
            }
            "render" => {
                if key.is_empty() {
                    eprintln!("Usage: syscribe --model <root> render <diagram_path>");
                    std::process::exit(1);
                }
                render::cmd_render(&elems, &resolver, key);
            }
            "diagram" => {
                let sub = subcommand_args.get(1).map(|s| s.as_str()).unwrap_or("");
                let rest: Vec<String> = subcommand_args.get(2..).unwrap_or(&[]).to_vec();
                diagram::cmd_diagram(&elems, &resolver, sub, &rest);
            }
            "validate" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let file_filter = rest.windows(2)
                    .find(|w| w[0] == "--file")
                    .map(|w| w[1].as_str());
                let gate = parse_gate_options(rest);
                // Ad-hoc results ingest for this run (does not write the sidecar).
                let results_file = rest.windows(2)
                    .find(|w| w[0] == "--results")
                    .map(|w| w[1].as_str());
                let mut vcfg_run = vcfg.clone();
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
                query::cmd_validate(&elems, &vcfg_run, &gate, file_filter, json);
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
                export::cmd_export(&elems, &vcfg, ndjson);
            }
            "types" => {
                query::cmd_types(&elems);
            }
            "untyped" => {
                query::cmd_untyped(&elems);
            }
            "list" => {
                if key.is_empty() {
                    eprintln!("Usage: syscribe --model <root> list <type> [scope] [--tag <tag>]");
                    std::process::exit(1);
                }
                let rest = subcommand_args.get(2..).unwrap_or(&[]);
                let tag = rest
                    .windows(2)
                    .find(|w| w[0] == "--tag")
                    .map(|w| w[1].as_str());
                // scope = first positional argument that is not a flag or flag value
                let mut scope = "";
                let mut i = 0;
                while i < rest.len() {
                    if rest[i] == "--tag" {
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
                query::cmd_list(&elems, key, scope, tag);
            }
            "matrix" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                let tag = rest
                    .windows(2)
                    .find(|w| w[0] == "--tag")
                    .map(|w| w[1].as_str());
                matrix::cmd_matrix(&elems, json, tag);
            }
            "feature-check" => {
                let rest = subcommand_args.get(1..).unwrap_or(&[]);
                let json = rest.iter().any(|a| a == "--json");
                query::cmd_feature_check(&elems, json);
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
            "trace" | "why" | "who-verifies" => {
                let result = validator::validate_with_config(&elems, &vcfg);
                match subcmd {
                    "trace" => query::cmd_trace(&elems, &resolver, &result, key),
                    "why" => query::cmd_why(&elems, &resolver, &result, key),
                    "who-verifies" => query::cmd_who_verifies(&elems, &resolver, &result, key),
                    _ => unreachable!(),
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
        let title = e.frontmatter.title.as_deref().unwrap_or("—");
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
            let title = e.frontmatter.title.as_deref().unwrap_or("—");
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
            println!("| {} | {} | {} |", qn, domain, sat_str);
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
            println!("| {} | {} |", e.qualified_name, domain);
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
