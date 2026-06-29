const TOC: &str = include_str!("../../../prompts/spec/toc.md");
const TYPES: &str = include_str!("../../../prompts/spec/types.md");
const FIELDS: &str = include_str!("../../../prompts/spec/fields.md");
const NAMESPACE: &str = include_str!("../../../prompts/spec/namespace.md");
const VALIDATION: &str = include_str!("../../../prompts/spec/validation.md");
const TRACEABILITY: &str = include_str!("../../../prompts/spec/traceability.md");
const SAFETY: &str = include_str!("../../../prompts/spec/safety.md");

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
            std::process::exit(1);
        }
    }
}
