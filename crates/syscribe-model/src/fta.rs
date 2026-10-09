//! Fault-tree analysis (IEC 61025 / ISO 26262-9) — GH #211, #212.
//!
//! One clean, reusable module with two layers:
//!
//! 1. A **pure Boolean core** ([`BoolTree`], [`analyze_bool`]) — a DAG of
//!    constants, basic-event variables and gates (`AND`/`OR`/`XOR`/`NOT`/`inhibit`)
//!    analysed through a reduced ordered BDD. It yields the exact top-event
//!    probability, the minimal cut sets, and the Birnbaum / Fussell-Vesely /
//!    RAW / RRW importance of every variable. It knows nothing about the model,
//!    so it is exercised against a brute-force truth-table oracle (see the tests).
//! 2. A **model adapter** ([`analyze_fault_tree`]) that reads one `FaultTree`
//!    element and its `FaultTreeGate`/`FaultTreeEvent` children, resolves
//!    `inputs:`, derives event probabilities, expands beta-factor common-cause
//!    groups, runs the core, and returns a [`FaultTreeAnalysis`] with stable,
//!    serialisable cut sets, probabilities and per-event [`EventRole`]s. The
//!    `metrics` roll-up, the `fault-tree analyze` command and the (planned)
//!    fault-tree diagram deriver all consume this one result.
//!
//! The structural checks of GH #212 ([`structural_issues`]) share the same
//! graph builder, so validation and analysis never disagree about what the tree
//! looks like.
//!
//! # Semantics
//!
//! * `AND` and `inhibit` are conjunctions (an `inhibit` gate's last input is the
//!   conditioning event). `OR` is a disjunction. `XOR` is parity. `NOT` negates
//!   its single input (extra inputs are an `E961` error and are OR-ed first).
//!   A gate with no inputs is constant FALSE.
//! * `basic` and `undeveloped` events are variables. A `house` event is a
//!   constant: TRUE when its `probability` is exactly `1`, otherwise FALSE.
//! * An event's probability is its explicit `probability:` when valid, else
//!   `1 − e^(−λ·t)` from `failureRate` (λ, /h) and the tree's `missionTime` (t).
//!   A gate's own `probability:` is informational and ignored.
//! * The **top** node is the tree node not referenced by any other node of the
//!   tree; when several qualify the one reaching most nodes wins (ties: id).
//! * Cut sets are the *minimal true points* of the structure function (set the
//!   members TRUE and every other event FALSE). For a coherent tree (no
//!   `NOT`/`XOR`) these are the classical minimal cut sets; the result carries
//!   `coherent = false` otherwise.
//! * A `ccfGroup:` of ≥2 events with a `ccfBeta:` β is expanded by the
//!   beta-factor model: each member `i` becomes `OR(i_ind, CCF_group)` with
//!   `P(i_ind) = (1−β)·p_i` and `P(CCF_group) = β·mean(p_i)`.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use serde::Serialize;
use thiserror::Error;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

// ── errors ───────────────────────────────────────────────────────────────────

/// Why an analysis could not be produced.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum FtaError {
    #[error("no FaultTree with id or qualified name '{0}'")]
    TreeNotFound(String),
    #[error("fault tree has no gates or events")]
    EmptyTree,
    #[error("fault tree has no top node (every node is referenced by another — a gate cycle)")]
    NoTopNode,
    #[error("fault tree contains a gate cycle: {}", .0.join(" -> "))]
    Cycle(Vec<String>),
    #[error("more than {0} minimal cut sets — raise the limit or set a cut-off order")]
    TooManyCutSets(usize),
}

// ── Boolean core ─────────────────────────────────────────────────────────────

/// Gate function of a [`Node::Gate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GateKind {
    And,
    Or,
    Xor,
    Not,
    Inhibit,
}

impl GateKind {
    /// Parse the `gateType:` spelling (`AND`, `OR`, `XOR`, `NOT`, `inhibit`).
    pub fn parse(s: &str) -> Option<GateKind> {
        match s {
            "AND" => Some(GateKind::And),
            "OR" => Some(GateKind::Or),
            "XOR" => Some(GateKind::Xor),
            "NOT" => Some(GateKind::Not),
            "inhibit" => Some(GateKind::Inhibit),
            _ => None,
        }
    }
}

/// One node of a [`BoolTree`].
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Const(bool),
    /// A basic-event variable (index into the probability vector).
    Var(usize),
    /// A gate over other node indices.
    Gate { kind: GateKind, inputs: Vec<usize> },
}

/// A Boolean DAG with a designated root. Variables are `0..num_vars`.
#[derive(Debug, Clone)]
pub struct BoolTree {
    pub nodes: Vec<Node>,
    pub root: usize,
    pub num_vars: usize,
}

/// Limits for [`analyze_bool`].
#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    /// Abort with [`FtaError::TooManyCutSets`] past this many cut sets.
    pub max_cut_sets: usize,
    /// Drop cut sets of larger order (cut-off); probabilities stay exact.
    pub max_order: Option<usize>,
    /// Expand `ccfGroup:` beta-factor groups (model adapter only).
    pub ccf: bool,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        AnalysisOptions { max_cut_sets: 200_000, max_order: None, ccf: true }
    }
}

/// One minimal cut set over variable indices (sorted ascending).
#[derive(Debug, Clone, PartialEq)]
pub struct VarCutSet {
    pub vars: Vec<usize>,
    /// Product of the member probabilities, when all are known.
    pub probability: Option<f64>,
}

/// Importance of one variable (all `None` when the needed probabilities are unknown).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct VarImportance {
    /// `P(top | x=1) − P(top | x=0)`.
    pub birnbaum: Option<f64>,
    /// `(P(top) − P(top | x=0)) / P(top)`.
    pub fussell_vesely: Option<f64>,
    /// Risk achievement worth `P(top | x=1) / P(top)`.
    pub raw: Option<f64>,
    /// Risk reduction worth `P(top) / P(top | x=0)`.
    pub rrw: Option<f64>,
}

/// Result of the Boolean core.
#[derive(Debug, Clone)]
pub struct BoolAnalysis {
    /// Minimal cut sets, sorted by (order, probability desc, vars).
    pub cut_sets: Vec<VarCutSet>,
    /// Exact top-event probability (BDD Shannon expansion).
    pub top_probability: Option<f64>,
    /// Rare-event approximation `Σ P(C)`.
    pub rare_event: Option<f64>,
    /// Min-cut upper bound `1 − Π(1 − P(C))`.
    pub min_cut_upper_bound: Option<f64>,
    /// Per-variable importance, indexed by variable.
    pub importance: Vec<VarImportance>,
    /// Variables the structure function actually depends on.
    pub support: Vec<bool>,
    /// False when a reachable `NOT`/`XOR` gate exists.
    pub coherent: bool,
    /// True when `max_order` discarded cut sets.
    pub truncated: bool,
}

// ---- BDD ----

const ZERO: u32 = 0;
const ONE: u32 = 1;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Op {
    And,
    Or,
    Xor,
}

struct Bdd {
    /// `(var, lo, hi)`; indices 0/1 are the terminals.
    nodes: Vec<(u32, u32, u32)>,
    unique: HashMap<(u32, u32, u32), u32>,
    cache: HashMap<(Op, u32, u32), u32>,
}

impl Bdd {
    fn new() -> Self {
        Bdd {
            nodes: vec![(u32::MAX, 0, 0), (u32::MAX, 1, 1)],
            unique: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    fn mk(&mut self, var: u32, lo: u32, hi: u32) -> u32 {
        if lo == hi {
            return lo;
        }
        if let Some(&n) = self.unique.get(&(var, lo, hi)) {
            return n;
        }
        let n = self.nodes.len() as u32;
        self.nodes.push((var, lo, hi));
        self.unique.insert((var, lo, hi), n);
        n
    }

    fn var(&mut self, v: usize) -> u32 {
        self.mk(v as u32, ZERO, ONE)
    }

    fn apply(&mut self, op: Op, a: u32, b: u32) -> u32 {
        match op {
            Op::And => {
                if a == ZERO || b == ZERO {
                    return ZERO;
                }
                if a == ONE {
                    return b;
                }
                if b == ONE || a == b {
                    return a;
                }
            }
            Op::Or => {
                if a == ONE || b == ONE {
                    return ONE;
                }
                if a == ZERO {
                    return b;
                }
                if b == ZERO || a == b {
                    return a;
                }
            }
            Op::Xor => {
                if a == b {
                    return ZERO;
                }
                if a == ZERO {
                    return b;
                }
                if b == ZERO {
                    return a;
                }
            }
        }
        let key = if a <= b { (op, a, b) } else { (op, b, a) };
        if let Some(&r) = self.cache.get(&key) {
            return r;
        }
        let (va, la, ha) = self.nodes[a as usize];
        let (vb, lb, hb) = self.nodes[b as usize];
        let v = va.min(vb);
        let (a0, a1) = if va == v { (la, ha) } else { (a, a) };
        let (b0, b1) = if vb == v { (lb, hb) } else { (b, b) };
        let lo = self.apply(op, a0, b0);
        let hi = self.apply(op, a1, b1);
        let r = self.mk(v, lo, hi);
        self.cache.insert(key, r);
        r
    }

    fn not(&mut self, a: u32) -> u32 {
        self.apply(Op::Xor, a, ONE)
    }

    /// Exact probability with per-variable probabilities `p`.
    fn prob(&self, n: u32, p: &[f64], memo: &mut HashMap<u32, f64>) -> f64 {
        if n == ZERO {
            return 0.0;
        }
        if n == ONE {
            return 1.0;
        }
        if let Some(&v) = memo.get(&n) {
            return v;
        }
        let (var, lo, hi) = self.nodes[n as usize];
        let pv = p[var as usize];
        let r = pv * self.prob(hi, p, memo) + (1.0 - pv) * self.prob(lo, p, memo);
        memo.insert(n, r);
        r
    }

    fn support(&self, n: u32, seen: &mut HashSet<u32>, out: &mut Vec<bool>) {
        if n <= ONE || !seen.insert(n) {
            return;
        }
        let (var, lo, hi) = self.nodes[n as usize];
        out[var as usize] = true;
        self.support(lo, seen, out);
        self.support(hi, seen, out);
    }

    /// Minimal true points (Rauzy's `minsol`, valid for non-coherent functions
    /// when read as "minimal TRUE-variable sets with the rest FALSE").
    fn min_sets(
        &self,
        n: u32,
        opts: &AnalysisOptions,
        memo: &mut HashMap<u32, Vec<Vec<u32>>>,
        truncated: &mut bool,
    ) -> Result<Vec<Vec<u32>>, FtaError> {
        if n == ZERO {
            return Ok(Vec::new());
        }
        if n == ONE {
            return Ok(vec![Vec::new()]);
        }
        if let Some(v) = memo.get(&n) {
            return Ok(v.clone());
        }
        let (var, lo, hi) = self.nodes[n as usize];
        let l = self.min_sets(lo, opts, memo, truncated)?;
        let h = self.min_sets(hi, opts, memo, truncated)?;
        let mut out = l.clone();
        for s in h {
            if l.iter().any(|x| is_subset(x, &s)) {
                continue;
            }
            if let Some(m) = opts.max_order {
                if s.len() + 1 > m {
                    *truncated = true;
                    continue;
                }
            }
            let mut t = Vec::with_capacity(s.len() + 1);
            t.push(var);
            t.extend_from_slice(&s);
            out.push(t);
        }
        if out.len() > opts.max_cut_sets {
            return Err(FtaError::TooManyCutSets(opts.max_cut_sets));
        }
        memo.insert(n, out.clone());
        Ok(out)
    }
}

/// `a ⊆ b` for ascending-sorted slices.
fn is_subset(a: &[u32], b: &[u32]) -> bool {
    let mut j = 0;
    for &x in a {
        while j < b.len() && b[j] < x {
            j += 1;
        }
        if j >= b.len() || b[j] != x {
            return false;
        }
        j += 1;
    }
    true
}

/// Find a node cycle reachable from `root`, as a list of node indices.
fn find_cycle(tree: &BoolTree) -> Option<Vec<usize>> {
    // 0 = unvisited, 1 = on stack, 2 = done
    let mut state = vec![0u8; tree.nodes.len()];
    let mut stack: Vec<usize> = Vec::new();
    fn dfs(t: &BoolTree, n: usize, state: &mut [u8], stack: &mut Vec<usize>) -> Option<Vec<usize>> {
        state[n] = 1;
        stack.push(n);
        if let Node::Gate { inputs, .. } = &t.nodes[n] {
            for &i in inputs {
                if state[i] == 1 {
                    let at = stack.iter().position(|&x| x == i).unwrap_or(0);
                    return Some(stack[at..].to_vec());
                }
                if state[i] == 0 {
                    if let Some(c) = dfs(t, i, state, stack) {
                        return Some(c);
                    }
                }
            }
        }
        stack.pop();
        state[n] = 2;
        None
    }
    dfs(tree, tree.root, &mut state, &mut stack)
}

/// Analyse a Boolean tree. `probs[v]` is variable `v`'s probability, when known.
pub fn analyze_bool(
    tree: &BoolTree,
    probs: &[Option<f64>],
    opts: &AnalysisOptions,
) -> Result<BoolAnalysis, FtaError> {
    if let Some(c) = find_cycle(tree) {
        return Err(FtaError::Cycle(c.iter().map(|i| format!("#{i}")).collect()));
    }
    let mut bdd = Bdd::new();
    let mut built: HashMap<usize, u32> = HashMap::new();
    let mut coherent = true;
    let top = build(tree, tree.root, &mut bdd, &mut built, &mut coherent);

    let mut support = vec![false; tree.num_vars];
    bdd.support(top, &mut HashSet::new(), &mut support);

    let mut truncated = false;
    let sets = bdd.min_sets(top, opts, &mut HashMap::new(), &mut truncated)?;

    let known = |v: usize| probs.get(v).copied().flatten();
    let all_known = (0..tree.num_vars).filter(|&v| support[v]).all(|v| known(v).is_some());
    let p_vec: Vec<f64> = (0..tree.num_vars).map(|v| known(v).unwrap_or(0.0)).collect();

    let mut cut_sets: Vec<VarCutSet> = sets
        .into_iter()
        .map(|s| {
            let vars: Vec<usize> = s.iter().map(|&v| v as usize).collect();
            let probability = vars
                .iter()
                .map(|&v| known(v))
                .try_fold(1.0, |acc, p| p.map(|p| acc * p));
            VarCutSet { vars, probability }
        })
        .collect();
    cut_sets.sort_by(|a, b| {
        a.vars
            .len()
            .cmp(&b.vars.len())
            .then_with(|| {
                b.probability
                    .unwrap_or(-1.0)
                    .partial_cmp(&a.probability.unwrap_or(-1.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.vars.cmp(&b.vars))
    });

    let mut importance = vec![VarImportance::default(); tree.num_vars];
    let mut top_probability = None;
    let mut rare_event = None;
    let mut min_cut_upper_bound = None;
    if all_known {
        let p_top = bdd.prob(top, &p_vec, &mut HashMap::new());
        top_probability = Some(p_top);
        if cut_sets.iter().all(|c| c.probability.is_some()) {
            rare_event = Some(cut_sets.iter().map(|c| c.probability.unwrap_or(0.0)).sum::<f64>().min(1.0));
            min_cut_upper_bound = Some(
                1.0 - cut_sets.iter().map(|c| 1.0 - c.probability.unwrap_or(0.0)).product::<f64>(),
            );
        }
        for v in 0..tree.num_vars {
            if !support[v] {
                continue;
            }
            let mut q = p_vec.clone();
            q[v] = 1.0;
            let p1 = bdd.prob(top, &q, &mut HashMap::new());
            q[v] = 0.0;
            let p0 = bdd.prob(top, &q, &mut HashMap::new());
            importance[v] = VarImportance {
                birnbaum: Some(p1 - p0),
                fussell_vesely: (p_top > 0.0).then(|| (p_top - p0) / p_top),
                raw: (p_top > 0.0).then(|| p1 / p_top),
                rrw: (p0 > 0.0).then(|| p_top / p0),
            };
        }
    }

    Ok(BoolAnalysis {
        cut_sets,
        top_probability,
        rare_event,
        min_cut_upper_bound,
        importance,
        support,
        coherent,
        truncated,
    })
}

fn build(
    t: &BoolTree,
    n: usize,
    bdd: &mut Bdd,
    built: &mut HashMap<usize, u32>,
    coherent: &mut bool,
) -> u32 {
    if let Some(&b) = built.get(&n) {
        return b;
    }
    let r = match &t.nodes[n] {
        Node::Const(true) => ONE,
        Node::Const(false) => ZERO,
        Node::Var(v) => bdd.var(*v),
        Node::Gate { kind, inputs } => {
            let ins: Vec<u32> = inputs.iter().map(|&i| build(t, i, bdd, built, coherent)).collect();
            match kind {
                GateKind::And | GateKind::Inhibit => {
                    ins.into_iter().fold(ONE, |a, b| bdd.apply(Op::And, a, b))
                }
                GateKind::Or => ins.into_iter().fold(ZERO, |a, b| bdd.apply(Op::Or, a, b)),
                GateKind::Xor => {
                    *coherent = false;
                    ins.into_iter().fold(ZERO, |a, b| bdd.apply(Op::Xor, a, b))
                }
                GateKind::Not => {
                    *coherent = false;
                    let o = ins.into_iter().fold(ZERO, |a, b| bdd.apply(Op::Or, a, b));
                    bdd.not(o)
                }
            }
        }
    };
    built.insert(n, r);
    r
}

// ── mission time / probability helpers ───────────────────────────────────────

/// Parse a `missionTime:` string (`"8760 h"`, `"1e9 h"`, `"10 y"`, `"365 d"`,
/// bare number = hours) into hours. `None` when unparsable or not positive.
pub fn parse_mission_time(s: &str) -> Option<f64> {
    let s = s.trim();
    let split = s
        .char_indices()
        .find(|&(i, c)| c.is_whitespace() || (c.is_alphabetic() && !(matches!(c, 'e' | 'E') && s[i + 1..].starts_with(|d: char| d.is_ascii_digit() || d == '-' || d == '+'))))
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    let value: f64 = s[..split].trim().parse().ok()?;
    let factor = match s[split..].trim().to_ascii_lowercase().as_str() {
        "" | "h" | "hr" | "hrs" | "hour" | "hours" => 1.0,
        "d" | "day" | "days" => 24.0,
        "y" | "yr" | "yrs" | "year" | "years" => 8760.0,
        _ => return None,
    };
    let h = value * factor;
    (h.is_finite() && h > 0.0).then_some(h)
}

/// Effective probability of an event: explicit valid `probability:`, else
/// `1 − e^(−λt)`.
pub fn event_probability(declared: Option<f64>, lambda: Option<f64>, mission_hours: Option<f64>) -> Option<f64> {
    if let Some(p) = declared {
        if p.is_finite() && (0.0..=1.0).contains(&p) {
            return Some(p);
        }
    }
    match (lambda, mission_hours) {
        (Some(l), Some(t)) if l.is_finite() && l >= 0.0 => Some(-(-l * t).exp_m1()),
        _ => None,
    }
}

// ── model structure ──────────────────────────────────────────────────────────

/// What a tree node is.
#[derive(Debug, Clone, PartialEq)]
pub enum FtKind {
    /// Gate with its parsed `gateType` (`None` when missing/invalid).
    Gate(Option<GateKind>),
    /// Event with its `eventKind` spelling.
    Event(String),
}

/// One gate or event of a fault tree.
#[derive(Debug, Clone)]
pub struct FtNode {
    pub id: String,
    pub qname: String,
    pub name: String,
    pub file_path: String,
    pub kind: FtKind,
    /// Indices into [`FtStructure::nodes`] (resolved `inputs:`).
    pub inputs: Vec<usize>,
    /// Number of `inputs:` entries written (incl. dangling ones).
    pub declared_inputs: usize,
    /// True when the node lives under the `FaultTree`'s own directory.
    pub in_tree: bool,
}

impl FtNode {
    pub fn is_gate(&self) -> bool {
        matches!(self.kind, FtKind::Gate(_))
    }
}

/// The resolved graph of one `FaultTree`.
#[derive(Debug, Clone)]
pub struct FtStructure {
    pub tree_id: String,
    pub tree_qname: String,
    pub tree_file: String,
    pub nodes: Vec<FtNode>,
    /// The chosen top node, if any.
    pub root: Option<usize>,
    /// Legacy flat form: a tree with events but **no gates** has no `root`;
    /// its events are an implicit OR (each alone causes the top event).
    pub flat: Vec<usize>,
    /// Indices of nodes reachable from `root` (or in `flat`), inclusive.
    pub reachable: BTreeSet<usize>,
    /// Gate cycles (each a list of node indices, entry order).
    pub cycles: Vec<Vec<usize>>,
}

fn is_ft_node(e: &RawElement) -> bool {
    matches!(
        e.frontmatter.element_type,
        Some(ElementType::FaultTreeGate) | Some(ElementType::FaultTreeEvent)
    )
}

fn node_label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

impl FtStructure {
    /// Build the graph of `tree` (a `FaultTree` element).
    pub fn build(elements: &[RawElement], resolver: &Resolver, tree: &RawElement) -> FtStructure {
        let prefix = format!("{}::", tree.qualified_name);
        let mut own: Vec<&RawElement> = elements
            .iter()
            .filter(|e| is_ft_node(e) && e.qualified_name.starts_with(&prefix))
            .collect();
        own.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));

        let mut nodes: Vec<FtNode> = Vec::new();
        let mut by_qname: HashMap<String, usize> = HashMap::new();
        let mut srcs: Vec<&RawElement> = Vec::new();
        for e in own {
            by_qname.insert(e.qualified_name.clone(), nodes.len());
            nodes.push(make_node(e, true));
            srcs.push(e);
        }
        // Resolve inputs, pulling in out-of-directory targets lazily.
        let mut i = 0;
        while i < srcs.len() {
            let e = srcs[i];
            let mut ins = Vec::new();
            for r in e.frontmatter.inputs.iter().flatten() {
                let Some(t) = resolver.resolve_ref(elements, r) else { continue };
                if !is_ft_node(t) {
                    continue;
                }
                let idx = *by_qname.entry(t.qualified_name.clone()).or_insert_with(|| {
                    nodes.push(make_node(t, false));
                    srcs.push(t);
                    nodes.len() - 1
                });
                if !ins.contains(&idx) {
                    ins.push(idx);
                }
            }
            nodes[i].inputs = ins;
            i += 1;
        }

        let mut s = FtStructure {
            tree_id: node_label(tree),
            tree_qname: tree.qualified_name.clone(),
            tree_file: tree.file_path.clone(),
            nodes,
            root: None,
            flat: Vec::new(),
            reachable: BTreeSet::new(),
            cycles: Vec::new(),
        };
        s.cycles = s.find_cycles();
        s.choose_root();
        s
    }

    fn reach_from(&self, start: usize) -> BTreeSet<usize> {
        let mut seen = BTreeSet::new();
        let mut stack = vec![start];
        while let Some(n) = stack.pop() {
            if seen.insert(n) {
                stack.extend(self.nodes[n].inputs.iter().copied());
            }
        }
        seen
    }

    fn choose_root(&mut self) {
        if !self.nodes.iter().any(|n| n.in_tree && n.is_gate()) {
            self.flat = (0..self.nodes.len()).filter(|&i| self.nodes[i].in_tree).collect();
            self.reachable = self.flat.iter().copied().collect();
            return;
        }
        let mut referenced = vec![false; self.nodes.len()];
        for n in self.nodes.iter().filter(|n| n.in_tree) {
            for &i in &n.inputs {
                referenced[i] = true;
            }
        }
        let mut best: Option<(usize, usize)> = None; // (reach size, idx)
        for (i, n) in self.nodes.iter().enumerate() {
            if !n.in_tree || referenced[i] {
                continue;
            }
            let size = self.reach_from(i).len();
            let better = match best {
                None => true,
                Some((bs, bi)) => {
                    size > bs
                        || (size == bs
                            && (n.is_gate(), std::cmp::Reverse(&n.id))
                                > (self.nodes[bi].is_gate(), std::cmp::Reverse(&self.nodes[bi].id)))
                }
            };
            if better {
                best = Some((size, i));
            }
        }
        if let Some((_, r)) = best {
            self.root = Some(r);
            self.reachable = self.reach_from(r);
        }
    }

    /// Strongly connected components with a cycle (Tarjan), as node-index lists.
    fn find_cycles(&self) -> Vec<Vec<usize>> {
        struct T<'a> {
            g: &'a FtStructure,
            index: Vec<Option<usize>>,
            low: Vec<usize>,
            on: Vec<bool>,
            stack: Vec<usize>,
            next: usize,
            out: Vec<Vec<usize>>,
        }
        fn go(t: &mut T, v: usize) {
            t.index[v] = Some(t.next);
            t.low[v] = t.next;
            t.next += 1;
            t.stack.push(v);
            t.on[v] = true;
            for k in 0..t.g.nodes[v].inputs.len() {
                let w = t.g.nodes[v].inputs[k];
                if t.index[w].is_none() {
                    go(t, w);
                    t.low[v] = t.low[v].min(t.low[w]);
                } else if t.on[w] {
                    t.low[v] = t.low[v].min(t.index[w].unwrap_or(0));
                }
            }
            if Some(t.low[v]) == t.index[v] {
                let mut comp = Vec::new();
                while let Some(w) = t.stack.pop() {
                    t.on[w] = false;
                    comp.push(w);
                    if w == v {
                        break;
                    }
                }
                if comp.len() > 1 || t.g.nodes[v].inputs.contains(&v) {
                    comp.sort_unstable();
                    t.out.push(comp);
                }
            }
        }
        let n = self.nodes.len();
        let mut t = T {
            g: self,
            index: vec![None; n],
            low: vec![0; n],
            on: vec![false; n],
            stack: Vec::new(),
            next: 0,
            out: Vec::new(),
        };
        for v in 0..n {
            if t.index[v].is_none() {
                go(&mut t, v);
            }
        }
        let mut out = t.out;
        out.sort();
        out
    }
}

fn make_node(e: &RawElement, in_tree: bool) -> FtNode {
    let fm = &e.frontmatter;
    let kind = if matches!(fm.element_type, Some(ElementType::FaultTreeGate)) {
        FtKind::Gate(fm.gate_type.as_deref().and_then(GateKind::parse))
    } else {
        FtKind::Event(fm.event_kind.clone().unwrap_or_else(|| "basic".to_string()))
    };
    FtNode {
        id: node_label(e),
        qname: e.qualified_name.clone(),
        name: fm.name.clone().unwrap_or_else(|| node_label(e)),
        file_path: e.file_path.clone(),
        kind,
        inputs: Vec::new(),
        declared_inputs: fm.inputs.as_ref().map_or(0, |v| v.len()),
        in_tree,
    }
}

/// All `FaultTree` elements of the model.
pub fn fault_trees(elements: &[RawElement]) -> impl Iterator<Item = &RawElement> {
    elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FaultTree)))
}

/// Find one `FaultTree` by stable id or qualified name.
pub fn find_fault_tree<'a>(elements: &'a [RawElement], id_or_qname: &str) -> Option<&'a RawElement> {
    fault_trees(elements)
        .find(|e| e.frontmatter.id.as_deref() == Some(id_or_qname) || e.qualified_name == id_or_qname)
}

// ── model analysis result ────────────────────────────────────────────────────

/// How an event relates to the top event — the per-event role a diagram can colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventRole {
    /// Member of an order-1 cut set (a single point of failure).
    SinglePoint,
    /// Smallest cut set containing it has order 2.
    DualPoint,
    /// Smallest cut set containing it has order ≥ 3.
    MultiPoint,
    /// Reachable, but in no cut set (e.g. masked by a TRUE/FALSE house event).
    Irrelevant,
    /// Not reachable from the top node.
    Unreachable,
    /// A `house` event (a constant, not a failure).
    House,
}

/// Where an analysed event comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "origin", rename_all = "snake_case")]
pub enum EventOrigin {
    /// A `FaultTreeEvent` of the model.
    Model,
    /// The synthetic common-cause event of a beta-factor group.
    Ccf { group: String },
}

/// Per-event analysis result (model events plus synthetic CCF events).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventResult {
    pub id: String,
    pub qname: String,
    pub name: String,
    pub file_path: String,
    pub kind: String,
    #[serde(flatten)]
    pub origin: EventOrigin,
    /// Declared `failureRate` (λ, /h).
    pub failure_rate: Option<f64>,
    /// λ after the beta-factor split (`(1−β)λ` for a CCF member; the CCF
    /// event's own `β·mean(λ)`); equals `failure_rate` otherwise.
    pub effective_failure_rate: Option<f64>,
    /// Effective probability used in the analysis.
    pub probability: Option<f64>,
    pub diagnostic_coverage: Option<f64>,
    pub latent_diagnostic_coverage: Option<f64>,
    pub role: EventRole,
    /// Order of the smallest cut set containing the event.
    pub min_cut_order: Option<usize>,
    pub cut_set_count: usize,
    pub birnbaum: Option<f64>,
    pub fussell_vesely: Option<f64>,
    pub raw: Option<f64>,
    pub rrw: Option<f64>,
}

/// One minimal cut set, by event id.
#[derive(Debug, Clone, Serialize)]
pub struct CutSetResult {
    pub order: usize,
    pub events: Vec<String>,
    pub probability: Option<f64>,
}

/// The complete analysis of one `FaultTree`.
#[derive(Debug, Clone, Serialize)]
pub struct FaultTreeAnalysis {
    pub tree_id: String,
    pub tree_qname: String,
    /// The `topEvent:` reference as written.
    pub top_event: Option<String>,
    /// Id of the top node the analysis started from.
    pub top_node: String,
    pub mission_time_hours: Option<f64>,
    pub coherent: bool,
    pub truncated: bool,
    pub cut_sets: Vec<CutSetResult>,
    pub top_probability: Option<f64>,
    pub rare_event_probability: Option<f64>,
    pub min_cut_upper_bound: Option<f64>,
    pub events: Vec<EventResult>,
    /// Non-fatal observations (unquantified events, ignored inputs, …).
    pub notes: Vec<String>,
}

impl FaultTreeAnalysis {
    /// Look up one event result by id.
    pub fn event(&self, id: &str) -> Option<&EventResult> {
        self.events.iter().find(|e| e.id == id)
    }
}

/// Analyse the fault tree `tree` (a `FaultTree` element).
pub fn analyze_fault_tree(
    elements: &[RawElement],
    resolver: &Resolver,
    tree: &RawElement,
    opts: &AnalysisOptions,
) -> Result<FaultTreeAnalysis, FtaError> {
    let st = FtStructure::build(elements, resolver, tree);
    if st.nodes.is_empty() {
        return Err(FtaError::EmptyTree);
    }
    if let Some(c) = st.cycles.first() {
        return Err(FtaError::Cycle(c.iter().map(|&i| st.nodes[i].id.clone()).collect()));
    }
    let tops: Vec<usize> = match st.root {
        Some(r) => vec![r],
        None if !st.flat.is_empty() => st.flat.clone(),
        None => return Err(FtaError::NoTopNode),
    };

    let src: HashMap<&str, &RawElement> = elements
        .iter()
        .filter(|e| is_ft_node(e))
        .map(|e| (e.qualified_name.as_str(), e))
        .collect();
    let mission = tree.frontmatter.mission_time.as_deref().and_then(parse_mission_time);
    let mut notes = Vec::new();
    if let Some(m) = tree.frontmatter.mission_time.as_deref() {
        if mission.is_none() {
            notes.push(format!("missionTime '{m}' is not a positive duration — λ-derived probabilities unavailable"));
        }
    }

    // ── variables (DFS order, reachable events only) ──
    struct Var {
        node: Option<usize>,
        id: String,
        origin: EventOrigin,
        lambda: Option<f64>,
        eff_lambda: Option<f64>,
        p: Option<f64>,
    }
    let mut vars: Vec<Var> = Vec::new();
    let mut var_of: HashMap<usize, usize> = HashMap::new();
    let mut order: Vec<usize> = Vec::new();
    {
        let mut seen = HashSet::new();
        fn dfs(st: &FtStructure, n: usize, seen: &mut HashSet<usize>, out: &mut Vec<usize>) {
            if !seen.insert(n) {
                return;
            }
            out.push(n);
            for &i in &st.nodes[n].inputs {
                dfs(st, i, seen, out);
            }
        }
        for &t in &tops {
            dfs(&st, t, &mut seen, &mut order);
        }
    }
    let mut house: HashMap<usize, bool> = HashMap::new();
    for &n in &order {
        let node = &st.nodes[n];
        let FtKind::Event(ref kind) = node.kind else { continue };
        let fm = &src[node.qname.as_str()].frontmatter;
        if kind == "house" {
            house.insert(n, fm.probability == Some(1.0));
            continue;
        }
        let lam = fm.failure_rate.filter(|l| l.is_finite() && *l >= 0.0);
        let p = event_probability(fm.probability, lam, mission);
        var_of.insert(n, vars.len());
        vars.push(Var { node: Some(n), id: node.id.clone(), origin: EventOrigin::Model, lambda: lam, eff_lambda: lam, p });
    }

    // ── beta-factor CCF expansion ──
    let mut ccf_var_of_member: HashMap<usize, usize> = HashMap::new(); // member var -> ccf var
    if opts.ccf {
        let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (vi, v) in vars.iter().enumerate() {
            let Some(n) = v.node else { continue };
            if let Some(g) = src[st.nodes[n].qname.as_str()].frontmatter.ccf_group.as_deref() {
                groups.entry(g.to_string()).or_default().push(vi);
            }
        }
        for (g, members) in groups {
            if members.len() < 2 {
                continue;
            }
            let beta = members
                .iter()
                .filter_map(|&vi| vars[vi].node)
                .filter_map(|n| src[st.nodes[n].qname.as_str()].frontmatter.ccf_beta)
                .filter(|b| b.is_finite() && (0.0..=1.0).contains(b))
                .fold(None, |a: Option<f64>, b| Some(a.map_or(b, |x| x.max(b))));
            let Some(beta) = beta else {
                notes.push(format!("ccfGroup '{g}' declares no valid ccfBeta — group not expanded"));
                continue;
            };
            let mean = |f: &dyn Fn(&Var) -> Option<f64>| -> Option<f64> {
                let xs: Option<Vec<f64>> = members.iter().map(|&m| f(&vars[m])).collect();
                xs.map(|x| x.iter().sum::<f64>() / x.len() as f64)
            };
            let ccf_p = mean(&|v| v.p).map(|m| beta * m);
            let ccf_l = mean(&|v| v.lambda).map(|m| beta * m);
            for &m in &members {
                vars[m].p = vars[m].p.map(|p| (1.0 - beta) * p);
                vars[m].eff_lambda = vars[m].lambda.map(|l| (1.0 - beta) * l);
            }
            let cv = vars.len();
            vars.push(Var {
                node: None,
                id: format!("CCF:{g}"),
                origin: EventOrigin::Ccf { group: g },
                lambda: ccf_l,
                eff_lambda: ccf_l,
                p: ccf_p,
            });
            for &m in &members {
                ccf_var_of_member.insert(m, cv);
            }
        }
    }

    // ── Boolean tree ──
    let mut nodes: Vec<Node> = Vec::new();
    let mut idx_of: HashMap<usize, usize> = HashMap::new();
    fn lower(
        st: &FtStructure,
        n: usize,
        nodes: &mut Vec<Node>,
        idx_of: &mut HashMap<usize, usize>,
        var_of: &HashMap<usize, usize>,
        house: &HashMap<usize, bool>,
        ccf: &HashMap<usize, usize>,
        notes: &mut Vec<String>,
    ) -> usize {
        if let Some(&i) = idx_of.get(&n) {
            return i;
        }
        let node = &st.nodes[n];
        let built = match &node.kind {
            FtKind::Event(_) => {
                if let Some(&h) = house.get(&n) {
                    Node::Const(h)
                } else {
                    let v = var_of[&n];
                    if let Some(&c) = ccf.get(&v) {
                        let a = nodes.len();
                        nodes.push(Node::Var(v));
                        let b = nodes.len();
                        nodes.push(Node::Var(c));
                        Node::Gate { kind: GateKind::Or, inputs: vec![a, b] }
                    } else {
                        Node::Var(v)
                    }
                }
            }
            FtKind::Gate(k) => {
                if node.inputs.is_empty() {
                    Node::Const(false)
                } else {
                    let kind = k.unwrap_or_else(|| {
                        notes.push(format!("gate {} has no valid gateType — treated as OR", node.id));
                        GateKind::Or
                    });
                    let ins = node
                        .inputs
                        .iter()
                        .map(|&i| lower(st, i, nodes, idx_of, var_of, house, ccf, notes))
                        .collect();
                    Node::Gate { kind, inputs: ins }
                }
            }
        };
        let i = nodes.len();
        nodes.push(built);
        idx_of.insert(n, i);
        i
    }
    let mut top_idx: Vec<usize> = Vec::new();
    for &t in &tops {
        top_idx.push(lower(&st, t, &mut nodes, &mut idx_of, &var_of, &house, &ccf_var_of_member, &mut notes));
    }
    let root_idx = if top_idx.len() == 1 {
        top_idx[0]
    } else {
        notes.push("tree has no gates — its events are treated as an implicit OR".to_string());
        nodes.push(Node::Gate { kind: GateKind::Or, inputs: top_idx });
        nodes.len() - 1
    };
    let bt = BoolTree { nodes, root: root_idx, num_vars: vars.len() };
    let probs: Vec<Option<f64>> = vars.iter().map(|v| v.p).collect();
    let a = analyze_bool(&bt, &probs, opts)?;

    // ── map back ──
    let mut min_order = vec![None::<usize>; vars.len()];
    let mut count = vec![0usize; vars.len()];
    for c in &a.cut_sets {
        for &v in &c.vars {
            count[v] += 1;
            min_order[v] = Some(min_order[v].map_or(c.vars.len(), |o| o.min(c.vars.len())));
        }
    }
    let mut events: Vec<EventResult> = Vec::new();
    // every FaultTreeEvent of the tree, reachable or not
    for (ni, node) in st.nodes.iter().enumerate() {
        let FtKind::Event(ref kind) = node.kind else { continue };
        if !node.in_tree {
            continue;
        }
        let fm = &src[node.qname.as_str()].frontmatter;
        let vi = var_of.get(&ni).copied();
        let role = if kind == "house" {
            EventRole::House
        } else if !st.reachable.contains(&ni) {
            EventRole::Unreachable
        } else {
            role_of(vi.and_then(|v| min_order[v]))
        };
        let imp = vi.map(|v| a.importance[v]).unwrap_or_default();
        let lam = fm.failure_rate;
        events.push(EventResult {
            id: node.id.clone(),
            qname: node.qname.clone(),
            name: node.name.clone(),
            file_path: node.file_path.clone(),
            kind: kind.clone(),
            origin: EventOrigin::Model,
            failure_rate: lam,
            effective_failure_rate: vi.map(|v| vars[v].eff_lambda).unwrap_or(lam),
            probability: vi.map(|v| vars[v].p).unwrap_or_else(|| event_probability(fm.probability, lam, mission)),
            diagnostic_coverage: fm.diagnostic_coverage,
            latent_diagnostic_coverage: fm.latent_diagnostic_coverage,
            role,
            min_cut_order: vi.and_then(|v| min_order[v]),
            cut_set_count: vi.map_or(0, |v| count[v]),
            birnbaum: imp.birnbaum,
            fussell_vesely: imp.fussell_vesely,
            raw: imp.raw,
            rrw: imp.rrw,
        });
    }
    for (vi, v) in vars.iter().enumerate() {
        let EventOrigin::Ccf { ref group } = v.origin else { continue };
        let imp = a.importance[vi];
        events.push(EventResult {
            id: v.id.clone(),
            qname: format!("{}::{}", st.tree_qname, v.id),
            name: format!("Common-cause failure of group {group}"),
            file_path: st.tree_file.clone(),
            kind: "basic".to_string(),
            origin: v.origin.clone(),
            failure_rate: v.lambda,
            effective_failure_rate: v.eff_lambda,
            probability: v.p,
            diagnostic_coverage: None,
            latent_diagnostic_coverage: None,
            role: role_of(min_order[vi]),
            min_cut_order: min_order[vi],
            cut_set_count: count[vi],
            birnbaum: imp.birnbaum,
            fussell_vesely: imp.fussell_vesely,
            raw: imp.raw,
            rrw: imp.rrw,
        });
    }
    events.sort_by(|a, b| a.id.cmp(&b.id));

    if a.top_probability.is_none() {
        let un: Vec<&str> = vars
            .iter()
            .enumerate()
            .filter(|(i, v)| a.support[*i] && v.p.is_none())
            .map(|(_, v)| v.id.as_str())
            .collect();
        notes.push(format!(
            "top-event probability unavailable — no probability/failureRate(+missionTime) for: {}",
            un.join(", ")
        ));
    }
    if !a.coherent {
        notes.push("tree contains NOT/XOR gates (non-coherent): cut sets are minimal true points with all other events FALSE".to_string());
    }
    if a.truncated {
        notes.push(format!(
            "cut sets above order {} were discarded (--max-order); probabilities are exact",
            opts.max_order.unwrap_or(0)
        ));
    }

    let cut_sets = a
        .cut_sets
        .iter()
        .map(|c| CutSetResult {
            order: c.vars.len(),
            events: c.vars.iter().map(|&v| vars[v].id.clone()).collect(),
            probability: c.probability,
        })
        .collect();

    Ok(FaultTreeAnalysis {
        tree_id: st.tree_id.clone(),
        tree_qname: st.tree_qname.clone(),
        top_event: tree.frontmatter.top_event.clone(),
        top_node: if tops.len() == 1 {
            st.nodes[tops[0]].id.clone()
        } else {
            format!("(implicit OR of {} events)", tops.len())
        },
        mission_time_hours: mission,
        coherent: a.coherent,
        truncated: a.truncated,
        cut_sets,
        top_probability: a.top_probability,
        rare_event_probability: a.rare_event,
        min_cut_upper_bound: a.min_cut_upper_bound,
        events,
        notes,
    })
}

fn role_of(min_order: Option<usize>) -> EventRole {
    match min_order {
        Some(1) => EventRole::SinglePoint,
        Some(2) => EventRole::DualPoint,
        Some(_) => EventRole::MultiPoint,
        None => EventRole::Irrelevant,
    }
}

// ── structural validation (GH #212) ──────────────────────────────────────────

/// One structural finding, converted to a validator `Finding` by the caller.
#[derive(Debug, Clone, PartialEq)]
pub struct StructuralIssue {
    pub code: &'static str,
    pub file_path: String,
    pub message: String,
    pub is_error: bool,
}

fn issue(code: &'static str, file: &str, is_error: bool, message: String) -> StructuralIssue {
    StructuralIssue { code, file_path: file.to_string(), message, is_error }
}

/// Structural checks of every fault tree and fault-tree node in the model:
/// gate cycles `E960`, gate arity `E961`, `failureRate` `E962` / probability
/// `E963` / `ccfBeta` `E964` ranges, unreachable nodes `W960`, stray nodes
/// outside any `FaultTree` directory `W961`, degenerate single-input AND/OR
/// `W962`, single-member CCF group `W963`, inconsistent betas `W964`, and an
/// unparsable `missionTime` `W967`.
pub fn structural_issues(elements: &[RawElement], resolver: &Resolver) -> Vec<StructuralIssue> {
    let mut out = Vec::new();
    let tree_prefixes: Vec<String> = fault_trees(elements).map(|t| format!("{}::", t.qualified_name)).collect();

    for e in elements.iter().filter(|e| is_ft_node(e) || matches!(e.frontmatter.element_type, Some(ElementType::FaultTree))) {
        let fm = &e.frontmatter;
        let label = node_label(e);
        let file = &e.file_path;
        // ranges
        if matches!(fm.element_type, Some(ElementType::FaultTreeEvent)) {
            if let Some(l) = fm.failure_rate {
                if !l.is_finite() || l < 0.0 {
                    out.push(issue("E962", file, true, format!("FaultTreeEvent '{label}' failureRate {l} must be a finite value ≥ 0")));
                }
            }
            if let Some(b) = fm.ccf_beta {
                if !b.is_finite() || !(0.0..=1.0).contains(&b) {
                    out.push(issue("E964", file, true, format!("FaultTreeEvent '{label}' ccfBeta {b} is out of range 0.0–1.0")));
                }
            }
        }
        if let Some(p) = fm.probability {
            if !p.is_finite() || !(0.0..=1.0).contains(&p) {
                out.push(issue("E963", file, true, format!("'{label}' probability {p} is out of range 0.0–1.0")));
            }
        }
        // stray
        if is_ft_node(e) && !tree_prefixes.iter().any(|p| e.qualified_name.starts_with(p)) {
            out.push(issue("W961", file, false, format!(
                "{} '{label}' is not inside any FaultTree directory — it is ignored by fault-tree analysis and metrics",
                fm.element_type.as_ref().map_or("fault-tree node", |t| t.name())
            )));
        }
        // gate arity
        if matches!(fm.element_type, Some(ElementType::FaultTreeGate)) {
            let n = fm.inputs.as_ref().map_or(0, |v| v.len());
            match fm.gate_type.as_deref().and_then(GateKind::parse) {
                Some(GateKind::Not) if n > 1 => out.push(issue("E961", file, true, format!("NOT gate '{label}' has {n} inputs — it takes exactly 1"))),
                Some(GateKind::Xor) if n > 2 => out.push(issue("E961", file, true, format!("XOR gate '{label}' has {n} inputs — it takes exactly 2"))),
                Some(GateKind::Inhibit) if n >= 1 && n < 2 => out.push(issue("E961", file, true, format!(
                    "inhibit gate '{label}' has no conditioning input — it needs the input event plus its condition (≥ 2 inputs)"))),
                Some(GateKind::And) | Some(GateKind::Or) if n == 1 => out.push(issue("W962", file, false, format!(
                    "{} gate '{label}' has a single input — it is a pass-through", fm.gate_type.as_deref().unwrap_or("")))),
                _ => {}
            }
            // self-input (also covered by cycles, but name it directly)
            for r in fm.inputs.iter().flatten() {
                if resolver.resolve_ref(elements, r).is_some_and(|t| t.qualified_name == e.qualified_name) {
                    out.push(issue("E960", file, true, format!("gate '{label}' lists itself in `inputs`")));
                }
            }
        }
    }

    // CCF groups per tree
    for t in fault_trees(elements) {
        let prefix = format!("{}::", t.qualified_name);
        let mut groups: BTreeMap<&str, Vec<&RawElement>> = BTreeMap::new();
        for e in elements.iter().filter(|e| {
            matches!(e.frontmatter.element_type, Some(ElementType::FaultTreeEvent)) && e.qualified_name.starts_with(&prefix)
        }) {
            if let Some(g) = e.frontmatter.ccf_group.as_deref() {
                groups.entry(g).or_default().push(e);
            }
        }
        for (g, ms) in groups {
            if ms.len() < 2 {
                out.push(issue("W963", &ms[0].file_path, false, format!("ccfGroup '{g}' has a single member — a common-cause group needs ≥ 2 events")));
                continue;
            }
            let betas: Vec<f64> = ms.iter().filter_map(|m| m.frontmatter.ccf_beta).collect();
            let consistent = betas.windows(2).all(|w| (w[0] - w[1]).abs() < 1e-12);
            if betas.is_empty() {
                out.push(issue("W964", &ms[0].file_path, false, format!("ccfGroup '{g}' declares no ccfBeta — the group is not expanded")));
            } else if !consistent || betas.len() != ms.len() {
                out.push(issue("W964", &ms[0].file_path, false, format!("ccfGroup '{g}' members disagree on ccfBeta (or some omit it) — the largest value is used")));
            }
        }
    }

    // per-tree graph checks
    for t in fault_trees(elements) {
        if let Some(m) = t.frontmatter.mission_time.as_deref() {
            if parse_mission_time(m).is_none() {
                out.push(issue("W967", &t.file_path, false, format!("FaultTree '{}' missionTime '{m}' is not a positive duration (e.g. \"8760 h\")", node_label(t))));
            }
        }
        let st = FtStructure::build(elements, resolver, t);
        for c in &st.cycles {
            let first = &st.nodes[c[0]];
            let ids: Vec<&str> = c.iter().map(|&i| st.nodes[i].id.as_str()).collect();
            out.push(issue("E960", &first.file_path, true, format!(
                "FaultTree '{}' has a gate cycle: {}", st.tree_id, ids.join(" -> "))));
        }
        if st.root.is_some() {
            for (i, n) in st.nodes.iter().enumerate() {
                if n.in_tree && !st.reachable.contains(&i) {
                    out.push(issue("W960", &n.file_path, false, format!(
                        "{} '{}' is not reachable from the top node '{}' of FaultTree '{}' — it does not affect the analysis",
                        if n.is_gate() { "gate" } else { "event" }, n.id, st.root.map_or("?", |r| st.nodes[r].id.as_str()), st.tree_id)));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Deterministic dependency-free PRNG (xorshift64).
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
        fn unit(&mut self) -> f64 {
            (self.next() % 10_000) as f64 / 10_000.0
        }
    }

    fn eval(t: &BoolTree, n: usize, a: &[bool]) -> bool {
        match &t.nodes[n] {
            Node::Const(b) => *b,
            Node::Var(v) => a[*v],
            Node::Gate { kind, inputs } => {
                let vs: Vec<bool> = inputs.iter().map(|&i| eval(t, i, a)).collect();
                match kind {
                    GateKind::And | GateKind::Inhibit => vs.iter().all(|&b| b),
                    GateKind::Or => vs.iter().any(|&b| b),
                    GateKind::Xor => vs.iter().fold(false, |x, &b| x ^ b),
                    GateKind::Not => !vs.iter().any(|&b| b),
                }
            }
        }
    }

    fn assignment(bits: usize, n: usize) -> Vec<bool> {
        (0..n).map(|i| bits >> i & 1 == 1).collect()
    }

    /// Random DAG: leaves are vars/consts, gates draw from earlier nodes.
    fn random_tree(rng: &mut Rng, nvars: usize, ngates: usize, coherent_only: bool) -> BoolTree {
        let mut nodes: Vec<Node> = (0..nvars).map(Node::Var).collect();
        if rng.below(6) == 0 {
            nodes.push(Node::Const(rng.below(2) == 0));
        }
        for _ in 0..ngates {
            let len = nodes.len();
            let kinds: &[GateKind] = if coherent_only {
                &[GateKind::And, GateKind::Or, GateKind::Inhibit]
            } else {
                &[GateKind::And, GateKind::Or, GateKind::Xor, GateKind::Not, GateKind::Inhibit]
            };
            let kind = kinds[rng.below(kinds.len())];
            let arity = match kind {
                GateKind::Not => 1,
                GateKind::Xor => 2,
                _ => 2 + rng.below(2),
            };
            let inputs: Vec<usize> = (0..arity).map(|_| rng.below(len)).collect();
            nodes.push(Node::Gate { kind, inputs });
        }
        let root = nodes.len() - 1;
        BoolTree { nodes, root, num_vars: nvars }
    }

    /// Exhaustive oracle: minimal true points, exact probability, importance.
    fn oracle(t: &BoolTree, p: &[f64]) -> (Vec<Vec<usize>>, f64) {
        let n = t.num_vars;
        let truth: Vec<bool> = (0..1usize << n).map(|b| eval(t, t.root, &assignment(b, n))).collect();
        let mut min: Vec<usize> = Vec::new();
        for b in 0..1usize << n {
            if truth[b] && !(0..1usize << n).any(|c| c != b && c & b == c && truth[c]) {
                min.push(b);
            }
        }
        let mut sets: Vec<Vec<usize>> = min.iter().map(|&b| (0..n).filter(|&i| b >> i & 1 == 1).collect()).collect();
        sets.sort();
        let mut prob = 0.0;
        for b in 0..1usize << n {
            if truth[b] {
                prob += (0..n).map(|i| if b >> i & 1 == 1 { p[i] } else { 1.0 - p[i] }).product::<f64>();
            }
        }
        (sets, prob)
    }

    fn oracle_prob(t: &BoolTree, p: &[f64]) -> f64 {
        oracle(t, p).1
    }

    #[test]
    fn random_trees_match_truth_table_oracle() {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        for round in 0..400 {
            let nvars = 2 + rng.below(6);
            let ngates = 1 + rng.below(9);
            let coherent_only = round % 2 == 0;
            let t = random_tree(&mut rng, nvars, ngates, coherent_only);
            let p: Vec<f64> = (0..nvars).map(|_| rng.unit()).collect();
            let probs: Vec<Option<f64>> = p.iter().map(|&x| Some(x)).collect();
            let a = analyze_bool(&t, &probs, &AnalysisOptions::default()).unwrap();
            let (sets, prob) = oracle(&t, &p);

            let mut got: Vec<Vec<usize>> = a.cut_sets.iter().map(|c| c.vars.clone()).collect();
            got.sort();
            assert_eq!(got, sets, "round {round}: cut sets differ for {t:?}");
            assert!(
                (a.top_probability.unwrap() - prob).abs() < 1e-9,
                "round {round}: P(top) {} vs oracle {prob}",
                a.top_probability.unwrap()
            );
            if coherent_only {
                assert!(a.coherent || !reachable_nonmono(&t));
            }
            // importance vs enumeration for every supported variable
            for v in 0..nvars {
                if !a.support[v] {
                    continue;
                }
                let mut q = p.clone();
                q[v] = 1.0;
                let p1 = oracle_prob(&t, &q);
                q[v] = 0.0;
                let p0 = oracle_prob(&t, &q);
                let imp = a.importance[v];
                assert!((imp.birnbaum.unwrap() - (p1 - p0)).abs() < 1e-9, "round {round}: Birnbaum x{v}");
                if prob > 0.0 {
                    assert!((imp.fussell_vesely.unwrap() - (prob - p0) / prob).abs() < 1e-9, "round {round}: FV x{v}");
                    assert!((imp.raw.unwrap() - p1 / prob).abs() < 1e-9, "round {round}: RAW x{v}");
                }
            }
            // rare-event >= exact for coherent trees, MCUB between
            if coherent_only {
                assert!(a.rare_event.unwrap() + 1e-12 >= prob.min(1.0) - 1e-9);
                assert!(a.min_cut_upper_bound.unwrap() + 1e-9 >= prob);
            }
        }
    }

    fn reachable_nonmono(t: &BoolTree) -> bool {
        t.nodes.iter().any(|n| matches!(n, Node::Gate { kind: GateKind::Xor | GateKind::Not, .. }))
    }

    #[test]
    fn and_vs_or_have_different_cut_sets() {
        // AND(a,b) -> one order-2 set; OR(a,b) -> two order-1 sets.
        let mk = |kind| BoolTree {
            nodes: vec![Node::Var(0), Node::Var(1), Node::Gate { kind, inputs: vec![0, 1] }],
            root: 2,
            num_vars: 2,
        };
        let p = [Some(0.1), Some(0.2)];
        let and = analyze_bool(&mk(GateKind::And), &p, &AnalysisOptions::default()).unwrap();
        let or = analyze_bool(&mk(GateKind::Or), &p, &AnalysisOptions::default()).unwrap();
        assert_eq!(and.cut_sets.len(), 1);
        assert_eq!(and.cut_sets[0].vars, vec![0, 1]);
        assert!((and.top_probability.unwrap() - 0.02).abs() < 1e-12);
        assert_eq!(or.cut_sets.len(), 2);
        assert!((or.top_probability.unwrap() - 0.28).abs() < 1e-12);
    }

    #[test]
    fn shared_event_absorption() {
        // OR(a, AND(a,b)) -> {a} only.
        let t = BoolTree {
            nodes: vec![
                Node::Var(0),
                Node::Var(1),
                Node::Gate { kind: GateKind::And, inputs: vec![0, 1] },
                Node::Gate { kind: GateKind::Or, inputs: vec![0, 2] },
            ],
            root: 3,
            num_vars: 2,
        };
        let a = analyze_bool(&t, &[Some(0.5), Some(0.5)], &AnalysisOptions::default()).unwrap();
        assert_eq!(a.cut_sets.iter().map(|c| c.vars.clone()).collect::<Vec<_>>(), vec![vec![0]]);
    }

    #[test]
    fn cycle_is_an_error_not_a_panic() {
        let t = BoolTree {
            nodes: vec![
                Node::Var(0),
                Node::Gate { kind: GateKind::Or, inputs: vec![0, 2] },
                Node::Gate { kind: GateKind::Or, inputs: vec![1] },
            ],
            root: 1,
            num_vars: 1,
        };
        assert!(matches!(analyze_bool(&t, &[Some(0.1)], &AnalysisOptions::default()), Err(FtaError::Cycle(_))));
    }

    #[test]
    fn cut_set_limit_and_order_cutoff() {
        // AND of 3 ORs of 2 -> 8 cut sets of order 3.
        let mut nodes: Vec<Node> = (0..6).map(Node::Var).collect();
        for i in 0..3 {
            nodes.push(Node::Gate { kind: GateKind::Or, inputs: vec![2 * i, 2 * i + 1] });
        }
        nodes.push(Node::Gate { kind: GateKind::And, inputs: vec![6, 7, 8] });
        let t = BoolTree { nodes, root: 9, num_vars: 6 };
        let p = vec![Some(0.1); 6];
        let full = analyze_bool(&t, &p, &AnalysisOptions::default()).unwrap();
        assert_eq!(full.cut_sets.len(), 8);
        let limited = AnalysisOptions { max_cut_sets: 4, ..Default::default() };
        assert!(matches!(analyze_bool(&t, &p, &limited), Err(FtaError::TooManyCutSets(4))));
        let cut = AnalysisOptions { max_order: Some(2), ..Default::default() };
        let a = analyze_bool(&t, &p, &cut).unwrap();
        assert!(a.cut_sets.is_empty() && a.truncated);
        // probability stays exact under a cut-off
        assert!((a.top_probability.unwrap() - full.top_probability.unwrap()).abs() < 1e-15);
    }

    #[test]
    fn unknown_probabilities_give_none_but_cut_sets_remain() {
        let t = BoolTree {
            nodes: vec![Node::Var(0), Node::Var(1), Node::Gate { kind: GateKind::Or, inputs: vec![0, 1] }],
            root: 2,
            num_vars: 2,
        };
        let a = analyze_bool(&t, &[Some(0.1), None], &AnalysisOptions::default()).unwrap();
        assert_eq!(a.cut_sets.len(), 2);
        assert_eq!(a.top_probability, None);
        assert_eq!(a.importance[0].birnbaum, None);
    }

    #[test]
    fn mission_time_parsing() {
        assert_eq!(parse_mission_time("8760 h"), Some(8760.0));
        assert_eq!(parse_mission_time("1e9 h"), Some(1e9));
        assert_eq!(parse_mission_time("1e9"), Some(1e9));
        assert_eq!(parse_mission_time("2 y"), Some(17520.0));
        assert_eq!(parse_mission_time("10d"), Some(240.0));
        assert_eq!(parse_mission_time("soon"), None);
        assert_eq!(parse_mission_time("-5 h"), None);
        assert_eq!(parse_mission_time("0 h"), None);
    }

    #[test]
    fn event_probability_rules() {
        assert_eq!(event_probability(Some(0.3), Some(1.0), Some(1.0)), Some(0.3));
        let p = event_probability(None, Some(1e-6), Some(1000.0)).unwrap();
        assert!((p - (1.0 - (-1e-3f64).exp())).abs() < 1e-15);
        assert_eq!(event_probability(None, Some(1e-6), None), None);
        assert_eq!(event_probability(Some(2.0), None, None), None);
        assert_eq!(event_probability(None, Some(-1.0), Some(10.0)), None);
    }

    // ── model adapter ──

    use crate::element::{ParseIssue, RawFrontmatter};

    fn el(qname: &str, yaml: &str) -> RawElement {
        RawElement {
            qualified_name: qname.to_string(),
            file_path: format!("{}.md", qname.replace("::", "/")),
            frontmatter: serde_yaml::from_str::<RawFrontmatter>(yaml).expect("yaml"),
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
            about_notes: Default::default(),
        }
    }

    fn ev(n: &str, extra: &str) -> RawElement {
        el(
            &format!("S::FT-X::FTE-XX-{n}"),
            &format!("type: FaultTreeEvent\nid: FTE-XX-{n}\nname: e{n}\neventKind: basic\n{extra}"),
        )
    }

    fn gate(n: &str, kind: &str, inputs: &[&str]) -> RawElement {
        let ins: String = inputs.iter().map(|i| format!("  - {i}\n")).collect();
        el(
            &format!("S::FT-X::FTG-XX-{n}"),
            &format!("type: FaultTreeGate\nid: FTG-XX-{n}\nname: g{n}\ngateType: {kind}\ninputs:\n{ins}"),
        )
    }

    fn tree(extra: &str) -> RawElement {
        el(
            "S::FT-X",
            &format!("type: FaultTree\nid: FT-X\nname: t\nstatus: draft\ntopEvent: SG-X\n{extra}"),
        )
    }

    fn run(els: Vec<RawElement>) -> Result<FaultTreeAnalysis, FtaError> {
        let r = Resolver::new(&els);
        let t = find_fault_tree(&els, "FT-X").unwrap();
        analyze_fault_tree(&els, &r, t, &AnalysisOptions::default())
    }

    #[test]
    fn adapter_gate_logic_changes_cut_sets() {
        let base = |kind: &str| {
            vec![
                tree("missionTime: \"1000 h\"\n"),
                ev("001", "failureRate: 1.0e-4\n"),
                ev("002", "failureRate: 2.0e-4\n"),
                gate("001", kind, &["FTE-XX-001", "FTE-XX-002"]),
            ]
        };
        let and = run(base("AND")).unwrap();
        let or = run(base("OR")).unwrap();
        assert_eq!(and.cut_sets.len(), 1);
        assert_eq!(and.cut_sets[0].order, 2);
        assert_eq!(or.cut_sets.len(), 2);
        assert!(and.top_probability.unwrap() < or.top_probability.unwrap());
        assert_eq!(and.event("FTE-XX-001").unwrap().role, EventRole::DualPoint);
        assert_eq!(or.event("FTE-XX-001").unwrap().role, EventRole::SinglePoint);
        let p1 = 1.0 - (-0.1f64).exp();
        assert!((or.event("FTE-XX-001").unwrap().probability.unwrap() - p1).abs() < 1e-12);
    }

    #[test]
    fn adapter_house_unreachable_and_unquantified() {
        let a = run(vec![
            tree(""),
            ev("001", "probability: 0.1\n"),
            el("S::FT-X::FTE-XX-H", "type: FaultTreeEvent\nid: FTE-XX-H\nname: h\neventKind: house\nprobability: 0.0\n"),
            ev("003", "probability: 0.5\n"),
            gate("001", "OR", &["FTE-XX-001"]),
            gate("002", "AND", &["FTG-XX-001", "FTE-XX-H"]),
            gate("003", "OR", &["FTG-XX-002", "FTE-XX-001"]),
        ])
        .unwrap();
        // top = FTG-XX-003 (largest reach); event 003 is a second, smaller root.
        assert_eq!(a.top_node, "FTG-XX-003");
        assert_eq!(a.event("FTE-XX-H").unwrap().role, EventRole::House);
        assert_eq!(a.event("FTE-XX-003").unwrap().role, EventRole::Unreachable);
        assert_eq!(a.cut_sets.len(), 1);

        let b = run(vec![
            tree(""),
            ev("001", ""),
            ev("002", "probability: 0.5\n"),
            gate("001", "OR", &["FTE-XX-001", "FTE-XX-002"]),
        ])
        .unwrap();
        assert_eq!(b.cut_sets.len(), 2);
        assert_eq!(b.top_probability, None);
        assert!(b.notes.iter().any(|n| n.contains("FTE-XX-001")));
    }

    #[test]
    fn adapter_beta_factor_ccf() {
        let g = "failureRate: 1.0e-3\nprobability: 0.01\nccfGroup: PAIR\nccfBeta: 0.1\n";
        let els = vec![
            tree(""),
            ev("001", g),
            ev("002", g),
            gate("001", "AND", &["FTE-XX-001", "FTE-XX-002"]),
        ];
        let a = run(els.clone()).unwrap();
        // {CCF} order 1, {e1,e2} order 2
        assert_eq!(a.cut_sets[0].events, vec!["CCF:PAIR"]);
        assert!((a.cut_sets[0].probability.unwrap() - 0.001).abs() < 1e-12);
        assert_eq!(a.cut_sets[1].order, 2);
        assert!((a.cut_sets[1].probability.unwrap() - 0.009 * 0.009).abs() < 1e-12);
        assert_eq!(a.event("CCF:PAIR").unwrap().role, EventRole::SinglePoint);
        assert!((a.event("FTE-XX-001").unwrap().effective_failure_rate.unwrap() - 0.9e-3).abs() < 1e-15);
        let exact = 1.0 - (1.0 - 0.001) * (1.0 - 0.000081);
        assert!((a.top_probability.unwrap() - exact).abs() < 1e-12);

        let r = Resolver::new(&els);
        let t = find_fault_tree(&els, "FT-X").unwrap();
        let off = analyze_fault_tree(&els, &r, t, &AnalysisOptions { ccf: false, ..Default::default() }).unwrap();
        assert_eq!(off.cut_sets.len(), 1);
    }

    #[test]
    fn adapter_cycle_and_empty_are_errors() {
        let els = vec![
            tree(""),
            ev("001", "probability: 0.1\n"),
            gate("001", "OR", &["FTG-XX-002", "FTE-XX-001"]),
            gate("002", "OR", &["FTG-XX-001"]),
        ];
        assert!(matches!(run(els), Err(FtaError::Cycle(_))));
        assert_eq!(run(vec![tree("")]).unwrap_err(), FtaError::EmptyTree);
    }

    fn codes_of(els: &[RawElement]) -> Vec<&'static str> {
        let r = Resolver::new(els);
        let mut v: Vec<&'static str> = structural_issues(els, &r).iter().map(|i| i.code).collect();
        v.sort();
        v
    }

    #[test]
    fn structural_cycle_selfloop_arity_ranges() {
        let cyc = vec![
            tree(""),
            ev("001", "probability: 0.1\n"),
            gate("001", "OR", &["FTG-XX-002", "FTE-XX-001"]),
            gate("002", "OR", &["FTG-XX-001"]),
        ];
        assert!(codes_of(&cyc).contains(&"E960"));
        let selfl = vec![tree(""), ev("001", ""), gate("001", "OR", &["FTG-XX-001", "FTE-XX-001"])];
        assert!(codes_of(&selfl).contains(&"E960"));
        let arity = vec![
            tree(""),
            ev("001", ""),
            ev("002", ""),
            gate("001", "NOT", &["FTE-XX-001", "FTE-XX-002"]),
            gate("002", "inhibit", &["FTE-XX-001"]),
            gate("003", "XOR", &["FTE-XX-001", "FTE-XX-002", "FTG-XX-001"]),
        ];
        assert_eq!(codes_of(&arity).iter().filter(|c| **c == "E961").count(), 3);
        let range = vec![
            tree(""),
            ev("001", "failureRate: -1.0e-9\nprobability: 1.5\nccfBeta: 2\n"),
            gate("001", "OR", &["FTE-XX-001", "FTE-XX-001"]),
        ];
        let c = codes_of(&range);
        for want in ["E962", "E963", "E964"] {
            assert!(c.contains(&want), "{want} in {c:?}");
        }
    }

    #[test]
    fn structural_unreachable_stray_clean() {
        let ok = vec![tree(""), ev("001", ""), ev("002", ""), gate("001", "AND", &["FTE-XX-001", "FTE-XX-002"])];
        assert!(codes_of(&ok).is_empty(), "{:?}", codes_of(&ok));
        let mut un = ok.clone();
        un.push(ev("003", ""));
        assert_eq!(codes_of(&un), vec!["W960"]);
        let mut stray = ok.clone();
        stray.push(el("Other::FTE-Z", "type: FaultTreeEvent\nid: FTE-ZZ-001\nname: z\neventKind: basic\n"));
        assert_eq!(codes_of(&stray), vec!["W961"]);
    }
}
