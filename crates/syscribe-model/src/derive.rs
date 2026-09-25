//! Declarative derive: block evaluator (REQ-TRS-DERIVE-001..005, issue #60).
//!
//! Each element may declare a `derive:` mapping of fieldName → formula (the
//! typed `RawFrontmatter::derive` field — recognised, so never `W047`). Formulas
//! are evaluated in dependency order: a field is evaluated after every derived
//! field it reads (`self.<field>`, `elements["Qname"].<field>`, or a
//! `children`/`parent` aggregate), independent fields in file-walk then block
//! order. Cross-element references are resolved against the full element set.
//!
//! The evaluation pipeline:
//!   Walker → derive_pass(elements) → Validator
//!
//! The pass populates `RawElement.derived` for each element; the validator and
//! query layer read derived fields from there.
//!
//! Finding codes (GH #127 — moved off `E500`–`E502`, which belong to Allocation
//! resolution): `E504` cyclic dependency between derived fields (GH #141 —
//! the cyclic fields are skipped), `E505` formula parse error / malformed
//! block, `E506` unknown element reference in `elements["QName"]`.

use crate::element::RawElement;

/// `(code, file, message)`. Shared shape for `RawElement.derive_findings`,
/// which now carries findings from more than just this module's derive pass
/// (see that field's doc comment) — other passes reuse this constructor
/// instead of hand-rolling the tuple.
pub(crate) type Finding = (String, String, String);

pub(crate) fn finding(code: &str, file: &str, message: &str) -> Finding {
    (code.to_string(), file.to_string(), message.to_string())
}

// ── Expression AST ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Expr {
    Num(f64),
    Str(String),
    /// self.<field> or self.custom_fields.<key>
    SelfField(FieldPath),
    /// elements["Qname"].<field>
    ElementField { qname: String, path: FieldPath },
    /// sum/max/min/count/collect over a collection source
    Aggregate { op: AggOp, source: CollSource, field: Option<FieldPath> },
    /// binary arithmetic
    Arith { op: ArithOp, lhs: Box<Expr>, rhs: Box<Expr> },
    /// expr ?? default
    Coalesce { expr: Box<Expr>, default: Box<Expr> },
}

#[derive(Debug, Clone, PartialEq)]
enum AggOp { Sum, Max, Min, Count, Collect }

#[derive(Debug, Clone)]
enum CollSource {
    Children,
    Parent,
}

#[derive(Debug, Clone)]
enum ArithOp { Add, Sub, Mul, Div }

/// A dotted field path, e.g. `custom_fields.wcet` or `silLevel`.
#[derive(Debug, Clone)]
struct FieldPath(Vec<String>);

impl FieldPath {}

// ── Value type ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Value {
    Num(f64),
    Str(String),
    List(Vec<Value>),
    Null,
}

impl Value {
    fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Num(n) => Some(*n),
            Value::Str(s) => s.trim().parse().ok(),
            _ => None,
        }
    }

    fn to_yaml(&self) -> serde_yaml::Value {
        match self {
            Value::Num(n) => serde_yaml::Value::Number(serde_yaml::Number::from(*n)),
            Value::Str(s) => serde_yaml::Value::String(s.clone()),
            Value::List(items) => serde_yaml::Value::Sequence(
                items.iter().map(|v| v.to_yaml()).collect()
            ),
            Value::Null => serde_yaml::Value::Null,
        }
    }
}

// ── Parser ────────────────────────────────────────────────────────────────────

/// Minimal recursive-descent parser for derive expressions.
struct Parser<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self { Parser { src, pos: 0 } }

    fn remaining(&self) -> &str { &self.src[self.pos..] }

    fn skip_ws(&mut self) {
        while self.pos < self.src.len() && self.src.as_bytes()[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_ws();
        self.src.as_bytes().get(self.pos).copied()
    }

    fn consume(&mut self, n: usize) { self.pos += n; }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_coalesce()
    }

    fn parse_coalesce(&mut self) -> Result<Expr, String> {
        let lhs = self.parse_arith()?;
        self.skip_ws();
        if self.remaining().starts_with("??") {
            self.consume(2);
            let rhs = self.parse_arith()?;
            Ok(Expr::Coalesce { expr: Box::new(lhs), default: Box::new(rhs) })
        } else {
            Ok(lhs)
        }
    }

    fn parse_arith(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_primary()?;
        loop {
            self.skip_ws();
            let op = match self.peek() {
                Some(b'+') => { self.consume(1); ArithOp::Add }
                Some(b'-') => { self.consume(1); ArithOp::Sub }
                Some(b'*') => { self.consume(1); ArithOp::Mul }
                Some(b'/') => { self.consume(1); ArithOp::Div }
                _ => break,
            };
            let rhs = self.parse_primary()?;
            lhs = Expr::Arith { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }
        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        // Parenthesised expression
        if self.remaining().starts_with('(') {
            self.consume(1);
            let e = self.parse_expr()?;
            self.skip_ws();
            if !self.remaining().starts_with(')') { return Err("expected ')'".to_string()); }
            self.consume(1);
            return Ok(e);
        }
        // Numeric literal
        if self.peek().is_some_and(|b| b.is_ascii_digit() || b == b'.') {
            return self.parse_number();
        }
        // String literal
        if self.peek() == Some(b'"') {
            return self.parse_string();
        }
        // Identifier-based: aggregate, self.*, elements[...]
        self.parse_ident_expr()
    }

    fn parse_number(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        let start = self.pos;
        while self.pos < self.src.len() {
            let b = self.src.as_bytes()[self.pos];
            if b.is_ascii_digit() || b == b'.' || b == b'e' || b == b'E' || b == b'-' || b == b'+' {
                self.pos += 1;
            } else { break; }
        }
        let s = &self.src[start..self.pos];
        s.parse::<f64>().map(Expr::Num).map_err(|_| format!("invalid number '{}'", s))
    }

    fn parse_string(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if !self.remaining().starts_with('"') { return Err("expected '\"'".to_string()); }
        self.consume(1);
        let start = self.pos;
        while self.pos < self.src.len() && self.src.as_bytes()[self.pos] != b'"' {
            self.pos += 1;
        }
        let s = self.src[start..self.pos].to_string();
        if self.remaining().starts_with('"') { self.consume(1); }
        Ok(Expr::Str(s))
    }

    fn parse_ident(&mut self) -> String {
        self.skip_ws();
        let start = self.pos;
        while self.pos < self.src.len() {
            let b = self.src.as_bytes()[self.pos];
            if b.is_ascii_alphanumeric() || b == b'_' { self.pos += 1; } else { break; }
        }
        self.src[start..self.pos].to_string()
    }

    fn parse_ident_expr(&mut self) -> Result<Expr, String> {
        let ident = self.parse_ident();
        self.skip_ws();
        match ident.as_str() {
            "sum" | "max" | "min" | "count" | "collect" => {
                let op = match ident.as_str() {
                    "sum" => AggOp::Sum, "max" => AggOp::Max, "min" => AggOp::Min,
                    "count" => AggOp::Count, "collect" => AggOp::Collect,
                    _ => unreachable!(),
                };
                if !self.remaining().starts_with('(') {
                    return Err(format!("expected '(' after {}", ident));
                }
                self.consume(1);
                // Parse: source or source.field
                let (source, field) = self.parse_collection_expr()?;
                self.skip_ws();
                if !self.remaining().starts_with(')') {
                    return Err("expected ')' after aggregate argument".to_string());
                }
                self.consume(1);
                Ok(Expr::Aggregate { op, source, field })
            }
            "self" => {
                self.skip_ws();
                if !self.remaining().starts_with('.') {
                    return Err("expected '.' after 'self'".to_string());
                }
                self.consume(1);
                let path = self.parse_field_path();
                Ok(Expr::SelfField(path))
            }
            "elements" => {
                self.skip_ws();
                if !self.remaining().starts_with('[') {
                    return Err("expected '[' after 'elements'".to_string());
                }
                self.consume(1);
                let qname = self.parse_string().and_then(|e| match e {
                    Expr::Str(s) => Ok(s),
                    _ => Err("expected string".to_string()),
                })?;
                self.skip_ws();
                if !self.remaining().starts_with(']') {
                    return Err("expected ']'".to_string());
                }
                self.consume(1);
                self.skip_ws();
                if !self.remaining().starts_with('.') {
                    return Err("expected '.' after elements[\"...\"]".to_string());
                }
                self.consume(1);
                let path = self.parse_field_path();
                Ok(Expr::ElementField { qname, path })
            }
            _ if !ident.is_empty() => {
                // Bare identifier — treat as field path starting with the ident
                let mut parts = vec![ident];
                while self.remaining().starts_with('.') {
                    self.consume(1);
                    parts.push(self.parse_ident());
                }
                // Treat as self.field shorthand
                Ok(Expr::SelfField(FieldPath(parts)))
            }
            _ => Err(format!("unexpected character at: '{}'", &self.remaining()[..self.remaining().len().min(16)])),
        }
    }

    fn parse_field_path(&mut self) -> FieldPath {
        let mut parts = vec![self.parse_ident()];
        while self.remaining().starts_with('.') {
            // peek ahead — don't consume if followed by only whitespace (end of path)
            let saved = self.pos;
            self.consume(1);
            let seg = self.parse_ident();
            if seg.is_empty() { self.pos = saved; break; }
            parts.push(seg);
        }
        FieldPath(parts)
    }

    fn parse_collection_expr(&mut self) -> Result<(CollSource, Option<FieldPath>), String> {
        self.skip_ws();
        let src_name = self.parse_ident();
        let source = match src_name.as_str() {
            "children" => CollSource::Children,
            "parent"   => CollSource::Parent,
            other      => return Err(format!("unknown collection source '{}'", other)),
        };
        self.skip_ws();
        if self.remaining().starts_with('.') {
            self.consume(1);
            let field = self.parse_field_path();
            Ok((source, Some(field)))
        } else {
            Ok((source, None))
        }
    }
}

fn parse_formula(formula: &str) -> Result<Expr, String> {
    let mut p = Parser::new(formula);
    let e = p.parse_expr()?;
    p.skip_ws();
    if p.pos < p.src.len() {
        return Err(format!("unexpected trailing content: '{}'", &p.src[p.pos..]));
    }
    Ok(e)
}

// ── Field resolution ──────────────────────────────────────────────────────────

/// Read a field from an element by dotted path.
/// Supports: `<field>`, `custom_fields.<key>`, or any RawFrontmatter field via serde_yaml round-trip.
fn read_field(elem: &RawElement, path: &FieldPath) -> Value {
    let parts = &path.0;
    if parts.is_empty() { return Value::Null; }

    // Check computed/derived fields first
    if parts.len() == 1 {
        if let Some(v) = elem.derived.get(parts[0].as_str()) {
            return yaml_to_value(v);
        }
    }

    // custom_fields.<key>
    if parts.len() >= 2 && parts[0] == "custom_fields" {
        let key = parts[1..].join(".");
        if let Some(v) = elem.frontmatter.custom_fields.get(&key) {
            return yaml_to_value(v);
        }
        return Value::Null;
    }

    // Standard frontmatter fields via serialization round-trip
    if parts.len() == 1 {
        let v = serde_yaml::to_value(&elem.frontmatter).unwrap_or(serde_yaml::Value::Null);
        if let serde_yaml::Value::Mapping(m) = &v {
            // camelCase key lookup
            let key = &parts[0];
            // Try camelCase and snake_case
            for (k, val) in m {
                if let Some(ks) = k.as_str() {
                    if ks == key.as_str() { return yaml_to_value(val); }
                }
            }
        }
    }
    Value::Null
}

fn yaml_to_value(v: &serde_yaml::Value) -> Value {
    match v {
        serde_yaml::Value::Null => Value::Null,
        serde_yaml::Value::Bool(b) => Value::Num(if *b { 1.0 } else { 0.0 }),
        serde_yaml::Value::Number(n) => Value::Num(n.as_f64().unwrap_or(0.0)),
        serde_yaml::Value::String(s) => {
            if let Ok(n) = s.trim().parse::<f64>() { Value::Num(n) }
            else { Value::Str(s.clone()) }
        }
        serde_yaml::Value::Sequence(seq) => {
            Value::List(seq.iter().map(yaml_to_value).collect())
        }
        _ => Value::Null,
    }
}

// ── Collection resolution ─────────────────────────────────────────────────────

fn resolve_collection<'a>(source: &CollSource, current: &RawElement, all: &'a [RawElement]) -> Vec<&'a RawElement> {
    let prefix = &current.qualified_name;
    match source {
        CollSource::Children => {
            all.iter().filter(|e| {
                // A direct child: qname starts with prefix::, no further :: separator
                if let Some(rest) = e.qualified_name.strip_prefix(prefix) {
                    if let Some(seg) = rest.strip_prefix("::") {
                        !seg.contains("::")
                    } else { false }
                } else { false }
            }).collect()
        }
        CollSource::Parent => {
            // Parent: qualified name with the last segment removed
            let parent_qn = match prefix.rfind("::") {
                Some(idx) => &prefix[..idx],
                None => return vec![],
            };
            all.iter().filter(|e| e.qualified_name == parent_qn).collect()
        }
    }
}

// ── Evaluator ─────────────────────────────────────────────────────────────────

fn eval(
    expr: &Expr,
    current: &RawElement,
    all: &[RawElement],
    findings: &mut Vec<Finding>,
) -> Value {
    match expr {
        Expr::Num(n) => Value::Num(*n),
        Expr::Str(s) => Value::Str(s.clone()),

        Expr::SelfField(path) => read_field(current, path),

        Expr::ElementField { qname, path } => {
            match all.iter().find(|e| &e.qualified_name == qname) {
                None => {
                    findings.push(finding("E506", &current.file_path,
                        &format!("derive: element '{}' not found in model", qname)));
                    Value::Null
                }
                Some(target) => read_field(target, path),
            }
        }

        Expr::Aggregate { op, source, field } => {
            let members = resolve_collection(source, current, all);
            match op {
                AggOp::Count => return Value::Num(members.len() as f64),
                AggOp::Collect => {
                    let Some(fp) = field else { return Value::List(vec![]); };
                    let vals: Vec<Value> = members.iter()
                        .map(|m| read_field(m, fp))
                        .filter(|v| !matches!(v, Value::Null))
                        .collect();
                    return Value::List(vals);
                }
                _ => {}
            }
            let Some(fp) = field else { return Value::Null; };
            let nums: Vec<f64> = members.iter()
                .map(|m| read_field(m, fp))
                .filter_map(|v| v.as_f64())
                .collect();
            match op {
                AggOp::Sum => Value::Num(nums.iter().sum()),
                AggOp::Max => nums.iter().cloned().reduce(f64::max).map(Value::Num).unwrap_or(Value::Null),
                AggOp::Min => nums.iter().cloned().reduce(f64::min).map(Value::Num).unwrap_or(Value::Null),
                _ => Value::Null,
            }
        }

        Expr::Arith { op, lhs, rhs } => {
            let l = eval(lhs, current, all, findings);
            let r = eval(rhs, current, all, findings);
            match (l.as_f64(), r.as_f64()) {
                (Some(a), Some(b)) => {
                    let result = match op {
                        ArithOp::Add => a + b,
                        ArithOp::Sub => a - b,
                        ArithOp::Mul => a * b,
                        ArithOp::Div => if b == 0.0 { return Value::Null; } else { a / b },
                    };
                    Value::Num(result)
                }
                _ => Value::Null,
            }
        }

        Expr::Coalesce { expr, default } => {
            let v = eval(expr, current, all, findings);
            if matches!(v, Value::Null) { eval(default, current, all, findings) } else { v }
        }
    }
}

// ── Dependency graph (E504, REQ-TRS-DERIVE-004) ─────────────────────────────

/// The derived fields a formula reads, as `(element index, field name)` pairs
/// naming a key of some element's own `derive:` block (`declared`). References
/// resolve exactly as `eval` resolves them: `self.<f>`, `elements["Q"].<f>`
/// (first element with that qualified name), and `children`/`parent` aggregates
/// over `<f>`. Only single-segment paths can read a derived field.
fn expr_deps(
    expr: &Expr,
    idx: usize,
    all: &[RawElement],
    by_qname: &std::collections::HashMap<&str, usize>,
    declared: &std::collections::HashMap<usize, Vec<String>>,
    out: &mut Vec<(usize, String)>,
) {
    let mut add = |i: usize, path: &FieldPath| {
        if path.0.len() == 1 && declared.get(&i).is_some_and(|ks| ks.contains(&path.0[0])) {
            out.push((i, path.0[0].clone()));
        }
    };
    match expr {
        Expr::Num(_) | Expr::Str(_) => {}
        Expr::SelfField(path) => add(idx, path),
        Expr::ElementField { qname, path } => {
            if let Some(&t) = by_qname.get(qname.as_str()) {
                add(t, path);
            }
        }
        Expr::Aggregate { source, field, .. } => {
            if let Some(fp) = field {
                for m in resolve_collection(source, &all[idx], all) {
                    if let Some(&mi) = by_qname.get(m.qualified_name.as_str()) {
                        add(mi, fp);
                    }
                }
            }
        }
        Expr::Arith { lhs, rhs, .. } => {
            expr_deps(lhs, idx, all, by_qname, declared, out);
            expr_deps(rhs, idx, all, by_qname, declared, out);
        }
        Expr::Coalesce { expr, default } => {
            expr_deps(expr, idx, all, by_qname, declared, out);
            expr_deps(default, idx, all, by_qname, declared, out);
        }
    }
}

/// Tarjan's strongly-connected components over `adj` (node → successors).
/// A component is a cycle when it has more than one node or a self-edge.
fn strongly_connected(adj: &[Vec<usize>]) -> Vec<Vec<usize>> {
    struct St<'a> {
        adj: &'a [Vec<usize>],
        index: Vec<Option<usize>>,
        low: Vec<usize>,
        on_stack: Vec<bool>,
        stack: Vec<usize>,
        next: usize,
        out: Vec<Vec<usize>>,
    }
    fn visit(st: &mut St, v: usize) {
        st.index[v] = Some(st.next);
        st.low[v] = st.next;
        st.next += 1;
        st.stack.push(v);
        st.on_stack[v] = true;
        for k in 0..st.adj[v].len() {
            let w = st.adj[v][k];
            match st.index[w] {
                None => {
                    visit(st, w);
                    st.low[v] = st.low[v].min(st.low[w]);
                }
                Some(wi) if st.on_stack[w] => st.low[v] = st.low[v].min(wi),
                Some(_) => {}
            }
        }
        if Some(st.low[v]) == st.index[v] {
            let mut comp = Vec::new();
            while let Some(w) = st.stack.pop() {
                st.on_stack[w] = false;
                comp.push(w);
                if w == v {
                    break;
                }
            }
            st.out.push(comp);
        }
    }
    let n = adj.len();
    let mut st = St {
        adj,
        index: vec![None; n],
        low: vec![0; n],
        on_stack: vec![false; n],
        stack: Vec::new(),
        next: 0,
        out: Vec::new(),
    };
    for v in 0..n {
        if st.index[v].is_none() {
            visit(&mut st, v);
        }
    }
    st.out
}

/// One concrete cycle `start → … → start` inside the cyclic component `comp`
/// (shortest, by BFS over edges that stay in the component).
fn cycle_path(adj: &[Vec<usize>], comp: &[usize], start: usize) -> Vec<usize> {
    use std::collections::{HashMap, HashSet, VecDeque};
    if adj[start].contains(&start) {
        return vec![start, start];
    }
    let members: HashSet<usize> = comp.iter().copied().collect();
    let mut prev: HashMap<usize, usize> = HashMap::new();
    let mut queue: VecDeque<usize> = VecDeque::from([start]);
    while let Some(v) = queue.pop_front() {
        for &w in &adj[v] {
            if w == start {
                let mut rev = Vec::new();
                let mut cur = v;
                while cur != start {
                    rev.push(cur);
                    cur = prev[&cur];
                }
                let mut path = vec![start];
                path.extend(rev.into_iter().rev());
                path.push(start);
                return path;
            }
            if members.contains(&w) && !prev.contains_key(&w) {
                prev.insert(w, v);
                queue.push_back(w);
            }
        }
    }
    let mut path = comp.to_vec();
    path.push(start);
    path
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Run the derive pass over all elements. Populates `RawElement.derived` and
/// `RawElement.derive_findings` for each element that has a `derive:` block.
///
/// Every formula is parsed first (`E505` on a parse failure, a block that is not
/// a mapping, or a non-string formula), then a dependency graph over derived
/// fields is built (REQ-TRS-DERIVE-004). Fields on a cycle are reported as
/// `E504` — once per participating element, naming the cycle — and are not
/// evaluated; every other field is evaluated in dependency order, so it sees the
/// computed value of each derived field it reads whatever the element order.
pub fn derive_pass(elements: &mut [RawElement]) {
    use std::collections::HashMap;

    struct Node {
        idx: usize,
        field: String,
        expr: Expr,
    }

    // 1. Parse every block (E505 for a malformed block, formula or parse).
    let mut nodes: Vec<Node> = Vec::new();
    let mut declared: HashMap<usize, Vec<String>> = HashMap::new();
    for (idx, elem) in elements.iter_mut().enumerate() {
        let Some(block) = elem.frontmatter.derive.clone() else { continue };
        let file = elem.file_path.clone();
        let serde_yaml::Value::Mapping(mapping) = block else {
            elem.derive_findings.push(finding(
                "E505",
                &file,
                "derive: block must be a mapping of field name → formula string",
            ));
            continue;
        };
        for (key_val, formula_val) in &mapping {
            let Some(field_name) = key_val.as_str() else { continue };
            // Every declared key is a derived field (even one that fails to
            // parse), so a reference to it is a derived-field dependency.
            declared.entry(idx).or_default().push(field_name.to_string());
            let Some(formula_str) = formula_val.as_str() else {
                elem.derive_findings.push(finding(
                    "E505",
                    &file,
                    &format!("derive formula for field '{}' must be a string", field_name),
                ));
                continue;
            };
            match parse_formula(formula_str) {
                Ok(expr) => nodes.push(Node { idx, field: field_name.to_string(), expr }),
                Err(e) => elem.derive_findings.push(finding(
                    "E505",
                    &file,
                    &format!("derive formula parse error for field '{}': {}", field_name, e),
                )),
            }
        }
    }
    if nodes.is_empty() {
        return;
    }

    // 2. Dependency graph: edge node → each derived field its formula reads.
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    {
        let mut by_qname: HashMap<&str, usize> = HashMap::new();
        for (i, e) in elements.iter().enumerate() {
            by_qname.entry(e.qualified_name.as_str()).or_insert(i);
        }
        let node_of: HashMap<(usize, &str), usize> = nodes
            .iter()
            .enumerate()
            .map(|(n, node)| ((node.idx, node.field.as_str()), n))
            .collect();
        for (n, node) in nodes.iter().enumerate() {
            let mut deps = Vec::new();
            expr_deps(&node.expr, node.idx, elements, &by_qname, &declared, &mut deps);
            for (di, df) in deps {
                if let Some(&d) = node_of.get(&(di, df.as_str())) {
                    if !adj[n].contains(&d) {
                        adj[n].push(d);
                    }
                }
            }
        }
    }

    // 3. Cycles (E504): report once per participating element, and skip.
    let label = |n: usize| format!("{}.{}", elements[nodes[n].idx].qualified_name, nodes[n].field);
    let mut cyclic = vec![false; nodes.len()];
    let mut cycle_findings: Vec<(usize, Finding)> = Vec::new();
    for comp in strongly_connected(&adj) {
        let is_cycle = comp.len() > 1 || adj[comp[0]].contains(&comp[0]);
        if !is_cycle {
            continue;
        }
        let start = *comp.iter().min().unwrap_or(&comp[0]);
        let chain: Vec<String> = cycle_path(&adj, &comp, start).into_iter().map(&label).collect();
        let mut files: Vec<usize> = comp.iter().map(|&n| nodes[n].idx).collect();
        files.sort_unstable();
        files.dedup();
        for &n in &comp {
            cyclic[n] = true;
        }
        let msg = format!(
            "derive: cyclic dependency {} — the fields in the cycle are not evaluated",
            chain.join(" → ")
        );
        for idx in files {
            cycle_findings.push((idx, finding("E504", &elements[idx].file_path, &msg)));
        }
    }
    for (idx, f) in cycle_findings {
        elements[idx].derive_findings.push(f);
    }

    // 4. Evaluate the acyclic fields in dependency order: an iterative
    //    post-order DFS seeded in file-walk, then block, order.
    let mut seen = vec![false; nodes.len()];
    let mut order: Vec<usize> = Vec::with_capacity(nodes.len());
    for root in 0..nodes.len() {
        if seen[root] || cyclic[root] {
            continue;
        }
        seen[root] = true;
        let mut stack: Vec<(usize, usize)> = vec![(root, 0)];
        while let Some(top) = stack.last_mut() {
            let (v, next) = *top;
            if let Some(&w) = adj[v].get(next) {
                top.1 += 1;
                if !seen[w] && !cyclic[w] {
                    seen[w] = true;
                    stack.push((w, 0));
                }
            } else {
                order.push(v);
                stack.pop();
            }
        }
    }
    for n in order {
        let idx = nodes[n].idx;
        let mut elem_findings: Vec<Finding> = Vec::new();
        let value = eval(&nodes[n].expr, &elements[idx], elements, &mut elem_findings);
        elements[idx].derived.insert(nodes[n].field.clone(), value.to_yaml());
        elements[idx].derive_findings.append(&mut elem_findings);
    }
}

#[cfg(test)]
mod tests {
    //! GH #141 — dependency-ordered evaluation and E504 cycle detection.
    use super::derive_pass;
    use crate::element::{RawElement, RawFrontmatter};

    fn elem(qn: &str, derive: &str) -> RawElement {
        let frontmatter = RawFrontmatter {
            derive: Some(serde_yaml::from_str(derive).expect("derive yaml")),
            ..Default::default()
        };
        RawElement {
            qualified_name: qn.to_string(),
            file_path: format!("{}.md", qn.replace("::", "/")),
            frontmatter,
            doc: String::new(),
            parse_issue: None,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn codes(e: &RawElement, code: &str) -> Vec<String> {
        e.derive_findings.iter().filter(|f| f.0 == code).map(|f| f.2.clone()).collect()
    }

    fn num(e: &RawElement, field: &str) -> Option<f64> {
        e.derived.get(field).and_then(|v| v.as_f64())
    }

    #[test]
    fn loop_within_one_block_is_e504_and_skipped() {
        let mut els = vec![elem("S::A", "a: self.b + 1\nb: self.a * 2\nc: '5'")];
        derive_pass(&mut els);
        let e504 = codes(&els[0], "E504");
        assert_eq!(e504.len(), 1, "one E504 per element per cycle: {e504:?}");
        assert!(e504[0].contains("S::A.a") && e504[0].contains("S::A.b"), "{e504:?}");
        assert!(!els[0].derived.contains_key("a") && !els[0].derived.contains_key("b"));
        assert_eq!(num(&els[0], "c"), Some(5.0), "fields outside the cycle still evaluate");
    }

    #[test]
    fn cross_element_chain_evaluates_regardless_of_walk_order() {
        let mut els = vec![
            elem("S::First", "x: 'elements[\"S::Second\"].y + 1'"),
            elem("S::Second", "y: 'elements[\"S::Third\"].z * 10'"),
            elem("S::Third", "z: '4'"),
        ];
        derive_pass(&mut els);
        assert!(els.iter().all(|e| codes(e, "E504").is_empty()));
        assert_eq!(num(&els[2], "z"), Some(4.0));
        assert_eq!(num(&els[1], "y"), Some(40.0));
        assert_eq!(num(&els[0], "x"), Some(41.0));
    }

    #[test]
    fn aggregate_over_children_participates_in_cycles() {
        // The parent sums its children's `v`; the child reads the parent's total.
        let mut els = vec![
            elem("S::P", "total: sum(children.v)"),
            elem("S::P::K", "v: sum(parent.total)"),
        ];
        derive_pass(&mut els);
        assert_eq!(codes(&els[0], "E504").len(), 1);
        assert_eq!(codes(&els[1], "E504").len(), 1);
    }

    #[test]
    fn malformed_blocks_are_e505() {
        let mut els = vec![elem("S::A", "5"), elem("S::B", "k: 3")];
        derive_pass(&mut els);
        assert_eq!(codes(&els[0], "E505").len(), 1);
        let b = codes(&els[1], "E505");
        assert!(b.len() == 1 && b[0].contains("'k'"), "{b:?}");
    }
}
