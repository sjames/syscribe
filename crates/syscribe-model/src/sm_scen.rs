//! Executable Gherkin scenarios for state machines (REQ-TRS-SMSCEN-001, GH #280): extraction of the
//! scenarios of a `TestCase` body, Scenario Outline expansion, the closed step vocabulary, and running a
//! scenario against a [`Machine`] to a verdict.

use serde::Serialize;

use crate::expr::{self, Value};
use crate::ir::Machine;
use crate::sm_run::{Options, Run, Step as TraceStep};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kw {
    Given,
    When,
    Then,
}

#[derive(Debug, Clone)]
pub struct GStep {
    pub kw: Kw,
    pub text: String,
    /// 1-based line within the `TestCase` body.
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Scenario {
    /// For an expanded outline row: `<title> [row N]`.
    pub title: String,
    /// The outline's own title, for an expanded row.
    pub outline: Option<String>,
    pub steps: Vec<GStep>,
    /// Gherkin tags (`@invariant`), the scenario's own after those of the feature.
    pub tags: Vec<String>,
    /// The requirement references of `@verifies REQ-X[,REQ-Y]` tags on the scenario or its feature (GH #344).
    pub verifies: Vec<String>,
    /// `# verifies: REQ-X` comments on the line before a step: (the step's line, the references).
    pub step_verifies: Vec<(usize, Vec<String>)>,
    /// Named properties (`Where <name> is <expression>`, GH #342): the scenario's own after those of its feature or
    /// Background, a scenario's property replacing the feature's of the same name.
    pub props: Vec<Prop>,
    /// Malformed `Where`/`Except` lines: (line, reason), reported as `E886`.
    pub prop_errors: Vec<(usize, String)>,
}

/// A named property of an invariant (GH #342): `Where <name> is <expression>`, with the `Except under <faults>:
/// <expression>` clauses that follow it.
#[derive(Debug, Clone, PartialEq)]
pub struct Prop {
    pub name: String,
    pub expr: String,
    pub line: usize,
    pub excepts: Vec<Except>,
}

/// `Except under A and B: <expression>`: the property in force when every listed fault is injected.
#[derive(Debug, Clone, PartialEq)]
pub struct Except {
    pub faults: Vec<String>,
    pub expr: String,
    pub line: usize,
}

impl Except {
    pub fn label(&self) -> String {
        format!("Except under {}", self.faults.join(" and "))
    }
}

/// Which clause of a property applies when `active` faults are injected.
#[derive(Debug, Clone, PartialEq)]
pub enum Clause<'a> {
    Full(&'a str),
    Degraded(&'a Except),
}

impl Prop {
    /// The clause in force: the full property unless an `Except` clause's faults are all injected; of several such
    /// clauses the one whose faults include those of every other. `Err` when no clause covers the others: the
    /// degradations are not combined, the set needs a clause of its own.
    pub fn clause(&self, active: &[&str]) -> Result<Clause<'_>, String> {
        let hits: Vec<&Except> = self.excepts.iter().filter(|e| e.faults.iter().all(|f| active.contains(&f.as_str()))).collect();
        if hits.is_empty() {
            return Ok(Clause::Full(&self.expr));
        }
        let covers = |a: &Except, b: &Except| b.faults.iter().all(|f| a.faults.contains(f));
        match hits.iter().find(|a| hits.iter().all(|b| covers(a, b))) {
            Some(e) => Ok(Clause::Degraded(e)),
            None => Err(format!(
                "no applicable degraded clause for '{}': {} apply and none covers the others; add `Except under {}: …`",
                self.name,
                // the clauses no other applicable clause covers: the ones a combined clause has to replace
                hits.iter().filter(|a| !hits.iter().any(|b| b.faults.len() > a.faults.len() && covers(b, a))).map(|e| format!("`{}`", e.label())).collect::<Vec<_>>().join(", "),
                {
                    let mut all: Vec<&str> = hits.iter().flat_map(|e| e.faults.iter().map(String::as_str)).collect();
                    all.sort();
                    all.dedup();
                    all.join(" and ")
                }
            )),
        }
    }
}

/// A `Where <name> is <expression>` line (the keyword already matched): `Err` when it is malformed.
fn parse_where(rest: &str) -> Result<(String, String), String> {
    let lower = rest.to_ascii_lowercase();
    let Some(i) = find_unquoted(&lower, " is ") else { return Err("`Where <name> is <expression>`".into()) };
    let (name, e) = (rest[..i].trim(), rest[i + 4..].trim());
    if name.is_empty() || e.is_empty() {
        return Err("`Where <name> is <expression>`".into());
    }
    check_prop_expr(e)?;
    Ok((name.to_string(), e.to_string()))
}

/// An `Except under A and B: <expression>` line (the keyword already matched).
fn parse_except(rest: &str) -> Result<(Vec<String>, String), String> {
    let Some((ids, e)) = rest.split_once(':') else { return Err("`Except under <fault> [and <fault>…]: <expression>`".into()) };
    let mut faults: Vec<String> = Vec::new();
    for id in ids.split(" and ").flat_map(|x| x.split(',')).map(str::trim) {
        if id.is_empty() || id.contains(char::is_whitespace) {
            return Err(format!("'{}' is not a list of fault ids joined by `and`", ids.trim()));
        }
        if !faults.iter().any(|f| f == id) {
            faults.push(id.to_string());
        }
    }
    let e = e.trim();
    if faults.is_empty() || e.is_empty() {
        return Err("`Except under <fault> [and <fault>…]: <expression>`".into());
    }
    check_prop_expr(e)?;
    faults.sort();
    Ok((faults, e.to_string()))
}

/// Whether `e` holds a Scenario Outline `<placeholder>`: such an expression is checked once a row fills it in.
fn has_placeholder(e: &str) -> bool {
    let b = e.as_bytes();
    (0..b.len()).any(|i| {
        b[i] == b'<' && b.get(i + 1).is_some_and(|c| c.is_ascii_alphabetic() || *c == b'_') && {
            let rest = &e[i + 1..];
            rest.find('>').is_some_and(|j| rest[..j].bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b' '))
        }
    })
}

/// The syntax check of a property expression; an outline expression is checked after expansion instead.
fn check_prop_expr(e: &str) -> Result<(), String> {
    if has_placeholder(e) {
        return Ok(());
    }
    crate::expr::parse(e).map(|_| ()).map_err(|x| format!("`{e}` is not an expression: {x}"))
}

/// Add a `Where`/`Except` line to the properties of one scope. A `Where` may replace a property of an enclosing
/// scope (one declared before `scope_start`), but naming a property twice in the same scope is an error.
fn add_prop_line(props: &mut Vec<Prop>, errors: &mut Vec<(usize, String)>, t: &str, line: usize, scope_start: usize) {
    let lower = t.to_ascii_lowercase();
    if lower.starts_with("where ") {
        match parse_where(&t["where ".len()..]) {
            Ok((name, expr)) => {
                if let Some(p) = props.iter().find(|p| p.name.eq_ignore_ascii_case(&name) && p.line >= scope_start) {
                    errors.push((line, format!("property '{name}' is already declared at line {} in the same scope", p.line)));
                    return;
                }
                props.retain(|p| !p.name.eq_ignore_ascii_case(&name));
                props.push(Prop { name, expr, line, excepts: vec![] });
            }
            Err(e) => errors.push((line, e)),
        }
    } else {
        match (parse_except(&t["except under ".len()..]), props.last_mut()) {
            (Err(e), _) => errors.push((line, e)),
            (Ok(_), None) => errors.push((line, "an `Except under` clause belongs to the `Where` before it, and there is none".into())),
            (Ok((faults, expr)), Some(p)) => {
                if p.excepts.iter().any(|x| x.faults == faults) {
                    errors.push((line, format!("'{}' already has a clause `Except under {}`", p.name, faults.join(" and "))));
                } else {
                    p.excepts.push(Except { faults, expr, line });
                }
            }
        }
    }
}

/// `E886` for `Where`/`Except` lines in a scenario whose subject is not a composite: properties are only checked
/// there, so on a state machine or an action they would be silently ignored.
pub(crate) fn stray_prop_findings(sc: &Scenario, seen: &mut std::collections::HashSet<(usize, String)>) -> Vec<(usize, String)> {
    let lines = sc.props.iter().flat_map(|p| std::iter::once(p.line).chain(p.excepts.iter().map(|e| e.line))).chain(sc.prop_errors.iter().map(|(l, _)| *l));
    let mut out = Vec::new();
    for line in lines {
        if seen.insert((line, "<prop>".into())) {
            out.push((line, "`Where`/`Except under` properties are for composite scenarios (a TestCase whose subject is a system); this subject ignores them".into()));
        }
    }
    out
}

fn is_prop_line(lower: &str) -> bool {
    lower.starts_with("where ") || lower.starts_with("except under ")
}

/// The requirement references of the `@verifies` tags on one tag line: `@verifies REQ-A,REQ-B`, `@verifies:REQ-A` or
/// `@verifies(REQ-A, REQ-B)`; the references run up to the next `@` tag.
pub fn verifies_tags(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut on = false;
    for tok in line.split_whitespace() {
        if tok.starts_with('#') {
            // a comment ends the tag line
            break;
        }
        if let Some(rest) = tok.strip_prefix('@') {
            let lower = rest.to_ascii_lowercase();
            on = lower == "verifies" || lower.starts_with("verifies:") || lower.starts_with("verifies(");
            if !on {
                continue;
            }
            let rest = &rest["verifies".len()..];
            out.extend(split_refs(rest.trim_start_matches([':', '('])));
            continue;
        }
        if on {
            out.extend(split_refs(tok));
        }
    }
    out
}

fn split_refs(s: &str) -> Vec<String> {
    s.split(',').map(|x| x.trim().trim_end_matches(')').trim()).filter(|x| !x.is_empty()).map(String::from).collect()
}

/// The references of a `# verifies: REQ-X, REQ-Y` comment line, if it is one.
fn verifies_comment(t: &str) -> Option<Vec<String>> {
    let body = t.strip_prefix('#')?.trim();
    let lower = body.to_ascii_lowercase();
    let rest = lower.strip_prefix("verifies:").map(|_| &body["verifies:".len()..])?;
    Some(split_refs(rest))
}

impl Scenario {
    /// Whether the scenario carries the tag, ignoring case and the leading `@`.
    pub fn has_tag(&self, tag: &str) -> bool {
        let want = tag.trim_start_matches('@');
        self.tags.iter().any(|t| t.trim_start_matches('@').eq_ignore_ascii_case(want))
    }

    /// The requirement references the scenario claims: its and its feature's `@verifies` tags and its steps'
    /// `# verifies:` comments, in order, without repeats. Empty when it claims none, so it inherits the TestCase's.
    pub fn claimed(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for r in self.verifies.iter().chain(self.step_verifies.iter().flat_map(|(_, v)| v)) {
            if !out.contains(r) {
                out.push(r.clone());
            }
        }
        out
    }
}

fn is_fence(line: &str) -> Option<bool> {
    let t = line.trim_start();
    if t.starts_with("```gherkin") {
        Some(true)
    } else if t.starts_with("```") {
        Some(false)
    } else {
        None
    }
}

fn kw_of(line: &str) -> Option<(Option<Kw>, &str, bool)> {
    let t = line.trim_start();
    for (w, k) in [("given", Some(Kw::Given)), ("when", Some(Kw::When)), ("then", Some(Kw::Then)), ("and", None), ("but", None)] {
        if let (Some(head), Some(rest)) = (t.get(..w.len()), t.get(w.len()..)) {
            if head.eq_ignore_ascii_case(w) && rest.starts_with(' ') {
                return Some((k, rest.trim(), k.is_none()));
            }
        }
    }
    None
}

fn cells(line: &str) -> Vec<String> {
    let t = line.trim();
    let t = t.strip_prefix('|').unwrap_or(t);
    let t = t.strip_suffix('|').filter(|_| !t.ends_with("\\|")).unwrap_or(t);
    let mut out = vec![String::new()];
    let mut chars = t.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'|') => {
                out.last_mut().unwrap().push('|');
                chars.next();
            }
            '|' => out.push(String::new()),
            c => out.last_mut().unwrap().push(c),
        }
    }
    out.into_iter().map(|c| c.trim().to_string()).collect()
}

/// `find` that ignores matches inside single or double quotes.
pub(crate) fn find_unquoted(hay: &str, needle: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (i, c) in hay.char_indices() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                }
            }
            None => {
                if c == '\'' || c == '"' {
                    quote = Some(c);
                } else if hay[i..].starts_with(needle) {
                    return Some(i);
                }
            }
        }
    }
    None
}

/// Every scenario of every ```gherkin block of a `TestCase` body, outlines expanded one scenario per row.
pub fn extract_scenarios(doc: &str) -> Vec<Scenario> {
    struct Raw {
        title: String,
        tags: Vec<String>,
        verifies: Vec<String>,
        step_verifies: Vec<(usize, Vec<String>)>,
        outline: bool,
        steps: Vec<GStep>,
        header: Vec<String>,
        header_pending: bool,
        rows: Vec<(Vec<String>, Vec<String>)>,
        /// The `@verifies` tags of the Examples table each row came from.
        row_verifies: Vec<Vec<String>>,
        examples_verifies: Vec<String>,
        props: Vec<Prop>,
        prop_errors: Vec<(usize, String)>,
        /// The line of the scenario's own header: its properties are those declared from here.
        start: usize,
    }
    let mut raws: Vec<Raw> = Vec::new();
    let mut background: Vec<GStep> = Vec::new();
    let mut in_background = false;
    let mut in_block = false;
    let mut in_doc = false;
    let mut in_examples = false;
    let mut last_kw = Kw::Given;
    let mut feature_tags: Vec<String> = Vec::new();
    let mut pending_tags: Vec<String> = Vec::new();
    let mut feature_verifies: Vec<String> = Vec::new();
    let mut pending_verifies: Vec<String> = Vec::new();
    let mut step_comment: Option<Vec<String>> = None;
    // the properties of the block's feature and Background, which every scenario after them sees
    let mut feature_props: Vec<Prop> = Vec::new();
    let mut feature_prop_errors: Vec<(usize, String)> = Vec::new();
    // where the feature-level scope starts, and the feature's own properties that each `Rule:` starts from
    let mut scope_start = 0usize;
    let mut rule_base: Option<(Vec<Prop>, Vec<(usize, String)>)> = None;
    // whether a `Where`/`Except` line belongs to the latest scenario rather than to the feature or rule
    let mut in_scenario = false;
    for (i, line) in doc.lines().enumerate() {
        if let Some(open) = is_fence(line) {
            // tags belong to the block they are in: none carry over from one block to the next
            feature_tags.clear();
            pending_tags.clear();
            feature_verifies.clear();
            pending_verifies.clear();
            step_comment = None;
            feature_props.clear();
            feature_prop_errors.clear();
            scope_start = 0;
            rule_base = None;
            in_scenario = false;
            in_block = open;
            in_examples = false;
            in_background = false;
            in_doc = false;
            continue;
        }
        if !in_block {
            continue;
        }
        let t = line.trim();
        if t.starts_with("\"\"\"") || t.starts_with("```") {
            in_doc = !in_doc;
            continue;
        }
        if in_doc {
            continue;
        }
        let lower = t.to_ascii_lowercase();
        if t.starts_with('@') {
            pending_tags.extend(t.split_whitespace().filter(|x| x.starts_with('@')).map(String::from));
            pending_verifies.extend(verifies_tags(t));
            continue;
        }
        if let Some(refs) = verifies_comment(t) {
            step_comment = Some(refs);
            continue;
        }
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if lower.starts_with("scenario") || lower.starts_with("example:") {
            // a `# verifies:` comment belongs to the next step of the same scenario
            step_comment = None;
        }
        if lower.starts_with("scenario outline:") || lower.starts_with("scenario template:") {
            let title = t[t.find(':').unwrap() + 1..].trim().to_string();
            raws.push(Raw { title, tags: feature_tags.iter().cloned().chain(std::mem::take(&mut pending_tags)).collect(), verifies: feature_verifies.iter().cloned().chain(std::mem::take(&mut pending_verifies)).collect(), step_verifies: vec![], outline: true, steps: vec![], header: vec![], header_pending: false, rows: vec![], row_verifies: vec![], examples_verifies: vec![], props: feature_props.clone(), prop_errors: feature_prop_errors.clone(), start: i + 1 });
            in_examples = false;
            in_background = false;
            in_scenario = true;
            continue;
        }
        if lower.starts_with("scenario:") || lower.starts_with("example:") {
            let title = t[t.find(':').unwrap() + 1..].trim().to_string();
            raws.push(Raw { title, tags: feature_tags.iter().cloned().chain(std::mem::take(&mut pending_tags)).collect(), verifies: feature_verifies.iter().cloned().chain(std::mem::take(&mut pending_verifies)).collect(), step_verifies: vec![], outline: false, steps: vec![], header: vec![], header_pending: false, rows: vec![], row_verifies: vec![], examples_verifies: vec![], props: feature_props.clone(), prop_errors: feature_prop_errors.clone(), start: i + 1 });
            in_examples = false;
            in_background = false;
            in_scenario = true;
            continue;
        }
        if lower.starts_with("examples:") || lower.starts_with("scenarios:") {
            pending_tags.clear();
            let ex_verifies = std::mem::take(&mut pending_verifies);
            step_comment = None;
            in_examples = true;
            if let Some(cur) = raws.last_mut() {
                // a further Examples table has its own header row, and its own `@verifies` tags for its rows
                cur.header.clear();
                cur.header_pending = true;
                cur.examples_verifies = ex_verifies;
            }
            continue;
        }
        if lower.starts_with("background:") {
            pending_tags.clear();
            pending_verifies.clear();
            step_comment = None;
            in_background = true;
            in_examples = false;
            in_scenario = false;
            continue;
        }
        if lower.starts_with("feature:") || lower.starts_with("rule:") {
            if lower.starts_with("feature:") {
                feature_tags = std::mem::take(&mut pending_tags);
                feature_verifies = std::mem::take(&mut pending_verifies);
                rule_base = None;
            } else {
                // each rule sees the feature's properties plus its own, never an earlier rule's
                match &rule_base {
                    None => rule_base = Some((feature_props.clone(), feature_prop_errors.clone())),
                    Some((p, e)) => {
                        feature_props = p.clone();
                        feature_prop_errors = e.clone();
                    }
                }
                // tags on a Rule are not supported: they must not leak to the next scenario
                pending_tags.clear();
                pending_verifies.clear();
            }
            in_examples = false;
            in_background = false;
            in_scenario = false;
            scope_start = i + 1;
            continue;
        }
        if is_prop_line(&lower) {
            match raws.last_mut() {
                Some(cur) if in_scenario && !in_background => add_prop_line(&mut cur.props, &mut cur.prop_errors, t, i + 1, cur.start),
                _ => add_prop_line(&mut feature_props, &mut feature_prop_errors, t, i + 1, scope_start),
            }
            continue;
        }
        if in_background {
            if let Some((k, text, cont)) = kw_of(t) {
                let kw = if cont { last_kw } else { k.unwrap() };
                last_kw = kw;
                background.push(GStep { kw, text: text.to_string(), line: i + 1 });
                // a Background step runs in every scenario, so its claim is every scenario's
                if let Some(refs) = step_comment.take() {
                    feature_verifies.extend(refs);
                }
            }
            continue;
        }
        let Some(cur) = raws.last_mut() else { continue };
        if in_examples && t.starts_with('|') {
            let c = cells(t);
            if cur.header_pending {
                cur.header = c;
                cur.header_pending = false;
            } else {
                cur.rows.push((cur.header.clone(), c));
                cur.row_verifies.push(cur.examples_verifies.clone());
            }
            continue;
        }
        if let Some((k, text, cont)) = kw_of(t) {
            let kw = if cont { last_kw } else { k.unwrap() };
            last_kw = kw;
            cur.steps.push(GStep { kw, text: text.to_string(), line: i + 1 });
            if let Some(refs) = step_comment.take() {
                cur.step_verifies.push((i + 1, refs));
            }
        }
    }
    for r in raws.iter_mut() {
        let mut steps = background.clone();
        steps.append(&mut r.steps);
        r.steps = steps;
    }
    let mut out = Vec::new();
    for r in raws {
        if !r.outline {
            out.push(Scenario { title: r.title, outline: None, steps: r.steps, tags: r.tags, verifies: r.verifies, step_verifies: r.step_verifies, props: r.props, prop_errors: r.prop_errors });
            continue;
        }
        if r.rows.is_empty() {
            // An outline with no example rows would silently run nothing: make it an erroring scenario.
            out.push(Scenario { title: format!("{} [no examples]", r.title), outline: Some(r.title.clone()), steps: vec![], tags: r.tags.clone(), verifies: r.verifies.clone(), step_verifies: r.step_verifies.clone(), props: r.props.clone(), prop_errors: r.prop_errors.clone() });
            continue;
        }
        for (n, (header, row)) in r.rows.iter().enumerate() {
            let sub = |s: &str| {
                let mut s = s.to_string();
                for (h, v) in header.iter().zip(row) {
                    s = s.replace(&format!("<{h}>"), v);
                }
                s
            };
            let props: Vec<Prop> = r.props.iter().map(|p| Prop { expr: sub(&p.expr), excepts: p.excepts.iter().map(|e| Except { expr: sub(&e.expr), ..e.clone() }).collect(), ..p.clone() }).collect();
            // an outline property is checked once the row has filled in its placeholders
            let mut prop_errors = r.prop_errors.clone();
            for (line, e) in props.iter().flat_map(|p| std::iter::once((p.line, &p.expr)).chain(p.excepts.iter().map(|x| (x.line, &x.expr)))) {
                if let Err(x) = crate::expr::parse(e) {
                    prop_errors.push((line, format!("row {}: `{e}` is not an expression: {x}", n + 1)));
                }
            }
            out.push(Scenario {
                title: format!("{} [row {}]", r.title, n + 1),
                outline: Some(r.title.clone()),
                steps: r.steps.iter().map(|s| GStep { kw: s.kw, text: sub(&s.text), line: s.line }).collect(),
                tags: r.tags.clone(),
                verifies: r.verifies.iter().chain(r.row_verifies.get(n).into_iter().flatten()).cloned().collect(),
                step_verifies: r.step_verifies.clone(),
                props,
                prop_errors,
            });
        }
    }
    out
}

/// A parsed step of the closed vocabulary.
#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    InState(String),
    Set(String, String),
    Inject(String, Vec<(String, String)>),
    Elapse(String),
    Expect(String),
}

pub(crate) fn split_top(s: &str) -> Vec<String> {
    let (mut out, mut cur, mut depth, mut quote) = (Vec::new(), String::new(), 0i32, None::<char>);
    for c in s.chars() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                }
                cur.push(c);
            }
            None => match c {
                '\'' | '"' => {
                    quote = Some(c);
                    cur.push(c);
                }
                '(' => {
                    depth += 1;
                    cur.push(c);
                }
                ')' => {
                    depth -= 1;
                    cur.push(c);
                }
                ',' if depth == 0 => out.push(std::mem::take(&mut cur)),
                _ => cur.push(c),
            },
        }
    }
    out.push(cur);
    out.into_iter().map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

/// Parse one step against the closed vocabulary; `Err` is the reason it is outside it (`E886`).
pub fn parse_step(kw: Kw, text: &str) -> Result<Cmd, String> {
    let t = text.trim();
    let lower = t.to_ascii_lowercase();
    if let Some(i) = find_unquoted(&lower, " is in state ") {
        if kw == Kw::When {
            return Err("`is in state` is a Given or Then step".into());
        }
        let s = t[i + " is in state ".len()..].trim().trim_matches(['"', '\'']);
        if s.is_empty() {
            return Err("`is in state` needs a state".into());
        }
        let s = s.replace('/', ".");
        return Ok(if kw == Kw::Given { Cmd::InState(s) } else { Cmd::Expect(format!("\u{0}state:{s}")) });
    }
    for suffix in [" elapses", " elapse"] {
        if kw == Kw::When && lower.ends_with(suffix) {
            let d = t[..t.len() - suffix.len()].trim();
            return match expr::parse(d) {
                Ok(_) => Ok(Cmd::Elapse(d.to_string())),
                Err(e) => Err(format!("`{d}` is not a duration: {e}")),
            };
        }
    }
    if let Some(i) = find_unquoted(&lower, " is injected") {
        if kw != Kw::When {
            return Err("`is injected` is a When step".into());
        }
        let ev = t[..i].trim();
        if ev.is_empty() || ev.contains(char::is_whitespace) {
            return Err(format!("'{ev}' is not an event name"));
        }
        let rest = t[i + " is injected".len()..].trim();
        let mut fields = Vec::new();
        if !rest.is_empty() {
            let Some(list) = rest.strip_prefix("with ").or_else(|| rest.strip_prefix("WITH ")).or_else(|| rest.strip_prefix("With ")) else {
                return Err(format!("expected `with <field> = <expression>` after `is injected`, got `{rest}`"));
            };
            for f in split_top(list) {
                let Some((k, v)) = f.split_once('=') else {
                    return Err(format!("payload `{f}` is not <field> = <expression>"));
                };
                if let Err(e) = expr::parse(v.trim()) {
                    return Err(format!("payload value `{}`: {e}", v.trim()));
                }
                fields.push((k.trim().to_string(), v.trim().to_string()));
            }
        }
        return Ok(Cmd::Inject(ev.to_string(), fields));
    }
    if let Some(i) = find_unquoted(&lower, " becomes ") {
        if kw != Kw::When {
            return Err("`becomes` is a When step".into());
        }
        let (var, e) = (t[..i].trim(), t[i + " becomes ".len()..].trim());
        return match expr::parse(e) {
            Ok(_) if !var.is_empty() && !var.contains(char::is_whitespace) => Ok(Cmd::Set(var.to_string(), e.to_string())),
            Ok(_) => Err(format!("'{var}' is not a variable name")),
            Err(x) => Err(format!("`{e}`: {x}")),
        };
    }
    if kw == Kw::Given {
        if let Some(i) = find_unquoted(&lower, " is ") {
            let (var, e) = (t[..i].trim(), t[i + 4..].trim());
            if !var.is_empty() && !var.contains(char::is_whitespace) {
                return match expr::parse(e) {
                    Ok(_) => Ok(Cmd::Set(var.to_string(), e.to_string())),
                    Err(x) => Err(format!("`{e}`: {x}")),
                };
            }
        }
        return Err("a Given step is `<machine> is in state <state>` or `<variable> is <expression>`".into());
    }
    if kw == Kw::Then {
        return match expr::parse(t) {
            Ok(_) => Ok(Cmd::Expect(t.to_string())),
            Err(e) => Err(format!("`{t}` is not an expression: {e}")),
        };
    }
    Err("a When step is `<event> is injected [with …]`, `<variable> becomes <expression>` or `<duration> elapse[s]`".into())
}

/// `E886` findings of a `TestCase` body: `(line, message)` for each step outside the vocabulary.
pub fn vocabulary_findings(doc: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for sc in extract_scenarios(doc) {
        if sc.steps.is_empty() {
            let why = if sc.title.ends_with("[no examples]") { "the Scenario Outline has no Examples rows" } else { "the scenario has no steps" };
            out.push((0, format!("scenario '{}': {why}", sc.title)));
        }
        for st in &sc.steps {
            if !seen.insert((st.line, st.text.clone())) {
                continue;
            }
            if let Err(e) = parse_step(st.kw, &st.text) {
                out.push((st.line, format!("scenario '{}': step `{}` is outside the executable vocabulary: {e}", sc.title, st.text)));
            }
        }
        out.extend(stray_prop_findings(&sc, &mut seen));
    }
    out
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Pass,
    Fail,
    Error,
}

/// What one part of a composite exercised in a scenario (REQ-TRS-SYSCOV-001).
#[derive(Debug, Clone)]
pub struct PartCoverage {
    pub part: String,
    /// The qualified name of the part's state machine.
    pub machine: String,
    pub coverage: crate::sm_run::Coverage,
}

#[derive(Debug, Clone, Serialize)]
pub struct Verdict {
    pub scenario: String,
    pub outline: Option<String>,
    pub status: Status,
    pub message: Option<String>,
    pub steps: Vec<String>,
    pub trace: Vec<TraceStep>,
    #[serde(skip)]
    pub coverage: crate::sm_run::Coverage,
    #[serde(skip)]
    pub act_coverage: crate::act_run::ActCoverage,
    /// Per part of a composite; empty for a single machine.
    #[serde(skip)]
    pub part_coverage: Vec<PartCoverage>,
    /// What identifies each plant of the run (FMU hash, FMI version, step, platform); empty without plants.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub plants: Vec<serde_json::Value>,
    /// The messages exchanged between the parts of a composite, in emission order; empty for a single machine.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<String>,
    /// The degraded clauses (`Except under …`, GH #342) the scenario was judged by: `'<property>': Except under A`.
    /// Empty when every named property was judged in full.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub degraded: Vec<String>,
}

fn const_value(run: &Run, src: &str) -> Result<Value, String> {
    run.eval_str(src)
}

/// Run one scenario on a fresh run of `machine`.
pub fn run_scenario(machine: &Machine, sc: &Scenario, opts: &Options) -> Verdict {
    run_scenario_with(machine, sc, opts, None)
}

/// Builds the plants of one run; called once per scenario so every scenario starts with fresh plants.
pub type PlantFactory<'a> = &'a dyn Fn() -> Result<crate::plant::CoSim, Vec<String>>;

/// [`run_scenario`] with plant models co-simulated on the shared clock (REQ-TRS-FMI-001).
pub fn run_scenario_with(machine: &Machine, sc: &Scenario, opts: &Options, plants: Option<PlantFactory>) -> Verdict {
    let mut run = match plants {
        None => Run::new(machine, opts.clone()),
        Some(f) => match f() {
            Ok(c) => Run::with_plants(machine, opts.clone(), c),
            Err(es) => {
                return Verdict {
                    degraded: Vec::new(),
                    scenario: sc.title.clone(),
                    outline: sc.outline.clone(),
                    status: Status::Error,
                    message: Some(format!("plants: {}", es.join("; "))),
                    steps: vec![],
                    trace: vec![],
                    coverage: Default::default(),
                    act_coverage: Default::default(),
                    part_coverage: vec![],
                    plants: vec![],
                    messages: vec![],
                }
            }
        },
    };
    let steps: Vec<String> = sc.steps.iter().map(|s| format!("{} {}", match s.kw { Kw::Given => "Given", Kw::When => "When", Kw::Then => "Then" }, s.text)).collect();
    let done = |run: &Run, status: Status, message: Option<String>| Verdict {
        degraded: Vec::new(),
        scenario: sc.title.clone(),
        outline: sc.outline.clone(),
        status,
        message,
        steps: steps.clone(),
        trace: run.trace().to_vec(),
        coverage: run.coverage(),
        act_coverage: Default::default(),
        part_coverage: vec![],
        plants: run.plant_provenance(),
        messages: vec![],
    };
    if let Some(e) = run.error() {
        let e = e.to_string();
        return done(&run, Status::Error, Some(e));
    }
    if sc.steps.is_empty() {
        let why = if sc.title.ends_with("[no examples]") { "the Scenario Outline has no Examples rows" } else { "the scenario has no steps" };
        return done(&run, Status::Error, Some(why.to_string()));
    }
    for (i, st) in sc.steps.iter().enumerate() {
        let label = format!("step {} (`{}`)", i + 1, steps[i]);
        let cmd = match parse_step(st.kw, &st.text) {
            Ok(c) => c,
            Err(e) => return done(&run, Status::Error, Some(format!("{label}: {e}"))),
        };
        let applied: Result<(), String> = match &cmd {
            Cmd::InState(s) => run.force(s),
            Cmd::Set(var, e) => const_value(&run, e).and_then(|v| {
                run.set(var, v);
                Ok(())
            }),
            Cmd::Inject(ev, fields) => fields
                .iter()
                .map(|(k, v)| const_value(&run, v).map(|x| (k.clone(), x)))
                .collect::<Result<Vec<_>, _>>()
                .map(|p| {
                    let p: Vec<(&str, Value)> = p.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                    run.send(ev, &p);
                }),
            Cmd::Elapse(d) => match const_value(&run, d) {
                Ok(Value::Num(q)) if expr::unit_scale("s").map(|x| x.1) == Some(q.dim) && q.v >= 0.0 => {
                    run.advance(q.v);
                    Ok(())
                }
                Ok(v) => Err(format!("`{d}` is a {}, not a non-negative duration", v.kind())),
                Err(e) => Err(e),
            },
            Cmd::Expect(e) => {
                if let Some(s) = e.strip_prefix("\u{0}state:") {
                    match run.holds_state(s) {
                        Ok(true) => Ok(()),
                        Ok(false) => {
                            return done(&run, Status::Fail, Some(format!("{label}: expected the machine in state {s}, it is in {}", run.active().join(", "))));
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    match run.eval_str(e) {
                        Ok(Value::Bool(true)) => Ok(()),
                        Ok(Value::Bool(false)) => {
                            let vars: Vec<String> = machine.variables.iter().filter_map(|v| run.var(&v.name).map(|x| format!("{} = {}", v.name, expr::format_value(&x)))).collect();
                            return done(&run, Status::Fail, Some(format!("{label}: `{e}` is false (state {}; {})", run.active().join(", "), vars.join(", "))));
                        }
                        Ok(v) => Err(format!("`{e}` is a {}, not a boolean", v.kind())),
                        Err(x) => Err(x),
                    }
                }
            }
        };
        if let Err(e) = applied {
            return done(&run, Status::Error, Some(format!("{label}: {e}")));
        }
        if let Some(e) = run.error() {
            let e = e.to_string();
            return done(&run, Status::Error, Some(format!("{label}: run error: {e}")));
        }
    }
    done(&run, Status::Pass, None)
}

/// The state machine a scenario `TestCase` runs against: its `subject:`, else the single `StateDef` that
/// satisfies the requirements it verifies.
pub fn subject_of<'a>(elements: &'a [crate::element::RawElement], tc: &crate::element::RawElement) -> Result<&'a crate::element::RawElement, String> {
    use crate::element::ElementType;
    let resolver = crate::resolver::Resolver::new(elements);
    let is_machine = |e: &crate::element::RawElement| matches!(e.frontmatter.element_type, Some(ElementType::StateDef) | Some(ElementType::State) | Some(ElementType::ActionDef) | Some(ElementType::Action));
    // an explicit `subject:` may also be a composite part; inference from `satisfies` stays with machines
    let is_subject = |e: &crate::element::RawElement| is_machine(e) || matches!(e.frontmatter.element_type, Some(ElementType::Part) | Some(ElementType::PartDef));
    if let Some(s) = &tc.frontmatter.subject {
        return match resolver.resolve_ref(elements, s) {
            Some(e) if is_subject(e) => Ok(e),
            Some(e) => Err(format!("subject '{s}' is a {:?}, not a StateDef, ActionDef or composite Part", e.frontmatter.element_type)),
            None => Err(format!("subject '{s}' does not resolve")),
        };
    }
    let reqs: Vec<&crate::element::RawElement> =
        tc.frontmatter.verifies.iter().flatten().filter_map(|r| resolver.resolve_ref(elements, r)).collect();
    let mut found: Vec<&crate::element::RawElement> = Vec::new();
    for e in elements.iter().filter(|e| is_machine(e)) {
        let satisfies = e.frontmatter.satisfies.iter().flatten().filter_map(|r| resolver.resolve_ref(elements, r));
        if satisfies.into_iter().any(|r| reqs.iter().any(|q| q.qualified_name == r.qualified_name)) && !found.iter().any(|f| f.qualified_name == e.qualified_name) {
            found.push(e);
        }
    }
    match found.len() {
        1 => Ok(found[0]),
        0 => Err("no `subject:` and no StateDef satisfies the requirements this TestCase verifies".into()),
        _ => Err(format!("no `subject:` and {} StateDefs satisfy the requirements this TestCase verifies", found.len())),
    }
}
