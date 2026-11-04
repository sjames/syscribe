//! `new feature | configuration | parameter` — scaffolding that creates a correct file the first time (GH #427,
//! `REQ-TRS-SCAFFOLD-001`). The common mistakes of a new author (`E201` no `FEAT-*` id, `E200` a malformed
//! `CONF-*`, `E751`/`E753` a wrong or earlier stage, `E680` a malformed range) are prevented before the file exists.
//!
//! [`plan`] is pure: it validates and produces the files to write. The CLI and the MCP `scaffold` tool each preview
//! the plan on a candidate copy through the guarded-write engine and commit only when it adds no error.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use syscribe_model::binding_time::BindingTime;
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::resolver::{is_conf_id, is_feat_id, Resolver};

pub type Fail = (i32, String);

/// One file to write: absolute path and full content.
#[derive(Debug, Clone)]
pub struct FileWrite {
    pub path: PathBuf,
    pub content: String,
    pub creates: bool,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub kind: &'static str,
    /// The id (or name) of what is created or extended.
    pub target: String,
    pub files: Vec<FileWrite>,
}

#[derive(Debug, Clone)]
pub enum Request {
    Feature { name: String, under: String, group: Option<String>, mandatory: bool, id: Option<String> },
    Configuration { id: String, name: Option<String>, from: Option<String>, stage: Option<String>, feature_model: Option<String>, under: Option<String>, select: Vec<(String, bool)> },
    Parameter { feature: String, name: String, ty: Option<String>, unit: Option<String>, range: Option<String>, enum_values: Vec<String>, default: Option<String>, required: bool, stage: Option<String>, latest: Option<String> },
}

fn basic_name(s: &str) -> bool {
    let mut c = s.chars();
    c.next().is_some_and(|f| f.is_ascii_alphabetic() || f == '_') && c.all(|x| x.is_ascii_alphanumeric() || x == '_')
}

fn is(e: &RawElement, t: ElementType) -> bool {
    e.frontmatter.element_type == Some(t)
}

fn find<'a>(elements: &'a [RawElement], key: &str) -> Option<&'a RawElement> {
    let r = Resolver::new(elements);
    r.resolve_ref(elements, key).or_else(|| r.get(elements, key))
}

fn model_dir_of(model_root: &Path, qname: &str) -> PathBuf {
    qname.split("::").fold(model_root.to_path_buf(), |p, s| p.join(s))
}

/// The directory a new feature file goes in, from its parent (a package, or a feature that is a group).
fn feature_dir(model_root: &Path, elements: &[RawElement], under: &str) -> Result<PathBuf, Fail> {
    match find(elements, under) {
        Some(p) if matches!(p.frontmatter.element_type, Some(ElementType::Package | ElementType::LibraryPackage | ElementType::Namespace)) => Ok(Path::new(&p.file_path).parent().map(Path::to_path_buf).unwrap_or_else(|| model_dir_of(model_root, &p.qualified_name))),
        Some(p) if is(p, ElementType::FeatureDef) => {
            if p.file_path.ends_with("_index.md") {
                Ok(Path::new(&p.file_path).parent().unwrap().to_path_buf())
            } else {
                Err((1, format!("{} is a leaf feature file ({}); a feature that groups others lives in <name>/_index.md — move it there first (`syscribe move`)", p.qualified_name, p.file_path)))
            }
        }
        Some(p) if is(p, ElementType::FeatureModel) => Err((1, format!("{} is a single-file feature model (`featureTree:`): add an entry to it, or use the MCP tool `edit_feature`", p.qualified_name))),
        Some(p) => Err((1, format!("{} is a {}, not a package or a group feature", p.qualified_name, crate::query::type_label(p.frontmatter.element_type.as_ref().unwrap_or(&ElementType::Unknown))))),
        None => {
            // A directory package with no `_index.md` of its own.
            let d = model_dir_of(model_root, under);
            if d.is_dir() {
                Ok(d)
            } else {
                Err((1, format!("'{under}' is not a package or group feature of this model")))
            }
        }
    }
}

/// A free `FEAT-*` id derived from the name (and, if taken, the names above it).
fn derive_feat_id(elements: &[RawElement], qname: &str) -> Option<String> {
    let segs: Vec<String> = qname.split("::").map(|s| s.chars().filter(|c| c.is_ascii_alphanumeric()).take(12).collect::<String>().to_uppercase()).collect();
    (1..=segs.len()).find_map(|take| {
        let id = format!("FEAT-{}", segs[segs.len() - take..].join("-"));
        (is_feat_id(&id) && !elements.iter().any(|e| e.frontmatter.id.as_deref() == Some(id.as_str()))).then_some(id)
    })
}

fn quote(s: &str) -> String {
    syscribe_model::frontmatter::yaml_scalar(s)
}

pub fn plan(model_root: &Path, elements: &[RawElement], req: &Request) -> Result<Plan, Fail> {
    match req {
        Request::Feature { name, under, group, mandatory, id } => {
            if !basic_name(name) {
                return Err((2, format!("'{name}' is not a basic name (letters, digits and `_`, not starting with a digit): rename it, e.g. `{}`", name.replace(['-', ' '], "_"))));
            }
            if let Some(g) = group {
                if !["optional", "alternative", "or"].contains(&g.as_str()) {
                    return Err((2, format!("group must be optional, alternative or or, not '{g}'")));
                }
            }
            let dir = feature_dir(model_root, elements, under)?;
            let path = dir.join(format!("{name}.md"));
            if path.exists() {
                return Err((1, format!("{} already exists", path.display())));
            }
            let parent_q = find(elements, under).map(|p| p.qualified_name.clone()).unwrap_or_else(|| under.to_string());
            let qname = format!("{parent_q}::{name}");
            if elements.iter().any(|e| e.qualified_name == qname) {
                return Err((1, format!("{qname} already exists")));
            }
            let id = match id {
                Some(i) if !is_feat_id(i) => return Err((2, format!("'{i}' is not a FEAT-* id (FEAT-, then segments of 2–12 capitals or digits)"))),
                Some(i) if elements.iter().any(|e| e.frontmatter.id.as_deref() == Some(i.as_str())) => return Err((1, format!("the id {i} is already used"))),
                Some(i) => i.clone(),
                None => derive_feat_id(elements, &qname).ok_or_else(|| (2, format!("no valid FEAT-* id can be derived from '{name}' (segments need 2–12 characters): give --id")))?,
            };
            let mut fm = format!("---\nid: {id}\ntype: FeatureDef\nname: {name}\n");
            if *mandatory {
                fm.push_str("mandatory: true\n");
            }
            if let Some(g) = group.as_ref().filter(|g| *g != "optional") {
                fm.push_str(&format!("groupKind: {g}\n"));
            }
            fm.push_str(&format!("---\n\n{name}: describe what this feature is and when a product selects it.\n"));
            Ok(Plan { kind: "feature", target: id, files: vec![FileWrite { path, content: fm, creates: true }] })
        }
        Request::Configuration { id, name, from, stage, feature_model, under, select } => {
            if !is_conf_id(id) {
                return Err((2, format!("'{id}' is not a CONF-* id (CONF-, then segments of 2–12 capitals or digits)")));
            }
            if elements.iter().any(|e| e.frontmatter.id.as_deref() == Some(id.as_str())) {
                return Err((1, format!("the id {id} is already used")));
            }
            let base = match from {
                Some(f) => Some(find(elements, f).filter(|e| is(e, ElementType::Configuration)).ok_or_else(|| (1, format!("Configuration not found: {f}")))?),
                None => None,
            };
            let stage_bt = match stage {
                Some(s) => Some(BindingTime::parse(s).ok_or_else(|| (2, format!("unknown stage '{s}'; the vocabulary is: {}", syscribe_model::binding_time::active().names_hint())))?),
                None => None,
            };
            if let (Some(b), Some(st)) = (base, stage_bt) {
                if let Some(bs) = b.frontmatter.binding_stage.as_deref().and_then(BindingTime::parse) {
                    if st < bs {
                        return Err((1, format!("a derived Configuration must bind at the same or a later stage than its base: {} is bound at {}, not {}", b.frontmatter.id.clone().unwrap_or_default(), bs.name(), st.name())));
                    }
                }
            }
            let fm_pkg = match (feature_model, base) {
                (Some(f), _) => f.clone(),
                (None, Some(b)) => b.frontmatter.feature_model.clone().ok_or_else(|| (2, "the base Configuration names no featureModel: give --feature-model".to_string()))?,
                (None, None) => {
                    let mut models: Vec<String> = elements.iter().filter(|e| is(e, ElementType::Configuration)).filter_map(|e| e.frontmatter.feature_model.clone()).collect();
                    models.sort();
                    models.dedup();
                    match models.as_slice() {
                        [one] => one.clone(),
                        _ => return Err((2, "give --feature-model <package> (or --from <Configuration>)".to_string())),
                    }
                }
            };
            let dir = match (under, base) {
                (Some(u), _) => feature_dir(model_root, elements, u).or_else(|_| find(elements, u).map(|p| Path::new(&p.file_path).parent().map(Path::to_path_buf).unwrap_or_default()).ok_or((1, format!("'{u}' is not a package"))))?,
                (None, Some(b)) => Path::new(&b.file_path).parent().map(Path::to_path_buf).unwrap_or_default(),
                (None, None) => elements.iter().find(|e| is(e, ElementType::Configuration)).and_then(|e| Path::new(&e.file_path).parent().map(Path::to_path_buf)).ok_or_else(|| (2, "give --under <package> for the new Configuration".to_string()))?,
            };
            let path = dir.join(format!("{id}.md"));
            if path.exists() {
                return Err((1, format!("{} already exists", path.display())));
            }
            let alias = syscribe_model::variability::feature_id_to_qname(elements);
            let mut fm = format!("---\ntype: Configuration\nid: {id}\nname: {}\nstatus: draft\nfeatureModel: {fm_pkg}\n", quote(name.as_deref().unwrap_or(id)));
            if let Some(st) = stage_bt {
                fm.push_str(&format!("bindingStage: {}\n", st.name()));
            }
            if let Some(b) = base {
                fm.push_str(&format!("derivedFrom: {}\n", b.frontmatter.id.clone().unwrap_or_else(|| b.qualified_name.clone())));
            }
            if !select.is_empty() {
                fm.push_str("features:\n");
                for (f, v) in select {
                    let q = syscribe_model::variability::canon_feature_ref(f, &alias);
                    if !elements.iter().any(|e| is(e, ElementType::FeatureDef) && e.qualified_name == q) {
                        return Err((1, format!("--select names no feature: {f}")));
                    }
                    fm.push_str(&format!("  {q}: {v}\n"));
                }
            }
            let body = match base {
                Some(b) => format!("Derived from {}{}.\n", b.frontmatter.id.clone().unwrap_or_default(), stage_bt.map(|s| format!(" at stage {}", s.name())).unwrap_or_default()),
                None => "Describe the product this Configuration selects.\n".to_string(),
            };
            Ok(Plan { kind: "configuration", target: id.clone(), files: vec![FileWrite { path, content: format!("{fm}---\n\n{body}"), creates: true }] })
        }
        Request::Parameter { feature, name, ty, unit, range, enum_values, default, required, stage, latest } => {
            if !basic_name(name) {
                return Err((2, format!("'{name}' is not a basic name (letters, digits and `_`)")));
            }
            let fd = find(elements, feature).filter(|e| is(e, ElementType::FeatureDef)).ok_or_else(|| (1, format!("FeatureDef not found: {feature}")))?;
            if fd.frontmatter.parameters.iter().flatten().any(|p| p.get("name").and_then(|n| n.as_str()) == Some(name.as_str())) {
                return Err((1, format!("{} already has a parameter '{name}'", fd.qualified_name)));
            }
            let ty_full = match ty.as_deref().unwrap_or("Real") {
                t @ ("Real" | "Integer" | "Boolean" | "String") => format!("ScalarValues::{t}"),
                t if t.starts_with("ScalarValues::") => t.to_string(),
                t => return Err((2, format!("type must be Real, Integer, Boolean or String, not '{t}'"))),
            };
            let bounds = match range {
                Some(r) => {
                    let (lo, hi) = r.split_once("..").and_then(|(a, b)| Some((a.trim().parse::<f64>().ok()?, b.trim().parse::<f64>().ok()?))).ok_or_else(|| (2, format!("range must be <lo>..<hi>, not '{r}'")))?;
                    if lo > hi {
                        return Err((2, format!("range {r} has its low end above its high end")));
                    }
                    Some((lo, hi))
                }
                None => None,
            };
            if !enum_values.is_empty() && (bounds.is_some() || ty_full != "ScalarValues::String") {
                return Err((2, "--enum goes with --type String and no --range".to_string()));
            }
            if let Some(d) = default {
                if !enum_values.is_empty() && !enum_values.contains(d) {
                    return Err((2, format!("default '{d}' is not one of the enumeration values")));
                }
                if let (Some((lo, hi)), Ok(v)) = (bounds, d.parse::<f64>()) {
                    if v < lo || v > hi {
                        return Err((2, format!("default {d} is outside the range {lo}..{hi}")));
                    }
                }
            }
            let parse_stage = |s: &Option<String>| -> Result<Option<BindingTime>, Fail> {
                match s {
                    Some(n) => BindingTime::parse(n).map(Some).ok_or_else(|| (2, format!("unknown stage '{n}'; the vocabulary is: {}", syscribe_model::binding_time::active().names_hint()))),
                    None => Ok(None),
                }
            };
            let (earliest, latest_bt) = (parse_stage(stage)?, parse_stage(latest)?);
            if let (Some(a), Some(b)) = (earliest, latest_bt) {
                if b < a {
                    return Err((2, format!("the latest stage {} is earlier than the earliest {}", b.name(), a.name())));
                }
            }
            let mut block = format!("  - name: {name}\n    type: {ty_full}\n");
            if let Some(u) = unit {
                block.push_str(&format!("    unit: {}\n", quote(u)));
            }
            if let Some(r) = range {
                block.push_str(&format!("    range: \"{}\"\n", r.trim()));
            }
            if !enum_values.is_empty() {
                block.push_str(&format!("    enumValues: [{}]\n", enum_values.iter().map(|v| quote(v)).collect::<Vec<_>>().join(", ")));
            }
            if let Some(d) = default {
                block.push_str(&format!("    default: {}\n", if d.parse::<f64>().is_ok() || d == "true" || d == "false" { d.clone() } else { quote(d) }));
            }
            if *required {
                block.push_str("    isRequired: true\n");
            }
            if let Some(s) = earliest {
                block.push_str(&format!("    bindingTime: {}\n", s.name()));
            }
            if let Some(s) = latest_bt {
                block.push_str(&format!("    latestBindingTime: {}\n", s.name()));
            }
            let original = std::fs::read_to_string(&fd.file_path).map_err(|e| (1, format!("cannot read {}: {e}", fd.file_path)))?;
            let content = append_parameter(&original, &block).ok_or_else(|| (1, format!("{} has no frontmatter, or an inline `parameters:` this command will not rewrite", fd.file_path)))?;
            Ok(Plan { kind: "parameter", target: format!("{}.{name}", fd.qualified_name), files: vec![FileWrite { path: PathBuf::from(&fd.file_path), content, creates: false }] })
        }
    }
}

/// Append a parameter block to the frontmatter's `parameters:` list (or add the section), touching nothing else.
fn append_parameter(content: &str, block: &str) -> Option<String> {
    let (yaml, _) = syscribe_model::frontmatter::split_frontmatter(content);
    let yaml = yaml?;
    let lines: Vec<&str> = yaml.lines().collect();
    let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    let block = block.trim_end_matches('\n');
    match lines.iter().position(|l| l.starts_with("parameters:")) {
        None => {
            out.push("parameters:".to_string());
            out.extend(block.lines().map(str::to_string));
        }
        Some(i) => {
            if !lines[i]["parameters:".len()..].trim().is_empty() {
                return None; // inline `parameters: []` and the like
            }
            let end = lines[i + 1..].iter().position(|l| !l.is_empty() && !l.starts_with(char::is_whitespace)).map_or(lines.len(), |n| i + 1 + n);
            let last = (i + 1..end).rev().find(|&j| !lines[j].trim().is_empty()).unwrap_or(i);
            for (k, l) in block.lines().enumerate() {
                out.insert(last + 1 + k, l.to_string());
            }
        }
    }
    Some(syscribe_model::frontmatter::splice_frontmatter(content, yaml, &out.join("\n")))
}

/// The report of a scaffold, from its plan and the guarded-write preview.
pub fn report(plan: &Plan, outcome: &syscribe_model::mutate::guard::GuardedWriteOutcome, written: bool, refused: Option<&str>, model_root: &Path) -> Value {
    let uniq = |v: &[(String, String, String)]| -> Vec<Value> {
        let mut seen = std::collections::HashSet::new();
        v.iter().filter(|e| seen.insert((e.0.clone(), e.2.clone()))).map(|(c, f, m)| json!({ "code": c, "file": f, "message": m })).collect()
    };
    json!({
        "schemaVersion": "1.0",
        "kind": plan.kind,
        "target": plan.target,
        "files": plan.files.iter().map(|f| json!({ "path": f.path.strip_prefix(model_root).unwrap_or(&f.path).to_string_lossy(), "creates": f.creates })).collect::<Vec<_>>(),
        "written": written,
        "refused": refused,
        "diff": outcome.diff,
        "newErrors": uniq(&outcome.new_errors),
        "newWarnings": uniq(&outcome.new_warnings),
        "resolvedErrors": uniq(&outcome.resolved_errors),
        "resolvedWarnings": uniq(&outcome.resolved_warnings),
    })
}

const USAGE: &str = "Usage: syscribe -m <root> new feature <Name> --under <package|group> [--group optional|alternative|or] [--mandatory] [--id FEAT-X]\n       syscribe -m <root> new configuration <CONF-ID> [--from <CONF>] [--stage <stage>] [--feature-model <pkg>] [--under <pkg>] [--name <text>] [--select <feature>=<bool>]...\n       syscribe -m <root> new parameter <feature> <name> [--type Real|Integer|Boolean|String] [--unit <u>] [--range lo..hi] [--enum a,b] [--default <v>] [--required] [--stage <s>] [--latest <s>]   (add --dry-run to preview, --json for the report)";

/// Parse the tokens after `new`.
pub fn parse_args(args: &[String]) -> Result<(Request, bool), String> {
    let json = args.iter().any(|a| a == "--json");
    let sub = args.first().map(String::as_str).ok_or_else(|| USAGE.to_string())?;
    let mut pos: Vec<String> = Vec::new();
    let mut opts: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut flags: std::collections::HashSet<String> = std::collections::HashSet::new();
    let takes_value = ["--under", "--group", "--id", "--from", "--stage", "--feature-model", "--name", "--select", "--type", "--unit", "--range", "--enum", "--default", "--latest"];
    let mut it = args.iter().skip(1);
    while let Some(a) = it.next() {
        if a == "--json" || a == "--dry-run" {
            continue;
        }
        if takes_value.contains(&a.as_str()) {
            let v = it.next().ok_or_else(|| format!("new: {a} needs a value"))?;
            opts.entry(a.clone()).or_default().push(v.clone());
        } else if a.starts_with("--") {
            flags.insert(a.clone());
        } else {
            pos.push(a.clone());
        }
    }
    let one = |k: &str| opts.get(k).and_then(|v| v.last()).cloned();
    let unknown = |allowed: &[&str]| -> Result<(), String> {
        for f in flags.iter().chain(opts.keys().map(|k| k).collect::<Vec<_>>().into_iter()) {
            if !allowed.contains(&f.as_str()) {
                return Err(format!("new {sub}: unknown option '{f}'"));
            }
        }
        Ok(())
    };
    let req = match sub {
        "feature" => {
            unknown(&["--under", "--group", "--id", "--mandatory"])?;
            let [name] = pos.as_slice() else { return Err(USAGE.to_string()) };
            Request::Feature { name: name.clone(), under: one("--under").ok_or("new feature: --under <package|group> is required")?, group: one("--group"), mandatory: flags.contains("--mandatory"), id: one("--id") }
        }
        "configuration" => {
            unknown(&["--from", "--stage", "--feature-model", "--under", "--name", "--select"])?;
            let [id] = pos.as_slice() else { return Err(USAGE.to_string()) };
            let select = opts.get("--select").into_iter().flatten().map(|s| crate::propagate::parse_choice(s).map_err(|m| format!("new configuration: --select {m}"))).collect::<Result<Vec<_>, _>>()?;
            Request::Configuration { id: id.clone(), name: one("--name"), from: one("--from"), stage: one("--stage"), feature_model: one("--feature-model"), under: one("--under"), select }
        }
        "parameter" => {
            unknown(&["--type", "--unit", "--range", "--enum", "--default", "--required", "--stage", "--latest"])?;
            let [feature, name] = pos.as_slice() else { return Err(USAGE.to_string()) };
            Request::Parameter {
                feature: feature.clone(),
                name: name.clone(),
                ty: one("--type"),
                unit: one("--unit"),
                range: one("--range"),
                enum_values: one("--enum").map(|e| e.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()).unwrap_or_default(),
                default: one("--default"),
                required: flags.contains("--required"),
                stage: one("--stage"),
                latest: one("--latest"),
            }
        }
        other => return Err(format!("new: unknown kind '{other}'; use feature, configuration or parameter\n{USAGE}")),
    };
    Ok((req, json))
}

pub fn cmd_new(model_root: &Path, elements: &[RawElement], config: &syscribe_model::config::ValidateConfig, args: &[String]) -> i32 {
    let (req, json) = match parse_args(args) {
        Ok(p) => p,
        Err(m) => {
            eprintln!("{m}");
            return 2;
        }
    };
    let plan = match plan(model_root, elements, &req) {
        Ok(p) => p,
        Err((c, m)) => {
            eprintln!("{m}");
            return c;
        }
    };
    for f in &plan.files {
        if let Some(e) = elements.iter().find(|e| Path::new(&e.file_path) == f.path) {
            crate::provenance::check_expect_hash(&f.path);
            crate::policy_gate::check_path(e);
        }
    }
    let files: Vec<(PathBuf, String)> = plan.files.iter().map(|f| (f.path.clone(), f.content.clone())).collect();
    let outcome = match crate::preview::preview_files(model_root, elements, config, &files) {
        Ok(o) => o,
        Err(r) => {
            eprintln!("{r}");
            return 1;
        }
    };
    let dry = syscribe_model::fsio::is_dry_run();
    let n_new = crate::preview::new_error_count(&outcome);
    let refused = (n_new > 0).then(|| format!("refused: it would introduce {n_new} error(s)"));
    let mut written = false;
    if refused.is_none() && !dry {
        for f in &plan.files {
            if let Some(d) = f.path.parent() {
                if let Err(e) = syscribe_model::fsio::create_dir_all(d) {
                    eprintln!("Cannot create {}: {e}", d.display());
                    return 1;
                }
            }
            if let Err(e) = syscribe_model::fsio::write(&f.path, &f.content) {
                eprintln!("Write failed for {}: {e}", f.path.display());
                return 1;
            }
        }
        if !json {
            crate::provenance::record_diff(&outcome.diff);
        }
        written = true;
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&report(&plan, &outcome, written, refused.as_deref(), model_root)).unwrap());
    } else {
        println!("# New {} {}\n", plan.kind, plan.target);
        for f in &plan.files {
            println!("- {} {}", if f.creates { "create" } else { "extend" }, f.path.strip_prefix(model_root).unwrap_or(&f.path).display());
        }
        let mut seen = std::collections::HashSet::new();
        for e in outcome.new_errors.iter().filter(|e| seen.insert((e.0.clone(), e.2.clone()))) {
            println!("- new error {}: {}", e.0, e.2);
        }
        let mut seen = std::collections::HashSet::new();
        for w in outcome.new_warnings.iter().filter(|w| seen.insert((w.0.clone(), w.2.clone()))) {
            println!("- new warning {}: {}", w.0, w.2);
        }
        match (&refused, written, dry) {
            (Some(r), ..) => println!("\n{r}. Nothing was written."),
            (None, true, _) => println!("\nWritten."),
            (None, false, true) => println!("\nDry run: nothing was written."),
            _ => {}
        }
    }
    if refused.is_some() {
        1
    } else {
        0
    }
}
