//! Table views for documents (GH #531, `REQ-TRS-VIEWTAB-001..003`).
//!
//! A `View` selects elements with `expose:`; with `columns:` it also declares the table a document
//! prints of them — which fields, under which headings, in which order — so a generator needs no
//! code per table. `rows:` takes one row per entry of a list inside each element (questions,
//! parameters, constraints) instead of one per element; `where:` keeps the rows that match; `sort:`
//! orders them; `nest:` indents children under their parent; `marks:` flags rows with a note.
//! `view render --format latex|markdown|csv|html` writes the result.

use std::collections::{BTreeSet, HashMap};

use serde_json::Value as J;

use crate::doc_render::Escape;
use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

/// The frontmatter keys of a table declaration (all in the `extra` catch-all).
pub const KEYS: &[&str] = &["columns", "rows", "where", "sort", "caption", "label", "nest", "marks"];

/// Fields every element answers to, beside its own frontmatter keys.
pub const ELEMENT_FIELDS: &[&str] = &[
    "ref", "id", "qname", "type", "name", "tag", "tags", "status", "file", "body", "summary", "verifiedBy", "satisfiedBy", "derivedChildren", "openQuestions",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    /// The field shown; empty for a `tagPrefix:` column.
    pub field: String,
    pub heading: String,
    /// Shown for a row with no value.
    pub empty: String,
    pub align: Align,
    /// LaTeX column width (`6cm`): the column wraps (`p{6cm}`).
    pub width: Option<String>,
    /// The rest of the element's first tag with this prefix (`phase-` shows `2` for `phase-2`).
    pub tag_prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mark {
    /// A condition in the `where:` language.
    pub when: String,
    pub symbol: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Table {
    pub columns: Vec<Column>,
    /// The list inside each element that gives the rows; `None` = one row per element.
    pub rows: Option<String>,
    pub where_: Option<String>,
    /// (field, descending)
    pub sort: Vec<(String, bool)>,
    pub caption: Option<String>,
    pub label: Option<String>,
    /// The field naming a row's parent element: children follow their parent, indented.
    pub nest: Option<String>,
    pub marks: Vec<Mark>,
}

fn heading_of(field: &str) -> String {
    // `answeredBy` -> `Answered by`, `element.name` -> `Name`
    let leaf = field.rsplit('.').next().unwrap_or(field);
    let mut out = String::new();
    for (i, c) in leaf.chars().enumerate() {
        if i == 0 {
            out.extend(c.to_uppercase());
        } else if c.is_uppercase() {
            out.push(' ');
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Whether a view declares a table (any of [`KEYS`] present).
pub fn present(extra: &HashMap<String, serde_yaml::Value>) -> bool {
    KEYS.iter().any(|k| extra.contains_key(*k))
}

/// The table a view declares; `Ok(None)` when it declares no `columns:`. `Err` lists what is malformed.
pub fn parse(extra: &HashMap<String, serde_yaml::Value>) -> Result<Option<Table>, Vec<String>> {
    use serde_yaml::Value as Y;
    if !present(extra) {
        return Ok(None);
    }
    let mut errs: Vec<String> = Vec::new();
    let mut t = Table::default();
    let text = |k: &str, errs: &mut Vec<String>| -> Option<String> {
        match extra.get(k) {
            None => None,
            Some(Y::String(s)) if !s.trim().is_empty() => Some(s.trim().to_string()),
            Some(v) => {
                errs.push(format!("`{k}:` must be a non-empty string, got {}", yaml_inline(v)));
                None
            }
        }
    };
    match extra.get("columns") {
        None => errs.push(format!(
            "a table view needs `columns:` (it declares {})",
            KEYS.iter().filter(|k| extra.contains_key(**k)).map(|k| format!("`{k}:`")).collect::<Vec<_>>().join(", ")
        )),
        Some(Y::Sequence(list)) if !list.is_empty() => {
            for (i, c) in list.iter().enumerate() {
                match c {
                    Y::String(f) if !f.trim().is_empty() => {
                        t.columns.push(Column { field: f.trim().to_string(), heading: heading_of(f.trim()), empty: String::new(), align: Align::Left, width: None, tag_prefix: None })
                    }
                    Y::Mapping(m) => {
                        let get = |k: &str| m.get(Y::String(k.into()));
                        let s = |k: &str| get(k).and_then(|v| v.as_str()).map(|s| s.to_string());
                        for k in m.keys().filter_map(|k| k.as_str()) {
                            if !["field", "heading", "empty", "align", "width", "tagPrefix"].contains(&k) {
                                errs.push(format!("columns[{i}] has no key `{k}` (keys: field, heading, empty, align, width, tagPrefix)"));
                            }
                        }
                        let field = s("field").filter(|f| !f.trim().is_empty());
                        let tag_prefix = s("tagPrefix").filter(|f| !f.is_empty());
                        if field.is_some() == tag_prefix.is_some() {
                            errs.push(format!("columns[{i}] needs exactly one of `field:` and `tagPrefix:`"));
                            continue;
                        }
                        let align = match s("align").as_deref() {
                            None | Some("left") => Align::Left,
                            Some("right") => Align::Right,
                            Some("center") => Align::Center,
                            Some(o) => {
                                errs.push(format!("columns[{i}] align `{o}` is not left, right or center"));
                                Align::Left
                            }
                        };
                        let field = field.map(|f| f.trim().to_string()).unwrap_or_default();
                        let heading = s("heading").unwrap_or_else(|| match &tag_prefix {
                            Some(p) => heading_of(p.trim_end_matches(['-', '_', ':'])),
                            None => heading_of(&field),
                        });
                        t.columns.push(Column { field, heading, empty: s("empty").unwrap_or_default(), align, width: s("width"), tag_prefix });
                    }
                    other => errs.push(format!("columns[{i}] must be a field name or a map with `field:`, got {}", yaml_inline(other))),
                }
            }
        }
        Some(v) => errs.push(format!("`columns:` must be a non-empty list, got {}", yaml_inline(v))),
    }
    t.rows = text("rows", &mut errs).filter(|r| r != "elements");
    t.where_ = text("where", &mut errs);
    t.caption = text("caption", &mut errs);
    t.label = text("label", &mut errs);
    t.nest = text("nest", &mut errs);
    match extra.get("sort") {
        None => {}
        Some(Y::String(s)) if !s.trim().is_empty() => t.sort.push(sort_key(s)),
        Some(Y::Sequence(l)) if l.iter().all(|x| x.as_str().is_some_and(|s| !s.trim().is_empty())) => t.sort = l.iter().filter_map(|x| x.as_str()).map(sort_key).collect(),
        Some(v) => errs.push(format!("`sort:` must be a field name or a list of them (`-field` for descending), got {}", yaml_inline(v))),
    }
    match extra.get("marks") {
        None => {}
        Some(Y::Sequence(l)) => {
            for (i, m) in l.iter().enumerate() {
                let get = |k: &str| m.get(k).and_then(|v| v.as_str()).map(str::to_string);
                match (get("when"), get("symbol")) {
                    (Some(when), Some(symbol)) if !when.trim().is_empty() && !symbol.is_empty() => t.marks.push(Mark { when, symbol, note: get("note") }),
                    _ => errs.push(format!("marks[{i}] needs `when:` (a condition) and `symbol:`")),
                }
            }
        }
        Some(v) => errs.push(format!("`marks:` must be a list of {{when, symbol, note}}, got {}", yaml_inline(v))),
    }
    if t.nest.is_some() && t.rows.is_some() {
        errs.push("`nest:` applies to one row per element; it cannot be combined with `rows:`".into());
    }
    for expr in t.where_.iter().chain(t.marks.iter().map(|m| &m.when)) {
        if let Err(m) = check_expr(expr) {
            errs.push(m);
        }
    }
    if errs.is_empty() {
        Ok(Some(t))
    } else {
        Err(errs)
    }
}

fn sort_key(s: &str) -> (String, bool) {
    let s = s.trim();
    match s.strip_prefix('-') {
        Some(f) => (f.trim().to_string(), true),
        None => (s.to_string(), false),
    }
}

fn yaml_inline(v: &serde_yaml::Value) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "?".into())
}

// ── conditions ───────────────────────────────────────────────────────────────

/// One clause of a condition: `field == value`, `field != value`, or `field` (it has a value).
struct Clause<'a> {
    field: &'a str,
    /// (negated, value)
    cmp: Option<(bool, &'a str)>,
}

fn parse_clause(c: &str) -> Option<Clause<'_>> {
    let is_field = |f: &str| !f.is_empty() && f.chars().all(|ch| ch.is_ascii_alphanumeric() || "_.-".contains(ch)) && f.chars().next().is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_');
    for (op, neg) in [("==", false), ("!=", true)] {
        if let Some((f, v)) = c.split_once(op) {
            let (f, v) = (f.trim(), v.trim().trim_matches(|ch| ch == '"' || ch == '\''));
            return (is_field(f) && !v.is_empty()).then_some(Clause { field: f, cmp: Some((neg, v)) });
        }
    }
    is_field(c.trim()).then_some(Clause { field: c.trim(), cmp: None })
}

fn split_expr(expr: &str) -> Vec<Vec<(bool, &str)>> {
    expr.split(" or ")
        .map(|d| {
            d.split(" and ")
                .map(|c| match c.trim().strip_prefix("not ") {
                    Some(r) => (true, r.trim()),
                    None => (false, c.trim()),
                })
                .collect()
        })
        .collect()
}

fn check_expr(expr: &str) -> Result<(), String> {
    for (_, c) in split_expr(expr).into_iter().flatten() {
        if parse_clause(c).is_none() {
            return Err(format!("condition clause `{c}` is not `<field> == <value>`, `<field> != <value>` or `<field>` (joined by `and`, `or`, `not`)"));
        }
    }
    Ok(())
}

fn fields_of_expr(expr: &str) -> Vec<String> {
    split_expr(expr).into_iter().flatten().filter_map(|(_, c)| parse_clause(c).map(|p| p.field.to_string())).collect()
}

fn scalar_text(v: &J) -> String {
    match v {
        J::String(s) => s.clone(),
        J::Null => String::new(),
        other => other.to_string(),
    }
}

fn is_empty(v: &J) -> bool {
    match v {
        J::Null => true,
        J::String(s) => s.trim().is_empty(),
        J::Array(a) => a.is_empty(),
        J::Object(o) => o.is_empty(),
        _ => false,
    }
}

fn clause_holds(get: &dyn Fn(&str) -> Option<J>, c: &Clause<'_>) -> bool {
    let v = get(c.field).filter(|v| !is_empty(v));
    match c.cmp {
        None => v.is_some(),
        Some((neg, want)) => {
            let hit = match &v {
                None => false,
                Some(J::Array(a)) => a.iter().any(|x| scalar_text(x) == want),
                Some(x) => scalar_text(x) == want,
            };
            hit != neg
        }
    }
}

/// Evaluate a condition (`a == x and not b or c != y`) against a row's fields.
pub fn holds(get: &dyn Fn(&str) -> Option<J>, expr: &str) -> bool {
    split_expr(expr).iter().any(|conj| conj.iter().all(|(neg, c)| parse_clause(c).is_some_and(|p| clause_holds(get, &p)) != *neg))
}

/// One `expose:`/`filterCondition:` clause on an element, when it is a field comparison
/// (`type == PlanningItem`, `tag == external-input`, `status != draft`); `None` otherwise.
pub fn element_clause(e: &RawElement, clause: &str) -> Option<bool> {
    let c = parse_clause(clause)?;
    c.cmp?;
    let fm = serde_json::to_value(&e.frontmatter).unwrap_or(J::Null);
    Some(clause_holds(&|f| own_field(e, &fm, f), &c))
}

// ── fields ───────────────────────────────────────────────────────────────────

fn label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

fn dotted<'a>(v: &'a J, path: &str) -> Option<&'a J> {
    path.split('.').try_fold(v, |cur, k| cur.get(k))
}

/// A field an element answers from itself alone (no model lookups).
fn own_field(e: &RawElement, fm: &J, f: &str) -> Option<J> {
    Some(match f {
        "ref" => J::String(label(e)),
        "id" => J::String(e.frontmatter.id.clone()?),
        "qname" => J::String(e.qualified_name.clone()),
        "type" => fm.get("type")?.clone(),
        "tag" | "tags" => fm.get("tags")?.clone(),
        "file" => J::String(e.file_path.clone()),
        "summary" => J::String(e.doc.lines().map(str::trim).find(|l| !l.is_empty() && !l.starts_with('#'))?.to_string()),
        "openQuestions" => {
            let n = crate::questions::parse(e).0.iter().filter(|q| q.status == "open").count();
            J::from(n)
        }
        _ => match f.strip_prefix("custom.") {
            Some(k) => serde_json::to_value(e.frontmatter.custom_fields.get(k)?).ok()?,
            None => dotted(fm, f)?.clone(),
        },
    })
}

/// What a table is built against.
pub struct Ctx<'a> {
    /// The elements the view is evaluated in (the projection under `--config`).
    pub elements: &'a [RawElement],
    /// The whole model, where feature parameters take their values.
    pub full: &'a [RawElement],
    pub config: Option<&'a RawElement>,
}

struct Indices {
    /// qname -> ids of the elements that verify / satisfy / derive from it
    verified_by: HashMap<String, Vec<String>>,
    satisfied_by: HashMap<String, Vec<String>>,
    derived_children: HashMap<String, Vec<String>>,
}

impl Indices {
    fn new(elements: &[RawElement], resolver: &Resolver) -> Self {
        let mut ix = Indices { verified_by: HashMap::new(), satisfied_by: HashMap::new(), derived_children: HashMap::new() };
        for e in elements {
            let fm = &e.frontmatter;
            for (list, map) in [(&fm.verifies, &mut ix.verified_by), (&fm.satisfies, &mut ix.satisfied_by), (&fm.derived_from, &mut ix.derived_children)] {
                for r in list.iter().flatten() {
                    if let Some(t) = resolver.resolve_ref(elements, r) {
                        let l = map.entry(t.qualified_name.clone()).or_default();
                        if !l.contains(&label(e)) {
                            l.push(label(e));
                        }
                    }
                }
            }
        }
        for m in [&mut ix.verified_by, &mut ix.satisfied_by, &mut ix.derived_children] {
            m.values_mut().for_each(|l| l.sort());
        }
        ix
    }
}

/// One row before it is laid out.
struct Row<'a> {
    elem: &'a RawElement,
    fm: J,
    /// The list entry, for `rows:`.
    entry: Option<J>,
}

struct Lookup<'a> {
    ctx: &'a Ctx<'a>,
    resolver: Resolver,
    full_resolver: Resolver,
    ix: Indices,
    rows_key: Option<String>,
}

impl Lookup<'_> {
    fn element_field(&self, r: &Row<'_>, f: &str) -> Option<J> {
        let q = &r.elem.qualified_name;
        match f {
            "verifiedBy" => self.ix.verified_by.get(q).map(|l| J::from(l.clone())),
            "satisfiedBy" => self.ix.satisfied_by.get(q).map(|l| J::from(l.clone())),
            "derivedChildren" => self.ix.derived_children.get(q).map(|l| J::from(l.clone())),
            "body" => Some(J::String(crate::value_refs::render(self.ctx.elements, &self.resolver, r.elem.doc.trim()))),
            _ => own_field(r.elem, &r.fm, f),
        }
    }

    /// The value of `f` for a row: the list entry first, then the element (`element.<f>` names it outright).
    fn field(&self, r: &Row<'_>, f: &str) -> Option<J> {
        if let Some(ef) = f.strip_prefix("element.") {
            return self.element_field(r, ef);
        }
        if let Some(entry) = &r.entry {
            if let Some(v) = self.entry_field(r, entry, f) {
                return Some(v);
            }
        }
        self.element_field(r, f)
    }

    fn entry_field(&self, r: &Row<'_>, entry: &J, f: &str) -> Option<J> {
        let own = || dotted(entry, f).filter(|v| !v.is_null()).cloned();
        let name = entry.get("name").and_then(|v| v.as_str());
        match (self.rows_key.as_deref(), f) {
            (Some("questions"), "ref") => Some(J::String(format!("{}#{}", label(r.elem), entry.get("id").map(scalar_text)?))),
            (Some("parameters"), "key") => Some(J::String(format!("{}.{}", r.elem.qualified_name, name?))),
            (Some("parameters"), "effectiveValue" | "quantity") => {
                let text = crate::placeholders::param_text(self.ctx.full, &self.full_resolver, self.ctx.config, &r.elem.qualified_name, name?)?;
                Some(match f {
                    "quantity" => quantity(&text, entry.get("unit").and_then(|v| v.as_str())),
                    _ => J::String(text),
                })
            }
            (_, "quantity") => {
                let v = entry.get("value").or_else(|| entry.get("default")).filter(|v| !v.is_null())?;
                Some(quantity(&scalar_text(v), entry.get("unit").and_then(|v| v.as_str())))
            }
            (_, "value") if !entry.is_object() => Some(entry.clone()),
            _ => own(),
        }
    }
}

/// A value and unit kept apart, so each output writes the quantity its own way.
fn quantity(value: &str, unit: Option<&str>) -> J {
    // A derived quantity already carries its unit in the text.
    let unit = unit.filter(|_| value.trim().parse::<f64>().is_ok());
    serde_json::json!({ "__quantity": { "value": value, "unit": unit } })
}

/// A laid-out cell.
#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Text(String),
    Quantity { value: String, unit: Option<String> },
}

impl Cell {
    fn of(v: &J) -> Cell {
        if let Some(q) = v.get("__quantity") {
            return Cell::Quantity { value: scalar_text(&q["value"]), unit: q["unit"].as_str().map(str::to_string) };
        }
        Cell::Text(match v {
            J::Array(a) => a.iter().map(scalar_text).collect::<Vec<_>>().join(", "),
            J::Object(_) => v.to_string(),
            other => scalar_text(other),
        })
    }

    fn write(&self, esc: Escape) -> String {
        match self {
            Cell::Text(s) => esc.text(&s.split_whitespace().collect::<Vec<_>>().join(" ")),
            Cell::Quantity { value, unit } => esc.quantity(value, unit.as_deref()),
        }
    }

    fn plain(&self) -> String {
        self.write(Escape::None)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuiltRow {
    pub cells: Vec<Cell>,
    /// Nesting depth under `nest:`.
    pub depth: usize,
    /// The symbols of the marks that apply.
    pub marks: Vec<String>,
}

/// A table ready to write.
#[derive(Debug, Clone, PartialEq)]
pub struct Built {
    pub columns: Vec<Column>,
    pub rows: Vec<BuiltRow>,
    pub caption: Option<String>,
    pub label: String,
    /// (symbol, note) of the marks that were applied to a row.
    pub notes: Vec<(String, String)>,
}

/// The columns a view with no `columns:` is printed with.
pub fn default_table() -> Table {
    let col = |field: &str, heading: &str| Column { field: field.into(), heading: heading.into(), empty: String::new(), align: Align::Left, width: None, tag_prefix: None };
    Table { columns: vec![col("qname", "Element"), col("type", "Type"), col("name", "Name")], ..Table::default() }
}

fn compare(a: &Option<J>, b: &Option<J>) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    let text = |v: &J| Cell::of(v).plain();
    match (a, b) {
        (None, None) => Equal,
        (None, _) => Greater, // rows without the field go last
        (_, None) => Less,
        (Some(x), Some(y)) => {
            let (tx, ty) = (text(x), text(y));
            match (tx.trim().parse::<f64>(), ty.trim().parse::<f64>()) {
                (Ok(m), Ok(n)) => m.partial_cmp(&n).unwrap_or(Equal),
                _ => natural(&tx, &ty),
            }
        }
    }
}

/// Text order in which a run of digits compares as a number (`Q2` before `Q10`).
fn natural(a: &str, b: &str) -> std::cmp::Ordering {
    let split = |s: &str| -> Vec<(bool, String)> {
        let mut out: Vec<(bool, String)> = Vec::new();
        for c in s.chars() {
            let d = c.is_ascii_digit();
            match out.last_mut() {
                Some((was, run)) if *was == d => run.push(c),
                _ => out.push((d, c.to_string())),
            }
        }
        out
    };
    let (pa, pb) = (split(a), split(b));
    for (x, y) in pa.iter().zip(pb.iter()) {
        let o = match (x.0, y.0) {
            (true, true) => x.1.trim_start_matches('0').len().cmp(&y.1.trim_start_matches('0').len()).then_with(|| x.1.trim_start_matches('0').cmp(y.1.trim_start_matches('0'))),
            _ => x.1.cmp(&y.1),
        };
        if o != std::cmp::Ordering::Equal {
            return o;
        }
    }
    pa.len().cmp(&pb.len())
}

/// Build `table` over the elements a view exposes (`items`, in qualified-name order).
pub fn build(ctx: &Ctx<'_>, view: &RawElement, items: &[&RawElement], table: &Table) -> Built {
    let lookup = Lookup {
        ctx,
        resolver: Resolver::new(ctx.elements),
        full_resolver: Resolver::new(ctx.full),
        ix: Indices::new(ctx.elements, &Resolver::new(ctx.elements)),
        rows_key: table.rows.clone(),
    };
    let mut rows: Vec<Row<'_>> = Vec::new();
    for e in items {
        let fm = serde_json::to_value(&e.frontmatter).unwrap_or(J::Null);
        match &table.rows {
            None => rows.push(Row { elem: e, fm, entry: None }),
            Some(key) => {
                for entry in fm.get(key).and_then(|v| v.as_array()).cloned().unwrap_or_default() {
                    rows.push(Row { elem: e, fm: fm.clone(), entry: Some(entry) });
                }
            }
        }
    }
    if let Some(w) = &table.where_ {
        rows.retain(|r| holds(&|f| lookup.field(r, f), w));
    }
    if !table.sort.is_empty() {
        rows.sort_by(|a, b| {
            table.sort.iter().fold(std::cmp::Ordering::Equal, |acc, (f, desc)| {
                acc.then_with(|| {
                    let o = compare(&lookup.field(a, f), &lookup.field(b, f));
                    if *desc { o.reverse() } else { o }
                })
            })
        });
    }
    // `nest:` — children follow their parent, one level deeper.
    let mut order: Vec<(usize, usize)> = (0..rows.len()).map(|i| (i, 0)).collect();
    if let Some(nf) = &table.nest {
        let at: HashMap<&str, usize> = rows.iter().enumerate().map(|(i, r)| (r.elem.qualified_name.as_str(), i)).collect();
        let parent: Vec<Option<usize>> = rows
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let p = lookup.field(r, nf).map(|v| Cell::of(&v).plain())?;
                let t = lookup.resolver.resolve_ref(ctx.elements, p.trim())?;
                at.get(t.qualified_name.as_str()).copied().filter(|j| *j != i)
            })
            .collect();
        let mut kids: HashMap<usize, Vec<usize>> = HashMap::new();
        for (i, p) in parent.iter().enumerate() {
            if let Some(p) = p {
                kids.entry(*p).or_default().push(i);
            }
        }
        order.clear();
        let mut seen: BTreeSet<usize> = BTreeSet::new();
        fn walk(i: usize, d: usize, kids: &HashMap<usize, Vec<usize>>, seen: &mut BTreeSet<usize>, out: &mut Vec<(usize, usize)>) {
            if !seen.insert(i) {
                return;
            }
            out.push((i, d));
            for k in kids.get(&i).into_iter().flatten() {
                walk(*k, d + 1, kids, seen, out);
            }
        }
        for i in 0..rows.len() {
            if parent[i].is_none() {
                walk(i, 0, &kids, &mut seen, &mut order);
            }
        }
        // A cycle has no root: keep its rows, unindented.
        for i in 0..rows.len() {
            walk(i, 0, &kids, &mut seen, &mut order);
        }
    }
    let mut notes: Vec<(String, String)> = Vec::new();
    let built_rows = order
        .into_iter()
        .map(|(i, depth)| {
            let r = &rows[i];
            let cells = table
                .columns
                .iter()
                .map(|c| {
                    let v = match &c.tag_prefix {
                        Some(p) => r.elem.frontmatter.tags.iter().flatten().find_map(|t| t.strip_prefix(p.as_str())).map(|s| J::String(s.to_string())),
                        None => lookup.field(r, &c.field),
                    };
                    match v.filter(|v| !is_empty(v)) {
                        Some(v) => Cell::of(&v),
                        None => Cell::Text(c.empty.clone()),
                    }
                })
                .collect();
            let mut marks = Vec::new();
            for m in &table.marks {
                if holds(&|f| lookup.field(r, f), &m.when) {
                    marks.push(m.symbol.clone());
                    if let Some(n) = &m.note {
                        if !notes.iter().any(|(s, _)| s == &m.symbol) {
                            notes.push((m.symbol.clone(), n.clone()));
                        }
                    }
                }
            }
            BuiltRow { cells, depth, marks }
        })
        .collect();
    let leaf = view.qualified_name.rsplit("::").next().unwrap_or(&view.qualified_name);
    let mut slug = String::new();
    for (i, c) in leaf.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            slug.push('-');
        }
        slug.extend(c.to_lowercase());
    }
    Built { columns: table.columns.clone(), rows: built_rows, caption: table.caption.clone(), label: table.label.clone().unwrap_or_else(|| format!("tab:{}", slug.replace('_', "-"))), notes }
}

// ── output ───────────────────────────────────────────────────────────────────

/// The document formats a table is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Latex,
    Markdown,
    Csv,
    Html,
}

impl Format {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "latex" | "tex" => Format::Latex,
            "markdown" | "md" => Format::Markdown,
            "csv" => Format::Csv,
            "html" => Format::Html,
            _ => return None,
        })
    }
}

impl Built {
    fn cell(&self, row: &BuiltRow, i: usize, esc: Escape) -> String {
        let mut s = row.cells[i].write(esc);
        if i == 0 {
            for m in &row.marks {
                s.push_str(&match esc {
                    Escape::Latex => format!("\\textsuperscript{{{}}}", esc.text(m)),
                    Escape::Html => format!("<sup>{}</sup>", esc.text(m)),
                    _ => esc.text(m),
                });
            }
        }
        s
    }

    pub fn write(&self, format: Format) -> String {
        match format {
            Format::Latex => self.latex(),
            Format::Markdown => self.markdown(),
            Format::Csv => self.csv(),
            Format::Html => self.html(),
        }
    }

    /// A `longtable` (breaks over pages, header repeated) with `booktabs` rules.
    fn latex(&self) -> String {
        let e = Escape::Latex;
        let spec: String = self
            .columns
            .iter()
            .map(|c| match (&c.width, c.align) {
                (Some(w), _) => format!("p{{{w}}}"),
                (None, Align::Right) => "r".into(),
                (None, Align::Center) => "c".into(),
                (None, Align::Left) => "l".into(),
            })
            .collect();
        let head = format!("{} \\\\\n", self.columns.iter().map(|c| e.text(&c.heading)).collect::<Vec<_>>().join(" & "));
        let n = self.columns.len();
        let mut out = format!("\\begin{{longtable}}{{{spec}}}\n");
        if let Some(c) = &self.caption {
            out.push_str(&format!("\\caption{{{}}}\\label{{{}}}\\\\\n", e.text(c), self.label));
        }
        out.push_str(&format!("\\toprule\n{head}\\midrule\n\\endfirsthead\n\\toprule\n{head}\\midrule\n\\endhead\n"));
        out.push_str(&format!("\\midrule\n\\multicolumn{{{n}}}{{r}}{{\\footnotesize\\itshape continued}} \\\\\n\\endfoot\n\\bottomrule\n"));
        for (sym, note) in &self.notes {
            out.push_str(&format!("\\multicolumn{{{n}}}{{l}}{{\\footnotesize \\textsuperscript{{{}}} {}}} \\\\\n", e.text(sym), e.text(note)));
        }
        out.push_str("\\endlastfoot\n");
        for r in &self.rows {
            let cells: Vec<String> = (0..n)
                .map(|i| {
                    let c = self.cell(r, i, e);
                    if i == 0 && r.depth > 0 { format!("\\hspace{{{}em}}{c}", r.depth) } else { c }
                })
                .collect();
            out.push_str(&format!("{} \\\\\n", cells.join(" & ")));
        }
        out.push_str("\\end{longtable}\n");
        out
    }

    fn markdown(&self) -> String {
        let e = Escape::Markdown;
        let mut out = String::new();
        if let Some(c) = &self.caption {
            out.push_str(&format!("**{}**\n\n", e.text(c)));
        }
        out.push_str(&format!("| {} |\n", self.columns.iter().map(|c| e.text(&c.heading)).collect::<Vec<_>>().join(" | ")));
        out.push_str(&format!(
            "|{}|\n",
            self.columns
                .iter()
                .map(|c| match c.align {
                    Align::Left => "---",
                    Align::Right => "---:",
                    Align::Center => ":---:",
                })
                .collect::<Vec<_>>()
                .join("|")
        ));
        for r in &self.rows {
            let cells: Vec<String> = (0..self.columns.len())
                .map(|i| {
                    let c = self.cell(r, i, e);
                    if i == 0 && r.depth > 0 { format!("{}{c}", "&nbsp;&nbsp;".repeat(r.depth)) } else { c }
                })
                .collect();
            out.push_str(&format!("| {} |\n", cells.join(" | ")));
        }
        for (sym, note) in &self.notes {
            out.push_str(&format!("\n{} {}\n", e.text(sym), e.text(note)));
        }
        out
    }

    /// RFC 4180: a header row, fields quoted when they hold a comma, a quote or a line break.
    fn csv(&self) -> String {
        let q = |s: &str| if s.contains([',', '"', '\n', '\r']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_string() };
        let mut out = format!("{}\r\n", self.columns.iter().map(|c| q(&c.heading)).collect::<Vec<_>>().join(","));
        for r in &self.rows {
            out.push_str(&format!("{}\r\n", (0..self.columns.len()).map(|i| q(&self.cell(r, i, Escape::None))).collect::<Vec<_>>().join(",")));
        }
        out
    }

    fn html(&self) -> String {
        let e = Escape::Html;
        let align = |c: &Column| match c.align {
            Align::Left => "",
            Align::Right => " style=\"text-align:right\"",
            Align::Center => " style=\"text-align:center\"",
        };
        let mut out = format!("<table id=\"{}\">\n", e.text(&self.label));
        if let Some(c) = &self.caption {
            out.push_str(&format!("<caption>{}</caption>\n", e.text(c)));
        }
        out.push_str(&format!("<thead><tr>{}</tr></thead>\n<tbody>\n", self.columns.iter().map(|c| format!("<th{}>{}</th>", align(c), e.text(&c.heading))).collect::<String>()));
        for r in &self.rows {
            let cells: String = (0..self.columns.len())
                .map(|i| {
                    let c = &self.columns[i];
                    let style = if i == 0 && r.depth > 0 { format!(" style=\"padding-left:{}em\"", r.depth * 2) } else { align(c).to_string() };
                    format!("<td{style}>{}</td>", self.cell(r, i, e))
                })
                .collect();
            out.push_str(&format!("<tr>{cells}</tr>\n"));
        }
        out.push_str("</tbody>\n");
        if !self.notes.is_empty() {
            let notes: Vec<String> = self.notes.iter().map(|(s, n)| format!("<sup>{}</sup> {}", e.text(s), e.text(n))).collect();
            out.push_str(&format!("<tfoot><tr><td colspan=\"{}\">{}</td></tr></tfoot>\n", self.columns.len(), notes.join("; ")));
        }
        out.push_str("</table>\n");
        out
    }

    /// The table as data: headings and rows of plain text.
    pub fn json(&self) -> J {
        serde_json::json!({
            "caption": self.caption,
            "label": self.label,
            "columns": self.columns.iter().map(|c| serde_json::json!({"field": if c.field.is_empty() { J::Null } else { J::String(c.field.clone()) }, "heading": c.heading})).collect::<Vec<_>>(),
            "rows": self.rows.iter().map(|r| serde_json::json!({
                "cells": r.cells.iter().map(|c| match c {
                    Cell::Text(s) => J::String(s.clone()),
                    Cell::Quantity { value, unit } => serde_json::json!({"value": value, "unit": unit}),
                }).collect::<Vec<_>>(),
                "depth": r.depth,
                "marks": r.marks,
            })).collect::<Vec<_>>(),
            "notes": self.notes.iter().map(|(s, n)| serde_json::json!({"symbol": s, "note": n})).collect::<Vec<_>>(),
        })
    }
}

// ── validation ───────────────────────────────────────────────────────────────

/// Fields a row of a known list answers to beside the keys its entries carry.
fn row_fields(rows: &str) -> &'static [&'static str] {
    match rows {
        "questions" => &["ref", "id", "text", "status", "recommended", "answer", "answeredBy", "answeredAt", "answerBy"],
        "parameters" => &["key", "name", "type", "unit", "default", "value", "range", "bindingTime", "latestBindingTime", "derivedFrom", "bindTo", "description", "buildVar", "effectiveValue", "quantity"],
        _ => &["quantity", "value"],
    }
}

/// What `validate` reports for a view's table declaration: `(is_error, message)` — an error for a
/// malformed declaration (`E507`), a warning for a column, sort key, `nest:` or condition that names
/// a field no row can have (`W504`).
pub fn findings(elements: &[RawElement], view: &RawElement, items: &[&RawElement]) -> Vec<(bool, String)> {
    let table = match parse(&view.frontmatter.extra) {
        Ok(None) => return Vec::new(),
        Ok(Some(t)) => t,
        Err(errs) => return errs.into_iter().map(|m| (true, m)).collect(),
    };
    let _ = elements;
    // Every key a frontmatter can carry, and the keys the exposed elements do carry.
    let schema: BTreeSet<String> = serde_json::to_value(crate::element::RawFrontmatter::default()).ok().and_then(|v| v.as_object().map(|o| o.keys().cloned().collect())).unwrap_or_default();
    let fms: Vec<J> = items.iter().map(|e| serde_json::to_value(&e.frontmatter).unwrap_or(J::Null)).collect();
    let element_known = |f: &str| -> bool {
        let head = f.split('.').next().unwrap_or(f);
        f.starts_with("custom.") || ELEMENT_FIELDS.contains(&f) || schema.contains(head) || fms.iter().any(|fm| fm.get(head).is_some())
    };
    let mut out = Vec::new();
    let entries: Vec<&J> = match &table.rows {
        None => Vec::new(),
        Some(key) => fms.iter().filter_map(|fm| fm.get(key).and_then(|v| v.as_array())).flatten().collect(),
    };
    if let Some(key) = &table.rows {
        let is_list_somewhere = fms.iter().any(|fm| fm.get(key).is_some_and(|v| v.is_array()));
        if !is_list_somewhere && !schema.contains(key) && !items.is_empty() {
            out.push((false, format!("`rows: {key}` names a list that none of the {} exposed element(s) carries", items.len())));
        }
    }
    let known = |f: &str| -> bool {
        if let Some(ef) = f.strip_prefix("element.") {
            return element_known(ef);
        }
        match &table.rows {
            None => element_known(f),
            Some(key) => {
                let head = f.split('.').next().unwrap_or(f);
                row_fields(key).contains(&f)
                    || entries.iter().any(|e| e.get(head).is_some())
                    || element_known(f)
                    // An unfamiliar list with no entries yet: nothing to judge the field against.
                    || (entries.is_empty() && !matches!(key.as_str(), "questions" | "parameters"))
            }
        }
    };
    let mut check = |what: &str, f: &str| {
        if !known(f) {
            let hint = match &table.rows {
                Some(k) => format!("a key of a `{k}:` entry, or a field of the element (`element.<field>`)"),
                None => "a frontmatter field, `custom.<key>`, or one of ref, qname, type, summary, body, verifiedBy, satisfiedBy, derivedChildren, openQuestions".to_string(),
            };
            out.push((false, format!("{what} names unknown field `{f}` — expected {hint}")));
        }
    };
    for (i, c) in table.columns.iter().enumerate() {
        if c.tag_prefix.is_none() {
            check(&format!("columns[{i}]"), &c.field);
        }
    }
    for (f, _) in &table.sort {
        check("`sort:`", f);
    }
    if let Some(n) = &table.nest {
        check("`nest:`", n);
    }
    for f in table.where_.iter().flat_map(|w| fields_of_expr(w)) {
        check("`where:`", &f);
    }
    for f in table.marks.iter().flat_map(|m| fields_of_expr(&m.when)) {
        check("`marks:`", &f);
    }
    out
}

/// Whether an element is a view that may declare a table.
pub fn is_view(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::View) | Some(ElementType::ViewDef))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extra(yaml: &str) -> HashMap<String, serde_yaml::Value> {
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn a_declaration_parses_and_defaults_its_headings() {
        let t = parse(&extra("columns:\n  - ref\n  - {field: answeredBy}\n  - {field: text, heading: Question, empty: \"—\", align: right, width: 6cm}\n  - {tagPrefix: phase-}\nrows: questions\nwhere: status == open\nsort: [-ref, text]\ncaption: Open questions\n")).unwrap().unwrap();
        assert_eq!(t.columns.iter().map(|c| c.heading.as_str()).collect::<Vec<_>>(), ["Ref", "Answered by", "Question", "Phase"]);
        assert_eq!((t.columns[2].empty.as_str(), t.columns[2].align, t.columns[2].width.as_deref()), ("—", Align::Right, Some("6cm")));
        assert_eq!(t.rows.as_deref(), Some("questions"));
        assert_eq!(t.sort, vec![("ref".to_string(), true), ("text".to_string(), false)]);
        assert!(parse(&extra("name: x\n")).unwrap().is_none());
    }

    #[test]
    fn a_malformed_declaration_lists_what_is_wrong() {
        let e = parse(&extra("columns: [{field: a, colour: red}, {heading: x}, 7]\nsort: 3\nwhere: \"a >= 2\"\nmarks: [{symbol: x}]\n")).unwrap_err();
        let all = e.join("\n");
        for want in ["no key `colour`", "exactly one of `field:` and `tagPrefix:`", "columns[2] must be", "`sort:` must be", "clause `a >= 2`", "marks[0] needs"] {
            assert!(all.contains(want), "{want}: {all}");
        }
        assert!(parse(&extra("caption: x\n")).unwrap_err()[0].contains("needs `columns:`"));
    }

    #[test]
    fn conditions_compare_fields_and_lists() {
        let get = |f: &str| match f {
            "status" => Some(J::String("open".into())),
            "tags" => Some(serde_json::json!(["a", "external-input"])),
            _ => None,
        };
        assert!(holds(&get, "status == open and tags == external-input"));
        assert!(holds(&get, "status != closed"));
        assert!(!holds(&get, "status == closed or not tags"));
        assert!(holds(&get, "missing == x or status"));
        assert!(!holds(&get, "missing"));
    }

    #[test]
    fn digits_sort_as_numbers() {
        assert_eq!(natural("PI-X#Q2", "PI-X#Q10"), std::cmp::Ordering::Less);
        assert_eq!(natural("a", "b"), std::cmp::Ordering::Less);
    }
}
