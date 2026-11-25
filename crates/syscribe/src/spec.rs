const TOC: &str = include_str!("../../../prompts/spec/toc.md");
const TYPES: &str = include_str!("../../../prompts/spec/types.md");
const FIELDS: &str = include_str!("../../../prompts/spec/fields.md");
const NAMESPACE: &str = include_str!("../../../prompts/spec/namespace.md");
const VALIDATION: &str = include_str!("../../../prompts/spec/validation.md");
const TRACEABILITY: &str = include_str!("../../../prompts/spec/traceability.md");
const EXPORT: &str = include_str!("../../../prompts/spec/export.md");
const SAFETY: &str = include_str!("../../../prompts/spec/safety.md");
// The sections below embed the model-guide pages themselves, so the CLI help and the
// documentation cannot drift apart.
const BEHAVIOUR: &str = concat!(
    include_str!("../../../docs/model-guide/state-machines.md"),
    "\n\n---\n\n",
    include_str!("../../../docs/model-guide/action-flows.md"),
    "\n\n---\n\n",
    include_str!("../../../docs/model-guide/plant-models.md"),
    "\n\n---\n\n",
    include_str!("../../../docs/model-guide/fmi-plants.md"),
    "\n\n---\n\n",
    include_str!("../../../docs/model-guide/behaviour-models.md"),
    "\n\n---\n\n",
    include_str!("../../../docs/model-guide/system-composition.md"),
);
const EVIDENCE: &str = include_str!("../../../docs/model-guide/executable-evidence.md");
const DIAGRAMS: &str = include_str!("../../../docs/format/diagrams.md");
const VARIABILITY: &str = include_str!("../../../docs/model-guide/variability.md");

/// The spec sections, keyed by the short name used in `syscribe spec <section>`
/// and in the MCP `syscribe://spec/<section>` resource URIs.
pub const SECTIONS: &[(&str, &str)] = &[
    ("toc", TOC),
    ("types", TYPES),
    ("fields", FIELDS),
    ("namespace", NAMESPACE),
    ("validation", VALIDATION),
    ("traceability", TRACEABILITY),
    ("safety", SAFETY),
    ("behaviour", BEHAVIOUR),
    ("evidence", EVIDENCE),
    ("diagrams", DIAGRAMS),
    ("variability", VARIABILITY),
    ("export", EXPORT),
];

pub fn cmd_spec(section: &str) {
    match section {
        "" | "toc" => print!("{}", TOC),
        "types" => print!("{}", TYPES),
        "fields" => print!("{}", FIELDS),
        "namespace" => print!("{}", NAMESPACE),
        "validation" => print!("{}", VALIDATION),
        "traceability" => print!("{}", TRACEABILITY),
        "safety" => print!("{}", SAFETY),
        "behaviour" => print!("{}", BEHAVIOUR),
        "evidence" => print!("{}", EVIDENCE),
        "diagrams" => print!("{}", DIAGRAMS),
        "variability" => print!("{}", VARIABILITY),
        "export" => print!("{}", EXPORT),
        other => {
            eprintln!("Unknown spec section: {other}");
            eprintln!();
            eprintln!("Available sections:");
            eprintln!("  syscribe spec              — table of contents");
            eprintln!("  syscribe spec types        — element type inventory");
            eprintln!("  syscribe spec fields       — complete frontmatter field reference");
            eprintln!("  syscribe spec namespace    — directory conventions, cross-refs, multiplicity");
            eprintln!("  syscribe spec validation   — all validation rule codes");
            eprintln!("  syscribe spec traceability — traceability rules R-001–R-007");
            eprintln!("  syscribe spec safety       — safety/security analysis elements");
            eprintln!("  syscribe spec behaviour    — state machines, action flows, plants, `models:`, composites");
            eprintln!("  syscribe spec evidence     — executable evidence: requirement-coverage, exec-coverage");
            eprintln!("  syscribe spec diagrams     — diagram elements, derived views, styles");
            eprintln!("  syscribe spec variability  — feature models, configurations, `{{{{Feature.param}}}}` placeholders");
            eprintln!("  syscribe spec export       — schema of `export` and `export --resolved`");
            std::process::exit(1);
        }
    }
}
