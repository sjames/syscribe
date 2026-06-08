//! `syscribe -m <root> connectivity <element>` — element-rooted, transitive
//! export of the connectivity subgraph: the elements reachable from a chosen
//! root plus the edges among them, in text / JSON / DOT form (issue #26).
//!
//! Running it on the model-root element dumps the whole model (the default edge
//! set includes `Contains`).
//!
//! Deferred (issue #26 MVP, also noted on `EdgeKind` in graph.rs): rich edge
//! labels — the `via` ConnectionDef/InterfaceDef and `fromEnd`/`toEnd` feature
//! chains. Edges here carry their `kind` only.
//!
//! The graph traversal itself lives in `syscribe_model::graph` so this crate
//! need not depend on `petgraph`; this module is purely CLI parsing + rendering.

use std::collections::{HashMap, HashSet};

use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::graph::{build_graph, connectivity_subgraph, EdgeKind, Subgraph};
use syscribe_model::resolver::Resolver;

use crate::query::type_label;

/// Output format for the connectivity walk.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    Text,
    Json,
    Dot,
}

/// All edge-kind names a user may name in `--kinds` (case-insensitive).
const KIND_NAMES: &[&str] = &[
    "connection",
    "flow",
    "binding",
    "succession",
    "featureTyped",
    "contains",
    "typedBy",
    "supertype",
    "subsets",
    "redefines",
    "satisfies",
    "verifies",
    "derivedFrom",
    "allocatedFrom",
    "allocatedTo",
    "conditionalOn",
];

/// Canonicalise a user-supplied (case-insensitive) kind token to its stable
/// `EdgeKind::name()` form.
fn canon_kind(tok: &str) -> Option<&'static str> {
    let t = tok.trim().to_ascii_lowercase();
    KIND_NAMES
        .iter()
        .copied()
        .find(|name| name.to_ascii_lowercase() == t)
}

/// The default edge kinds to follow: the wiring plus structure, so the model
/// root dumps the whole model via `contains`.
fn default_kinds() -> HashSet<String> {
    [
        "connection",
        "flow",
        "binding",
        "succession",
        "featureTyped",
        "contains",
        "typedBy",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// Parsed CLI options for the `connectivity` command.
struct Opts {
    root: String,
    depth: Option<usize>,
    format: Format,
    kinds: HashSet<String>,
    undirected: bool,
}

fn usage() {
    eprintln!(
        "Usage: syscribe -m <root> connectivity <element> \
         [--depth N] [--format text|dot|json] [--kinds <csv>] [--undirected]"
    );
}

/// Parse `connectivity <element> [flags]`. `args` is everything after the
/// subcommand word. Returns `None` (after printing usage) on a bad invocation.
fn parse_opts(args: &[String]) -> Option<Opts> {
    let mut root: Option<String> = None;
    let mut depth: Option<usize> = None;
    let mut format = Format::Text;
    let mut kinds: Option<HashSet<String>> = None;
    let mut undirected = false;

    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--depth" => {
                let v = args.get(i + 1)?;
                match v.parse::<usize>() {
                    Ok(n) => depth = Some(n),
                    Err(_) => {
                        eprintln!("connectivity: --depth expects a non-negative integer");
                        return None;
                    }
                }
                i += 2;
            }
            "--format" => {
                let v = args.get(i + 1)?;
                format = match v.as_str() {
                    "text" => Format::Text,
                    "json" => Format::Json,
                    "dot" => Format::Dot,
                    other => {
                        eprintln!("connectivity: unknown --format '{other}' (text|dot|json)");
                        return None;
                    }
                };
                i += 2;
            }
            "--json" => {
                // convenience alias, mirrors other commands
                format = Format::Json;
                i += 1;
            }
            "--kinds" => {
                let v = args.get(i + 1)?;
                let mut set = HashSet::new();
                for tok in v.split(',') {
                    if tok.trim().is_empty() {
                        continue;
                    }
                    match canon_kind(tok) {
                        Some(name) => {
                            set.insert(name.to_string());
                        }
                        None => {
                            eprintln!("connectivity: unknown edge kind '{}'", tok.trim());
                            return None;
                        }
                    }
                }
                kinds = Some(set);
                i += 2;
            }
            "--undirected" => {
                undirected = true;
                i += 1;
            }
            other if other.starts_with("--") => {
                eprintln!("connectivity: unknown flag '{other}'");
                return None;
            }
            other => {
                if root.is_none() {
                    root = Some(other.to_string());
                } else {
                    eprintln!("connectivity: unexpected argument '{other}'");
                    return None;
                }
                i += 1;
            }
        }
    }

    let Some(root) = root else {
        usage();
        return None;
    };

    Some(Opts {
        root,
        depth,
        format,
        kinds: kinds.unwrap_or_else(default_kinds),
        undirected,
    })
}

/// Resolve an element's type label by qname, via the resolver.
fn type_str(resolver: &Resolver, elements: &[RawElement], qname: &str) -> String {
    resolver
        .get(elements, qname)
        .and_then(|e| e.frontmatter.element_type.clone())
        .map(|t| type_label(&t).to_string())
        .unwrap_or_else(|| "?".to_string())
}

// ── Text renderer ───────────────────────────────────────────────────────────

/// Render an indented tree from the root using ├──/└──/│ connectors. Each node
/// line is `<qname> [<Type>]`; child lines carry the edge kind. A node already
/// expanded elsewhere is shown but not re-expanded (marked with ` (*)`).
fn render_text(resolver: &Resolver, elements: &[RawElement], sub: &Subgraph) {
    // adjacency among reachable nodes, in edge order.
    let mut adj: HashMap<&str, Vec<(&str, &EdgeKind)>> = HashMap::new();
    for (f, t, k) in &sub.edges {
        adj.entry(f.as_str()).or_default().push((t.as_str(), k));
    }

    println!("{} [{}]", sub.root, type_str(resolver, elements, &sub.root));

    let mut expanded: HashSet<&str> = HashSet::from([sub.root.as_str()]);
    print_children(resolver, elements, &adj, &sub.root, "", &mut expanded);
}

fn print_children<'a>(
    resolver: &Resolver,
    elements: &[RawElement],
    adj: &HashMap<&'a str, Vec<(&'a str, &'a EdgeKind)>>,
    node: &'a str,
    prefix: &str,
    expanded: &mut HashSet<&'a str>,
) {
    let Some(children) = adj.get(node) else { return };
    let n = children.len();
    for (i, (child, kind)) in children.iter().enumerate() {
        let last = i == n - 1;
        let connector = if last { "└── " } else { "├── " };
        let revisit = expanded.contains(child);
        let mark = if revisit { " (*)" } else { "" };
        println!(
            "{prefix}{connector}[{}] {} [{}]{mark}",
            kind.name(),
            child,
            type_str(resolver, elements, child),
        );
        if revisit {
            continue;
        }
        expanded.insert(child);
        let child_prefix = format!("{prefix}{}", if last { "    " } else { "│   " });
        print_children(resolver, elements, adj, child, &child_prefix, expanded);
    }
}

// ── JSON renderer ─────────────────────────────────────────────────────────────

fn render_json(resolver: &Resolver, elements: &[RawElement], sub: &Subgraph) {
    let nodes: Vec<serde_json::Value> = sub
        .nodes
        .iter()
        .map(|qname| {
            let elem = resolver.get(elements, qname);
            let ty = elem
                .and_then(|e| e.frontmatter.element_type.clone())
                .map(|t| type_label(&t).to_string());
            let id = elem.and_then(|e| e.frontmatter.id.clone());
            serde_json::json!({ "qualifiedName": qname, "type": ty, "id": id })
        })
        .collect();

    let edges: Vec<serde_json::Value> = sub
        .edges
        .iter()
        .map(|(f, t, k)| serde_json::json!({ "from": f, "to": t, "kind": k.name() }))
        .collect();

    let doc = serde_json::json!({ "root": sub.root, "nodes": nodes, "edges": edges });
    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
}

// ── DOT renderer + styling (single source of truth) ──────────────────────────

/// Visual style for a node, keyed off its element type. The single source of
/// truth for shape/colour used by the DOT writer and the legend.
struct NodeStyle {
    shape: &'static str,
    /// Extra shape modifier appended to `style=` (e.g. `,rounded`), or "".
    style_extra: &'static str,
    fill: &'static str,
    border: &'static str,
    peripheries: u8,
}

/// Map an element type to its (shape, fill, border, peripheries). Definitions
/// get `peripheries=2`; the matching usage shares the shape with a single
/// border. Colour encodes the domain/concern (pale fill + saturated border).
fn node_style(et: &ElementType) -> NodeStyle {
    use ElementType::*;
    // (shape, style_extra, fill, border)
    let (shape, style_extra, fill, border) = match et {
        // Structure (parts)
        PartDef | Part => ("box", "", "#E8F0FE", "#1A73E8"),
        // Structure (items / occurrences / individuals)
        ItemDef | Item | OccurrenceDef | Occurrence | EventOccurrenceDef | EventOccurrence
        | IndividualDef | Individual => ("box", "", "#E0F7F4", "#00897B"),
        // Ports
        PortDef | Port => ("circle", "", "#E8F0FE", "#1A73E8"),
        // Connections / interfaces
        ConnectionDef | Connection | InterfaceDef | Interface | BindingConnector => {
            ("hexagon", "", "#E1F5FE", "#0277BD")
        }
        // Flow
        FlowDef | Flow => ("cds", "", "#E1F5FE", "#0277BD"),
        // Behaviour
        ActionDef | Action | StateDef | State | ExhibitState | CalculationDef | Calculation
        | ConstraintDef | Constraint | UseCaseDef | UseCase | SuccessionDef | Succession => {
            ("box", ",rounded", "#E6F4EA", "#1E8E3E")
        }
        // Requirements
        Requirement | RequirementDef => ("note", "", "#E8F0FE", "#1A73E8"),
        // Verification
        TestCase | VerificationCaseDef | VerificationCase | AnalysisCaseDef | AnalysisCase => {
            ("note", "", "#E6F4EA", "#1E8E3E")
        }
        // Decisions
        ADR => ("note", "", "#EFEBE9", "#6D4C41"),
        // Variability
        FeatureDef => ("diamond", "", "#F3E5F5", "#8E24AA"),
        Configuration => ("box3d", "", "#F3E5F5", "#8E24AA"),
        // Packaging
        Package | LibraryPackage | Namespace => ("folder", "", "#F5F5F5", "#9E9E9E"),
        // Safety
        SafetyGoal => ("doubleoctagon", "", "#FCE8E6", "#C5221F"),
        HazardousEvent | FMEASheet | FMEAEntry | FaultTree | FaultTreeGate | FaultTreeEvent => {
            ("octagon", "", "#FEF7E0", "#E37400")
        }
        // Security
        CybersecurityGoal => ("doubleoctagon", "", "#F3E8FD", "#6A1B9A"),
        ThreatScenario | DamageScenario | SecurityControl | VulnerabilityReport | TARASheet => {
            ("octagon", "", "#F3E8FD", "#6A1B9A")
        }
        // Views
        ViewDef | View | ViewpointDef | Rendering | RenderingDef | Diagram => {
            ("tab", "", "#FAFAFA", "#BDBDBD")
        }
        // Allocation / dependency (as a node)
        Allocation | AllocationDef | Dependency => ("parallelogram", "", "#F5F5F5", "#9E9E9E"),
        // Fallback
        _ => ("box", "", "#FFFFFF", "#616161"),
    };

    // Definition types get a double border.
    let peripheries = if is_definition(et) { 2 } else { 1 };
    NodeStyle { shape, style_extra, fill, border, peripheries }
}

/// True for `*Def` / definition-family element types (double border in DOT).
fn is_definition(et: &ElementType) -> bool {
    use ElementType::*;
    matches!(
        et,
        PartDef
            | ItemDef
            | AttributeDef
            | PortDef
            | ConnectionDef
            | InterfaceDef
            | ActionDef
            | ConstraintDef
            | RequirementDef
            | CalculationDef
            | StateDef
            | FlowDef
            | UseCaseDef
            | ViewpointDef
            | ViewDef
            | MetadataDef
            | EnumerationDef
            | OccurrenceDef
            | EventOccurrenceDef
            | VerificationCaseDef
            | AnalysisCaseDef
            | AllocationDef
            | ConcernDef
            | CaseDef
            | IndividualDef
            | SuccessionDef
            | RenderingDef
    )
}

/// Style attributes for an edge of a given kind: `(color, style, penwidth)`.
/// Wiring stands out with `penwidth=2`; containment is dashed grey;
/// traceability is coloured-dashed; typing/supertype is solid black.
fn edge_style(k: &EdgeKind) -> (&'static str, &'static str, &'static str) {
    use EdgeKind::*;
    match k {
        Contains => ("#9E9E9E", "dashed", "1"),
        Connection | Flow | Binding | Succession => ("#0277BD", "solid", "2"),
        TypedBy | FeatureTyped | Supertype | Subsets | Redefines => ("#000000", "solid", "1"),
        Verifies | DerivedFrom | DerivedFromSafetyGoal | DerivedFromSecurityGoal => {
            ("#1E8E3E", "dashed", "1")
        }
        Satisfies => ("#1A73E8", "dashed", "1"),
        AllocatedFrom | AllocatedTo => ("#9E9E9E", "dotted", "1"),
        _ => ("#616161", "solid", "1"),
    }
}

fn dot_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn emit_node(id: &str, label: &str, st: &NodeStyle) -> String {
    format!(
        "  \"{}\" [label=\"{}\", shape={}, style=\"filled{}\", \
         fillcolor=\"{}\", color=\"{}\", peripheries={}];",
        dot_escape(id),
        dot_escape(label),
        st.shape,
        st.style_extra,
        st.fill,
        st.border,
        st.peripheries,
    )
}

/// A representative element type per family, for the legend.
fn legend_families() -> Vec<(&'static str, ElementType)> {
    use ElementType::*;
    vec![
        ("Structure (part)", PartDef),
        ("Structure (item)", ItemDef),
        ("Port", PortDef),
        ("Connection", ConnectionDef),
        ("Flow", FlowDef),
        ("Behavior", ActionDef),
        ("Requirement", Requirement),
        ("Verification", TestCase),
        ("Decision (ADR)", ADR),
        ("Variability (feature)", FeatureDef),
        ("Variability (config)", Configuration),
        ("Package", Package),
        ("Safety goal", SafetyGoal),
        ("Safety analysis", HazardousEvent),
        ("Security goal", CybersecurityGoal),
        ("Security analysis", ThreatScenario),
        ("View", ViewDef),
        ("Allocation", Allocation),
    ]
}

fn render_dot(resolver: &Resolver, elements: &[RawElement], sub: &Subgraph) {
    println!("digraph connectivity {{");
    println!("  rankdir=LR;");
    println!("  node [fontname=\"Helvetica\", fontsize=10];");
    println!("  edge [fontname=\"Helvetica\", fontsize=9];");
    println!("  label=\"connectivity: {}\";", dot_escape(&sub.root));
    println!("  labelloc=t;");
    println!();

    // Nodes.
    for qname in &sub.nodes {
        let et = resolver
            .get(elements, qname)
            .and_then(|e| e.frontmatter.element_type.clone());
        let st = et.as_ref().map(node_style).unwrap_or(NodeStyle {
            shape: "box",
            style_extra: "",
            fill: "#FFFFFF",
            border: "#616161",
            peripheries: 1,
        });
        println!("{}", emit_node(qname, qname, &st));
    }
    println!();

    // Edges.
    for (f, t, k) in &sub.edges {
        let (color, style, pen) = edge_style(k);
        println!(
            "  \"{}\" -> \"{}\" [label=\"{}\", color=\"{}\", style={}, penwidth={}];",
            dot_escape(f),
            dot_escape(t),
            k.name(),
            color,
            style,
            pen,
        );
    }
    println!();

    // Legend subgraph: one node per family, self-documenting the mapping.
    println!("  subgraph cluster_legend {{");
    println!("    label=\"Legend\";");
    println!("    style=dashed; color=\"#BDBDBD\"; fontname=\"Helvetica\";");
    for (name, et) in legend_families() {
        let st = node_style(&et);
        let id = format!("legend::{name}");
        let label = format!("{name}\\n({})", type_label(&et));
        println!("  {}", emit_node(&id, &label, &st));
    }
    println!("  }}");

    println!("}}");
}

/// Entry point for the `connectivity` subcommand. `args` is everything after the
/// subcommand word. Returns the process exit code (0 = success).
pub fn cmd_connectivity(elements: &[RawElement], resolver: &Resolver, args: &[String]) -> i32 {
    let Some(opts) = parse_opts(args) else {
        return 2;
    };

    let Some(root_elem) = resolver.resolve_ref(elements, &opts.root) else {
        eprintln!("connectivity: unknown element '{}'", opts.root);
        return 1;
    };
    let root_qname = root_elem.qualified_name.clone();

    let (graph, idx) = build_graph(elements);
    let Some(sub) =
        connectivity_subgraph(&graph, &idx, &root_qname, &opts.kinds, opts.depth, opts.undirected)
    else {
        eprintln!("connectivity: element '{root_qname}' is not in the model graph");
        return 1;
    };

    match opts.format {
        Format::Text => render_text(resolver, elements, &sub),
        Format::Json => render_json(resolver, elements, &sub),
        Format::Dot => render_dot(resolver, elements, &sub),
    }
    0
}
