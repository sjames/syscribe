//! Documented-example drift gate.
//!
//! Every fenced YAML (or `---`-delimited frontmatter) example that declares a
//! top-level `type:` in the user-facing format documentation is materialised
//! as a file in a scratch model and run through `syscribe validate --json`.
//! An example passes when it produces no findings beyond the small
//! [`TOLERATED`] set — codes that only mean "this snippet references something
//! that lives outside the snippet" or "this isolated element is not yet
//! traced", which is inherent to a snippet shown on its own.
//!
//! Documents covered: see [`DOC_SOURCES`].
//!
//! ## Markers
//!
//! An HTML comment on the last non-blank line before a fence changes how it
//! is checked:
//!
//! * `<!-- syscribe-example: expect E123 W456 reason="…" -->` — the example
//!   is intentionally invalid (or demonstrates an advisory): each listed code
//!   must appear, and no other non-tolerated code may appear.
//! * `<!-- syscribe-example: skip reason="…" -->` — not a model example
//!   (partial fragment, schema table rendered as YAML, …); not checked.
//! * `<!-- syscribe-example: path="Some/Dir/File.md" -->` — place the element
//!   at that model-relative path instead of the derived one (for examples whose
//!   meaning depends on their position in the namespace tree).
//! * `<!-- syscribe-example: config="<file-name>" -->` — copy
//!   `crates/syscribe/tests/doc_examples/<file-name>` in as the scratch
//!   model's `.syscribe.toml` (for examples that only validate under a project
//!   configuration, e.g. a `[users]` roster or a `[profiles.*]` profile).
//!
//! Options combine: `<!-- syscribe-example: expect W042 reason="…" path="A/B.md" -->`.
//! Every `skip` and `expect` marker must carry a `reason="…"`; when the reason
//! is a tool bug, cite its issue (`reason="GH #123: …"`).
//!
//! ## Grouping
//!
//! Unmarked examples under the same Markdown heading of the same document are
//! placed in one scratch model so they can reference each other (a
//! `Requirement` next to the `TestCase` that verifies it). An example whose
//! target path is already taken in its section's model (e.g. a "before" and an
//! "after" version of the same element) spills into a fresh model. Examples
//! carrying `expect`/`config` markers always get a model of their own.
//!
//! Set `SYSCRIBE_DOC_EXAMPLES_VERBOSE=1` to print every example's outcome.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use syscribe_model::element::ElementType;

/// Documentation sources scanned for examples, relative to the workspace root.
/// A trailing `/` means "every `*.md` file directly in that directory"
/// (symlinks are skipped — `docs/format/spec.md` is a link to the spec).
const DOC_SOURCES: &[&str] = &[
    "spec/markdown-sysml-format.md",
    "prompts/spec/",
    "prompts/create-model.md",
    "prompts/create-magicgrid-model.md",
    "docs/format/",
    "docs/guides/",
    "docs/model-guide/",
];

/// Finding codes tolerated on any example. Keep this list minimal: every entry
/// must be a code that fires only because a snippet is shown in isolation.
/// Schema, id, enum and unknown-field codes (E0xx, E025, W047, …) are never
/// tolerated.
const TOLERATED: &[(&str, &str)] = &[
    // ── Unresolved cross-references. A snippet names elements that the
    //    surrounding (imaginary) model defines; only the *shape* of the
    //    reference is under test here, and a wrong field name or a malformed
    //    value is still caught by the schema/enum codes, which are never
    //    tolerated. Each code below fires only for "target not found".
    ("E102", "unresolved verifies target"),
    ("E103", "unresolved derivedFrom target"),
    ("E110", "unresolved supertype target"),
    ("E111", "unresolved typedBy target"),
    ("E114", "unresolved satisfies target"),
    ("E209", "unresolved appliesWhen feature"),
    ("E234", "Configuration derivedFrom base not in the snippet"),
    ("E311", "unresolved breakdownAdr"),
    ("E316", "unresolved refines target"),
    ("E317", "metadata: application names a MetadataDef not in the snippet"),
    ("E502", "unresolved allocatedFrom"),
    ("E503", "unresolved allocatedTo"),
    ("E516", "unresolved subConfigurations entry"),
    ("E632", "unresolved links: target"),
    ("E601", "unresolved TestPlan testCases entry"),
    ("E603", "unresolved TestPlan demonstrates entry"),
    ("E606", "unresolved TestPlan configurations entry"),
    ("E704", "unresolved ReviewRecord reviews entry"),
    ("E710", "unresolved PlanningItem parent"),
    ("E714", "unresolved PlanningItem achieves"),
    ("E716", "unresolved PlanningItem evidence ref"),
    ("E720", "unresolved PlanningItem blockedBy"),
    ("E825", "unresolved hazardousEvents"),
    ("E826", "unresolved damageScenarios"),
    ("E827", "unresolved threatScenarios"),
    ("E828", "unresolved implementsGoals"),
    ("E829", "unresolved mitigatedBy"),
    ("E830", "unresolved affectedElements"),
    ("E831", "unresolved derivedFromCybersecurityGoal"),
    ("E832", "unresolved derivedFromSafetyGoal"),
    ("E844", "unresolved DamageScenario hazardRef"),
    ("E851", "unresolved ConfirmationMeasure confirms"),
    ("E855", "unresolved Argument supports"),
    ("E858", "unresolved AssumptionOfUse appliesTo"),
    ("E864", "unresolved DamageScenario assets"),
    ("E902", "unresolved FaultTree topEvent"),
    ("E906", "unresolved fault-tree gate inputs"),
    ("E917", "unresolved AttackTree threatRef"),
    ("E920", "unresolved attack-tree gate inputs"),
    ("E927", "unresolved FaultTreeEvent ref"),
    ("E954", "unresolved Conduit zone"),
    ("E955", "unresolved Zone member"),
    ("W062", "unresolved TradeStudy objective"),
    ("W064", "unresolved TradeStudy alternative element"),
    ("W079", "unresolved state-machine behavior reference"),
    ("W401", "unresolved diagram subject"),
    ("W402", "unresolved diagram shape ref"),
    ("W404", "unresolved operation returnType"),
    ("W408", "unresolved Mermaid %% ref: annotation"),
    ("W501", "unresolved exhibitsStates entry"),
    ("W904", "unresolved FMEAEntry ref"),
    ("W927", "unresolved FMEAEntry ftaRef"),
    // ── Files the snippet points at on disk (source, test, evidence,
    //    companion diagram files) are not part of the snippet.
    ("W004", "TestCase sourceFile not on disk"),
    ("W023", "implementedBy path not on disk"),
    ("E402", "companion SVG not on disk"),
    ("W414", "companion .puml not on disk"),
    ("E717", "PlanningItem evidence path not on disk"),
    // ── Coverage / orphan advisories: an element shown on its own is, by
    //    construction, not yet traced to the rest of a model.
    ("W002", "Requirement has no active TestCase"),
    ("W005", "Requirement has no derivedFrom/derivedChildren (orphan)"),
    ("W007", "definition never used as a supertype or type"),
    ("W029", "timing requirement has no measuring TestCase"),
    ("W036", "AttackTree has no child gates/steps"),
    ("W300", "leaf Requirement has no satisfying element"),
    ("W306", "high-integrity Requirement has no satisfying element"),
    ("W612", "TestPlan's TestCases live outside the snippet"),
    ("W800", "HazardousEvent not referenced by a SafetyGoal"),
    ("W802", "CybersecurityGoal not implemented by a SecurityControl"),
    ("W804", "CybersecurityGoal has no derived Requirement"),
    ("W805", "SafetyGoal has no derived Requirement"),
    ("W810", "Asset not referenced by a DamageScenario"),
    ("W900", "FaultTree has no child gates/events"),
    ("W953", "Zone referenced by no Conduit"),
];

/// Codes tolerated only when the snippet shows frontmatter alone (no body):
/// the checker then supplies a placeholder body, so body-derived checks
/// cannot pass. An example that *does* show a body gets no such allowance.
const TOLERATED_WITHOUT_BODY: &[(&str, &str)] = &[
    ("W001", "normative 'shall' text lives in the omitted body"),
    ("E011", "the ```gherkin block lives in the omitted body"),
    ("E106", "testFunctions scenarios live in the omitted body"),
    ("W406", "the inline SVG lives in the omitted body"),
];

#[derive(Debug, Clone, Default)]
struct Marker {
    skip: Option<String>,
    expect: Option<Vec<String>>,
    path: Option<String>,
    config: Option<String>,
}

#[derive(Debug, Clone)]
struct ElemSrc {
    frontmatter: String,
    body: String,
    rel_path: String,
}

#[derive(Debug, Clone)]
struct Example {
    doc: String,
    line: usize,
    section: String,
    marker: Marker,
    elements: Vec<ElemSrc>,
    /// Set when the example could not be materialised (e.g. YAML syntax error).
    broken: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Finding {
    code: String,
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    message: String,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn doc_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for src in DOC_SOURCES {
        if let Some(dir) = src.strip_suffix('/') {
            let mut files: Vec<PathBuf> = std::fs::read_dir(root.join(dir))
                .unwrap_or_else(|e| panic!("read_dir {dir}: {e}"))
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("md"))
                .collect();
            files.sort();
            out.extend(files);
        } else {
            out.push(root.join(src));
        }
    }
    out
}

fn parse_marker(line: &str) -> Option<Marker> {
    let t = line.trim();
    let inner = t.strip_prefix("<!--")?.strip_suffix("-->")?.trim();
    let rest = inner.strip_prefix("syscribe-example:")?.trim();
    let mut m = Marker::default();
    // Tokenise: bare words and key="quoted value" pairs.
    let mut words: Vec<String> = Vec::new();
    let mut chars = rest.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        let mut tok = String::new();
        let mut in_q = false;
        while let Some(&c) = chars.peek() {
            if c == '"' {
                in_q = !in_q;
                chars.next();
                continue;
            }
            if c.is_whitespace() && !in_q {
                break;
            }
            tok.push(c);
            chars.next();
        }
        words.push(tok);
    }
    let mut mode: Option<&str> = None;
    let mut reason: Option<String> = None;
    for w in &words {
        if let Some(v) = w.strip_prefix("reason=") {
            reason = Some(v.to_string());
        } else if let Some(v) = w.strip_prefix("path=") {
            m.path = Some(v.to_string());
        } else if let Some(v) = w.strip_prefix("config=") {
            m.config = Some(v.to_string());
        } else if w == "skip" {
            mode = Some("skip");
            m.skip = Some(String::new());
        } else if w == "expect" {
            mode = Some("expect");
            m.expect.get_or_insert_with(Vec::new);
        } else if mode == Some("expect") {
            m.expect.get_or_insert_with(Vec::new).push(w.clone());
        } else {
            panic!("unrecognised syscribe-example marker token `{w}` in `{t}`");
        }
    }
    if m.skip.is_some() {
        let r = reason.clone().unwrap_or_default();
        assert!(
            !r.trim().is_empty(),
            "a `skip` marker must give a reason=\"…\": `{t}`"
        );
        m.skip = Some(r);
    }
    if let Some(exp) = &m.expect {
        assert!(!exp.is_empty(), "an `expect` marker must list at least one code: `{t}`");
        assert!(
            reason.as_deref().map(|r| !r.trim().is_empty()).unwrap_or(false),
            "an `expect` marker must give a reason=\"…\": `{t}`"
        );
    }
    Some(m)
}

fn is_basic_name(s: &str) -> bool {
    let mut cs = s.chars();
    matches!(cs.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && cs.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn has_top_level_type(chunk: &str) -> bool {
    chunk.lines().any(|l| {
        l.strip_prefix("type:")
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
    })
}

/// Split a fence body into element sources: (frontmatter, body, path hint).
///
/// A path hint is a `# model/Dir/File.md` comment, either as the first line of
/// the frontmatter chunk or as the last line of the chunk just before it (the
/// "one comment line, then `---`" layout used for multi-file listings).
fn split_elements(content: &str, yaml_fence: bool) -> Vec<(String, String, Option<String>)> {
    let mut chunks: Vec<String> = vec![String::new()];
    for l in content.lines() {
        if l.trim_end() == "---" {
            chunks.push(String::new());
        } else {
            let c = chunks.last_mut().unwrap();
            c.push_str(l);
            c.push('\n');
        }
    }
    let mut out: Vec<(String, String, Option<String>)> = Vec::new();
    let mut prev_hint: Option<String> = None;
    for c in chunks {
        if has_top_level_type(&c) {
            let hint = path_hint(&c).or(prev_hint.take());
            out.push((c, String::new(), hint));
            continue;
        }
        // A trailing hint line belongs to the next element, not to this body.
        let mut c = c;
        prev_hint = c
            .lines()
            .filter(|l| !l.trim().is_empty())
            .last()
            .and_then(path_hint);
        if prev_hint.is_some() {
            let keep: Vec<&str> = {
                let mut ls: Vec<&str> = c.lines().collect();
                while ls.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
                    ls.pop();
                }
                ls.pop();
                ls
            };
            c = keep.join("\n");
            c.push('\n');
        }
        // Between documents of a ```yaml fence, `#` lines are YAML comments
        // (captions), not a Markdown body.
        if yaml_fence && c.lines().all(|l| l.trim().is_empty() || l.trim_start().starts_with('#')) {
            continue;
        }
        if let Some(last) = out.last_mut() {
            if !last.1.is_empty() {
                last.1.push_str("---\n");
            }
            last.1.push_str(&c);
        }
    }
    out
}

/// A path hint comment as the first line of a chunk: `# model/Foo/Bar.md`.
fn path_hint(fm: &str) -> Option<String> {
    let first = fm.lines().find(|l| !l.trim().is_empty())?.trim();
    let p = first.strip_prefix('#')?.trim();
    let p = p.split_whitespace().next()?;
    if !p.ends_with(".md") || p.contains("://") {
        return None;
    }
    Some(p.trim_start_matches("./").trim_start_matches("model/").to_string())
}

fn yaml_str(map: &serde_yaml::Mapping, key: &str) -> Option<String> {
    match map.get(serde_yaml::Value::String(key.into()))? {
        serde_yaml::Value::String(s) => Some(s.clone()),
        v @ (serde_yaml::Value::Number(_) | serde_yaml::Value::Bool(_)) => {
            Some(serde_yaml::to_string(v).ok()?.trim().to_string())
        }
        _ => None,
    }
}

fn element_type(name: &str) -> Option<ElementType> {
    ElementType::ALL.iter().find(|t| t.name() == name).cloned()
}

/// Derive a model-relative path for one element.
fn derive_path(map: &serde_yaml::Mapping, n: usize) -> String {
    let ty = yaml_str(map, "type").unwrap_or_default();
    let id = yaml_str(map, "id");
    let name = yaml_str(map, "name");
    let et = element_type(&ty);
    if matches!(ty.as_str(), "Package" | "LibraryPackage" | "Namespace") {
        let seg = name
            .filter(|s| is_basic_name(s))
            .or_else(|| id.clone().filter(|s| is_basic_name(s)))
            .unwrap_or_else(|| format!("Pkg{n}"));
        return format!("{seg}/_index.md");
    }
    if et.map(|t| t.is_id_identified()).unwrap_or(false) {
        if let Some(id) = id {
            if !id.is_empty() && !id.contains('/') {
                return format!("{id}.md");
            }
        }
        return format!("Example{n}.md");
    }
    if let Some(id) = &id {
        if id.contains("::") {
            let segs: Vec<&str> = id.split("::").collect();
            if segs.iter().all(|s| is_basic_name(s)) {
                return format!("{}.md", segs.join("/"));
            }
        }
    }
    if let Some(name) = name.filter(|s| !s.is_empty() && !s.contains('/')) {
        if is_basic_name(&name) {
            return format!("{name}.md");
        }
        // Keep the stem file-system safe but still visibly non-basic so W042
        // (a genuine doc defect for name-identified types) surfaces.
        let stem: String = name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ' ' { c } else { '_' })
            .collect();
        return format!("{stem}.md");
    }
    if let Some(id) = id.filter(|s| is_basic_name(s)) {
        return format!("{id}.md");
    }
    format!("Example{n}.md")
}

fn extract(root: &Path) -> Vec<Example> {
    let mut out = Vec::new();
    let mut counter = 0usize;
    for path in doc_files(root) {
        let rel = path.strip_prefix(root).unwrap().display().to_string();
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel}: {e}"));
        let lines: Vec<&str> = text.lines().collect();
        let mut section = String::from("(top)");
        let mut i = 0;
        let mut last_nonblank: Option<&str> = None;
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();
            let fence_ch = if trimmed.starts_with("```") {
                Some('`')
            } else if trimmed.starts_with("~~~") {
                Some('~')
            } else {
                None
            };
            let Some(fc) = fence_ch else {
                if line.starts_with('#') && line.trim_start_matches('#').starts_with(' ') {
                    section = line.trim().to_string();
                }
                if !line.trim().is_empty() {
                    last_nonblank = Some(line);
                }
                i += 1;
                continue;
            };
            let flen = trimmed.chars().take_while(|&c| c == fc).count();
            let lang = trimmed[flen..].trim().split_whitespace().next().unwrap_or("").to_lowercase();
            let open_line = i + 1;
            let marker = last_nonblank.and_then(parse_marker);
            // Collect the body.
            let mut body = Vec::new();
            // A line opening a fence *with an info string* inside a fence of
            // the same length is content to CommonMark, and the bare closer
            // that follows it ends the outer fence early — the rendered page
            // is broken. Remember it so an example that does this fails.
            let mut nested_fence: Option<usize> = None;
            i += 1;
            while i < lines.len() {
                let t = lines[i].trim();
                if t.starts_with(&fc.to_string().repeat(flen)) && t.chars().all(|c| c == fc) {
                    break;
                }
                let run = t.chars().take_while(|&c| c == fc).count();
                if run == flen && t[run..].starts_with(|c: char| c.is_ascii_alphabetic()) {
                    nested_fence.get_or_insert(i + 1);
                }
                let l = lines[i];
                let strip = l.len() - l.trim_start().len();
                body.push(&l[strip.min(indent)..]);
                i += 1;
            }
            i += 1; // closing fence
            last_nonblank = None;
            if !matches!(lang.as_str(), "" | "yaml" | "yml" | "markdown" | "md") {
                if marker.is_some() {
                    panic!("{rel}:{open_line}: syscribe-example marker on a non-YAML fence");
                }
                continue;
            }
            let content = body.join("\n");
            let parts = split_elements(&content, matches!(lang.as_str(), "yaml" | "yml"));
            if parts.is_empty() {
                if marker.is_some() {
                    panic!(
                        "{rel}:{open_line}: syscribe-example marker on a fence with no top-level `type:`"
                    );
                }
                continue;
            }
            let marker = marker.unwrap_or_default();
            let mut ex = Example {
                doc: rel.clone(),
                line: open_line,
                section: section.clone(),
                marker: marker.clone(),
                elements: Vec::new(),
                broken: nested_fence.map(|l| {
                    format!(
                        "fence nested at line {l} with the same length as its outer fence — \
                         lengthen the outer fence (e.g. ````markdown) so the page renders"
                    )
                }),
            };
            for (k, (fm, b, hint)) in parts.iter().enumerate() {
                if ex.broken.is_some() {
                    break;
                }
                counter += 1;
                let map = match serde_yaml::from_str::<serde_yaml::Value>(fm) {
                    Ok(serde_yaml::Value::Mapping(m)) => m,
                    Ok(_) => {
                        ex.broken = Some("frontmatter is not a YAML mapping".into());
                        break;
                    }
                    Err(e) => {
                        ex.broken = Some(format!("YAML does not parse: {e}"));
                        break;
                    }
                };
                let rel_path = if k == 0 && marker.path.is_some() {
                    marker.path.clone().unwrap()
                } else {
                    hint.clone().unwrap_or_else(|| derive_path(&map, counter))
                };
                ex.elements.push(ElemSrc {
                    frontmatter: fm.clone(),
                    body: b.clone(),
                    rel_path,
                });
            }
            out.push(ex);
        }
    }
    out
}

struct Group {
    dir: PathBuf,
    members: Vec<usize>,
    used: BTreeSet<String>,
    config: Option<String>,
}

fn run_validate(model: &Path) -> Vec<Finding> {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(model)
        .arg("validate")
        .arg("--json")
        .output()
        .expect("spawn syscribe");
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "validate --json on {} did not emit JSON ({e}):\nstdout: {stdout}\nstderr: {}",
            model.display(),
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

#[test]
fn documented_yaml_examples_validate() {
    let root = workspace_root();
    let examples = extract(&root);
    let verbose = std::env::var_os("SYSCRIBE_DOC_EXAMPLES_VERBOSE").is_some();
    let tolerated: BTreeSet<&str> = TOLERATED.iter().map(|(c, _)| *c).collect();
    let body_tolerated: BTreeSet<&str> = TOLERATED_WITHOUT_BODY.iter().map(|(c, _)| *c).collect();

    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("doc-examples");
    let _ = std::fs::remove_dir_all(&scratch);

    // Assign examples to scratch models.
    let mut groups: Vec<Group> = Vec::new();
    let mut open: BTreeMap<(String, String), usize> = BTreeMap::new();
    for (idx, ex) in examples.iter().enumerate() {
        if ex.marker.skip.is_some() || ex.broken.is_some() {
            continue;
        }
        let own = ex.marker.expect.is_some() || ex.marker.config.is_some();
        let key = (ex.doc.clone(), ex.section.clone());
        let fits = |g: &Group| ex.elements.iter().all(|e| !g.used.contains(&e.rel_path));
        let gi = match (own, open.get(&key)) {
            (false, Some(&gi)) if fits(&groups[gi]) => gi,
            _ => {
                let gi = groups.len();
                groups.push(Group {
                    dir: scratch.join(format!("g{gi:04}")).join("model"),
                    members: Vec::new(),
                    used: BTreeSet::new(),
                    config: ex.marker.config.clone(),
                });
                if !own {
                    open.insert(key, gi);
                }
                gi
            }
        };
        let g = &mut groups[gi];
        g.members.push(idx);
        for e in &ex.elements {
            g.used.insert(e.rel_path.clone());
        }
    }

    let mut failures: Vec<String> = Vec::new();
    for ex in &examples {
        if let Some(b) = &ex.broken {
            if ex.marker.skip.is_none() {
                failures.push(format!("{}:{} {} — {b}", ex.doc, ex.line, ex.section));
            }
        }
    }

    let mut checked: BTreeMap<String, usize> = BTreeMap::new();
    for g in &groups {
        std::fs::create_dir_all(&g.dir).unwrap();
        if let Some(cfg) = &g.config {
            let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/doc_examples").join(cfg);
            std::fs::copy(&src, g.dir.join(".syscribe.toml"))
                .unwrap_or_else(|e| panic!("config {}: {e}", src.display()));
        }
        // file -> (example index, body omitted in the snippet)
        let mut owner: BTreeMap<PathBuf, (usize, bool)> = BTreeMap::new();
        for &m in &g.members {
            for e in &examples[m].elements {
                let p = g.dir.join(&e.rel_path);
                std::fs::create_dir_all(p.parent().unwrap()).unwrap();
                let mut text = format!("---\n{}---\n", e.frontmatter);
                let omitted = e.body.trim().is_empty();
                if omitted {
                    text.push_str("\nExample body.\n");
                } else {
                    text.push_str(&e.body);
                }
                std::fs::write(&p, text).unwrap();
                owner.insert(p, (m, omitted));
            }
        }
        let findings = run_validate(&g.dir);
        let mut per: BTreeMap<usize, Vec<(&Finding, bool)>> = BTreeMap::new();
        for f in &findings {
            let who: Vec<(usize, bool)> =
                match f.file.as_ref().and_then(|p| owner.get(Path::new(p))) {
                    Some(&o) => vec![o],
                    None => g.members.iter().map(|&m| (m, false)).collect(),
                };
            for (m, omitted) in who {
                per.entry(m).or_default().push((f, omitted));
            }
        }
        for &m in &g.members {
            let ex = &examples[m];
            *checked.entry(ex.doc.clone()).or_default() += 1;
            let fs = per.get(&m).cloned().unwrap_or_default();
            let codes: BTreeSet<&str> = fs.iter().map(|(f, _)| f.code.as_str()).collect();
            let is_tolerated = |f: &Finding, omitted: bool| {
                tolerated.contains(f.code.as_str())
                    || (omitted && body_tolerated.contains(f.code.as_str()))
            };
            let mut problems = Vec::new();
            match &ex.marker.expect {
                Some(exp) => {
                    for c in exp {
                        if !codes.contains(c.as_str()) {
                            problems.push(format!("expected {c} did not fire"));
                        }
                    }
                    for &(f, omitted) in &fs {
                        if !is_tolerated(f, omitted) && !exp.contains(&f.code) {
                            problems.push(format!("{}: {}", f.code, f.message));
                        }
                    }
                }
                None => {
                    for &(f, omitted) in &fs {
                        if !is_tolerated(f, omitted) {
                            problems.push(format!("{}: {}", f.code, f.message));
                        }
                    }
                }
            }
            if verbose {
                eprintln!(
                    "{} {}:{} {} [{}]",
                    if problems.is_empty() { "ok  " } else { "FAIL" },
                    ex.doc,
                    ex.line,
                    ex.section,
                    codes.iter().copied().collect::<Vec<_>>().join(" ")
                );
            }
            if !problems.is_empty() {
                failures.push(format!(
                    "{}:{} {} (scratch {})\n    {}",
                    ex.doc,
                    ex.line,
                    ex.section,
                    g.dir.display(),
                    problems.join("\n    ")
                ));
            }
        }
    }

    let skipped = examples.iter().filter(|e| e.marker.skip.is_some()).count();
    eprintln!(
        "doc examples: {} found, {} skipped, {} checked in {} scratch models",
        examples.len(),
        skipped,
        checked.values().sum::<usize>(),
        groups.len()
    );
    for (doc, n) in &checked {
        eprintln!("  {n:4}  {doc}");
    }
    assert!(
        failures.is_empty(),
        "{} documented example(s) drifted from the tool:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}
