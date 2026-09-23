//! `follow` and `link-types` — user-defined link types from the command line
//! (ADR-SYS-LINKTYPE-001; REQ-TRS-LINKTYPE-007/008). Both are read-only; the
//! traversal and JSON shapes live in `syscribe_model::link_types` so the MCP
//! `follow`/`link_types` tools return exactly the same data.

use syscribe_model::{
    config::ValidateConfig,
    element::RawElement,
    link_types::{self, LinkTypeRegistry},
    resolver::Resolver,
};

/// REQ-TRS-LINKTYPE-007 — `follow <elem> <link> [--reverse] [--transitive]
/// [--depth N] [--format text|json|dot]`. Returns the process exit code: 0 on
/// success, 1 for an unknown element, 2 for a usage error or an unknown link
/// (which also prints the available link names).
pub fn cmd_follow(elements: &[RawElement], resolver: &Resolver, vcfg: &ValidateConfig, args: &[String]) -> i32 {
    let usage = "Usage: syscribe -m <root> follow <elem> <link> [--reverse] [--transitive] [--depth N] [--format text|json|dot]";
    let mut positionals: Vec<&str> = Vec::new();
    let mut reverse = false;
    let mut transitive = false;
    let mut depth: Option<usize> = None;
    let mut format = "text";
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--reverse" => reverse = true,
            "--transitive" => transitive = true,
            "--depth" => {
                match args.get(i + 1).and_then(|v| v.parse::<usize>().ok()) {
                    Some(n) => depth = Some(n),
                    None => {
                        eprintln!("follow: --depth expects a non-negative integer");
                        return 2;
                    }
                }
                i += 1;
            }
            "--format" => {
                match args.get(i + 1).map(String::as_str) {
                    Some(f @ ("text" | "json" | "dot")) => format = f,
                    other => {
                        eprintln!("follow: unknown --format '{}' (text|json|dot)", other.unwrap_or(""));
                        return 2;
                    }
                }
                i += 1;
            }
            "--json" => format = "json",
            other if other.starts_with("--") => {
                eprintln!("follow: unknown option '{other}'");
                eprintln!("{usage}");
                return 2;
            }
            other => positionals.push(other),
        }
        i += 1;
    }
    let (Some(elem_ref), Some(link)) = (positionals.first(), positionals.get(1)) else {
        eprintln!("{usage}");
        return 2;
    };

    let reg = &vcfg.link_types;
    let Some(start) = syscribe_model::suspect::resolve_target(elements, resolver, elem_ref) else {
        eprintln!("follow: element '{elem_ref}' does not resolve to a known element");
        return 1;
    };
    // One hop by default; --transitive to a fixed point; --depth N bounds it
    // (and implies transitive).
    let max_depth = match (depth, transitive) {
        (Some(n), _) => Some(n),
        (None, true) => None,
        (None, false) => Some(1),
    };
    let result = match link_types::follow(elements, resolver, reg, start, link, reverse, max_depth) {
        Ok(r) => r,
        Err(msg) => {
            eprintln!("follow: {msg}");
            return 2;
        }
    };

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&link_types::follow_json(&result)).unwrap_or_default());
        }
        "dot" => {
            let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
            println!("digraph follow {{");
            println!("  rankdir=LR; node [shape=box, style=rounded];");
            println!("  \"{}\" [style=\"rounded,filled\", fillcolor=\"#cde\"];", esc(&result.start));
            let arrow_label = if result.direction == "reverse" && reg.get(link).is_some() {
                format!("{} (reverse)", link)
            } else {
                link.to_string()
            };
            for (from, to) in &result.edges {
                println!("  \"{}\" -> \"{}\" [label=\"{}\"];", esc(from), esc(to), esc(&arrow_label));
            }
            println!("}}");
        }
        _ => {
            let mode = match max_depth {
                Some(1) => "one hop".to_string(),
                Some(n) => format!("up to {n} hops"),
                None => "transitive".to_string(),
            };
            println!("# Follow: {} —{}→ ({}, {})", result.start, result.link, result.direction, mode);
            println!();
            if result.hits.is_empty() {
                println!("_(no elements reached)_");
                return 0;
            }
            println!("| Depth | Element | Type | Name | From |");
            println!("|---|---|---|---|---|");
            for h in &result.hits {
                let shown = h.id.clone().unwrap_or_else(|| h.qname.clone());
                println!(
                    "| {} | {} | {} | {} | {} |",
                    h.depth,
                    shown,
                    h.type_name,
                    h.name.as_deref().unwrap_or("—"),
                    h.from
                );
            }
            println!();
            println!("{} element(s) reached", result.hits.len());
        }
    }
    0
}

/// REQ-TRS-LINKTYPE-008 — `link-types [--json]`: the declared vocabulary with
/// every rule and its instance count. With none declared, a hint on how to
/// declare one (exit 0 either way).
pub fn cmd_link_types(elements: &[RawElement], reg: &LinkTypeRegistry, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(&link_types::link_types_json(elements, reg)).unwrap_or_default());
        return;
    }
    if reg.is_empty() {
        println!("No link types declared.");
        println!();
        if reg.is_declared() {
            println!("The [linkTypes] table in .syscribe.toml has no valid entry — run `validate` and fix the W630 findings.");
        } else {
            println!("Declare one with a `[linkTypes.<name>]` table in <model_root>/.syscribe.toml, e.g.:");
            println!();
            println!("    [linkTypes.mitigates]");
            println!("    description = \"A control mitigates a hazard\"");
            println!("    inverse = \"mitigatedBy\"");
            println!("    sourceTypes = [\"PartDef\"]");
            println!("    targetTypes = [\"Requirement\"]");
            println!();
            println!("then author instances on the source element:  links: {{ mitigates: [REQ-001] }}");
        }
        return;
    }
    let counts = link_types::instance_counts(elements, reg);
    println!("# Link types");
    println!();
    println!("| Name | Inverse | Source → Target | Cardinality | Acyclic | Suspect | Extends | Relax | Coverage | Count |");
    println!("|---|---|---|---|---|---|---|---|---|---|");
    for t in reg.types() {
        let src = t.source_types.as_ref().map_or("any".to_string(), |v| v.join(", "));
        let tgt = t.target_types.as_ref().map_or("any".to_string(), |v| v.join(", "));
        let (ext, relax, cov) = match t.extends {
            Some(b) => (
                b.name().to_string(),
                if t.relax.is_empty() { "—".to_string() } else { t.relax.join(", ") },
                t.coverage.to_string(),
            ),
            None => ("—".to_string(), "—".to_string(), "—".to_string()),
        };
        println!(
            "| {} | {} | {} → {} | {} | {} | {} | {} | {} | {} | {} |",
            t.name,
            t.inverse.as_deref().unwrap_or("—"),
            src,
            tgt,
            t.cardinality,
            t.acyclic,
            t.suspect,
            ext,
            relax,
            cov,
            counts.get(&t.name).copied().unwrap_or(0)
        );
    }
    let described: Vec<_> = reg.types().iter().filter(|t| t.description.is_some()).collect();
    if !described.is_empty() {
        println!();
        for t in described {
            println!("- **{}** — {}", t.name, t.description.as_deref().unwrap_or(""));
        }
    }
}
