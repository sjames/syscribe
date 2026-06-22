//! Declarative derive: block evaluator (REQ-TRS-DERIVE-001..005, issue #60).
//!
//! Each element may declare a `derive:` mapping of fieldName → formula. Formulas
//! are evaluated top-to-bottom within an element; derived fields from earlier
//! entries are visible to later ones via `self.<field>`. Cross-element references
//! (`elements["Qname"]`) are resolved against the full element set.
//!
//! The evaluation pipeline:
//!   Walker → derive_pass(elements) → Validator
//!
//! The pass populates `RawElement.derived` for each element; the validator and
//! query layer read derived fields from there.

use crate::element::RawElement;

type Finding = (String, String, String); // (code, file, message)

fn finding(code: &str, file: &str, message: &str) -> Finding {
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
                    findings.push(finding("E502", &current.file_path,
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

// ── Public API ────────────────────────────────────────────────────────────────

/// Run the derive pass over all elements. Populates `RawElement.derived` and
/// `RawElement.derive_findings` for each element that has a `derive:` block.
pub fn derive_pass(elements: &mut [RawElement]) {
    let mut findings: Vec<Finding> = Vec::new();

    // Process elements sequentially — simple strategy: iterate up to 3 times
    // so that cross-element dependencies resolve (children computed before parents
    // need them). A full topo-sort is left as a future enhancement once cycle
    // detection (E500) is added. For now: single forward pass with self-chaining.
    for idx in 0..elements.len() {
        let derive_block = {
            let fm = &elements[idx].frontmatter;
            // The `derive:` key lands in `extra` since it's not a declared field
            fm.extra.get("derive").cloned()
        };
        let Some(derive_val) = derive_block else { continue; };
        let serde_yaml::Value::Mapping(mapping) = derive_val else { continue; };

        let mut elem_findings: Vec<Finding> = Vec::new();

        for (key_val, formula_val) in &mapping {
            let Some(field_name) = key_val.as_str() else { continue; };
            let Some(formula_str) = formula_val.as_str() else { continue; };

            let expr = match parse_formula(formula_str) {
                Ok(e) => e,
                Err(e) => {
                    elem_findings.push(finding("E501", &elements[idx].file_path,
                        &format!("derive formula parse error for field '{}': {}", field_name, e)));
                    continue;
                }
            };

            // Evaluate with current snapshot of derived fields already computed.
            let value = {
                let elem_snap: &RawElement = &elements[idx];
                let all: &[RawElement] = elements;
                eval(&expr, elem_snap, all, &mut elem_findings)
            };

            elements[idx].derived.insert(field_name.to_string(), value.to_yaml());
        }

        elements[idx].derive_findings.append(&mut elem_findings);
        let _ = &mut findings; // suppress unused warning
    }
}
