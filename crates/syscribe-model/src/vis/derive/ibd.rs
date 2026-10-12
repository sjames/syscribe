//! The IBD generator (`REQ-TRS-VIS-005`).
//!
//! Subject: a `PartDef` or `Part`. Produces a `Boundary` for the subject with
//! its own ports on the boundary, a `Block` child per owned part usage
//! (inline `features:` typed by a definition, or child `Part`/`Item`
//! elements) carrying that usage's ports (its own port features plus the
//! ports its definition declares), and `Connection`/`Flow`/`Binding`/
//! `Succession` edges from the subject's `connections:`,
//! `flowConnections:`, `bindingConnections:` and `successionConnections:`,
//! resolving dotted feature chains (`engine.powerOut`) exactly as the
//! graph builder does. The generator never invents a port: a chain whose
//! endpoint is not a node of the diagram produces no edge.

use std::collections::HashSet;

use crate::connections::parse_entry;
use crate::element::{ElementType, RawElement};
use crate::members::direct_members;
use crate::resolver::Resolver;

use super::super::ir::{derived_shape_id, DiagramGraph, Edge, EdgeKind, FlowKind, Node, NodeKind, NodeMark, Tone};
use super::super::manifest::Issue;
use super::{applies_when_text, block_node, display_name, features_of, map_str, port_node, short_name, w418, w420, w421, yaml_strings, Feature, FeatureRole, Filters};

/// One owned part usage of the subject (or, nested, of a usage's definition).
struct Usage<'a> {
    name: String,
    /// The usage element when it is a child `Part` file; `None` for an inline feature.
    element: Option<&'a RawElement>,
    /// The definition the usage is typed by, when it resolves.
    definition: Option<&'a RawElement>,
    /// Port features declared on the usage itself (inline features of a child
    /// `Part` element).
    own_ports: Vec<Feature>,
    multiplicity: Option<String>,
    /// The inline feature's own `appliesWhen:` (GH #336).
    applies_when: Option<String>,
    /// `role: external` (an actor) or `role: environment` (the test set-up) (GH #337).
    role: Option<String>,
}

/// The part usages `owner` declares: its inline part features, then (when `with_children`) its child
/// `Part`/`Item` files.
fn usages_of<'a>(owner: &'a RawElement, with_children: bool, elements: &'a [RawElement], resolver: &Resolver) -> Vec<Usage<'a>> {
    let mut usages: Vec<Usage> = features_of(owner, elements, resolver)
        .into_iter()
        .filter(|f| f.role == FeatureRole::Part)
        .map(|f| Usage {
            definition: f.typed_by.as_deref().and_then(|t| resolver.resolve_ref(elements, t)),
            name: f.name,
            element: None,
            own_ports: Vec::new(),
            multiplicity: f.multiplicity,
            applies_when: f.applies_when,
            role: f.usage_role,
        })
        .collect();
    if !with_children {
        return usages;
    }
    for child in direct_members(elements, &owner.qualified_name) {
        if !matches!(child.frontmatter.element_type, Some(ElementType::Part) | Some(ElementType::Item)) {
            continue;
        }
        let name = short_name(&child.qualified_name).to_string();
        if usages.iter().any(|u| u.name == name) {
            continue;
        }
        usages.push(Usage {
            name,
            element: Some(child),
            definition: yaml_strings(child.frontmatter.typed_by.as_ref()).first().and_then(|t| resolver.resolve_ref(elements, t)),
            own_ports: features_of(child, elements, resolver).into_iter().filter(|f| f.role == FeatureRole::Port).collect(),
            multiplicity: child.frontmatter.multiplicity.clone(),
            applies_when: None,
            role: None,
        });
    }
    usages
}

/// What the generator learnt about each usage node it drew, for containment and grouping.
struct Drawn {
    id: String,
    /// The usage's definition, else the usage element.
    definition: Option<String>,
    element_ref: String,
    domain: Option<String>,
    /// `tags:` of the usage element and its definition (GH #338).
    tags: Vec<String>,
    /// The features its `appliesWhen:` (inline, usage element or definition) names, not negated (GH #338).
    features: Vec<String>,
}

struct Ctx<'a, 'b> {
    elements: &'a [RawElement],
    resolver: &'b Resolver,
    depth: usize,
    drawn: Vec<Drawn>,
    /// The definitions on the path from the subject to the usage being drawn: a definition that contains
    /// itself, directly or through others, is drawn once on a path and not opened again.
    path: Vec<String>,
}

/// The deepest `depth:` an IBD follows; a larger value is capped here.
pub const MAX_DEPTH: usize = 16;

/// `base`, or `base-2`, `base-3`, … when a node already has that id: two members whose names map to the same
/// id (a port and a part of one name, `a::b` and `a_b`) are both drawn.
fn fresh_id(graph: &DiagramGraph, base: String) -> String {
    if graph.node(&base).is_none() {
        return base;
    }
    (2..).map(|n| format!("{base}-{n}")).find(|id| graph.node(id).is_none()).unwrap()
}

/// The safety level an element declares, as a badge (`SIL 2`, `ASIL B`), or `safety` for a `safety` banner.
fn safety_badge(e: &RawElement, banners: &[String]) -> Option<String> {
    let fm = &e.frontmatter;
    let asil = fm.asil_level.as_deref().map(|a| {
        let a = a.trim();
        let a = if a.len() > 4 && a[..4].eq_ignore_ascii_case("asil") { a[4..].trim() } else { a };
        a.to_ascii_uppercase()
    });
    if let Some(a) = asil.filter(|a| crate::asil::rank(a).is_some_and(|r| r > 0)) {
        // a decomposed level keeps its notation: `B(D)`
        return Some(match fm.decomposed_from.as_deref().filter(|d| crate::asil::rank(d).is_some()) {
            Some(d) => format!("ASIL {a}({})", d.trim().to_ascii_uppercase()),
            None => format!("ASIL {a}"),
        });
    }
    if let Some(l) = fm.sil_level.filter(|l| *l >= 1) {
        return Some(format!("SIL {l}"));
    }
    banners.iter().any(|b| b.eq_ignore_ascii_case("safety")).then(|| "safety".to_string())
}

/// The mark of a usage node (GH #336): a safety level outlines it, an `appliesWhen:` badges it optional.
/// `None` when neither applies, so the node keeps the colours of its kind.
fn usage_mark(u: &Usage, banners: &[String]) -> Option<NodeMark> {
    let srcs: Vec<&RawElement> = u.element.into_iter().chain(u.definition).collect();
    let safety = srcs.iter().find_map(|e| safety_badge(e, banners));
    let applies = u
        .applies_when
        .clone()
        .or_else(|| srcs.iter().find_map(|e| e.frontmatter.applies_when.as_ref().map(applies_when_text)));
    if safety.is_none() && applies.is_none() {
        return None;
    }
    let mut badges = Vec::new();
    badges.extend(safety.clone());
    if applies.is_some() {
        badges.push("optional".to_string());
    }
    Some(NodeMark {
        detail: None,
        status: None,
        value: applies.map(|a| format!("when {}", a.rsplit("::").next().filter(|_| !a.contains(' ')).unwrap_or(&a))),
        tone: if safety.is_some() { Tone::Warn } else { Tone::Neutral },
        badges,
        emphasis: safety.is_some(),
    })
}

impl Ctx<'_, '_> {
    /// Draw usage `u` of `owner_qn` inside `parent`, its ports, and — `level` below the depth — its own
    /// usages and connections.
    fn draw(&mut self, graph: &mut DiagramGraph, u: &Usage, owner_qn: &str, owner: &RawElement, parent: &str, level: usize) {
        let (elements, resolver) = (self.elements, self.resolver);
        let qn = format!("{owner_qn}::{}", u.name);
        let id = fresh_id(graph, derived_shape_id(&qn));
        let type_label = u.definition.map(display_name);
        let mut label = match &type_label {
            Some(t) => format!("{} : {t}", u.name),
            None => u.name.clone(),
        };
        if let Some(m) = u.multiplicity.as_deref() {
            if m != "1" {
                label.push_str(&format!(" [{m}]"));
            }
        }
        let mut node = match u.element {
            Some(e) => block_node(id.clone(), e, NodeKind::Block, Some(parent.to_string()), label, elements, resolver),
            None => {
                let mut n = block_node(id.clone(), owner, NodeKind::Block, Some(parent.to_string()), label, elements, resolver);
                n.element_ref = qn.clone();
                n.is_abstract = false;
                // An inline `features:` usage has no `metadata:` of its own.
                n.banners = Vec::new();
                n
            }
        };
        node.element_type = Some("Part".to_string());
        // an actor outside the product, or a part of the test set-up, says so (GH #337); the latter is drawn dashed
        node.stereotype = Some(match u.role.as_deref() {
            Some("external") => "actor",
            Some("environment") => "environment",
            _ => "part",
        }.to_string());
        let def_banners = u.definition.map(|d| super::super::banners_of(d, elements, resolver)).unwrap_or_default();
        node.mark = usage_mark(u, &node.banners.iter().chain(def_banners.iter()).cloned().collect::<Vec<_>>());
        graph.nodes.push(node);
        self.drawn.push(Drawn {
            id: id.clone(),
            definition: u.definition.map(|d| d.qualified_name.clone()),
            element_ref: qn.clone(),
            domain: u.element.and_then(|e| e.frontmatter.domain.clone()).or_else(|| u.definition.and_then(|d| d.frontmatter.domain.clone())),
            tags: u.element.into_iter().chain(u.definition).flat_map(|e| e.frontmatter.tags.clone().unwrap_or_default()).collect(),
            features: {
                let mut exprs: Vec<serde_yaml::Value> = u.applies_when.iter().map(|a| serde_yaml::Value::String(a.clone())).collect();
                exprs.extend(u.element.into_iter().chain(u.definition).filter_map(|e| e.frontmatter.applies_when.clone()));
                exprs.iter().filter_map(|v| crate::variability::applies_when_expr(v).ok().flatten()).flat_map(|x| x.positive_operands()).collect()
            },
        });

        // Ports: the usage's own, then its definition's (own wins on a name clash).
        let def_ports: Vec<Feature> = u
            .definition
            .map(|d| features_of(d, elements, resolver).into_iter().filter(|f| f.role == FeatureRole::Port).collect())
            .unwrap_or_default();
        let mut port_names: Vec<&str> = Vec::new();
        for f in u.own_ports.iter().chain(def_ports.iter()) {
            if port_names.contains(&f.name.as_str()) {
                continue;
            }
            port_names.push(&f.name);
            let pid = fresh_id(graph, derived_shape_id(&format!("{qn}::{}", f.name)));
            graph.nodes.push(port_node(pid, &qn, f, &id));
        }

        // Nested usages (GH #336): the usage element's and its definition's part usages, inline or child
        // `Part` files. A definition already open on this path is not opened again.
        let def_qn = u.definition.map(|d| d.qualified_name.clone());
        let cyclic = def_qn.as_ref().is_some_and(|d| self.path.contains(d));
        if level < self.depth && !cyclic {
            let mut inner: Vec<(Usage, &RawElement)> = u.element.map(|e| usages_of(e, true, elements, resolver).into_iter().map(|v| (v, e)).collect()).unwrap_or_default();
            if let Some(d) = u.definition {
                for v in usages_of(d, true, elements, resolver) {
                    if !inner.iter().any(|(w, _)| w.name == v.name) {
                        inner.push((v, d));
                    }
                }
            }
            self.path.extend(def_qn.clone());
            for (v, o) in &inner {
                self.draw(graph, v, &qn, o, &id, level + 1);
            }
            if def_qn.is_some() {
                self.path.pop();
            }
            for owner in u.definition.into_iter().chain(u.element) {
                connect(graph, owner, &qn);
            }
        }
    }
}

pub fn generate(
    graph: &mut DiagramGraph,
    subject: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    filters: &Filters,
    issues: &mut Vec<Issue>,
) {
    let group_by_domain = match filters.group_by.as_deref() {
        None => false,
        Some("domain") => true,
        Some(other) => {
            issues.push(w420(format!("`groupBy` '{other}' is not a grouping (the only one is: domain) — ignored")));
            false
        }
    };
    let Some(st) = subject.frontmatter.element_type.as_ref() else { return };
    if !matches!(st, ElementType::PartDef | ElementType::Part | ElementType::ItemDef | ElementType::Item) {
        issues.push(w418(format!(
            "`subject` '{}' is a {} — an IBD subject must be a PartDef or Part",
            subject.qualified_name,
            st.name()
        )));
        return;
    }
    let sq = subject.qualified_name.as_str();

    // ── the boundary and its ports ──────────────────────────────────────
    let boundary_id = derived_shape_id(sq);
    graph.nodes.push(block_node(boundary_id.clone(), subject, NodeKind::Boundary, None, display_name(subject), elements, resolver));
    let subject_features = features_of(subject, elements, resolver);
    // A `Part` subject also carries the ports of its definition.
    let subject_def_ports: Vec<Feature> = if matches!(st, ElementType::Part | ElementType::Item) {
        yaml_strings(subject.frontmatter.typed_by.as_ref())
            .first()
            .and_then(|t| resolver.resolve_ref(elements, t))
            .map(|d| features_of(d, elements, resolver).into_iter().filter(|f| f.role == FeatureRole::Port).collect())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    for f in subject_features.iter().chain(subject_def_ports.iter()).filter(|f| f.role == FeatureRole::Port) {
        let qn = format!("{sq}::{}", f.name);
        let id = derived_shape_id(&qn);
        if graph.node(&id).is_none() {
            graph.nodes.push(port_node(id, sq, f, &boundary_id));
        }
    }

    // ── owned part usages ───────────────────────────────────────────────
    let mut usages = usages_of(subject, true, elements, resolver);
    let candidates: Vec<(String, String)> = usages.iter().map(|u| (format!("{sq}::{}", u.name), u.name.clone())).collect();
    filters.unmatched_issues(&candidates, issues);
    usages.retain(|u| filters.keeps(&format!("{sq}::{}", u.name), &u.name));

    // the subject (and a `Part` subject's definition) is open on every path
    let mut path = vec![sq.to_string()];
    path.extend(yaml_strings(subject.frontmatter.typed_by.as_ref()).first().and_then(|t| resolver.resolve_ref(elements, t)).map(|d| d.qualified_name.clone()));
    let mut ctx = Ctx { elements, resolver, depth: filters.depth.unwrap_or(1).clamp(1, MAX_DEPTH), drawn: Vec::new(), path };
    for u in &usages {
        ctx.draw(graph, u, sq, subject, &boundary_id, 1);
    }

    // ── edges from the subject's connection lists ───────────────────────
    connect(graph, subject, sq);

    // ── deployment containment (GH #336) ────────────────────────────────
    let deployed = deploy(graph, &ctx.drawn, elements, resolver);

    // ── domain frames (GH #336) ─────────────────────────────────────────
    if group_by_domain {
        frame_by_domain(graph, &ctx.drawn, &deployed, &boundary_id, sq);
    }
    parents_first_and_view(graph, &ctx.drawn, filters, elements, issues);
}

/// Edges from `owner`'s connection lists, its chains read relative to the usage `prefix` (the subject
/// itself at the top). A chain `a.b` names port `b` of usage `a`; a single segment names a usage or a
/// boundary port. Resolution is by the deterministic ids; a chain to a node not drawn makes no edge.
fn connect(graph: &mut DiagramGraph, owner: &RawElement, prefix: &str) {
    let chain_to_id = |graph: &DiagramGraph, chain: &str| -> Option<String> {
        let qn = format!("{prefix}::{}", chain.trim().replace('.', "::"));
        let id = derived_shape_id(&qn);
        graph.node(&id).map(|_| id)
    };
    let mut taken: HashSet<String> = graph.edges.iter().map(|e| e.id.clone()).collect();
    let mut edges = Vec::new();
    let lists: [(Option<&Vec<serde_yaml::Value>>, EdgeKind); 4] = [
        (owner.frontmatter.connections.as_ref(), EdgeKind::Connection),
        (owner.frontmatter.flow_connections.as_ref(), EdgeKind::Flow),
        (owner.frontmatter.binding_connections.as_ref(), EdgeKind::Binding),
        (owner.frontmatter.succession_connections.as_ref(), EdgeKind::Succession),
    ];
    for (list, kind) in lists {
        let Some(list) = list else { continue };
        for entry in list {
            let (ends, name): (Vec<String>, Option<String>) = match kind {
                EdgeKind::Succession => {
                    let m = entry.as_mapping();
                    let after = m.and_then(|m| map_str(m, "after"));
                    let before = m.and_then(|m| map_str(m, "before"));
                    match (after, before) {
                        (Some(a), Some(b)) => (vec![a.to_string(), b.to_string()], m.and_then(|m| map_str(m, "name")).map(str::to_string)),
                        _ => continue,
                    }
                }
                _ => match parse_entry(entry) {
                    Some(p) => {
                        let name = entry
                            .as_mapping()
                            .and_then(|m| map_str(m, "name").map(str::to_string))
                            .or_else(|| p.typed_by.as_deref().map(|t| short_name(t).to_string()));
                        (p.endpoints.into_iter().map(|e| e.chain).collect(), name)
                    }
                    None => continue,
                },
            };
            if ends.len() < 2 {
                continue;
            }
            let (Some(src), Some(tgt)) = (chain_to_id(graph, &ends[0]), chain_to_id(graph, &ends[1])) else { continue };
            let base = format!("e-{}-{src}-{tgt}", kind.as_str());
            let id = (1..).map(|n| if n == 1 { base.clone() } else { format!("{base}-{n}") }).find(|id| !taken.contains(id)).unwrap();
            taken.insert(id.clone());
            // what it carries (GH #335): the entry's own `flowKind:`, else either port's. A binding equates two
            // features and a succession orders them; neither carries anything, so they keep their own look.
            let flow = matches!(kind, EdgeKind::Connection | EdgeKind::Flow)
                .then(|| {
                    entry
                        .as_mapping()
                        .and_then(|m| map_str(m, "flowKind"))
                        .and_then(FlowKind::parse)
                        .or_else(|| graph.node(&src).and_then(|n| n.flow))
                        .or_else(|| graph.node(&tgt).and_then(|n| n.flow))
                })
                .flatten();
            edges.push(Edge {
                id,
                element_ref: Some(owner.qualified_name.clone()),
                source: src,
                target: tgt,
                kind,
                label: name,
                waypoints: None,
                flow,
                faded: false,
            });
        }
    }
    graph.edges.extend(edges);
}

/// Whether node `a` is `b` or inside it.
fn within(graph: &DiagramGraph, a: &str, b: &str) -> bool {
    let mut cur = Some(a.to_string());
    while let Some(c) = cur {
        if c == b {
            return true;
        }
        cur = graph.node(&c).and_then(|n| n.parent.clone());
    }
    false
}

/// The ids of `id` and its ancestors, innermost first.
fn ancestry(graph: &DiagramGraph, id: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = Some(id.to_string());
    while let Some(c) = cur {
        if out.contains(&c) {
            break;
        }
        cur = graph.node(&c).and_then(|n| n.parent.clone());
        out.push(c);
    }
    out
}

/// Move each drawn deployment package (`isDeploymentPackage: true`) inside the node it is allocated to
/// (any §12.9 form): a usage of the target definition, else the innermost drawn usage whose definition
/// contains one. Among several candidates the one nearest the package wins (the deepest common ancestor with
/// it, so each board's firmware goes into that board's MCU), then one not yet hosting a package of the same
/// definition (so `fwA`, `fwB` pair with `m1`, `m2`), then the first drawn. Returns the ids moved.
fn deploy(graph: &mut DiagramGraph, drawn: &[Drawn], elements: &[RawElement], resolver: &Resolver) -> Vec<String> {
    let edges = crate::validator::allocation_edges(elements, resolver);
    if edges.is_empty() {
        return Vec::new();
    }
    let is_package = |q: &str| elements.iter().any(|e| e.qualified_name == q && e.frontmatter.is_deployment_package == Some(true));
    // Definitions that contain (through part usages, inline or child `Part` files, any depth) a usage typed by `target`.
    let contains = |def: &str, target: &str| -> bool {
        let mut seen: Vec<String> = Vec::new();
        let mut todo = vec![def.to_string()];
        while let Some(d) = todo.pop() {
            if seen.contains(&d) {
                continue;
            }
            seen.push(d.clone());
            let Some(e) = elements.iter().find(|e| e.qualified_name == d) else { continue };
            for u in usages_of(e, true, elements, resolver) {
                for t in u.definition.into_iter().chain(u.element) {
                    if t.qualified_name == target {
                        return true;
                    }
                    todo.push(t.qualified_name.clone());
                }
            }
        }
        false
    };
    let mut moved: Vec<String> = Vec::new();
    // (host id, package definition) pairs already placed
    let mut hosting: Vec<(String, Option<String>)> = Vec::new();
    for (from, to) in &edges {
        let sources: Vec<&Drawn> =
            drawn.iter().filter(|d| (d.definition.as_deref() == Some(from.as_str()) || d.element_ref == *from) && (is_package(from) || d.definition.as_deref().is_some_and(is_package))).collect();
        for src in sources {
            if moved.contains(&src.id) {
                continue;
            }
            let src_up = ancestry(graph, &src.id);
            let usable = |d: &&Drawn| d.id != src.id && !within(graph, &d.id, &src.id);
            let direct: Vec<&Drawn> = drawn.iter().filter(usable).filter(|d| d.definition.as_deref() == Some(to.as_str()) || d.element_ref == *to).collect();
            let candidates: Vec<&Drawn> = if direct.is_empty() {
                drawn.iter().filter(usable).filter(|d| d.definition.as_deref().is_some_and(|def| contains(def, to))).collect()
            } else {
                direct
            };
            // (common-ancestor depth, own depth, free) — larger is better; the first drawn wins a tie
            let score = |d: &Drawn| {
                let up = ancestry(graph, &d.id);
                let common = up.iter().find(|a| src_up.contains(a)).map(|a| ancestry(graph, a).len()).unwrap_or(0);
                let free = !hosting.iter().any(|(h, def)| *h == d.id && *def == src.definition);
                (common, up.len(), free)
            };
            let mut best: Option<(&Drawn, (usize, usize, bool))> = None;
            for c in candidates {
                let sc = score(c);
                if best.as_ref().map_or(true, |(_, b)| sc > *b) {
                    best = Some((c, sc));
                }
            }
            if let Some((host, _)) = best {
                if let Some(n) = graph.nodes.iter_mut().find(|n| n.id == src.id) {
                    n.parent = Some(host.id.clone());
                    moved.push(src.id.clone());
                    hosting.push((host.id.clone(), src.definition.clone()));
                }
            }
        }
    }
    moved
}

/// Frame the subject's direct parts by domain (GH #336): one `Swimlane` per domain in the order system,
/// hardware, software, then any other value; a part with no domain stays in the boundary. A deployment
/// package moved inside its hardware stays there, and carries its own domain as a banner.
fn frame_by_domain(graph: &mut DiagramGraph, drawn: &[Drawn], deployed: &[String], boundary_id: &str, sq: &str) {
    let mut domains: Vec<String> = Vec::new();
    for d in drawn {
        let top = graph.node(&d.id).and_then(|n| n.parent.as_deref()) == Some(boundary_id);
        if let (true, Some(dom)) = (top, d.domain.as_ref()) {
            if !domains.contains(dom) {
                domains.push(dom.clone());
            }
        }
    }
    let rank = |d: &str| ["system", "hardware", "software"].iter().position(|x| *x == d).unwrap_or(3);
    domains.sort_by(|a, b| rank(a).cmp(&rank(b)).then(a.cmp(b)));
    for dom in &domains {
        let id = fresh_id(graph, derived_shape_id(&format!("{sq}::domain::{dom}")));
        graph.nodes.push(Node {
            id: id.clone(),
            element_ref: sq.to_string(),
            resolved: true,
            element_type: None,
            kind: NodeKind::Swimlane,
            label: dom.clone(),
            stereotype: Some("domain".to_string()),
            parent: Some(boundary_id.to_string()),
            direction: None,
            side: None,
            lines: Vec::new(),
            is_abstract: false,
            pin: None,
            banners: Vec::new(),
            feature: None,
            mark: None,
            flow: None,
            faded: false,
        });
        for d in drawn.iter().filter(|d| d.domain.as_deref() == Some(dom.as_str())) {
            if let Some(n) = graph.nodes.iter_mut().find(|n| n.id == d.id && n.parent.as_deref() == Some(boundary_id)) {
                n.parent = Some(id.clone());
            }
        }
    }
    for d in drawn.iter().filter(|d| deployed.contains(&d.id)) {
        let Some(dom) = d.domain.clone() else { continue };
        if let Some(n) = graph.nodes.iter_mut().find(|n| n.id == d.id) {
            if !n.banners.contains(&dom) {
                n.banners.push(dom);
            }
        }
    }
}

/// Parents before children, then the view (GH #338): with the mapping form of `include:`, the parts outside it, their
/// ports and the edges outside it are marked faded. A part is in the view when it passes every key given: its
/// domain is listed, it carries a listed tag, its `appliesWhen:` names a listed feature (or one beneath it), and, with
/// `kinds:`, an edge of a listed flow kind reaches it or its ports. An edge is in the view when its kind is listed
/// (with `kinds:`) and its ends are on parts in the view or on the boundary. The parts that contain a part in the view
/// stay as its frame.
fn parents_first_and_view(graph: &mut DiagramGraph, drawn: &[Drawn], filters: &Filters, elements: &[RawElement], issues: &mut Vec<Issue>) {
    graph.parents_first();
    let Some(view) = filters.view.as_ref() else { return };
    for k in view.unknown.keys() {
        issues.push(w421(format!("`include` has an unknown key '{k}' (known: members, kinds, domains, tags, features, depth) — ignored")));
    }
    for k in &view.malformed {
        let what = if k == "depth" { "a whole number" } else { "a list of names" };
        issues.push(w421(format!("`include.{k}` must be {what} — ignored")));
    }
    let mut kinds: Option<Vec<FlowKind>> = None;
    if let Some(ks) = &view.kinds {
        let mut v = Vec::new();
        for k in ks {
            match FlowKind::parse(k) {
                Some(f) => v.push(f),
                None => issues.push(w421(format!(
                    "`include.kinds` entry '{k}' is not a flow kind (the kinds are: {}) — ignored",
                    FlowKind::ALL.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")
                ))),
            }
        }
        // a key whose every entry is ignored is ignored as a whole, rather than selecting nothing
        kinds = (!v.is_empty()).then_some(v);
    }
    let mut domains: Option<Vec<String>> = None;
    if let Some(ds) = &view.domains {
        let mut v = Vec::new();
        for d in ds {
            if ["system", "hardware", "software"].contains(&d.as_str()) {
                v.push(d.clone());
            } else {
                issues.push(w421(format!("`include.domains` entry '{d}' is not a domain (system, hardware, software) — ignored")));
            }
        }
        domains = (!v.is_empty()).then_some(v);
    }
    let alias = crate::variability::feature_id_to_qname(elements);
    let canon = |f: &str| crate::variability::canon_feature_ref(f, &alias);
    let mut wanted: Option<Vec<String>> = None;
    if let Some(fs) = &view.features {
        let mut v = Vec::new();
        for w in fs.iter().map(|f| canon(f)) {
            if elements.iter().any(|e| e.qualified_name == w && e.frontmatter.element_type == Some(ElementType::FeatureDef)) {
                v.push(w);
            } else {
                issues.push(w421(format!("`include.features` entry '{w}' is not a feature — ignored")));
            }
        }
        wanted = (!v.is_empty()).then_some(v);
    }
    let criteria = kinds.is_some() || domains.is_some() || view.tags.is_some() || wanted.is_some();
    if !criteria {
        return;
    }
    // the part (a drawn usage) a node belongs to: itself, or the usage a port sits on
    let owner = |g: &DiagramGraph, id: &str| -> Option<String> {
        let n = g.node(id)?;
        if n.kind == NodeKind::Port { n.parent.clone() } else { Some(n.id.clone()) }
    };
    let in_kinds = |e: &Edge| kinds.as_ref().map_or(true, |ks| e.flow.is_some_and(|f| ks.contains(&f)));
    let mut reached: HashSet<String> = HashSet::new();
    if kinds.is_some() {
        for e in graph.edges.iter().filter(|e| in_kinds(e)) {
            reached.extend(owner(graph, &e.source));
            reached.extend(owner(graph, &e.target));
        }
    }
    let focused: HashSet<String> = drawn
        .iter()
        .filter(|d| domains.as_ref().map_or(true, |ds| d.domain.as_ref().is_some_and(|x| ds.contains(x))))
        .filter(|d| view.tags.as_ref().map_or(true, |ts| d.tags.iter().any(|t| ts.contains(t))))
        .filter(|d| {
            wanted.as_ref().map_or(true, |ws| {
                d.features.iter().map(|f| canon(f)).any(|f| ws.iter().any(|w| f == *w || f.starts_with(&format!("{w}::"))))
            })
        })
        .filter(|d| kinds.is_none() || reached.contains(&d.id))
        .map(|d| d.id.clone())
        .collect();
    // kept: the parts in the view and every node that contains one; the boundary and its ports
    let mut kept: HashSet<String> = HashSet::new();
    for id in &focused {
        let mut cur = Some(id.clone());
        while let Some(c) = cur {
            if !kept.insert(c.clone()) {
                break;
            }
            cur = graph.node(&c).and_then(|n| n.parent.clone());
        }
    }
    let boundary: Option<String> = graph.nodes.iter().find(|n| n.kind == NodeKind::Boundary).map(|n| n.id.clone());
    let fade: HashSet<String> = graph
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Block | NodeKind::Swimlane) && !kept.contains(&n.id))
        .map(|n| n.id.clone())
        .collect();
    let on_view = |g: &DiagramGraph, id: &str| -> bool {
        match owner(g, id) {
            Some(o) => focused.contains(&o) || Some(&o) == boundary.as_ref(),
            None => false,
        }
    };
    let faded_edges: Vec<bool> = graph.edges.iter().map(|e| !(in_kinds(e) && on_view(graph, &e.source) && on_view(graph, &e.target))).collect();
    for n in graph.nodes.iter_mut() {
        let own = if n.kind == NodeKind::Port { n.parent.clone().unwrap_or_default() } else { n.id.clone() };
        n.faded = fade.contains(&own) || (n.kind == NodeKind::Swimlane && fade.contains(&n.id));
    }
    for (e, f) in graph.edges.iter_mut().zip(faded_edges) {
        e.faded = f;
    }
}

#[cfg(test)]
mod tests {
    use super::super::testkit::*;
    use super::*;
    use crate::vis::ir::PortDirection;

    #[test]
    fn partdef_subject_yields_boundary_parts_ports_and_edges() {
        let d = diagram("IBD", "Sys::PowerSystem", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        let b = g.node("s-sys-powersystem").unwrap();
        assert_eq!(b.kind, NodeKind::Boundary);
        assert_eq!(b.stereotype.as_deref(), Some("part def"));
        // Boundary port.
        let main_out = g.node("s-sys-powersystem-mainout").unwrap();
        assert_eq!(main_out.kind, NodeKind::Port);
        assert_eq!(main_out.parent.as_deref(), Some("s-sys-powersystem"));
        assert_eq!(main_out.direction, Some(PortDirection::Out));
        // Owned parts: two inline features plus the child Part element.
        let parts: Vec<(&str, &str)> = g
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Block)
            .map(|n| (n.id.as_str(), n.label.as_str()))
            .collect();
        assert_eq!(
            parts,
            vec![
                ("s-sys-powersystem-engine", "engine : Engine"),
                ("s-sys-powersystem-motor", "motor : Motor [2]"),
                ("s-sys-powersystem-aux", "aux : Motor"),
            ]
        );
        assert!(g.nodes.iter().filter(|n| n.kind == NodeKind::Block).all(|n| n.parent.as_deref() == Some("s-sys-powersystem")));
        // Ports come from the typing definition, with direction.
        let p = g.node("s-sys-powersystem-engine-powerout").unwrap();
        assert_eq!(p.parent.as_deref(), Some("s-sys-powersystem-engine"));
        assert_eq!(p.direction, Some(PortDirection::Out));
        assert_eq!(p.element_ref, "Sys::PowerSystem::engine::powerOut");
        assert_eq!(g.node("s-sys-powersystem-aux-powerin").unwrap().direction, Some(PortDirection::In));
        // Edges: the connection port→port and the binding to the boundary port.
        let edges: Vec<(EdgeKind, &str, &str, Option<&str>)> =
            g.edges.iter().map(|e| (e.kind, e.source.as_str(), e.target.as_str(), e.label.as_deref())).collect();
        assert_eq!(
            edges,
            vec![
                (EdgeKind::Connection, "s-sys-powersystem-engine-powerout", "s-sys-powersystem-motor-powerin", Some("PowerLink")),
                (EdgeKind::Binding, "s-sys-powersystem-motor-powerin", "s-sys-powersystem-mainout", None),
            ]
        );
        assert_eq!(g.edges[0].id, "e-connection-s-sys-powersystem-engine-powerout-s-sys-powersystem-motor-powerin");
        assert_eq!(g.layout_hints.hierarchical, true);
    }

    #[test]
    fn part_usage_subject_carries_its_definitions_ports() {
        let d = diagram("IBD", "Sys::PowerSystem::aux", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(g.node("s-sys-powersystem-aux").unwrap().kind, NodeKind::Boundary);
        let p = g.node("s-sys-powersystem-aux-powerin").unwrap();
        assert_eq!(p.kind, NodeKind::Port);
        assert_eq!(p.direction, Some(PortDirection::In));
    }

    #[test]
    fn exclude_drops_a_usage_and_its_edges_and_the_generator_never_invents_a_port() {
        let d = diagram("IBD", "Sys::PowerSystem", |fm| fm.exclude = Some(vec!["motor".into()]));
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert!(g.node("s-sys-powersystem-motor").is_none());
        assert!(g.edges.is_empty(), "both edges touched motor.powerIn: {:?}", g.edges);
    }

    #[test]
    fn include_naming_no_usage_is_w417() {
        let d = diagram("IBD", "Sys::PowerSystem", |fm| fm.include = Some(vec!["engine".into(), "turbine".into()]));
        let (g, issues) = derive_it(&d);
        assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Block).count(), 1);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "W417");
        assert!(issues[0].message.contains("'turbine'"));
    }

    #[test]
    fn wrong_subject_type_is_w418() {
        let d = diagram("IBD", "Sys", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(g.nodes.is_empty());
        assert_eq!(issues[0].code, "W418");
        assert!(issues[0].message.contains("Package"));
    }

    #[test]
    fn a_chain_to_an_unknown_port_produces_no_edge() {
        let mut d = diagram("IBD", "Sys::PowerSystem", |_| {});
        // Subject override with a dangling chain is awkward through the shared
        // model, so derive against a subject copy whose connections point nowhere.
        let mut elements = model();
        let ps = elements.iter_mut().find(|e| e.qualified_name == "Sys::PowerSystem").unwrap();
        ps.frontmatter.connections = Some(yaml_list("- {from: engine.powerOut, to: ghost.powerIn}\n"));
        ps.frontmatter.binding_connections = None;
        elements.push(d.clone());
        let resolver = Resolver::new(&elements);
        d.frontmatter.subject = Some("Sys::PowerSystem".into());
        let (g, issues) = super::super::derive(&d, crate::vis::ir::DiagramKind::Ibd, &elements, &resolver);
        assert!(issues.is_empty());
        assert!(g.edges.is_empty());
    }
}
