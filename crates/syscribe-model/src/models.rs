//! `models:` — the trace link from an executable part to the architecture element it is a behavioural model of
//! (GH #330), and the correspondence checks it enables: ports, connections, and whether a TestCase's subject can be
//! evidence for the requirements it verifies. All of it is dormant unless some part declares `models:`.
//!
//! A *binding* is one place that says "these executable ports model those architecture elements": a `PartDef`/`Part`
//! with `models:`, or a part usage (an entry of a composite's `features:`) that carries its own `models:` or
//! `portMap:`. A usage-level `portMap:` lets two instances of one executable part model two chains of one element.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde_yaml::{Mapping, Value as Y};

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::sm_exec::field;
use crate::system::{self, Dir, PortSpec};

/// `(code, file, message)`.
pub type Finding = (&'static str, String, String);

fn is_part(e: &RawElement) -> bool {
    matches!(
        e.frontmatter.element_type,
        Some(ElementType::PartDef) | Some(ElementType::Part)
    )
}

fn strings(v: Option<&Y>) -> Vec<String> {
    match v {
        Some(Y::String(s)) => vec![s.clone()],
        Some(Y::Sequence(xs)) => xs
            .iter()
            .filter_map(|x| x.as_str().map(String::from))
            .collect(),
        _ => Vec::new(),
    }
}

/// A `PartDef`/`Part` that has a `domain:` on itself or along its definition chain.
pub fn is_architecture(elements: &[RawElement], resolver: &Resolver, e: &RawElement) -> bool {
    is_part(e)
        && system::chain(elements, resolver, e)
            .iter()
            .any(|d| d.frontmatter.domain.is_some())
}

/// The qualified names an architecture element is known by: itself and, for a `Part`, the definition it is typed by.
/// A `models:` target and a connection end match when their key sets meet.
fn keys(elements: &[RawElement], resolver: &Resolver, e: &RawElement) -> BTreeSet<String> {
    let mut k = BTreeSet::from([e.qualified_name.clone()]);
    if matches!(e.frontmatter.element_type, Some(ElementType::Part)) {
        for t in e.frontmatter.typed_by.iter().flat_map(|v| strings(Some(v))) {
            if let Some(d) = resolver.resolve_ref(elements, &t) {
                k.insert(d.qualified_name.clone());
            }
        }
    }
    k
}

/// The architecture ports each executable port corresponds to.
type PortMap = BTreeMap<String, Vec<String>>;

/// `portMap:` as executable port → architecture port(s): a name, a list (one executable port standing for several
/// architecture ports, e.g. a trip line that fans out), or `null` (no counterpart on purpose, an empty list here).
fn port_map(m: Option<&Mapping>) -> PortMap {
    m.into_iter()
        .flatten()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), strings(Some(v)))))
        .collect()
}

/// The architecture ports `port` corresponds to under `pm`: by default the port of the same name.
fn mapped(pm: &PortMap, port: &str) -> Vec<String> {
    pm.get(port)
        .cloned()
        .unwrap_or_else(|| vec![port.to_string()])
}

/// The architecture ports executable port `port` of `def` corresponds to: its `portMap:` entry; else the port of the same
/// name; else, when it is typed by a `PortDef`, the one port of the modelled elements typed by the same `PortDef` (GH #331:
/// a shared interface makes `portMap:` unnecessary).
fn counterparts(
    elements: &[RawElement],
    resolver: &Resolver,
    def: &RawElement,
    targets: &[&RawElement],
    pm: &PortMap,
    port: &str,
    index: &HashMap<String, Vec<PortSpec>>,
    pctx: &crate::port_types::Ctx,
) -> Vec<String> {
    if pm.contains_key(port) {
        return mapped(pm, port);
    }
    let same = targets.iter().any(|t| {
        system::def_ports_in(elements, resolver, t, index)
            .iter()
            .any(|q| q.name == port)
    });
    if !same {
        if let Some(ty) = crate::port_types::port_type_of(elements, resolver, pctx, def, port) {
            let typed: BTreeSet<String> = targets
                .iter()
                .flat_map(|t| crate::port_types::typed_ports(elements, resolver, pctx, t).to_vec())
                .filter(|p| (&p.ty, p.conjugated) == (&ty.0, ty.1))
                .map(|p| p.name)
                .collect();
            if typed.len() == 1 {
                return typed.into_iter().collect();
            }
        }
    }
    vec![port.to_string()]
}

struct Binding<'a> {
    file: String,
    /// How a finding names the holder: `'Pkg::Monitor'` or `part 'sensorSw' of 'Pkg::Chain'`.
    holder: String,
    def: &'a RawElement,
    targets: Vec<&'a RawElement>,
    port_map: PortMap,
    abstracts: Vec<String>,
}

/// The nearest `models:`/`portMap:`/`abstracts:` along a definition chain.
fn def_binding_fields<'a>(
    elements: &'a [RawElement],
    resolver: &Resolver,
    def: &'a RawElement,
) -> (Vec<String>, PortMap, Vec<String>) {
    let ch = system::chain(elements, resolver, def);
    let models = ch
        .iter()
        .find_map(|d| d.frontmatter.models.clone().filter(|m| !m.is_empty()))
        .unwrap_or_default();
    let pm = ch
        .iter()
        .find_map(|d| d.frontmatter.port_map.as_ref())
        .map(|m| port_map(Some(m)))
        .unwrap_or_default();
    let abstracts = ch
        .iter()
        .flat_map(|d| d.frontmatter.abstracts.clone().unwrap_or_default())
        .collect();
    (models, pm, abstracts)
}

/// The modelling a part usage (a composite's `features:` entry typed by `def`) carries: its own `models:`/`portMap:`
/// over the definition's.
fn usage_binding<'a>(
    elements: &'a [RawElement],
    resolver: &Resolver,
    def: &'a RawElement,
    entry: &Mapping,
) -> (Vec<String>, PortMap, Vec<String>) {
    let (mut models, mut pm, mut abstracts) = def_binding_fields(elements, resolver, def);
    let own = strings(field(entry, "models"));
    if !own.is_empty() {
        models = own;
    }
    if let Some(m) = field(entry, "portMap").and_then(|v| v.as_mapping()) {
        pm = port_map(Some(m));
    }
    abstracts.extend(strings(field(entry, "abstracts")));
    (models, pm, abstracts)
}

/// Whether any element declares `models:`, or a part usage carries `models:`/`portMap:`.
pub fn active(elements: &[RawElement]) -> bool {
    elements.iter().any(|e| {
        e.frontmatter.models.as_ref().is_some_and(|m| !m.is_empty())
            || e.frontmatter.features.iter().flatten().any(|f| {
                f.as_mapping()
                    .is_some_and(|m| field(m, "models").is_some() || field(m, "portMap").is_some())
            })
    })
}

/// Resolve `models:` targets, reporting E990 for one that does not resolve or is not an architecture element.
fn resolve_targets<'a>(
    elements: &'a [RawElement],
    resolver: &Resolver,
    refs: &[String],
    holder: &str,
    file: &str,
    out: &mut Vec<Finding>,
) -> Vec<&'a RawElement> {
    let mut targets = Vec::new();
    for r in refs {
        match resolver.resolve_ref(elements, r) {
            None => out.push(("E990", file.to_string(), format!("{holder}: `models: {r}` does not resolve"))),
            Some(t) if !is_architecture(elements, resolver, t) => out.push((
                "E990",
                file.to_string(),
                format!("{holder}: `models: {r}` is a {}, not an architecture element (a PartDef or Part with a `domain:`)", t.frontmatter.element_type.as_ref().map(|t| format!("{t:?}")).unwrap_or_else(|| "element of no type".into())),
            )),
            Some(t) => {
                if !targets.iter().any(|x: &&RawElement| x.qualified_name == t.qualified_name) {
                    targets.push(t);
                }
            }
        }
    }
    targets
}

fn bindings<'a>(
    elements: &'a [RawElement],
    resolver: &Resolver,
    out: &mut Vec<Finding>,
) -> Vec<Binding<'a>> {
    let mut v = Vec::new();
    // definitions every one of whose uses can map its ports: a part usage carrying its own `portMap:` (one-to-many)
    let usage_mapped: std::collections::HashSet<&str> = elements
        .iter()
        .filter(|e| is_part(e))
        .flat_map(|e| e.frontmatter.features.iter().flatten())
        .filter_map(|f| f.as_mapping())
        .filter(|m| field(m, "portMap").is_some())
        .filter_map(|m| {
            field(m, "typedBy")
                .and_then(|t| t.as_str())
                .and_then(|t| resolver.resolve_ref(elements, t))
        })
        .map(|d| d.qualified_name.as_str())
        .collect();
    for e in elements {
        let own = e.frontmatter.models.clone().unwrap_or_default();
        let holder = format!("'{}'", e.qualified_name);
        if !own.is_empty() && !is_part(e) {
            out.push((
                "E990",
                e.file_path.clone(),
                format!("{holder}: `models:` is recognised on a PartDef or Part only"),
            ));
        } else if is_part(e) {
            // its own `models:` (reported here when a target is wrong) or one inherited along its definition chain
            let (models, pm, abstracts) = def_binding_fields(elements, resolver, e);
            if !models.is_empty() {
                let targets = if own.is_empty() {
                    resolve_targets(
                        elements,
                        resolver,
                        &models,
                        &holder,
                        &e.file_path,
                        &mut Vec::new(),
                    )
                } else {
                    resolve_targets(elements, resolver, &own, &holder, &e.file_path, out)
                };
                // with no map of its own, a definition whose usages map their ports is checked through them only
                if !(pm.is_empty() && usage_mapped.contains(e.qualified_name.as_str())) {
                    v.push(Binding {
                        file: e.file_path.clone(),
                        holder,
                        def: e,
                        targets,
                        port_map: pm,
                        abstracts,
                    });
                }
            }
        }
        if !is_part(e) {
            continue;
        }
        for f in e.frontmatter.features.iter().flatten() {
            let Some(m) = f.as_mapping() else { continue };
            if field(m, "models").is_none() && field(m, "portMap").is_none() {
                continue;
            }
            let name = field(m, "name").and_then(|n| n.as_str()).unwrap_or("?");
            let holder = format!("part '{name}' of '{}'", e.qualified_name);
            // GH #332, GH #337: an environment part or an actor is not the product, so it models nothing
            if let Some(role) = system::role_of(m) {
                out.push(("W993", e.file_path.clone(), format!("{holder}: is `role: {role}`, so its `models:`/`portMap:` is ignored; it is not the product and has no architecture counterpart")));
                continue;
            }
            let Some(def) = field(m, "typedBy")
                .and_then(|t| t.as_str())
                .and_then(|t| resolver.resolve_ref(elements, t))
                .filter(|d| is_part(d))
            else {
                out.push(("E990", e.file_path.clone(), format!("{holder}: `models:`/`portMap:` on a feature needs a `typedBy:` that resolves to a PartDef or Part")));
                continue;
            };
            let (models, pm, abstracts) = usage_binding(elements, resolver, def, m);
            if models.is_empty() {
                out.push((
                    "E990",
                    e.file_path.clone(),
                    format!(
                        "{holder}: has a `portMap:` but neither it nor '{}' declares `models:`",
                        def.qualified_name
                    ),
                ));
                continue;
            }
            let targets = if field(m, "models").is_some() {
                resolve_targets(elements, resolver, &models, &holder, &e.file_path, out)
            } else {
                resolve_targets(
                    elements,
                    resolver,
                    &models,
                    &holder,
                    &e.file_path,
                    &mut Vec::new(),
                )
            };
            v.push(Binding {
                file: e.file_path.clone(),
                holder,
                def,
                targets,
                port_map: pm,
                abstracts,
            });
        }
    }
    v
}

fn compatible(a: Dir, b: Dir) -> bool {
    a == b || a == Dir::InOut || b == Dir::InOut
}

fn dir_name(d: Dir) -> &'static str {
    match d {
        Dir::In => "in",
        Dir::Out => "out",
        Dir::InOut => "inout",
    }
}

/// Architecture element qname → the qualified names of the executable parts (and part usages) that model it.
pub fn modelled_by(elements: &[RawElement], resolver: &Resolver) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    if !active(elements) {
        return out;
    }
    for b in bindings(elements, resolver, &mut Vec::new()) {
        for t in &b.targets {
            let list = out.entry(t.qualified_name.clone()).or_default();
            if !list.contains(&b.def.qualified_name) {
                list.push(b.def.qualified_name.clone());
            }
        }
    }
    out
}

/// What a composite subject exercises: for each architecture key, the requirements each part that reaches it leaves
/// out (`abstracts:` entries that resolve to a `Requirement`). An architecture element contained directly leaves out
/// nothing.
#[derive(Debug, Default)]
pub struct Exercised {
    by: BTreeMap<String, Vec<BTreeSet<String>>>,
}

impl Exercised {
    fn add(&mut self, keys: BTreeSet<String>, left_out: &BTreeSet<String>) {
        for k in keys {
            self.by.entry(k).or_default().push(left_out.clone());
        }
    }

    /// Whether some part reaches architecture key `key` at all.
    pub fn reaches(&self, key: &str) -> bool {
        self.by.contains_key(key)
    }

    /// Whether some part reaches `key` without naming requirement `req` (a qualified name) in its `abstracts:`.
    pub fn evidences(&self, key: &str, req: &str) -> bool {
        self.by
            .get(key)
            .is_some_and(|v| v.iter().any(|l| !l.contains(req)))
    }
}

/// The `abstracts:` entries that resolve to a `Requirement`, by qualified name: what a model says it does not exercise.
fn abstracted_requirements(
    elements: &[RawElement],
    resolver: &Resolver,
    abstracts: &[String],
) -> BTreeSet<String> {
    abstracts
        .iter()
        // a stable id or a qualified name only: a port name in `abstracts:` that happens to equal a requirement's
        // display name is not that requirement
        .filter_map(|a| {
            resolver
                .get_by_id(elements, a)
                .or_else(|| resolver.get(elements, a))
        })
        .filter(|r| matches!(r.frontmatter.element_type, Some(ElementType::Requirement)))
        .map(|r| r.qualified_name.clone())
        .collect()
}

/// The architecture elements satisfying each requirement, by the requirement's qualified name (GH #333/#344).
pub fn satisfiers(elements: &[RawElement], resolver: &Resolver) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for e in elements.iter().filter(|e| is_architecture(elements, resolver, e)) {
        for r in e.frontmatter.satisfies.iter().flatten() {
            if let Some(req) = resolver.resolve_ref(elements, r) {
                let list = out.entry(req.qualified_name.clone()).or_default();
                if !list.contains(&e.qualified_name) {
                    list.push(e.qualified_name.clone());
                }
            }
        }
    }
    out
}

/// Whether `reach` (what a subject exercises) can be evidence for requirement `req` (a qualified name) satisfied by the
/// architecture elements `satisfiers`: some part reaches one of them without naming `req` in `abstracts:`. A requirement
/// no architecture element satisfies has nothing to compare, so any subject counts (as for `W992`).
pub fn evidences(elements: &[RawElement], resolver: &Resolver, reach: &Exercised, req: &str, satisfiers: &[String]) -> bool {
    satisfiers.is_empty()
        || satisfiers.iter().filter_map(|s| resolver.get(elements, s)).any(|s| keys(elements, resolver, s).iter().any(|k| reach.evidences(k, req)))
}

/// Whether part definition `def` is, or models, one of `targets` (architecture qualified names): the parts whose
/// machines carry the behaviour of those elements.
pub fn carries(elements: &[RawElement], resolver: &Resolver, def: &RawElement, usage: Option<&Mapping>, targets: &[String]) -> bool {
    let mine: BTreeSet<String> = if is_architecture(elements, resolver, def) {
        keys_up(elements, resolver, def)
    } else {
        let models = match usage {
            Some(m) => usage_binding(elements, resolver, def, m).0,
            None => def_binding_fields(elements, resolver, def).0,
        };
        models.iter().filter_map(|r| resolver.resolve_ref(elements, r)).flat_map(|t| keys_up(elements, resolver, t)).collect()
    };
    targets.iter().any(|t| mine.contains(t))
}

/// [`keys`] plus the supertypes along the definition chain: a requirement satisfied by an abstract base is exercised by a
/// part that models one of its specialisations.
fn keys_up(elements: &[RawElement], resolver: &Resolver, e: &RawElement) -> BTreeSet<String> {
    let mut k = keys(elements, resolver, e);
    k.extend(
        system::chain(elements, resolver, e)
            .iter()
            .map(|d| d.qualified_name.clone()),
    );
    k
}

/// The architecture elements a composite `subject` can exercise: itself and every part reachable through `features:`,
/// with what each models (GH #330; #344 reuses this at scenario level).
pub fn exercised_by(
    elements: &[RawElement],
    resolver: &Resolver,
    subject: &RawElement,
) -> Exercised {
    let mut out = Exercised::default();
    // (definition, usage name) pairs already walked: finite, so the walk ends without a cap
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack: Vec<(&RawElement, Option<Mapping>)> = vec![(subject, None)];
    while let Some((e, usage)) = stack.pop() {
        let key = format!(
            "{}#{}",
            e.qualified_name,
            usage
                .as_ref()
                .and_then(|m| field(m, "name"))
                .and_then(|n| n.as_str())
                .unwrap_or("")
        );
        if !seen.insert(key) {
            continue;
        }
        if is_architecture(elements, resolver, e) {
            out.add(keys_up(elements, resolver, e), &BTreeSet::new());
        }
        let (models, _, abstracts) = match &usage {
            Some(m) => usage_binding(elements, resolver, e, m),
            None => def_binding_fields(elements, resolver, e),
        };
        let left_out = abstracted_requirements(elements, resolver, &abstracts);
        for r in models {
            if let Some(t) = resolver
                .resolve_ref(elements, &r)
                .filter(|t| is_architecture(elements, resolver, t))
            {
                out.add(keys_up(elements, resolver, t), &left_out);
            }
        }
        for f in system::composite_view(elements, resolver, e).features {
            let Some(m) = f.as_mapping() else { continue };
            if field(m, "type").and_then(|t| t.as_str()) == Some("Port") {
                continue;
            }
            // an environment part (GH #332) exercises nothing of the product
            if system::is_outside(m) {
                continue;
            }
            if let Some(d) = field(m, "typedBy")
                .and_then(|t| t.as_str())
                .and_then(|t| resolver.resolve_ref(elements, t))
                .filter(|d| is_part(d))
            {
                stack.push((d, Some(m.clone())));
            }
        }
    }
    out
}

/// What a state-machine `subject` exercises: the parts that exhibit it, and what each models (GH #344).
pub fn exercised_by_machine(elements: &[RawElement], resolver: &Resolver, machine: &RawElement) -> Exercised {
    let mut out = Exercised::default();
    for e in elements.iter().filter(|e| is_part(e)) {
        let ch = system::chain(elements, resolver, e);
        let exhibits = ch.iter().flat_map(|d| d.frontmatter.exhibits_states.iter().flatten()).any(|st| resolver.resolve_ref(elements, st).is_some_and(|m| m.qualified_name == machine.qualified_name));
        if !exhibits {
            continue;
        }
        if is_architecture(elements, resolver, e) {
            out.add(keys_up(elements, resolver, e), &BTreeSet::new());
        }
        let (models, _, abstracts) = def_binding_fields(elements, resolver, e);
        let left_out = abstracted_requirements(elements, resolver, &abstracts);
        for t in models.iter().filter_map(|r| resolver.resolve_ref(elements, r)).filter(|t| is_architecture(elements, resolver, t)) {
            out.add(keys_up(elements, resolver, t), &left_out);
        }
    }
    out
}

/// One architecture connection between two part usages, by the definitions they are typed by.
struct ArchEdge {
    /// The keys of each end's type: a usage typed by a `Part` matches both the Part and its definition.
    a: BTreeSet<String>,
    an: String,
    pa: String,
    b: BTreeSet<String>,
    bn: String,
    pb: String,
    composite: String,
}

/// Whether composite `c` is a verification composite (GH #332): it holds an environment part, or an executable part
/// that models an architecture element. Its connections are not architecture, even between two architecture elements
/// it uses directly, and only in it does an architecture element used directly count as its own counterpart.
fn is_verification_composite(elements: &[RawElement], resolver: &Resolver, features: &[Y]) -> bool {
    features.iter().filter_map(|f| f.as_mapping()).any(|m| {
        system::is_environment(m)
            || field(m, "typedBy")
                .and_then(|t| t.as_str())
                .and_then(|t| resolver.resolve_ref(elements, t))
                .filter(|d| is_part(d) && !is_architecture(elements, resolver, d))
                .is_some_and(|d| !usage_binding(elements, resolver, d, m).0.is_empty())
    })
}

fn arch_edges(elements: &[RawElement], resolver: &Resolver) -> Vec<ArchEdge> {
    let mut v = Vec::new();
    for c in elements.iter().filter(|e| {
        is_part(e)
            && e.frontmatter
                .connections
                .as_ref()
                .is_some_and(|c| !c.is_empty())
    }) {
        let view = system::composite_view(elements, resolver, c);
        let def_of = |part: &str| -> Option<&RawElement> {
            view.features
                .iter()
                .filter_map(|f| f.as_mapping())
                .find(|m| field(m, "name").and_then(|n| n.as_str()) == Some(part))
                .and_then(|m| field(m, "typedBy"))
                .and_then(|t| t.as_str())
                .and_then(|t| resolver.resolve_ref(elements, t))
        };
        if !is_architecture(elements, resolver, c)
            && is_verification_composite(elements, resolver, &view.features)
        {
            continue;
        }
        for conn in &view.connections {
            let Some(m) = conn.as_mapping() else { continue };
            let (Some(from), Some(to)) = (
                field(m, "from").and_then(|v| v.as_str()),
                field(m, "to").and_then(|v| v.as_str()),
            ) else {
                continue;
            };
            let (Some((fa, fp)), Some((ta, tp))) = (system::endpoint(from), system::endpoint(to))
            else {
                continue;
            };
            let (Some(da), Some(db)) = (def_of(fa), def_of(ta)) else {
                continue;
            };
            if is_architecture(elements, resolver, da) && is_architecture(elements, resolver, db) {
                v.push(ArchEdge {
                    a: keys(elements, resolver, da),
                    an: da.qualified_name.clone(),
                    pa: fp.to_string(),
                    b: keys(elements, resolver, db),
                    bn: db.qualified_name.clone(),
                    pb: tp.to_string(),
                    composite: c.qualified_name.clone(),
                });
            }
        }
    }
    v
}

/// Every `models:` finding (E990, E991, W990, W991, W992). Empty unless some part declares `models:`.
pub fn findings(elements: &[RawElement], resolver: &Resolver) -> Vec<Finding> {
    let mut out = role_findings(elements, resolver);
    out.extend(boundary_port_findings(elements, resolver));
    if !active(elements) {
        return out;
    }
    let bs = bindings(elements, resolver, &mut out);

    // E991: each executable port corresponds to a port of a modelled element, in a compatible direction.
    let port_index = system::child_port_index(elements);
    let pctx = crate::port_types::Ctx::new(elements);
    let ports_cache: std::cell::RefCell<HashMap<String, Vec<PortSpec>>> = Default::default();
    let ports = |d: &RawElement| -> Vec<PortSpec> {
        ports_cache
            .borrow_mut()
            .entry(d.qualified_name.clone())
            .or_insert_with(|| system::def_ports_in(elements, resolver, d, &port_index))
            .clone()
    };
    // (architecture qname, port) covered by some executable port
    let mut covered: BTreeSet<(String, String)> = BTreeSet::new();
    for b in bs.iter().filter(|b| !b.targets.is_empty()) {
        let exec = ports(b.def);
        for k in b.port_map.keys() {
            if !exec.iter().any(|p| &p.name == k) {
                out.push((
                    "E991",
                    b.file.clone(),
                    format!(
                        "{}: `portMap:` key '{k}' is not a port of '{}'",
                        b.holder, b.def.qualified_name
                    ),
                ));
            }
        }
        let names = b
            .targets
            .iter()
            .map(|t| format!("'{}'", t.qualified_name))
            .collect::<Vec<_>>()
            .join(", ");
        for p in &exec {
            for m in counterparts(
                elements,
                resolver,
                b.def,
                &b.targets,
                &b.port_map,
                &p.name,
                &port_index,
                &pctx,
            ) {
                let hits: Vec<(&RawElement, PortSpec)> = b
                    .targets
                    .iter()
                    .flat_map(|t| {
                        ports(t)
                            .into_iter()
                            .filter(|q| q.name == m)
                            .map(move |q| (*t, q))
                    })
                    .collect();
                if hits.is_empty() {
                    let via = if m == p.name {
                        String::new()
                    } else {
                        format!(" (mapped to '{m}')")
                    };
                    out.push((
                    "E991",
                    b.file.clone(),
                    format!("{}: port '{}'{via} corresponds to no port of {names}; map it with `portMap:` (to `null` if it has no architecture counterpart)", b.holder, p.name),
                ));
                    continue;
                }
                for (t, q) in &hits {
                    // a counterpart in the wrong direction is still a counterpart: E991 here, not also W990
                    covered.insert((t.qualified_name.clone(), q.name.clone()));
                    // GH #331: ports typed on both sides share the interface
                    if let (Some(a), Some(c)) = (
                        crate::port_types::port_type_of(elements, resolver, &pctx, b.def, &p.name),
                        crate::port_types::port_type_of(elements, resolver, &pctx, t, &q.name),
                    ) {
                        if a != c {
                            out.push((
                                "E991",
                                b.file.clone(),
                                format!(
                                    "{}: port '{}' is typed by '{}' but its counterpart '{}.{}' is typed by '{}'",
                                    b.holder,
                                    p.name,
                                    crate::port_types::show(&a),
                                    t.qualified_name,
                                    q.name,
                                    crate::port_types::show(&c)
                                ),
                            ));
                        }
                    }
                    if !compatible(p.dir, q.dir) {
                        out.push((
                            "E991",
                            b.file.clone(),
                            format!(
                                "{}: port '{}' is `{}` but its counterpart '{}.{}' is `{}`",
                                b.holder,
                                p.name,
                                dir_name(p.dir),
                                t.qualified_name,
                                q.name,
                                dir_name(q.dir)
                            ),
                        ));
                    }
                }
            }
        }
    }

    // W990: each port of a modelled element has a counterpart in one of the parts that model it (on the union), unless
    // one of them names it in `abstracts:`.
    let mut by_target: BTreeMap<String, Vec<&Binding>> = BTreeMap::new();
    for b in &bs {
        for t in &b.targets {
            by_target
                .entry(t.qualified_name.clone())
                .or_default()
                .push(b);
        }
    }
    let mut abstracted: BTreeSet<(String, String)> = BTreeSet::new();
    for (tq, list) in &by_target {
        let Some(t) = resolver.resolve_ref(elements, tq) else {
            continue;
        };
        let abstracts: BTreeSet<&String> = list.iter().flat_map(|b| &b.abstracts).collect();
        let mut holders: Vec<String> = list.iter().map(|b| b.holder.clone()).collect();
        holders.sort();
        holders.dedup();
        for q in ports(t) {
            if abstracts.contains(&q.name) {
                abstracted.insert((tq.clone(), q.name.clone()));
                continue;
            }
            if !covered.contains(&(tq.clone(), q.name.clone())) {
                let file = list
                    .iter()
                    .map(|b| b.file.clone())
                    .min()
                    .unwrap_or_default();
                out.push((
                    "W990",
                    file,
                    format!("port '{}' of '{tq}' has no counterpart in {}, the part(s) that model it; add one, map one with `portMap:`, or name '{}' in `abstracts:`", q.name, holders.join(", "), q.name),
                ));
            }
        }
    }

    // W991: connections correspond between an executable composite and the architecture.
    let edges = arch_edges(elements, resolver);
    let key_cache: std::cell::RefCell<HashMap<String, BTreeSet<String>>> = Default::default();
    let tkeys = |t: &RawElement| -> BTreeSet<String> {
        key_cache
            .borrow_mut()
            .entry(t.qualified_name.clone())
            .or_insert_with(|| keys(elements, resolver, t))
            .clone()
    };
    // edges by port name, so a lookup does not scan every architecture connection
    let mut by_port: HashMap<&str, Vec<&ArchEdge>> = HashMap::new();
    for e in &edges {
        by_port.entry(e.pa.as_str()).or_default().push(e);
        if e.pb != e.pa {
            by_port.entry(e.pb.as_str()).or_default().push(e);
        }
    }
    let joined = |ka: &BTreeSet<String>, pa: &str, kb: &BTreeSet<String>, pb: &str| -> bool {
        by_port.get(pa).into_iter().flatten().any(|e| {
            (!ka.is_disjoint(&e.a) && e.pa == pa && !kb.is_disjoint(&e.b) && e.pb == pb)
                || (!kb.is_disjoint(&e.a) && e.pa == pb && !ka.is_disjoint(&e.b) && e.pb == pa)
        })
    };
    // (architecture key, port) covered by some executable port, or exempted by `abstracts:`
    let expand = |set: &BTreeSet<(String, String)>| -> std::collections::HashSet<(String, String)> {
        set.iter()
            .flat_map(|(t, p)| {
                tkeys_of(elements, resolver, t)
                    .into_iter()
                    .map(move |k| (k, p.clone()))
            })
            .collect()
    };
    let mut covered_keys = expand(&covered);
    // GH #332: an architecture element used directly in a verification composite covers its own ports
    for c in elements
        .iter()
        .filter(|e| is_part(e) && !is_architecture(elements, resolver, e))
    {
        let view = system::composite_view(elements, resolver, c);
        if !is_verification_composite(elements, resolver, &view.features) {
            continue;
        }
        for m in view.features.iter().filter_map(|f| f.as_mapping()) {
            if system::is_outside(m) {
                continue;
            }
            let Some(d) = field(m, "typedBy")
                .and_then(|t| t.as_str())
                .and_then(|t| resolver.resolve_ref(elements, t))
                .filter(|d| is_part(d) && is_architecture(elements, resolver, d))
            else {
                continue;
            };
            for k in tkeys(d) {
                for p in ports(d) {
                    covered_keys.insert((k.clone(), p.name));
                }
            }
        }
    }
    let abstracted_keys = expand(&abstracted);
    let hit_any =
        |ks: &BTreeSet<String>, p: &str, set: &std::collections::HashSet<(String, String)>| {
            ks.iter().any(|k| set.contains(&(k.clone(), p.to_string())))
        };
    for c in elements.iter().filter(|e| {
        is_part(e)
            && !is_architecture(elements, resolver, e)
            // a subtype that adds no parts or connections of its own repeats its base's findings, reported there
            && !(e.frontmatter.supertype.is_some() && e.frontmatter.features.is_none() && e.frontmatter.connections.is_none())
    }) {
        let view = system::composite_view(elements, resolver, c);
        if !is_verification_composite(elements, resolver, &view.features) {
            continue; // no modelling or environment part: nothing to correspond
        }
        // part name → (keys of everything it models, its port map)
        #[allow(clippy::type_complexity)]
        let mut parts: BTreeMap<
            String,
            (
                BTreeSet<String>,
                PortMap,
                Option<String>,
                &RawElement,
                Vec<&RawElement>,
            ),
        > = BTreeMap::new();
        for f in &view.features {
            let Some(m) = f.as_mapping() else { continue };
            let (Some(name), Some(def)) = (
                field(m, "name").and_then(|n| n.as_str()),
                field(m, "typedBy")
                    .and_then(|t| t.as_str())
                    .and_then(|t| resolver.resolve_ref(elements, t))
                    .filter(|d| is_part(d)),
            ) else {
                continue;
            };
            if system::is_outside(m) {
                continue; // GH #332: not the product, so no architecture counterpart
            }
            let (models, pm, _) = usage_binding(elements, resolver, def, m);
            // GH #332: an architecture element used directly is its own counterpart
            let ts: Vec<&RawElement> = if is_architecture(elements, resolver, def) {
                vec![def]
            } else {
                models
                    .iter()
                    .filter_map(|r| resolver.resolve_ref(elements, r))
                    .filter(|t| is_architecture(elements, resolver, t))
                    .collect()
            };
            let ks: BTreeSet<String> = ts.iter().flat_map(|t| tkeys(t)).collect();
            if !ks.is_empty() {
                let doms: BTreeSet<String> = ts
                    .iter()
                    .filter_map(|t| domain_of(elements, resolver, t))
                    .collect();
                let dom = if doms.len() == 1 {
                    doms.into_iter().next()
                } else {
                    None
                };
                parts.insert(name.to_string(), (ks, pm, dom, def, ts));
            }
        }
        if parts.len() < 2 {
            continue;
        }
        let mut exec_pairs: Vec<(BTreeSet<String>, String, BTreeSet<String>, String)> = Vec::new();
        for conn in &view.connections {
            let Some(m) = conn.as_mapping() else { continue };
            let (Some(from), Some(to)) = (
                field(m, "from").and_then(|v| v.as_str()),
                field(m, "to").and_then(|v| v.as_str()),
            ) else {
                continue;
            };
            let (Some((a, pa)), Some((b, pb))) = (system::endpoint(from), system::endpoint(to))
            else {
                continue;
            };
            let (Some((ka, ma, da, dfa, tsa)), Some((kb, mb, db, dfb, tsb))) =
                (parts.get(a), parts.get(b))
            else {
                continue;
            };
            let (pas, pbs) = (
                counterparts(elements, resolver, dfa, tsa, ma, pa, &port_index, &pctx),
                counterparts(elements, resolver, dfb, tsb, mb, pb, &port_index, &pctx),
            );
            if pas.is_empty() || pbs.is_empty() {
                continue; // an end mapped to `null`
            }
            for pa2 in &pas {
                for pb2 in &pbs {
                    exec_pairs.push((ka.clone(), pa2.clone(), kb.clone(), pb2.clone()));
                }
            }
            if !ka.is_disjoint(kb) {
                continue; // inside one modelled element
            }
            // §12.6: software and hardware meet through allocation, never through an architecture connection, so a
            // connection across that boundary has nothing to correspond to.
            if matches!(
                (da.as_deref(), db.as_deref()),
                (Some("software"), Some("hardware")) | (Some("hardware"), Some("software"))
            ) {
                continue;
            }
            if !pas
                .iter()
                .any(|pa2| pbs.iter().any(|pb2| joined(ka, pa2, kb, pb2)))
            {
                let side = |k: &BTreeSet<String>| k.iter().next().cloned().unwrap_or_default();
                let alts = |ps: &[String]| {
                    if ps.len() == 1 {
                        ps[0].clone()
                    } else {
                        format!("{{{}}}", ps.join(", "))
                    }
                };
                out.push((
                    "W991",
                    c.file_path.clone(),
                    format!("connection {from} → {to} of '{}' has no counterpart in the architecture: no connection joins '{}.{}' and '{}.{}'", c.qualified_name, side(ka), alts(&pas), side(kb), alts(&pbs)),
                ));
            }
        }
        // the reverse: an architecture connection between two elements this composite models
        let all: BTreeSet<String> = parts.values().flat_map(|(k, ..)| k.iter().cloned()).collect();
        let mut reported: BTreeSet<(String, String, String, String)> = BTreeSet::new();
        for e in &edges {
            if e.a.is_disjoint(&all) || e.b.is_disjoint(&all) || !e.a.is_disjoint(&e.b) {
                continue;
            }
            if hit_any(&e.a, &e.pa, &abstracted_keys) || hit_any(&e.b, &e.pb, &abstracted_keys) {
                continue;
            }
            // a port no executable part covers is already W990
            if !hit_any(&e.a, &e.pa, &covered_keys) || !hit_any(&e.b, &e.pb, &covered_keys) {
                continue;
            }
            let hit = exec_pairs.iter().any(|(ka, pa, kb, pb)| {
                (!ka.is_disjoint(&e.a) && pa == &e.pa && !kb.is_disjoint(&e.b) && pb == &e.pb)
                    || (!kb.is_disjoint(&e.a) && pb == &e.pa && !ka.is_disjoint(&e.b) && pa == &e.pb)
            });
            if !hit && reported.insert((e.an.clone(), e.pa.clone(), e.bn.clone(), e.pb.clone())) {
                out.push((
                    "W991",
                    c.file_path.clone(),
                    format!("architecture connection '{}.{}' → '{}.{}' (in '{}') has no counterpart in '{}', which models both ends", e.an, e.pa, e.bn, e.pb, e.composite, c.qualified_name),
                ));
            }
        }
    }

    // W992: a TestCase verifies a requirement whose architecture satisfiers its subject does not exercise.
    let mut satisfiers: HashMap<String, Vec<&RawElement>> = HashMap::new();
    for e in elements
        .iter()
        .filter(|e| is_architecture(elements, resolver, e))
    {
        for r in e.frontmatter.satisfies.iter().flatten() {
            if let Some(req) = resolver.resolve_ref(elements, r) {
                satisfiers
                    .entry(req.qualified_name.clone())
                    .or_default()
                    .push(e);
            }
        }
    }
    let mut reach_cache: HashMap<String, Exercised> = HashMap::new();
    for tc in elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::TestCase)))
    {
        let Some(subject) = tc
            .frontmatter
            .subject
            .as_deref()
            .and_then(|s| resolver.resolve_ref(elements, s))
            .filter(|s| is_part(s))
        else {
            continue;
        };
        let reach = reach_cache
            .entry(subject.qualified_name.clone())
            .or_insert_with(|| exercised_by(elements, resolver, subject));
        for r in tc.frontmatter.verifies.iter().flatten() {
            let Some(req) = resolver.resolve_ref(elements, r) else {
                continue;
            };
            let Some(sat) = satisfiers.get(&req.qualified_name) else {
                continue;
            };
            if sat.iter().any(|s| {
                tkeys(s)
                    .iter()
                    .any(|k| reach.evidences(k, &req.qualified_name))
            }) {
                continue;
            }
            let left_out = sat
                .iter()
                .any(|s| tkeys(s).iter().any(|k| reach.reaches(k)));
            let names = sat
                .iter()
                .map(|s| format!("'{}'", s.qualified_name))
                .collect::<Vec<_>>()
                .join(", ");
            let tc_id = tc.frontmatter.id.as_deref().unwrap_or(&tc.qualified_name);
            let why = if left_out {
                format!(
                    "the parts of its subject '{}' that model {} name {r} in `abstracts:`",
                    subject.qualified_name,
                    if sat.len() == 1 { "it" } else { "them" }
                )
            } else {
                format!(
                    "its subject '{}' contains no part that models {}",
                    subject.qualified_name,
                    if sat.len() == 1 { "it" } else { "any of them" }
                )
            };
            out.push((
                "W992",
                tc.file_path.clone(),
                format!("{tc_id} verifies {r}, satisfied by {names}, but {why}: its scenarios cannot be evidence for it"),
            ));
        }
    }
    out
}

/// W996 (GH #337): in a context composite (a part that holds at least one `role: external` usage), a port of a
/// product part that no connection, flow or binding of the composite reaches. Opt-in by construction: it applies only
/// where a context composite says which parts are actors.
fn boundary_port_findings(elements: &[RawElement], resolver: &Resolver) -> Vec<Finding> {
    let mut out = Vec::new();
    let index = system::child_port_index(elements);
    for c in elements.iter().filter(|e| is_part(e)) {
        if !system::is_context(c) {
            continue;
        }
        // the usages: inline part features and child `Part` files, each with the element its ports come from
        let mut usages: Vec<(String, &RawElement, bool)> = Vec::new();
        for m in c.frontmatter.features.iter().flatten().filter_map(|f| f.as_mapping()) {
            let Some(name) = field(m, "name").and_then(|n| n.as_str()) else { continue };
            if field(m, "type").and_then(|t| t.as_str()) == Some("Port") {
                continue;
            }
            if let Some(def) = field(m, "typedBy").and_then(|t| t.as_str()).and_then(|t| resolver.resolve_ref(elements, t)).filter(|d| is_part(d)) {
                usages.push((name.to_string(), def, system::is_outside(m)));
            }
        }
        for child in elements.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Part)) && e.qualified_name.rsplit_once("::").is_some_and(|(p, _)| p == c.qualified_name)) {
            let name = child.qualified_name.rsplit_once("::").map_or("", |(_, n)| n);
            if !usages.iter().any(|(n, ..)| n == name) {
                usages.push((name.to_string(), child, false));
            }
        }
        // an entry reaches its ends only when every end is on a part that is drawn (a projected-out actor reaches
        // nothing) or on the context's own port
        let own: Vec<String> = port_names(elements, resolver, c, &index).into_iter().map(|(n, _)| n).collect();
        let mut reached: BTreeSet<String> = BTreeSet::new();
        let mut written: BTreeSet<String> = BTreeSet::new();
        for list in [&c.frontmatter.connections, &c.frontmatter.flow_connections, &c.frontmatter.binding_connections] {
            for entry in list.iter().flatten() {
                let Some(p) = crate::connections::parse_entry(entry) else { continue };
                let chains: Vec<String> = p.endpoints.into_iter().map(|e| e.chain.trim().replace("::", ".")).collect();
                let live = chains.iter().all(|ch| {
                    let head = ch.split('.').next().unwrap_or("");
                    usages.iter().any(|(n, ..)| n == head) || own.iter().any(|o| o == head)
                });
                if live {
                    reached.extend(chains.iter().cloned());
                }
                written.extend(chains);
            }
        }
        for (name, from, outside) in &usages {
            if *outside {
                continue;
            }
            for (pname, gated) in port_names(elements, resolver, from, &index) {
                // a port with its own `appliesWhen:` stays in a projected variant (inline features are not projected),
                // so it is judged by what is written, as in the whole model
                let at = format!("{name}.{pname}");
                if !(if gated { written.contains(&at) } else { reached.contains(&at) }) {
                    out.push((
                        "W996",
                        c.file_path.clone(),
                        format!("'{}': port '{pname}' of '{name}' ({}) connects to nothing in this context; connect it to an actor (`role: external`) or another part", c.qualified_name, from.qualified_name),
                    ));
                }
            }
        }
    }
    out
}

/// The ports of a part (usage or definition) along its `supertype:`/`typedBy:` chain: inline `type: Port` features,
/// features typed by a `PortDef`, and child `Port` files; each with whether it is an inline feature gated by `appliesWhen:`.
fn port_names(elements: &[RawElement], resolver: &Resolver, e: &RawElement, index: &std::collections::HashMap<String, Vec<system::PortSpec>>) -> Vec<(String, bool)> {
    let mut names: Vec<(String, bool)> = Vec::new();
    for d in system::chain_with(elements, resolver, e, true) {
        for p in d.frontmatter.features.iter().flatten().filter_map(|f| f.as_mapping()) {
            let Some(pname) = field(p, "name").and_then(|n| n.as_str()) else { continue };
            let is_port = field(p, "type").and_then(|t| t.as_str()) == Some("Port")
                || field(p, "typedBy")
                    .and_then(|t| t.as_str())
                    .and_then(|t| resolver.resolve_ref(elements, t))
                    .is_some_and(|t| t.frontmatter.element_type == Some(ElementType::PortDef));
            if is_port && !names.iter().any(|(n, _)| n == pname) {
                names.push((pname.to_string(), field(p, "appliesWhen").is_some()));
            }
        }
        for p in index.get(d.qualified_name.as_str()).into_iter().flatten() {
            if !names.iter().any(|(n, _)| *n == p.name) {
                names.push((p.name.clone(), false));
            }
        }
    }
    names
}

/// E992 (GH #332): a `role:` on a composite's `features:` entry that is not a known role, or that is not on a part usage.
fn role_findings(elements: &[RawElement], resolver: &Resolver) -> Vec<Finding> {
    let mut out = Vec::new();
    for e in elements.iter().filter(|e| is_part(e)) {
        for f in e.frontmatter.features.iter().flatten() {
            let Some(m) = f.as_mapping() else { continue };
            let Some(role) = field(m, "role") else {
                continue;
            };
            let name = field(m, "name").and_then(|n| n.as_str()).unwrap_or("?");
            let known = system::ROLES.join(", ");
            match role.as_str() {
                Some(r) if system::ROLES.contains(&r) => {}
                Some(r) => {
                    out.push(("E992", e.file_path.clone(), format!("'{}': feature '{name}' has `role: {r}`, which is not a role (the roles are: {known})", e.qualified_name)));
                    continue;
                }
                None => {
                    out.push(("E992", e.file_path.clone(), format!("'{}': feature '{name}' has a `role:` that is not a string (the roles are: {known})", e.qualified_name)));
                    continue;
                }
            }
            // a `typedBy:` that does not resolve is reported on its own; here only a port or a non-part type
            let ty = field(m, "typedBy").and_then(|t| t.as_str());
            let resolved = ty.and_then(|t| resolver.resolve_ref(elements, t));
            let not_part_usage = field(m, "type").and_then(|t| t.as_str()) == Some("Port")
                || ty.is_none()
                || resolved.is_some_and(|d| !is_part(d));
            if not_part_usage {
                out.push(("E992", e.file_path.clone(), format!("'{}': feature '{name}' has `role:`, which is recognised on a part usage only (a feature typed by a PartDef or Part)", e.qualified_name)));
            }
        }
    }
    out
}

/// The nearest `domain:` along an element's definition chain.
fn domain_of(elements: &[RawElement], resolver: &Resolver, e: &RawElement) -> Option<String> {
    system::chain(elements, resolver, e)
        .iter()
        .find_map(|d| d.frontmatter.domain.clone())
}

fn tkeys_of(elements: &[RawElement], resolver: &Resolver, q: &str) -> BTreeSet<String> {
    resolver
        .resolve_ref(elements, q)
        .map(|t| keys(elements, resolver, t))
        .unwrap_or_default()
}
