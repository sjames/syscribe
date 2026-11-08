//! Code that no design element or TestCase names (GH #451, `REQ-TRS-TRACECODE-001..004`).
//!
//! `implementedBy:` and `sourceFile:` say which code realises or verifies an element; this module
//! answers the reverse question. A `[traceCode]` table in `<model_root>/.syscribe.toml` names the
//! source roots and how a *code unit* is recognised under them; every unit that no `implementedBy:`
//! or `sourceFile:` names is *untraced*. A baseline file lists the known untraced units with a
//! reason, and only shrinks: a new untraced unit, or a baseline entry that is now traced or gone, is
//! a failure (`syscribe trace-code` exits 2; `[traceCode] validate = true` adds `W326`/`W327`).
//!
//! ```toml
//! [traceCode]
//! roots = ["crates", "scripts", "tests"]        # repository-relative
//! exclude = ["crates/*/vendor/**"]              # globs; target/, node_modules/, .git/ always
//! baseline = "trace-code-baseline.toml"         # relative to the model root (the default)
//! validate = false                              # W326/W327 in `validate`
//! build_files = ["Cargo.toml", "package.json"]  # what makes a `build-dir` (default list below)
//! units = [
//!   { kind = "cargo-crate", under = "crates" },
//!   { kind = "file", under = "scripts", extensions = ["sh", "py"] },
//!   { kind = "file", under = "tests", extensions = ["rs"] },
//! ]
//! ```
//!
//! Paths in `implementedBy:`/`sourceFile:` are resolved with [`ValidateConfig::classify_source`],
//! exactly as `W023` resolves them (`repo:`/`model:` prefixes, `file://`, absolute, otherwise
//! model-root-relative; remote URIs and package-registry references are external and name no
//! local code).

use crate::config::{lenient_canonical, SourceLocation, ValidateConfig};
use crate::element::RawElement;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Build files that make a directory a `build-dir` unit when `build_files` is not set.
pub const DEFAULT_BUILD_FILES: &[&str] = &["Cargo.toml", "package.json", "CMakeLists.txt", "Makefile", "go.mod", "pyproject.toml", "setup.py"];

/// Directory names never walked, wherever they occur.
pub const ALWAYS_EXCLUDED: &[&str] = &["target", "node_modules", ".git"];

/// The default baseline file, relative to the model root.
pub const DEFAULT_BASELINE: &str = "trace-code-baseline.toml";

/// How a code unit is recognised.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitKind {
    /// A directory holding a `Cargo.toml` with a `[package]` table.
    CargoCrate,
    /// A directory holding any of the configured build files.
    BuildDir,
    /// A single file, filtered by extension.
    File,
}

impl UnitKind {
    pub fn as_str(self) -> &'static str {
        match self {
            UnitKind::CargoCrate => "cargo-crate",
            UnitKind::BuildDir => "build-dir",
            UnitKind::File => "file",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        match s {
            "cargo-crate" => Some(UnitKind::CargoCrate),
            "build-dir" => Some(UnitKind::BuildDir),
            "file" => Some(UnitKind::File),
            _ => None,
        }
    }
}

/// One `units` rule: a kind, optionally limited to a repository-relative subtree and (for `file`) to
/// some extensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitRule {
    pub kind: UnitKind,
    pub under: Option<String>,
    /// Lower-case, without the dot. Empty: every file.
    pub extensions: Vec<String>,
}

impl UnitRule {
    fn applies_to(&self, rel: &str) -> bool {
        match &self.under {
            None => true,
            Some(u) => u.is_empty() || rel == u || rel.starts_with(&format!("{u}/")),
        }
    }
}

/// The `[traceCode]` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCodeConfig {
    pub roots: Vec<String>,
    pub rules: Vec<UnitRule>,
    pub build_files: Vec<String>,
    pub exclude: Vec<String>,
    /// Relative to the model root.
    pub baseline: String,
    pub validate: bool,
    /// Malformed entries, each ignored (`W328`).
    pub problems: Vec<String>,
}

impl Default for TraceCodeConfig {
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            rules: vec![UnitRule { kind: UnitKind::BuildDir, under: None, extensions: Vec::new() }],
            build_files: DEFAULT_BUILD_FILES.iter().map(|s| s.to_string()).collect(),
            exclude: Vec::new(),
            baseline: DEFAULT_BASELINE.to_string(),
            validate: false,
            problems: Vec::new(),
        }
    }
}

/// A repository-relative path written by a user: `/` separators, no leading `./` or trailing `/`.
fn norm_rel(s: &str) -> String {
    let s = s.trim().replace('\\', "/");
    let s = s.trim_start_matches("./").trim_matches('/');
    s.split('/').filter(|p| !p.is_empty() && *p != ".").collect::<Vec<_>>().join("/")
}

fn string_list(v: &toml::Value, key: &str, problems: &mut Vec<String>) -> Option<Vec<String>> {
    match v.as_array() {
        Some(a) => {
            let mut out = Vec::new();
            for item in a {
                match item.as_str() {
                    Some(s) if !s.trim().is_empty() => out.push(s.trim().to_string()),
                    _ => problems.push(format!("[traceCode] {key}: entry {item} is not a non-empty string (ignored)")),
                }
            }
            Some(out)
        }
        None => {
            problems.push(format!("[traceCode] {key} must be a list of strings, got {v}"));
            None
        }
    }
}

fn parse_rule(v: &toml::Value, problems: &mut Vec<String>) -> Option<UnitRule> {
    const KINDS: &str = "cargo-crate, build-dir or file";
    if let Some(s) = v.as_str() {
        return match UnitKind::parse(s) {
            Some(kind) => Some(UnitRule { kind, under: None, extensions: Vec::new() }),
            None => {
                problems.push(format!("[traceCode] units: unknown kind '{s}' (expected {KINDS}; ignored)"));
                None
            }
        };
    }
    let Some(t) = v.as_table() else {
        problems.push(format!("[traceCode] units: entry {v} is neither a kind nor a {{ kind, under?, extensions? }} table (ignored)"));
        return None;
    };
    let mut kind = None;
    let mut under = None;
    let mut extensions = Vec::new();
    for (k, val) in t {
        match k.as_str() {
            "kind" => match val.as_str() {
                Some(s) => match UnitKind::parse(s) {
                    Some(x) => kind = Some(x),
                    None => {
                        problems.push(format!("[traceCode] units: unknown kind '{s}' (expected {KINDS}; entry ignored)"));
                        return None;
                    }
                },
                None => {
                    problems.push(format!("[traceCode] units: kind must be a string, got {val} (entry ignored)"));
                    return None;
                }
            },
            "under" => match val.as_str() {
                Some(s) => under = Some(norm_rel(s)),
                None => {
                    problems.push(format!("[traceCode] units: under must be a string, got {val} (entry ignored)"));
                    return None;
                }
            },
            "extensions" => match string_list(val, "units.extensions", problems) {
                Some(list) => extensions = list.iter().map(|e| e.trim_start_matches('.').to_ascii_lowercase()).collect(),
                None => return None,
            },
            other => problems.push(format!("[traceCode] units: unknown key '{other}' (known: kind, under, extensions; key ignored)")),
        }
    }
    let Some(kind) = kind else {
        problems.push("[traceCode] units: an entry has no kind (entry ignored)".to_string());
        return None;
    };
    if kind != UnitKind::File && !extensions.is_empty() {
        problems.push(format!("[traceCode] units: extensions apply only to kind = \"file\", not '{}' (ignored)", kind.as_str()));
        extensions.clear();
    }
    Some(UnitRule { kind, under, extensions })
}

impl TraceCodeConfig {
    /// `None` when `<model_root>/.syscribe.toml` has no `[traceCode]` table (the feature is off).
    pub fn load(model_root: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(model_root.join(".syscribe.toml")).ok()?;
        let root = toml::from_str::<toml::Value>(&text).ok()?;
        let v = root.get("traceCode").or_else(|| root.get("trace_code"))?;
        Some(Self::from_value(v))
    }

    fn from_value(v: &toml::Value) -> Self {
        let mut c = TraceCodeConfig::default();
        let Some(t) = v.as_table() else {
            c.problems.push("[traceCode] must be a table".to_string());
            return c;
        };
        let mut problems = Vec::new();
        for (k, val) in t {
            match k.as_str() {
                "roots" => {
                    if let Some(list) = string_list(val, "roots", &mut problems) {
                        c.roots = list.iter().map(|s| norm_rel(s)).collect();
                    }
                }
                "units" => {
                    let entries: Vec<toml::Value> = match val {
                        toml::Value::Array(a) => a.clone(),
                        other => vec![other.clone()],
                    };
                    let rules: Vec<UnitRule> = entries.iter().filter_map(|e| parse_rule(e, &mut problems)).collect();
                    if !rules.is_empty() {
                        c.rules = rules;
                    }
                }
                "exclude" => {
                    if let Some(list) = string_list(val, "exclude", &mut problems) {
                        c.exclude = list.iter().map(|s| s.trim_start_matches("./").to_string()).collect();
                    }
                }
                "build_files" | "buildFiles" => {
                    if let Some(list) = string_list(val, k, &mut problems) {
                        c.build_files = list;
                    }
                }
                "baseline" => match val.as_str() {
                    Some(s) if !s.trim().is_empty() => c.baseline = s.trim().to_string(),
                    _ => problems.push(format!("[traceCode] baseline must be a non-empty string (a path relative to the model root), got {val}")),
                },
                "validate" => match val.as_bool() {
                    Some(b) => c.validate = b,
                    None => problems.push(format!("[traceCode] validate must be true or false, got {val}")),
                },
                other => problems.push(format!(
                    "[traceCode] unknown key '{other}' (known: roots, units, exclude, build_files, baseline, validate; ignored)"
                )),
            }
        }
        if c.roots.is_empty() {
            problems.push("[traceCode] roots is missing or empty: name the repository-relative source directories, e.g. roots = [\"crates\", \"scripts\"]".to_string());
        }
        c.problems = problems;
        c
    }
}

/// A code unit found under the roots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    /// Repository-relative, `/`-separated.
    pub path: String,
    pub kind: UnitKind,
}

fn is_cargo_crate(dir: &Path) -> bool {
    std::fs::read_to_string(dir.join("Cargo.toml"))
        .ok()
        .and_then(|t| toml::from_str::<toml::Value>(&t).ok())
        .is_some_and(|v| v.get("package").is_some_and(toml::Value::is_table))
}

fn excluded(globs: &[String], rel: &str, is_dir: bool) -> bool {
    globs.iter().any(|g| {
        let g = g.trim_end_matches('/');
        crate::annotations::glob_match(g, rel)
            || (is_dir && crate::annotations::glob_match(g, &format!("{rel}/")))
            || (!g.contains('/') && rel.rsplit('/').next().is_some_and(|base| crate::annotations::glob_match(g, base)))
    })
}

/// Every code unit under the configured roots of `repo_root`. Roots that do not exist are returned as
/// problems. Units do not nest: below a directory unit nothing else is a unit.
pub fn discover_units(repo_root: &Path, cfg: &TraceCodeConfig) -> (Vec<Unit>, Vec<String>) {
    let mut units = Vec::new();
    let mut problems = Vec::new();
    for root in &cfg.roots {
        let dir = if root.is_empty() { repo_root.to_path_buf() } else { repo_root.join(root) };
        if !dir.is_dir() {
            problems.push(format!("[traceCode] roots: '{root}' is not a directory under {}", repo_root.display()));
            continue;
        }
        walk(&dir, root, cfg, &mut units);
    }
    units.sort_by(|a, b| a.path.cmp(&b.path));
    units.dedup_by(|a, b| a.path == b.path);
    (units, problems)
}

fn walk(dir: &Path, rel: &str, cfg: &TraceCodeConfig, out: &mut Vec<Unit>) {
    if !rel.is_empty() && excluded(&cfg.exclude, rel, true) {
        return;
    }
    // Is this directory itself a unit?
    for rule in cfg.rules.iter().filter(|r| r.kind != UnitKind::File && r.applies_to(rel)) {
        let hit = match rule.kind {
            UnitKind::CargoCrate => is_cargo_crate(dir),
            UnitKind::BuildDir => cfg.build_files.iter().any(|f| dir.join(f).is_file()),
            UnitKind::File => false,
        };
        if hit {
            out.push(Unit { path: rel.to_string(), kind: rule.kind });
            return;
        }
    }
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let name = e.file_name().to_string_lossy().into_owned();
        let child = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() {
            if ALWAYS_EXCLUDED.contains(&name.as_str()) {
                continue;
            }
            walk(&e.path(), &child, cfg, out);
        } else if ft.is_file() {
            if excluded(&cfg.exclude, &child, false) {
                continue;
            }
            let ext = Path::new(&name).extension().map(|x| x.to_string_lossy().to_ascii_lowercase());
            let is_unit = cfg.rules.iter().filter(|r| r.kind == UnitKind::File && r.applies_to(&child)).any(|r| {
                r.extensions.is_empty() || ext.as_deref().is_some_and(|x| r.extensions.iter().any(|w| w == x))
            });
            if is_unit {
                out.push(Unit { path: child, kind: UnitKind::File });
            }
        }
    }
}

/// A local path an element names, relative to the repository root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedPath {
    /// Repository-relative; empty when the entry names the repository root itself.
    pub rel: String,
    /// The element's id, or its qualified name.
    pub element: String,
    /// `implementedBy` or `sourceFile`.
    pub via: &'static str,
}

/// `p` relative to `repo_root` (both made canonical), `/`-separated; `None` outside it.
fn rel_to(repo_root: &Path, p: &Path) -> Option<String> {
    lenient_canonical(p).strip_prefix(repo_root).ok().map(|r| r.to_string_lossy().replace('\\', "/").trim_end_matches('/').to_string())
}

/// Every local path that `elements` name through `implementedBy:` or `sourceFile:`, resolved like
/// `W023` and made relative to `repo_root` (paths outside it, remote URIs and package references are
/// skipped).
pub fn named_paths(elements: &[RawElement], config: &ValidateConfig, repo_root: &Path) -> Vec<NamedPath> {
    let repo_root = lenient_canonical(repo_root);
    let mut out = Vec::new();
    for e in elements {
        let label = e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone());
        let mut add = |value: &str, via: &'static str| {
            if let SourceLocation::Local(p) = config.classify_source(value) {
                if let Some(rel) = rel_to(&repo_root, &p) {
                    out.push(NamedPath { rel, element: label.clone(), via });
                }
            }
        };
        for v in e.frontmatter.implemented_by.as_deref().unwrap_or(&[]) {
            add(v, "implementedBy");
        }
        if let Some(v) = &e.frontmatter.source_file {
            add(v, "sourceFile");
        }
    }
    out
}

/// The containment rule: `named` names `unit` when it is the unit itself, a path inside it, or a
/// directory containing it (the repository root contains everything).
pub fn names(named: &str, unit: &str) -> bool {
    named.is_empty() || named == unit || named.starts_with(&format!("{unit}/")) || unit.starts_with(&format!("{named}/"))
}

/// One entry of the baseline file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineEntry {
    pub path: String,
    pub reason: String,
}

/// Read a baseline file: `[[untraced]]` tables with `path` and `reason`. A missing file is an empty
/// baseline; anything malformed is an error naming the file.
pub fn load_baseline(path: &Path) -> Result<Vec<BaselineEntry>, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("cannot read {}: {e}", path.display())),
    };
    let v: toml::Value = toml::from_str(&text).map_err(|e| format!("{} is not valid TOML: {e}", path.display()))?;
    let Some(t) = v.as_table() else { return Err(format!("{} must be a TOML table", path.display())) };
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for (k, val) in t {
        if k != "untraced" {
            return Err(format!("{}: unknown key '{k}' (expected [[untraced]] entries)", path.display()));
        }
        let Some(arr) = val.as_array() else { return Err(format!("{}: untraced must be an array of tables ([[untraced]])", path.display())) };
        for (i, item) in arr.iter().enumerate() {
            let n = i + 1;
            let Some(it) = item.as_table() else { return Err(format!("{}: untraced entry {n} is not a table", path.display())) };
            let Some(p) = it.get("path").and_then(|p| p.as_str()).map(norm_rel).filter(|p| !p.is_empty()) else {
                return Err(format!("{}: untraced entry {n} has no path", path.display()));
            };
            let reason = match it.get("reason") {
                None => String::new(),
                Some(r) => r.as_str().map(str::to_string).ok_or_else(|| format!("{}: untraced entry {n} ({p}): reason must be a string", path.display()))?,
            };
            if let Some(other) = it.keys().find(|k| *k != "path" && *k != "reason") {
                return Err(format!("{}: untraced entry {n} ({p}): unknown key '{other}' (expected path, reason)", path.display()));
            }
            if !seen.insert(p.clone()) {
                return Err(format!("{}: '{p}' is listed twice", path.display()));
            }
            out.push(BaselineEntry { path: p, reason });
        }
    }
    Ok(out)
}

fn toml_str(s: &str) -> String {
    toml::Value::String(s.to_string()).to_string()
}

/// The baseline file's text for `entries` (sorted by path).
pub fn render_baseline(entries: &[BaselineEntry]) -> String {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| a.path.cmp(&b.path));
    let mut o = String::from(
        "# Code units that no implementedBy: or sourceFile: names yet (GH #451).\n\
         # `syscribe trace-code` fails on an untraced unit missing here and on an entry here that is\n\
         # now traced or gone, so this list only shrinks. Refresh with `syscribe trace-code --write-baseline`\n\
         # (it keeps each reason); replace every TODO with the real reason.\n",
    );
    for e in &sorted {
        o.push_str(&format!("\n[[untraced]]\npath = {}\nreason = {}\n", toml_str(&e.path), toml_str(&e.reason)));
    }
    o
}

/// An untraced unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Untraced {
    pub path: String,
    pub kind: UnitKind,
    pub in_baseline: bool,
}

/// A traced unit and what names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Traced {
    pub path: String,
    pub kind: UnitKind,
    /// `(element, via)`, sorted, de-duplicated.
    pub named_by: Vec<(String, &'static str)>,
}

/// A baseline entry that no longer describes an untraced unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stale {
    pub path: String,
    pub reason: String,
    /// `traced` (a unit some element now names), `missing` (the path is gone) or `not-a-unit` (it
    /// exists but is no unit under the current rules, e.g. excluded).
    pub why: &'static str,
}

/// The result for one repository.
#[derive(Debug, Clone)]
pub struct Report {
    pub repo_root: PathBuf,
    pub baseline_path: PathBuf,
    pub untraced: Vec<Untraced>,
    pub traced: Vec<Traced>,
    pub stale: Vec<Stale>,
    pub baseline: Vec<BaselineEntry>,
}

impl Report {
    /// New untraced units (not in the baseline).
    pub fn new_untraced(&self) -> impl Iterator<Item = &Untraced> {
        self.untraced.iter().filter(|u| !u.in_baseline)
    }
    /// Whether the report passes: nothing new, nothing stale.
    pub fn clean(&self) -> bool {
        self.new_untraced().next().is_none() && self.stale.is_empty()
    }
    /// The baseline `--write-baseline` writes: every untraced unit, keeping existing reasons and giving
    /// new entries `TODO`.
    pub fn refreshed_baseline(&self) -> Vec<BaselineEntry> {
        let old: BTreeMap<&str, &str> = self.baseline.iter().map(|e| (e.path.as_str(), e.reason.as_str())).collect();
        self.untraced
            .iter()
            .map(|u| BaselineEntry { path: u.path.clone(), reason: old.get(u.path.as_str()).map(|r| r.to_string()).unwrap_or_else(|| "TODO".to_string()) })
            .collect()
    }
}

/// The repository whose code `model_root`'s model describes: `override_root`, else the repository root
/// `repo:` paths resolve against (`repo_root` in `.syscribe.toml`, else the git work tree), else the
/// model root's parent.
pub fn repo_root_for(model_root: &Path, config: &ValidateConfig, override_root: Option<&Path>) -> PathBuf {
    let r = override_root
        .map(Path::to_path_buf)
        .or_else(|| config.repo_root.clone())
        .unwrap_or_else(|| lenient_canonical(model_root).parent().map(Path::to_path_buf).unwrap_or_else(|| model_root.to_path_buf()));
    lenient_canonical(&r)
}

/// Analyse one repository. `Err` when the baseline cannot be read. Configuration problems are the
/// caller's to report (`cfg.problems`, plus the returned root problems).
pub fn analyse(
    model_root: &Path,
    elements: &[RawElement],
    config: &ValidateConfig,
    cfg: &TraceCodeConfig,
    repo_root: &Path,
) -> Result<(Report, Vec<String>), String> {
    let baseline_path = model_root.join(&cfg.baseline);
    let baseline = load_baseline(&baseline_path)?;
    let (units, root_problems) = discover_units(repo_root, cfg);
    let named = named_paths(elements, config, repo_root);
    let in_baseline: BTreeSet<&str> = baseline.iter().map(|e| e.path.as_str()).collect();

    let mut untraced = Vec::new();
    let mut traced = Vec::new();
    for u in &units {
        let mut by: Vec<(String, &'static str)> = named.iter().filter(|n| names(&n.rel, &u.path)).map(|n| (n.element.clone(), n.via)).collect();
        by.sort();
        by.dedup();
        if by.is_empty() {
            untraced.push(Untraced { path: u.path.clone(), kind: u.kind, in_baseline: in_baseline.contains(u.path.as_str()) });
        } else {
            traced.push(Traced { path: u.path.clone(), kind: u.kind, named_by: by });
        }
    }
    let untraced_set: BTreeSet<&str> = untraced.iter().map(|u| u.path.as_str()).collect();
    let traced_set: BTreeSet<&str> = traced.iter().map(|u| u.path.as_str()).collect();
    let stale = baseline
        .iter()
        .filter(|e| !untraced_set.contains(e.path.as_str()))
        .map(|e| Stale {
            path: e.path.clone(),
            reason: e.reason.clone(),
            why: if traced_set.contains(e.path.as_str()) {
                "traced"
            } else if !repo_root.join(&e.path).exists() {
                "missing"
            } else {
                "not-a-unit"
            },
        })
        .collect();
    Ok((Report { repo_root: repo_root.to_path_buf(), baseline_path, untraced, traced, stale, baseline }, root_problems))
}

/// The `validate` findings for `[traceCode]`, as `(code, file, message)`: `W328` for every
/// configuration problem (whenever the table is present), and — with `validate = true` — `W326` for
/// each new untraced unit and `W327` for each stale baseline entry. Empty without the table.
pub fn validate_findings(model_root: &Path, elements: &[RawElement], config: &ValidateConfig) -> Vec<(&'static str, String, String)> {
    let Some(cfg) = TraceCodeConfig::load(model_root) else { return Vec::new() };
    let cfg_file = model_root.join(".syscribe.toml").display().to_string();
    let mut out: Vec<(&'static str, String, String)> = cfg.problems.iter().map(|p| ("W328", cfg_file.clone(), format!("{p} — see `syscribe help trace-code`"))).collect();
    if !cfg.validate {
        return out;
    }
    let repo_root = repo_root_for(model_root, config, None);
    match analyse(model_root, elements, config, &cfg, &repo_root) {
        Err(e) => out.push(("W328", cfg_file.clone(), format!("[traceCode] baseline: {e}"))),
        Ok((report, root_problems)) => {
            for p in root_problems {
                out.push(("W328", cfg_file.clone(), p));
            }
            let bl = report.baseline_path.display().to_string();
            for u in report.new_untraced() {
                out.push((
                    "W326",
                    cfg_file.clone(),
                    format!(
                        "code unit '{}' ({}) is named by no implementedBy: or sourceFile: and is not in the trace-code baseline — add the design element (implementedBy:) or TestCase (sourceFile:) that names it, or baseline it with a reason (`syscribe trace-code --write-baseline`)",
                        u.path,
                        u.kind.as_str()
                    ),
                ));
            }
            for s in &report.stale {
                let what = match s.why {
                    "traced" => "is now traced",
                    "missing" => "no longer exists",
                    _ => "is no longer a code unit under [traceCode]",
                };
                out.push((
                    "W327",
                    bl.clone(),
                    format!("trace-code baseline entry '{}' {what} — remove it (`syscribe trace-code --write-baseline` drops stale entries)", s.path),
                ));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn containment_rule() {
        assert!(names("crates/a", "crates/a"));
        assert!(names("crates/a/src/lib.rs", "crates/a"));
        assert!(names("crates", "crates/a"));
        assert!(names("", "crates/a"));
        assert!(!names("crates/ab", "crates/a"));
        assert!(!names("crates/a", "crates/ab"));
    }

    #[test]
    fn config_shapes_and_problems() {
        let v: toml::Value = toml::from_str("roots = [\"./crates/\"]\nunits = \"cargo-crate\"\nbuildFiles = [\"BUILD\"]\n").unwrap();
        let c = TraceCodeConfig::from_value(&v);
        assert!(c.problems.is_empty(), "{:?}", c.problems);
        assert_eq!(c.roots, ["crates"]);
        assert_eq!(c.rules[0].kind, UnitKind::CargoCrate);
        assert_eq!(c.build_files, ["BUILD"]);
        let v: toml::Value = toml::from_str("units = [{ kind = \"module\" }, { kind = \"file\", extensions = [\".SH\"], under = \"s/\" }]\nvalidate = \"yes\"\n").unwrap();
        let c = TraceCodeConfig::from_value(&v);
        assert_eq!(c.problems.len(), 3, "{:?}", c.problems);
        assert_eq!(c.rules, [UnitRule { kind: UnitKind::File, under: Some("s".into()), extensions: vec!["sh".into()] }]);
    }

    #[test]
    fn baseline_round_trip() {
        let dir = std::env::temp_dir().join(format!("syscribe-tcb-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("b.toml");
        let e = vec![BaselineEntry { path: "z/x".into(), reason: "a \"quoted\" reason".into() }, BaselineEntry { path: "a".into(), reason: "TODO".into() }];
        std::fs::write(&p, render_baseline(&e)).unwrap();
        let back = load_baseline(&p).unwrap();
        assert_eq!(back[0].path, "a");
        assert_eq!(back[1].reason, "a \"quoted\" reason");
        std::fs::write(&p, "[[untraced]]\npath = \"a\"\n[[untraced]]\npath = \"a\"\n").unwrap();
        assert!(load_baseline(&p).unwrap_err().contains("twice"));
        assert_eq!(load_baseline(&dir.join("none.toml")).unwrap(), vec![]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
