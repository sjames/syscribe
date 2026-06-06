//! Product-line variability primitives (§9).
//!
//! This module is purely about the *variability dimension*: parsing and
//! evaluating `appliesWhen:` conditions over `FeatureDef` qualified names, and
//! deciding whether the dimension is active at all (the opt-in principle —
//! [`REQ-TRS-VAR-001`]).
//!
//! It is deliberately free of any I/O or model-graph knowledge: callers pass in
//! a closure that answers "is this feature selected?". This keeps the grammar
//! and truth-table semantics unit-testable in isolation.

use crate::element::{ElementType, RawElement};

/// A boolean expression over `FeatureDef` qualified names.
///
/// Grammar (precedence `not` > `and` > `or`):
///
/// ```text
/// or   := and ( "or"  and )*
/// and  := not ( "and" not )*
/// not  := "not" not | atom
/// atom := "(" or ")" | QNAME
/// ```
///
/// A bare qualified name parses to [`FeatureExpr::Feat`] — the back-compatible
/// single-term case. A YAML *list* of names is handled by the caller as the
/// AND of its terms (legacy semantics).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureExpr {
    Feat(String),
    Not(Box<FeatureExpr>),
    And(Box<FeatureExpr>, Box<FeatureExpr>),
    Or(Box<FeatureExpr>, Box<FeatureExpr>),
}

impl FeatureExpr {
    /// Evaluate the expression. `selected(qname)` returns whether that feature
    /// is selected in the configuration under evaluation.
    pub fn eval(&self, selected: &impl Fn(&str) -> bool) -> bool {
        match self {
            FeatureExpr::Feat(q) => selected(q),
            FeatureExpr::Not(e) => !e.eval(selected),
            FeatureExpr::And(a, b) => a.eval(selected) && b.eval(selected),
            FeatureExpr::Or(a, b) => a.eval(selected) || b.eval(selected),
        }
    }

    /// Every `FeatureDef` qualified name referenced by the expression, in source
    /// order. Used for cross-reference resolution (E209).
    pub fn operands(&self) -> Vec<String> {
        let mut out = Vec::new();
        self.collect(&mut out);
        out
    }

    fn collect(&self, out: &mut Vec<String>) {
        match self {
            FeatureExpr::Feat(q) => out.push(q.clone()),
            FeatureExpr::Not(e) => e.collect(out),
            FeatureExpr::And(a, b) | FeatureExpr::Or(a, b) => {
                a.collect(out);
                b.collect(out);
            }
        }
    }
}

// ── Tokenizer ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    And,
    Or,
    Not,
    LParen,
    RParen,
    Ident(String),
}

fn tokenize(s: &str) -> Result<Vec<Tok>, String> {
    let mut toks = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            '(' => {
                chars.next();
                toks.push(Tok::LParen);
            }
            ')' => {
                chars.next();
                toks.push(Tok::RParen);
            }
            // QName / keyword run: alphanumerics, '_', ':' (for '::'), '.'.
            c if c.is_alphanumeric() || c == '_' || c == ':' || c == '.' => {
                let mut word = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' || c == ':' || c == '.' {
                        word.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match word.as_str() {
                    "and" => toks.push(Tok::And),
                    "or" => toks.push(Tok::Or),
                    "not" => toks.push(Tok::Not),
                    _ => toks.push(Tok::Ident(word)),
                }
            }
            other => return Err(format!("unexpected character '{}' in appliesWhen", other)),
        }
    }
    Ok(toks)
}

// ── Recursive-descent parser ───────────────────────────────────────────────────

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn parse_or(&mut self) -> Result<FeatureExpr, String> {
        let mut lhs = self.parse_and()?;
        while matches!(self.peek(), Some(Tok::Or)) {
            self.next();
            let rhs = self.parse_and()?;
            lhs = FeatureExpr::Or(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<FeatureExpr, String> {
        let mut lhs = self.parse_not()?;
        while matches!(self.peek(), Some(Tok::And)) {
            self.next();
            let rhs = self.parse_not()?;
            lhs = FeatureExpr::And(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_not(&mut self) -> Result<FeatureExpr, String> {
        if matches!(self.peek(), Some(Tok::Not)) {
            self.next();
            let inner = self.parse_not()?;
            Ok(FeatureExpr::Not(Box::new(inner)))
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Result<FeatureExpr, String> {
        match self.next() {
            Some(Tok::LParen) => {
                let inner = self.parse_or()?;
                match self.next() {
                    Some(Tok::RParen) => Ok(inner),
                    _ => Err("missing closing ')' in appliesWhen".into()),
                }
            }
            Some(Tok::Ident(q)) => Ok(FeatureExpr::Feat(q)),
            Some(t) => Err(format!("unexpected token {:?} in appliesWhen", t)),
            None => Err("unexpected end of appliesWhen expression".into()),
        }
    }
}

/// Parse a single `appliesWhen:` string into a [`FeatureExpr`].
pub fn parse(s: &str) -> Result<FeatureExpr, String> {
    let toks = tokenize(s)?;
    if toks.is_empty() {
        return Err("empty appliesWhen expression".into());
    }
    let mut p = Parser { toks, pos: 0 };
    let expr = p.parse_or()?;
    if p.pos != p.toks.len() {
        return Err("trailing tokens in appliesWhen expression".into());
    }
    Ok(expr)
}

/// Interpret an `appliesWhen:` YAML value.
///
/// * absent / null  → `Ok(None)` (element is unconditional / always active)
/// * string         → parsed boolean expression
/// * list of strings → AND of the listed bare features (legacy semantics)
pub fn applies_when_expr(v: &serde_yaml::Value) -> Result<Option<FeatureExpr>, String> {
    match v {
        serde_yaml::Value::Null => Ok(None),
        serde_yaml::Value::String(s) => parse(s).map(Some),
        serde_yaml::Value::Sequence(seq) => {
            let mut acc: Option<FeatureExpr> = None;
            for item in seq {
                let term = match item {
                    serde_yaml::Value::String(s) => parse(s)?,
                    other => {
                        return Err(format!("expected string in appliesWhen list, got {:?}", other))
                    }
                };
                acc = Some(match acc {
                    None => term,
                    Some(prev) => FeatureExpr::And(Box::new(prev), Box::new(term)),
                });
            }
            Ok(acc)
        }
        other => Err(format!("appliesWhen must be a string or list, got {:?}", other)),
    }
}

// ── Activation predicate (opt-in principle, REQ-TRS-VAR-001) ────────────────────

/// The variability dimension is *active* iff at least one `FeatureDef` exists
/// **and** at least one element links to it — a `Configuration`, or any element
/// carrying `appliesWhen:`. Otherwise the dimension is dormant and all
/// variant-only behaviour falls back to the flat view.
pub fn is_active(elements: &[RawElement]) -> bool {
    let mut has_feature_def = false;
    let mut has_link = false;
    for e in elements {
        match e.frontmatter.element_type {
            Some(ElementType::FeatureDef) => has_feature_def = true,
            Some(ElementType::Configuration) => has_link = true,
            _ => {}
        }
        if e.frontmatter.applies_when.is_some() {
            has_link = true;
        }
        if has_feature_def && has_link {
            return true;
        }
    }
    has_feature_def && has_link
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(expr: &str, on: &[&str]) -> bool {
        let e = parse(expr).unwrap();
        e.eval(&|q: &str| on.contains(&q))
    }

    #[test]
    fn bare_qname() {
        assert!(ev("F::A", &["F::A"]));
        assert!(!ev("F::A", &["F::B"]));
    }

    #[test]
    fn and_or_not_truth_table() {
        // AND
        assert!(ev("F::A and F::B", &["F::A", "F::B"]));
        assert!(!ev("F::A and F::B", &["F::A"]));
        // OR
        assert!(ev("F::A or F::B", &["F::B"]));
        assert!(!ev("F::A or F::B", &[]));
        // NOT
        assert!(ev("not F::A", &[]));
        assert!(!ev("not F::A", &["F::A"]));
    }

    #[test]
    fn precedence_and_parens() {
        // not binds tighter than and; and tighter than or
        assert!(ev("F::A or F::B and F::C", &["F::A"]));
        assert!(!ev("(F::A or F::B) and not F::A", &["F::A"]));
        assert!(ev("(F::A or F::B) and not F::A", &["F::B"]));
    }

    #[test]
    fn operand_collection() {
        let e = parse("(F::A or F::B) and not F::C").unwrap();
        assert_eq!(e.operands(), vec!["F::A", "F::B", "F::C"]);
    }

    #[test]
    fn errors() {
        assert!(parse("F::A and").is_err());
        assert!(parse("(F::A").is_err());
        assert!(parse("and F::A").is_err());
        assert!(parse("").is_err());
    }

    #[test]
    fn list_is_and() {
        let v: serde_yaml::Value = serde_yaml::from_str("[F::A, F::B]").unwrap();
        let e = applies_when_expr(&v).unwrap().unwrap();
        assert!(e.eval(&|q: &str| ["F::A", "F::B"].contains(&q)));
        assert!(!e.eval(&|q: &str| ["F::A"].contains(&q)));
    }
}
