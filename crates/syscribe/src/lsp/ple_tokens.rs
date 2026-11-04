//! `textDocument/semanticTokens/full` for the variability fields (GH #422, `REQ-TRS-LSPTOK-001`).
//!
//! Colours feature references, `FEAT-*` ids, parameter paths and stage names in frontmatter expressions and keys, so
//! a typo in `appliesWhen:` or `parameterConstraints:` is visible before validation runs. A reference that resolves to
//! nothing carries the `unresolved` modifier.
//!
//! Legend (also in `syscribe help lsp`):
//!
//! | index | token type   | used for                                         |
//! |------:|--------------|--------------------------------------------------|
//! | 0     | `class`      | a feature, by qualified name                     |
//! | 1     | `enumMember` | a feature, by `FEAT-*` id                        |
//! | 2     | `property`   | a parameter path `<feature>.<parameter>`         |
//! | 3     | `keyword`    | a binding-time stage name                        |
//!
//! | bit | modifier     | meaning                                          |
//! |----:|--------------|--------------------------------------------------|
//! | 0   | `unresolved` | names no feature, parameter or stage of the model |

use std::collections::HashSet;
use std::path::Path;

use tower_lsp::lsp_types::*;

use syscribe_model::binding_time::active;
use syscribe_model::element::ElementType;

use super::store::LspStore;

pub(super) const TYPES: &[&str] = &["class", "enumMember", "property", "keyword"];
pub(super) const MODIFIERS: &[&str] = &["unresolved"];

pub(super) fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TYPES.iter().map(|t| SemanticTokenType::new(t)).collect(),
        token_modifiers: MODIFIERS.iter().map(|m| SemanticTokenModifier::new(m)).collect(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Feature,
    Param,
    Stage,
}

fn is_token_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | ':' | '.' | '-')
}

const WORDS: &[&str] = &["and", "or", "not", "true", "false", "AND", "OR", "NOT", "xor", "implies"];

/// `(start, end)` char spans of the tokens of `s` that could be references.
fn spans(s: &str) -> Vec<(usize, usize)> {
    let chars: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if is_token_char(chars[i]) {
            let a = i;
            while i < chars.len() && is_token_char(chars[i]) {
                i += 1;
            }
            // A mapping key's colon is not part of the name before it.
            let mut b = i;
            if chars[b - 1] == ':' && chars.get(b).is_none_or(|c| *c == ' ') {
                b -= 1;
            }
            let t: String = chars[a..b].iter().collect();
            if b > a && !WORDS.contains(&t.as_str()) && t.parse::<f64>().is_err() {
                out.push((a, b));
            }
        } else {
            i += 1;
        }
    }
    out
}

fn entry_key(line: &str) -> Option<(String, usize)> {
    let b = line.as_bytes();
    let i = (0..b.len()).find(|&i| b[i] == b':' && (i + 1 == b.len() || b[i + 1] == b' '))?;
    let key = line[..i].trim_start().trim_start_matches("- ").trim().trim_matches(|c| c == '"' || c == '\'');
    Some((key.to_string(), i))
}

struct Vocab {
    features: HashSet<String>,
    ids: HashSet<String>,
    params: HashSet<String>,
    stages: HashSet<String>,
}

fn vocab(store: &LspStore) -> Vocab {
    let mut v = Vocab { features: HashSet::new(), ids: HashSet::new(), params: HashSet::new(), stages: active().stages().iter().map(|s| s.name.clone()).collect() };
    for fd in store.elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::FeatureDef)) {
        v.features.insert(fd.qualified_name.clone());
        if let Some(id) = &fd.frontmatter.id {
            v.ids.insert(id.clone());
        }
        for p in fd.frontmatter.parameters.iter().flatten() {
            if let Some(n) = p.get("name").and_then(|n| n.as_str()) {
                v.params.insert(format!("{}.{n}", fd.qualified_name));
            }
        }
    }
    v
}

/// Which part of a line holds references, and of what kind: `(kinds, from, to)` in characters. `entry` is the line's
/// own `key:` (its name and the column of its colon); `field` is the enclosing top-level key.
fn region(field: &str, entry: Option<(String, usize)>, top_level: bool, len: usize) -> Option<(Vec<Kind>, usize, usize)> {
    match entry {
        Some((inner, colon)) => {
            // The value of a stage, an `appliesWhen:` or a constraint's `expression:`.
            if matches!(inner.as_str(), "bindingTime" | "latestBindingTime" | "bindingStage" | "appliesWhen" | "expression") {
                return line_kind(field, Some(&inner), true).map(|k| (k, colon + 1, len));
            }
            // A mapping whose keys are the references.
            if !top_level && matches!(field, "features" | "parameterBindings" | "parameterNarrowing") {
                return line_kind(field, Some(&inner), false).map(|k| (k, 0, colon));
            }
            // An inline `requires: [..]` / `excludes: [..]`.
            if matches!(field, "requires" | "excludes") {
                return line_kind(field, Some(&inner), false).map(|k| (k, colon + 1, len));
            }
            None
        }
        // A list item or continuation line of one of the expression fields.
        None => line_kind(field, None, false).filter(|_| matches!(field, "requires" | "excludes" | "appliesWhen")).map(|k| (k, 0, len)),
    }
}

/// What the tokens of one line are, given the frontmatter field they sit in.
fn line_kind(field: &str, inner: Option<&str>, in_value: bool) -> Option<Vec<Kind>> {
    if in_value {
        match inner {
            Some("bindingTime" | "latestBindingTime" | "bindingStage") => return Some(vec![Kind::Stage]),
            Some("appliesWhen") => return Some(vec![Kind::Feature]),
            Some("expression") if field == "parameterConstraints" => return Some(vec![Kind::Param, Kind::Feature]),
            _ => {}
        }
    }
    match field {
        "appliesWhen" | "requires" | "excludes" => Some(vec![Kind::Feature]),
        "features" if !in_value => Some(vec![Kind::Feature]),
        "parameterBindings" | "parameterNarrowing" if !in_value => Some(vec![Kind::Param]),
        _ => None,
    }
}

pub(super) fn tokens(store: &LspStore, path: &Path) -> Vec<SemanticToken> {
    let Ok(text) = std::fs::read_to_string(path) else { return vec![] };
    let lines: Vec<&str> = text.lines().collect();
    if lines.first().copied() != Some("---") {
        return vec![];
    }
    let Some(close) = lines.iter().enumerate().skip(1).find(|(_, l)| **l == "---").map(|(i, _)| i) else { return vec![] };
    let v = vocab(store);
    // (line, start, len, type, modifiers)
    let mut raw: Vec<(u32, u32, u32, u32, u32)> = Vec::new();
    let mut field = String::new();
    for (i, l) in lines.iter().enumerate().take(close).skip(1) {
        if !l.starts_with(char::is_whitespace) && !l.trim().is_empty() {
            if let Some((k, _)) = entry_key(l) {
                field = k;
            }
        }
        let top_level = !l.starts_with(char::is_whitespace);
        let Some((kinds, from, to)) = region(&field, entry_key(l), top_level, l.chars().count()) else { continue };
        let chars: Vec<char> = l.chars().collect();
        let slice: String = chars[from.min(chars.len())..to.min(chars.len())].iter().collect();
        for (a, b) in spans(&slice) {
            let tok: String = slice.chars().skip(a).take(b - a).collect();
            let col = from + a;
            let (ty, unresolved) = classify(&v, &tok, &kinds);
            let Some(ty) = ty else { continue };
            // UTF-16 column.
            let col16: usize = chars[..col].iter().map(|c| c.len_utf16()).sum();
            let len16: usize = tok.chars().map(|c| c.len_utf16()).sum();
            raw.push((i as u32, col16 as u32, len16 as u32, ty, unresolved as u32));
        }
    }
    raw.sort();
    let (mut pl, mut pc) = (0u32, 0u32);
    raw.into_iter()
        .map(|(line, col, len, ty, m)| {
            let dl = line - pl;
            let ds = if dl == 0 { col - pc } else { col };
            (pl, pc) = (line, col);
            SemanticToken { delta_line: dl, delta_start: ds, length: len, token_type: ty, token_modifiers_bitset: m }
        })
        .collect()
}

/// The token type index (`None` to skip) and whether the reference is unresolved.
fn classify(v: &Vocab, tok: &str, kinds: &[Kind]) -> (Option<u32>, bool) {
    if kinds.contains(&Kind::Stage) {
        return (Some(3), !v.stages.contains(tok));
    }
    if tok.starts_with("FEAT-") {
        return (Some(1), !v.ids.contains(tok));
    }
    // `Features::Motor.kv` is a parameter path; `Features::Motor` a feature.
    let looks_path = tok.contains("::") || tok.contains('.');
    if kinds.contains(&Kind::Param) && looks_path && tok.rsplit_once('.').is_some_and(|(f, _)| f.contains("::") || v.features.contains(f) || v.ids.contains(f)) {
        return (Some(2), !v.params.contains(tok) && !tok.rsplit_once('.').is_some_and(|(f, p)| v.ids.contains(f) && v.params.iter().any(|x| x.ends_with(&format!(".{p}")))));
    }
    if kinds.contains(&Kind::Feature) && (tok.contains("::") || v.features.contains(tok)) {
        return (Some(0), !v.features.contains(tok));
    }
    (None, false)
}
