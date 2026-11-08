//! Requirement item anchors and item / question references in prose (GH #453,
//! `REQ-TRS-ITEMREF-001..006`).
//!
//! **Items.** A Requirement body is often a numbered list. Every *top-level* ordered-list
//! item (a line starting at column 0 with `<digits>.` or `<digits>)`, outside fenced code)
//! is an **item**, numbered by position across the whole body (1-based; the number the
//! author typed is ignored). An item may begin with a stable **anchor** `{#slug}`
//! (`slug` = `^[a-z][a-z0-9-]*$`):
//!
//! ```markdown
//! 1. {#grant-keys} **Server keys.** The server shall …
//! ```
//!
//! An item's text runs from its marker line to the next top-level item, a heading or fence
//! at column 0, or a column-0 line after a blank line (the end of the list).
//!
//! **References.** In the body of any element, outside fenced code blocks:
//! `REQ-X#grant-keys` (item by anchor), `REQ-X#5` (item by number), `REQ-X item 5` (legacy,
//! case-insensitive `item`) and `PI-Y#Q31` (a PlanningItem question, GH #436). Only a
//! reference whose element part resolves (by stable id or qualified name) to a
//! `Requirement` (item forms) or a `PlanningItem` (`#Q<n>`) is checked; anything else is
//! left alone, so an unresolved id is never reported twice.
//!
//! **Findings.** `W092` dangling item/question reference, or an anchor defined twice /
//! malformed in one Requirement; `W093` a committed element (status past draft, see
//! [`is_committed_status`]) cites a question that is still `open`; `W094` a citation of an
//! `answered` question with "to confirm"-style wording in the same sentence (phrases from
//! `[references] open_markers`, defaults [`DEFAULT_OPEN_MARKERS`]).
//!
//! **Item-granular suspect links.** A `traceBaselines:` key `REQ-X#slug` stores
//! [`item_hash`] of that one item, so editing a different item never flips it (`W090`).

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::validator::{Finding, Severity};

/// Phrases that mark a citation as still awaiting an answer (`W094`), matched
/// case-insensitively on word boundaries within the citing sentence.
pub const DEFAULT_OPEN_MARKERS: &[&str] =
    &["to confirm", "to be confirmed", "TBC", "pending confirmation", "awaiting"];

/// The `[references]` table of `.syscribe.toml`.
#[derive(Debug, Clone, Default)]
pub struct ReferencesConfig {
    /// `open_markers`; `None` = [`DEFAULT_OPEN_MARKERS`]. An empty list disables `W094`.
    pub open_markers: Option<Vec<String>>,
    /// `copied_values = true` (GH #452): `W095` for a literal copy of a linked element's
    /// attribute value. Off by default.
    pub copied_values: bool,
    /// Malformed entries (reported as `W094` against `.syscribe.toml`; the entry is ignored).
    pub problems: Vec<String>,
}

impl ReferencesConfig {
    /// Load `[references]` from `<model_root>/.syscribe.toml`.
    pub fn load(model_root: &Path) -> Self {
        let mut cfg = ReferencesConfig::default();
        let Some(root) = std::fs::read_to_string(model_root.join(".syscribe.toml"))
            .ok()
            .and_then(|t| toml::from_str::<toml::Value>(&t).ok())
        else {
            return cfg;
        };
        let Some(tbl) = root.get("references") else { return cfg };
        let Some(tbl) = tbl.as_table() else {
            cfg.problems.push("[references] must be a table".into());
            return cfg;
        };
        for (k, v) in tbl {
            match k.as_str() {
                "open_markers" | "openMarkers" => match v.as_array() {
                    Some(arr) if arr.iter().all(|x| x.as_str().is_some_and(|s| !s.trim().is_empty())) => {
                        cfg.open_markers = Some(arr.iter().filter_map(|x| x.as_str()).map(|s| s.trim().to_string()).collect());
                    }
                    _ => cfg.problems.push(format!(
                        "[references] {k} must be a list of non-empty strings, got {v}; the default phrases are used"
                    )),
                },
                "copied_values" | "copiedValues" => match v.as_bool() {
                    Some(b) => cfg.copied_values = b,
                    None => cfg.problems.push(format!(
                        "[references] {k} must be true or false, got {v}; W095 stays off"
                    )),
                },
                other => cfg.problems.push(format!("[references] unknown key '{other}' (expected open_markers, copied_values)")),
            }
        }
        cfg
    }

    /// The active marker phrases.
    pub fn markers(&self) -> Vec<String> {
        match &self.open_markers {
            Some(m) => m.clone(),
            None => DEFAULT_OPEN_MARKERS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// One top-level ordered-list item of a Requirement body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// 1-based position across the body.
    pub number: usize,
    /// The `{#slug}` anchor, when present and well formed.
    pub anchor: Option<String>,
    /// The item's text with the marker kept and the anchor token removed.
    pub text: String,
    /// 0-based line of the marker within the body.
    pub line: usize,
}

/// `^[a-z][a-z0-9-]*$`.
pub fn is_slug(s: &str) -> bool {
    let mut b = s.bytes();
    matches!(b.next(), Some(c) if c.is_ascii_lowercase())
        && b.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

fn is_fence(line: &str) -> bool {
    let indent = line.len() - line.trim_start().len();
    let t = line.trim_start();
    indent < 4 && (t.starts_with("```") || t.starts_with("~~~"))
}

/// A column-0 ordered-list marker: `(marker_len, rest)` where `rest` follows the marker's
/// whitespace.
fn ordered_marker(line: &str) -> Option<(usize, &str)> {
    let digits = line.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digits == 0 || digits > 9 {
        return None;
    }
    let after = &line[digits..];
    let delim = after.chars().next()?;
    if delim != '.' && delim != ')' {
        return None;
    }
    let after = &after[1..];
    if !(after.is_empty() || after.starts_with(' ') || after.starts_with('\t')) {
        return None;
    }
    let rest = after.trim_start();
    Some((line.len() - rest.len(), rest))
}

/// An anchor token at the start of an item's text: `Ok(Some((slug, token_len_incl_ws)))`,
/// `Ok(None)` when there is none, `Err(token)` when it is malformed (`{#Bad}`).
fn anchor_token(rest: &str) -> Result<Option<(String, usize)>, String> {
    let Some(inner) = rest.strip_prefix("{#") else { return Ok(None) };
    let Some(close) = inner.find('}') else { return Ok(None) };
    let slug = &inner[..close];
    let token = &rest[..close + 3];
    if !is_slug(slug) {
        return Err(token.to_string());
    }
    let after = &rest[close + 3..];
    let ws = after.len() - after.trim_start_matches([' ', '\t']).len();
    Ok(Some((slug.to_string(), close + 3 + ws)))
}

/// The items of a body and the anchor problems found (`W092` messages without the owner).
pub fn parse_items(doc: &str) -> (Vec<Item>, Vec<String>) {
    let lines: Vec<&str> = doc.lines().collect();
    let mut items: Vec<Item> = Vec::new();
    let mut problems = Vec::new();
    let mut in_fence = false;
    // Current item's collected lines.
    let mut cur: Option<(Item, Vec<String>)> = None;
    let mut prev_blank = false;
    let finish = |cur: &mut Option<(Item, Vec<String>)>, items: &mut Vec<Item>| {
        if let Some((mut it, ls)) = cur.take() {
            it.text = ls.join("\n").trim_end().to_string();
            items.push(it);
        }
    };
    for (i, line) in lines.iter().enumerate() {
        if is_fence(line) {
            if !in_fence && !line.starts_with(' ') && !line.starts_with('\t') {
                finish(&mut cur, &mut items);
            }
            in_fence = !in_fence;
            if let Some((_, ls)) = cur.as_mut() {
                ls.push(line.to_string());
            }
            prev_blank = false;
            continue;
        }
        if in_fence {
            if let Some((_, ls)) = cur.as_mut() {
                ls.push(line.to_string());
            }
            continue;
        }
        if let Some((mlen, rest)) = ordered_marker(line) {
            finish(&mut cur, &mut items);
            let number = items.len() + 1;
            let (anchor, text) = match anchor_token(rest) {
                Ok(Some((slug, tlen))) => (Some(slug), format!("{}{}", &line[..mlen], &rest[tlen..])),
                Ok(None) => (None, line.to_string()),
                Err(tok) => {
                    problems.push(format!("item {number} has a malformed anchor `{tok}` — an anchor is `{{#slug}}` with slug `[a-z][a-z0-9-]*`"));
                    (None, line.to_string())
                }
            };
            cur = Some((Item { number, anchor, text: String::new(), line: i }, vec![text]));
            prev_blank = false;
            continue;
        }
        let blank = line.trim().is_empty();
        if cur.is_some() {
            let col0 = !blank && !line.starts_with(' ') && !line.starts_with('\t');
            if col0 && (prev_blank || line.starts_with('#')) {
                finish(&mut cur, &mut items);
            } else if let Some((_, ls)) = cur.as_mut() {
                ls.push(line.to_string());
            }
        }
        prev_blank = blank;
    }
    finish(&mut cur, &mut items);
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for it in &items {
        if let Some(a) = it.anchor.as_deref() {
            if let Some(first) = seen.get(a) {
                problems.push(format!("anchor `{{#{a}}}` is defined twice (items {first} and {}) — anchors must be unique within a requirement", it.number));
            } else {
                seen.insert(a, it.number);
            }
        }
    }
    (items, problems)
}

/// The body with every item anchor token removed (for display); numbers are kept.
pub fn strip_anchors(doc: &str) -> String {
    rewrite_anchors(doc, |_| String::new())
}

/// The body with every item anchor token replaced by `f(slug)` (export-html uses this to
/// emit an HTML anchor).
pub fn rewrite_anchors(doc: &str, f: impl Fn(&str) -> String) -> String {
    let mut out = String::with_capacity(doc.len());
    let mut in_fence = false;
    for line in doc.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if is_fence(bare) {
            in_fence = !in_fence;
            out.push_str(line);
            continue;
        }
        if !in_fence {
            if let Some((mlen, rest)) = ordered_marker(bare) {
                if let Ok(Some((slug, tlen))) = anchor_token(rest) {
                    out.push_str(&line[..mlen]);
                    out.push_str(&f(&slug));
                    out.push_str(&line[mlen + tlen..]);
                    continue;
                }
            }
        }
        out.push_str(line);
    }
    out
}

/// `blake3:<hex>` of one item's text (anchor token removed, line ends normalized,
/// trailing whitespace trimmed) — the `traceBaselines` value for a `REQ-X#slug` key.
pub fn item_hash(item: &Item) -> String {
    let norm: Vec<&str> = item.text.lines().map(str::trim_end).collect();
    let text = norm.join("\n");
    // The marker's digits are positional: drop them so an insertion above does not
    // change the hash of an unchanged item.
    let text = match ordered_marker(&text) {
        Some((mlen, _)) => text[mlen..].to_string(),
        None => text,
    };
    format!("blake3:{}", blake3::hash(text.trim().as_bytes()).to_hex())
}

/// The part after the element in a reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Part {
    /// `#slug`
    Anchor(String),
    /// `#5` or ` item 5`
    Number(usize),
    /// `#Q31`
    Question(String),
}

impl Part {
    /// The canonical suffix: `#slug`, `#5`, `#Q31`.
    pub fn suffix(&self) -> String {
        match self {
            Part::Anchor(s) => format!("#{s}"),
            Part::Number(n) => format!("#{n}"),
            Part::Question(q) => format!("#{q}"),
        }
    }
}

/// Classify the text after `#`.
fn classify(frag: &str) -> Option<Part> {
    if !frag.is_empty() && frag.bytes().all(|b| b.is_ascii_digit()) {
        return frag.parse().ok().map(Part::Number);
    }
    if crate::questions::is_question_id(frag) {
        return Some(Part::Question(frag.to_string()));
    }
    let slug = frag.trim_end_matches('-');
    if is_slug(slug) {
        return Some(Part::Anchor(slug.to_string()));
    }
    None
}

/// Parse a reference key given on the command line: `REQ-X#slug`, `REQ-X#5`, `PI-Y#Q3`,
/// `REQ-X item 5`. `None` when it has no item/question part.
pub fn parse_ref(key: &str) -> Option<(String, Part)> {
    let key = key.trim();
    if let Some((elem, frag)) = key.split_once('#') {
        if elem.is_empty() {
            return None;
        }
        return classify(frag).filter(|p| !matches!(p, Part::Anchor(s) if s.len() != frag.len())).map(|p| (elem.to_string(), p));
    }
    let lower = key.to_ascii_lowercase();
    let pos = lower.rfind(" item ")?;
    let n: usize = key[pos + 6..].trim().parse().ok()?;
    Some((key[..pos].trim().to_string(), Part::Number(n)))
}

/// One reference found in prose.
#[derive(Debug, Clone)]
pub struct Citation {
    /// The element part as written.
    pub elem: String,
    pub part: Part,
    /// The whole reference as written (`REQ-X#slug`, `REQ-X item 5`).
    pub raw: String,
    /// 0-based line within the scanned text.
    pub line: usize,
    /// The sentence of the line containing the reference (for `W094`).
    pub sentence: String,
}

static CITE_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

fn cite_re() -> &'static regex::Regex {
    CITE_RE.get_or_init(|| {
        regex::Regex::new(
            r"(?:^|[^A-Za-z0-9_\-])([A-Z][A-Z0-9]*(?:-[A-Z0-9]+)+)(?:#([A-Za-z0-9][A-Za-z0-9\-]*)|[ \t]+(?i:item)[ \t]+([0-9]+)\b)",
        )
        .expect("citation regex")
    })
}

/// The sentence of `line` around byte range `start..end`.
fn sentence_around(line: &str, start: usize, end: usize) -> String {
    let is_term = |i: usize| {
        let b = line.as_bytes();
        matches!(b[i], b'.' | b'!' | b'?' | b';') && (i + 1 >= b.len() || b[i + 1].is_ascii_whitespace())
    };
    let mut s = 0;
    for i in (0..start).rev() {
        if is_term(i) {
            s = i + 1;
            break;
        }
    }
    let mut e = line.len();
    for i in end..line.len() {
        if is_term(i) {
            e = i;
            break;
        }
    }
    line[s..e].trim().to_string()
}

/// Every reference in `text`, outside fenced code blocks.
pub fn scan_citations(text: &str) -> Vec<Citation> {
    let mut out = Vec::new();
    let mut in_fence = false;
    for (ln, line) in text.lines().enumerate() {
        if is_fence(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        for cap in cite_re().captures_iter(line) {
            let Some(id) = cap.get(1) else { continue };
            let (part, end) = if let Some(frag) = cap.get(2) {
                match classify(frag.as_str()) {
                    Some(p) => {
                        let end = match &p {
                            Part::Anchor(s) => frag.start() + s.len(),
                            _ => frag.end(),
                        };
                        (p, end)
                    }
                    None => continue,
                }
            } else if let Some(n) = cap.get(3) {
                match n.as_str().parse() {
                    Ok(v) => (Part::Number(v), n.end()),
                    Err(_) => continue,
                }
            } else {
                continue;
            };
            out.push(Citation {
                elem: id.as_str().to_string(),
                part,
                raw: line[id.start()..end].to_string(),
                line: ln,
                sentence: sentence_around(line, id.start(), end),
            });
        }
    }
    out
}

/// Resolve the element part of a reference: by stable id or exact qualified name only
/// (never the display-name fallback).
pub fn resolve_elem<'a>(elements: &'a [RawElement], resolver: &Resolver, r: &str) -> Option<&'a RawElement> {
    resolver
        .resolve_ref(elements, r)
        .filter(|e| e.frontmatter.id.as_deref() == Some(r) || e.qualified_name == r)
        .or_else(|| elements.iter().find(|e| e.frontmatter.id.as_deref() == Some(r)))
}

/// Whether `part` is a kind of reference `target` can carry (items on a Requirement,
/// questions on a PlanningItem). Other combinations are not checked.
pub fn applies(target: &RawElement, part: &Part) -> bool {
    match (target.frontmatter.element_type.as_ref(), part) {
        (Some(ElementType::Requirement), Part::Anchor(_) | Part::Number(_)) => true,
        (Some(ElementType::PlanningItem), Part::Question(_)) => true,
        _ => false,
    }
}

/// What a reference resolves to.
#[derive(Debug, Clone)]
pub enum Target {
    Item(Item),
    Question(crate::questions::Question),
}

/// Look up `part` on `target`: `Ok` with the item/question, `Err` with why it is missing.
pub fn lookup(target: &RawElement, part: &Part) -> Result<Target, String> {
    let label = target.frontmatter.id.clone().unwrap_or_else(|| target.qualified_name.clone());
    match part {
        Part::Anchor(slug) => {
            let (items, _) = parse_items(&target.doc);
            if let Some(it) = items.iter().find(|i| i.anchor.as_deref() == Some(slug.as_str())) {
                return Ok(Target::Item(it.clone()));
            }
            let anchors: Vec<String> = items.iter().filter_map(|i| i.anchor.clone()).collect();
            Err(if anchors.is_empty() {
                format!("{label} defines no item anchors — add `{{#{slug}}}` after the item's number, or cite the item by number")
            } else {
                format!("{label} has no item anchor `{slug}` (anchors: {})", anchors.join(", "))
            })
        }
        Part::Number(n) => {
            let (items, _) = parse_items(&target.doc);
            match items.iter().find(|i| i.number == *n) {
                Some(it) => Ok(Target::Item(it.clone())),
                None => Err(format!("{label} has {} numbered item(s), so item {n} does not exist", items.len())),
            }
        }
        Part::Question(q) => {
            let (qs, _) = crate::questions::parse(target);
            match qs.into_iter().find(|x| &x.id == q) {
                Some(x) => Ok(Target::Question(x)),
                None => Err(format!("{label} has no question {q}")),
            }
        }
    }
}

/// An element citing something, with the citations.
#[derive(Debug, Clone)]
pub struct Citer<'a> {
    pub source: &'a RawElement,
    pub citation: Citation,
    /// The resolved target's qualified name.
    pub target_qname: String,
}

/// Every checked citation in the model (element part resolves and the part applies).
pub fn all_citations<'a>(elements: &'a [RawElement], resolver: &Resolver) -> Vec<Citer<'a>> {
    let mut out = Vec::new();
    for e in elements {
        if e.doc.is_empty() {
            continue;
        }
        for c in scan_citations(&e.doc) {
            let Some(t) = resolve_elem(elements, resolver, &c.elem) else { continue };
            if !applies(t, &c.part) {
                continue;
            }
            out.push(Citer { source: e, target_qname: t.qualified_name.clone(), citation: c });
        }
    }
    out
}

/// Elements citing `target_qname` — at `part`, or any item/question when `None`. A by-number
/// and a by-anchor citation of the same item both match either form.
pub fn citers_of<'a>(
    elements: &'a [RawElement],
    resolver: &Resolver,
    target_qname: &str,
    part: Option<&Part>,
) -> Vec<Citer<'a>> {
    let target = elements.iter().find(|e| e.qualified_name == target_qname);
    let wanted_number = match (target, part) {
        (Some(t), Some(p @ (Part::Anchor(_) | Part::Number(_)))) => match lookup(t, p) {
            Ok(Target::Item(it)) => Some(it.number),
            _ => None,
        },
        _ => None,
    };
    let number_of = |p: &Part| -> Option<usize> {
        match (target, p) {
            (Some(t), Part::Anchor(_) | Part::Number(_)) => match lookup(t, p) {
                Ok(Target::Item(it)) => Some(it.number),
                _ => None,
            },
            _ => None,
        }
    };
    all_citations(elements, resolver)
        .into_iter()
        .filter(|c| c.target_qname == target_qname)
        .filter(|c| match part {
            None => true,
            Some(p) => {
                &c.citation.part == p
                    || (wanted_number.is_some() && number_of(&c.citation.part) == wanted_number)
            }
        })
        .collect()
}

/// Statuses at which an element is not yet committed: citing an open question is
/// expected there. Anything else (`review`, `approved`, `implemented`, `verified`,
/// `active`, `accepted`, `done`, …) is committed and `W093` applies. No status at all is
/// treated as uncommitted. Retiring statuses are exempt too (the element is going away).
pub fn is_committed_status(status: Option<&str>) -> bool {
    const UNCOMMITTED: &[&str] = &[
        "draft", "proposed", "todo", "in_progress", "blocked", "open",
        "deprecated", "superseded", "retired", "obsolete", "withdrawn", "rejected",
    ];
    match status {
        None => false,
        Some(s) => !UNCOMMITTED.contains(&s),
    }
}

/// Whether `sentence` contains one of `markers` on word boundaries (case-insensitive).
pub fn open_marker_in(sentence: &str, markers: &[String]) -> Option<String> {
    let hay = sentence.to_lowercase();
    for m in markers {
        let needle = m.to_lowercase();
        if needle.is_empty() {
            continue;
        }
        let mut from = 0;
        while let Some(pos) = hay[from..].find(&needle) {
            let s = from + pos;
            let e = s + needle.len();
            let before_ok = s == 0 || !hay[..s].chars().next_back().is_some_and(|c| c.is_alphanumeric());
            let after_ok = e >= hay.len() || !hay[e..].chars().next().is_some_and(|c| c.is_alphanumeric());
            if before_ok && after_ok {
                return Some(m.clone());
            }
            from = s + needle.len().max(1);
        }
    }
    None
}

fn label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

fn warn(code: &'static str, file: &str, message: String) -> Finding {
    Finding { code, file: file.to_string(), message, severity: Severity::Warning }
}

/// `W092`–`W094` over the model (see the module docs).
pub fn findings(elements: &[RawElement], resolver: &Resolver, cfg: &ReferencesConfig, config_file: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for p in &cfg.problems {
        out.push(warn("W094", config_file, p.clone()));
    }
    let markers = cfg.markers();
    // Anchor problems on each Requirement.
    for e in elements {
        if e.frontmatter.element_type != Some(ElementType::Requirement) || e.doc.is_empty() {
            continue;
        }
        for p in parse_items(&e.doc).1 {
            out.push(warn("W092", &e.file_path, format!("{}: {p}", label(e))));
        }
    }
    for c in all_citations(elements, resolver) {
        let src = c.source;
        let Some(target) = elements.iter().find(|e| e.qualified_name == c.target_qname) else { continue };
        let mut seen: HashSet<(&'static str, String)> = HashSet::new();
        let mut push = |code: &'static str, msg: String, out: &mut Vec<Finding>| {
            if seen.insert((code, c.citation.raw.clone())) {
                out.push(warn(code, &src.file_path, msg));
            }
        };
        match lookup(target, &c.citation.part) {
            Err(why) => push(
                "W092",
                format!("{} cites `{}`, which does not exist: {why}", label(src), c.citation.raw),
                &mut out,
            ),
            Ok(Target::Item(_)) => {}
            Ok(Target::Question(q)) => {
                if std::ptr::eq(src, target) {
                    continue; // an item mentioning its own question; W317 covers `done`
                }
                let status = src.frontmatter.status.as_deref();
                if q.status == "open" && is_committed_status(status) {
                    push(
                        "W093",
                        format!(
                            "{} (status `{}`) cites `{}`, which is still open: \"{}\" — answer it (`set {} question.answer {} text=…`) or move {} back to draft",
                            label(src), status.unwrap_or_default(), c.citation.raw, q.text, q.item, q.id, label(src)
                        ),
                        &mut out,
                    );
                }
                if q.status == "answered" {
                    if let Some(m) = open_marker_in(&c.citation.sentence, &markers) {
                        push(
                            "W094",
                            format!(
                                "{} cites `{}` with \"{m}\", but the question is answered (\"{}\") — update the sentence to state the answer",
                                label(src), c.citation.raw, q.answer.as_deref().unwrap_or_default()
                            ),
                            &mut out,
                        );
                    }
                }
            }
        }
    }
    // Dedupe identical findings (same element citing the same thing on several lines).
    let mut keys: BTreeSet<(String, &'static str, String)> = BTreeSet::new();
    out.retain(|f| keys.insert((f.file.clone(), f.code, f.message.clone())));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_are_numbered_by_position_and_anchors_parsed() {
        let doc = "Intro.\n\n1. {#grant-keys} **Server keys.** The server\n   shall hold keys.\n2. Second item.\n\n```\n3. not an item\n```\n\n7. {#late} Third.\n\nAfter.\n";
        let (items, problems) = parse_items(doc);
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].anchor.as_deref(), Some("grant-keys"));
        assert_eq!(items[0].text, "1. **Server keys.** The server\n   shall hold keys.");
        assert_eq!(items[1].number, 2);
        assert_eq!(items[2].number, 3);
        assert_eq!(items[2].anchor.as_deref(), Some("late"));
    }

    #[test]
    fn duplicate_and_malformed_anchors_are_problems() {
        let (_, p) = parse_items("1. {#a} x\n2. {#a} y\n3. {#Bad} z\n");
        assert_eq!(p.len(), 2, "{p:?}");
    }

    #[test]
    fn strip_keeps_numbers() {
        assert_eq!(strip_anchors("1. {#a} x\n2. y\n"), "1. x\n2. y\n");
    }

    #[test]
    fn item_hash_ignores_position_and_anchor() {
        let (a, _) = parse_items("1. {#k} Same text.\n");
        let (b, _) = parse_items("1. Other.\n2. Same text.\n");
        assert_eq!(item_hash(&a[0]), item_hash(&b[1]));
    }

    #[test]
    fn citations_in_all_forms() {
        let cs = scan_citations("See REQ-AB-001#grant-keys, REQ-AB-001#5 and REQ-AB-001 Item 2; PI-X-001#Q31.\n```\nREQ-AB-001#9\n```\n");
        let parts: Vec<(&str, Part)> = cs.iter().map(|c| (c.raw.as_str(), c.part.clone())).collect();
        assert_eq!(
            parts,
            vec![
                ("REQ-AB-001#grant-keys", Part::Anchor("grant-keys".into())),
                ("REQ-AB-001#5", Part::Number(5)),
                ("REQ-AB-001 Item 2", Part::Number(2)),
                ("PI-X-001#Q31", Part::Question("Q31".into())),
            ]
        );
    }

    #[test]
    fn parse_ref_forms() {
        assert_eq!(parse_ref("REQ-X-001#a-b"), Some(("REQ-X-001".into(), Part::Anchor("a-b".into()))));
        assert_eq!(parse_ref("REQ-X-001 item 4"), Some(("REQ-X-001".into(), Part::Number(4))));
        assert_eq!(parse_ref("PI-X-001#Q2"), Some(("PI-X-001".into(), Part::Question("Q2".into()))));
        assert_eq!(parse_ref("REQ-X-001"), None);
    }

    #[test]
    fn markers_match_on_word_boundaries() {
        let m: Vec<String> = DEFAULT_OPEN_MARKERS.iter().map(|s| s.to_string()).collect();
        assert!(open_marker_in("Owner to confirm, PI-1#Q1", &m).is_some());
        assert!(open_marker_in("tbc per PI-1#Q1", &m).is_some());
        assert!(open_marker_in("the TBCX register", &m).is_none());
    }
}
