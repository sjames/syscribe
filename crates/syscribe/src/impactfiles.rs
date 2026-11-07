//! `impact --files <path>…` / `impact --since <rev>` — start impact analysis from changed files
//! instead of one named element (GH #434, `REQ-TRS-IMPACTFILES-001`).
//!
//! Each changed path is mapped to model elements by the element's own file, `implementedBy:`,
//! `sourceFile:` and `evidence[].path` (resolved with the same rules as `W023`/`E717`, so a
//! `repo:`/`model:` prefix or a bare model-root-relative path all work), the existing traversal is
//! run from that whole set, and the result is grouped by element type with each element marked
//! `changed` (its own file is in the same change set) or not.

use crate::impact::{traverse, Direction};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use syscribe_model::baseline::element_type_name;
use syscribe_model::config::{SourceLocation, ValidateConfig};
use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;

/// The default reach: unit → the requirement it satisfies (1) → the tests that verify it (2).
pub const DEFAULT_DEPTH: usize = 2;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hit {
    pub id: String,
    pub qualified_name: String,
    #[serde(rename = "type")]
    pub element_type: String,
    /// How the file maps to the element: `file`, `implementedBy`, `sourceFile` or `evidence`.
    pub via: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMap {
    pub file: String,
    pub elements: Vec<Hit>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Affected {
    pub id: String,
    pub qualified_name: String,
    #[serde(rename = "type")]
    pub element_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub distance: usize,
    pub via: String,
    /// The element's own file is among the changed files.
    pub changed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub changed_files: Vec<String>,
    pub mapped: Vec<FileMap>,
    pub unmapped: Vec<String>,
    pub affected: Vec<Affected>,
    /// Requirements and tests the change touches that were not themselves changed.
    pub unreviewed: Vec<Affected>,
}

/// `p` made absolute (against the working directory) with `.`/`..` removed, and the longest existing
/// prefix canonicalised, so a path that no longer exists (a deleted file) still compares equal.
pub(crate) fn lenient_canonical(p: &Path) -> PathBuf {
    syscribe_model::config::lenient_canonical(p)
}

/// [`lenient_canonical`] under the name other modules use.
pub(crate) fn lenient_path(p: &Path) -> PathBuf {
    lenient_canonical(p)
}

/// `p` relative to the git root, with `/` separators; `None` when it lies outside the repository.
pub fn to_git_rel(git_root: &Path, p: &Path) -> Option<String> {
    lenient_canonical(p).strip_prefix(git_root).ok().map(|r| r.to_string_lossy().replace('\\', "/"))
}

/// Files that differ between `rev` and the working tree (staged, unstaged and untracked), as paths
/// relative to the git root.
pub fn changed_since(git_root: &Path, rev: &str) -> Result<Vec<String>, String> {
    if rev.starts_with('-') {
        return Err(format!("`{rev}` is not a git revision"));
    }
    syscribe_model::config::git_output(git_root, &["rev-parse", "--verify", "--quiet", &format!("{rev}^{{commit}}")])
        .ok_or_else(|| format!("`{rev}` does not name a commit in this repository"))?;
    let run = |args: &[&str]| -> Result<Vec<String>, String> {
        let o = std::process::Command::new("git").arg("-C").arg(git_root).args(args).output().map_err(|e| format!("cannot run git: {e}"))?;
        if !o.status.success() {
            return Err(format!("git {}: {}", args.join(" "), String::from_utf8_lossy(&o.stderr).trim()));
        }
        Ok(String::from_utf8_lossy(&o.stdout).split('\0').filter(|s| !s.is_empty()).map(String::from).collect())
    };
    let mut files = run(&["diff", "--name-only", "-z", "--no-renames", rev])?;
    files.extend(run(&["ls-files", "-z", "--others", "--exclude-standard"])?);
    files.sort();
    files.dedup();
    Ok(files)
}

/// One way a file can bind to an element.
struct Key {
    rel: String,
    is_dir: bool,
    via: &'static str,
}

/// Every file or directory `e` is bound to.
fn keys_of(e: &RawElement, config: &ValidateConfig, git_root: &Path) -> Vec<Key> {
    let mut out = Vec::new();
    let mut push = |rel: Option<String>, is_dir: bool, via: &'static str| {
        if let Some(rel) = rel {
            out.push(Key { rel: rel.trim_end_matches('/').to_string(), is_dir, via });
        }
    };
    push(to_git_rel(git_root, Path::new(&e.file_path)), false, "file");
    let local = |value: &str| -> Option<(String, bool)> {
        match config.classify_source(value) {
            SourceLocation::Local(p) => {
                let is_dir = value.trim_end().ends_with('/') || p.is_dir();
                to_git_rel(git_root, &p).map(|r| (r, is_dir))
            }
            _ => None,
        }
    };
    for v in e.frontmatter.implemented_by.as_deref().unwrap_or(&[]) {
        if let Some((r, d)) = local(v) {
            push(Some(r), d, "implementedBy");
        }
    }
    if let Some(v) = &e.frontmatter.source_file {
        if let Some((r, d)) = local(v) {
            push(Some(r), d, "sourceFile");
        }
    }
    for entry in e.frontmatter.evidence.as_deref().unwrap_or(&[]) {
        if let Some(p) = entry.as_mapping().and_then(|m| m.get(serde_yaml::Value::String("path".into()))).and_then(|v| v.as_str()) {
            if let Some((r, d)) = local(p) {
                push(Some(r), d, "evidence");
            }
        }
    }
    out
}

fn label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

fn tname(e: &RawElement) -> String {
    e.frontmatter.element_type.as_ref().map(element_type_name).unwrap_or_default()
}

/// Map `changed` (git-root-relative paths) to elements, traverse from them, and report.
pub fn analyse(
    elements: &[RawElement],
    config: &ValidateConfig,
    git_root: &Path,
    changed: &[String],
    direction: Direction,
    depth: Option<usize>,
) -> Report {
    let resolver = Resolver::new(elements);
    let keyed: Vec<(usize, Vec<Key>)> = elements.iter().enumerate().map(|(i, e)| (i, keys_of(e, config, git_root))).collect();
    let changed_set: HashSet<&str> = changed.iter().map(String::as_str).collect();

    let mut mapped: Vec<FileMap> = Vec::new();
    let mut unmapped: Vec<String> = Vec::new();
    let mut seeds: Vec<usize> = Vec::new();
    for f in changed {
        let mut hits: Vec<Hit> = Vec::new();
        for (i, keys) in &keyed {
            // The strongest binding wins when one element is bound to the file several ways.
            let best = keys
                .iter()
                .filter(|k| k.rel == *f || (k.is_dir && f.starts_with(&format!("{}/", k.rel))))
                .min_by_key(|k| ["file", "implementedBy", "sourceFile", "evidence"].iter().position(|v| *v == k.via));
            if let Some(k) = best {
                let e = &elements[*i];
                hits.push(Hit { id: label(e), qualified_name: e.qualified_name.clone(), element_type: tname(e), via: k.via });
                if !seeds.contains(i) {
                    seeds.push(*i);
                }
            }
        }
        if hits.is_empty() {
            unmapped.push(f.clone());
        } else {
            hits.sort_by(|a, b| a.id.cmp(&b.id));
            mapped.push(FileMap { file: f.clone(), elements: hits });
        }
    }

    let own_file_changed = |e: &RawElement| to_git_rel(git_root, Path::new(&e.file_path)).is_some_and(|r| changed_set.contains(r.as_str()));
    let seed_q: Vec<String> = seeds.iter().map(|i| elements[*i].qualified_name.clone()).collect();
    let reached = traverse(elements, &resolver, &seed_q, direction, depth, &|_| true);

    let mut affected: Vec<Affected> = Vec::new();
    let mut add = |e: &RawElement, distance: usize, via: String| {
        affected.push(Affected {
            id: label(e),
            qualified_name: e.qualified_name.clone(),
            element_type: tname(e),
            status: e.frontmatter.status.clone(),
            distance,
            via,
            changed: own_file_changed(e),
        });
    };
    for i in &seeds {
        add(&elements[*i], 0, "changed file".to_string());
    }
    for n in &reached {
        if let Some(e) = resolver.get(elements, &n.qname) {
            add(e, n.depth, n.via.to_string());
        }
    }
    affected.sort_by(|a, b| (a.distance, &a.element_type, &a.id).cmp(&(b.distance, &b.element_type, &b.id)));
    let unreviewed: Vec<Affected> = affected
        .iter()
        .filter(|a| !a.changed && matches!(a.element_type.as_str(), "Requirement" | "TestCase"))
        .cloned()
        .collect();
    Report { changed_files: changed.to_vec(), mapped, unmapped, affected, unreviewed }
}

fn first_flag_index(args: &[String]) -> Option<usize> {
    args.iter().position(|a| a.starts_with("--"))
}

/// `args` are the tokens after `impact`. Returns the exit code: `0`, `1` for usage errors, `2` when
/// `--fail-on-unreviewed` finds something unreviewed.
pub fn cmd_impact_files(model_root: &Path, elements: &[RawElement], config: &ValidateConfig, args: &[String]) -> i32 {
    let usage = "Usage: syscribe -m <root> impact (--files <path>... | --since <rev>) [--direction downstream|upstream|both] [--depth N] [--format text|json] [--fail-on-unreviewed]";
    if let Some(i) = first_flag_index(args) {
        if i > 0 {
            eprintln!("impact: name an element or use --files/--since, not both\n{usage}");
            return 1;
        }
    }
    let value = |flag: &str| args.windows(2).find(|w| w[0] == flag).map(|w| w[1].as_str());
    let mut files: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--files" {
            i += 1;
            while i < args.len() && !args[i].starts_with("--") {
                files.push(args[i].clone());
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    let since = value("--since");
    if args.iter().any(|a| a == "--files") && files.is_empty() {
        eprintln!("impact: --files needs at least one path\n{usage}");
        return 1;
    }
    let direction = match crate::cliargs::enum_value("impact", args, "--direction", &["downstream", "upstream", "both"], "both") {
        Ok("upstream") => Direction::Upstream,
        Ok("downstream") => Direction::Downstream,
        Ok(_) => Direction::Both,
        Err(m) => {
            eprintln!("{m}");
            return 1;
        }
    };
    let depth = match crate::cliargs::usize_value("impact", args, "--depth") {
        Ok(d) => Some(d.unwrap_or(DEFAULT_DEPTH)),
        Err(m) => {
            eprintln!("{m}");
            return 1;
        }
    };
    let format = match crate::cliargs::enum_value("impact", args, "--format", &["text", "json", "dot"], "text") {
        Ok("dot") => {
            eprintln!("impact: --format dot is for a single element; --files/--since support text and json");
            return 1;
        }
        Ok(f) => f,
        Err(m) => {
            eprintln!("{m}");
            return 1;
        }
    };
    let fail = args.iter().any(|a| a == "--fail-on-unreviewed");

    let Some(git_root) = syscribe_model::config::detect_git_root(model_root) else {
        eprintln!("impact: {} is not inside a git repository, which --files/--since need to name files", model_root.display());
        return 1;
    };
    let mut changed: Vec<String> = files.iter().map(|f| to_git_rel(&git_root, Path::new(f)).unwrap_or_else(|| f.clone())).collect();
    if let Some(rev) = since {
        match changed_since(&git_root, rev) {
            Ok(mut more) => changed.append(&mut more),
            Err(m) => {
                eprintln!("impact: {m}");
                return 1;
            }
        }
    }
    changed.sort();
    changed.dedup();
    let report = analyse(elements, config, &git_root, &changed, direction, depth);

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "schemaVersion": "1.0", "report": report })).unwrap());
    } else {
        print_text(&report);
    }
    if fail && !report.unreviewed.is_empty() {
        if format != "json" {
            eprintln!("impact: {} requirement/test element(s) are affected but were not changed", report.unreviewed.len());
        }
        return 2;
    }
    0
}

fn print_text(r: &Report) {
    println!("# Impact of {} changed file(s)\n", r.changed_files.len());
    if r.changed_files.is_empty() {
        println!("No changed files.");
        return;
    }
    println!("Changed files → model elements:");
    for m in &r.mapped {
        let els: Vec<String> = m.elements.iter().map(|h| format!("{} ({}, {})", h.id, h.element_type, h.via)).collect();
        println!("  {}  →  {}", m.file, els.join("; "));
    }
    if !r.unmapped.is_empty() {
        println!("\nNot mapped to any model element ({}):", r.unmapped.len());
        for f in &r.unmapped {
            println!("  {f}");
        }
    }
    if r.affected.is_empty() {
        println!("\nNo model elements affected.");
        return;
    }
    let mut by_type: BTreeMap<&str, Vec<&Affected>> = BTreeMap::new();
    for a in &r.affected {
        by_type.entry(a.element_type.as_str()).or_default().push(a);
    }
    println!("\nAffected ({} element(s)):", r.affected.len());
    for (ty, list) in &by_type {
        println!("  {ty} ({})", list.len());
        for a in list {
            let st = a.status.as_deref().map(|s| format!(", {s}")).unwrap_or_default();
            let ch = if a.changed { "changed" } else { "not changed" };
            println!("    {} [{}{}]  distance {}, {}  — {}", a.id, ty, st, a.distance, a.via, ch);
        }
    }
    if !r.unreviewed.is_empty() {
        println!("\nAffected requirements/tests that were not changed ({}):", r.unreviewed.len());
        for a in &r.unreviewed {
            println!("  {} ({})", a.id, a.element_type);
        }
    }
}
