//! The machine contract for front ends (GH #413, #414; `REQ-TRS-CONTRACT-001`, `REQ-TRS-CAPS-001`).
//!
//! * [`SCHEMAS`] — every `--json` form a front end depends on, each with a published JSON Schema (embedded from
//!   `crates/syscribe/schemas/`) and a `schemaVersion`. `syscribe schema [<name>]` prints them.
//! * [`capabilities`] — what this binary supports, in one cheap call: `syscribe capabilities --json`, and the
//!   `experimental.syscribe` field of the LSP `initialize` result (same document).
//!
//! A compatible change adds an optional property; removing or retyping one, or changing its meaning, bumps the
//! schema's `schemaVersion` and is recorded in `docs/releases/json-contract.md`.

use serde_json::{json, Value};

/// A published JSON contract.
pub struct Contract {
    /// The name `syscribe schema <name>` takes.
    pub name: &'static str,
    /// The command (and flags) that emits it.
    pub command: &'static str,
    /// Current `schemaVersion` value.
    pub version: &'static str,
    pub schema: &'static str,
    /// `cli` or `lsp` (a notification a front end receives).
    pub transport: &'static str,
}

macro_rules! c {
    ($n:literal, $cmd:literal) => {
        Contract { name: $n, command: $cmd, version: "1.0", schema: include_str!(concat!("../schemas/", $n, ".schema.json")), transport: "cli" }
    };
    ($n:literal, $cmd:literal, lsp) => {
        Contract { name: $n, command: $cmd, version: "1.0", schema: include_str!(concat!("../schemas/", $n, ".schema.json")), transport: "lsp" }
    };
}

pub const SCHEMAS: &[Contract] = &[
    c!("features", "features --json"),
    c!("feature", "feature <name> --json"),
    c!("configure", "configure <Configuration> --json"),
    c!("residual", "residual --config <Configuration> --json"),
    c!("matrix", "matrix --json"),
    c!("diff", "diff --config <A> --config <B> --json"),
    c!("feature-check", "feature-check [--deep] --json"),
    c!("params-check", "params check <FILE> --config <Configuration> --json"),
    c!("propagate", "propagate [--from <Configuration>] [--select <f>=<bool>] --json"),
    c!("scaffold", "new feature|configuration|parameter ... --json"),
    c!("bind", "bind <Configuration> <feature | feature.parameter> <value> --json"),
    c!("timeline", "timeline <Configuration> --json"),
    c!("change-impact", "change-impact <element> --set <field>=<yaml> --json"),
    c!("health", "health --json"),
    c!("children", "children [<node>] [--view containment|configurations] --json"),
    c!("explain-feature", "explain-feature <feature> [--from <Configuration>] --json"),
    c!("model-changed", "syscribe/modelChanged", lsp),
];

/// Optional capabilities and the commands each one needs, so a front end can ask "can I show the configurator?"
/// rather than infer it from a version number.
const OPTIONAL_FEATURES: &[(&str, &[&str])] = &[
    ("featureModel", &["features", "feature", "feature-check"]),
    ("assistedConfiguration", &["configure"]),
    ("interactiveConfiguration", &["propagate"]),
    ("featureExplanation", &["explain-feature"]),
    ("treeChildren", &["children"]),
    ("projectHealth", &["health"]),
    ("changeImpact", &["change-impact"]),
    ("stagedBinding", &["bind", "timeline"]),
    ("scaffolding", &["new"]),
    ("stagedConfiguration", &["residual", "params", "binding-times"]),
    ("paramsExport", &["params"]),
    ("variantDiff", &["diff"]),
    ("coverageMatrix", &["matrix"]),
    ("environmentMatrix", &["matrix --by"]),
    ("briefingContext", &["context"]),
    ("provenance", &["log"]),
];

/// The capabilities document. `lsp_requests` and `mcp_tools` are passed in so the document is the same wherever it
/// is asked for.
pub fn capabilities(mcp_tools: &[String]) -> Value {
    let commands: Vec<&str> = crate::help::commands().map(|(n, _)| n).collect();
    // A need is a command, or `command --flag` when the capability is an option of one.
    let cmd = |n: &&str| match n.split_once(' ') {
        None => commands.contains(n),
        Some((c, flag)) => crate::cliargs::options_of(c).is_some_and(|o| o.contains(&flag)),
    };
    let mut optional = serde_json::Map::new();
    for (name, needs) in OPTIONAL_FEATURES {
        optional.insert((*name).to_string(), json!(needs.iter().all(cmd)));
    }
    let mut cmds = Vec::new();
    for n in &commands {
        let options = crate::cliargs::options_of(n);
        cmds.push(json!({"name": n, "options": options}));
    }
    let schemas: serde_json::Map<String, Value> = SCHEMAS
        .iter()
        .map(|c| (c.name.to_string(), json!({"version": c.version, "command": c.command, "transport": c.transport, "available": c.transport == "lsp" || c.command.split_whitespace().next().is_some_and(|w| commands.contains(&w))})))
        .collect();
    json!({
        "schemaVersion": "1.0",
        "name": "syscribe",
        "version": env!("CARGO_PKG_VERSION"),
        "commands": cmds,
        "jsonSchemas": schemas,
        "features": optional,
        "lspRequests": crate::lsp::CUSTOM_REQUESTS,
        "lspNotifications": crate::lsp::CUSTOM_NOTIFICATIONS,
        "mcpTools": mcp_tools,
    })
}

const SCHEMA_USAGE: &str = "Usage: syscribe schema [<name>]    (no name lists the published contracts)";

/// `syscribe schema [<name>]`. Returns the exit code.
pub fn cmd_schema(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        None => {
            println!("# Published JSON contracts\n\n| Name | Version | Emitted by |\n|---|---|---|");
            for c in SCHEMAS {
                if c.transport == "lsp" {
                    println!("| {} | {} | LSP notification `{}` |", c.name, c.version, c.command);
                } else {
                    println!("| {} | {} | `syscribe {}` |", c.name, c.version, c.command);
                }
            }
            println!("\nPrint one with `syscribe schema <name>`.");
            0
        }
        Some(n) if n.starts_with('-') => {
            eprintln!("{SCHEMA_USAGE}");
            2
        }
        Some(n) => match SCHEMAS.iter().find(|c| c.name == n) {
            Some(c) => {
                print!("{}", c.schema);
                0
            }
            None => {
                eprintln!("schema: unknown contract '{n}'.{} Published: {}", crate::suggest::did_you_mean(n, SCHEMAS.iter().map(|c| c.name)), SCHEMAS.iter().map(|c| c.name).collect::<Vec<_>>().join(", "));
                1
            }
        },
    }
}

/// `syscribe capabilities [--json]`. Returns the exit code.
pub fn cmd_capabilities(args: &[String]) -> i32 {
    let doc = capabilities(&crate::mcp::tool_names());
    if args.iter().any(|a| a == "--json") {
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
        return 0;
    }
    println!("# syscribe {}\n", doc["version"].as_str().unwrap_or(""));
    println!("## Features\n");
    for (k, v) in doc["features"].as_object().unwrap() {
        println!("- {k}: {}", if v.as_bool() == Some(true) { "yes" } else { "no" });
    }
    println!("\n## JSON contracts\n");
    for (k, v) in doc["jsonSchemas"].as_object().unwrap() {
        println!("- {k} {} — `syscribe {}`", v["version"].as_str().unwrap_or(""), v["command"].as_str().unwrap_or(""));
    }
    println!("\n{} commands, {} MCP tools, {} LSP custom requests. Use `--json` for the full document.", doc["commands"].as_array().unwrap().len(), doc["mcpTools"].as_array().unwrap().len(), doc["lspRequests"].as_array().unwrap().len());
    0
}
