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
use crate::mutate::mv::{move_element, reference_edits, valid_qname};
use crate::resolver::Resolver;
use crate::walker::{is_synthesized, walk_model};

/// One file to put back (`content: Some`) or delete (`content: None`), relative to the model root.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreFile {
    pub rel: String,
    pub content: Option<String>,
    /// What the file must look like right now for this restore to apply: `"absent"`, or the blake3 of
    /// its content. An undo carries it so an edit made elsewhere in between is not silently lost;
    /// absent (as in a hand-written request) means unchecked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expect: Option<String>,
}

/// `"absent"` for a missing file, else the blake3 of its content: the state a conditional restore expects.
fn state_of(path: &Path) -> String {
    match std::fs::read(path) {
        Ok(bytes) => blake3::hash(&bytes).to_hex().to_string(),
        Err(_) => "absent".to_string(),
    }
}

fn state_of_content(content: &Option<String>) -> String {
    content.as_ref().map_or_else(|| "absent".to_string(), |c| blake3::hash(c.as_bytes()).to_hex().to_string())
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
    /// Mark a feature abstract: a grouping feature with no implementation of its own. It is still
    /// a feature of the model: it can be selected, constrained and counted like any other.
    #[serde(rename_all = "camelCase")]
    SetAbstract { feature: String, is_abstract: bool },
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
    /// Add or replace one parameter of a feature, by its `name` (the whole declaration: `type`, `range`, `default`, ...).
    SetParameter { feature: String, parameter: Value },
    /// Remove a parameter. Refused while any configuration still binds it: remove those bindings first.
    RemoveParameter { feature: String, name: String },
    /// Remove one configuration's binding of a feature's parameter (`parameterBindings:`).
    RemoveBinding { configuration: String, feature: String, name: String },
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

    /// A model-relative path, or an error: absolute paths, `..` and empty segments would let a
    /// request (an edit op arrives from HTTP and MCP) touch files outside the model.
    fn confine(&self, rel: &str) -> Result<PathBuf, String> {
        confine_rel(&self.root, rel)
    }

    fn read(&self, rel: &str) -> Result<String, String> {
        std::fs::read_to_string(self.confine(rel)?).map_err(|e| format!("cannot read {rel}: {e}"))
    }

    fn write(&self, rel: &str, content: &str) -> Result<(), String> {
        let path = self.confine(rel)?;
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).map_err(|e| format!("cannot create {}: {e}", p.display()))?;
        }
        std::fs::write(&path, content).map_err(|e| format!("cannot write {rel}: {e}"))
    }
}

/// `root.join(rel)` when `rel` stays inside `root` by construction (only normal components).
pub fn confine_rel(root: &Path, rel: &str) -> Result<PathBuf, String> {
    let p = Path::new(rel);
    let ok = !rel.is_empty() && p.components().all(|c| matches!(c, std::path::Component::Normal(_)));
    if !ok {
        return Err(format!("'{rel}' is not a path inside the model: absolute paths and '..' are refused"));
    }
    Ok(root.join(p))
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
        EditOp::SetAbstract { feature, is_abstract } => set_abstract(&cx, feature, *is_abstract),
        EditOp::Move { feature, new_parent } => move_feature(&cx, feature, new_parent.as_deref()),
        EditOp::AddConstraint { feature, kind, target } => constraint(&cx, feature, kind, target, true),
        EditOp::RemoveConstraint { feature, kind, target } => constraint(&cx, feature, kind, target, false),
        EditOp::SetParameter { feature, parameter } => set_parameter(&cx, feature, parameter),
        EditOp::RemoveParameter { feature, name } => remove_parameter(&cx, feature, name),
        EditOp::RemoveBinding { configuration, feature, name } => remove_binding(&cx, configuration, feature, name),
        EditOp::Restore { files } => restore(&cx, files),
    }
}

fn restore(cx: &Ctx, files: &[RestoreFile]) -> Result<EditOutcome, String> {
    let mut undo = Vec::new();
    // Refuse the whole request before touching anything, so a bad path half-way cannot leave a partial restore.
    let paths: Vec<PathBuf> = files.iter().map(|f| cx.confine(&f.rel)).collect::<Result<_, _>>()?;
    for (f, path) in files.iter().zip(&paths) {
        if let Some(expect) = &f.expect {
            if &state_of(path) != expect {
                return Err(format!("{} has changed since the edit (it was edited elsewhere): the undo is refused so that change is not lost", f.rel));
            }
        }
    }
    for (f, path) in files.iter().zip(&paths) {
        let before = std::fs::read_to_string(path).ok();
        undo.push(RestoreFile { rel: f.rel.clone(), content: before, expect: Some(state_of_content(&f.content)) });
        match &f.content {
            Some(c) => cx.write(&f.rel, c)?,
            None => {
                let _ = std::fs::remove_file(path);
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
    let text = format!("---\n{yaml}---\n");
    cx.write(&rel, &text)?;
    let expect = Some(state_of_content(&Some(text)));
    Ok(EditOutcome { undo: EditOp::Restore { files: vec![RestoreFile { rel, content: None, expect }] }, feature: Some(qname) })
}

fn descendants<'a>(cx: &'a Ctx, q: &str, out: &mut Vec<&'a FeatureNode>) {
    for f in cx.tree.iter().filter(|f| f.parent.as_deref() == Some(q)) {
        out.push(f);
        descendants(cx, &f.qname, out);
    }
}

// ── A transaction: every file an edit touches, so the edit can be undone ─────

/// Records each file's content the first time an edit reads or changes it, so
/// [`Tx::undo`] can put the whole set back (a file that did not exist is deleted).
struct Tx<'a> {
    cx: &'a Ctx,
    originals: std::collections::BTreeMap<String, Option<String>>,
}

impl<'a> Tx<'a> {
    fn new(cx: &'a Ctx) -> Tx<'a> {
        Tx { cx, originals: Default::default() }
    }
    fn note(&mut self, rel: &str) {
        self.originals.entry(rel.to_string()).or_insert_with(|| std::fs::read_to_string(self.cx.root.join(rel)).ok());
    }
    fn read(&mut self, rel: &str) -> Result<String, String> {
        self.note(rel);
        self.cx.read(rel)
    }
    fn write(&mut self, rel: &str, content: &str) -> Result<(), String> {
        self.note(rel);
        self.cx.write(rel, content)
    }
    fn delete(&mut self, rel: &str) -> Result<(), String> {
        self.note(rel);
        std::fs::remove_file(self.cx.root.join(rel)).map_err(|e| format!("cannot delete {rel}: {e}"))
    }
    /// Patch the YAML frontmatter of `rel`; `mutate` may refuse, and then nothing is written.
    fn patch(&mut self, rel: &str, mutate: impl FnOnce(&mut serde_yaml::Mapping) -> Result<(), String>) -> Result<bool, String> {
        let before = self.read(rel)?;
        let mut refusal: Option<String> = None;
        let after = patch_frontmatter(&before, None, |m| {
            if let Err(e) = mutate(m) {
                refusal = Some(e);
            }
        })
        .map_err(|e| e.to_string())?;
        if let Some(e) = refusal {
            return Err(e);
        }
        if after == before {
            return Ok(false);
        }
        self.write(rel, &after)?;
        Ok(true)
    }
    fn undo(self) -> EditOp {
        let root = self.cx.root.clone();
        let files = self
            .originals
            .into_iter()
            .rev()
            .map(|(rel, content)| {
                let expect = Some(state_of(&root.join(&rel)));
                RestoreFile { rel, content, expect }
            })
            .collect();
        EditOp::Restore { files }
    }
}

fn rel_of(root: &Path, file_path: &str) -> String {
    Path::new(file_path).strip_prefix(root).unwrap_or(Path::new(file_path)).to_string_lossy().replace('\\', "/")
}

// ── Sheet helpers ────────────────────────────────────────────────────────────

/// The `FeatureModel` sheet a feature is an entry of.
struct Sheet {
    rel: String,
    qname: String,
}

fn sheet_of(cx: &Ctx, f: &FeatureNode) -> Option<Sheet> {
    let e = cx.element(&f.qname)?;
    let rel = rel_of(&cx.root, &e.file_path);
    if !is_synthesized(e, Path::new(&rel)) {
        return None;
    }
    Some(Sheet { qname: crate::walker::derive_qname(Path::new(&rel)), rel })
}

/// A feature's dotted path relative to its sheet (`Platform.CortexM`).
fn rel_name(sheet_qname: &str, qname: &str) -> String {
    qname.strip_prefix(&format!("{sheet_qname}::")).unwrap_or(qname).replace("::", ".")
}

/// The qualified name a sheet value names: an absolute qname as written, a `FEAT-*`
/// id through the alias map, anything else a dotted path relative to the sheet.
fn resolve_in_sheet(sheet_qname: &str, alias: &std::collections::HashMap<String, String>, s: &str) -> String {
    if s.contains("::") {
        s.to_string()
    } else if s.starts_with("FEAT") {
        alias.get(s).cloned().unwrap_or_else(|| s.to_string())
    } else if sheet_qname.is_empty() {
        s.replace('.', "::")
    } else {
        format!("{sheet_qname}::{}", s.replace('.', "::"))
    }
}

fn entries_mut(top: &mut serde_yaml::Mapping) -> Option<&mut Vec<serde_yaml::Value>> {
    match top.get_mut(key("featureTree")) {
        Some(serde_yaml::Value::Sequence(v)) => Some(v),
        _ => None,
    }
}

fn entry_name(v: &serde_yaml::Value) -> Option<&str> {
    v.as_mapping().and_then(|m| m.get(key("name"))).and_then(|n| n.as_str())
}

/// The feature's own frontmatter map: its file's, or its entry in its sheet's `featureTree:`.
fn with_feature_map(cx: &Ctx, tx: &mut Tx, f: &FeatureNode, mutate: impl FnOnce(&mut serde_yaml::Mapping, Option<&mut serde_yaml::Mapping>, &str) -> Result<(), String>) -> Result<(), String> {
    match sheet_of(cx, f) {
        Some(sh) => {
            let name = rel_name(&sh.qname, &f.qname);
            tx.patch(&sh.rel, |top| {
                // The entry and the sheet's other lists are different parts of one map: take the entry out, edit, put it back.
                let idx = entries_mut(top)
                    .and_then(|v| v.iter().position(|e| entry_name(e) == Some(name.as_str())))
                    .ok_or_else(|| format!("could not find '{name}' in the featureTree of {}", sh.rel))?;
                let mut entry = match entries_mut(top).map(|v| v[idx].clone()) {
                    Some(serde_yaml::Value::Mapping(m)) => m,
                    _ => return Err(format!("'{name}' is not a mapping in the featureTree of {}", sh.rel)),
                };
                mutate(&mut entry, Some(top), &name)?;
                if let Some(v) = entries_mut(top) {
                    v[idx] = serde_yaml::Value::Mapping(entry);
                }
                Ok(())
            })?;
        }
        None => {
            let rel = cx.own_file(f)?;
            tx.patch(&rel, |m| mutate(m, None, ""))?;
        }
    }
    Ok(())
}

// ── Remove ───────────────────────────────────────────────────────────────────

fn remove(cx: &Ctx, feature: &str, subtree: bool) -> Result<EditOutcome, String> {
    let f = cx.feature(feature)?;
    let mut doomed: Vec<&FeatureNode> = vec![f];
    descendants(cx, &f.qname, &mut doomed);
    if doomed.len() > 1 && !subtree {
        return Err(format!("'{}' has {} feature(s) below it: remove its subtree, or move them first", f.qname, doomed.len() - 1));
    }
    let gone_q: HashSet<String> = doomed.iter().map(|d| d.qname.clone()).collect();
    let mut tx = Tx::new(cx);
    // Delete each feature's file, or its entries from the sheet that holds them.
    let mut sheet_entries: std::collections::BTreeMap<String, (String, HashSet<String>)> = Default::default();
    for d in &doomed {
        match sheet_of(cx, d) {
            Some(sh) => {
                let name = rel_name(&sh.qname, &d.qname);
                sheet_entries.entry(sh.rel.clone()).or_insert_with(|| (sh.qname.clone(), HashSet::new())).1.insert(name);
            }
            None => {
                let rel = cx.own_file(d)?;
                if cx.root.join(&rel).exists() {
                    tx.delete(&rel)?;
                }
            }
        }
    }
    for (rel, (_, names)) in &sheet_entries {
        tx.patch(rel, |top| {
            if let Some(v) = entries_mut(top) {
                v.retain(|e| entry_name(e).is_none_or(|n| !names.contains(n)));
            }
            Ok(())
        })?;
    }
    // Constraints, configuration choices and bindings that named a removed feature go with it.
    let ids: HashSet<String> = doomed.iter().filter_map(|d| d.id.clone()).collect();
    let canon = |s: &str| cx.alias.get(s).cloned().unwrap_or_else(|| s.to_string());
    let names_gone = |s: &str| ids.contains(s) || gone_q.contains(&canon(s));
    let binding_gone = |k: &str| k.split_once('.').is_some_and(|(q, _)| gone_q.contains(&canon(q)));
    let mut seen: HashSet<String> = HashSet::new();
    for e in &cx.elements {
        let rel = rel_of(&cx.root, &e.file_path);
        if !seen.insert(rel.clone()) || !cx.root.join(&rel).exists() {
            continue;
        }
        let ty = e.frontmatter.element_type.clone();
        let sheet_q = crate::walker::derive_qname(Path::new(&rel));
        if is_synthesized(e, Path::new(&rel)) {
            continue; // reached through its sheet, below
        }
        match ty {
            Some(ElementType::FeatureDef) => {
                tx.patch(&rel, |map| {
                    for k in ["requires", "excludes"] {
                        let items = strings_of(map.get(key(k)));
                        let kept: Vec<String> = items.iter().filter(|s| !names_gone(s)).cloned().collect();
                        if kept.len() != items.len() {
                            set_strings(map, k, kept);
                        }
                    }
                    Ok(())
                })?;
            }
            Some(ElementType::Configuration) => {
                tx.patch(&rel, |map| {
                    if let Some(serde_yaml::Value::Mapping(feats)) = map.get_mut(key("features")) {
                        let dead: Vec<serde_yaml::Value> = feats.keys().filter(|k| k.as_str().is_some_and(|s| names_gone(s))).cloned().collect();
                        for k in dead {
                            feats.remove(&k);
                        }
                    }
                    if let Some(serde_yaml::Value::Mapping(b)) = map.get_mut(key("parameterBindings")) {
                        let dead: Vec<serde_yaml::Value> = b.keys().filter(|k| k.as_str().is_some_and(|s| binding_gone(s))).cloned().collect();
                        let had = !dead.is_empty();
                        for k in dead {
                            b.remove(&k);
                        }
                        if had && b.is_empty() {
                            map.remove(key("parameterBindings"));
                        }
                    }
                    Ok(())
                })?;
            }
            Some(ElementType::FeatureModel) => {
                let alias = &cx.alias;
                tx.patch(&rel, |top| {
                    let gone_here = |s: &str| ids.contains(s) || gone_q.contains(&resolve_in_sheet(&sheet_q, alias, s));
                    if let Some(entries) = entries_mut(top) {
                        for entry in entries.iter_mut() {
                            if let serde_yaml::Value::Mapping(m) = entry {
                                for k in ["requires", "excludes"] {
                                    let items = strings_of(m.get(key(k)));
                                    let kept: Vec<String> = items.iter().filter(|s| !gone_here(s)).cloned().collect();
                                    if kept.len() != items.len() {
                                        set_strings(m, k, kept);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(serde_yaml::Value::Sequence(list)) = top.get_mut(key("crossTreeConstraints")) {
                        for item in list.iter_mut() {
                            if let serde_yaml::Value::Mapping(m) = item {
                                for k in ["requires", "excludes"] {
                                    let items = strings_of(m.get(key(k)));
                                    let kept: Vec<String> = items.iter().filter(|s| !gone_here(s)).cloned().collect();
                                    if kept.len() != items.len() {
                                        set_strings(m, k, kept);
                                    }
                                }
                            }
                        }
                        list.retain(|item| {
                            let Some(m) = item.as_mapping() else { return true };
                            let owner_gone = m.get(key("feature")).and_then(|v| v.as_str()).is_some_and(gone_here);
                            let empty = m.get(key("requires")).is_none() && m.get(key("excludes")).is_none();
                            !owner_gone && !empty
                        });
                        if list.is_empty() {
                            top.remove(key("crossTreeConstraints"));
                        }
                    }
                    Ok(())
                })?;
            }
            _ => {}
        }
    }
    Ok(EditOutcome { undo: tx.undo(), feature: None })
}

// ── Field edits (a feature's file, or its sheet entry) ───────────────────────

fn set_group(cx: &Ctx, feature: &str, group_kind: &str) -> Result<EditOutcome, String> {
    if !matches!(group_kind, "optional" | "alternative" | "or") {
        return Err(format!("group kind '{group_kind}' is not optional, alternative or or"));
    }
    let f = cx.feature(feature)?.clone();
    let mut tx = Tx::new(cx);
    with_feature_map(cx, &mut tx, &f, |map, _, _| {
        if group_kind == "optional" {
            map.remove(key("groupKind"));
        } else {
            map.insert(key("groupKind"), key(group_kind));
        }
        Ok(())
    })?;
    Ok(EditOutcome { undo: tx.undo(), feature: Some(f.qname) })
}

fn set_mandatory(cx: &Ctx, feature: &str, mandatory: bool) -> Result<EditOutcome, String> {
    let f = cx.feature(feature)?.clone();
    let mut tx = Tx::new(cx);
    with_feature_map(cx, &mut tx, &f, |map, _, _| {
        // The legacy `groupKind: mandatory` shorthand would override the flag: drop it.
        if map.get(key("groupKind")).and_then(|v| v.as_str()) == Some("mandatory") {
            map.remove(key("groupKind"));
        }
        if mandatory {
            map.insert(key("mandatory"), serde_yaml::Value::Bool(true));
        } else {
            map.remove(key("mandatory"));
        }
        Ok(())
    })?;
    Ok(EditOutcome { undo: tx.undo(), feature: Some(f.qname) })
}

fn set_abstract(cx: &Ctx, feature: &str, is_abstract: bool) -> Result<EditOutcome, String> {
    let f = cx.feature(feature)?.clone();
    if is_abstract {
        // A configuration that names an abstract feature is `E238`, so making the feature abstract
        // under one would turn it into an error: the entries go first, as with a bound parameter.
        let naming: Vec<String> = cx
            .elements
            .iter()
            .filter(|e| e.frontmatter.element_type == Some(ElementType::Configuration))
            .filter(|e| {
                e.frontmatter.declared_feature_selections().keys().any(|k| cx.alias.get(k.as_str()).map(String::as_str).unwrap_or(k) == f.qname)
            })
            .map(|e| e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone()))
            .collect();
        if !naming.is_empty() {
            return Err(format!("'{}' is named by {}: remove it from their `features:` first (an abstract feature is not a choice, E238)", f.qname, naming.join(", ")));
        }
    }
    let mut tx = Tx::new(cx);
    with_feature_map(cx, &mut tx, &f, |map, _, _| {
        if is_abstract {
            map.insert(key("isAbstract"), serde_yaml::Value::Bool(true));
        } else {
            map.remove(key("isAbstract"));
        }
        Ok(())
    })?;
    Ok(EditOutcome { undo: tx.undo(), feature: Some(f.qname) })
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
    let sheet_q = sheet_of(cx, &f).map(|s| s.qname).unwrap_or_default();
    let canon = |s: &str| resolve_in_sheet(&sheet_q, &cx.alias, s);
    let same = |s: &str| canon(s) == t.qname;
    let mut tx = Tx::new(cx);
    let (fq, tq) = (f.qname.clone(), t.qname.clone());
    with_feature_map(cx, &mut tx, &f, |map, top, _| {
        let mut items = strings_of(map.get(key(kind)));
        // A sheet may also hold the constraint in its `crossTreeConstraints:` list.
        let mut in_list = false;
        if let Some(top) = top.as_deref() {
            if let Some(serde_yaml::Value::Sequence(list)) = top.get(key("crossTreeConstraints")) {
                in_list = list.iter().filter_map(|i| i.as_mapping()).any(|m| {
                    m.get(key("feature")).and_then(|v| v.as_str()).is_some_and(|s| canon(s) == fq) && strings_of(m.get(key(kind))).iter().any(|s| same(s))
                });
            }
        }
        if add {
            if in_list || items.iter().any(|s| same(s)) {
                return Err(format!("'{fq}' already {kind} '{tq}'"));
            }
            items.push(spelled.clone());
            set_strings(map, kind, items);
        } else {
            let had = items.iter().any(|s| same(s));
            if !had && !in_list {
                return Err(format!("'{fq}' does not {} '{tq}'", kind.trim_end_matches('s')));
            }
            items.retain(|s| !same(s));
            set_strings(map, kind, items);
            if let Some(top) = top {
                if let Some(serde_yaml::Value::Sequence(list)) = top.get_mut(key("crossTreeConstraints")) {
                    for item in list.iter_mut() {
                        if let serde_yaml::Value::Mapping(m) = item {
                            if m.get(key("feature")).and_then(|v| v.as_str()).is_some_and(|s| canon(s) == fq) {
                                let kept: Vec<String> = strings_of(m.get(key(kind))).into_iter().filter(|s| !same(s)).collect();
                                set_strings(m, kind, kept);
                            }
                        }
                    }
                    list.retain(|i| i.as_mapping().is_none_or(|m| m.get(key("requires")).is_some() || m.get(key("excludes")).is_some()));
                    if list.is_empty() {
                        top.remove(key("crossTreeConstraints"));
                    }
                }
            }
        }
        Ok(())
    })?;
    Ok(EditOutcome { undo: tx.undo(), feature: Some(f.qname) })
}

// ── Parameters ───────────────────────────────────────────────────────────────

fn param_name(v: &serde_yaml::Value) -> Option<&str> {
    v.as_mapping().and_then(|m| m.get(key("name"))).and_then(|n| n.as_str())
}

/// Add or replace one parameter of a feature, by name. `parameter` is the whole
/// declaration (`name`, `type`, `range`, `enumValues`, `default`, `isRequired`, `isFixed`,
/// `value`, `unit`, `bindingTime`, …); the validator judges its contents.
fn set_parameter(cx: &Ctx, feature: &str, parameter: &Value) -> Result<EditOutcome, String> {
    let f = cx.feature(feature)?.clone();
    let obj = parameter.as_object().ok_or("a parameter is an object with a name")?;
    let name = obj.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
    if name.is_empty() || !valid_qname(&name) || name.contains("::") {
        return Err(format!("'{name}' is not a valid parameter name: use letters, digits and underscores, not starting with a digit"));
    }
    let decl = serde_yaml::to_value(parameter).map_err(|e| e.to_string())?;
    let mut tx = Tx::new(cx);
    with_feature_map(cx, &mut tx, &f, |map, _, _| {
        let mut list: Vec<serde_yaml::Value> = match map.get(key("parameters")) {
            Some(serde_yaml::Value::Sequence(s)) => s.clone(),
            _ => Vec::new(),
        };
        match list.iter().position(|p| param_name(p) == Some(name.as_str())) {
            Some(i) => list[i] = decl.clone(),
            None => list.push(decl.clone()),
        }
        map.insert(key("parameters"), serde_yaml::Value::Sequence(list));
        Ok(())
    })?;
    Ok(EditOutcome { undo: tx.undo(), feature: Some(f.qname) })
}

/// The configurations (by id, else qualified name) that bind `name` of the feature `q`.
fn binders(cx: &Ctx, q: &str, name: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for e in cx.elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Configuration)) {
        let Some(b) = e.frontmatter.parameter_bindings.as_ref().and_then(|v| v.as_mapping()) else { continue };
        let bound = b.keys().filter_map(|k| k.as_str()).any(|k| k.split_once('.').is_some_and(|(f, p)| p == name && cx.alias.get(f).map(String::as_str).unwrap_or(f) == q));
        if bound {
            out.push((e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone()), rel_of(&cx.root, &e.file_path)));
        }
    }
    out.extend(peer_binders(cx, q, name));
    out
}

/// GH #193 — the `Configuration`s of `[repos]` peers (a consolidating tier, §14.7) that bind
/// `name` of the feature `q`, either by its native qualified name or through their own
/// `repoImports:` mount of this model (the key is then translated back to this model's native
/// qualified name before comparing). Each hit is labelled
/// `<id> (repo '<alias>')`. A peer that cannot be read is skipped (the check is advisory
/// against files this model does not own, never a reason to fail the edit).
fn peer_binders(cx: &Ctx, q: &str, name: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for repo in crate::config::load_repos(&cx.root).into_iter().filter(|r| r.exists) {
        let Ok(peer_elems) = walk_model(&repo.model_root) else { continue };
        let peer_repos = crate::config::load_repos(&repo.model_root);
        let mounts = crate::config::repo_mounts(&peer_elems, &peer_repos);
        // A bare key names a peer feature by its native qualified name (section 14.4: local
        // first, then the first peer that knows it) unless the peer model has an element of
        // that name itself.
        let holds_q = peer_repos.iter().any(|r| r.qnames.contains(q));
        let local_q = peer_elems.iter().any(|e| e.qualified_name == q);
        for e in peer_elems.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Configuration)) {
            let Some(b) = e.frontmatter.parameter_bindings.as_ref().and_then(|v| v.as_mapping()) else { continue };
            let bound = b.keys().filter_map(|k| k.as_str()).filter_map(|k| k.rsplit_once('.')).any(|(f, p)| {
                p == name
                    && ((f == q && holds_q && !local_q)
                        || mounts.iter().any(|m| {
                        // The mount must point at a repo that really holds `q` (the candidate
                        // copy of this model is a sibling directory, so compare by content).
                        let native = if f == m.mount {
                            Some(m.peer_qname.clone())
                        } else {
                            f.strip_prefix(m.mount.as_str()).and_then(|t| t.strip_prefix("::")).map(|rest| format!("{}::{}", m.peer_qname, rest))
                        };
                        native.as_deref() == Some(q) && peer_repos.iter().any(|r| r.alias == m.alias && r.qnames.contains(q))
                        }))
            });
            if bound {
                let id = e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone());
                out.push((format!("{id} (repo '{}')", repo.alias), rel_of(&repo.model_root, &e.file_path)));
            }
        }
    }
    out
}

/// Remove a parameter of a feature. A parameter some configuration binds is not removed:
/// the bindings go first (`removeBinding`), so nothing is left naming a parameter that is gone.
fn remove_parameter(cx: &Ctx, feature: &str, name: &str) -> Result<EditOutcome, String> {
    let f = cx.feature(feature)?.clone();
    let bound = binders(cx, &f.qname, name);
    if !bound.is_empty() {
        let who: Vec<String> = bound.iter().map(|(id, _)| id.clone()).collect();
        return Err(format!(
            "parameter '{name}' of '{}' is bound by {}: remove those bindings first (removeBinding)",
            f.qname,
            who.join(", ")
        ));
    }
    let mut tx = Tx::new(cx);
    with_feature_map(cx, &mut tx, &f, |map, _, _| {
        let mut list: Vec<serde_yaml::Value> = match map.get(key("parameters")) {
            Some(serde_yaml::Value::Sequence(s)) => s.clone(),
            _ => Vec::new(),
        };
        let before = list.len();
        list.retain(|p| param_name(p) != Some(name));
        if list.len() == before {
            return Err(format!("'{}' has no parameter '{name}'", f.qname));
        }
        if list.is_empty() {
            map.remove(key("parameters"));
        } else {
            map.insert(key("parameters"), serde_yaml::Value::Sequence(list));
        }
        Ok(())
    })?;
    Ok(EditOutcome { undo: tx.undo(), feature: Some(f.qname) })
}

/// Remove one configuration's binding of a parameter.
fn remove_binding(cx: &Ctx, configuration: &str, feature: &str, name: &str) -> Result<EditOutcome, String> {
    let f = cx.feature(feature)?.clone();
    let conf = cx
        .elements
        .iter()
        .find(|e| e.frontmatter.element_type == Some(ElementType::Configuration) && (e.frontmatter.id.as_deref() == Some(configuration) || e.qualified_name == configuration.replace('/', "::")))
        .ok_or_else(|| format!("'{configuration}' is not a Configuration of this model"))?;
    let rel = rel_of(&cx.root, &conf.file_path);
    let mut tx = Tx::new(cx);
    tx.patch(&rel, |m| {
        let Some(serde_yaml::Value::Mapping(b)) = m.get_mut(key("parameterBindings")) else {
            return Err(format!("'{configuration}' binds no parameters"));
        };
        let dead: Vec<serde_yaml::Value> = b
            .keys()
            .filter(|k| k.as_str().and_then(|s| s.split_once('.')).is_some_and(|(q, p)| p == name && cx.alias.get(q).map(String::as_str).unwrap_or(q) == f.qname))
            .cloned()
            .collect();
        if dead.is_empty() {
            return Err(format!("'{configuration}' does not bind parameter '{name}' of '{}'", f.qname));
        }
        for k in dead {
            b.remove(&k);
        }
        if b.is_empty() {
            m.remove(key("parameterBindings"));
        }
        Ok(())
    })?;
    Ok(EditOutcome { undo: tx.undo(), feature: Some(f.qname) })
}

// ── Rename and move ──────────────────────────────────────────────────────────

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
    let exact = relocate(cx, &f, &new_q)?;
    // The label follows the file name when it was the file name (a sheet entry's
    // `name:` is its path, already changed).
    let cx2 = Ctx::load(&cx.root)?;
    if let Ok(nf) = cx2.feature(&new_q) {
        let nf = nf.clone();
        if sheet_of(&cx2, &nf).is_none() && cx2.element(&nf.qname).and_then(|e| e.frontmatter.name.clone()).as_deref() == Some(old_short.as_str()) {
            let mut tx = Tx::new(&cx2);
            with_feature_map(&cx2, &mut tx, &nf, |map, _, _| {
                map.insert(key("name"), key(name));
                Ok(())
            })?;
        }
    }
    Ok(EditOutcome { undo: exact.unwrap_or(EditOp::Rename { feature: new_q.clone(), name: old_short }), feature: Some(new_q) })
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
        None => match sheet_of(cx, &f) {
            // A root of a sheet entry stays in its sheet: its qualified name is the sheet's, then its own.
            Some(sh) => sh.qname,
            None => {
                let roots: Vec<&FeatureNode> = cx.tree.iter().filter(|r| r.parent.is_none() && r.qname != f.qname).collect();
                let Some(first) = roots.first() else { return Err("there is no other root to place it beside".to_string()) };
                first.qname.rsplit_once("::").map(|(p, _)| p.to_string()).unwrap_or_default()
            }
        },
    };
    let new_q = if base.is_empty() { short.clone() } else { format!("{base}::{short}") };
    if new_q == f.qname {
        return Err(format!("'{}' is already there", f.qname));
    }
    let old_parent = f.parent.clone();
    let exact = relocate(cx, &f, &new_q)?;
    Ok(EditOutcome { undo: exact.unwrap_or(EditOp::Move { feature: new_q.clone(), new_parent: old_parent }), feature: Some(new_q) })
}

/// Move a feature to `new_q`, rewriting every reference to it: a feature file (and the
/// directory of its children, when it has one beside it) is relocated; a sheet entry is
/// renamed in its sheet together with the entries below it.
/// Returns the exact undo when the move touched a sheet (its files are put back as they were), else `None`:
/// the inverse move is then the undo.
fn relocate(cx: &Ctx, f: &FeatureNode, new_q: &str) -> Result<Option<EditOp>, String> {
    if let Some(sh) = sheet_of(cx, f) {
        return relocate_in_sheet(cx, f, &sh, new_q).map(Some);
    }
    cx.own_file(f)?;
    let resolver = Resolver::new(&cx.elements);
    move_element(&cx.root, &cx.elements, &resolver, &f.qname, new_q, false).map_err(|e| e.to_string())?;
    Ok(None)
}

fn relocate_in_sheet(cx: &Ctx, f: &FeatureNode, sh: &Sheet, new_q: &str) -> Result<EditOp, String> {
    // The new place must be in the same sheet, and so must everything below the feature.
    let in_sheet = |q: &str| q == sh.qname || q.starts_with(&format!("{}::", sh.qname)) || sh.qname.is_empty();
    if !in_sheet(new_q) {
        return Err(format!("'{}' is an entry of the sheet {}: it can only be renamed or moved within that sheet", f.qname, sh.rel));
    }
    let mut below = Vec::new();
    descendants(cx, &f.qname, &mut below);
    for d in &below {
        if sheet_of(cx, d).map(|s| s.rel) != Some(sh.rel.clone()) {
            return Err(format!("'{}' has the feature '{}' below it that is not an entry of {}: move that first", f.qname, d.qname, sh.rel));
        }
    }
    if cx.elements.iter().any(|e| e.qualified_name == new_q) {
        return Err(format!("'{new_q}' already exists"));
    }
    let (old_rel, new_rel) = (rel_name(&sh.qname, &f.qname), rel_name(&sh.qname, new_q));
    let mut tx = Tx::new(cx);
    // 1. Every absolute qualified-name reference, wherever it is written (configurations, `appliesWhen:`, other sheets, prose).
    for (path, orig, updated) in reference_edits(&cx.root, &cx.elements, &f.qname, new_q) {
        let rel = rel_of(&cx.root, &path.to_string_lossy());
        tx.note(&rel);
        std::fs::write(&path, updated).map_err(|e| format!("cannot write {rel}: {e}"))?;
        let _ = orig;
    }
    // 2. The sheet: rename the entries, keep each id stable, and follow the dotted paths that name them.
    let ids: std::collections::HashMap<String, String> = std::iter::once(f).chain(below.iter().copied()).filter_map(|d| d.id.clone().map(|i| (rel_name(&sh.qname, &d.qname), i))).collect();
    let moved = |s: &str| s == old_rel || s.starts_with(&format!("{old_rel}."));
    let renamed = |s: &str| format!("{new_rel}{}", &s[old_rel.len()..]);
    tx.patch(&sh.rel, |top| {
        let taken: HashSet<String> = entries_mut(top).map(|v| v.iter().filter_map(|e| entry_name(e).map(str::to_string)).collect()).unwrap_or_default();
        if taken.contains(&new_rel) {
            return Err(format!("the sheet {} already has an entry '{new_rel}'", sh.rel));
        }
        if let Some(entries) = entries_mut(top) {
            for entry in entries.iter_mut() {
                let serde_yaml::Value::Mapping(m) = entry else { continue };
                let Some(n) = m.get(key("name")).and_then(|v| v.as_str()).map(str::to_string) else { continue };
                if moved(&n) {
                    // A derived id would change with the name: write it down first.
                    let has_id = m.get(key("id")).and_then(|v| v.as_str()).is_some_and(|s| !s.is_empty());
                    if !has_id {
                        if let Some(id) = ids.get(&n) {
                            m.insert(key("id"), key(id));
                        }
                    }
                    m.insert(key("name"), key(&renamed(&n)));
                }
            }
        }
        // Dotted paths that name a moved feature, in constraints written relative to the sheet.
        let relative = |s: &str| !s.contains("::") && !s.starts_with("FEAT") && moved(s);
        let fix = |m: &mut serde_yaml::Mapping, k: &str| {
            let items = strings_of(m.get(key(k)));
            if items.iter().any(|s| relative(s)) {
                set_strings(m, k, items.into_iter().map(|s| if relative(&s) { renamed(&s) } else { s }).collect());
            }
        };
        if let Some(entries) = entries_mut(top) {
            for entry in entries.iter_mut() {
                if let serde_yaml::Value::Mapping(m) = entry {
                    fix(m, "requires");
                    fix(m, "excludes");
                }
            }
        }
        if let Some(serde_yaml::Value::Sequence(list)) = top.get_mut(key("crossTreeConstraints")) {
            for item in list.iter_mut() {
                if let serde_yaml::Value::Mapping(m) = item {
                    if let Some(owner) = m.get(key("feature")).and_then(|v| v.as_str()).map(str::to_string) {
                        if relative(&owner) {
                            m.insert(key("feature"), key(&renamed(&owner)));
                        }
                    }
                    fix(m, "requires");
                    fix(m, "excludes");
                }
            }
        }
        Ok(())
    })?;
    Ok(tx.undo())
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
