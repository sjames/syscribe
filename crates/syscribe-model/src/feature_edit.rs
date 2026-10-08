//! Semantic edits of the feature model (`REQ-TRS-FMED-004`, `ADR-SYS-FMED-001`).
//!
//! An [`EditOp`] says what to change (`add` a feature under a parent, `move` it,
//! `rename` it, set its group kind or membership, add or remove a `requires:` or
//! `excludes:` constraint, `remove` it); [`apply`] performs it against a model
//! root by editing the files the feature lives in, and returns the operation
//! that undoes it. It never decides whether the edit is wise: callers run it
//! inside a guarded write and compare [`feature_model::analysis_json`] before and
//! after ([`analysis_delta`]) to preview what the edit does to the model's validity.
//!
//! Features live either one per file (a directory is the group: `_index.md` is
//! the group's own file) or as entries of a `FeatureModel` sheet's `featureTree:`.
//! Every operation works on the per-file layout. On a sheet entry only
//! `setGroup` and `setMandatory` are supported; the rest say so and name the sheet.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::element::{ElementType, RawElement};
use crate::feature_tree::{feature_tree, FeatureNode};
use crate::frontmatter::patch_frontmatter;
use crate::mutate::mv::{move_element, valid_qname};
use crate::resolver::Resolver;
use crate::walker::{is_synthesized, walk_model};

/// One file to put back (`content: Some`) or delete (`content: None`), relative to the model root.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreFile {
    pub rel: String,
    pub content: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum EditOp {
    /// A new feature `name` under `parent` (a feature, by qualified name or id), or a new root when absent.
    #[serde(rename_all = "camelCase")]
    Add {
        #[serde(default)]
        parent: Option<String>,
        name: String,
        #[serde(default)]
        group_kind: Option<String>,
        #[serde(default)]
        mandatory: Option<bool>,
    },
    /// Delete a feature; with `subtree` also everything below it, else it must be a leaf.
    #[serde(rename_all = "camelCase")]
    Remove {
        feature: String,
        #[serde(default)]
        subtree: bool,
    },
    /// Give a feature a new name (its qualified name changes; references are rewritten).
    Rename { feature: String, name: String },
    /// Change how a feature's children are grouped: `optional`, `alternative` or `or`.
    #[serde(rename_all = "camelCase")]
    SetGroup { feature: String, group_kind: String },
    /// Make a feature a mandatory or optional member of its parent.
    SetMandatory { feature: String, mandatory: bool },
    /// Reparent a feature (with its subtree) under `new_parent`, or make it a root when absent.
    #[serde(rename_all = "camelCase")]
    Move {
        feature: String,
        #[serde(default)]
        new_parent: Option<String>,
    },
    /// `kind` is `requires` or `excludes`.
    AddConstraint { feature: String, kind: String, target: String },
    RemoveConstraint { feature: String, kind: String, target: String },
    /// Put files back as they were (the inverse of `add`, `remove` and the field edits).
    Restore { files: Vec<RestoreFile> },
}

/// What an applied edit returns.
#[derive(Debug, Clone, PartialEq)]
pub struct EditOutcome {
    /// The operation that undoes this one.
    pub undo: EditOp,
    /// The qualified name of the feature the edit left in place (new for `add`, `rename` and `move`).
    pub feature: Option<String>,
}

struct Ctx {
    root: PathBuf,
    elements: Vec<RawElement>,
    tree: Vec<FeatureNode>,
    alias: std::collections::HashMap<String, String>,
}

impl Ctx {
    fn load(root: &Path) -> Result<Ctx, String> {
        let elements = walk_model(root).map_err(|e| e.to_string())?;
        let tree = feature_tree(&elements);
        let alias = crate::variability::feature_id_to_qname(&elements);
        Ok(Ctx { root: root.to_path_buf(), elements, tree, alias })
    }

    fn qname(&self, r: &str) -> String {
        self.alias.get(r).cloned().unwrap_or_else(|| r.replace('/', "::"))
    }

    fn feature(&self, r: &str) -> Result<&FeatureNode, String> {
        let q = self.qname(r);
        self.tree.iter().find(|f| f.qname == q).ok_or_else(|| format!("'{r}' is not a feature of this model"))
    }

    fn element(&self, qname: &str) -> Option<&RawElement> {
        self.elements.iter().find(|e| e.qualified_name == qname && e.frontmatter.element_type == Some(ElementType::FeatureDef))
    }

    /// The feature's own file, relative to the model root, refusing a sheet entry.
    fn own_file(&self, f: &FeatureNode) -> Result<String, String> {
        let e = self.element(&f.qname).ok_or_else(|| format!("'{}' has no element", f.qname))?;
        let rel = Path::new(&e.file_path).strip_prefix(&self.root).unwrap_or(Path::new(&e.file_path)).to_path_buf();
        if is_synthesized(e, &rel) {
            return Err(sheet_message(&f.qname, &e.file_path, &self.root));
        }
        Ok(rel.to_string_lossy().replace('\\', "/"))
    }

    fn read(&self, rel: &str) -> Result<String, String> {
        std::fs::read_to_string(self.root.join(rel)).map_err(|e| format!("cannot read {rel}: {e}"))
    }

    fn write(&self, rel: &str, content: &str) -> Result<(), String> {
        let path = self.root.join(rel);
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).map_err(|e| format!("cannot create {}: {e}", p.display()))?;
        }
        std::fs::write(&path, content).map_err(|e| format!("cannot write {rel}: {e}"))
    }
}

fn sheet_message(qname: &str, file: &str, root: &Path) -> String {
    let rel = Path::new(file).strip_prefix(root).unwrap_or(Path::new(file)).display().to_string();
    format!("'{qname}' is an entry of the feature-model sheet {rel}: only its group kind and membership can be changed here; edit the sheet for anything else")
}

fn strings_of(v: Option<&serde_yaml::Value>) -> Vec<String> {
    match v {
        Some(serde_yaml::Value::String(s)) => vec![s.clone()],
        Some(serde_yaml::Value::Sequence(seq)) => seq.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
        _ => Vec::new(),
    }
}

fn set_strings(map: &mut serde_yaml::Mapping, key: &str, items: Vec<String>) {
    let k = serde_yaml::Value::String(key.to_string());
    if items.is_empty() {
        map.remove(&k);
    } else {
        map.insert(k, serde_yaml::Value::Sequence(items.into_iter().map(serde_yaml::Value::String).collect()));
    }
}

fn key(k: &str) -> serde_yaml::Value {
    serde_yaml::Value::String(k.to_string())
}

/// `FEAT-` plus the name's letters and digits upper-cased, kept within the id grammar
/// (segments of 2 to 12 characters) and distinct from every id in use.
fn new_feature_id(name: &str, taken: &HashSet<String>) -> String {
    let mut seg: String = name.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_uppercase();
    seg.truncate(12);
    while seg.len() < 2 {
        seg.push('X');
    }
    let base = format!("FEAT-{seg}");
    if !taken.contains(&base) {
        return base;
    }
    for n in 2..1000 {
        let cand = format!("{base}-{n:02}");
        if !taken.contains(&cand) {
            return cand;
        }
    }
    base
}

/// Apply `op` to the model rooted at `root`. See the module doc.
pub fn apply(root: &Path, op: &EditOp) -> Result<EditOutcome, String> {
    let cx = Ctx::load(root)?;
    match op {
        EditOp::Add { parent, name, group_kind, mandatory } => add(&cx, parent.as_deref(), name, group_kind.as_deref(), *mandatory),
        EditOp::Remove { feature, subtree } => remove(&cx, feature, *subtree),
        EditOp::Rename { feature, name } => rename(&cx, feature, name),
        EditOp::SetGroup { feature, group_kind } => set_group(&cx, feature, group_kind),
        EditOp::SetMandatory { feature, mandatory } => set_mandatory(&cx, feature, *mandatory),
        EditOp::Move { feature, new_parent } => move_feature(&cx, feature, new_parent.as_deref()),
        EditOp::AddConstraint { feature, kind, target } => constraint(&cx, feature, kind, target, true),
        EditOp::RemoveConstraint { feature, kind, target } => constraint(&cx, feature, kind, target, false),
        EditOp::Restore { files } => restore(&cx, files),
    }
}

fn restore(cx: &Ctx, files: &[RestoreFile]) -> Result<EditOutcome, String> {
    let mut undo = Vec::new();
    for f in files {
        let before = std::fs::read_to_string(cx.root.join(&f.rel)).ok();
        undo.push(RestoreFile { rel: f.rel.clone(), content: before });
        match &f.content {
            Some(c) => cx.write(&f.rel, c)?,
            None => {
                let _ = std::fs::remove_file(cx.root.join(&f.rel));
            }
        }
    }
    Ok(EditOutcome { undo: EditOp::Restore { files: undo }, feature: None })
}

fn add(cx: &Ctx, parent: Option<&str>, name: &str, group_kind: Option<&str>, mandatory: Option<bool>) -> Result<EditOutcome, String> {
    let name = name.trim();
    let parent_q = match parent {
        Some(p) => Some(cx.feature(p)?.qname.clone()),
        None => None,
    };
    let base = match &parent_q {
        Some(p) => p.clone(),
        None => {
            // A new root goes beside the existing roots.
            let roots: Vec<&FeatureNode> = cx.tree.iter().filter(|f| f.parent.is_none()).collect();
            let Some(first) = roots.first() else {
                return Err("the model has no features yet: create the first one as a FeatureDef element".to_string());
            };
            let mut prefix: Vec<&str> = first.qname.split("::").collect();
            prefix.pop();
            for r in &roots[1..] {
                let segs: Vec<&str> = r.qname.split("::").collect();
                let common = prefix.iter().zip(segs.iter()).take_while(|(a, b)| a == b).count();
                prefix.truncate(common);
            }
            prefix.join("::")
        }
    };
    let qname = if base.is_empty() { name.to_string() } else { format!("{base}::{name}") };
    if !valid_qname(&qname) || name.contains("::") || name.is_empty() {
        return Err(format!("'{name}' is not a valid feature name: use letters, digits and underscores, not starting with a digit"));
    }
    if cx.elements.iter().any(|e| e.qualified_name == qname) {
        return Err(format!("'{qname}' already exists"));
    }
    if let Some(g) = group_kind {
        if !matches!(g, "optional" | "alternative" | "or") {
            return Err(format!("group kind '{g}' is not optional, alternative or or"));
        }
    }
    let taken: HashSet<String> = cx.elements.iter().filter_map(|e| e.frontmatter.id.clone()).collect();
    let id = new_feature_id(name, &taken);
    let mut map = serde_yaml::Mapping::new();
    map.insert(key("type"), key("FeatureDef"));
    map.insert(key("id"), key(&id));
    map.insert(key("name"), key(name));
    if let Some(g) = group_kind.filter(|g| *g != "optional") {
        map.insert(key("groupKind"), key(g));
    }
    if mandatory == Some(true) {
        map.insert(key("mandatory"), serde_yaml::Value::Bool(true));
    }
    let yaml = serde_yaml::to_string(&serde_yaml::Value::Mapping(map)).map_err(|e| e.to_string())?;
    let rel = format!("{}.md", qname.replace("::", "/"));
    cx.write(&rel, &format!("---\n{yaml}---\n"))?;
    Ok(EditOutcome { undo: EditOp::Restore { files: vec![RestoreFile { rel, content: None }] }, feature: Some(qname) })
}

fn descendants<'a>(cx: &'a Ctx, q: &str, out: &mut Vec<&'a FeatureNode>) {
    for f in cx.tree.iter().filter(|f| f.parent.as_deref() == Some(q)) {
        out.push(f);
        descendants(cx, &f.qname, out);
    }
}

fn remove(cx: &Ctx, feature: &str, subtree: bool) -> Result<EditOutcome, String> {
    let f = cx.feature(feature)?;
    let mut doomed: Vec<&FeatureNode> = vec![f];
    descendants(cx, &f.qname, &mut doomed);
    if doomed.len() > 1 && !subtree {
        return Err(format!("'{}' has {} feature(s) below it: remove its subtree, or move them first", f.qname, doomed.len() - 1));
    }
    let gone: HashSet<String> = doomed.iter().flat_map(|d| [Some(d.qname.clone()), d.id.clone()]).flatten().collect();
    let mut undo: Vec<RestoreFile> = Vec::new();
    let mut deleted: HashSet<String> = HashSet::new();
    for d in &doomed {
        let rel = cx.own_file(d)?;
        if deleted.insert(rel.clone()) {
            undo.push(RestoreFile { rel: rel.clone(), content: Some(cx.read(&rel)?) });
            std::fs::remove_file(cx.root.join(&rel)).map_err(|e| format!("cannot delete {rel}: {e}"))?;
        }
    }
    // Constraints and configuration choices that named a removed feature go with it.
    let canon = |s: &str| cx.alias.get(s).cloned().unwrap_or_else(|| s.to_string());
    for e in &cx.elements {
        let rel = Path::new(&e.file_path).strip_prefix(&cx.root).unwrap_or(Path::new(&e.file_path)).to_string_lossy().replace('\\', "/");
        if deleted.contains(&rel) || is_synthesized(e, Path::new(&rel)) {
            continue;
        }
        let is_feature = e.frontmatter.element_type == Some(ElementType::FeatureDef);
        let is_config = e.frontmatter.element_type == Some(ElementType::Configuration);
        if !is_feature && !is_config {
            continue;
        }
        let content = cx.read(&rel)?;
        let mut touched = false;
        let patched = patch_frontmatter(&content, None, |map| {
            if is_feature {
                for k in ["requires", "excludes"] {
                    let items = strings_of(map.get(key(k)));
                    let kept: Vec<String> = items.iter().filter(|s| !gone.contains(&canon(s))).cloned().collect();
                    if kept.len() != items.len() {
                        touched = true;
                        set_strings(map, k, kept);
                    }
                }
            } else if let Some(serde_yaml::Value::Mapping(feats)) = map.get_mut(key("features")) {
                let doomed_keys: Vec<serde_yaml::Value> = feats.keys().filter(|k| k.as_str().is_some_and(|s| gone.contains(&canon(s)))).cloned().collect();
                for k in doomed_keys {
                    feats.remove(&k);
                    touched = true;
                }
            }
        })
        .map_err(|e| e.to_string())?;
        if touched {
            undo.push(RestoreFile { rel: rel.clone(), content: Some(content) });
            cx.write(&rel, &patched)?;
        }
    }
    undo.reverse();
    Ok(EditOutcome { undo: EditOp::Restore { files: undo }, feature: None })
}

/// Patch the frontmatter of the feature's own file, returning the undo that puts it back.
fn patch_own(cx: &Ctx, f: &FeatureNode, mutate: impl FnOnce(&mut serde_yaml::Mapping)) -> Result<EditOutcome, String> {
    let rel = cx.own_file(f)?;
    let before = cx.read(&rel)?;
    let after = patch_frontmatter(&before, None, mutate).map_err(|e| e.to_string())?;
    cx.write(&rel, &after)?;
    Ok(EditOutcome { undo: EditOp::Restore { files: vec![RestoreFile { rel, content: Some(before) }] }, feature: Some(f.qname.clone()) })
}

/// Patch a feature's `featureTree:` entry in its sheet.
fn patch_sheet_entry(cx: &Ctx, f: &FeatureNode, mutate: impl FnOnce(&mut serde_yaml::Mapping)) -> Result<EditOutcome, String> {
    let e = cx.element(&f.qname).ok_or_else(|| format!("'{}' has no element", f.qname))?;
    let rel = Path::new(&e.file_path).strip_prefix(&cx.root).unwrap_or(Path::new(&e.file_path)).to_string_lossy().replace('\\', "/");
    let sheet_qname = crate::walker::derive_qname(Path::new(&rel));
    let rel_name = f.qname.strip_prefix(&format!("{sheet_qname}::")).unwrap_or(&f.qname).replace("::", ".");
    let before = cx.read(&rel)?;
    let mut found = false;
    let mut mutate = Some(mutate);
    let after = patch_frontmatter(&before, None, |map| {
        if let Some(serde_yaml::Value::Sequence(entries)) = map.get_mut(key("featureTree")) {
            for entry in entries.iter_mut() {
                if let serde_yaml::Value::Mapping(m) = entry {
                    if m.get(key("name")).and_then(|v| v.as_str()) == Some(rel_name.as_str()) {
                        if let Some(f) = mutate.take() {
                            f(m);
                            found = true;
                        }
                    }
                }
            }
        }
    })
    .map_err(|e| e.to_string())?;
    if !found {
        return Err(format!("could not find '{rel_name}' in the featureTree of {rel}"));
    }
    cx.write(&rel, &after)?;
    Ok(EditOutcome { undo: EditOp::Restore { files: vec![RestoreFile { rel, content: Some(before) }] }, feature: Some(f.qname.clone()) })
}

fn is_sheet_entry(cx: &Ctx, f: &FeatureNode) -> bool {
    cx.element(&f.qname)
        .map(|e| {
            let rel = Path::new(&e.file_path).strip_prefix(&cx.root).unwrap_or(Path::new(&e.file_path)).to_path_buf();
            is_synthesized(e, &rel)
        })
        .unwrap_or(false)
}

fn set_group(cx: &Ctx, feature: &str, group_kind: &str) -> Result<EditOutcome, String> {
    if !matches!(group_kind, "optional" | "alternative" | "or") {
        return Err(format!("group kind '{group_kind}' is not optional, alternative or or"));
    }
    let f = cx.feature(feature)?.clone();
    let apply = |map: &mut serde_yaml::Mapping| {
        if group_kind == "optional" {
            map.remove(key("groupKind"));
        } else {
            map.insert(key("groupKind"), key(group_kind));
        }
    };
    if is_sheet_entry(cx, &f) {
        patch_sheet_entry(cx, &f, apply)
    } else {
        patch_own(cx, &f, apply)
    }
}

fn set_mandatory(cx: &Ctx, feature: &str, mandatory: bool) -> Result<EditOutcome, String> {
    let f = cx.feature(feature)?.clone();
    let apply = |map: &mut serde_yaml::Mapping| {
        // The legacy `groupKind: mandatory` shorthand would override the flag: drop it.
        if map.get(key("groupKind")).and_then(|v| v.as_str()) == Some("mandatory") {
            map.remove(key("groupKind"));
        }
        if mandatory {
            map.insert(key("mandatory"), serde_yaml::Value::Bool(true));
        } else {
            map.remove(key("mandatory"));
        }
    };
    if is_sheet_entry(cx, &f) {
        patch_sheet_entry(cx, &f, apply)
    } else {
        patch_own(cx, &f, apply)
    }
}

fn constraint(cx: &Ctx, feature: &str, kind: &str, target: &str, add: bool) -> Result<EditOutcome, String> {
    if kind != "requires" && kind != "excludes" {
        return Err(format!("constraint kind '{kind}' is not requires or excludes"));
    }
    let f = cx.feature(feature)?.clone();
    let t = cx.feature(target)?.clone();
    if f.qname == t.qname {
        return Err("a feature cannot constrain itself".to_string());
    }
    // Reference the target by its stable id when it has one, so a rename leaves the constraint intact.
    let spelled = t.id.clone().filter(|i| crate::resolver::is_feat_id(i)).unwrap_or_else(|| t.qname.clone());
    let canon = |s: &str| cx.alias.get(s).cloned().unwrap_or_else(|| s.to_string());
    let same = |s: &str| canon(s) == t.qname;
    let rel = cx.own_file(&f)?;
    let mut already = false;
    let mut absent = false;
    let out = patch_own(cx, &f, |map| {
        let mut items = strings_of(map.get(key(kind)));
        if add {
            if items.iter().any(|s| same(s)) {
                already = true;
            } else {
                items.push(spelled.clone());
            }
        } else if items.iter().any(|s| same(s)) {
            items.retain(|s| !same(s));
        } else {
            absent = true;
        }
        set_strings(map, kind, items);
    })?;
    if already {
        cx.write(&rel, &restore_content(&out))?;
        return Err(format!("'{}' already {kind} '{}'", f.qname, t.qname));
    }
    if absent {
        cx.write(&rel, &restore_content(&out))?;
        return Err(format!("'{}' does not {} '{}'", f.qname, kind.trim_end_matches('s'), t.qname));
    }
    Ok(out)
}

fn restore_content(out: &EditOutcome) -> String {
    match &out.undo {
        EditOp::Restore { files } => files.first().and_then(|f| f.content.clone()).unwrap_or_default(),
        _ => String::new(),
    }
}

fn rename(cx: &Ctx, feature: &str, name: &str) -> Result<EditOutcome, String> {
    let f = cx.feature(feature)?.clone();
    let name = name.trim();
    if name.is_empty() || name.contains("::") || !valid_qname(name) {
        return Err(format!("'{name}' is not a valid feature name: use letters, digits and underscores, not starting with a digit"));
    }
    let old_short = f.qname.rsplit("::").next().unwrap_or(&f.qname).to_string();
    let new_q = match f.qname.rsplit_once("::") {
        Some((p, _)) => format!("{p}::{name}"),
        None => name.to_string(),
    };
    relocate(cx, &f, &new_q)?;
    // The label follows the file name when it was the file name.
    let cx2 = Ctx::load(&cx.root)?;
    if let Ok(nf) = cx2.feature(&new_q) {
        let nf = nf.clone();
        if cx2.element(&nf.qname).and_then(|e| e.frontmatter.name.clone()).as_deref() == Some(old_short.as_str()) {
            patch_own(&cx2, &nf, |map| {
                map.insert(key("name"), key(name));
            })?;
        }
    }
    Ok(EditOutcome { undo: EditOp::Rename { feature: new_q.clone(), name: old_short }, feature: Some(new_q) })
}

fn move_feature(cx: &Ctx, feature: &str, new_parent: Option<&str>) -> Result<EditOutcome, String> {
    let f = cx.feature(feature)?.clone();
    let short = f.qname.rsplit("::").next().unwrap_or(&f.qname).to_string();
    let parent_q = match new_parent {
        Some(p) => {
            let p = cx.feature(p)?.qname.clone();
            let mut under = Vec::new();
            descendants(cx, &f.qname, &mut under);
            if p == f.qname || under.iter().any(|d| d.qname == p) {
                return Err(format!("cannot move '{}' under '{p}': that is inside its own subtree", f.qname));
            }
            Some(p)
        }
        None => None,
    };
    let base = match &parent_q {
        Some(p) => p.clone(),
        None => {
            let roots: Vec<&FeatureNode> = cx.tree.iter().filter(|r| r.parent.is_none() && r.qname != f.qname).collect();
            let Some(first) = roots.first() else { return Err("there is no other root to place it beside".to_string()) };
            first.qname.rsplit_once("::").map(|(p, _)| p.to_string()).unwrap_or_default()
        }
    };
    let new_q = if base.is_empty() { short.clone() } else { format!("{base}::{short}") };
    if new_q == f.qname {
        return Err(format!("'{}' is already there", f.qname));
    }
    let old_parent = f.parent.clone();
    relocate(cx, &f, &new_q)?;
    Ok(EditOutcome { undo: EditOp::Move { feature: new_q.clone(), new_parent: old_parent }, feature: Some(new_q) })
}

/// Move a feature (and the directory of its children, when it has one beside its
/// file) to `new_q`, rewriting every reference to it.
fn relocate(cx: &Ctx, f: &FeatureNode, new_q: &str) -> Result<(), String> {
    cx.own_file(f)?; // refuse a sheet entry
    let resolver = Resolver::new(&cx.elements);
    let dir = cx.root.join(f.qname.replace("::", "/"));
    let file_is_index = cx.element(&f.qname).is_some_and(|e| Path::new(&e.file_path).file_name().is_some_and(|n| n == "_index.md"));
    // A feature that is a plain file with a directory of children beside it
    // needs two moves: `move_element` relocates a qualified name's directory
    // when there is one, and its file only when there is none.
    let two_moves = dir.is_dir() && !file_is_index;
    move_element(&cx.root, &cx.elements, &resolver, &f.qname, new_q, false).map_err(|e| e.to_string())?;
    if two_moves {
        let elements = walk_model(&cx.root).map_err(|e| e.to_string())?;
        let resolver = Resolver::new(&elements);
        move_element(&cx.root, &elements, &resolver, &f.qname, new_q, false).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── Validity preview ─────────────────────────────────────────────────────────

fn states(a: &Value) -> std::collections::BTreeMap<String, String> {
    a["features"]
        .as_object()
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v["state"].as_str().unwrap_or("normal").to_string())).collect())
        .unwrap_or_default()
}

fn names(a: &Value, key: &str) -> HashSet<String> {
    a[key].as_array().map(|l| l.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default()
}

/// What an edit did to the model's validity, from the analysis before and after
/// (`feature_model::analysis_json` of each): features that became dead or
/// false-optional (or stopped being), whether the model became void or healed,
/// and configurations that became invalid. `worsens` is true when anything got worse.
pub fn analysis_delta(before: &Value, after: &Value) -> Value {
    let (b, a) = (states(before), states(after));
    let mut new_dead = Vec::new();
    let mut resolved_dead = Vec::new();
    let mut new_false_optional = Vec::new();
    let mut resolved_false_optional = Vec::new();
    for (q, s) in &a {
        let was = b.get(q).map(String::as_str);
        match (was, s.as_str()) {
            (w, "dead") if w != Some("dead") && w.is_some() => new_dead.push(q.clone()),
            (w, "falseOptional") if w != Some("falseOptional") && w.is_some() => new_false_optional.push(q.clone()),
            _ => {}
        }
    }
    for (q, s) in &b {
        match (s.as_str(), a.get(q).map(String::as_str)) {
            ("dead", Some(now)) if now != "dead" => resolved_dead.push(q.clone()),
            ("falseOptional", Some(now)) if now != "falseOptional" => resolved_false_optional.push(q.clone()),
            _ => {}
        }
    }
    let (void_before, void_after) = (before["void"] == true, after["void"] == true);
    let (inv_b, inv_a) = (names(before, "invalidConfigurations"), names(after, "invalidConfigurations"));
    let mut new_invalid: Vec<String> = inv_a.difference(&inv_b).cloned().collect();
    let mut resolved_invalid: Vec<String> = inv_b.difference(&inv_a).cloned().collect();
    new_invalid.sort();
    resolved_invalid.sort();
    // A model that was void shows every feature dead: report the void, not a flood of features.
    if void_after {
        new_dead.clear();
    }
    let became_void = void_after && !void_before;
    let worsens = became_void || !new_dead.is_empty() || !new_false_optional.is_empty() || !new_invalid.is_empty();
    json!({
        "worsens": worsens,
        "becameVoid": became_void,
        "healedVoid": void_before && !void_after,
        "newDead": new_dead,
        "resolvedDead": resolved_dead,
        "newFalseOptional": new_false_optional,
        "resolvedFalseOptional": resolved_false_optional,
        "newInvalidConfigurations": new_invalid,
        "resolvedInvalidConfigurations": resolved_invalid,
        "conflicts": after["conflicts"],
        "countsBefore": before["counts"],
        "countsAfter": after["counts"],
    })
}
