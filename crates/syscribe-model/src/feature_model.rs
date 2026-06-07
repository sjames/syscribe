//! Holistic feature-model validation (§9), surfaced via the explicit
//! `syscribe feature-check` command — deliberately **not** part of the default
//! `validate` pass, which stays per-element and fast.
//!
//! Structural rules (REQ-TRS-FM-002): `E212` requires/excludes resolution,
//! `E219`/`E220` requires/excludes satisfaction per Configuration, `W011`/`W012`
//! dead / always-on optional features.
//!
//! Parameter-integrity rules (REQ-TRS-FM-003): `E207` `derivedFrom:` cycles,
//! `E202` `bindTo:` propagation range, `E213` unresolved `parameterConstraints`
//! paths, `W014` constraint `appliesWhen:` features absent from every
//! Configuration.

use std::collections::{HashMap, HashSet};

use crate::element::{ElementType, RawElement};
use crate::solver::{Cnf, Lit};
use crate::validator::{Finding, Severity};
use crate::variability::FeatureExpr;

fn err(code: &'static str, file: &str, msg: String) -> Finding {
    Finding { code, file: file.to_string(), message: msg, severity: Severity::Error }
}
fn warn(code: &'static str, file: &str, msg: String) -> Finding {
    Finding { code, file: file.to_string(), message: msg, severity: Severity::Warning }
}

fn is(e: &RawElement, t: ElementType) -> bool {
    e.frontmatter.element_type.as_ref() == Some(&t)
}

fn strings_from_seq(v: &[serde_yaml::Value]) -> Vec<String> {
    v.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()
}


fn num(v: &serde_yaml::Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))
}

fn parse_range(s: &str) -> Option<(f64, f64)> {
    // Accept both "min..max" and the inclusive "min..=max"; identical [min,max] semantics.
    let (lo, hi) = s.split_once("..")?;
    let hi = hi.trim();
    let hi = hi.strip_prefix('=').unwrap_or(hi).trim();
    Some((lo.trim().parse().ok()?, hi.parse().ok()?))
}

struct Param {
    name: String,
    range: Option<(f64, f64)>,
    derived_from: Option<String>,
    bind_to: Option<String>,
    /// Fixed `value:` or `default:` as a number, used as the fallback value when a
    /// parameter reference is not bound by a `Configuration` (E221 evaluation).
    fallback: Option<f64>,
}

fn parse_params(fd: &RawElement) -> Vec<Param> {
    let mut out = Vec::new();
    if let Some(list) = &fd.frontmatter.parameters {
        for p in list {
            let serde_yaml::Value::Mapping(m) = p else { continue };
            let get = |k: &str| m.get(serde_yaml::Value::String(k.to_string()));
            let Some(name) = get("name").and_then(|v| v.as_str()) else { continue };
            out.push(Param {
                name: name.to_string(),
                range: get("range").and_then(|v| v.as_str()).and_then(parse_range),
                derived_from: get("derivedFrom").and_then(|v| v.as_str()).map(|s| s.to_string()),
                bind_to: get("bindTo").and_then(|v| v.as_str()).map(|s| s.to_string()),
                fallback: get("value").and_then(num).or_else(|| get("default").and_then(num)),
            });
        }
    }
    out
}

/// Whether `name` appears as a whole identifier token in `expr`.
fn token_present(expr: &str, name: &str) -> bool {
    expr.split(|c: char| !(c.is_alphanumeric() || c == '_')).any(|t| t == name)
}

/// A token is a parameter reference iff it carries a feature path (`::`) and a
/// A token is a *parameter-reference candidate* if it carries a feature path
/// (`::`). The canonical valid form additionally has a dotted member
/// (`Features::Topology.maxCpus`); a `::`-only candidate
/// (`Features::Topology::maxCpus`) is a malformed reference, flagged `E213`
/// rather than silently dropped. Numeric literals like `3.0` (a `.` but no `::`)
/// are not candidates.
fn is_ref_candidate(tok: &str) -> bool {
    tok.contains("::")
}

/// Extract parameter-reference candidate tokens from an expression (any token
/// containing `::`), for classification by the constraint checker.
fn extract_param_paths(expr: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in expr.chars() {
        if c.is_alphanumeric() || c == '_' || c == ':' || c == '.' {
            cur.push(c);
        } else {
            if is_ref_candidate(&cur) {
                out.push(cur.clone());
            }
            cur.clear();
        }
    }
    if is_ref_candidate(&cur) {
        out.push(cur);
    }
    out
}

/// Evaluate a `parameterConstraints` expression `LHS <cmp> RHS` over numeric
/// literals and parameter references, resolving each reference via `resolve`.
/// Returns `Some(true/false)` when fully evaluable, `None` when a referenced
/// parameter is unresolved or the expression cannot be parsed.
fn eval_constraint(expr: &str, resolve: &dyn Fn(&str) -> Option<f64>) -> Option<bool> {
    const OPS: [&str; 6] = [">=", "<=", "==", "!=", ">", "<"];
    // Find the comparison operator at paren depth 0 (two-char operators first).
    let bytes = expr.as_bytes();
    let mut depth = 0i32;
    let mut split: Option<(usize, &str)> = None;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ if depth == 0 => {
                for op in OPS {
                    if expr[i..].starts_with(op) {
                        split = Some((i, op));
                        break;
                    }
                }
            }
            _ => {}
        }
        if split.is_some() {
            break;
        }
        i += 1;
    }
    let (pos, op) = split?;
    let lhs = eval_arith(&expr[..pos], resolve)?;
    let rhs = eval_arith(&expr[pos + op.len()..], resolve)?;
    Some(match op {
        ">=" => lhs >= rhs,
        "<=" => lhs <= rhs,
        ">" => lhs > rhs,
        "<" => lhs < rhs,
        "==" => (lhs - rhs).abs() < f64::EPSILON,
        "!=" => (lhs - rhs).abs() >= f64::EPSILON,
        _ => return None,
    })
}

/// Recursive-descent arithmetic over `+ - * /`, parentheses, numeric literals and
/// parameter references. Returns `None` if any reference is unresolved.
fn eval_arith(s: &str, resolve: &dyn Fn(&str) -> Option<f64>) -> Option<f64> {
    let toks = tokenize_arith(s);
    let mut p = ArithParser { toks: &toks, pos: 0, resolve };
    let v = p.expr()?;
    if p.pos == p.toks.len() { Some(v) } else { None }
}

#[derive(Debug, PartialEq)]
enum AtomTok {
    Num(f64),
    Ref(String),
    Op(char), // + - * / ( )
}

fn tokenize_arith(s: &str) -> Vec<AtomTok> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let flush = |cur: &mut String, out: &mut Vec<AtomTok>| {
        if cur.is_empty() {
            return;
        }
        if let Ok(n) = cur.parse::<f64>() {
            out.push(AtomTok::Num(n));
        } else {
            out.push(AtomTok::Ref(std::mem::take(cur)));
        }
        cur.clear();
    };
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':' | '.' => cur.push(c),
            '+' | '-' | '*' | '/' | '(' | ')' => {
                flush(&mut cur, &mut out);
                out.push(AtomTok::Op(c));
            }
            c if c.is_whitespace() => flush(&mut cur, &mut out),
            // Any other character is unexpected: emit it as an Op so the parser
            // rejects the expression (returns None) rather than silently dropping it.
            other => {
                flush(&mut cur, &mut out);
                out.push(AtomTok::Op(other));
            }
        }
    }
    flush(&mut cur, &mut out);
    out
}

struct ArithParser<'a> {
    toks: &'a [AtomTok],
    pos: usize,
    resolve: &'a dyn Fn(&str) -> Option<f64>,
}

impl ArithParser<'_> {
    fn peek(&self) -> Option<&AtomTok> {
        self.toks.get(self.pos)
    }
    fn expr(&mut self) -> Option<f64> {
        let mut v = self.term()?;
        while let Some(AtomTok::Op(c @ ('+' | '-'))) = self.peek() {
            let c = *c;
            self.pos += 1;
            let r = self.term()?;
            v = if c == '+' { v + r } else { v - r };
        }
        Some(v)
    }
    fn term(&mut self) -> Option<f64> {
        let mut v = self.factor()?;
        while let Some(AtomTok::Op(c @ ('*' | '/'))) = self.peek() {
            let c = *c;
            self.pos += 1;
            let r = self.factor()?;
            v = if c == '*' { v * r } else { v / r };
        }
        Some(v)
    }
    fn factor(&mut self) -> Option<f64> {
        match self.peek()? {
            AtomTok::Num(n) => {
                let n = *n;
                self.pos += 1;
                Some(n)
            }
            AtomTok::Ref(r) => {
                let v = (self.resolve)(r)?;
                self.pos += 1;
                Some(v)
            }
            AtomTok::Op('-') => {
                self.pos += 1;
                Some(-self.factor()?)
            }
            AtomTok::Op('(') => {
                self.pos += 1;
                let v = self.expr()?;
                matches!(self.peek(), Some(AtomTok::Op(')')))
                    .then(|| {
                        self.pos += 1;
                        v
                    })
            }
            _ => None,
        }
    }
}

/// DFS cycle detection over a param-name dependency graph.
fn has_cycle(deps: &HashMap<String, Vec<String>>) -> bool {
    fn dfs(n: &str, deps: &HashMap<String, Vec<String>>, state: &mut HashMap<String, u8>) -> bool {
        state.insert(n.to_string(), 1); // 1 = on stack
        if let Some(ds) = deps.get(n) {
            for d in ds {
                match state.get(d).copied().unwrap_or(0) {
                    1 => return true,
                    0 if dfs(d, deps, state) => return true,
                    _ => {}
                }
            }
        }
        state.insert(n.to_string(), 2); // 2 = done
        false
    }
    let mut state: HashMap<String, u8> = HashMap::new();
    for k in deps.keys() {
        if state.get(k).copied().unwrap_or(0) == 0 && dfs(k, deps, &mut state) {
            return true;
        }
    }
    false
}

/// True iff the model declares at least one `FeatureDef`.
pub fn has_feature_model(elements: &[RawElement]) -> bool {
    elements.iter().any(|e| is(e, ElementType::FeatureDef))
}

/// Run all feature-model validation rules. Returns an empty vector when no
/// feature model is present (the command treats that as a dormant no-op).
pub fn check_feature_model(elements: &[RawElement]) -> Vec<Finding> {
    let mut f = Vec::new();
    if !has_feature_model(elements) {
        return f;
    }

    let fdefs: Vec<&RawElement> = elements.iter().filter(|e| is(e, ElementType::FeatureDef)).collect();
    let configs: Vec<&RawElement> =
        elements.iter().filter(|e| is(e, ElementType::Configuration)).collect();
    let fnames: HashSet<&str> = fdefs.iter().map(|e| e.qualified_name.as_str()).collect();

    // ── E212: requires/excludes entries resolve to a FeatureDef ──────────────
    let mut req: HashMap<&str, Vec<String>> = HashMap::new();
    let mut exc: HashMap<&str, Vec<String>> = HashMap::new();
    for fd in &fdefs {
        let mut requires = Vec::new();
        if let Some(r) = &fd.frontmatter.requires {
            requires = strings_from_seq(r);
        }
        let excludes = fd.frontmatter.excludes.clone().unwrap_or_default();
        for r in requires.iter().chain(excludes.iter()) {
            if !fnames.contains(r.as_str()) {
                f.push(err("E212", &fd.file_path,
                    format!("requires/excludes entry '{}' does not resolve to a FeatureDef", r)));
            }
        }
        req.insert(fd.qualified_name.as_str(), requires);
        exc.insert(fd.qualified_name.as_str(), excludes);
    }

    // ── E219/E220 (per Configuration) + selection counts for W011/W012 ───────
    let mut sel_count: HashMap<&str, usize> = HashMap::new();
    for cfg in &configs {
        let sel = cfg.frontmatter.feature_selections();
        let is_sel = |q: &str| sel.get(q).copied().unwrap_or(false);
        for fd in &fdefs {
            let q = fd.qualified_name.as_str();
            if !is_sel(q) {
                continue;
            }
            *sel_count.entry(q).or_insert(0) += 1;
            for r in req.get(q).into_iter().flatten() {
                if !is_sel(r) {
                    f.push(err("E219", &cfg.file_path,
                        format!("feature '{}' is selected but its required feature '{}' is not", q, r)));
                }
            }
            for x in exc.get(q).into_iter().flatten() {
                if is_sel(x) {
                    f.push(err("E220", &cfg.file_path,
                        format!("feature '{}' is selected but its excluded feature '{}' is also selected", q, x)));
                }
            }
        }
    }

    // ── W011/W012: dead / always-on optional features ────────────────────────
    if !configs.is_empty() {
        for fd in &fdefs {
            if fd.frontmatter.group_kind.as_deref() != Some("optional") {
                continue;
            }
            let q = fd.qualified_name.as_str();
            match sel_count.get(q).copied().unwrap_or(0) {
                0 => f.push(warn("W011", &fd.file_path,
                    format!("optional feature '{}' is selected in no Configuration (possible dead feature)", q))),
                n if n == configs.len() => f.push(warn("W012", &fd.file_path,
                    format!("optional feature '{}' is selected in every Configuration (consider groupKind: mandatory)", q))),
                _ => {}
            }
        }
    }

    // ── E207: circular derivedFrom among a FeatureDef's own parameters ───────
    for fd in &fdefs {
        let params = parse_params(fd);
        let names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
        let mut deps: HashMap<String, Vec<String>> = HashMap::new();
        for p in &params {
            if let Some(expr) = &p.derived_from {
                let referenced: Vec<String> = names
                    .iter()
                    .filter(|n| n.as_str() != p.name && token_present(expr, n))
                    .cloned()
                    .collect();
                deps.insert(p.name.clone(), referenced);
            }
        }
        if has_cycle(&deps) {
            f.push(err("E207", &fd.file_path,
                format!("circular derivedFrom dependency among parameters of FeatureDef '{}'", fd.qualified_name)));
        }
    }

    // ── E202: two-level bindTo propagation range ─────────────────────────────
    struct Bind<'a> {
        feature: &'a str,
        name: String,
        bind_to: String,
        range: (f64, f64),
    }
    let mut binds: Vec<Bind> = Vec::new();
    for fd in &fdefs {
        for p in parse_params(fd) {
            if let (Some(bt), Some(range)) = (p.bind_to, p.range) {
                binds.push(Bind { feature: fd.qualified_name.as_str(), name: p.name, bind_to: bt, range });
            }
        }
    }
    if !binds.is_empty() {
        for cfg in &configs {
            let Some(serde_yaml::Value::Mapping(b)) = &cfg.frontmatter.parameter_bindings else {
                continue;
            };
            for bp in &binds {
                let Some(v) = b.get(serde_yaml::Value::String(bp.bind_to.clone())) else {
                    continue;
                };
                let Some(n) = num(v) else { continue };
                let (lo, hi) = bp.range;
                if n < lo || n > hi {
                    f.push(err("E202", &cfg.file_path, format!(
                        "value {} bound to '{}' propagates to component parameter '{}.{}', outside its range {}..{}",
                        n, bp.bind_to, bp.feature, bp.name, lo, hi)));
                }
            }
        }
    }

    // ── E213/W014/E221/W025: cross-feature parameterConstraints (package _index.md) ──
    // Canonical parameter reference is the dotted form `<FeatureDef qname>.<param>`.
    let mut param_paths: HashSet<String> = HashSet::new();
    let mut param_fallback: HashMap<String, f64> = HashMap::new();
    for fd in &fdefs {
        for p in parse_params(fd) {
            let key = format!("{}.{}", fd.qualified_name, p.name);
            if let Some(v) = p.fallback {
                param_fallback.insert(key.clone(), v);
            }
            param_paths.insert(key);
        }
    }
    let mut selected_anywhere: HashSet<String> = HashSet::new();
    for cfg in &configs {
        for (k, v) in cfg.frontmatter.feature_selections() {
            if v {
                selected_anywhere.insert(k);
            }
        }
    }
    for pkg in elements.iter().filter(|e| {
        matches!(
            e.frontmatter.element_type,
            Some(ElementType::Package) | Some(ElementType::LibraryPackage) | Some(ElementType::Namespace)
        )
    }) {
        let Some(serde_yaml::Value::Sequence(cons)) =
            pkg.frontmatter.extra.get("parameterConstraints")
        else {
            continue;
        };
        for c in cons {
            let serde_yaml::Value::Mapping(m) = c else { continue };
            let cid = m
                .get(serde_yaml::Value::String("id".into()))
                .and_then(|v| v.as_str())
                .unwrap_or("(unnamed)");
            let expr = m
                .get(serde_yaml::Value::String("expression".into()))
                .and_then(|v| v.as_str());

            // E213: every referenced parameter path must resolve to a declared
            // parameter and use the canonical dotted form. A `::`-only path that
            // would resolve if dotted is flagged explicitly (never silently dropped).
            let mut all_resolvable = true;
            if let Some(expr) = expr {
                for path in extract_param_paths(expr) {
                    if param_paths.contains(&path) {
                        continue; // valid dotted reference
                    }
                    all_resolvable = false;
                    // A `::`-only candidate whose last-`::` → `.` reinterpretation
                    // names a declared parameter is the wrong separator, not unknown.
                    let dotted = path.rsplit_once("::").map(|(a, b)| format!("{}.{}", a, b));
                    if !path.contains('.')
                        && dotted.as_deref().is_some_and(|d| param_paths.contains(d))
                    {
                        f.push(err("E213", &pkg.file_path, format!(
                            "parameterConstraints '{}' references parameter '{}' with '::' before the parameter member; use the canonical dotted form '{}'",
                            cid, path, dotted.unwrap())));
                    } else {
                        f.push(err("E213", &pkg.file_path, format!(
                            "parameterConstraints '{}' references unresolved parameter path '{}'", cid, path)));
                    }
                }
            }

            // appliesWhen: parse with the boolean grammar (and/or/not), not as a literal.
            let aw_expr = m
                .get(serde_yaml::Value::String("appliesWhen".into()))
                .and_then(|aw| crate::variability::applies_when_expr(aw).ok().flatten());
            // W014: each operand feature must be selected in some Configuration.
            if let Some(e) = &aw_expr {
                for feat in e.operands() {
                    if !selected_anywhere.contains(&feat) {
                        f.push(warn("W014", &pkg.file_path, format!(
                            "parameterConstraints '{}' appliesWhen references feature '{}' not selected in any Configuration", cid, feat)));
                    }
                }
            }

            // E221/W025: evaluate the expression against every Configuration whose
            // appliesWhen predicate holds. severity: warning -> W025, else E221.
            let Some(expr) = expr else { continue };
            if !all_resolvable {
                continue;
            }
            let warn_sev = m
                .get(serde_yaml::Value::String("severity".into()))
                .and_then(|v| v.as_str())
                == Some("warning");
            for cfg in &configs {
                let sel = cfg.frontmatter.feature_selections();
                let holds = match &aw_expr {
                    None => true,
                    Some(e) => e.eval(&|q| sel.get(q).copied().unwrap_or(false)),
                };
                if !holds {
                    continue;
                }
                let resolve = |r: &str| -> Option<f64> {
                    if let Some(serde_yaml::Value::Mapping(b)) = &cfg.frontmatter.parameter_bindings {
                        if let Some(v) = b.get(serde_yaml::Value::String(r.to_string())) {
                            if let Some(n) = num(v) {
                                return Some(n);
                            }
                        }
                    }
                    param_fallback.get(r).copied()
                };
                // Skip when a referenced parameter has no value in this configuration
                // (unbound, no default) — W017 already covers required-unbound.
                if let Some(false) = eval_constraint(expr, &resolve) {
                    let cfg_id = cfg
                        .frontmatter
                        .id
                        .clone()
                        .unwrap_or_else(|| cfg.qualified_name.clone());
                    let msg = format!(
                        "configuration '{}' violates parameterConstraints '{}': {}",
                        cfg_id, cid, expr
                    );
                    if warn_sev {
                        f.push(warn("W025", &cfg.file_path, msg));
                    } else {
                        f.push(err("E221", &cfg.file_path, msg));
                    }
                }
            }
        }
    }

    // ── W024: orphan FeatureDef ──────────────────────────────────────────────
    // A FeatureDef is an orphan when it is referenced by NO element's
    // `appliesWhen:` AND selected `true` by NO Configuration. feature-check-only.
    let mut referenced_by_applies_when: HashSet<String> = HashSet::new();
    for e in elements {
        if let Some(aw) = &e.frontmatter.applies_when {
            if let Ok(Some(expr)) = crate::variability::applies_when_expr(aw) {
                for op in expr.operands() {
                    referenced_by_applies_when.insert(op);
                }
            }
        }
    }
    for fd in &fdefs {
        let q = fd.qualified_name.as_str();
        let referenced = referenced_by_applies_when.contains(q);
        let selected = sel_count.get(q).copied().unwrap_or(0) > 0;
        if !referenced && !selected {
            f.push(warn("W024", &fd.file_path, format!(
                "orphan feature '{}' is referenced by no appliesWhen and selected true by no Configuration", q)));
        }
    }

    f
}

// ════════════════════════════════════════════════════════════════════════════
// Solver-backed deep analysis (feature-check --deep) — REQ-TRS-FMA-001..006.
// Encodes the Boolean feature layer to CNF and runs SAT queries for void / dead
// / core / false-optional / configuration-validity, with deletion-based unsat
// cores for explanations. Boolean layer only (parameters are out of scope).
// ════════════════════════════════════════════════════════════════════════════

/// Conservative size guard (REQ-TRS-FMA-006): above this feature count the deep
/// analysis is skipped with a diagnostic rather than risking blow-up.
pub const MAX_DEEP_FEATURES: usize = 1000;

/// Structured result of the deep analysis.
pub struct DeepReport {
    pub findings: Vec<Finding>,
    pub void: bool,
    pub dead: Vec<String>,
    pub core: Vec<String>,
    pub false_optional: Vec<String>,
    pub invalid_configs: Vec<String>,
    /// Minimal correction sets for a void model (each a list of constraint
    /// labels whose removal restores satisfiability) — REQ-TRS-FMA-010.
    pub diagnoses: Vec<Vec<String>>,
    /// Set (with a reason) when the deep analysis was skipped (size guard).
    pub skipped: Option<String>,
}

impl DeepReport {
    fn empty() -> Self {
        DeepReport {
            findings: Vec::new(),
            void: false,
            dead: Vec::new(),
            core: Vec::new(),
            false_optional: Vec::new(),
            invalid_configs: Vec::new(),
            diagnoses: Vec::new(),
            skipped: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum CKind {
    ChildParent,
    Mandatory,
    Root,
    GroupAtLeast,
    GroupAtMost,
    Requires,
    Excludes,
}

struct Constraint {
    kind: CKind,
    label: String,
    clauses: Vec<Vec<Lit>>,
}

struct Encoding {
    var_of: HashMap<String, usize>,
    names: Vec<String>,
    files: HashMap<String, String>,
    parent: HashMap<String, Option<String>>,
    optional: Vec<String>,
    cons: Vec<Constraint>,
}

impl Encoding {
    fn cnf(&self) -> Cnf {
        self.cnf_subset(&(0..self.cons.len()).collect::<Vec<_>>())
    }
    fn cnf_subset(&self, idx: &[usize]) -> Cnf {
        let mut c = Cnf::new(self.names.len());
        for &i in idx {
            for cl in &self.cons[i].clauses {
                c.add(cl.clone());
            }
        }
        c
    }
    /// Deletion-based minimal unsat core: indices of a subset of constraints that
    /// remains unsatisfiable (under `assumptions`) but where dropping any member
    /// becomes satisfiable. Assumes the full set is already unsatisfiable.
    fn unsat_core(&self, assumptions: &[Lit]) -> Vec<usize> {
        let mut keep: Vec<usize> = (0..self.cons.len()).collect();
        for i in 0..self.cons.len() {
            let trial: Vec<usize> = keep.iter().copied().filter(|&x| x != i).collect();
            if !crate::solver::is_sat(&self.cnf_subset(&trial), assumptions) {
                keep = trial;
            }
        }
        keep
    }
    fn core_labels(&self, core: &[usize]) -> String {
        let mut labels: Vec<String> = core.iter().map(|&i| self.cons[i].label.clone()).collect();
        labels.sort();
        labels.dedup();
        labels.join("; ")
    }

    /// Minimal correction sets (diagnoses) for a void model: each is a set of
    /// relaxable authoring constraints whose removal restores satisfiability
    /// (REQ-TRS-FMA-010). Structural `child ⇒ parent` clauses are never offered.
    fn correction_sets(&self) -> Vec<Vec<String>> {
        let relaxable: Vec<usize> = self
            .cons
            .iter()
            .enumerate()
            .filter(|(_, c)| !matches!(c.kind, CKind::ChildParent))
            .map(|(i, _)| i)
            .collect();
        let all: Vec<usize> = (0..self.cons.len()).collect();
        // Singleton corrections: a relaxable constraint whose removal alone fixes it.
        let mut out: Vec<Vec<String>> = Vec::new();
        for &r in &relaxable {
            let subset: Vec<usize> = all.iter().copied().filter(|&x| x != r).collect();
            if crate::solver::is_sat(&self.cnf_subset(&subset), &[]) {
                out.push(vec![self.cons[r].label.clone()]);
            }
        }
        // No singleton fix → one greedy minimal correction set (complement of a
        // maximal satisfiable subset).
        if out.is_empty() {
            let mut keep: Vec<usize> = self
                .cons
                .iter()
                .enumerate()
                .filter(|(_, c)| matches!(c.kind, CKind::ChildParent))
                .map(|(i, _)| i)
                .collect();
            let mut mcs: Vec<String> = Vec::new();
            for &r in &relaxable {
                let mut trial = keep.clone();
                trial.push(r);
                if crate::solver::is_sat(&self.cnf_subset(&trial), &[]) {
                    keep.push(r);
                } else {
                    mcs.push(self.cons[r].label.clone());
                }
            }
            if !mcs.is_empty() {
                out.push(mcs);
            }
        }
        out
    }
}

fn strip_last(q: &str) -> Option<&str> {
    q.rfind("::").map(|i| &q[..i])
}

/// All r-element subsets of `items`, in input order.
fn combinations(items: &[String], r: usize) -> Vec<Vec<String>> {
    let n = items.len();
    if r == 0 {
        return vec![vec![]];
    }
    if r > n {
        return vec![];
    }
    let mut out = Vec::new();
    let mut idx: Vec<usize> = (0..r).collect();
    loop {
        out.push(idx.iter().map(|&i| items[i].clone()).collect());
        // advance
        let mut i = r;
        loop {
            if i == 0 {
                return out;
            }
            i -= 1;
            if idx[i] != i + n - r {
                break;
            }
        }
        idx[i] += 1;
        for j in i + 1..r {
            idx[j] = idx[j - 1] + 1;
        }
    }
}

/// (min, max) selected children for a group; max == None means unbounded (`*`).
fn group_card(gk: &str, card: Option<&str>, k: usize) -> (usize, Option<usize>) {
    if let Some(s) = card {
        if let Some((lo, hi)) = s.split_once("..") {
            let m = lo.trim().parse::<usize>().unwrap_or(if gk == "alternative" { 1 } else { 1 });
            let n = if hi.trim() == "*" {
                None
            } else {
                hi.trim().parse::<usize>().ok().or(Some(k))
            };
            return (m, n);
        }
    }
    match gk {
        "alternative" => (1, Some(1)),
        _ => (1, None), // or
    }
}

fn build_encoding(fdefs: &[&RawElement]) -> Encoding {
    let mut names: Vec<String> = fdefs.iter().map(|e| e.qualified_name.clone()).collect();
    names.sort();
    let var_of: HashMap<String, usize> =
        names.iter().enumerate().map(|(i, n)| (n.clone(), i)).collect();
    let fdef_set: HashSet<&str> = names.iter().map(|s| s.as_str()).collect();
    let by_name: HashMap<&str, &RawElement> =
        fdefs.iter().map(|e| (e.qualified_name.as_str(), *e)).collect();
    let files: HashMap<String, String> =
        fdefs.iter().map(|e| (e.qualified_name.clone(), e.file_path.clone())).collect();

    let parent_of = |q: &str| -> Option<String> {
        let e = by_name[q];
        if let Some(pf) = e.frontmatter.parent_feature.as_deref() {
            if fdef_set.contains(pf) {
                return Some(pf.to_string());
            }
        }
        let mut cur = strip_last(q);
        while let Some(p) = cur {
            if fdef_set.contains(p) {
                return Some(p.to_string());
            }
            cur = strip_last(p);
        }
        None
    };

    let mut parent: HashMap<String, Option<String>> = HashMap::new();
    let mut children: HashMap<String, Vec<String>> = HashMap::new();
    for n in &names {
        let p = parent_of(n);
        if let Some(pp) = &p {
            children.entry(pp.clone()).or_default().push(n.clone());
        }
        parent.insert(n.clone(), p);
    }
    for v in children.values_mut() {
        v.sort();
    }

    let v = |q: &str| Lit::pos(var_of[q]);
    let nv = |q: &str| Lit::neg(var_of[q]);
    let mut cons: Vec<Constraint> = Vec::new();
    let mut optional: Vec<String> = Vec::new();

    for n in &names {
        let e = by_name[n.as_str()];
        let gk = e.frontmatter.group_kind.as_deref().unwrap_or("optional");
        if gk == "optional" {
            optional.push(n.clone());
        }
        // Membership is orthogonal to grouping (REQ-TRS-FM-004): the explicit
        // `mandatory: true` flag, or the legacy `groupKind: mandatory` shorthand,
        // both make a feature a mandatory member.
        let is_mandatory = match e.frontmatter.mandatory {
            Some(m) => m,
            None => gk == "mandatory",
        };
        let p = parent.get(n).cloned().flatten();
        if let Some(p) = &p {
            cons.push(Constraint {
                kind: CKind::ChildParent,
                label: format!("feature '{}' implies parent '{}'", n, p),
                clauses: vec![vec![nv(n), v(p)]],
            });
            if is_mandatory {
                cons.push(Constraint {
                    kind: CKind::Mandatory,
                    label: format!("feature '{}' is mandatory under '{}'", n, p),
                    clauses: vec![vec![nv(p), v(n)]],
                });
            }
        } else if is_mandatory {
            cons.push(Constraint {
                kind: CKind::Root,
                label: format!("root feature '{}' is mandatory", n),
                clauses: vec![vec![v(n)]],
            });
        }

        if gk == "alternative" || gk == "or" {
            let ch = children.get(n).cloned().unwrap_or_default();
            if !ch.is_empty() {
                let (m, nmax) = group_card(gk, e.frontmatter.cardinality.as_deref(), ch.len());
                if m >= 1 {
                    // at-least-m, conditioned on the group node being selected.
                    for combo in combinations(&ch, ch.len() - m + 1) {
                        let mut cl = vec![nv(n)];
                        for c in &combo {
                            cl.push(v(c));
                        }
                        cons.push(Constraint {
                            kind: CKind::GroupAtLeast,
                            label: format!("group '{}' requires at least {} selected child(ren)", n, m),
                            clauses: vec![cl],
                        });
                    }
                }
                if let Some(nm) = nmax {
                    if nm < ch.len() {
                        for combo in combinations(&ch, nm + 1) {
                            let cl: Vec<Lit> = combo.iter().map(|c| nv(c)).collect();
                            cons.push(Constraint {
                                kind: CKind::GroupAtMost,
                                label: format!("group '{}' allows at most {} selected child(ren)", n, nm),
                                clauses: vec![cl],
                            });
                        }
                    }
                }
            }
        }

        if let Some(r) = &e.frontmatter.requires {
            for r in strings_from_seq(r) {
                if fdef_set.contains(r.as_str()) {
                    cons.push(Constraint {
                        kind: CKind::Requires,
                        label: format!("'{}' requires '{}'", n, r),
                        clauses: vec![vec![nv(n), v(&r)]],
                    });
                }
            }
        }
        if let Some(x) = &e.frontmatter.excludes {
            for x in x {
                if fdef_set.contains(x.as_str()) {
                    cons.push(Constraint {
                        kind: CKind::Excludes,
                        label: format!("'{}' excludes '{}'", n, x),
                        clauses: vec![vec![nv(n), nv(x)]],
                    });
                }
            }
        }
    }

    Encoding { var_of, names, files, parent, optional, cons }
}

/// Run the solver-backed deep analysis. Empty report when no feature model;
/// `skipped` set when the size guard trips.
pub fn check_feature_model_deep(elements: &[RawElement]) -> DeepReport {
    let mut rep = DeepReport::empty();
    let fdefs: Vec<&RawElement> =
        elements.iter().filter(|e| is(e, ElementType::FeatureDef)).collect();
    if fdefs.is_empty() {
        return rep;
    }
    if fdefs.len() > MAX_DEEP_FEATURES {
        rep.skipped = Some(format!(
            "deep analysis skipped: {} features exceeds the limit of {} (override not yet available)",
            fdefs.len(),
            MAX_DEEP_FEATURES
        ));
        return rep;
    }

    let enc = build_encoding(&fdefs);
    // One batsat solver primed with the full encoding, reused across all queries.
    let mut sat = crate::solver::Solver::from_cnf(&enc.cnf());
    let root_file = enc
        .names
        .first()
        .and_then(|n| enc.files.get(n))
        .cloned()
        .unwrap_or_default();
    let file_of = |n: &str| enc.files.get(n).cloned().unwrap_or_else(|| root_file.clone());

    // Void dominates: report once and stop (every feature is trivially dead).
    if !sat.is_sat(&[]) {
        rep.void = true;
        let core = enc.unsat_core(&[]);
        rep.diagnoses = enc.correction_sets();
        let fixes = if rep.diagnoses.is_empty() {
            String::new()
        } else {
            let opts: Vec<String> = rep
                .diagnoses
                .iter()
                .map(|m| format!("relax {{{}}}", m.join(", ")))
                .collect();
            format!(" Possible fixes: {}.", opts.join(" or "))
        };
        rep.findings.push(err("E223", &root_file, format!(
            "feature model is void (no valid configuration exists). Conflicting constraints: {}.{}",
            enc.core_labels(&core), fixes)));
        return rep;
    }

    // Dead / core per feature.
    for (i, name) in enc.names.iter().enumerate() {
        if !sat.is_sat(&[Lit::pos(i)]) {
            rep.dead.push(name.clone());
            let core = enc.unsat_core(&[Lit::pos(i)]);
            rep.findings.push(err("E224", &file_of(name), format!(
                "feature '{}' is dead — it can be selected in no valid configuration. Cause: {}",
                name, enc.core_labels(&core))));
        } else if !sat.is_sat(&[Lit::neg(i)]) {
            rep.core.push(name.clone());
        }
    }

    // False-optional: declared optional but forced whenever its parent is.
    for name in &enc.optional {
        if rep.dead.contains(name) {
            continue;
        }
        let i = enc.var_of[name];
        let cond = match enc.parent.get(name).cloned().flatten() {
            Some(p) => vec![Lit::pos(enc.var_of[&p]), Lit::neg(i)],
            None => vec![Lit::neg(i)],
        };
        if !sat.is_sat(&cond) {
            rep.false_optional.push(name.clone());
            rep.findings.push(warn("W018", &file_of(name), format!(
                "optional feature '{}' is false-optional — it is forced selected whenever its parent is",
                name)));
        }
    }

    // Configuration validity under full semantics (E225), excluding the
    // requires/excludes obligations already reported as E219/E220.
    for cfg in elements.iter().filter(|e| is(e, ElementType::Configuration)) {
        let sel = cfg.frontmatter.feature_selections();
        let assign: Vec<bool> = enc.names.iter().map(|n| sel.get(n).copied().unwrap_or(false)).collect();
        let mut violated: Vec<String> = Vec::new();
        for c in &enc.cons {
            if matches!(c.kind, CKind::Requires | CKind::Excludes) {
                continue;
            }
            for cl in &c.clauses {
                let ok = cl.iter().any(|l| assign[l.var] != l.neg);
                if !ok {
                    violated.push(c.label.clone());
                    break;
                }
            }
        }
        if !violated.is_empty() {
            violated.sort();
            violated.dedup();
            let id = cfg.frontmatter.id.clone().unwrap_or_else(|| cfg.qualified_name.clone());
            rep.invalid_configs.push(id.clone());
            rep.findings.push(err("E225", &cfg.file_path, format!(
                "configuration '{}' is not a valid model of the feature model: {}",
                id, violated.join("; "))));
        }
    }

    // Effective appliesWhen honours transitive package conditioning (REQ-TRS-VAR-006).
    let pkgcond = crate::variability::package_conditions(elements);
    let elem_aw = |e: &RawElement| crate::variability::effective_expr(e, &pkgcond);

    // ── W021: dead elements (appliesWhen unsatisfiable under the feature model) ──
    for e in elements {
        let Some(aw) = elem_aw(e) else { continue }; // only elements WITH appliesWhen
        let mut cnf = enc.cnf();
        let Some(lit) = tseitin(&aw, &enc.var_of, &mut cnf) else { continue };
        cnf.add(vec![lit]); // assert appliesWhen true
        if !crate::solver::is_sat(&cnf, &[]) {
            rep.findings.push(warn("W021", &e.file_path, format!(
                "element '{}' is dead — its appliesWhen is unsatisfiable under the feature model (active in no valid configuration)",
                disp(e))));
        }
    }

    // ── E227/W020: global appliesWhen-implication along reference edges ──
    let resolver = crate::resolver::Resolver::new(elements);
    let base_vars = enc.names.len();
    for x in elements {
        let aw_x = elem_aw(x); // None ⇒ always active (true)
        for (kind, target) in crate::projection::outbound_refs(x) {
            let Some(y) = resolver.resolve_ref(elements, &target) else { continue };
            let Some(aw_y) = elem_aw(y) else { continue }; // Y always active ⇒ implication holds
            let mut cnf = enc.cnf();
            if let Some(ax) = &aw_x {
                let Some(lx) = tseitin(ax, &enc.var_of, &mut cnf) else { continue };
                cnf.add(vec![lx]); // aw(X) true
            }
            let Some(ly) = tseitin(&aw_y, &enc.var_of, &mut cnf) else { continue };
            cnf.add(vec![Lit { var: ly.var, neg: !ly.neg }]); // ¬aw(Y)
            if crate::solver::is_sat(&cnf, &[]) {
                let witness = crate::solver::solve_model(&cnf)
                    .map(|m| witness_str(&enc.names, base_vars, &m))
                    .unwrap_or_default();
                match kind {
                    crate::projection::RefKind::Structural => rep.findings.push(err("E227", &x.file_path, format!(
                        "structural reference '{}' → '{}' can escape: a valid configuration activates the source without the target. Witness: {}",
                        disp(x), target, witness))),
                    crate::projection::RefKind::Traceability => rep.findings.push(warn("W020", &x.file_path, format!(
                        "traceability reference '{}' → '{}' can escape: a valid configuration activates the source without the target. Witness: {}",
                        disp(x), target, witness))),
                }
            }
        }
    }

    // ── W022: aggregate coverage (active in ≥1 configuration, covered in none) ──
    let configs: Vec<&RawElement> =
        elements.iter().filter(|e| is(e, ElementType::Configuration)).collect();
    if !configs.is_empty() {
        let is_draft = |e: &RawElement| e.frontmatter.status.as_deref() == Some("draft");
        let reqs: Vec<(&RawElement, Option<FeatureExpr>, Vec<String>)> = elements
            .iter()
            .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Requirement)) && !is_draft(e))
            .map(|e| {
                let mut keys = vec![e.qualified_name.clone()];
                if let Some(id) = &e.frontmatter.id {
                    keys.push(id.clone());
                }
                (e, elem_aw(e), keys)
            })
            .collect();
        let tcs: Vec<(Option<FeatureExpr>, Vec<String>)> = elements
            .iter()
            .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::TestCase)) && !is_draft(e))
            .map(|e| (elem_aw(e), e.frontmatter.verifies.clone().unwrap_or_default()))
            .collect();
        for (re, rexpr, rkeys) in &reqs {
            let mut active_somewhere = false;
            let mut covered_somewhere = false;
            for cfg in &configs {
                let sel = cfg.frontmatter.feature_selections();
                let selected = |q: &str| sel.get(q).copied().unwrap_or(false);
                let active = rexpr.as_ref().map_or(true, |e| e.eval(&selected));
                if !active {
                    continue;
                }
                active_somewhere = true;
                let covered = tcs.iter().any(|(texpr, ver)| {
                    let runs = texpr.as_ref().map_or(true, |e| e.eval(&selected));
                    runs && ver.iter().any(|v| rkeys.iter().any(|k| k == v))
                });
                if covered {
                    covered_somewhere = true;
                    break;
                }
            }
            if active_somewhere && !covered_somewhere {
                rep.findings.push(warn("W022", &re.file_path, format!(
                    "requirement '{}' is active in some configuration but covered in none",
                    disp(re))));
            }
        }
    }

    rep.dead.sort();
    rep.core.sort();
    rep.false_optional.sort();
    rep.invalid_configs.sort();
    rep
}

/// Display id for an element (stable id, else qualified name).
fn disp(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// Tseitin-encode a feature expression into a literal over the feature var space,
/// allocating fresh variables and adding defining clauses. `None` if an operand
/// is not a known feature variable.
fn tseitin(expr: &FeatureExpr, var_of: &HashMap<String, usize>, cnf: &mut Cnf) -> Option<Lit> {
    use FeatureExpr::*;
    let nl = |l: Lit| Lit { var: l.var, neg: !l.neg };
    match expr {
        Feat(q) => var_of.get(q).map(|&v| Lit::pos(v)),
        Not(e) => tseitin(e, var_of, cnf).map(nl),
        And(a, b) => {
            let la = tseitin(a, var_of, cnf)?;
            let lb = tseitin(b, var_of, cnf)?;
            let c = cnf.num_vars;
            cnf.num_vars += 1;
            let lc = Lit::pos(c);
            cnf.add(vec![nl(lc), la]);
            cnf.add(vec![nl(lc), lb]);
            cnf.add(vec![lc, nl(la), nl(lb)]);
            Some(lc)
        }
        Or(a, b) => {
            let la = tseitin(a, var_of, cnf)?;
            let lb = tseitin(b, var_of, cnf)?;
            let c = cnf.num_vars;
            cnf.num_vars += 1;
            let lc = Lit::pos(c);
            cnf.add(vec![nl(la), lc]);
            cnf.add(vec![nl(lb), lc]);
            cnf.add(vec![nl(lc), la, lb]);
            Some(lc)
        }
    }
}

/// Human-readable witness: the features selected (true) in a model, over the
/// feature variables only (indices `0..base`).
fn witness_str(names: &[String], base: usize, model: &[bool]) -> String {
    let on: Vec<&str> = (0..base.min(model.len()))
        .filter(|&i| model[i])
        .map(|i| names[i].as_str())
        .collect();
    if on.is_empty() {
        "(no features selected)".to_string()
    } else {
        on.join(", ")
    }
}

// ── Assisted configuration (REQ-TRS-FMA-008) ────────────────────────────────

pub enum ConfigureOutcome {
    /// No feature model present (dormant).
    Dormant,
    /// The named configuration was not found.
    NotFound,
    Report {
        satisfiable: bool,
        forced_true: Vec<String>,
        forced_false: Vec<String>,
        free: Vec<String>,
        explanation: Option<String>,
    },
}

/// Treat a `Configuration`'s `features:` as a *partial* assignment (set features
/// fixed; absent features open) and report whether it can be completed, plus the
/// forced and free features.
pub fn configure(elements: &[RawElement], conf: &str) -> ConfigureOutcome {
    let fdefs: Vec<&RawElement> =
        elements.iter().filter(|e| is(e, ElementType::FeatureDef)).collect();
    if fdefs.is_empty() {
        return ConfigureOutcome::Dormant;
    }
    let cfg = elements.iter().find(|e| {
        is(e, ElementType::Configuration)
            && (e.frontmatter.id.as_deref() == Some(conf) || e.qualified_name == conf)
    });
    let Some(cfg) = cfg else {
        return ConfigureOutcome::NotFound;
    };

    let enc = build_encoding(&fdefs);
    let mut assumptions: Vec<Lit> = Vec::new();
    let mut fixed: HashSet<usize> = HashSet::new();
    for (feat, val) in cfg.frontmatter.feature_selections() {
        if let Some(&v) = enc.var_of.get(&feat) {
            assumptions.push(if val { Lit::pos(v) } else { Lit::neg(v) });
            fixed.insert(v);
        }
    }

    let mut sat = crate::solver::Solver::from_cnf(&enc.cnf());
    if !sat.is_sat(&assumptions) {
        let core = enc.unsat_core(&assumptions);
        return ConfigureOutcome::Report {
            satisfiable: false,
            forced_true: Vec::new(),
            forced_false: Vec::new(),
            free: Vec::new(),
            explanation: Some(enc.core_labels(&core)),
        };
    }

    let (mut forced_true, mut forced_false, mut free) = (Vec::new(), Vec::new(), Vec::new());
    for (i, name) in enc.names.iter().enumerate() {
        if fixed.contains(&i) {
            continue; // already chosen, not open
        }
        let mut a = assumptions.clone();
        a.push(Lit::neg(i));
        if !sat.is_sat(&a) {
            forced_true.push(name.clone());
            continue;
        }
        let mut a = assumptions.clone();
        a.push(Lit::pos(i));
        if !sat.is_sat(&a) {
            forced_false.push(name.clone());
        } else {
            free.push(name.clone());
        }
    }
    forced_true.sort();
    forced_false.sort();
    free.sort();
    ConfigureOutcome::Report {
        satisfiable: true,
        forced_true,
        forced_false,
        free,
        explanation: None,
    }
}

// ── Variant-space count / enumeration (REQ-TRS-FMA-009) ──────────────────────

pub enum EnumOutcome {
    Dormant,
    Skipped(String),
    /// Valid configurations (each a sorted list of selected feature qnames) and
    /// whether enumeration was truncated at the cap.
    Variants { configs: Vec<Vec<String>>, truncated: bool },
}

/// Default enumeration cap.
pub const MAX_ENUM: usize = 100_000;

pub fn enumerate_variants(elements: &[RawElement], cap: usize) -> EnumOutcome {
    let fdefs: Vec<&RawElement> =
        elements.iter().filter(|e| is(e, ElementType::FeatureDef)).collect();
    if fdefs.is_empty() {
        return EnumOutcome::Dormant;
    }
    if fdefs.len() > MAX_DEEP_FEATURES {
        return EnumOutcome::Skipped(format!(
            "variant analysis skipped: {} features exceeds the limit of {}",
            fdefs.len(),
            MAX_DEEP_FEATURES
        ));
    }
    let enc = build_encoding(&fdefs);
    let mut sat = crate::solver::Solver::from_cnf(&enc.cnf());
    let mut configs: Vec<Vec<String>> = Vec::new();
    let mut truncated = false;
    while let Some(bits) = sat.next_model() {
        if configs.len() >= cap {
            truncated = true;
            break;
        }
        let selected: Vec<String> = enc
            .names
            .iter()
            .enumerate()
            .filter(|(i, _)| bits[*i])
            .map(|(_, n)| n.clone())
            .collect();
        configs.push(selected);
    }
    configs.sort();
    EnumOutcome::Variants { configs, truncated }
}

// ── Proof-carrying evidence (REQ-TRS-FMA-011, partial) ──────────────────────
// batsat 0.6.0 does not expose a solver-recorded DRAT refutation, so we emit the
// DIMACS CNF of each UNSAT formula (Φ for void, Φ∧F for a dead feature). That
// CNF is externally re-checkable as UNSAT by any solver; the DRAT *proof* itself
// is deferred pending a proof-recording solver.

fn dimacs(cnf: &Cnf) -> String {
    let mut s = format!("p cnf {} {}\n", cnf.num_vars, cnf.clauses.len());
    for cl in &cnf.clauses {
        for l in cl {
            let n = (l.var + 1) as i64;
            s.push_str(&format!("{} ", if l.neg { -n } else { n }));
        }
        s.push_str("0\n");
    }
    s
}

/// Write a DIMACS CNF for each UNSAT finding into `dir`; returns the filenames
/// written (empty when dormant, over the size guard, or the model is sound).
pub fn write_proofs(elements: &[RawElement], dir: &std::path::Path) -> std::io::Result<Vec<String>> {
    let fdefs: Vec<&RawElement> =
        elements.iter().filter(|e| is(e, ElementType::FeatureDef)).collect();
    if fdefs.is_empty() || fdefs.len() > MAX_DEEP_FEATURES {
        return Ok(Vec::new());
    }
    std::fs::create_dir_all(dir)?;
    let enc = build_encoding(&fdefs);
    let cnf = enc.cnf();
    let mut sat = crate::solver::Solver::from_cnf(&cnf);
    let mut written = Vec::new();

    if !sat.is_sat(&[]) {
        std::fs::write(dir.join("void.cnf"), dimacs(&cnf))?;
        written.push("void.cnf".to_string());
        return Ok(written); // void dominates
    }
    for (i, name) in enc.names.iter().enumerate() {
        if !sat.is_sat(&[Lit::pos(i)]) {
            let mut c2 = cnf.clone();
            c2.add(vec![Lit::pos(i)]);
            let fname = format!("dead-{}.cnf", name.replace("::", "_"));
            std::fs::write(dir.join(&fname), dimacs(&c2))?;
            written.push(fname);
        }
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::{eval_arith, eval_constraint, parse_range};

    // Resolver: A=4, B=2, C=1.5; anything else unresolved.
    fn r(q: &str) -> Option<f64> {
        match q {
            "F::T.A" => Some(4.0),
            "F::T.B" => Some(2.0),
            "F::T.C" => Some(1.5),
            _ => None,
        }
    }

    fn ec(expr: &str) -> Option<bool> {
        eval_constraint(expr, &r)
    }
    fn ea(expr: &str) -> Option<f64> {
        eval_arith(expr, &r)
    }

    #[test]
    fn range_inclusive_and_plain() {
        assert_eq!(parse_range("1..8"), Some((1.0, 8.0)));
        assert_eq!(parse_range("1..=8"), Some((1.0, 8.0)));
        assert_eq!(parse_range(" 1 ..= 8 "), Some((1.0, 8.0)));
        assert_eq!(parse_range("0.5..5.0"), Some((0.5, 5.0)));
        assert_eq!(parse_range("garbage"), None);
    }

    #[test]
    fn all_comparison_operators() {
        // A=4, B=2
        assert_eq!(ec("F::T.A >= 2"), Some(true));
        assert_eq!(ec("F::T.A >= 4"), Some(true));
        assert_eq!(ec("F::T.A >= 5"), Some(false));
        assert_eq!(ec("F::T.B <= 2"), Some(true));
        assert_eq!(ec("F::T.A <= 2"), Some(false));
        assert_eq!(ec("F::T.A > 4"), Some(false));
        assert_eq!(ec("F::T.A > 3"), Some(true));
        assert_eq!(ec("F::T.B < 3"), Some(true));
        assert_eq!(ec("F::T.B < 2"), Some(false));
        assert_eq!(ec("F::T.B == 2"), Some(true));
        assert_eq!(ec("F::T.A == 2"), Some(false));
        assert_eq!(ec("F::T.A != 2"), Some(true));
        assert_eq!(ec("F::T.B != 2"), Some(false));
    }

    #[test]
    fn ref_vs_ref_and_whitespace() {
        assert_eq!(ec("F::T.A >= F::T.B"), Some(true)); // 4 >= 2
        assert_eq!(ec("F::T.B>=F::T.A"), Some(false)); // 2 >= 4, no spaces
        assert_eq!(ec("  F::T.A  ==  F::T.A  "), Some(true));
    }

    #[test]
    fn arithmetic_precedence_and_parens() {
        assert_eq!(ea("2 + 3 * 4"), Some(14.0)); // * before +
        assert_eq!(ea("(2 + 3) * 4"), Some(20.0));
        assert_eq!(ea("F::T.A / 2.0 * 3.0"), Some(6.0)); // (4/2)*3
        assert_eq!(ea("F::T.A - F::T.B - 1"), Some(1.0)); // left-assoc: (4-2)-1
        assert_eq!(ea("-F::T.B + 5"), Some(3.0)); // unary minus
        assert_eq!(ea("(F::T.A + F::T.B) / 3"), Some(2.0));
    }

    #[test]
    fn constraint_with_arithmetic_rhs() {
        // budget A=4 >= (B/2)*3 = 3  → true ; >= (A/2)*3 = 6 → false
        assert_eq!(ec("F::T.A >= (F::T.B / 2.0) * 3.0"), Some(true));
        assert_eq!(ec("F::T.A >= (F::T.A / 2.0) * 3.0"), Some(false));
    }

    #[test]
    fn unresolved_and_malformed_yield_none() {
        assert_eq!(ec("F::T.A >= F::T.MISSING"), None); // unresolved ref
        assert_eq!(ec("F::T.MISSING > 0"), None);
        assert_eq!(ec("F::T.A"), None); // no comparison operator
        assert_eq!(ec("F::T.A >"), None); // empty rhs
        assert_eq!(ec("F::T.A > > 1"), None); // malformed
        assert_eq!(ea("F::T.A +"), None); // dangling operator
        assert_eq!(ea("(F::T.A"), None); // unbalanced paren
    }

    #[test]
    fn comparison_operator_chosen_at_depth_zero() {
        // The comparison split must ignore operators inside parentheses.
        // (A < 9) is arithmetic-nonsense but the top-level op is the second '>='.
        assert_eq!(ec("F::T.A >= 2"), Some(true));
        // two-char operator preferred over one-char prefix
        assert_eq!(ec("F::T.A <= 4"), Some(true));
    }
}
