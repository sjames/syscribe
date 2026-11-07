//! Value references in prose and scenarios (GH #452, `REQ-TRS-ATTRREF-001..005`).
//!
//! **Syntax.** `{{Elem.attr}}` in an element body, where `Elem` is a stable id, a qualified
//! name or a unique basic name ([`resolve_element`]) and `attr` names one of its attributes, as a dotted path where values nest
//! (`{{PolicydBounds.snapshot.header}}`, `{{PolicydBounds.Budget.limitKb}}`). A `|unit` suffix is
//! accepted (the unit is always rendered).
//!
//! **Attributes** ([`attributes_of`]) are the named, single-valued slots of an element:
//! - an inline `features:` entry's `value:` (else its `default:`), and nested `features:` entries
//!   by dotted path (`snapshot.header`);
//! - an `Attribute` child element's `value:` (an `AttributeDef` child's `default:`), by its name;
//!   an `Attribute`/`AttributeDef` element's own `value`/`default`;
//! - a stereotype tagged value, `Stereotype.key` (defaults applied; `key` alone also resolves
//!   when exactly one application carries that key);
//! - the budget/timing scalar fields [`FIELD_ATTRS`] (`ram`, `flash`, `period`, `wcet`, …).
//!
//! An attribute whose value is not a single scalar (a list, a map, or a feature with neither
//! `value:` nor `default:`, such as a port) exists but has no value.
//!
//! **Where.** References and literals are read in the body outside code: inline code spans and
//! fenced blocks stay literal, except fenced `gherkin` / `feature` blocks (TestCase scenarios),
//! which are read (inline code spans inside them stay literal).
//!
//! **Feature parameters.** A reference whose element is a `FeatureDef`, or a `::` path / `FEAT-`
//! id that resolves to nothing, is a feature-parameter placeholder (`crate::placeholders`, GH #264)
//! and is left to it. An unresolved bare name starting with a lowercase letter (`{{ user.name }}`)
//! is template text and is ignored.
//!
//! **Findings.** `E331` an unresolvable reference; `W095` (opt-in, `[references] copied_values`) a
//! numeric literal equal to the current value of an attribute of an element the holder links to;
//! `W096` a literal equal to a *previous* value of such an attribute, from the git history of the
//! file that holds it (draft holders exempt).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use regex::Regex;
use serde_yaml::Value as Y;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::validator::{Finding, Severity};

/// Frontmatter scalar fields read as attributes of the element that holds them (YAML spelling).
pub const FIELD_ATTRS: &[&str] = &[
    "ram",
    "flash",
    "nvm",
    "stack",
    "ramCapacity",
    "flashCapacity",
    "nvmCapacity",
    "margin",
    "period",
    "deadline",
    "priority",
    "cpuMargin",
    "wcet",
    "failureRate",
    "diagnosticCoverage",
];

/// Literals whose absolute value is below this are never compared (`W095`/`W096`).
pub const MIN_LITERAL: f64 = 10.0;
/// Integers in this range read as years and are never compared.
pub const YEARS: std::ops::RangeInclusive<i64> = 1900..=2100;

// ── attributes ───────────────────────────────────────────────────────────────

/// Where an attribute's value is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrKind {
    Feature,
    AttributeElement,
    Tagged,
    Field,
}

/// One named attribute of an element.
#[derive(Debug, Clone, PartialEq)]
pub struct Attr {
    /// The path after `Elem.` (`snapshotFixedBytes`, `snapshot.header`, `Budget.limitKb`).
    pub path: String,
    /// The single scalar value as written; `None` when the attribute holds no scalar.
    pub value: Option<String>,
    /// Its `unit:`, when declared.
    pub unit: Option<String>,
    pub kind: AttrKind,
    /// The file that holds the value.
    pub file: String,
    /// The attribute's path among the [`own_attributes`] of that file's element (differs from
    /// `path` for an `Attribute` child, whose own path is `value`/`default`).
    pub own_path: String,
}

impl Attr {
    /// The value with its unit, as rendered in place of a reference (`47244 B`).
    pub fn rendered(&self) -> Option<String> {
        let v = self.value.as_ref()?;
        Some(match self.unit.as_deref().map(|u| u.strip_prefix("SI::").unwrap_or(u)).filter(|u| !u.is_empty()) {
            Some(u) => format!("{v} {u}"),
            None => v.clone(),
        })
    }

    /// The numeric magnitude of the value when it is a plain number (optionally followed by a
    /// unit word): `47244`, `"47 244"`, `"12 KiB"`. Expressions and literals of other kinds are
    /// `None`.
    pub fn number(&self) -> Option<f64> {
        numeric_value(self.value.as_deref()?)
    }
}

fn scalar(v: &Y) -> Option<String> {
    match v {
        Y::String(s) => Some(s.trim().to_string()),
        Y::Number(n) => Some(n.to_string()),
        Y::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn feature_attrs(list: &[Y], prefix: &str, file: &str, out: &mut Vec<Attr>) {
    for f in list {
        let Some(name) = f.get("name").and_then(Y::as_str) else { continue };
        let path = if prefix.is_empty() { name.to_string() } else { format!("{prefix}.{name}") };
        let value = f.get("value").and_then(scalar).or_else(|| f.get("default").and_then(scalar));
        let unit = f.get("unit").and_then(scalar);
        out.push(Attr { path: path.clone(), value, unit, kind: AttrKind::Feature, file: file.to_string(), own_path: path.clone() });
        if let Some(Y::Sequence(sub)) = f.get("features") {
            feature_attrs(sub, &path, file, out);
        }
    }
}

/// The attributes held in `e`'s own file: features, tagged values, the [`FIELD_ATTRS`] and, for an
/// `Attribute`/`AttributeDef`, its own `value`/`default`.
pub fn own_attributes(elements: &[RawElement], resolver: &Resolver, e: &RawElement) -> Vec<Attr> {
    let fm = &e.frontmatter;
    let file = e.file_path.as_str();
    let mut out = Vec::new();
    if let Some(fs) = fm.features.as_ref() {
        feature_attrs(fs, "", file, &mut out);
    }
    if matches!(fm.element_type, Some(ElementType::Attribute | ElementType::AttributeDef)) {
        for (k, v) in [("value", &fm.value), ("default", &fm.default)] {
            if let Some(v) = v {
                out.push(Attr { path: k.into(), value: scalar(v), unit: fm.unit.clone(), kind: AttrKind::Field, file: file.into(), own_path: k.into() });
            }
        }
    }
    let fields: [(&str, Option<String>); 15] = [
        ("ram", fm.ram.as_ref().and_then(scalar)),
        ("flash", fm.flash.as_ref().and_then(scalar)),
        ("nvm", fm.nvm.as_ref().and_then(scalar)),
        ("stack", fm.stack.as_ref().and_then(scalar)),
        ("ramCapacity", fm.ram_capacity.as_ref().and_then(scalar)),
        ("flashCapacity", fm.flash_capacity.as_ref().and_then(scalar)),
        ("nvmCapacity", fm.nvm_capacity.as_ref().and_then(scalar)),
        ("margin", fm.margin.as_ref().and_then(scalar)),
        ("period", fm.period.as_ref().and_then(scalar)),
        ("deadline", fm.deadline.as_ref().and_then(scalar)),
        ("priority", fm.priority.as_ref().and_then(scalar)),
        ("cpuMargin", fm.cpu_margin.as_ref().and_then(scalar)),
        ("wcet", fm.wcet.clone()),
        ("failureRate", fm.failure_rate.map(|x| x.to_string())),
        ("diagnosticCoverage", fm.diagnostic_coverage.map(|x| x.to_string())),
    ];
    for (k, v) in fields {
        if let Some(v) = v {
            out.push(Attr { path: k.into(), value: Some(v), unit: None, kind: AttrKind::Field, file: file.into(), own_path: k.into() });
        }
    }
    if fm.metadata.as_ref().is_some_and(|m| !m.is_empty()) {
        for app in crate::tagged_values::effective_applications(elements, resolver, e) {
            let label = app.def.rsplit("::").next().unwrap_or(&app.def).to_string();
            let specs = crate::tagged_values::find_metadata_def(elements, resolver, &app.def)
                .map(|d| crate::tagged_values::merged_specs(&crate::tagged_values::lineage(elements, resolver, d)))
                .unwrap_or_default();
            for (k, v) in &app.values {
                let path = format!("{label}.{k}");
                let unit = specs.iter().find(|s| &s.name == k).and_then(|s| s.unit.clone());
                out.push(Attr { path: path.clone(), value: scalar(v), unit, kind: AttrKind::Tagged, file: file.into(), own_path: path });
            }
        }
    }
    out
}

/// `Attribute` / `AttributeDef` elements by the qualified name of the element that owns them.
pub struct ChildIndex<'a> {
    by_parent: HashMap<&'a str, Vec<&'a RawElement>>,
}

impl<'a> ChildIndex<'a> {
    pub fn new(elements: &'a [RawElement]) -> Self {
        let mut by_parent: HashMap<&str, Vec<&RawElement>> = HashMap::new();
        for e in elements {
            if !matches!(e.frontmatter.element_type, Some(ElementType::Attribute | ElementType::AttributeDef)) {
                continue;
            }
            if let Some((parent, _)) = e.qualified_name.rsplit_once("::") {
                by_parent.entry(parent).or_default().push(e);
            }
        }
        ChildIndex { by_parent }
    }
}

/// Every attribute of `e`: [`own_attributes`] plus its `Attribute` / `AttributeDef` children.
pub fn attributes_of(elements: &[RawElement], resolver: &Resolver, children: &ChildIndex<'_>, e: &RawElement) -> Vec<Attr> {
    let mut out = own_attributes(elements, resolver, e);
    for c in children.by_parent.get(e.qualified_name.as_str()).into_iter().flatten() {
        let seg = c.qualified_name.rsplit("::").next().unwrap_or(&c.qualified_name).to_string();
        if out.iter().any(|a| a.path == seg) {
            continue; // an inline feature of the same name wins
        }
        let fm = &c.frontmatter;
        let (own_path, v) = match (&fm.value, &fm.default) {
            (Some(v), _) => ("value", Some(v)),
            (None, Some(d)) => ("default", Some(d)),
            _ => ("value", None),
        };
        out.push(Attr {
            path: seg,
            value: v.and_then(scalar),
            unit: fm.unit.clone(),
            kind: AttrKind::AttributeElement,
            file: c.file_path.clone(),
            own_path: own_path.into(),
        });
    }
    out
}

/// Look `path` up among `attrs`: an exact path, else the one tagged value whose key is `path`.
pub fn lookup<'a>(attrs: &'a [Attr], path: &str) -> Option<&'a Attr> {
    if let Some(a) = attrs.iter().find(|a| a.path == path) {
        return Some(a);
    }
    let mut tagged = attrs.iter().filter(|a| a.kind == AttrKind::Tagged && a.path.split_once('.').is_some_and(|(_, k)| k == path));
    match (tagged.next(), tagged.next()) {
        (Some(a), None) => Some(a),
        _ => None,
    }
}

/// The attribute paths that carry a value, for messages.
pub fn available(attrs: &[Attr]) -> Vec<String> {
    attrs.iter().filter(|a| a.value.is_some()).map(|a| a.path.clone()).collect()
}

// ── scanning ─────────────────────────────────────────────────────────────────

/// Byte ranges of `text` that are code: fenced blocks (except `gherkin`/`feature` ones, whose fence
/// lines alone are excluded) and inline code spans.
pub fn code_ranges(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut pos = 0;
    // (start, fence char, fence length, is a scenario block)
    let mut fence: Option<(usize, char, usize, bool)> = None;
    for line in text.split_inclusive('\n') {
        let end = pos + line.len();
        let indent = line.len() - line.trim_start().len();
        let t = line.trim_start();
        let run = |c: char| t.chars().take_while(|x| *x == c).count();
        let fence_here = if indent < 4 && t.starts_with("```") {
            Some(('`', run('`')))
        } else if indent < 4 && t.starts_with("~~~") {
            Some(('~', run('~')))
        } else {
            None
        };
        match (fence, fence_here) {
            (Some((s, c, n, scenario)), Some((fc, fnn))) if fc == c && fnn >= n && t[fnn..].trim().is_empty() => {
                if scenario {
                    out.push((pos, end));
                } else {
                    out.push((s, end));
                }
                fence = None;
            }
            (None, Some((fc, fnn))) => {
                let info = t[fnn..].trim().split(|c: char| c.is_whitespace() || c == '{').next().unwrap_or("").to_ascii_lowercase();
                let scenario = matches!(info.as_str(), "gherkin" | "feature" | "cucumber");
                if scenario {
                    out.push((pos, end));
                }
                fence = Some((pos, fc, fnn, scenario));
            }
            (Some((_, _, _, false)), _) => {}
            _ => inline_spans(line, pos, &mut out),
        }
        pos = end;
    }
    if let Some((s, _, _, false)) = fence {
        out.push((s, text.len())); // an unterminated fence runs to the end
    }
    out
}

/// Inline code spans of one line: a run of N backticks closes at the next run of N.
fn inline_spans(line: &str, pos: usize, out: &mut Vec<(usize, usize)>) {
    let b = line.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'`' {
            i += 1;
            continue;
        }
        let start = i;
        while i < b.len() && b[i] == b'`' {
            i += 1;
        }
        let n = i - start;
        let mut j = i;
        let mut close = None;
        while j < b.len() {
            if b[j] == b'`' {
                let cs = j;
                while j < b.len() && b[j] == b'`' {
                    j += 1;
                }
                if j - cs == n {
                    close = Some(j);
                    break;
                }
            } else {
                j += 1;
            }
        }
        match close {
            Some(c) => {
                out.push((pos + start, pos + c));
                i = c;
            }
            None => break,
        }
    }
}

fn in_ranges(ranges: &[(usize, usize)], at: usize) -> bool {
    ranges.iter().any(|&(a, b)| at >= a && at < b)
}

fn ref_re() -> &'static Regex {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\{\{\s*([A-Za-z_][A-Za-z0-9_\-:]*)\.([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)\s*(?:\|\s*unit)?\s*\}\}")
            .expect("value reference regex")
    })
}

/// One `{{Elem.attr}}` occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueRef {
    /// As written, braces included.
    pub raw: String,
    pub elem: String,
    pub path: String,
    /// 0-based line within the scanned text.
    pub line: usize,
    /// Byte range within the scanned text.
    pub start: usize,
    pub end: usize,
}

impl ValueRef {
    /// `Elem.attr`.
    pub fn key(&self) -> String {
        format!("{}.{}", self.elem, self.path)
    }
}

fn line_of(text: &str, at: usize) -> usize {
    text[..at].bytes().filter(|b| *b == b'\n').count()
}

/// Every `{{X.y}}`-shaped span of `text` outside code (see the module docs), in order. Whether
/// it is a value reference is decided by [`resolve`].
pub fn scan(text: &str) -> Vec<ValueRef> {
    if !text.contains("{{") {
        return Vec::new();
    }
    let code = code_ranges(text);
    ref_re()
        .captures_iter(text)
        .filter_map(|c| {
            let m = c.get(0)?;
            if in_ranges(&code, m.start()) {
                return None;
            }
            Some(ValueRef {
                raw: m.as_str().to_string(),
                elem: c[1].to_string(),
                path: c[2].to_string(),
                line: line_of(text, m.start()),
                start: m.start(),
                end: m.end(),
            })
        })
        .collect()
}

/// What a `{{X.y}}` span is.
#[derive(Debug, Clone)]
pub enum Resolution<'a> {
    /// Not a value reference: a feature-parameter placeholder or template text.
    NotOurs,
    /// A value reference to `attr` of `elem`.
    Value { elem: &'a RawElement, attr: Attr },
    /// A value reference that does not resolve (`E331`): why.
    Error(String),
}

/// Whether the element part of a `{{X.y}}` span resolves to an element other than a `FeatureDef`
/// (so it is a value reference, not a feature-parameter placeholder).
pub fn is_value_target(elements: &[RawElement], resolver: &Resolver, elem: &str) -> bool {
    match resolve_element(elements, resolver, elem) {
        ElemMatch::Found(t) => t.frontmatter.element_type != Some(ElementType::FeatureDef),
        ElemMatch::Ambiguous(c) => c.iter().any(|t| t.frontmatter.element_type != Some(ElementType::FeatureDef)),
        ElemMatch::None => false,
    }
}

/// How the element part of a value reference resolves.
#[derive(Debug, Clone)]
pub enum ElemMatch<'a> {
    Found(&'a RawElement),
    /// Several elements share the basic name.
    Ambiguous(Vec<&'a RawElement>),
    None,
}

/// Resolve the element part of a value reference: by stable id, then exact qualified name, then a
/// unique last qualified-name segment (basic name) across the model — so `{{Bounds.size}}` finds
/// `Arch::Bounds`. Never the display-name or fuzzy fallback.
pub fn resolve_element<'a>(elements: &'a [RawElement], resolver: &Resolver, elem: &str) -> ElemMatch<'a> {
    if let Some(t) = crate::req_items::resolve_elem(elements, resolver, elem) {
        return ElemMatch::Found(t);
    }
    if elem.contains("::") {
        return ElemMatch::None;
    }
    let mut hits: Vec<&RawElement> = Vec::new();
    for e in elements {
        if e.qualified_name.rsplit("::").next() == Some(elem) && !hits.iter().any(|h| h.qualified_name == e.qualified_name) {
            hits.push(e);
        }
    }
    match hits.len() {
        0 => ElemMatch::None,
        1 => ElemMatch::Found(hits[0]),
        _ => ElemMatch::Ambiguous(hits),
    }
}

/// The message for an element part that names several elements.
pub fn ambiguous_message(elem: &str, candidates: &[&RawElement]) -> String {
    let mut q: Vec<&str> = candidates.iter().map(|c| c.qualified_name.as_str()).collect();
    q.sort_unstable();
    format!(
        "element `{elem}` is ambiguous — {} elements have that name: {}; write the full qualified name or the stable id",
        q.len(),
        q.iter().map(|x| format!("`{x}`")).collect::<Vec<_>>().join(", ")
    )
}

/// Label used for an element in messages and suggested references: its id, else its qname.
pub fn label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// Resolve the element part of a reference; `None` when the span is not a value reference.
fn resolve_target<'a>(elements: &'a [RawElement], resolver: &Resolver, elem: &str) -> Option<Result<&'a RawElement, String>> {
    match resolve_element(elements, resolver, elem) {
        ElemMatch::Found(t) if t.frontmatter.element_type == Some(ElementType::FeatureDef) => None,
        ElemMatch::Found(t) => Some(Ok(t)),
        ElemMatch::Ambiguous(c) if c.iter().all(|t| t.frontmatter.element_type == Some(ElementType::FeatureDef)) => None,
        ElemMatch::Ambiguous(c) => Some(Err(ambiguous_message(elem, &c))),
        ElemMatch::None if elem.starts_with("FEAT-") || elem.contains("::") => None,
        ElemMatch::None if elem.chars().next().is_some_and(|c| c.is_ascii_lowercase()) => None,
        ElemMatch::None => Some(Err(format!(
            "element `{elem}` does not resolve to any element (by stable id, qualified name or basic name) — check the name with `check-ref`, or write the full qualified name"
        ))),
    }
}

/// Resolve one reference.
pub fn resolve<'a>(elements: &'a [RawElement], resolver: &Resolver, children: &ChildIndex<'_>, r: &ValueRef) -> Resolution<'a> {
    let t = match resolve_target(elements, resolver, &r.elem) {
        None => return Resolution::NotOurs,
        Some(Err(m)) => return Resolution::Error(m),
        Some(Ok(t)) => t,
    };
    let attrs = attributes_of(elements, resolver, children, t);
    match lookup(&attrs, &r.path) {
        Some(a) if a.value.is_some() => Resolution::Value { elem: t, attr: a.clone() },
        Some(a) => Resolution::Error(format!(
            "`{}.{}` has no scalar value (a list, a map, or a feature with no `value:`/`default:`) — reference an attribute with a single value: {}",
            r.elem,
            a.path,
            list_or_none(&available(&attrs))
        )),
        None => Resolution::Error(format!(
            "{} has no attribute `{}` — attributes with a value: {}",
            r.elem,
            r.path,
            list_or_none(&available(&attrs))
        )),
    }
}

fn list_or_none(v: &[String]) -> String {
    if v.is_empty() {
        "none (declare one under `features:` with a `value:`)".to_string()
    } else {
        v.join(", ")
    }
}

/// `text` with every resolvable value reference replaced by its value and unit; anything else is
/// kept as written.
pub fn render(elements: &[RawElement], resolver: &Resolver, text: &str) -> String {
    let refs = scan(text);
    if refs.is_empty() {
        return text.to_string();
    }
    let children = ChildIndex::new(elements);
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for r in refs {
        if let Resolution::Value { attr, .. } = resolve(elements, resolver, &children, &r) {
            if let Some(v) = attr.rendered() {
                out.push_str(&text[last..r.start]);
                out.push_str(&v);
                last = r.end;
            }
        }
    }
    out.push_str(&text[last..]);
    out
}

// ── literals ─────────────────────────────────────────────────────────────────

/// One numeric literal of prose.
#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    /// As written (`46 852`).
    pub text: String,
    pub value: f64,
    /// 0-based line within the scanned text.
    pub line: usize,
}

fn lit_re() -> &'static Regex {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"[0-9]{1,3}(?:[ ,_'\u{00A0}\u{202F}][0-9]{3})+(?:\.[0-9]+)?|[0-9]+(?:\.[0-9]+)?").expect("literal regex")
    })
}

fn parse_grouped(s: &str) -> Option<f64> {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    digits.parse().ok()
}

/// The magnitude of a value written as a plain number, optionally followed by a unit word.
pub fn numeric_value(text: &str) -> Option<f64> {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"^\s*([+-]?)([0-9]{1,3}(?:[ ,_'][0-9]{3})+(?:\.[0-9]+)?|[0-9]+(?:\.[0-9]+)?)(?:\s*[A-Za-zµ°%][A-Za-z0-9µ°%/^·*]*)?\s*$")
            .expect("numeric value regex")
    });
    let c = re.captures(text)?;
    let v = parse_grouped(&c[2])?;
    Some(if &c[1] == "-" { -v } else { v })
}

/// Whether a literal of value `v` is too common to compare (below [`MIN_LITERAL`], or a year).
pub fn is_noise(v: f64) -> bool {
    v.abs() < MIN_LITERAL || (v.fract() == 0.0 && YEARS.contains(&(v as i64)))
}

fn word_after(rest: &str) -> &str {
    let n = rest.find(|c: char| !(c.is_alphanumeric() || c == 'µ')).unwrap_or(rest.len());
    &rest[..n]
}

/// Every whole-token numeric literal of `text` outside code and outside `{{…}}` spans, with
/// [`is_noise`] values removed. A literal is not whole when it touches a letter, digit, `_`, `.`,
/// `#`, `/`, `\`, `:` or an id-style `-` (`TC-47244`, `v1.2`, `2026-11-07`, `12:30`, `0x1F`); a letter
/// run right after it is allowed only when it is a unit (`10ms`, `4KiB`).
pub fn scan_literals(text: &str) -> Vec<Literal> {
    static BRACES: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let braces = BRACES.get_or_init(|| Regex::new(r"\{\{[^}]*\}\}").expect("braces regex"));
    let mut skip = code_ranges(text);
    skip.extend(braces.find_iter(text).map(|m| (m.start(), m.end())));
    let mut out = Vec::new();
    for m in lit_re().find_iter(text) {
        if in_ranges(&skip, m.start()) {
            continue;
        }
        let before = text[..m.start()].chars().next_back();
        let before2 = text[..m.start()].chars().rev().nth(1);
        let bad_before = match before {
            None => false,
            Some(c) if c.is_alphanumeric() || matches!(c, '_' | '.' | '#' | '/' | '\\' | ':' | '@') => true,
            Some('-') => before2.is_some_and(|c| c.is_alphanumeric()),
            _ => false,
        };
        if bad_before {
            continue;
        }
        let rest = &text[m.end()..];
        let mut it = rest.chars();
        let after = it.next();
        let after2 = it.next();
        let bad_after = match after {
            None => false,
            Some(c) if c.is_ascii_digit() || c == '_' => true,
            Some('.' | '-' | ':' | '/' | ',') => after2.is_some_and(|c| c.is_alphanumeric()),
            Some(c) if c.is_alphabetic() || c == 'µ' => crate::expr::unit_scale(word_after(rest)).is_none(),
            _ => false,
        };
        if bad_after {
            continue;
        }
        let Some(value) = parse_grouped(m.as_str()) else { continue };
        if is_noise(value) {
            continue;
        }
        out.push(Literal { text: m.as_str().to_string(), value, line: line_of(text, m.start()) });
    }
    out
}

/// Whether two magnitudes are the same number.
pub fn same(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}

// ── links ────────────────────────────────────────────────────────────────────

/// The elements `e` links to: every outgoing reference of [`crate::projection::outbound_refs`]
/// (satisfies, verifies, verifiesElement, derivedFrom, allocatedTo/From, typedBy, supertype,
/// evidence `ref:`, user-defined `links:`, …) plus `achieves`, `refines` and the elements of its
/// body's value references. Never `e` itself or a `FeatureDef`.
pub fn linked_targets<'a>(elements: &'a [RawElement], resolver: &Resolver, e: &RawElement) -> Vec<&'a RawElement> {
    let mut refs: Vec<String> = crate::projection::outbound_refs(e).into_iter().map(|(_, s)| s).collect();
    refs.extend(e.frontmatter.achieves.iter().flatten().cloned());
    refs.extend(e.frontmatter.refines.iter().flatten().cloned());
    let mut found: Vec<&RawElement> = refs.iter().filter_map(|r| resolver.resolve_ref(elements, r)).collect();
    for r in scan(&e.doc) {
        if let ElemMatch::Found(t) = resolve_element(elements, resolver, &r.elem) {
            found.push(t);
        }
    }
    let mut out: Vec<&RawElement> = Vec::new();
    for t in found {
        if std::ptr::eq(t, e) || t.frontmatter.element_type == Some(ElementType::FeatureDef) || out.iter().any(|x| std::ptr::eq(*x, t)) {
            continue;
        }
        out.push(t);
    }
    out
}

// ── history ──────────────────────────────────────────────────────────────────

/// Previous values of attributes, read from git: file → own path → previous values as written
/// (newest first, deduplicated). Cached per file.
pub struct History {
    root: Option<std::path::PathBuf>,
    /// The repository top, discovered on first use (`Some(None)`: no git / no repository).
    top: Option<Option<std::path::PathBuf>>,
    limit: usize,
    cache: HashMap<String, HashMap<String, Vec<(String, String)>>>,
}

impl History {
    /// Without a model root, or when git or the repository is missing, there are no previous
    /// values. The repository is looked up on first use.
    pub fn new(model_root: Option<&Path>, limit: usize) -> Self {
        History { root: model_root.map(Path::to_path_buf), top: None, limit, cache: HashMap::new() }
    }

    fn top(&mut self) -> Option<std::path::PathBuf> {
        if self.top.is_none() {
            let found = self.root.as_deref().and_then(crate::commit_evidence::Repo::discover).map(|r| r.top().to_path_buf());
            self.top = Some(found);
        }
        self.top.clone().flatten()
    }

    /// `(previous value, commit)` pairs of `attr` whose number differs from its current value.
    pub fn previous(&mut self, elements: &[RawElement], resolver: &Resolver, holder_of: &dyn Fn(&str) -> Option<String>, attr: &Attr) -> Vec<(String, String)> {
        let Some(top) = self.top() else { return Vec::new() };
        let limit = self.limit;
        let map = self.cache.entry(attr.file.clone()).or_insert_with(|| {
            let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
            let file = std::fs::canonicalize(&attr.file).unwrap_or_else(|_| std::path::PathBuf::from(&attr.file));
            let Ok(rel) = file.strip_prefix(&top) else { return map };
            let rel = rel.to_string_lossy().replace('\\', "/");
            let qname = holder_of(&attr.file).unwrap_or_default();
            for (sha, _date, path, content) in crate::amendments::file_versions(&top, &rel, limit) {
                let Some(content) = content else { continue };
                let (Some(yaml), body) = crate::frontmatter::split_frontmatter(&content) else { continue };
                let Ok(fm) = crate::frontmatter::parse_frontmatter(yaml) else { continue };
                let old = RawElement {
                    qualified_name: qname.clone(),
                    file_path: path,
                    frontmatter: fm,
                    doc: body.to_string(),
                    parse_issue: None,
                    derived: Default::default(),
                    derive_findings: Vec::new(),
                    locale_docs: Default::default(),
                    about_notes: Vec::new(),
                };
                for a in own_attributes(elements, resolver, &old) {
                    if let Some(v) = a.value {
                        let list = map.entry(a.own_path).or_default();
                        if !list.iter().any(|(x, _)| x == &v) {
                            list.push((v, sha.clone()));
                        }
                    }
                }
            }
            map
        });
        let current = attr.number();
        map.get(&attr.own_path)
            .into_iter()
            .flatten()
            .filter(|(v, _)| match (numeric_value(v), current) {
                (Some(p), Some(c)) => !same(p, c),
                (Some(_), None) => true,
                _ => false,
            })
            .cloned()
            .collect()
    }
}

// ── copies ───────────────────────────────────────────────────────────────────

/// A literal in a holder's text that equals an attribute value of a linked element.
#[derive(Debug, Clone)]
pub struct Copy<'a> {
    pub literal: Literal,
    pub target: &'a RawElement,
    pub attr: Attr,
    /// For a stale copy: the previous value matched and the commit that held it.
    pub previous: Option<(String, String)>,
}

impl Copy<'_> {
    /// `{{<target>.<path>}}`, the reference to write instead.
    pub fn suggestion(&self) -> String {
        format!("{{{{{}.{}}}}}", label(self.target), self.attr.path)
    }
}

/// Literal copies of current values (`stale = false`) or of previous values (`stale = true`, with
/// `history`) of the attributes of `targets`, in `text`. A literal that equals any current value is
/// never reported as stale.
pub fn copies_in<'a>(
    elements: &'a [RawElement],
    resolver: &Resolver,
    children: &ChildIndex<'_>,
    text: &str,
    targets: &[&'a RawElement],
    cache: &mut HashMap<String, Vec<Attr>>,
    mut history: Option<&mut History>,
) -> (Vec<Copy<'a>>, Vec<Copy<'a>>) {
    let mut current = Vec::new();
    let mut stale = Vec::new();
    if targets.is_empty() {
        return (current, stale);
    }
    let lits = scan_literals(text);
    if lits.is_empty() {
        return (current, stale);
    }
    let attrs: Vec<(&RawElement, Vec<Attr>)> = targets
        .iter()
        .map(|t| {
            let list = cache
                .entry(t.qualified_name.clone())
                .or_insert_with(|| attributes_of(elements, resolver, children, t).into_iter().filter(|a| a.number().is_some()).collect());
            (*t, list.clone())
        })
        .filter(|(_, l)| !l.is_empty())
        .collect();
    if attrs.is_empty() {
        return (current, stale);
    }
    let holder_of = |file: &str| elements.iter().find(|e| e.file_path == file).map(|e| e.qualified_name.clone());
    for lit in &lits {
        let mut matched = false;
        for (t, list) in &attrs {
            for a in list {
                if a.number().is_some_and(|n| same(n, lit.value)) {
                    matched = true;
                    current.push(Copy { literal: lit.clone(), target: t, attr: a.clone(), previous: None });
                }
            }
        }
        if matched {
            continue;
        }
        let Some(h) = history.as_deref_mut() else { continue };
        for (t, list) in &attrs {
            for a in list {
                for (pv, sha) in h.previous(elements, resolver, &holder_of, a) {
                    if numeric_value(&pv).is_some_and(|p| same(p, lit.value)) {
                        stale.push(Copy { literal: lit.clone(), target: t, attr: a.clone(), previous: Some((pv, sha)) });
                        break;
                    }
                }
            }
        }
    }
    (current, stale)
}

// ── findings ─────────────────────────────────────────────────────────────────

fn finding(code: &'static str, file: &str, message: String, severity: Severity) -> Finding {
    Finding { code, file: file.to_string(), message, severity }
}

/// `E331`, `W095` (when `copied_values`) and `W096` (with git) over the model.
pub fn findings(
    elements: &[RawElement],
    resolver: &Resolver,
    copied_values: bool,
    model_root: Option<&Path>,
    history_limit: usize,
) -> Vec<Finding> {
    let mut out = Vec::new();
    let children = ChildIndex::new(elements);
    let mut history = History::new(model_root, history_limit);
    let mut cache: HashMap<String, Vec<Attr>> = HashMap::new();
    for e in elements {
        if e.doc.is_empty() {
            continue;
        }
        let who = label(e);
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for r in scan(&e.doc) {
            if let Resolution::Error(why) = resolve(elements, resolver, &children, &r) {
                if seen.insert(r.raw.clone()) {
                    out.push(finding(
                        "E331",
                        &e.file_path,
                        format!("{who}: value reference `{}` (line {}) does not resolve: {why}", r.raw, r.line + 1),
                        Severity::Error,
                    ));
                }
            }
        }
        let want_stale = model_root.is_some() && e.frontmatter.status.as_deref() != Some("draft");
        if !copied_values && !want_stale {
            continue;
        }
        let targets = linked_targets(elements, resolver, e);
        if targets.is_empty() {
            continue;
        }
        let (current, stale) = copies_in(elements, resolver, &children, &e.doc, &targets, &mut cache, want_stale.then_some(&mut history));
        if copied_values {
            let mut grouped: BTreeMap<(String, String), (String, Vec<usize>, String)> = BTreeMap::new();
            for c in &current {
                let key = (format!("{}.{}", label(c.target), c.attr.path), c.literal.text.clone());
                let entry = grouped.entry(key).or_insert_with(|| (c.attr.rendered().unwrap_or_default(), Vec::new(), c.suggestion()));
                entry.1.push(c.literal.line + 1);
            }
            for ((attr, lit), (val, lines, sugg)) in grouped {
                out.push(finding(
                    "W095",
                    &e.file_path,
                    format!(
                        "{who} states `{lit}` (line {}), the current value of {attr} ({val}), as a literal — write `{sugg}` so the text follows the design value",
                        lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(", ")
                    ),
                    Severity::Warning,
                ));
            }
        }
        let mut grouped: BTreeMap<(String, String), (String, String, String, Vec<usize>, String)> = BTreeMap::new();
        for c in &stale {
            let Some((pv, sha)) = &c.previous else { continue };
            let key = (format!("{}.{}", label(c.target), c.attr.path), c.literal.text.clone());
            let entry = grouped
                .entry(key)
                .or_insert_with(|| (pv.clone(), sha.clone(), c.attr.rendered().unwrap_or_default(), Vec::new(), c.suggestion()));
            entry.3.push(c.literal.line + 1);
        }
        for ((attr, lit), (pv, sha, now, lines, sugg)) in grouped {
            out.push(finding(
                "W096",
                &e.file_path,
                format!(
                    "{who} states `{lit}` (line {}), a previous value of {attr} ({pv} in commit {}); it is now {now} — a stale copy: update it, or write `{sugg}` so it follows the design value",
                    lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(", "),
                    &sha[..sha.len().min(10)]
                ),
                Severity::Warning,
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_skips_code_but_reads_gherkin() {
        let t = "A {{X.a}} `{{X.b}}`\n```text\n{{X.c}}\n```\n```gherkin\nGiven {{X.d}} and `{{X.e}}`\n```\n";
        let paths: Vec<String> = scan(t).into_iter().map(|r| r.path).collect();
        assert_eq!(paths, ["a", "d"]);
    }

    #[test]
    fn dotted_paths_and_unit_suffix() {
        let r = &scan("{{ Pkg::Elem.snapshot.header | unit }}")[0];
        assert_eq!((r.elem.as_str(), r.path.as_str()), ("Pkg::Elem", "snapshot.header"));
    }

    #[test]
    fn literals_are_whole_tokens_and_noise_is_dropped() {
        let t = "Size 47 244 bytes, 47,244 B, 1500ms, 8 retries, in 2026, TC-47244, v12.5.1, 0x1F00, 2026-11-07, 12:30, 3.75, 12.5 %, -40 C";
        let v: Vec<f64> = scan_literals(t).into_iter().map(|l| l.value).collect();
        assert_eq!(v, [47244.0, 47244.0, 1500.0, 12.5, 40.0]);
    }

    #[test]
    fn numeric_values() {
        assert_eq!(numeric_value("47 244"), Some(47244.0));
        assert_eq!(numeric_value("12 KiB"), Some(12.0));
        assert_eq!(numeric_value("0.6"), Some(0.6));
        assert_eq!(numeric_value("throttlePercent + 0.01"), None);
        assert_eq!(numeric_value("FlightMode::hover"), None);
    }
}
