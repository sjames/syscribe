use std::collections::HashMap;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

/// Extract qualified name strings from a field that may be a YAML String or Sequence.
fn yaml_strings(v: &serde_yaml::Value) -> Vec<&str> {
    match v {
        serde_yaml::Value::String(s) => vec![s.as_str()],
        serde_yaml::Value::Sequence(seq) => seq.iter().filter_map(|x| x.as_str()).collect(),
        _ => vec![],
    }
}

/// Edge kinds in the model graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    Contains,              // parent package → child element
    Supertype,             // element → its supertype definition
    TypedBy,               // usage → its type definition
    Subsets,               // feature → subsetted feature
    Redefines,             // feature → redefined feature
    Verifies,              // test case → requirement (native or SysML)
    DerivedFrom,           // child requirement → parent requirement
    AllocatedFrom,         // allocation → source element
    AllocatedTo,           // allocation → target element
    ConditionalOn,         // element → FeatureDef (appliesWhen:)
    Satisfies,             // Part/Config → Requirement/FeatureDef (satisfies:)
    // Part-to-part wiring (resolved from connection feature chains; issue #26)
    Connection,            // connections:        (from/to or ends[].binds)
    Flow,                  // flowConnections:    (from/to)
    Binding,               // bindingConnections: (left/right)
    Succession,            // successionConnections: (from/to)
    // Feature-level typing: owner → the type of an inline `features:` entry that
    // declares `typedBy:`. Distinct from the top-level `TypedBy` so it does NOT
    // participate in typedBy cycle detection (E107); it exists so a structural
    // walk (e.g. `connectivity`) can reach a part's sub-part types (issue #26).
    FeatureTyped,
    // Safety analysis (ISO 26262 / IEC 61508)
    TopEvent,              // FaultTree → SafetyGoal
    FaultTreeInput,        // FaultTreeGate → input gate/event
    HazardousEventRef,     // SafetyGoal → HazardousEvent
    DerivedFromSafetyGoal, // Requirement → SafetyGoal
    // Security analysis (ISO/SAE 21434)
    DamageScenarioRef,       // ThreatScenario → DamageScenario
    ThreatScenarioRef,       // CybersecurityGoal → ThreatScenario
    ImplementsGoal,          // SecurityControl → CybersecurityGoal
    MitigatedBy,             // VulnerabilityReport → SecurityControl
    DerivedFromSecurityGoal, // Requirement → CybersecurityGoal
    ThreatRef,               // AttackTree → ThreatScenario
    AttackTreeInput,         // AttackTreeGate → input gate/step
}

impl EdgeKind {
    /// Lower-camel canonical name of an edge kind. Stable: used by the
    /// `connectivity` CLI for `--kinds` filtering and the JSON `kind` field.
    pub fn name(&self) -> &'static str {
        match self {
            EdgeKind::Contains => "contains",
            EdgeKind::Supertype => "supertype",
            EdgeKind::TypedBy => "typedBy",
            EdgeKind::Subsets => "subsets",
            EdgeKind::Redefines => "redefines",
            EdgeKind::Verifies => "verifies",
            EdgeKind::DerivedFrom => "derivedFrom",
            EdgeKind::AllocatedFrom => "allocatedFrom",
            EdgeKind::AllocatedTo => "allocatedTo",
            EdgeKind::ConditionalOn => "conditionalOn",
            EdgeKind::Satisfies => "satisfies",
            EdgeKind::Connection => "connection",
            EdgeKind::Flow => "flow",
            EdgeKind::Binding => "binding",
            EdgeKind::Succession => "succession",
            EdgeKind::FeatureTyped => "featureTyped",
            EdgeKind::TopEvent => "topEvent",
            EdgeKind::FaultTreeInput => "faultTreeInput",
            EdgeKind::HazardousEventRef => "hazardousEventRef",
            EdgeKind::DerivedFromSafetyGoal => "derivedFromSafetyGoal",
            EdgeKind::DamageScenarioRef => "damageScenarioRef",
            EdgeKind::ThreatScenarioRef => "threatScenarioRef",
            EdgeKind::ImplementsGoal => "implementsGoal",
            EdgeKind::MitigatedBy => "mitigatedBy",
            EdgeKind::DerivedFromSecurityGoal => "derivedFromSecurityGoal",
            EdgeKind::ThreatRef => "threatRef",
            EdgeKind::AttackTreeInput => "attackTreeInput",
        }
    }
}

pub type ModelGraph = DiGraph<String, EdgeKind>;

/// Build a graph from a flat list of elements.
/// Returns (graph, node_index_by_qname).
pub fn build_graph(elements: &[RawElement]) -> (ModelGraph, HashMap<String, NodeIndex>) {
    let resolver = Resolver::new(elements);
    let mut graph = DiGraph::new();
    let mut idx: HashMap<String, NodeIndex> = HashMap::new();

    // First pass: add all nodes
    for elem in elements {
        let ni = graph.add_node(elem.qualified_name.clone());
        idx.insert(elem.qualified_name.clone(), ni);
    }

    // Helper: resolve a ref string to a NodeIndex using ID-first lookup.
    let resolve_to_idx = |r: &str| -> Option<NodeIndex> {
        resolver
            .resolve_ref(elements, r)
            .and_then(|e| idx.get(&e.qualified_name))
            .copied()
    };

    // Second pass: add edges
    for elem in elements {
        let Some(&src) = idx.get(&elem.qualified_name) else { continue };

        // Contains: if qname has a "::" prefix, parent is everything before last "::"
        if let Some(sep) = elem.qualified_name.rfind("::") {
            let parent_qn = &elem.qualified_name[..sep];
            if let Some(&parent_ni) = idx.get(parent_qn) {
                graph.add_edge(parent_ni, src, EdgeKind::Contains);
            }
        } else if !elem.qualified_name.is_empty() {
            // Top-level element (no "::"): contained by the model-root element
            // (the root `_index.md`, which carries the empty qualified name), if
            // one exists. This lets a walk from the model root reach the whole
            // model via Contains (issue #26). No-op for models without a root
            // `_index.md`; the root element itself is excluded by the guard.
            if let Some(&root_ni) = idx.get("") {
                graph.add_edge(root_ni, src, EdgeKind::Contains);
            }
        }

        let fm = &elem.frontmatter;

        // Supertype — may be a string or a sequence
        if let Some(ref st) = fm.supertype {
            for s in yaml_strings(st) {
                if let Some(dst) = idx.get(s).copied() {
                    graph.add_edge(src, dst, EdgeKind::Supertype);
                }
            }
        }

        // TypedBy — may be a string or a sequence
        if let Some(ref tb) = fm.typed_by {
            for s in yaml_strings(tb) {
                if let Some(dst) = idx.get(s).copied() {
                    graph.add_edge(src, dst, EdgeKind::TypedBy);
                }
            }
        }

        // FeatureTyped — owner → the type of each inline `features:` entry that
        // declares `typedBy:`. Kept distinct from `TypedBy` so it is invisible to
        // typedBy cycle detection (E107); it gives a structural walk a path from a
        // part to its sub-part types (issue #26).
        if let Some(ref feats) = fm.features {
            for f in feats {
                let serde_yaml::Value::Mapping(m) = f else { continue };
                if let Some(tb) = m
                    .get(serde_yaml::Value::from("typedBy"))
                    .and_then(|v| v.as_str())
                {
                    if let Some(dst) = resolve_to_idx(tb) {
                        if dst != src {
                            graph.add_edge(src, dst, EdgeKind::FeatureTyped);
                        }
                    }
                }
            }
        }

        // Subsets (Vec<String>)
        if let Some(ref ss) = fm.subsets {
            for s in ss {
                if let Some(dst) = idx.get(s.as_str()).copied() {
                    graph.add_edge(src, dst, EdgeKind::Subsets);
                }
            }
        }

        // Redefines — may be a string or a sequence
        if let Some(ref rd) = fm.redefines {
            for s in yaml_strings(rd) {
                if let Some(dst) = idx.get(s).copied() {
                    graph.add_edge(src, dst, EdgeKind::Redefines);
                }
            }
        }

        // Verifies — use ID-based resolution so REQ-* refs work
        if let Some(ref vs) = fm.verifies {
            for v in vs {
                if let Some(dst) = resolve_to_idx(v) {
                    graph.add_edge(src, dst, EdgeKind::Verifies);
                }
            }
        }

        // DerivedFrom — use ID-based resolution
        if let Some(ref dfs) = fm.derived_from {
            for df in dfs {
                if let Some(dst) = resolve_to_idx(df) {
                    graph.add_edge(src, dst, EdgeKind::DerivedFrom);
                }
            }
        }

        // allocatedFrom / allocatedTo
        if let Some(ref afs) = fm.allocated_from {
            for af in afs {
                if let Some(dst) = idx.get(af.as_str()).copied() {
                    graph.add_edge(src, dst, EdgeKind::AllocatedFrom);
                }
            }
        }
        if let Some(ref ats) = fm.allocated_to {
            for at_ in ats {
                if let Some(dst) = idx.get(at_.as_str()).copied() {
                    graph.add_edge(src, dst, EdgeKind::AllocatedTo);
                }
            }
        }

        // Satisfies — each entry resolves to a Requirement or FeatureDef
        if let Some(ref sat) = fm.satisfies {
            for s in sat {
                if let Some(dst) = resolve_to_idx(s) {
                    graph.add_edge(src, dst, EdgeKind::Satisfies);
                }
            }
        }

        // AppliesWhen — each resolved target must be a FeatureDef
        if let Some(ref aw) = fm.applies_when {
            for s in yaml_strings(aw) {
                if let Some(dst) = idx.get(s).copied() {
                    graph.add_edge(src, dst, EdgeKind::ConditionalOn);
                }
            }
        }

        // ── Part-to-part wiring (issue #26) ──────────────────────────────────
        //
        // Resolve each connection endpoint feature chain to its owning element
        // and add an edge of the matching kind. Endpoint resolution (the crux):
        //   * head = the chain segment before the first '.'
        //   * if this element's `features:` has a mapping with `name == head`
        //     carrying `typedBy: T`, the endpoint node = resolve_ref(T);
        //   * else resolve_ref(chain) (whole chain as a qname/id);
        //   * else the endpoint is unresolved → skip it.
        // Binary connections add one edge between the two ends; n-ary ones add a
        // star from the first resolved end to each other resolved end. Self-edges
        // (head resolving to this element) are skipped.
        //
        // NOTE (deferred, issue #26 MVP): edges carry `kind` only — rich edge
        // labels (the `via` ConnectionDef/InterfaceDef and the from/to feature
        // chains) are out of scope here.
        let resolve_endpoint = |chain: &str| -> Option<NodeIndex> {
            let head = chain.split('.').next().unwrap_or(chain);
            // feature-typed endpoint: look up `head` in this element's features.
            if let Some(feats) = &fm.features {
                for f in feats {
                    let serde_yaml::Value::Mapping(m) = f else { continue };
                    let name = m.get(serde_yaml::Value::from("name")).and_then(|v| v.as_str());
                    if name != Some(head) {
                        continue;
                    }
                    if let Some(tb) = m
                        .get(serde_yaml::Value::from("typedBy"))
                        .and_then(|v| v.as_str())
                    {
                        return resolve_to_idx(tb);
                    }
                }
            }
            // otherwise treat the whole chain as a qname/id.
            resolve_to_idx(chain)
        };

        // Add wiring edges from a resolved endpoint list (binary or n-ary star).
        let add_wiring = |ends: Vec<NodeIndex>, kind: EdgeKind, graph: &mut ModelGraph| {
            // dedup & drop self-edges; keep first occurrence order.
            let resolved: Vec<NodeIndex> = {
                let mut seen = std::collections::HashSet::new();
                ends.into_iter().filter(|n| *n != src && seen.insert(*n)).collect()
            };
            let Some((&first, rest)) = resolved.split_first() else { return };
            if rest.is_empty() {
                return; // need at least two distinct endpoints to form an edge
            }
            for &other in rest {
                graph.add_edge(first, other, kind);
            }
        };

        // Collect a single connection entry's resolved endpoints, given the YAML
        // field names that hold its endpoint chains.
        let entry_endpoints = |entry: &serde_yaml::Value| -> Vec<NodeIndex> {
            let serde_yaml::Value::Mapping(m) = entry else { return vec![] };
            let mut chains: Vec<String> = Vec::new();
            // binary forms
            for key in ["from", "to", "left", "right"] {
                if let Some(s) = m.get(serde_yaml::Value::from(key)).and_then(|v| v.as_str()) {
                    chains.push(s.to_string());
                }
            }
            // n-ary form: ends: [{ binds: <chain> }, …]
            if let Some(serde_yaml::Value::Sequence(seq)) =
                m.get(serde_yaml::Value::from("ends"))
            {
                for e in seq {
                    if let serde_yaml::Value::Mapping(em) = e {
                        if let Some(s) = em
                            .get(serde_yaml::Value::from("binds"))
                            .and_then(|v| v.as_str())
                        {
                            chains.push(s.to_string());
                        }
                    }
                }
            }
            chains.iter().filter_map(|c| resolve_endpoint(c)).collect()
        };

        for (list, kind) in [
            (&fm.connections, EdgeKind::Connection),
            (&fm.flow_connections, EdgeKind::Flow),
            (&fm.binding_connections, EdgeKind::Binding),
            (&fm.succession_connections, EdgeKind::Succession),
        ] {
            if let Some(entries) = list {
                for entry in entries {
                    let ends = entry_endpoints(entry);
                    add_wiring(ends, kind, &mut graph);
                }
            }
        }

        // ── Safety analysis ──────────────────────────────────────────────────

        // topEvent: FaultTree → SafetyGoal
        if let Some(ref te) = fm.top_event {
            if let Some(dst) = resolve_to_idx(te) {
                graph.add_edge(src, dst, EdgeKind::TopEvent);
            }
        }

        // inputs: FaultTreeGate / AttackTreeGate → input gates/events/steps
        if let Some(ref ins) = fm.inputs {
            let kind = if matches!(fm.element_type, Some(ElementType::AttackTreeGate)) {
                EdgeKind::AttackTreeInput
            } else {
                EdgeKind::FaultTreeInput
            };
            for i in ins {
                if let Some(dst) = resolve_to_idx(i) {
                    graph.add_edge(src, dst, kind);
                }
            }
        }

        // hazardousEvents: SafetyGoal → HazardousEvent refs
        if let Some(ref hes) = fm.hazardous_events {
            for he in hes {
                if let Some(dst) = resolve_to_idx(he) {
                    graph.add_edge(src, dst, EdgeKind::HazardousEventRef);
                }
            }
        }

        // derivedFromSafetyGoal: Requirement → SafetyGoal
        if let Some(ref sg) = fm.derived_from_safety_goal {
            if let Some(dst) = resolve_to_idx(sg) {
                graph.add_edge(src, dst, EdgeKind::DerivedFromSafetyGoal);
            }
        }

        // ── Security analysis ────────────────────────────────────────────────

        // damageScenarios: ThreatScenario → DamageScenario refs
        if let Some(ref ds) = fm.damage_scenarios {
            for d in ds {
                if let Some(dst) = resolve_to_idx(d) {
                    graph.add_edge(src, dst, EdgeKind::DamageScenarioRef);
                }
            }
        }

        // threatRef: AttackTree → ThreatScenario
        if let Some(ref tr) = fm.threat_ref {
            if let Some(dst) = resolve_to_idx(tr) {
                graph.add_edge(src, dst, EdgeKind::ThreatRef);
            }
        }

        // threatScenarios: CybersecurityGoal → ThreatScenario refs
        if let Some(ref ts) = fm.threat_scenarios {
            for t in ts {
                if let Some(dst) = resolve_to_idx(t) {
                    graph.add_edge(src, dst, EdgeKind::ThreatScenarioRef);
                }
            }
        }

        // implementsGoals: SecurityControl → CybersecurityGoal refs
        if let Some(ref ig) = fm.implements_goals {
            for g in ig {
                if let Some(dst) = resolve_to_idx(g) {
                    graph.add_edge(src, dst, EdgeKind::ImplementsGoal);
                }
            }
        }

        // mitigatedBy: VulnerabilityReport → SecurityControl refs
        if let Some(ref mbs) = fm.mitigated_by {
            for mb in mbs {
                if let Some(dst) = resolve_to_idx(mb) {
                    graph.add_edge(src, dst, EdgeKind::MitigatedBy);
                }
            }
        }

        // derivedFromSecurityGoal: Requirement → CybersecurityGoal
        if let Some(ref csg) = fm.derived_from_cybersecurity_goal {
            if let Some(dst) = resolve_to_idx(csg) {
                graph.add_edge(src, dst, EdgeKind::DerivedFromSecurityGoal);
            }
        }
    }

    (graph, idx)
}

/// A connectivity walk result, expressed in plain qualified names so callers
/// outside this crate need not depend on `petgraph` (issue #26). `nodes` is in
/// breadth-first discovery order from the root; `edges` are the deduplicated
/// `(from, to, kind)` edges among the reachable nodes that match the followed
/// kinds.
pub struct Subgraph {
    pub root: String,
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String, EdgeKind)>,
}

/// BFS outward from `root_qname` over the graph, following only edge kinds whose
/// [`EdgeKind::name`] is in `follow`. `depth` bounds the number of hops (`None`
/// = unbounded). With `undirected`, edges are traversed in both directions.
///
/// Returns `None` if the root qname is not a graph node. The reusable backbone
/// behind the `connectivity` command (and available to the server/diagram paths).
pub fn connectivity_subgraph(
    graph: &ModelGraph,
    idx: &HashMap<String, NodeIndex>,
    root_qname: &str,
    follow: &std::collections::HashSet<String>,
    depth: Option<usize>,
    undirected: bool,
) -> Option<Subgraph> {
    use petgraph::Direction;
    use std::collections::{HashSet, VecDeque};

    let &root = idx.get(root_qname)?;
    let followed = |k: &EdgeKind| follow.contains(k.name());

    let mut nodes: Vec<NodeIndex> = vec![root];
    let mut visited: HashSet<NodeIndex> = HashSet::from([root]);
    let mut queue: VecDeque<(NodeIndex, usize)> = VecDeque::from([(root, 0usize)]);

    while let Some((n, d)) = queue.pop_front() {
        if depth.map(|max| d >= max).unwrap_or(false) {
            continue;
        }
        for e in graph.edges_directed(n, Direction::Outgoing) {
            if !followed(e.weight()) {
                continue;
            }
            let t = e.target();
            if visited.insert(t) {
                nodes.push(t);
                queue.push_back((t, d + 1));
            }
        }
        if undirected {
            for e in graph.edges_directed(n, Direction::Incoming) {
                if !followed(e.weight()) {
                    continue;
                }
                let s = e.source();
                if visited.insert(s) {
                    nodes.push(s);
                    queue.push_back((s, d + 1));
                }
            }
        }
    }

    let in_set: HashSet<NodeIndex> = nodes.iter().copied().collect();
    let mut seen: HashSet<(NodeIndex, NodeIndex, &'static str)> = HashSet::new();
    let mut edges: Vec<(String, String, EdgeKind)> = Vec::new();
    for &n in &nodes {
        for e in graph.edges_directed(n, Direction::Outgoing) {
            if !followed(e.weight()) {
                continue;
            }
            let t = e.target();
            if !in_set.contains(&t) {
                continue;
            }
            if seen.insert((e.source(), t, e.weight().name())) {
                edges.push((graph[e.source()].clone(), graph[t].clone(), *e.weight()));
            }
        }
    }

    Some(Subgraph {
        root: root_qname.to_string(),
        nodes: nodes.into_iter().map(|n| graph[n].clone()).collect(),
        edges,
    })
}

/// Return qualified names of direct children (Contains edges from node).
pub fn children_of<'a>(
    graph: &'a ModelGraph,
    idx: &'a HashMap<String, NodeIndex>,
    qname: &str,
) -> Vec<&'a str> {
    let Some(&ni) = idx.get(qname) else { return vec![] };
    graph
        .edges(ni)
        .filter(|e| *e.weight() == EdgeKind::Contains)
        .map(|e| graph[e.target()].as_str())
        .collect()
}
