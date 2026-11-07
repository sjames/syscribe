//! The project-specific agent guide (GH #441) and the MCP server instructions (GH #440),
//! generated from one [`Guide`] built from the loaded model and `.syscribe.toml`, so the two
//! cannot disagree with each other or with the model.
//!
//! Output is deterministic (sorted, no timestamps, no tool version) so `agent-guide --check`
//! only fails when something the guide states has actually changed.

use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;
use syscribe_model::element::RawElement;
use syscribe_model::resolver::STABLE_ID_KINDS;

pub const BEGIN: &str = "<!-- syscribe:agent-guide -->";
pub const END: &str = "<!-- /syscribe:agent-guide -->";

/// Sections of the guide, in output order.
pub const SECTIONS: &[&str] = &["ids", "types", "gates", "rules", "policy"];

/// Types always described (the ones agents author and move through a lifecycle).
const CORE_TYPES: &[&str] = &["Requirement", "TestCase", "TestPlan", "ADR", "PlanningItem"];
/// Id-identified types always listed in the id table.
const CORE_ID_TYPES: &[&str] = &["Requirement", "TestCase", "ADR", "PlanningItem"];

/// The commands worth naming for each lifecycle type: the ones that create, change or evidence it.
/// (`help <Type>` lists every command that mentions a type; a short guide wants the working set.)
/// A test checks every name here is a real command.
pub(crate) fn type_commands(ty: &str) -> &'static [&'static str] {
    match ty {
        "Requirement" => &["next-id", "set", "trace", "who-verifies", "requirement-coverage"],
        "TestCase" => &["next-id", "set", "scaffold-gherkin", "ingest-results", "who-verifies"],
        "TestPlan" => &["next-id", "set", "testplan"],
        "ADR" => &["next-id", "set"],
        "PlanningItem" => &["next-id", "set", "claim", "release", "steps", "questions"],
        "ReviewRecord" => &["reviews", "set"],
        "ResidualRisk" => &["residual", "set"],
        _ => &[],
    }
}

pub struct IdRow {
    pub ty: String,
    pub prefix: String,
    pub extras: Vec<String>,
    pub next_id: String,
}

pub struct TypeRow {
    pub ty: String,
    pub statuses: Vec<String>,
    pub required: Vec<String>,
    pub commands: Vec<String>,
}

pub struct Guide {
    /// The model root as a command line should spell it (relative to the git root when inside one).
    pub root: String,
    pub ids: Vec<IdRow>,
    pub types: Vec<TypeRow>,
    /// Commands to run, and what each checks, before declaring work finished.
    pub validate: Vec<(String, String)>,
    pub gates: Vec<String>,
    /// The `[agents]` policy in plain words (empty when none is configured).
    pub policy: Vec<String>,
}

/// How a command line should name the model root: relative to the git root when the model is
/// inside a repository (so the guide does not depend on the caller's working directory).
fn display_root(model_root: &Path) -> String {
    let abs = std::fs::canonicalize(model_root).unwrap_or_else(|_| model_root.to_path_buf());
    if let Some(git) = syscribe_model::config::detect_git_root(&abs) {
        if let Ok(rel) = abs.strip_prefix(&git) {
            let r = rel.to_string_lossy().replace('\\', "/");
            return if r.is_empty() { ".".into() } else { format!("{r}/") };
        }
    }
    format!("{}/", model_root.to_string_lossy().trim_end_matches('/'))
}

pub fn build(model_root: &Path, elements: &[RawElement]) -> Guide {
    let root = display_root(model_root);
    let extras = syscribe_model::config::load_id_prefixes(model_root);
    let present: BTreeSet<String> = elements
        .iter()
        .filter_map(|e| e.frontmatter.element_type.as_ref().map(syscribe_model::baseline::element_type_name))
        .collect();

    let mut ids = Vec::new();
    for (ty, prefix, _) in STABLE_ID_KINDS {
        let configured = extras.get(*ty).map(|v| !v.is_empty()).unwrap_or(false);
        if !(CORE_ID_TYPES.contains(ty) || present.contains(*ty) || configured) {
            continue;
        }
        let mut ex: Vec<String> = extras.get(*ty).cloned().unwrap_or_default();
        ex.sort();
        ids.push(IdRow {
            ty: (*ty).to_string(),
            prefix: (*prefix).to_string(),
            extras: ex,
            next_id: format!("syscribe -m {root} next-id {prefix}-<AREA>"),
        });
    }

    let mut type_names: Vec<&str> = CORE_TYPES.to_vec();
    for t in ["ReviewRecord", "ResidualRisk"] {
        if present.contains(t) {
            type_names.push(t);
        }
    }
    let mut types = Vec::new();
    for ty in type_names {
        let Ok(et) = serde_json::from_value::<syscribe_model::element::ElementType>(Value::String(ty.to_string())) else { continue };
        let statuses = syscribe_model::hints::status_vocabulary(&et)
            .map(|v| v.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default();
        let required = crate::mcp::type_field_specs(ty)
            .iter()
            .filter(|f| f["required"].as_bool().unwrap_or(false))
            .filter_map(|f| f["name"].as_str().map(String::from))
            .collect();
        let commands = type_commands(ty).iter().map(|c| c.to_string()).collect();
        types.push(TypeRow { ty: ty.to_string(), statuses, required, commands });
    }

    // Gates: `validate` already trips on [gates] deny_meta and [[gates.coverage]] (exit 2), so the
    // one command covers them; profiles are checked separately by name.
    let mut gates: Vec<String> = Vec::new();
    for c in syscribe_model::tagged_values::load_gate_conditions(model_root) {
        gates.push(format!("deny-meta `{c}` — no element may satisfy it"));
    }
    if let Ok(cov) = syscribe_model::tagged_values::load_coverage_gates(model_root) {
        for g in cov {
            gates.push(format!("coverage `{}` — at least {}% complete", g.text, g.min_complete));
        }
    }
    // `[planning]` gates (GH #455/#456): warnings, so `validate` alone does not fail on them.
    let planning = syscribe_model::config::PlanningPolicy::load(model_root);
    if planning.require_testcase_evidence {
        gates.push(format!("planning `require_testcase_evidence` — record every active TestCase you deliver as evidence on its PlanningItem (`syscribe -m {root} set <PI-id> evidence.add ref=<TC-id>`), or `W319` is raised"));
    }
    if planning.require_commit_evidence {
        gates.push(format!("planning `require_commit_evidence` — before a leaf PlanningItem is done, record the commit that delivered it (`syscribe -m {root} set <PI-id> evidence.add commit=<sha>`), or `W321` is raised"));
    }
    if planning.check_commit_paths {
        gates.push("planning `check_commit_paths` — a commit named as evidence must touch a path the item names (its `path:` evidence, or the file, `implementedBy:` or `sourceFile:` of its `ref:` evidence), or `W320` is raised".to_string());
    }
    // `[traceability] require_amendments` (GH #454): a warning, so `validate` alone does not fail on it.
    if syscribe_model::config::require_amendments(model_root) {
        gates.push(format!("traceability `require_amendments` — when you change an accepted ADR or an approved Requirement, record it (`syscribe -m {root} set <id> amendment.add kind=clarification|scope-change reason=\"…\" --by <you>`), or `W324` is raised"));
    }
    // `[references] copied_values` (GH #452): a warning, so `validate` alone does not fail on it.
    if syscribe_model::req_items::ReferencesConfig::load(model_root).copied_values {
        gates.push("references `copied_values` — write a design value as `{{Elem.attr}}`, not as the number, in the body of an element that links to it, or `W095` is raised".to_string());
    }
    // `[traceCode]` (GH #451): code no element names. `validate = true` makes it W326/W327 (warnings).
    let trace_code = syscribe_model::trace_code::TraceCodeConfig::load(model_root);
    if trace_code.as_ref().is_some_and(|c| c.validate) {
        gates.push(format!("traceCode `validate` — design before code: new code (a crate, build directory or file under the `[traceCode]` roots) must be named by a design element's `implementedBy:` or a TestCase's `sourceFile:`, or `W326` is raised; a baseline entry that is now traced must be dropped (`syscribe -m {root} trace-code --write-baseline`), or `W327` is raised"));
    }
    let mut validate = vec![(
        format!("syscribe -m {root} validate"),
        if gates.is_empty() { "the model's own checks".to_string() } else { "the model's own checks and the gates above".to_string() },
    )];
    if trace_code.is_some() {
        validate.push((format!("syscribe -m {root} trace-code"), "every code unit under the `[traceCode]` roots is named by an `implementedBy:`/`sourceFile:` or listed in the trace-code baseline".to_string()));
    }
    let mut profiles: Vec<(String, syscribe_model::config::Profile)> =
        syscribe_model::config::load_profiles(model_root).into_iter().collect();
    profiles.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, p) in profiles {
        let mut scope: Vec<String> = Vec::new();
        if let Some(s) = &p.sil {
            scope.push(format!("integrity {s}"));
        }
        if let Some(s) = &p.status {
            scope.push(format!("status {s}"));
        }
        if let Some(s) = &p.tag {
            scope.push(format!("tag {s}"));
        }
        let mut promote = p.promote.clone();
        promote.sort();
        let what = format!(
            "promotes {}{}",
            if promote.is_empty() { "no codes".to_string() } else { promote.join(", ") },
            if scope.is_empty() { String::new() } else { format!(" for {}", scope.join(", ")) }
        );
        validate.push((format!("syscribe -m {root} validate --profile {name}"), what));
    }
    let policy = syscribe_model::agent_policy::load(model_root).map(|p| p.describe()).unwrap_or_default();
    Guide { root, ids, types, validate, gates, policy }
}

/// The safe-mutation rules. Every command named here exists (a test checks that).
fn rules(root: &str) -> Vec<String> {
    vec![
        format!("Never invent an id: `syscribe -m {root} next-id <PREFIX>-<AREA>` returns the next free one; start a new element from `syscribe template <Type>` (no model needed)."),
        format!("Change an existing field with `syscribe -m {root} set <id> status=<value>` (also `evidence.add`, `achieves.add`, `<list>.add`) rather than editing YAML: the value is checked before anything is written."),
        format!("When blocked on a decision, record it instead of guessing: `syscribe -m {root} set <PI> question.add text=\"…\" [recommended=…]`; list the open ones with `questions --open` and see a breakdown with `steps <PI>`. A question with `answerBy: human` (shown in `questions --json`, marked *(for a person)* in `questions`) is a person's to answer, never yours."),
        format!("Attribute and record a change: add `--by <you> --reason \"<why>\" --commit` to a writing command and it commits just the files it changed, you as author; `syscribe -m {root} log <element>` shows who changed what. `--expect-hash <contentHash>` (from `show`) refuses an edit based on a stale read."),
        format!("Rename or move with `syscribe -m {root} move <src> <dest>`, which rewrites every reference."),
        format!("Own shared work with `syscribe -m {root} claim <PI-id> --by <agent>` and `release <PI-id>`; never hand-edit `claimedBy`/`claimedAt`."),
        "Preview before writing where the command allows it: `--dry-run` on `set`, `move`, `claim`, `release` and `applies-when`.".to_string(),
        format!("Amend, never silently edit, an accepted ADR or an approved Requirement: change the text and record why with `syscribe -m {root} set <id> amendment.add kind=clarification|scope-change reason=\"…\" [ref=…] --by <you>`; a scope-change needs `ref=` to the owner's decision (an answered question `PI-X#Qn` or a ReviewRecord). To replace a decision, write a new ADR with `supersedes:`."),
        "Cite one item of a requirement by its anchor, not its number: write `{#slug}` after the item's number in the requirement body and cite it as `REQ-X#slug` (a number such as `REQ-X#5` shifts when an item is inserted); cite a PlanningItem question as `PI-X#Q3`. `check-ref REQ-X#slug` confirms it resolves; a dangling citation is `W092`.".to_string(),
        format!("Write a design value as a value reference, never a copied number: `{{{{Elem.attr}}}}` (an element id or qualified name, then an attribute such as a `features:` entry with a `value:`) in prose or a gherkin scenario renders the current value and unit; `syscribe -m {root} refs <Elem>.<attr>` lists every use and every copy. An unresolvable reference is `E331`; a literal equal to a previous value of a linked attribute is `W096` (a stale copy)."),
        format!("Design before code: before adding a crate, module, script or test file, add the design element whose `implementedBy:` names it or the TestCase whose `sourceFile:` names it; `syscribe -m {root} trace-code` lists the code nothing names (with a `[traceCode]` table in `.syscribe.toml`)."),
        "Check a reference with `check-ref <ref>`; an unfamiliar finding with `syscribe explain <code>`; a type with `syscribe help <Type>`.".to_string(),
        "A task is not finished until every command under \"Before declaring work done\" exits 0.".to_string(),
    ]
}

fn want(sections: &[String], name: &str) -> bool {
    sections.is_empty() || sections.iter().any(|s| s == name)
}

/// The guide as Markdown, wrapped in the region markers.
pub fn markdown(g: &Guide, sections: &[String]) -> String {
    let mut o = String::new();
    o.push_str(BEGIN);
    o.push('\n');
    o.push_str("<!-- Generated by `syscribe agent-guide` from the model and .syscribe.toml. Edit those, then regenerate; do not edit this block. -->\n\n");
    o.push_str(&format!("## Syscribe model guide (`{}`)\n\n", g.root));
    o.push_str("Discover with `syscribe help <Type>`, `syscribe help --search <keyword>`, `syscribe help tasks` and `syscribe explain <code>`.\n");
    if want(sections, "ids") {
        o.push_str("\n### Ids\n\n| Type | Prefix | Also accepted | Next free id |\n|---|---|---|---|\n");
        for r in &g.ids {
            let extra = if r.extras.is_empty() { "—".to_string() } else { r.extras.join(", ") };
            o.push_str(&format!("| {} | `{}` | {} | `{}` |\n", r.ty, r.prefix, extra, r.next_id));
        }
    }
    if want(sections, "types") {
        o.push_str("\n### Types\n\n");
        for t in &g.types {
            let statuses = if t.statuses.is_empty() { "free text".to_string() } else { t.statuses.join(" · ") };
            o.push_str(&format!(
                "- **{}** — status: {}; required: {}; commands: {}\n",
                t.ty,
                statuses,
                t.required.iter().map(|r| format!("`{r}`")).collect::<Vec<_>>().join(", "),
                t.commands.iter().map(|c| format!("`{c}`")).collect::<Vec<_>>().join(", ")
            ));
        }
    }
    if want(sections, "gates") {
        o.push_str("\n### Before declaring work done\n\nRun these; each must exit 0 (`1` errors, `2` a tripped gate).\n\n");
        for (cmd, what) in &g.validate {
            o.push_str(&format!("- `{cmd}` — {what}\n"));
        }
        if !g.gates.is_empty() {
            o.push_str("\nGates in `.syscribe.toml`:\n\n");
            for gate in &g.gates {
                o.push_str(&format!("- {gate}\n"));
            }
        }
    }
    if !g.policy.is_empty() && want(sections, "policy") {
        o.push_str("\n### Project policy\n\nThese rules apply to a change that names an actor (`--by`); a person edits the file or passes `--as-human`. A refused change exits 1 and says which rule.\n\n");
        for r in &g.policy {
            o.push_str(&format!("- {r}\n"));
        }
    }
    if want(sections, "rules") {
        o.push_str("\n### Safe mutation\n\n");
        for r in rules(&g.root) {
            o.push_str(&format!("- {r}\n"));
        }
    }
    o.push('\n');
    o.push_str(END);
    o.push('\n');
    o
}

pub fn json(g: &Guide, sections: &[String]) -> Value {
    let mut o = serde_json::Map::new();
    o.insert("schemaVersion".into(), json!("1.0"));
    o.insert("root".into(), json!(g.root));
    if want(sections, "ids") {
        o.insert(
            "ids".into(),
            json!(g.ids.iter().map(|r| json!({"type": r.ty, "prefix": r.prefix, "extraPrefixes": r.extras, "nextId": r.next_id})).collect::<Vec<_>>()),
        );
    }
    if want(sections, "types") {
        o.insert(
            "types".into(),
            json!(g.types.iter().map(|t| json!({"type": t.ty, "statuses": t.statuses, "required": t.required, "commands": t.commands})).collect::<Vec<_>>()),
        );
    }
    if want(sections, "gates") {
        o.insert("validate".into(), json!(g.validate.iter().map(|(c, w)| json!({"command": c, "checks": w})).collect::<Vec<_>>()));
        o.insert("gates".into(), json!(g.gates));
    }
    if want(sections, "rules") {
        o.insert("rules".into(), json!(rules(&g.root)));
    }
    if want(sections, "policy") {
        o.insert("policy".into(), json!(g.policy));
    }
    Value::Object(o)
}

/// Replace (or append) the marked region of `existing` with `block`; text outside it is untouched.
/// `Err` when the markers are unbalanced or repeated, so a damaged file is never guessed at.
pub fn splice(existing: &str, block: &str) -> Result<String, String> {
    match region(existing)? {
        Some((s, e)) => Ok(format!("{}{}{}", &existing[..s], block, &existing[e..])),
        None => {
            let mut out = existing.to_string();
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(block);
            Ok(out)
        }
    }
}

/// Byte range of the marked region in `text`, including both marker lines and the newline after the end marker.
pub fn region(text: &str) -> Result<Option<(usize, usize)>, String> {
    let begins: Vec<usize> = text.match_indices(BEGIN).map(|(i, _)| i).collect();
    let ends: Vec<usize> = text.match_indices(END).map(|(i, _)| i).collect();
    match (begins.as_slice(), ends.as_slice()) {
        ([], []) => Ok(None),
        ([b], [e]) if b < e => {
            let mut stop = e + END.len();
            if text[stop..].starts_with('\n') {
                stop += 1;
            }
            Ok(Some((*b, stop)))
        }
        _ => Err(format!("expected exactly one `{BEGIN}` … `{END}` pair, in that order")),
    }
}

/// The MCP `initialize` instructions: a short, model-specific orientation (GH #440).
pub fn mcp_instructions(g: &Guide, read_only: bool) -> String {
    let mut o = String::new();
    o.push_str("Query and guard-write a Syscribe systems model over MCP. Read tools are token-efficient; ");
    o.push_str("write tools default to dry_run and refuse commits that leave a dangling reference or link-type error; ");
    o.push_str("every other new error is listed in the delta with gating:false, so read newErrors. ");
    o.push_str("Relationship vocabulary is per-project: call the `link_types` tool to discover the project's declared link types before authoring a `links:` field, and never invent an undeclared one (E630); use `follow` to traverse any link. ");
    o.push_str("The server watches the model files and reloads automatically when they change on disk (unless started with --no-watch); call `reload` only to force a re-read.\n\n");
    if read_only {
        o.push_str("This server is READ-ONLY: it exposes no tool that writes.\n\n");
    }
    o.push_str("Orient first: `describe_type` (fields, status vocabulary, the commands that act on a type), `gates`, `link_types`, `explain_finding`.\n");
    if !read_only {
        o.push_str("Add an element: `next_id` → `template` → `create_element` (dry_run, read the delta, then dry_run:false) → `validate_element`. Change one: `update_element` (dry_run first). ");
    }
    o.push_str("Before calling work done run `validate` and `gates`; neither may report an error or a tripped gate.\n");
    o.push_str("Rules: never invent an id (`next_id`); never set `claimedBy`/`claimedAt` through a tool — ownership is the CLI `syscribe claim`/`release`; ");
    if read_only {
        o.push_str("`questions` lists the PlanningItems' open decisions and `steps` shows a PlanningItem's breakdown; ");
    } else {
        o.push_str("when blocked on a decision record it with the `question` tool instead of guessing (`questions` lists the open ones, `steps` shows a PlanningItem's breakdown); ");
    }
    o.push_str("cite one requirement item by its anchor (`{#slug}` after the item's number, cited as `REQ-X#slug`), not by number, and a question as `PI-X#Q3` (W092 flags a dangling one); ");
    o.push_str("write a design value as a value reference `{{Elem.attr}}` instead of copying the number (E331 flags an unresolvable one, W096 a stale copy of a previous value); ");
    o.push_str("amend, never silently edit, an accepted ADR or approved Requirement — append an `amendments:` entry `{date, by, reason, kind: clarification|scope-change, ref?}` (a scope-change needs `ref` to the owner's answered question or ReviewRecord); ");
    o.push_str("a `status` outside the type's vocabulary is refused; pass `by`, `reason`, `commit` and `expect_hash` to a write tool to attribute it, commit just that change and refuse a stale edit (`log` reads the history).\n\n");
    o.push_str("Id prefixes: ");
    o.push_str(
        &g.ids
            .iter()
            .map(|r| {
                if r.extras.is_empty() {
                    format!("{} {}", r.ty, r.prefix)
                } else {
                    format!("{} {} (also {})", r.ty, r.prefix, r.extras.join(", "))
                }
            })
            .collect::<Vec<_>>()
            .join("; "),
    );
    o.push_str(".\nStatus vocabularies: ");
    o.push_str(
        &g.types
            .iter()
            .filter(|t| !t.statuses.is_empty())
            .map(|t| format!("{} {}", t.ty, t.statuses.join("|")))
            .collect::<Vec<_>>()
            .join("; "),
    );
    o.push_str(".\n");
    if !g.gates.is_empty() || g.validate.len() > 1 {
        o.push_str("Gates: ");
        let mut parts: Vec<String> = g.gates.clone();
        parts.extend(g.validate.iter().skip(1).map(|(c, _)| match c.split_once(" validate ") {
            Some((_, p)) => format!("profile run `validate {p}`"),
            // Not a profile: another command that must exit 0 (`trace-code`, GH #451).
            None => format!("run `{}`", c.split_once(&format!("-m {} ", g.root)).map_or(c.as_str(), |(_, rest)| rest)),
        }));
        o.push_str(&parts.join("; "));
        o.push_str(".\n");
    }
    if !g.policy.is_empty() {
        o.push_str("Project policy (every MCP write is held to it; a refusal names the rule): ");
        o.push_str(&g.policy.join(" "));
        o.push('\n');
    }
    o.push_str("Workflow prompts: `add-requirement-with-test`, `record-test-evidence`, `review-change-set`, `close-planning-item`, `check-variant`.");
    o
}

/// `agent-guide [--format markdown|json] [--sections a,b] [--write <file> | --check <file>]`.
/// Returns the process exit code: `0` ok / up to date, `1` usage error, unwritable file, or a stale region.
pub fn cmd_agent_guide(model_root: &Path, elements: &[RawElement], args: &[String]) -> i32 {
    let value = |flag: &str| args.windows(2).find(|w| w[0] == flag).map(|w| w[1].as_str());
    let format = match crate::cliargs::enum_value("agent-guide", args, "--format", &["markdown", "json"], "markdown") {
        Ok(f) => f,
        Err(m) => {
            eprintln!("{m}");
            return 1;
        }
    };
    let sections: Vec<String> = value("--sections")
        .map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    if let Some(bad) = sections.iter().find(|s| !SECTIONS.contains(&s.as_str())) {
        eprintln!(
            "agent-guide: unknown section '{bad}';{} valid sections: {}",
            crate::suggest::did_you_mean(bad, SECTIONS.iter().copied()),
            SECTIONS.join(", ")
        );
        return 1;
    }
    let (write, check) = (value("--write"), value("--check"));
    if write.is_some() && check.is_some() {
        eprintln!("agent-guide: --write and --check are mutually exclusive");
        return 1;
    }
    if (write.is_some() || check.is_some()) && format == "json" {
        eprintln!("agent-guide: --write and --check work on the Markdown block; omit --format json");
        return 1;
    }
    let g = build(model_root, elements);
    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&json(&g, &sections)).unwrap());
        return 0;
    }
    let block = markdown(&g, &sections);
    if let Some(path) = write {
        let existing = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => {
                eprintln!("agent-guide: cannot read {path}: {e}");
                return 1;
            }
        };
        let updated = match splice(&existing, &block) {
            Ok(u) => u,
            Err(m) => {
                eprintln!("agent-guide: {path}: {m}");
                return 1;
            }
        };
        if updated == existing {
            println!("{path} is already up to date.");
            return 0;
        }
        if let Err(e) = syscribe_model::fsio::write(path, updated) {
            eprintln!("agent-guide: cannot write {path}: {e}");
            return 1;
        }
        println!("{} the agent-guide region of {path}.", syscribe_model::fsio::verb("Updated", "Would update"));
        return 0;
    }
    if let Some(path) = check {
        let existing = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("agent-guide: cannot read {path}: {e}");
                return 1;
            }
        };
        let current = match region(&existing) {
            Ok(Some((s, e))) => existing[s..e].to_string(),
            Ok(None) => {
                eprintln!("{path} has no `{BEGIN}` region. Run `syscribe -m <root> agent-guide --write {path}`.");
                return 1;
            }
            Err(m) => {
                eprintln!("agent-guide: {path}: {m}");
                return 1;
            }
        };
        if current.trim_end() == block.trim_end() {
            println!("{path} is up to date.");
            return 0;
        }
        eprintln!("{path}: the agent-guide region is out of date. Regenerate it with `syscribe -m <root> agent-guide --write {path}`.");
        return 1;
    }
    print!("{block}");
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_command_the_guide_names_exists() {
        for ty in ["Requirement", "TestCase", "TestPlan", "ADR", "PlanningItem", "ReviewRecord", "ResidualRisk"] {
            for c in type_commands(ty) {
                assert!(crate::help::is_command(c), "{ty}: `{c}` is not a command");
            }
        }
    }

    #[test]
    fn splice_replaces_only_the_region() {
        let old = format!("# Title\n\nhand written\n\n{BEGIN}\nold\n{END}\n\nmore hand written\n");
        let out = splice(&old, &format!("{BEGIN}\nnew\n{END}\n")).unwrap();
        assert_eq!(out, format!("# Title\n\nhand written\n\n{BEGIN}\nnew\n{END}\n\nmore hand written\n"));
    }

    #[test]
    fn splice_appends_when_there_is_no_region_and_keeps_existing_text() {
        let out = splice("# Title\n", &format!("{BEGIN}\nx\n{END}\n")).unwrap();
        assert!(out.starts_with("# Title\n\n") && out.ends_with(&format!("{END}\n")));
        assert_eq!(splice("", &format!("{BEGIN}\nx\n{END}\n")).unwrap(), format!("{BEGIN}\nx\n{END}\n"));
    }

    #[test]
    fn damaged_markers_are_refused() {
        assert!(splice(&format!("{BEGIN}\nonly begin\n"), "x").is_err());
        assert!(splice(&format!("{END}\n{BEGIN}\n"), "x").is_err());
        assert!(splice(&format!("{BEGIN}\n{END}\n{BEGIN}\n{END}\n"), "x").is_err());
    }
}
