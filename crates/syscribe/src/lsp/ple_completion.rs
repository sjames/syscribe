//! Product-line-aware completion (GH #419, `REQ-TRS-LSPPLE-001`): feature ids and qualified names where a feature is
//! expected (`appliesWhen:`, a Configuration's `features:` keys, `requires:`/`excludes:`), parameter paths where a
//! parameter is expected (`parameterBindings:` keys, `parameterConstraints:` expressions), the enumeration of a
//! parameter being bound, and the binding-time stages of the project's vocabulary. Items carry documentation: type,
//! unit, range, default, requiredness and binding window.

use tower_lsp::lsp_types::*;

use syscribe_model::binding_time::{active, BindingWindow};
use syscribe_model::element::{ElementType, RawElement};

use super::store::LspStore;

/// Which kind of thing the cursor is asking for.
#[derive(Debug, PartialEq, Eq)]
enum Want {
    Feature,
    Parameter,
    /// The enumeration (or `true`/`false`) of the parameter bound by this key.
    ParameterValue(String),
    Stage,
    Bool,
}

fn is_token_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | ':' | '.' | '-')
}

/// The `key` of a `key: rest` line (a `::` inside a qualified name does not end it), and whether `col` is past it.
fn entry_key(line: &str) -> Option<(String, usize)> {
    let b = line.as_bytes();
    let i = (0..b.len()).find(|&i| b[i] == b':' && (i + 1 == b.len() || b[i + 1] == b' '))?;
    let key = line[..i].trim_start().trim_start_matches("- ").trim().trim_matches(|c| c == '"' || c == '\'');
    Some((key.to_string(), i))
}

/// The top-level frontmatter key enclosing `line`, if the line is in the frontmatter.
fn top_level_key(lines: &[&str], line: usize) -> Option<(String, bool)> {
    if lines.first().copied() != Some("---") {
        return None;
    }
    let close = lines.iter().enumerate().skip(1).find(|(_, l)| **l == "---").map(|(i, _)| i)?;
    if line == 0 || line >= close {
        return None;
    }
    for i in (1..=line).rev() {
        let l = lines[i];
        if !l.starts_with(char::is_whitespace) && !l.trim().is_empty() {
            let (k, _) = entry_key(l)?;
            return Some((k, i == line));
        }
    }
    None
}

fn want(lines: &[&str], line: usize, col: usize) -> Option<Want> {
    let text = lines.get(line)?;
    let (field, on_field_line) = top_level_key(lines, line)?;
    let entry = entry_key(text);
    // Past the colon of this line's own `key:`?
    let in_value = entry.as_ref().is_some_and(|(_, i)| col > *i);
    let inner = entry.as_ref().map(|(k, _)| k.as_str());
    if on_field_line && !in_value && text.trim_start() == *text {
        return None; // on the field's own name
    }
    if in_value {
        match inner {
            Some("bindingTime" | "latestBindingTime" | "bindingStage") => return Some(Want::Stage),
            Some("appliesWhen") => return Some(Want::Feature),
            Some("expression") if field == "parameterConstraints" => return Some(Want::Parameter),
            _ => {}
        }
    }
    match field.as_str() {
        "appliesWhen" | "requires" | "excludes" => Some(Want::Feature),
        "features" if entry.is_some() && in_value => Some(Want::Bool),
        "features" => Some(Want::Feature),
        "parameterBindings" if in_value => Some(Want::ParameterValue(inner?.to_string())),
        "parameterBindings" => Some(Want::Parameter),
        _ => None,
    }
}

fn is(e: &RawElement, t: ElementType) -> bool {
    e.frontmatter.element_type == Some(t)
}

fn param_name(p: &serde_yaml::Value) -> Option<&str> {
    p.get("name").and_then(|n| n.as_str())
}

fn window_text(earliest: Option<&str>, latest: Option<&str>) -> Option<String> {
    let (w, _) = BindingWindow::read(earliest, latest);
    w.map(|w| if w.earliest == w.latest { w.earliest.name() } else { format!("{}..{}", w.earliest.name(), w.latest.name()) })
}

fn feature_items(store: &LspStore) -> Vec<CompletionItem> {
    let mut out = Vec::new();
    for fd in store.elements.iter().filter(|e| is(e, ElementType::FeatureDef)) {
        let f = &fd.frontmatter;
        let id = f.id.clone();
        let params: Vec<&str> = f.parameters.iter().flatten().filter_map(param_name).collect();
        let mut doc = format!("**{}**", f.name.clone().unwrap_or_else(|| fd.qualified_name.clone()));
        if let Some(i) = &id {
            doc.push_str(&format!(" (`{i}`)"));
        }
        doc.push_str(&format!("\n\n`{}`", fd.qualified_name));
        let mut facts = vec![format!("group {}", f.group_kind.as_deref().unwrap_or("optional"))];
        if f.mandatory == Some(true) && f.group_kind.as_deref() != Some("mandatory") {
            facts.push("mandatory".into());
        }
        if f.is_abstract == Some(true) {
            facts.push("abstract".into());
        }
        if let Some(w) = window_text(f.binding_time.as_deref(), f.latest_binding_time.as_deref()) {
            facts.push(format!("bound {w}"));
        }
        doc.push_str(&format!("\n\n{}", facts.join(" · ")));
        if !params.is_empty() {
            doc.push_str(&format!("\n\nparameters: {}", params.join(", ")));
        }
        let documentation = Some(Documentation::MarkupContent(MarkupContent { kind: MarkupKind::Markdown, value: doc }));
        // Both spellings are valid wherever a feature is named: offer the qualified name (filterable by its id too) and the id.
        out.push(CompletionItem {
            label: fd.qualified_name.clone(),
            detail: id.clone().or_else(|| f.name.clone()),
            kind: Some(CompletionItemKind::VALUE),
            filter_text: Some(match &id {
                Some(i) => format!("{} {}", fd.qualified_name, i),
                None => fd.qualified_name.clone(),
            }),
            documentation: documentation.clone(),
            ..Default::default()
        });
        if let Some(i) = id {
            out.push(CompletionItem {
                label: i,
                detail: Some(fd.qualified_name.clone()),
                kind: Some(CompletionItemKind::VALUE),
                documentation,
                ..Default::default()
            });
        }
    }
    out
}

fn param_doc(fd: &RawElement, p: &serde_yaml::Value) -> String {
    let s = |k: &str| p.get(k).and_then(|v| match v {
        serde_yaml::Value::String(t) => Some(t.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    });
    let mut lines = vec![format!("`{}`{}", s("type").unwrap_or_else(|| "(untyped)".into()), s("unit").map(|u| format!(" in {u}")).unwrap_or_default())];
    let mut facts = Vec::new();
    if let Some(r) = s("range") {
        facts.push(format!("range {r}"));
    }
    if let Some(vs) = p.get("enumValues").and_then(|v| v.as_sequence()) {
        facts.push(format!("one of {}", vs.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", ")));
    }
    if let Some(d) = s("default") {
        facts.push(format!("default {d}"));
    }
    if p.get("isRequired").and_then(|v| v.as_bool()) == Some(true) {
        facts.push("required".into());
    }
    if let Some(w) = window_text(s("bindingTime").as_deref().or(fd.frontmatter.binding_time.as_deref()), s("latestBindingTime").as_deref().or(fd.frontmatter.latest_binding_time.as_deref())) {
        facts.push(format!("bound {w}"));
    }
    if !facts.is_empty() {
        lines.push(facts.join(" · "));
    }
    lines.push(format!("of feature `{}`", fd.qualified_name));
    lines.join("\n\n")
}

fn parameter_items(store: &LspStore) -> Vec<CompletionItem> {
    let mut out = Vec::new();
    for fd in store.elements.iter().filter(|e| is(e, ElementType::FeatureDef)) {
        for p in fd.frontmatter.parameters.iter().flatten() {
            let Some(name) = param_name(p) else { continue };
            let ty = p.get("type").and_then(|t| t.as_str()).map(|t| t.rsplit("::").next().unwrap_or(t).to_string());
            out.push(CompletionItem {
                label: format!("{}.{name}", fd.qualified_name),
                detail: ty,
                kind: Some(CompletionItemKind::PROPERTY),
                documentation: Some(Documentation::MarkupContent(MarkupContent { kind: MarkupKind::Markdown, value: param_doc(fd, p) })),
                ..Default::default()
            });
        }
    }
    out
}

/// The enumeration (or booleans) of the parameter `key` names (`Feature.param`, the feature by qualified name or id).
fn value_items(store: &LspStore, key: &str) -> Vec<CompletionItem> {
    let alias = syscribe_model::variability::feature_id_to_qname(&store.elements);
    let Some((f, pname)) = key.rsplit_once('.') else { return vec![] };
    let fq = syscribe_model::variability::canon_feature_ref(f, &alias);
    let Some(fd) = store.elements.iter().find(|e| is(e, ElementType::FeatureDef) && e.qualified_name == fq) else { return vec![] };
    let Some(p) = fd.frontmatter.parameters.iter().flatten().find(|p| param_name(p) == Some(pname)) else { return vec![] };
    let doc = Some(Documentation::MarkupContent(MarkupContent { kind: MarkupKind::Markdown, value: param_doc(fd, p) }));
    let item = |label: String| CompletionItem { label, kind: Some(CompletionItemKind::ENUM_MEMBER), documentation: doc.clone(), ..Default::default() };
    if let Some(vs) = p.get("enumValues").and_then(|v| v.as_sequence()) {
        return vs.iter().filter_map(|v| v.as_str().map(|s| item(s.to_string()))).collect();
    }
    if p.get("type").and_then(|t| t.as_str()).is_some_and(|t| t.ends_with("Boolean")) {
        return vec![item("true".into()), item("false".into())];
    }
    vec![]
}

fn stage_items() -> Vec<CompletionItem> {
    active()
        .stages()
        .iter()
        .map(|s| CompletionItem {
            label: s.name.clone(),
            detail: s.binder.clone(),
            kind: Some(CompletionItemKind::CONSTANT),
            documentation: s.description.clone().map(|d| Documentation::MarkupContent(MarkupContent { kind: MarkupKind::Markdown, value: d })),
            ..Default::default()
        })
        .collect()
}

/// Replace the token under the cursor, so a qualified name completes as one unit.
fn with_edit(items: Vec<CompletionItem>, line: u32, start: u32, end: u32) -> Vec<CompletionItem> {
    let range = Range::new(Position::new(line, start), Position::new(line, end));
    items
        .into_iter()
        .map(|mut i| {
            i.text_edit = Some(CompletionTextEdit::Edit(TextEdit { range, new_text: i.label.clone() }));
            i
        })
        .collect()
}

pub(super) fn candidates(store: &LspStore, path: &std::path::Path, position: Position) -> Option<CompletionResponse> {
    let text = std::fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = text.lines().collect();
    let (line, col) = (position.line as usize, position.character as usize);
    let w = want(&lines, line, col)?;
    let items = match w {
        Want::Feature => feature_items(store),
        Want::Parameter => parameter_items(store),
        Want::ParameterValue(key) => value_items(store, &key),
        Want::Stage => stage_items(),
        Want::Bool => ["true", "false"].iter().map(|b| CompletionItem { label: b.to_string(), kind: Some(CompletionItemKind::KEYWORD), ..Default::default() }).collect(),
    };
    let chars: Vec<char> = lines.get(line)?.chars().collect();
    let c = col.min(chars.len());
    let mut start = c;
    while start > 0 && is_token_char(chars[start - 1]) {
        start -= 1;
    }
    let mut end = c;
    while end < chars.len() && is_token_char(chars[end]) {
        end += 1;
    }
    // The colon that ends a mapping key is not part of the name before it.
    if end > start && chars[end - 1] == ':' && chars.get(end).is_none_or(|n| *n == ' ') {
        end -= 1;
    }
    Some(CompletionResponse::Array(with_edit(items, position.line, start as u32, end as u32)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(src: &str, line: usize, col: usize) -> Option<Want> {
        let lines: Vec<&str> = src.lines().collect();
        want(&lines, line, col)
    }

    #[test]
    fn contexts_are_recognised() {
        let src = "---\ntype: PartDef\nappliesWhen: Features::A and\nfeatures:\n  Features::B: true\nparameterBindings:\n  Features::C.x: 5\nbindingStage: load\nrequires:\n  - Features::D\n---\n";
        assert_eq!(at(src, 2, 20), Some(Want::Feature), "inside the appliesWhen expression");
        assert_eq!(at(src, 2, 4), None, "on the field name itself");
        assert_eq!(at(src, 4, 5), Some(Want::Feature), "a features: key");
        assert_eq!(at(src, 4, 20), Some(Want::Bool), "a features: value");
        assert_eq!(at(src, 6, 6), Some(Want::Parameter), "a parameterBindings key");
        assert_eq!(at(src, 6, 18), Some(Want::ParameterValue("Features::C.x".into())), "a binding value");
        assert_eq!(at(src, 7, 16), Some(Want::Stage), "a stage");
        assert_eq!(at(src, 9, 8), Some(Want::Feature), "a requires list item");
        assert_eq!(at(src, 1, 8), None, "an unrelated field");
    }
}
