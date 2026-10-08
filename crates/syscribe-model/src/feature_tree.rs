//! The feature tree as plain data (`REQ-TRS-FMED-001`): every `FeatureDef` with
//! its parent, children, group kind, membership, constraints and parameters,
//! resolved exactly as the SAT encoding in [`crate::feature_model`] resolves them
//! (parent by `parentFeature:` else the nearest enclosing feature in the
//! qualified name; `FEAT-*` ids accepted in `requires:`/`excludes:`).
//!
//! The feature diagram, the analysis and configurator APIs and the editor all
//! read the tree through this one function so they cannot disagree about it.

use std::collections::{HashMap, HashSet};

use crate::element::{ElementType, RawElement};

/// How a feature's children are grouped (`groupKind:`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GroupKind {
    /// Children are independent.
    Optional,
    /// Exactly one child (XOR), unless a `cardinality:` says otherwise.
    Alternative,
    /// At least one child (OR), unless a `cardinality:` says otherwise.
    Or,
}

impl GroupKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            GroupKind::Optional => "optional",
            GroupKind::Alternative => "alternative",
            GroupKind::Or => "or",
        }
    }
}

/// One `FeatureDef`.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureNode {
    pub qname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub group_kind: GroupKind,
    /// A mandatory member of its parent's selection (`mandatory: true`, or the
    /// legacy `groupKind: mandatory`).
    pub mandatory: bool,
    pub is_abstract: bool,
    /// `requires:` targets, as qualified names of features that exist.
    pub requires: Vec<String>,
    /// `excludes:` targets, as qualified names of features that exist.
    pub excludes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cardinality: Option<String>,
    /// Typed parameters, as `name: type` (`unit` appended when declared).
    pub parameters: Vec<String>,
    /// The parameter declarations as written (`name`, `type`, `range`, `default`, ...), for the editor.
    pub parameter_decls: Vec<serde_json::Value>,
    pub file: String,
    /// Depth below its root, 0 for a root.
    pub depth: usize,
}

fn strip_last(q: &str) -> Option<&str> {
    q.rfind("::").map(|i| &q[..i])
}

fn strings(v: &[serde_yaml::Value]) -> Vec<String> {
    v.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()
}

/// Every `FeatureDef` in tree order: roots sorted by qualified name, each
/// followed depth-first by its children sorted by qualified name.
pub fn feature_tree(elements: &[RawElement]) -> Vec<FeatureNode> {
    let fdefs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| e.frontmatter.element_type == Some(ElementType::FeatureDef))
        .collect();
    if fdefs.is_empty() {
        return Vec::new();
    }
    let names: HashSet<&str> = fdefs.iter().map(|e| e.qualified_name.as_str()).collect();
    let alias: HashMap<&str, &str> = fdefs
        .iter()
        .filter_map(|e| {
            e.frontmatter.id.as_deref().filter(|id| crate::resolver::is_feat_id(id)).map(|id| (id, e.qualified_name.as_str()))
        })
        .collect();
    let canon = |r: &str| -> Option<String> {
        let q = alias.get(r).copied().unwrap_or(r);
        names.contains(q).then(|| q.to_string())
    };

    let parent_of = |e: &RawElement| -> Option<String> {
        if let Some(pf) = e.frontmatter.parent_feature.as_deref() {
            if names.contains(pf) {
                return Some(pf.to_string());
            }
        }
        let mut cur = strip_last(&e.qualified_name);
        while let Some(p) = cur {
            if names.contains(p) {
                return Some(p.to_string());
            }
            cur = strip_last(p);
        }
        None
    };

    let mut nodes: HashMap<String, FeatureNode> = HashMap::new();
    for e in &fdefs {
        let fm = &e.frontmatter;
        let gk_text = fm.group_kind.as_deref().unwrap_or("optional");
        let group_kind = match gk_text {
            "alternative" => GroupKind::Alternative,
            "or" => GroupKind::Or,
            _ => GroupKind::Optional,
        };
        let mandatory = fm.mandatory.unwrap_or(gk_text == "mandatory");
        let parameters = fm
            .parameters
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .filter_map(|p| {
                let m = p.as_mapping()?;
                let get = |k: &str| m.get(serde_yaml::Value::String(k.into())).and_then(|v| v.as_str());
                let name = get("name")?;
                let mut s = match get("type") {
                    Some(t) => format!("{name}: {t}"),
                    None => name.to_string(),
                };
                if let Some(u) = get("unit") {
                    s.push_str(&format!(" [{u}]"));
                }
                Some(s)
            })
            .collect();
        let node = FeatureNode {
            qname: e.qualified_name.clone(),
            id: fm.id.clone(),
            name: fm.name.clone().unwrap_or_else(|| e.qualified_name.rsplit("::").next().unwrap_or(&e.qualified_name).to_string()),
            parent: parent_of(e),
            children: Vec::new(),
            group_kind,
            mandatory,
            is_abstract: fm.is_abstract.unwrap_or(false),
            requires: fm.requires.as_deref().map(strings).unwrap_or_default().iter().filter_map(|r| canon(r)).collect(),
            excludes: fm.excludes.clone().unwrap_or_default().iter().filter_map(|r| canon(r)).collect(),
            cardinality: fm.cardinality.clone(),
            parameters,
            parameter_decls: fm.parameters.as_deref().unwrap_or(&[]).iter().filter_map(|p| serde_json::to_value(p).ok()).collect(),
            file: e.file_path.clone(),
            depth: 0,
        };
        nodes.insert(e.qualified_name.clone(), node);
    }
    let mut kids: HashMap<String, Vec<String>> = HashMap::new();
    let mut roots: Vec<String> = Vec::new();
    for (q, n) in &nodes {
        match &n.parent {
            Some(p) => kids.entry(p.clone()).or_default().push(q.clone()),
            None => roots.push(q.clone()),
        }
    }
    roots.sort();
    for v in kids.values_mut() {
        v.sort();
    }
    let mut out = Vec::with_capacity(nodes.len());
    let mut seen: HashSet<String> = HashSet::new();
    fn walk(q: &str, depth: usize, nodes: &mut HashMap<String, FeatureNode>, kids: &HashMap<String, Vec<String>>, out: &mut Vec<FeatureNode>, seen: &mut HashSet<String>) {
        if !seen.insert(q.to_string()) {
            return;
        }
        let children = kids.get(q).cloned().unwrap_or_default();
        if let Some(mut n) = nodes.remove(q) {
            n.depth = depth;
            n.children = children.clone();
            out.push(n);
        }
        for c in &children {
            walk(c, depth + 1, nodes, kids, out, seen);
        }
    }
    for r in &roots {
        walk(r, 0, &mut nodes, &kids, &mut out, &mut seen);
    }
    out
}

/// Whether `qname` is `root` or lies under it (an empty `root` contains everything).
pub fn is_under(qname: &str, root: &str) -> bool {
    root.is_empty() || qname == root || qname.strip_prefix(root).is_some_and(|r| r.starts_with("::"))
}
