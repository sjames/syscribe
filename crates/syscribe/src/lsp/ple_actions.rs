//! Quick fixes for the product-line and binding-time diagnostics (GH #421, `REQ-TRS-LSPFIX-001`).
//!
//! Offered where one fix is obvious: `E222` (the closest declared parameter or feature), `E754` (remove the
//! rebinding), `E751` (a valid stage), `E752` (remove a binding or selection the stage cannot bind), `E756` (clamp the
//! value into what was narrowed), `W016` (`selections:` → `features:`), `E201` (a derived `FEAT-*` id).
//!
//! Every fix is a `WorkspaceEdit` that is **validated in memory first**, as rename is: the edited file is re-parsed,
//! the whole candidate model is re-validated, and the fix is offered only if the diagnostic it answers is gone and no
//! new finding appears. The server never writes the file.

use std::collections::HashMap;
use std::path::Path;

use tower_lsp::lsp_types::*;

use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::validator::{validate_with_config, Finding};

use super::store::LspStore;

struct Fix {
    title: String,
    edits: Vec<TextEdit>,
    preferred: bool,
}

/// The single-quoted segments of a finding message, in order.
fn quoted(msg: &str) -> Vec<&str> {
    msg.split('\'').skip(1).step_by(2).collect()
}

/// `key: rest` split at the first colon that ends the key (a `::` inside a qualified name does not).
fn split_key(line: &str) -> Option<(usize, usize)> {
    let b = line.as_bytes();
    let i = (0..b.len()).find(|&i| b[i] == b':' && (i + 1 == b.len() || b[i + 1] == b' '))?;
    let lead = line.len() - line.trim_start().len();
    Some((lead, i))
}

fn range(line: usize, a: usize, b: usize) -> Range {
    Range::new(Position::new(line as u32, a as u32), Position::new(line as u32, b as u32))
}

/// Delete a whole line.
fn delete_line(line: usize) -> TextEdit {
    TextEdit { range: Range::new(Position::new(line as u32, 0), Position::new(line as u32 + 1, 0)), new_text: String::new() }
}

/// The line of the entry `key` inside the top-level block `section:`, with its key's column span.
fn section_entry(lines: &[&str], section: &str, key: &str, canon: &dyn Fn(&str) -> String) -> Option<(usize, usize, usize)> {
    let start = lines.iter().position(|l| l.starts_with(&format!("{section}:")))?;
    for (j, l) in lines.iter().enumerate().skip(start + 1) {
        if !l.is_empty() && !l.starts_with(char::is_whitespace) {
            break;
        }
        if let Some((lead, colon)) = split_key(l) {
            let k = l[lead..colon].trim_matches(|c| c == '"' || c == '\'');
            if canon(k) == key {
                return Some((j, lead, colon));
            }
        }
    }
    None
}

/// Apply single-line edits to `text` (ASCII-safe: columns are characters).
fn apply(text: &str, edits: &[TextEdit]) -> String {
    let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
    let mut es: Vec<&TextEdit> = edits.iter().collect();
    es.sort_by_key(|e| std::cmp::Reverse((e.range.start.line, e.range.start.character)));
    for e in es {
        let (sl, sc, el, ec) = (e.range.start.line as usize, e.range.start.character as usize, e.range.end.line as usize, e.range.end.character as usize);
        if sl >= lines.len() {
            continue;
        }
        let head: String = lines[sl].chars().take(sc).collect();
        let tail: String = if el < lines.len() { lines[el].chars().skip(ec).collect() } else { String::new() };
        let merged = format!("{head}{}{tail}", e.new_text);
        let end = el.min(lines.len() - 1);
        lines.splice(sl..=end, merged.split('\n').map(str::to_string));
    }
    lines.join("\n")
}

/// Would the candidate model, with `elem`'s file replaced by `new_text`, answer `answered` and add nothing?
fn validated(store: &LspStore, elem: &RawElement, new_text: &str, before: &[Finding], answered: &Finding) -> bool {
    let (yaml, body) = syscribe_model::frontmatter::split_frontmatter(new_text);
    let Some(yaml) = yaml else { return false };
    let Ok(fm) = syscribe_model::frontmatter::parse_frontmatter(yaml) else { return false };
    let mut candidate: Vec<RawElement> = store.elements.clone();
    let Some(slot) = candidate.iter_mut().find(|e| e.file_path == elem.file_path) else { return false };
    slot.frontmatter = fm;
    slot.doc = body.trim_start_matches('\n').to_string();
    syscribe_model::config_inherit::apply_configuration_inheritance(&mut candidate);
    let after = validate_with_config(&candidate, &store.config).findings;
    let gone = !after.iter().any(|f| f.code == answered.code && f.file == answered.file && f.message == answered.message);
    // A fix may not introduce an error. New means a (code, file) pair that was not there before (a reworded message
    // for an old one is not new); a warning the fix merely reveals — a selection that was ignored now counts and
    // leaves a required parameter unbound, say — is the model telling the truth and does not block the fix.
    let nothing_new = after.iter().all(|f| !matches!(f.severity, syscribe_model::validator::Severity::Error) || before.iter().any(|b| b.code == f.code && b.file == f.file));
    gone && nothing_new
}

fn fixes_for(store: &LspStore, elem: &RawElement, lines: &[&str], f: &Finding) -> Vec<Fix> {
    let q = quoted(&f.message);
    let alias = syscribe_model::variability::feature_id_to_qname(&store.elements);
    let canon_param = |k: &str| match k.rsplit_once('.') {
        Some((feat, p)) => format!("{}.{p}", syscribe_model::variability::canon_feature_ref(feat, &alias)),
        None => k.to_string(),
    };
    let canon_feat = |k: &str| syscribe_model::variability::canon_feature_ref(k, &alias);
    let mut out = Vec::new();
    match f.code {
        "E222" => {
            // "parameterBindings path 'P' references …; did you mean 'S'?" (or "key 'P' is not a …" with no hint)
            if let (Some(bad), Some(good)) = (q.first(), f.message.split("did you mean '").nth(1).and_then(|r| r.split('\'').next())) {
                if let Some((j, lead, colon)) = section_entry(lines, "parameterBindings", bad, &|k| k.to_string()).or_else(|| section_entry(lines, "parameterNarrowing", bad, &|k| k.to_string())) {
                    out.push(Fix { title: format!("Change to '{good}'"), edits: vec![TextEdit { range: range(j, lead, colon), new_text: good.to_string() }], preferred: true });
                }
            }
        }
        "E754" if f.message.contains("rebinds '") => {
            if let Some(key) = q.get(1) {
                if let Some((j, ..)) = section_entry(lines, "parameterBindings", key, &canon_param) {
                    out.push(Fix { title: format!("Remove the rebinding of '{key}'"), edits: vec![delete_line(j)], preferred: true });
                }
            }
        }
        "E754" if f.message.contains("sets feature '") => {
            if let Some(feat) = q.get(1) {
                if let Some((j, ..)) = section_entry(lines, "features", feat, &canon_feat) {
                    out.push(Fix { title: format!("Remove the selection of '{feat}'"), edits: vec![delete_line(j)], preferred: true });
                }
            }
        }
        "E751" => {
            if let Some(bad) = q.get(1) {
                if let Some(j) = lines.iter().position(|l| l.starts_with("bindingStage:")) {
                    let colon = lines[j].find(':').unwrap_or(0);
                    let value_start = colon + 1 + lines[j][colon + 1..].len() - lines[j][colon + 1..].trim_start().len();
                    let stages: Vec<String> = syscribe_model::binding_time::active().stages().iter().map(|s| s.name.clone()).collect();
                    let closest = syscribe_model::variability::closest_name(bad, stages.iter().map(String::as_str)).map(str::to_string);
                    for s in &stages {
                        out.push(Fix {
                            title: format!("Use bindingStage '{s}'"),
                            edits: vec![TextEdit { range: range(j, value_start, lines[j].chars().count()), new_text: s.clone() }],
                            preferred: closest.as_deref() == Some(s.as_str()),
                        });
                    }
                }
            }
        }
        "E752" if f.message.contains("parameterBindings binds '") => {
            if let (Some(key), Some(stage)) = (q.first(), f.message.split("bindingStage ").nth(1).and_then(|r| r.split(',').next())) {
                if let Some((j, ..)) = section_entry(lines, "parameterBindings", key, &canon_param) {
                    out.push(Fix { title: format!("Remove the binding of '{key}' (it cannot be bound at {stage})"), edits: vec![delete_line(j)], preferred: false });
                }
            }
        }
        "E752" if f.message.contains("selects feature '") => {
            if let Some(feat) = q.get(1) {
                if let Some((j, ..)) = section_entry(lines, "features", feat, &canon_feat) {
                    out.push(Fix { title: format!("Remove the selection of '{feat}' (it cannot be bound at this stage)"), edits: vec![delete_line(j)], preferred: false });
                }
            }
        }
        "E756" => {
            // "parameterBindings 'K' = V is outside LO..HI — what the parameter was narrowed to …"
            if let Some(key) = q.first() {
                let bound = f.message.split(" is outside ").nth(1).and_then(|r| r.split(" — ").next()).unwrap_or("").trim();
                let nums: Vec<f64> = bound.trim_matches(|c| c == '[' || c == ']' || c == '(' || c == ')').split("..").filter_map(|x| x.trim().parse().ok()).collect();
                let value: Option<f64> = f.message.split(" = ").nth(1).and_then(|r| r.split(' ').next()).and_then(|v| v.parse().ok());
                if let (Some((lo, hi)), Some(v)) = (nums.first().zip(nums.get(1)), value) {
                    let target = v.clamp(*lo, *hi);
                    if let Some((j, _, colon)) = section_entry(lines, "parameterBindings", key, &canon_param) {
                        let line = lines[j];
                        let from = colon + 1 + line[colon + 1..].len() - line[colon + 1..].trim_start().len();
                        let text = if target.fract() == 0.0 { format!("{}", target as i64) } else { format!("{target}") };
                        out.push(Fix { title: format!("Change the value to {text} (inside {bound})"), edits: vec![TextEdit { range: range(j, from, line.chars().count()), new_text: text }], preferred: true });
                    }
                }
            }
        }
        "W016" => {
            if let Some(j) = lines.iter().position(|l| l.starts_with("selections:")) {
                out.push(Fix { title: "Rename `selections:` to `features:`".into(), edits: vec![TextEdit { range: range(j, 0, "selections".len()), new_text: "features".into() }], preferred: true });
            }
        }
        "E201" if f.message.contains("FEAT-*") && elem.frontmatter.element_type == Some(ElementType::FeatureDef) => {
            let segs: Vec<String> = elem.qualified_name.split("::").map(|s| s.chars().filter(|c| c.is_ascii_alphanumeric()).take(12).collect::<String>().to_uppercase()).collect();
            // The leaf name, then with more of the path until the id is free.
            for take in 1..=segs.len() {
                let id = format!("FEAT-{}", segs[segs.len() - take..].join("-"));
                if store.elements.iter().any(|e| e.frontmatter.id.as_deref() == Some(id.as_str())) {
                    continue;
                }
                out.push(Fix { title: format!("Add id: {id}"), edits: vec![TextEdit { range: range(1, 0, 0), new_text: format!("id: {id}\n") }], preferred: true });
                break;
            }
        }
        _ => {}
    }
    out
}

pub(super) fn quick_fixes(store: &LspStore, path: &Path) -> Vec<CodeActionOrCommand> {
    let path_str = path.to_string_lossy().to_string();
    let Some(elem) = store.elements.iter().find(|e| e.file_path == path_str) else { return vec![] };
    let Ok(text) = std::fs::read_to_string(path) else { return vec![] };
    let lines: Vec<&str> = text.lines().collect();
    let all = validate_with_config(&store.elements, &store.config).findings;
    let mine: Vec<&Finding> = all.iter().filter(|f| f.file == path_str && matches!(f.code, "E222" | "E754" | "E751" | "E752" | "E756" | "W016" | "E201")).collect();
    if mine.is_empty() {
        return vec![];
    }
    let Some(uri) = super::path_to_uri(&path_str) else { return vec![] };
    let mut out = Vec::new();
    for f in mine {
        let mut kept: Vec<Fix> = fixes_for(store, elem, &lines, f).into_iter().filter(|fix| validated(store, elem, &apply(&text, &fix.edits), &all, f)).collect();
        // The only fix that survives is the obvious one.
        if kept.len() == 1 {
            kept[0].preferred = true;
        }
        for fix in kept {
            let mut changes = HashMap::new();
            changes.insert(uri.clone(), fix.edits);
            out.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: fix.title,
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: None,
                edit: Some(WorkspaceEdit { changes: Some(changes), ..Default::default() }),
                command: None,
                is_preferred: Some(fix.preferred),
                disabled: None,
                data: Some(serde_json::json!({ "code": f.code })),
            }));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_apply_in_place() {
        let t = "a: 1\nb: 2\nc: 3";
        assert_eq!(apply(t, &[delete_line(1)]), "a: 1\nc: 3");
        assert_eq!(apply(t, &[TextEdit { range: range(0, 3, 4), new_text: "9".into() }]), "a: 9\nb: 2\nc: 3");
        assert_eq!(apply(t, &[TextEdit { range: range(1, 0, 0), new_text: "x: 0\n".into() }]), "a: 1\nx: 0\nb: 2\nc: 3");
    }

    #[test]
    fn messages_are_quoted_apart() {
        let q = quoted("Configuration 'C' rebinds 'F.p' — already bound by 'B' (bindingStage compile)");
        assert_eq!(q, ["C", "F.p", "B"]);
    }
}
