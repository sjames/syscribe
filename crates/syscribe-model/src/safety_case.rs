//! Safety-argument (GSN) tree construction and completeness analysis
//! (issues #20, #217).
//!
//! Pure data: [`build`] walks the model once and returns a [`SafetyCase`] — a tree
//! of [`SafetyCaseNode`]s per `SafetyGoal` with a rolled-up [`NodeStatus`], a
//! per-goal [`GoalVerdict`] and a [`Completeness`] summary. No rendering lives
//! here, so text/JSON renderers (the `safety-case` CLI command) and diagram
//! derivers (Mermaid/DOT/sprotty) all consume the same traversal.
//!
//! Per goal the tree contains:
//!
//! * the `Argument`s whose `supports` names the goal (recursing through
//!   `supports` and `evidence` links; cycles are cut and flagged);
//! * each argument's `evidence` — Requirements (expanded with their
//!   `derivedChildren`, transitively, and their verifying TestCases),
//!   TestCases, sub-Arguments, AssumptionsOfUse;
//! * the **implicit chain** `SafetyGoal <- Requirement (derivedFromSafetyGoal)
//!   <- derivedChildren* <- TestCase (verifies)`, always folded in (unless
//!   disabled by [`BuildOptions::include_implicit`]) and de-duplicated against
//!   requirements already cited explicitly under the same goal;
//! * `AssumptionOfUse`s whose `appliesTo` names the goal or an argument.
//!
//! Test verdicts come from the caller (the results sidecar lives in the CLI
//! crate) through the `verdict_of` closure.

use std::collections::{HashMap, HashSet};

use crate::element::RawElement;
use crate::resolver::Resolver;

/// Ingested verdict of a TestCase leaf.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    Unknown,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Pass => "pass",
            Verdict::Fail => "fail",
            Verdict::Unknown => "unknown",
        }
    }
}

/// What a node in the tree is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// The top `SafetyGoal`.
    Goal,
    /// GSN claim (`argumentType: claim`, the default).
    Claim,
    Strategy,
    Solution,
    /// GSN context / justification / assumption nodes (`argumentType: context|justification|assumption`).
    Context,
    Justification,
    Assumption,
    /// A claim explicitly declared undeveloped (`argumentType: undeveloped`).
    UndevelopedClaim,
    Requirement,
    TestCase,
    /// An `AssumptionOfUse` element.
    AssumptionOfUse,
    /// Evidence of any other element type (the debug name of the type).
    Other(String),
    /// A reference that does not resolve (`id` holds the raw reference).
    Unresolved,
}

impl NodeKind {
    /// Short label used by renderers (`claim`, `strategy`, `evidence:Requirement`, `AoU`, ...).
    pub fn label(&self) -> String {
        match self {
            NodeKind::Goal => "SafetyGoal".into(),
            NodeKind::Claim => "claim".into(),
            NodeKind::Strategy => "strategy".into(),
            NodeKind::Solution => "solution".into(),
            NodeKind::Context => "context".into(),
            NodeKind::Justification => "justification".into(),
            NodeKind::Assumption => "assumption".into(),
            NodeKind::UndevelopedClaim => "undeveloped".into(),
            NodeKind::Requirement => "evidence:Requirement".into(),
            NodeKind::TestCase => "evidence:TestCase".into(),
            NodeKind::AssumptionOfUse => "AoU".into(),
            NodeKind::Other(t) => format!("evidence:{t}"),
            NodeKind::Unresolved => "unresolved".into(),
        }
    }

    /// True for nodes that give context to a claim rather than support it
    /// (they never affect the support status).
    pub fn is_context(&self) -> bool {
        matches!(
            self,
            NodeKind::Context | NodeKind::Justification | NodeKind::Assumption | NodeKind::AssumptionOfUse
        )
    }

    /// True for `Argument`-element kinds (claim/strategy/solution/context/...).
    pub fn is_argument(&self) -> bool {
        matches!(
            self,
            NodeKind::Claim
                | NodeKind::Strategy
                | NodeKind::Solution
                | NodeKind::Context
                | NodeKind::Justification
                | NodeKind::Assumption
                | NodeKind::UndevelopedClaim
        )
    }
}

/// Rolled-up support status of a node, ordered best to worst.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeStatus {
    /// Context / justification / assumption nodes — not part of the support chain.
    Context,
    /// Every supporting leaf passed.
    Supported,
    /// Supported only by TestCases with no recorded verdict.
    Unverified,
    /// No supporting evidence at all (an undeveloped goal/claim/requirement).
    Undeveloped,
    /// A reference that does not resolve, or a cycle.
    Unresolved,
    /// A supporting TestCase failed.
    Failing,
}

impl NodeStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            NodeStatus::Context => "context",
            NodeStatus::Supported => "supported",
            NodeStatus::Unverified => "unverified",
            NodeStatus::Undeveloped => "undeveloped",
            NodeStatus::Unresolved => "unresolved",
            NodeStatus::Failing => "failing",
        }
    }
}

/// Overall verdict of one goal's argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalVerdict {
    /// All support chains end in passing evidence.
    Supported,
    /// Some part is undeveloped, unverified, or unresolved.
    Incomplete,
    /// At least one supporting TestCase failed.
    Failing,
}

impl GoalVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            GoalVerdict::Supported => "supported",
            GoalVerdict::Incomplete => "incomplete",
            GoalVerdict::Failing => "failing",
        }
    }
}

/// One node of the safety-argument tree.
#[derive(Debug, Clone)]
pub struct SafetyCaseNode {
    pub kind: NodeKind,
    /// Stable id, else qualified name (the raw reference for [`NodeKind::Unresolved`]).
    pub id: String,
    pub qualified_name: Option<String>,
    pub title: String,
    pub status: NodeStatus,
    /// True when this node is the *origin* of undevelopedness: a support node
    /// with no support children (or an explicit undeveloped claim).
    pub undeveloped: bool,
    /// TestCase verdict (TestCase nodes only).
    pub verdict: Option<Verdict>,
    /// Requirement `decompositionKind`, if declared.
    pub decomposition_kind: Option<String>,
    /// Folded in from the implicit chain rather than cited by an Argument.
    pub implicit: bool,
    /// Expansion stopped here because the node is its own ancestor.
    pub cycle: bool,
    pub children: Vec<SafetyCaseNode>,
}

impl SafetyCaseNode {
    /// Pre-order walk of this node and all descendants.
    pub fn walk<'a>(&'a self, f: &mut impl FnMut(&'a SafetyCaseNode)) {
        f(self);
        for c in &self.children {
            c.walk(f);
        }
    }
}

/// Counts summarising how complete an argument is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Completeness {
    pub goals: usize,
    pub goals_supported: usize,
    pub goals_incomplete: usize,
    pub goals_failing: usize,
    /// Distinct nodes that are the origin of undevelopedness.
    pub undeveloped_nodes: usize,
    /// Ids of those nodes (sorted, distinct).
    pub undeveloped_ids: Vec<String>,
    pub unresolved_refs: usize,
    pub cycles: usize,
    /// Distinct Requirements in the trees.
    pub requirements: usize,
    /// Distinct leaf Requirements with no verifying TestCase.
    pub requirements_without_tests: usize,
    /// Distinct TestCases in the trees.
    pub testcases: usize,
    pub tests_pass: usize,
    pub tests_fail: usize,
    pub tests_unknown: usize,
}

/// One goal's tree with its verdict and summary.
#[derive(Debug, Clone)]
pub struct GoalTree {
    pub root: SafetyCaseNode,
    pub verdict: GoalVerdict,
    pub completeness: Completeness,
}

/// The whole result: one tree per selected `SafetyGoal`.
#[derive(Debug, Clone, Default)]
pub struct SafetyCase {
    pub goals: Vec<GoalTree>,
    /// Summary over all goals (distinct counts are across the whole case).
    pub completeness: Completeness,
}

impl SafetyCase {
    /// True when any TestCase leaf anywhere has no recorded verdict.
    pub fn any_unknown(&self) -> bool {
        self.completeness.tests_unknown > 0
    }
}

/// Build options.
#[derive(Debug, Clone, Copy)]
pub struct BuildOptions {
    /// Fold in the implicit `derivedFromSafetyGoal` Requirement chain.
    pub include_implicit: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self { include_implicit: true }
    }
}

fn disp_id(e: &RawElement) -> &str {
    e.frontmatter.id.as_deref().unwrap_or(&e.qualified_name)
}

fn disp_title(e: &RawElement) -> &str {
    e.frontmatter.name.as_deref().unwrap_or("")
}

/// The `SafetyGoal`s selected by `goal_filter` (empty = all), in model order.
pub fn select_goals<'a>(elements: &'a [RawElement], goal_filter: &str) -> Vec<&'a RawElement> {
    elements
        .iter()
        .filter(|e| Resolver::is_safety_goal(e))
        .filter(|e| {
            goal_filter.is_empty()
                || disp_id(e) == goal_filter
                || e.qualified_name == goal_filter
                || e.frontmatter.id.as_deref() == Some(goal_filter)
        })
        .collect()
}

/// Lookup tables built once per [`build`] so the walk is linear.
struct Index<'a> {
    elements: &'a [RawElement],
    resolver: &'a Resolver,
    /// target qname -> Arguments whose `supports` resolves to it.
    args_supporting: HashMap<String, Vec<&'a RawElement>>,
    /// target qname -> AssumptionsOfUse whose `appliesTo` resolves to it.
    aou_for: HashMap<String, Vec<&'a RawElement>>,
    /// parent requirement qname -> requirements `derivedFrom` it.
    derived_children: HashMap<String, Vec<&'a RawElement>>,
    /// requirement qname -> TestCases that `verifies` it.
    tests_for: HashMap<String, Vec<&'a RawElement>>,
    /// goal qname -> requirements with `derivedFromSafetyGoal` resolving to it.
    reqs_for_goal: HashMap<String, Vec<&'a RawElement>>,
}

impl<'a> Index<'a> {
    fn new(elements: &'a [RawElement], resolver: &'a Resolver) -> Self {
        let mut ix = Index {
            elements,
            resolver,
            args_supporting: HashMap::new(),
            aou_for: HashMap::new(),
            derived_children: HashMap::new(),
            tests_for: HashMap::new(),
            reqs_for_goal: HashMap::new(),
        };
        let resolve_q = |r: &str| resolver.resolve_ref(elements, r).map(|t| t.qualified_name.clone());
        for e in elements {
            if Resolver::is_argument(e) {
                for s in e.frontmatter.supports.as_deref().unwrap_or(&[]) {
                    if let Some(q) = resolve_q(s) {
                        ix.args_supporting.entry(q).or_default().push(e);
                    }
                }
            } else if Resolver::is_assumption_of_use(e) {
                for s in e.frontmatter.applies_to.as_deref().unwrap_or(&[]) {
                    if let Some(q) = resolve_q(s) {
                        ix.aou_for.entry(q).or_default().push(e);
                    }
                }
            } else if Resolver::is_native_requirement(e) {
                for d in e.frontmatter.derived_from.as_deref().unwrap_or(&[]) {
                    if let Some(q) = resolve_q(d) {
                        ix.derived_children.entry(q).or_default().push(e);
                    }
                }
                if let Some(g) = e.frontmatter.derived_from_safety_goal.as_deref() {
                    if let Some(q) = resolve_q(g) {
                        ix.reqs_for_goal.entry(q).or_default().push(e);
                    }
                }
            } else if Resolver::is_native_testcase(e) {
                for v in e.frontmatter.verifies.as_deref().unwrap_or(&[]) {
                    if let Some(q) = resolve_q(v) {
                        ix.tests_for.entry(q).or_default().push(e);
                    }
                }
            }
        }
        ix
    }
}

struct Builder<'a, 'v> {
    ix: &'a Index<'a>,
    verdict_of: &'v dyn Fn(&RawElement) -> Verdict,
}

fn new_node(kind: NodeKind, e: &RawElement) -> SafetyCaseNode {
    SafetyCaseNode {
        kind,
        id: disp_id(e).to_string(),
        qualified_name: Some(e.qualified_name.clone()),
        title: disp_title(e).to_string(),
        status: NodeStatus::Supported,
        undeveloped: false,
        verdict: None,
        decomposition_kind: None,
        implicit: false,
        cycle: false,
        children: Vec::new(),
    }
}

fn argument_kind(e: &RawElement) -> NodeKind {
    match e.frontmatter.argument_type.as_deref().unwrap_or("claim") {
        "strategy" => NodeKind::Strategy,
        "solution" => NodeKind::Solution,
        "context" => NodeKind::Context,
        "justification" => NodeKind::Justification,
        "assumption" => NodeKind::Assumption,
        "undeveloped" => NodeKind::UndevelopedClaim,
        _ => NodeKind::Claim,
    }
}

impl<'a, 'v> Builder<'a, 'v> {
    fn goal(&self, goal: &'a RawElement, include_implicit: bool) -> SafetyCaseNode {
        let mut node = new_node(NodeKind::Goal, goal);
        let mut path: HashSet<String> = HashSet::new();
        path.insert(goal.qualified_name.clone());
        if let Some(args) = self.ix.args_supporting.get(&goal.qualified_name) {
            for a in args {
                node.children.push(self.argument(a, &mut path));
            }
        }
        if include_implicit {
            // Requirements already cited under this goal are not repeated.
            let mut cited: HashSet<String> = HashSet::new();
            for c in &node.children {
                c.walk(&mut |n| {
                    if n.kind == NodeKind::Requirement {
                        if let Some(q) = &n.qualified_name {
                            cited.insert(q.clone());
                        }
                    }
                });
            }
            if let Some(reqs) = self.ix.reqs_for_goal.get(&goal.qualified_name) {
                for r in reqs {
                    if cited.contains(&r.qualified_name) {
                        continue;
                    }
                    let mut rn = self.requirement(r, &mut path);
                    rn.implicit = true;
                    node.children.push(rn);
                }
            }
        }
        if let Some(aous) = self.ix.aou_for.get(&goal.qualified_name) {
            for a in aous {
                node.children.push(new_node(NodeKind::AssumptionOfUse, a));
            }
        }
        finalize(&mut node);
        node
    }

    fn argument(&self, arg: &'a RawElement, path: &mut HashSet<String>) -> SafetyCaseNode {
        let mut node = new_node(argument_kind(arg), arg);
        if !path.insert(arg.qualified_name.clone()) {
            node.cycle = true;
            finalize(&mut node);
            return node;
        }
        let mut seen: HashSet<String> = HashSet::new();
        // Evidence references, in authored order.
        for r in arg.frontmatter.evidence.as_deref().unwrap_or(&[]).iter().filter_map(|v| v.as_str()) {
            match self.ix.resolver.resolve_ref(self.ix.elements, r) {
                None => node.children.push(unresolved(r)),
                Some(t) => {
                    if !seen.insert(t.qualified_name.clone()) {
                        continue;
                    }
                    node.children.push(self.evidence_target(t, path));
                }
            }
        }
        // Arguments that declare `supports` on this one.
        if let Some(subs) = self.ix.args_supporting.get(&arg.qualified_name) {
            for s in subs {
                if seen.insert(s.qualified_name.clone()) {
                    node.children.push(self.argument(s, path));
                }
            }
        }
        if let Some(aous) = self.ix.aou_for.get(&arg.qualified_name) {
            for a in aous {
                if seen.insert(a.qualified_name.clone()) {
                    node.children.push(new_node(NodeKind::AssumptionOfUse, a));
                }
            }
        }
        path.remove(&arg.qualified_name);
        finalize(&mut node);
        node
    }

    fn evidence_target(&self, t: &'a RawElement, path: &mut HashSet<String>) -> SafetyCaseNode {
        if Resolver::is_argument(t) {
            self.argument(t, path)
        } else if Resolver::is_native_requirement(t) {
            self.requirement(t, path)
        } else if Resolver::is_native_testcase(t) {
            self.testcase(t)
        } else if Resolver::is_assumption_of_use(t) {
            new_node(NodeKind::AssumptionOfUse, t)
        } else {
            let ty = t
                .frontmatter
                .element_type
                .as_ref()
                .map(|x| format!("{x:?}"))
                .unwrap_or_else(|| "Unknown".into());
            let mut n = new_node(NodeKind::Other(ty), t);
            finalize_leaf_supported(&mut n);
            n
        }
    }

    fn testcase(&self, tc: &'a RawElement) -> SafetyCaseNode {
        let mut n = new_node(NodeKind::TestCase, tc);
        let v = (self.verdict_of)(tc);
        n.verdict = Some(v);
        n.status = match v {
            Verdict::Pass => NodeStatus::Supported,
            Verdict::Fail => NodeStatus::Failing,
            Verdict::Unknown => NodeStatus::Unverified,
        };
        n
    }

    /// A Requirement with its derived children (transitively) and verifying tests.
    fn requirement(&self, req: &'a RawElement, path: &mut HashSet<String>) -> SafetyCaseNode {
        let mut node = new_node(NodeKind::Requirement, req);
        node.decomposition_kind = req.frontmatter.decomposition_kind.clone();
        if !path.insert(req.qualified_name.clone()) {
            node.cycle = true;
            finalize(&mut node);
            return node;
        }
        if let Some(kids) = self.ix.derived_children.get(&req.qualified_name) {
            for k in kids {
                node.children.push(self.requirement(k, path));
            }
        }
        if let Some(tcs) = self.ix.tests_for.get(&req.qualified_name) {
            for tc in tcs {
                node.children.push(self.testcase(tc));
            }
        }
        path.remove(&req.qualified_name);
        finalize(&mut node);
        node
    }
}

fn unresolved(r: &str) -> SafetyCaseNode {
    SafetyCaseNode {
        kind: NodeKind::Unresolved,
        id: r.to_string(),
        qualified_name: None,
        title: String::new(),
        status: NodeStatus::Unresolved,
        undeveloped: false,
        verdict: None,
        decomposition_kind: None,
        implicit: false,
        cycle: false,
        children: Vec::new(),
    }
}

fn finalize_leaf_supported(n: &mut SafetyCaseNode) {
    n.status = NodeStatus::Supported;
}

/// Roll `node.status` up from its (already finalised) children.
fn finalize(node: &mut SafetyCaseNode) {
    if node.cycle {
        node.status = NodeStatus::Unresolved;
        return;
    }
    if node.kind.is_context() {
        node.status = NodeStatus::Context;
        return;
    }
    if node.kind == NodeKind::UndevelopedClaim {
        node.status = NodeStatus::Undeveloped;
        node.undeveloped = true;
        return;
    }
    let worst = node
        .children
        .iter()
        .filter(|c| !c.kind.is_context())
        .map(|c| c.status)
        .max();
    match worst {
        None => {
            node.status = NodeStatus::Undeveloped;
            node.undeveloped = true;
        }
        Some(s) => node.status = s,
    }
}

fn summarize(root: &SafetyCaseNode, verdict: GoalVerdict) -> Completeness {
    let mut c = Completeness { goals: 1, ..Default::default() };
    match verdict {
        GoalVerdict::Supported => c.goals_supported = 1,
        GoalVerdict::Incomplete => c.goals_incomplete = 1,
        GoalVerdict::Failing => c.goals_failing = 1,
    }
    let mut reqs: HashSet<&str> = HashSet::new();
    let mut untested: HashSet<&str> = HashSet::new();
    let mut tcs: HashMap<&str, Verdict> = HashMap::new();
    let mut und: HashSet<&str> = HashSet::new();
    root.walk(&mut |n| {
        if n.undeveloped {
            und.insert(n.id.as_str());
        }
        if n.kind == NodeKind::Unresolved {
            c.unresolved_refs += 1;
        }
        if n.cycle {
            c.cycles += 1;
        }
        match n.kind {
            NodeKind::Requirement => {
                reqs.insert(n.id.as_str());
                if n.undeveloped {
                    untested.insert(n.id.as_str());
                }
            }
            NodeKind::TestCase => {
                tcs.insert(n.id.as_str(), n.verdict.unwrap_or(Verdict::Unknown));
            }
            _ => {}
        }
    });
    c.requirements = reqs.len();
    c.requirements_without_tests = untested.len();
    c.testcases = tcs.len();
    for v in tcs.values() {
        match v {
            Verdict::Pass => c.tests_pass += 1,
            Verdict::Fail => c.tests_fail += 1,
            Verdict::Unknown => c.tests_unknown += 1,
        }
    }
    let mut ids: Vec<String> = und.into_iter().map(String::from).collect();
    ids.sort();
    c.undeveloped_nodes = ids.len();
    c.undeveloped_ids = ids;
    c
}

fn merge(total: &mut Completeness, goals: &[GoalTree]) {
    // Distinct counts across goals need the union, so recompute from the trees.
    let mut reqs: HashSet<&str> = HashSet::new();
    let mut untested: HashSet<&str> = HashSet::new();
    let mut tcs: HashMap<&str, Verdict> = HashMap::new();
    let mut und: HashSet<&str> = HashSet::new();
    for g in goals {
        total.goals += 1;
        match g.verdict {
            GoalVerdict::Supported => total.goals_supported += 1,
            GoalVerdict::Incomplete => total.goals_incomplete += 1,
            GoalVerdict::Failing => total.goals_failing += 1,
        }
        total.unresolved_refs += g.completeness.unresolved_refs;
        total.cycles += g.completeness.cycles;
        g.root.walk(&mut |n| {
            if n.undeveloped {
                und.insert(n.id.as_str());
            }
            match n.kind {
                NodeKind::Requirement => {
                    reqs.insert(n.id.as_str());
                    if n.undeveloped {
                        untested.insert(n.id.as_str());
                    }
                }
                NodeKind::TestCase => {
                    tcs.insert(n.id.as_str(), n.verdict.unwrap_or(Verdict::Unknown));
                }
                _ => {}
            }
        });
    }
    total.requirements = reqs.len();
    total.requirements_without_tests = untested.len();
    total.testcases = tcs.len();
    for v in tcs.values() {
        match v {
            Verdict::Pass => total.tests_pass += 1,
            Verdict::Fail => total.tests_fail += 1,
            Verdict::Unknown => total.tests_unknown += 1,
        }
    }
    let mut ids: Vec<String> = und.into_iter().map(String::from).collect();
    ids.sort();
    total.undeveloped_nodes = ids.len();
    total.undeveloped_ids = ids;
}

/// Build the safety case for the goals selected by `goal_filter` (empty = all).
///
/// `verdict_of` maps a TestCase to its ingested verdict (callers hold the
/// results sidecar). An empty result with a non-empty filter means the id is
/// unknown.
pub fn build(
    elements: &[RawElement],
    resolver: &Resolver,
    goal_filter: &str,
    opts: BuildOptions,
    verdict_of: &dyn Fn(&RawElement) -> Verdict,
) -> SafetyCase {
    let ix = Index::new(elements, resolver);
    let b = Builder { ix: &ix, verdict_of };
    let mut goals = Vec::new();
    for g in select_goals(elements, goal_filter) {
        let root = b.goal(g, opts.include_implicit);
        let verdict = match root.status {
            NodeStatus::Failing => GoalVerdict::Failing,
            NodeStatus::Supported => GoalVerdict::Supported,
            _ => GoalVerdict::Incomplete,
        };
        let completeness = summarize(&root, verdict);
        goals.push(GoalTree { root, verdict, completeness });
    }
    let mut completeness = Completeness::default();
    merge(&mut completeness, &goals);
    SafetyCase { goals, completeness }
}
