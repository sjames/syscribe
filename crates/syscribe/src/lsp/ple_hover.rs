//! Variability hover and code lenses (GH #420, `REQ-TRS-LSPHOVER-001`).
//!
//! * hover on an element's `appliesWhen:` — the Configurations it is active in;
//! * hover on a `parameterBindings:` key — the parameter's type, unit, range, window and the parameter constraints
//!   it takes part in, as they stand for that Configuration;
//! * code lens above a gated element — `active in N of M configurations`;
//! * code lens on a parameter of a `FeatureDef` — `bound at <stage> by <Configuration>`.
//!
//! All of it is computed by the projection and provenance code the CLI uses (`is_active_canon`,
//! `provenance::provenance`, `constraint_states`), not re-implemented here.

use std::path::Path;

use tower_lsp::lsp_types::*;

use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::projection::{canonical_selection, is_active_canon};
use syscribe_model::provenance::{provenance, Source};

use super::store::LspStore;

fn md(text: String) -> HoverContents {
    HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value: text })
}

fn ident(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

fn configurations(store: &LspStore) -> Vec<&RawElement> {
    let mut v: Vec<&RawElement> = store.elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Configuration)).collect();
    v.sort_by_key(|c| ident(c));
    v
}

/// The Configurations `elem` is active in, and the number considered; `None` for an element that is not gated.
pub(super) fn activity(store: &LspStore, elem: &RawElement) -> Option<(Vec<String>, Vec<String>)> {
    let pkg = syscribe_model::variability::package_conditions(&store.elements);
    syscribe_model::variability::effective_applies_when(elem, &pkg)?;
    let alias = syscribe_model::variability::feature_id_to_qname(&store.elements);
    let cfgs = configurations(store);
    if cfgs.is_empty() {
        return None;
    }
    let (mut on, mut off) = (Vec::new(), Vec::new());
    for c in cfgs {
        let sel = canonical_selection(&store.elements, c);
        if is_active_canon(elem, &sel, &pkg, &alias) {
            on.push(ident(c));
        } else {
            off.push(ident(c));
        }
    }
    Some((on, off))
}

/// `entry key` of a `key: rest` line (a `::` inside a qualified name does not end it) and the byte index of its colon.
fn entry_key(line: &str) -> Option<(String, usize, usize)> {
    let b = line.as_bytes();
    let i = (0..b.len()).find(|&i| b[i] == b':' && (i + 1 == b.len() || b[i + 1] == b' '))?;
    let lead = line.len() - line.trim_start().len();
    let key = line[lead..i].trim_start_matches("- ").trim();
    Some((key.to_string(), lead, i))
}

fn top_level_key(lines: &[&str], line: usize) -> Option<String> {
    for i in (1..=line).rev() {
        let l = lines.get(i)?;
        if !l.starts_with(char::is_whitespace) && !l.trim().is_empty() {
            return entry_key(l).map(|(k, ..)| k);
        }
        if *l == "---" {
            return None;
        }
    }
    None
}

pub(super) fn hover(store: &LspStore, path: &Path, position: Position) -> Option<Hover> {
    let text = std::fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = text.lines().collect();
    let (line, col) = (position.line as usize, position.character as usize);
    let (key, lead, colon) = entry_key(lines.get(line)?)?;
    let on_key = col >= lead && col <= colon;
    if !on_key {
        return None;
    }
    let path_str = path.to_string_lossy();
    let elem = store.elements.iter().find(|e| e.file_path == path_str)?;

    if key == "appliesWhen" && lead == 0 {
        let (on, off) = activity(store, elem)?;
        let total = on.len() + off.len();
        let mut s = format!("**Active in {} of {} configuration{}**", on.len(), total, if total == 1 { "" } else { "s" });
        if !on.is_empty() {
            s.push_str(&format!("\n\n{}", on.iter().map(|c| format!("- `{c}`")).collect::<Vec<_>>().join("\n")));
        }
        if !off.is_empty() {
            s.push_str(&format!("\n\nInactive in: {}", off.iter().map(|c| format!("`{c}`")).collect::<Vec<_>>().join(", ")));
        }
        return Some(Hover { contents: md(s), range: None });
    }

    // A binding, or the narrowing of a parameter's allowed values, in a Configuration.
    let section = top_level_key(&lines, line);
    if matches!(section.as_deref(), Some("parameterBindings" | "parameterNarrowing")) && elem.frontmatter.element_type == Some(ElementType::Configuration) {
        return Some(Hover { contents: md(binding_markdown(store, elem, &key, section.as_deref() == Some("parameterNarrowing"))?), range: None });
    }
    None
}

/// What a `parameterBindings:` key means for this Configuration.
fn binding_markdown(store: &LspStore, cfg: &RawElement, key: &str, narrowing: bool) -> Option<String> {
    let alias = syscribe_model::variability::feature_id_to_qname(&store.elements);
    let (f, pname) = key.rsplit_once('.')?;
    let fq = syscribe_model::variability::canon_feature_ref(f, &alias);
    let fd = store.elements.iter().find(|e| e.frontmatter.element_type == Some(ElementType::FeatureDef) && e.qualified_name == fq)?;
    let p = fd.frontmatter.parameters.iter().flatten().find(|p| p.get("name").and_then(|n| n.as_str()) == Some(pname))?;
    let get = |k: &str| {
        p.get(k).map(|v| match v {
            serde_yaml::Value::String(s) => s.clone(),
            serde_yaml::Value::Number(n) => n.to_string(),
            serde_yaml::Value::Bool(b) => b.to_string(),
            other => format!("{other:?}"),
        })
    };
    let canon = format!("{fq}.{pname}");
    let mut s = format!("**{canon}**\n\n`{}`{}", get("type").unwrap_or_else(|| "(untyped)".into()), get("unit").map(|u| format!(" in {u}")).unwrap_or_default());
    let mut facts = Vec::new();
    if let Some(r) = get("range") {
        facts.push(format!("range {r}"));
    }
    if let Some(vs) = p.get("enumValues").and_then(|v| v.as_sequence()) {
        facts.push(format!("one of {}", vs.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", ")));
    }
    let (w, _) = syscribe_model::binding_time::BindingWindow::read(
        get("bindingTime").as_deref().or(fd.frontmatter.binding_time.as_deref()),
        get("latestBindingTime").as_deref().or(fd.frontmatter.latest_binding_time.as_deref()),
    );
    if let Some(w) = w {
        facts.push(format!("bindable {}..{}", w.earliest.name(), w.latest.name()));
    }
    if let Some(d) = get("default") {
        facts.push(format!("default {d}"));
    }
    if !facts.is_empty() {
        s.push_str(&format!("\n\n{}", facts.join(" · ")));
    }
    if let Some(stage) = &cfg.frontmatter.binding_stage {
        s.push_str(&format!("\n\n{} here at stage `{stage}`.", if narrowing { "Narrowed" } else { "Bound" }));
    }
    // The parameter constraints this value takes part in, as they stand for this Configuration.
    if let Some(idx) = store.elements.iter().position(|e| std::ptr::eq(e, cfg)) {
        let prefix = format!("{canon}");
        let cs: Vec<_> = syscribe_model::feature_model::constraint_states(&store.elements, idx).into_iter().filter(|c| c.expression.contains(&prefix)).collect();
        if !cs.is_empty() {
            s.push_str("\n\n**Constraints**\n");
            for c in cs {
                let st = match c.state {
                    syscribe_model::feature_model::Tri::Satisfied => "satisfied",
                    syscribe_model::feature_model::Tri::Violated => "violated",
                    syscribe_model::feature_model::Tri::Undetermined => "undetermined",
                };
                s.push_str(&format!("\n- `{}` — `{}` — {st}", c.id, c.expression));
            }
        }
    }
    Some(s)
}

fn line_range(lines: &[&str], line: usize) -> Range {
    Range::new(Position::new(line as u32, 0), Position::new(line as u32, lines.get(line).map_or(0, |l| l.chars().count()) as u32))
}

/// The variability lenses of one file.
pub(super) fn lenses(store: &LspStore, path: &Path) -> Vec<CodeLens> {
    let Ok(text) = std::fs::read_to_string(path) else { return vec![] };
    let lines: Vec<&str> = text.lines().collect();
    let path_str = path.to_string_lossy();
    let Some(elem) = store.elements.iter().find(|e| e.file_path == path_str) else { return vec![] };
    let lens = |range: Range, title: String| CodeLens { range, command: Some(Command { title, command: String::new(), arguments: None }), data: None };
    let mut out = Vec::new();

    if let Some((on, off)) = activity(store, elem) {
        // On the `appliesWhen:` line when the element has its own; else at the top of its frontmatter.
        let at = lines.iter().position(|l| l.starts_with("appliesWhen:")).map(|i| line_range(&lines, i)).unwrap_or_else(|| super::frontmatter_range(path));
        let total = on.len() + off.len();
        out.push(lens(at, format!("active in {} of {} configuration{}", on.len(), total, if total == 1 { "" } else { "s" })));
    }

    if elem.frontmatter.element_type == Some(ElementType::FeatureDef) {
        let cfgs = configurations(store);
        let provs: Vec<(String, syscribe_model::provenance::ConfigProvenance)> = cfgs
            .iter()
            .filter_map(|c| store.elements.iter().position(|e| std::ptr::eq(e, *c)).map(|i| (ident(c), provenance(&store.elements, i))))
            .collect();
        for p in elem.frontmatter.parameters.iter().flatten() {
            let Some(name) = p.get("name").and_then(|n| n.as_str()) else { continue };
            let Some(at) = lines.iter().position(|l| l.trim_start().trim_start_matches("- ").starts_with(&format!("name: {name}"))) else { continue };
            let key = format!("{}.{name}", elem.qualified_name);
            let mut binders: Vec<(Option<String>, String)> = Vec::new();
            for (cid, prov) in &provs {
                if let Some(pp) = prov.parameters.iter().find(|q| q.key == key) {
                    if matches!(pp.source, Source::Own) {
                        binders.push((pp.stage.clone(), cid.clone()));
                    }
                }
            }
            let title = match binders.split_first() {
                None => match p.get("default") {
                    Some(d) => format!("unbound — default {}", serde_yaml::to_string(d).unwrap_or_default().trim()),
                    None => "unbound".to_string(),
                },
                Some(((stage, cid), rest)) => {
                    let more = if rest.is_empty() { String::new() } else { format!(" (+{} more)", rest.len()) };
                    match stage {
                        Some(s) => format!("bound at {s} by {cid}{more}"),
                        None => format!("bound by {cid}{more}"),
                    }
                }
            };
            out.push(lens(line_range(&lines, at), title));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_and_top_level_fields() {
        let (k, lead, colon) = entry_key("  Features::Propulsion.motorKV: 1000").unwrap();
        assert_eq!((k.as_str(), lead, colon), ("Features::Propulsion.motorKV", 2, 30));
        let lines = ["---", "type: Configuration", "parameterBindings:", "  Features::Propulsion.motorKV: 1000", "---"];
        assert_eq!(top_level_key(&lines, 3).as_deref(), Some("parameterBindings"));
        assert_eq!(top_level_key(&lines, 1).as_deref(), Some("type"));
    }
}
