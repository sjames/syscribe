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
use std::collections::HashMap;

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

    /// Rewrite every operand through `canon` so the expression is expressed in a
    /// single canonical key space (a `FeatureDef`'s qualified name). An operand
    /// keyed by a `FEAT-*` stable id is mapped to the owning FeatureDef's qname;
    /// any operand `canon` does not know about is left unchanged. This lets an
    /// `appliesWhen: FEAT-ABS-001` gate identically to `appliesWhen: F::Anti_Lock`
    /// (REQ-TRS-ID-006).
    pub fn canonicalize(&self, canon: &impl Fn(&str) -> String) -> FeatureExpr {
        match self {
            FeatureExpr::Feat(q) => FeatureExpr::Feat(canon(q)),
            FeatureExpr::Not(e) => FeatureExpr::Not(Box::new(e.canonicalize(canon))),
            FeatureExpr::And(a, b) => {
                FeatureExpr::And(Box::new(a.canonicalize(canon)), Box::new(b.canonicalize(canon)))
            }
            FeatureExpr::Or(a, b) => {
                FeatureExpr::Or(Box::new(a.canonicalize(canon)), Box::new(b.canonicalize(canon)))
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
            //
            // A '-' is normally illegal (feature *names* are SysMLv2 basic names —
            // GH #42). The single exception (REQ-TRS-ID-006) is a stable-id-shaped
            // token such as `FEAT-ABS-001`: when we hit a '-' mid-token we only keep
            // consuming if the accumulated run *with* the hyphen run still parses as
            // a stable id. A hyphen that is part of a hyphenated *name*
            // (e.g. `Features::Anti-Lock`) does not satisfy the stable-id grammar and
            // therefore falls through to the E209 error path below.
            c if c.is_alphanumeric() || c == '_' || c == ':' || c == '.' => {
                let mut word = String::new();
                loop {
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '_' || c == ':' || c == '.' {
                            word.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    // Hyphen continuation: only valid as part of a stable-id token.
                    if chars.peek() == Some(&'-') {
                        // Greedily collect the candidate hyphenated continuation.
                        let mut candidate = word.clone();
                        let mut lookahead = chars.clone();
                        while let Some(&c) = lookahead.peek() {
                            if c == '-' || c.is_alphanumeric() {
                                candidate.push(c);
                                lookahead.next();
                            } else {
                                break;
                            }
                        }
                        if crate::resolver::is_stable_id(&candidate) {
                            // Accept the whole stable-id token.
                            word = candidate;
                            chars = lookahead;
                            continue;
                        }
                        return Err(
                            "unexpected character '-' in appliesWhen — feature names must be SysMLv2 \
                             basic names (letters/digits/_); only a stable-id-shaped reference \
                             (e.g. FEAT-ABS-001) may contain hyphens (W042)"
                                .to_string(),
                        );
                    }
                    break;
                }
                match word.as_str() {
                    "and" => toks.push(Tok::And),
                    "or" => toks.push(Tok::Or),
                    "not" => toks.push(Tok::Not),
                    _ => toks.push(Tok::Ident(word)),
                }
            }
            '-' => {
                return Err(
                    "unexpected character '-' in appliesWhen — feature names must be SysMLv2 basic \
                     names (letters/digits/_); rename hyphenated features using '_' or CamelCase (W042)"
                        .to_string(),
                )
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

// ── Feature-reference canonicalization (REQ-TRS-ID-006) ─────────────────────────

/// Map of `FeatureDef` stable `id` (FEAT-*) → its qualified name, for every
/// `FeatureDef` that declares one. Used to normalize feature references — in
/// `appliesWhen` and in a `Configuration`'s `features:` keys — so an id alias and
/// the qname resolve to the *same* canonical key.
pub fn feature_id_to_qname(elements: &[RawElement]) -> HashMap<String, String> {
    let mut m = HashMap::new();
    for e in elements {
        if matches!(e.frontmatter.element_type, Some(ElementType::FeatureDef)) {
            if let Some(id) = e.frontmatter.id.as_deref() {
                if crate::resolver::is_feat_id(id) {
                    m.insert(id.to_string(), e.qualified_name.clone());
                }
            }
        }
    }
    m
}

/// Canonicalize a single feature reference: a `FEAT-*` id maps to the owning
/// FeatureDef's qualified name; anything else is returned unchanged.
pub fn canon_feature_ref(r: &str, alias: &HashMap<String, String>) -> String {
    alias.get(r).cloned().unwrap_or_else(|| r.to_string())
}

/// A `Configuration`'s feature selection with every id-keyed entry rewritten to
/// the FeatureDef's qualified name (canonical key space). When a config keys both
/// an id and its qname (unusual), the qname-keyed entry wins.
pub fn canon_selection(
    sel: &std::collections::BTreeMap<String, bool>,
    alias: &HashMap<String, String>,
) -> std::collections::BTreeMap<String, bool> {
    let mut out = std::collections::BTreeMap::new();
    // id-keyed entries first, so an explicit qname entry overrides them.
    for (k, v) in sel {
        if let Some(q) = alias.get(k) {
            out.insert(q.clone(), *v);
        }
    }
    for (k, v) in sel {
        if !alias.contains_key(k) {
            out.insert(k.clone(), *v);
        }
    }
    out
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

/// Map of package qualified-name → its declared `appliesWhen:` value, for every
/// `Package`/`LibraryPackage`/`Namespace` element that declares one. Used to
/// resolve an element's *effective* condition (transitive package `appliesWhen`,
/// [`REQ-TRS-VAR-006`]).
pub fn package_conditions(elements: &[RawElement]) -> HashMap<String, serde_yaml::Value> {
    let mut m = HashMap::new();
    for e in elements {
        if matches!(
            e.frontmatter.element_type,
            Some(ElementType::Package) | Some(ElementType::LibraryPackage) | Some(ElementType::Namespace)
        ) {
            if let Some(aw) = &e.frontmatter.applies_when {
                m.insert(e.qualified_name.clone(), aw.clone());
            }
        }
    }
    m
}

/// The nearest ancestor *package* of `elem` (a proper qname prefix) that declares
/// `appliesWhen:`, if any. Excludes the element itself.
pub fn ancestor_package_with_aw(
    elem: &RawElement,
    pkg: &HashMap<String, serde_yaml::Value>,
) -> Option<String> {
    if pkg.is_empty() {
        return None;
    }
    let segs: Vec<&str> = elem.qualified_name.split("::").filter(|s| !s.is_empty()).collect();
    // proper prefixes, longest first
    let mut i = segs.len().saturating_sub(1);
    while i >= 1 {
        let prefix = segs[..i].join("::");
        if pkg.contains_key(&prefix) {
            return Some(prefix);
        }
        i -= 1;
    }
    None
}

/// An element's *effective* `appliesWhen:` — its own if declared, otherwise the
/// nearest ancestor package's (transitive package conditioning). Returns the
/// value and the source: `None` source means the element's own declaration,
/// `Some(pkg_qname)` means it was inherited from that package.
pub fn effective_applies_when(
    elem: &RawElement,
    pkg: &HashMap<String, serde_yaml::Value>,
) -> Option<(serde_yaml::Value, Option<String>)> {
    if let Some(aw) = &elem.frontmatter.applies_when {
        return Some((aw.clone(), None));
    }
    ancestor_package_with_aw(elem, pkg).map(|p| {
        let v = pkg.get(&p).cloned().unwrap();
        (v, Some(p))
    })
}

/// The effective `appliesWhen:` parsed to a [`FeatureExpr`] (own or inherited).
pub fn effective_expr(
    elem: &RawElement,
    pkg: &HashMap<String, serde_yaml::Value>,
) -> Option<FeatureExpr> {
    effective_applies_when(elem, pkg).and_then(|(v, _)| applies_when_expr(&v).ok().flatten())
}

/// As [`effective_expr`], but with every operand canonicalized through the feature
/// id→qname `alias` map so a `FEAT-*` reference gates identically to the qname
/// (REQ-TRS-ID-006). Pass [`feature_id_to_qname`] for `alias`.
pub fn effective_expr_canon(
    elem: &RawElement,
    pkg: &HashMap<String, serde_yaml::Value>,
    alias: &HashMap<String, String>,
) -> Option<FeatureExpr> {
    effective_expr(elem, pkg).map(|e| e.canonicalize(&|q: &str| canon_feature_ref(q, alias)))
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

    // ── exhaustive boolean-logic oracle ──────────────────────────────────────
    // Render an AST with explicit parens (unambiguous), parse it back, and assert
    // the parsed expression evaluates identically to the original AST across ALL
    // variable assignments. This rigorously exercises tokenizer + parser + eval +
    // operands for arbitrary nested structures, independent of precedence.
    fn render(e: &FeatureExpr) -> String {
        match e {
            FeatureExpr::Feat(q) => q.clone(),
            FeatureExpr::Not(a) => format!("(not {})", render(a)),
            FeatureExpr::And(a, b) => format!("({} and {})", render(a), render(b)),
            FeatureExpr::Or(a, b) => format!("({} or {})", render(a), render(b)),
        }
    }

    fn xorshift(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    fn gen(state: &mut u64, depth: u32, vars: &[&str]) -> FeatureExpr {
        if depth == 0 || xorshift(state) % 3 == 0 {
            let v = vars[(xorshift(state) as usize) % vars.len()];
            return FeatureExpr::Feat(v.to_string());
        }
        match xorshift(state) % 3 {
            0 => FeatureExpr::Not(Box::new(gen(state, depth - 1, vars))),
            1 => FeatureExpr::And(
                Box::new(gen(state, depth - 1, vars)),
                Box::new(gen(state, depth - 1, vars)),
            ),
            _ => FeatureExpr::Or(
                Box::new(gen(state, depth - 1, vars)),
                Box::new(gen(state, depth - 1, vars)),
            ),
        }
    }

    fn assign<'a>(vars: &'a [&'a str], mask: u32) -> impl Fn(&str) -> bool + 'a {
        move |q: &str| {
            vars.iter()
                .position(|v| *v == q)
                .map(|i| (mask >> i) & 1 == 1)
                .unwrap_or(false)
        }
    }

    #[test]
    fn render_parse_eval_oracle() {
        let vars = ["F::A", "F::B", "F::C"];
        let mut state = 0x9E37_79B9_7F4A_7C15u64;
        for _ in 0..3000 {
            let ast = gen(&mut state, 4, &vars);
            let s = render(&ast);
            let parsed = parse(&s).unwrap_or_else(|e| panic!("parse failed for `{s}`: {e}"));
            for mask in 0u32..(1 << vars.len()) {
                let on = assign(&vars, mask);
                assert_eq!(
                    parsed.eval(&on),
                    ast.eval(&on),
                    "eval mismatch for `{s}` @ mask {mask:03b}"
                );
            }
            let (mut a, mut p) = (ast.operands(), parsed.operands());
            a.sort();
            a.dedup();
            p.sort();
            p.dedup();
            assert_eq!(a, p, "operands mismatch for `{s}`");
        }
    }

    #[test]
    fn precedence_matches_explicit_parens() {
        // not > and > or. Each bare form must be equivalent to its fully-parenthesized
        // reference across every assignment.
        let vars = ["F::A", "F::B", "F::C"];
        let cases = [
            ("F::A or F::B and F::C", "F::A or (F::B and F::C)"),
            ("F::A and F::B or F::C", "(F::A and F::B) or F::C"),
            ("not F::A and F::B", "(not F::A) and F::B"),
            ("not F::A or F::B", "(not F::A) or F::B"),
            ("not not F::A", "F::A"),
            ("F::A and F::B and F::C", "(F::A and F::B) and F::C"),
            ("F::A or F::B or F::C", "(F::A or F::B) or F::C"),
        ];
        for (bare, paren) in cases {
            let b = parse(bare).unwrap();
            let p = parse(paren).unwrap();
            for mask in 0u32..(1 << vars.len()) {
                let on = assign(&vars, mask);
                assert_eq!(b.eval(&on), p.eval(&on), "`{bare}` vs `{paren}` @ {mask:03b}");
            }
        }
    }

    #[test]
    fn operator_substring_identifiers() {
        // and/or/not as substrings of a qname must NOT be tokenized as operators.
        let e = parse("F::Android and F::Normal or not F::Sensor").unwrap();
        assert_eq!(e.operands(), vec!["F::Android", "F::Normal", "F::Sensor"]);
        assert_eq!(parse("F::Sandbox").unwrap(), FeatureExpr::Feat("F::Sandbox".into()));
        // operators require a separator: "notF::A" is a single identifier.
        assert_eq!(parse("notF::A").unwrap(), FeatureExpr::Feat("notF::A".into()));
        assert_eq!(parse("F::orange").unwrap(), FeatureExpr::Feat("F::orange".into()));
    }

    #[test]
    fn stable_id_token_accepted_hyphen_name_rejected() {
        // A stable-id-shaped token is one atom even though it contains hyphens.
        let e = parse("FEAT-ABS-001").unwrap();
        assert_eq!(e, FeatureExpr::Feat("FEAT-ABS-001".into()));
        // …and composes in the boolean grammar.
        let e = parse("FEAT-ABS-001 and not FEAT-ESC-002").unwrap();
        assert_eq!(e.operands(), vec!["FEAT-ABS-001", "FEAT-ESC-002"]);
        // A hyphenated NAME is NOT a stable id → still rejected (GH #42 / W042).
        assert!(parse("Features::Anti-Lock").is_err());
        assert!(parse("F::A and Features::Anti-Lock").is_err());
        // A bare leading hyphen is still rejected.
        assert!(parse("-F::A").is_err());
    }

    #[test]
    fn canonicalize_maps_id_operands_to_qname() {
        let mut alias = std::collections::HashMap::new();
        alias.insert("FEAT-ABS-001".to_string(), "Features::Anti_Lock".to_string());
        let e = parse("FEAT-ABS-001 and not F::X").unwrap();
        let c = e.canonicalize(&|q: &str| canon_feature_ref(q, &alias));
        assert_eq!(c.operands(), vec!["Features::Anti_Lock", "F::X"]);
    }

    #[test]
    fn whitespace_nesting_and_double_negation() {
        assert!(parse("  F::A   and(  F::B )")
            .unwrap()
            .eval(&|q: &str| ["F::A", "F::B"].contains(&q)));
        assert!(parse("(((F::A)))").unwrap().eval(&|q: &str| q == "F::A"));
        // double negation is identity; triple negation is a single negation.
        assert!(parse("not not F::A").unwrap().eval(&|q: &str| q == "F::A"));
        assert!(parse("not not not F::A").unwrap().eval(&|_q: &str| false));
    }
}
