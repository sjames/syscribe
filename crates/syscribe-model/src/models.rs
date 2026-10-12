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
    matches!(e.frontmatter.element_type, Some(ElementType::PartDef) | Some(ElementType::Part))
}

fn strings(v: Option<&Y>) -> Vec<String> {
    match v {
        Some(Y::String(s)) => vec![s.clone()],
        Some(Y::Sequence(xs)) => xs.iter().filter_map(|x| x.as_str().map(String::from)).collect(),
        _ => Vec::new(),
    }
}

/// A `PartDef`/`Part` that has a `domain:` on itself or along its definition chain.
pub fn is_architecture(elements: &[RawElement], resolver: &Resolver, e: &RawElement) -> bool {
    is_part(e) && system::chain(elements, resolver, e).iter().any(|d| d.frontmatter.domain.is_some())
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

/// `portMap:` as executable port → architecture port, `None` for a port mapped to `null` (no counterpart on purpose).
fn port_map(m: Option<&Mapping>) -> BTreeMap<String, Option<String>> {
    m.into_iter()
        .flatten()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str().map(String::from))))
        .collect()
}

struct Binding<'a> {
    file: String,
    /// How a finding names the holder: `'Pkg::Monitor'` or `part 'sensorSw' of 'Pkg::Chain'`.
    holder: String,
    def: &'a RawElement,
    targets: Vec<&'a RawElement>,
    port_map: BTreeMap<String, Option<String>>,
    abstracts: Vec<String>,
}

impl Binding<'_> {
    fn mapped(&self, port: &str) -> Option<String> {
        match self.port_map.get(port) {
            Some(m) => m.clone(),
            None => Some(port.to_string()),
        }
    }
}

/// The nearest `models:`/`portMap:`/`abstracts:` along a definition chain.
fn def_binding_fields<'a>(elements: &'a [RawElement], resolver: &Resolver, def: &'a RawElement) -> (Vec<String>, BTreeMap<String, Option<String>>, Vec<String>) {
    let ch = system::chain(elements, resolver, def);
    let models = ch.iter().find_map(|d| d.frontmatter.models.clone().filter(|m| !m.is_empty())).unwrap_or_default();
    let pm = ch.iter().find_map(|d| d.frontmatter.port_map.as_ref()).map(|m| port_map(Some(m))).unwrap_or_default();
    let abstracts = ch.iter().flat_map(|d| d.frontmatter.abstracts.clone().unwrap_or_default()).collect();
    (models, pm, abstracts)
}

/// The modelling a part usage (a composite's `features:` entry typed by `def`) carries: its own `models:`/`portMap:`
/// over the definition's.
fn usage_binding<'a>(elements: &'a [RawElement], resolver: &Resolver, def: &'a RawElement, entry: &Mapping) -> (Vec<String>, BTreeMap<String, Option<String>>, Vec<String>) {
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
            || e.frontmatter.features.iter().flatten().any(|f| f.as_mapping().is_some_and(|m| field(m, "models").is_some() || field(m, "portMap").is_some()))
    })
}

/// Resolve `models:` targets, reporting E990 for one that does not resolve or is not an architecture element.
fn resolve_targets<'a>(elements: &'a [RawElement], resolver: &Resolver, refs: &[String], holder: &str, file: &str, out: &mut Vec<Finding>) -> Vec<&'a RawElement> {
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

fn bindings<'a>(elements: &'a [RawElement], resolver: &Resolver, out: &mut Vec<Finding>) -> Vec<Binding<'a>> {
    let mut v = Vec::new();
    for e in elements {
        let own = e.frontmatter.models.clone().unwrap_or_default();
        if !own.is_empty() {
            let holder = format!("'{}'", e.qualified_name);
            if !is_part(e) {
                out.push(("E990", e.file_path.clone(), format!("{holder}: `models:` is recognised on a PartDef or Part only")));
            } else {
                let targets = resolve_targets(elements, resolver, &own, &holder, &e.file_path, out);
                let (_, pm, abstracts) = def_binding_fields(elements, resolver, e);
                v.push(Binding { file: e.file_path.clone(), holder, def: e, targets, port_map: pm, abstracts });
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
            let Some(def) = field(m, "typedBy").and_then(|t| t.as_str()).and_then(|t| resolver.resolve_ref(elements, t)).filter(|d| is_part(d)) else {
                out.push(("E990", e.file_path.clone(), format!("{holder}: `models:`/`portMap:` on a feature needs a `typedBy:` that resolves to a PartDef or Part")));
                continue;
            };
            let (models, pm, abstracts) = usage_binding(elements, resolver, def, m);
            if models.is_empty() {
                out.push(("E990", e.file_path.clone(), format!("{holder}: has a `portMap:` but neither it nor '{}' declares `models:`", def.qualified_name)));
                continue;
            }
            let targets = if field(m, "models").is_some() { resolve_targets(elements, resolver, &models, &holder, &e.file_path, out) } else { resolve_targets(elements, resolver, &models, &holder, &e.file_path, &mut Vec::new()) };
            v.push(Binding { file: e.file_path.clone(), holder, def, targets, port_map: pm, abstracts });
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

/// The architecture elements a composite `subject` can exercise: itself and every part reachable through `features:`,
/// with what each models (GH #330; #344 reuses this at scenario level).
pub fn exercised_by(elements: &[RawElement], resolver: &Resolver, subject: &RawElement) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack: Vec<(&RawElement, Option<Mapping>)> = vec![(subject, None)];
    while let Some((e, usage)) = stack.pop() {
        if seen.len() > 512 {
            break;
        }
        let key = format!("{}#{}", e.qualified_name, usage.as_ref().and_then(|m| field(m, "name")).and_then(|n| n.as_str()).unwrap_or(""));
        if !seen.insert(key) {
            continue;
        }
        if is_architecture(elements, resolver, e) {
            out.extend(keys(elements, resolver, e));
        }
        let models = match &usage {
            Some(m) => usage_binding(elements, resolver, e, m).0,
            None => def_binding_fields(elements, resolver, e).0,
        };
        for r in models {
            if let Some(t) = resolver.resolve_ref(elements, &r) {
                out.extend(keys(elements, resolver, t));
            }
        }
        for f in system::composite_view(elements, resolver, e).features {
            let Some(m) = f.as_mapping() else { continue };
            if field(m, "type").and_then(|t| t.as_str()) == Some("Port") {
                continue;
            }
            if let Some(d) = field(m, "typedBy").and_then(|t| t.as_str()).and_then(|t| resolver.resolve_ref(elements, t)).filter(|d| is_part(d)) {
                stack.push((d, Some(m.clone())));
            }
        }
    }
    out
}

/// One architecture connection between two part usages, by the definitions they are typed by.
struct ArchEdge {
    a: String,
    pa: String,
    b: String,
    pb: String,
    composite: String,
}

fn arch_edges(elements: &[RawElement], resolver: &Resolver) -> Vec<ArchEdge> {
    let mut v = Vec::new();
    for c in elements.iter().filter(|e| is_part(e) && e.frontmatter.connections.as_ref().is_some_and(|c| !c.is_empty())) {
        let view = system::composite_view(elements, resolver, c);
        let def_of = |part: &str| -> Option<&RawElement> {
            view.features.iter().filter_map(|f| f.as_mapping()).find(|m| field(m, "name").and_then(|n| n.as_str()) == Some(part)).and_then(|m| field(m, "typedBy")).and_then(|t| t.as_str()).and_then(|t| resolver.resolve_ref(elements, t))
        };
        for conn in &view.connections {
            let Some(m) = conn.as_mapping() else { continue };
            let (Some(from), Some(to)) = (field(m, "from").and_then(|v| v.as_str()), field(m, "to").and_then(|v| v.as_str())) else { continue };
            let (Some((fa, fp)), Some((ta, tp))) = (system::endpoint(from), system::endpoint(to)) else { continue };
            let (Some(da), Some(db)) = (def_of(fa), def_of(ta)) else { continue };
            if is_architecture(elements, resolver, da) && is_architecture(elements, resolver, db) {
                v.push(ArchEdge { a: da.qualified_name.clone(), pa: fp.to_string(), b: db.qualified_name.clone(), pb: tp.to_string(), composite: c.qualified_name.clone() });
            }
        }
    }
    v
}

/// Every `models:` finding (E990, E991, W990, W991, W992). Empty unless some part declares `models:`.
pub fn findings(elements: &[RawElement], resolver: &Resolver) -> Vec<Finding> {
    let mut out = Vec::new();
    if !active(elements) {
        return out;
    }
    let bs = bindings(elements, resolver, &mut out);

    // E991: each executable port corresponds to a port of a modelled element, in a compatible direction.
    let ports_cache: std::cell::RefCell<HashMap<String, Vec<PortSpec>>> = Default::default();
    let ports = |d: &RawElement| -> Vec<PortSpec> { ports_cache.borrow_mut().entry(d.qualified_name.clone()).or_insert_with(|| system::def_ports(elements, resolver, d)).clone() };
    // (architecture qname, port) covered by some executable port
    let mut covered: BTreeSet<(String, String)> = BTreeSet::new();
    for b in bs.iter().filter(|b| !b.targets.is_empty()) {
        let exec = ports(b.def);
        for k in b.port_map.keys() {
            if !exec.iter().any(|p| &p.name == k) {
                out.push(("E991", b.file.clone(), format!("{}: `portMap:` key '{k}' is not a port of '{}'", b.holder, b.def.qualified_name)));
            }
        }
        let names = b.targets.iter().map(|t| format!("'{}'", t.qualified_name)).collect::<Vec<_>>().join(", ");
        for p in &exec {
            let Some(m) = b.mapped(&p.name) else { continue };
            let hits: Vec<(&RawElement, PortSpec)> = b.targets.iter().flat_map(|t| ports(t).into_iter().filter(|q| q.name == m).map(move |q| (*t, q))).collect();
            if hits.is_empty() {
                let via = if m == p.name { String::new() } else { format!(" (mapped to '{m}')") };
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
                if !compatible(p.dir, q.dir) {
                    out.push((
                        "E991",
                        b.file.clone(),
                        format!("{}: port '{}' is `{}` but its counterpart '{}.{}' is `{}`", b.holder, p.name, dir_name(p.dir), t.qualified_name, q.name, dir_name(q.dir)),
                    ));
                }
            }
        }
    }

    // W990: each port of a modelled element has a counterpart in one of the parts that model it (on the union), unless
    // one of them names it in `abstracts:`.
    let mut by_target: BTreeMap<String, Vec<&Binding>> = BTreeMap::new();
    for b in &bs {
        for t in &b.targets {
            by_target.entry(t.qualified_name.clone()).or_default().push(b);
        }
    }
    let mut abstracted: BTreeSet<(String, String)> = BTreeSet::new();
    for (tq, list) in &by_target {
        let Some(t) = resolver.resolve_ref(elements, tq) else { continue };
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
                let file = list.iter().map(|b| b.file.clone()).min().unwrap_or_default();
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
    let tkeys = |t: &RawElement| -> BTreeSet<String> { key_cache.borrow_mut().entry(t.qualified_name.clone()).or_insert_with(|| keys(elements, resolver, t)).clone() };
    let joined = |ka: &BTreeSet<String>, pa: &str, kb: &BTreeSet<String>, pb: &str| -> Option<&ArchEdge> {
        edges.iter().find(|e| (ka.contains(&e.a) && e.pa == pa && kb.contains(&e.b) && e.pb == pb) || (kb.contains(&e.a) && e.pa == pb && ka.contains(&e.b) && e.pb == pa))
    };
    for c in elements.iter().filter(|e| is_part(e) && !is_architecture(elements, resolver, e)) {
        let view = system::composite_view(elements, resolver, c);
        // part name → (keys of everything it models, its port map)
        let mut parts: BTreeMap<String, (BTreeSet<String>, BTreeMap<String, Option<String>>)> = BTreeMap::new();
        for f in &view.features {
            let Some(m) = f.as_mapping() else { continue };
            let (Some(name), Some(def)) = (field(m, "name").and_then(|n| n.as_str()), field(m, "typedBy").and_then(|t| t.as_str()).and_then(|t| resolver.resolve_ref(elements, t)).filter(|d| is_part(d))) else { continue };
            let (models, pm, _) = usage_binding(elements, resolver, def, m);
            let ks: BTreeSet<String> = models.iter().filter_map(|r| resolver.resolve_ref(elements, r)).filter(|t| is_architecture(elements, resolver, t)).flat_map(|t| tkeys(t)).collect();
            if !ks.is_empty() {
                parts.insert(name.to_string(), (ks, pm));
            }
        }
        if parts.len() < 2 {
            continue;
        }
        let map = |pm: &BTreeMap<String, Option<String>>, p: &str| -> Option<String> { pm.get(p).cloned().unwrap_or_else(|| Some(p.to_string())) };
        let mut exec_pairs: Vec<(BTreeSet<String>, String, BTreeSet<String>, String)> = Vec::new();
        for conn in &view.connections {
            let Some(m) = conn.as_mapping() else { continue };
            let (Some(from), Some(to)) = (field(m, "from").and_then(|v| v.as_str()), field(m, "to").and_then(|v| v.as_str())) else { continue };
            let (Some((a, pa)), Some((b, pb))) = (system::endpoint(from), system::endpoint(to)) else { continue };
            let (Some((ka, ma)), Some((kb, mb))) = (parts.get(a), parts.get(b)) else { continue };
            let (Some(pa2), Some(pb2)) = (map(ma, pa), map(mb, pb)) else { continue };
            exec_pairs.push((ka.clone(), pa2.clone(), kb.clone(), pb2.clone()));
            if !ka.is_disjoint(kb) {
                continue; // inside one modelled element
            }
            if joined(ka, &pa2, kb, &pb2).is_none() {
                let side = |k: &BTreeSet<String>| k.iter().next().cloned().unwrap_or_default();
                out.push((
                    "W991",
                    c.file_path.clone(),
                    format!("connection {from} → {to} of '{}' has no counterpart in the architecture: no connection joins '{}.{pa2}' and '{}.{pb2}'", c.qualified_name, side(ka), side(kb)),
                ));
            }
        }
        // the reverse: an architecture connection between two elements this composite models
        let all: BTreeSet<&String> = parts.values().flat_map(|(k, _)| k).collect();
        let mut reported: BTreeSet<(String, String, String, String)> = BTreeSet::new();
        for e in &edges {
            if !all.contains(&e.a) || !all.contains(&e.b) || e.a == e.b {
                continue;
            }
            if abstracted.contains(&(e.a.clone(), e.pa.clone())) || abstracted.contains(&(e.b.clone(), e.pb.clone())) {
                continue;
            }
            // a port no executable part covers is already W990
            let cov = |q: &str, p: &str| covered.iter().any(|(t, pp)| pp == p && tkeys_of(elements, resolver, t).contains(q));
            if !cov(&e.a, &e.pa) || !cov(&e.b, &e.pb) {
                continue;
            }
            let hit = exec_pairs.iter().any(|(ka, pa, kb, pb)| (ka.contains(&e.a) && pa == &e.pa && kb.contains(&e.b) && pb == &e.pb) || (kb.contains(&e.a) && pb == &e.pa && ka.contains(&e.b) && pa == &e.pb));
            if !hit && reported.insert((e.a.clone(), e.pa.clone(), e.b.clone(), e.pb.clone())) {
                out.push((
                    "W991",
                    c.file_path.clone(),
                    format!("architecture connection '{}.{}' → '{}.{}' (in '{}') has no counterpart in '{}', which models both ends", e.a, e.pa, e.b, e.pb, e.composite, c.qualified_name),
                ));
            }
        }
    }

    // W992: a TestCase verifies a requirement whose architecture satisfiers its subject does not exercise.
    let mut satisfiers: HashMap<String, Vec<&RawElement>> = HashMap::new();
    for e in elements.iter().filter(|e| is_architecture(elements, resolver, e)) {
        for r in e.frontmatter.satisfies.iter().flatten() {
            if let Some(req) = resolver.resolve_ref(elements, r) {
                satisfiers.entry(req.qualified_name.clone()).or_default().push(e);
            }
        }
    }
    for tc in elements.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::TestCase))) {
        let Some(subject) = tc.frontmatter.subject.as_deref().and_then(|s| resolver.resolve_ref(elements, s)).filter(|s| is_part(s)) else { continue };
        let reach = exercised_by(elements, resolver, subject);
        for r in tc.frontmatter.verifies.iter().flatten() {
            let Some(req) = resolver.resolve_ref(elements, r) else { continue };
            let Some(sat) = satisfiers.get(&req.qualified_name) else { continue };
            if sat.iter().any(|s| !tkeys(s).is_disjoint(&reach)) {
                continue;
            }
            let names = sat.iter().map(|s| format!("'{}'", s.qualified_name)).collect::<Vec<_>>().join(", ");
            out.push((
                "W992",
                tc.file_path.clone(),
                format!(
                    "{} verifies {r}, satisfied by {names}, but its subject '{}' contains no part that models {}: its scenarios cannot be evidence for it",
                    tc.frontmatter.id.as_deref().unwrap_or(&tc.qualified_name),
                    subject.qualified_name,
                    if sat.len() == 1 { "it" } else { "any of them" }
                ),
            ));
        }
    }
    out
}

fn tkeys_of(elements: &[RawElement], resolver: &Resolver, q: &str) -> BTreeSet<String> {
    resolver.resolve_ref(elements, q).map(|t| keys(elements, resolver, t)).unwrap_or_default()
}
