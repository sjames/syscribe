use syscribe_model::{
    element::{ElementType, RawElement},
    resolver::Resolver,
    validator::ValidationResult,
};

// ── Shared helpers ────────────────────────────────────────────────────────────

pub fn type_label(et: &ElementType) -> &'static str {
    match et {
        ElementType::PartDef => "PartDef",
        ElementType::Part => "Part",
        ElementType::ItemDef => "ItemDef",
        ElementType::Item => "Item",
        ElementType::PortDef => "PortDef",
        ElementType::Port => "Port",
        ElementType::ConnectionDef => "ConnectionDef",
        ElementType::Connection => "Connection",
        ElementType::InterfaceDef => "InterfaceDef",
        ElementType::Interface => "Interface",
        ElementType::ActionDef => "ActionDef",
        ElementType::Action => "Action",
        ElementType::Requirement => "Requirement",
        ElementType::RequirementDef => "RequirementDef",
        ElementType::TestCase => "TestCase",
        ElementType::ADR => "ADR",
        ElementType::Package => "Package",
        ElementType::Allocation => "Allocation",
        ElementType::AllocationDef => "AllocationDef",
        ElementType::FlowDef => "FlowDef",
        ElementType::EnumerationDef => "EnumerationDef",
        ElementType::AttributeDef => "AttributeDef",
        ElementType::FeatureDef => "FeatureDef",
        ElementType::Configuration => "Configuration",
        ElementType::StateDef => "StateDef",
        ElementType::UseCaseDef => "UseCaseDef",
        ElementType::ViewDef => "ViewDef",
        ElementType::ViewpointDef => "ViewpointDef",
        ElementType::MetadataDef => "MetadataDef",
        ElementType::ConstraintDef => "ConstraintDef",
        ElementType::CalculationDef => "CalculationDef",
        ElementType::VerificationCaseDef => "VerificationCaseDef",
        ElementType::AnalysisCaseDef => "AnalysisCaseDef",
        ElementType::VerificationCase => "VerificationCase",
        ElementType::AnalysisCase => "AnalysisCase",
        ElementType::Diagram => "Diagram",
        ElementType::View => "View",
        ElementType::Metadata => "Metadata",
        ElementType::Calculation => "Calculation",
        ElementType::Constraint => "Constraint",
        ElementType::LibraryPackage => "LibraryPackage",
        ElementType::Namespace => "Namespace",
        ElementType::Dependency => "Dependency",
        ElementType::UseCase => "UseCase",
        ElementType::State => "State",
        ElementType::Enumeration => "Enumeration",
        _ => "Other",
    }
}

fn tl(et: Option<&ElementType>) -> &'static str {
    et.map(type_label).unwrap_or("?")
}

fn yaml_strings(v: &serde_yaml::Value) -> Vec<String> {
    match v {
        serde_yaml::Value::String(s) => vec![s.clone()],
        serde_yaml::Value::Sequence(seq) => {
            seq.iter().filter_map(|x| x.as_str().map(String::from)).collect()
        }
        _ => vec![],
    }
}

fn doc_excerpt(doc: &str, max: usize) -> String {
    let trimmed = doc.trim();
    if trimmed.len() <= max {
        trimmed.replace('\n', " ")
    } else {
        let cut = trimmed.char_indices().nth(max).map(|(i, _)| i).unwrap_or(max);
        format!("{}…", trimmed[..cut].replace('\n', " "))
    }
}

fn gherkin_count(doc: &str) -> usize {
    doc.lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("Scenario:") || t.starts_with("Scenario Outline:")
        })
        .count()
}

/// Resolve by qname first, then by stable ID.
fn resolve<'a>(elements: &'a [RawElement], resolver: &Resolver, key: &str) -> Option<&'a RawElement> {
    resolver.get(elements, key).or_else(|| resolver.get_by_id(elements, key))
}

// ── Namespace tree helpers ────────────────────────────────────────────────────

/// Direct namespace children of `parent_qname` (exactly one more `::` segment).
fn ns_children<'a>(elements: &'a [RawElement], parent_qname: &str) -> Vec<&'a RawElement> {
    let prefix = if parent_qname.is_empty() {
        String::new()
    } else {
        format!("{}::", parent_qname)
    };
    let mut children: Vec<&RawElement> = elements
        .iter()
        .filter(|e| {
            if prefix.is_empty() {
                !e.qualified_name.contains("::")
            } else {
                e.qualified_name.starts_with(&prefix)
                    && !e.qualified_name[prefix.len()..].contains("::")
            }
        })
        .collect();
    children.sort_by_key(|e| e.qualified_name.as_str());
    children
}

// ── Fuzzy search ──────────────────────────────────────────────────────────────

/// Tiered fuzzy score: 0 = no match, higher = better.
pub fn fuzzy_score(elem: &RawElement, pattern: &str) -> u32 {
    let qn = &elem.qualified_name;
    let pat_lc = pattern.to_lowercase();
    let qn_lc = qn.to_lowercase();

    // Stable ID exact
    if let Some(id) = &elem.frontmatter.id {
        if id == pattern { return 100; }
        if id.to_lowercase() == pat_lc { return 92; }
        if id.to_lowercase().contains(&pat_lc) { return 55; }
    }
    // qname exact
    if qn == pattern { return 100; }
    if qn_lc == pat_lc { return 90; }
    // Last segment exact
    if qn.split("::").last().map(|s| s.to_lowercase()) == Some(pat_lc.clone()) {
        return 82;
    }
    // name field exact
    if let Some(nm) = &elem.frontmatter.name {
        if nm.to_lowercase() == pat_lc { return 80; }
    }
    // qname contains
    if qn_lc.contains(&pat_lc) { return 65; }
    // title contains
    if let Some(t) = &elem.frontmatter.title {
        if t.to_lowercase().contains(&pat_lc) { return 50; }
    }
    // name contains
    if let Some(nm) = &elem.frontmatter.name {
        if nm.to_lowercase().contains(&pat_lc) { return 48; }
    }
    // Any segment starts-with
    if qn.split("::").any(|seg| seg.to_lowercase().starts_with(&pat_lc)) {
        return 35;
    }
    // Doc body (first 2000 chars)
    let doc_slice = &elem.doc[..elem.doc.len().min(2000)];
    if doc_slice.to_lowercase().contains(&pat_lc) { return 15; }

    0
}

// ── Reverse-reference collection ──────────────────────────────────────────────

/// Collect (relationship_label, target_qname) pairs from a single element's frontmatter.
fn outbound_refs(elem: &RawElement) -> Vec<(String, String)> {
    let fm = &elem.frontmatter;
    let mut out: Vec<(String, String)> = Vec::new();

    let mut push_yaml = |label: &str, v: &serde_yaml::Value| {
        for s in yaml_strings(v) {
            out.push((label.to_string(), s));
        }
    };

    if let Some(ref v) = fm.supertype { push_yaml("supertype", v); }
    if let Some(ref v) = fm.typed_by { push_yaml("typedBy", v); }
    if let Some(ref v) = fm.redefines { push_yaml("redefines", v); }
    if let Some(ref subs) = fm.subsets {
        for s in subs { out.push(("subsets".into(), s.clone())); }
    }
    if let Some(ref df) = fm.derived_from {
        for s in df { out.push(("derivedFrom".into(), s.clone())); }
    }
    if let Some(ref ver) = fm.verifies {
        for s in ver { out.push(("verifies".into(), s.clone())); }
    }
    if let Some(ref sat) = fm.satisfies {
        for s in sat { out.push(("satisfies".into(), s.clone())); }
    }
    if let Some(ref s) = fm.breakdown_adr { out.push(("breakdownAdr".into(), s.clone())); }
    if let Some(ref s) = fm.allocated_from { out.push(("allocatedFrom".into(), s.clone())); }
    if let Some(ref s) = fm.allocated_to { out.push(("allocatedTo".into(), s.clone())); }
    if let Some(ref es) = fm.exhibits_states {
        for s in es { out.push(("exhibitsStates".into(), s.clone())); }
    }
    if let Some(ref s) = fm.subject { out.push(("subject".into(), s.clone())); }
    if let Some(ref cl) = fm.clients {
        for s in cl { out.push(("clients".into(), s.clone())); }
    }
    if let Some(ref su) = fm.suppliers {
        for s in su { out.push(("suppliers".into(), s.clone())); }
    }

    // Walk shapes and edges YAML trees for nested "ref" keys
    fn collect_yaml_refs(v: &serde_yaml::Value, out: &mut Vec<(String, String)>, label: &str) {
        match v {
            serde_yaml::Value::Mapping(m) => {
                if let Some(r) = m.get("ref") {
                    if let Some(s) = r.as_str() {
                        out.push((label.to_string(), s.to_string()));
                    }
                }
                for val in m.values() {
                    collect_yaml_refs(val, out, label);
                }
            }
            serde_yaml::Value::Sequence(seq) => {
                for item in seq { collect_yaml_refs(item, out, label); }
            }
            _ => {}
        }
    }

    if let Some(ref shapes) = fm.shapes {
        collect_yaml_refs(shapes, &mut out, "shapes.ref");
    }
    if let Some(ref edges) = fm.edges {
        collect_yaml_refs(edges, &mut out, "edges.ref");
    }

    // connections / features typed-by
    if let Some(ref feats) = fm.features {
        for feat in feats {
            if let serde_yaml::Value::Mapping(m) = feat {
                for key in ["typedBy", "typed_by", "supertype"] {
                    if let Some(v) = m.get(key) {
                        for s in yaml_strings(v) {
                            out.push((format!("feature.{key}"), s));
                        }
                    }
                }
            }
        }
    }

    out
}

// ── cmd: show ─────────────────────────────────────────────────────────────────

pub fn cmd_show(elements: &[RawElement], resolver: &Resolver, key: &str) {
    let Some(elem) = resolve(elements, resolver, key) else {
        eprintln!("Element not found: {key}");
        eprintln!("Tip: use `find` to search by partial name.");
        return;
    };
    let fm = &elem.frontmatter;
    let type_str = tl(fm.element_type.as_ref());

    println!("# {}", elem.qualified_name);
    println!();
    println!("| Field | Value |");
    println!("|---|---|");
    println!("| **type** | {} |", type_str);
    println!("| **file** | {} |", elem.file_path);
    if let Some(ref id) = fm.id { println!("| **id** | {} |", id); }
    if let Some(ref t) = fm.title { println!("| **title** | {} |", t); }
    if let Some(ref s) = fm.status { println!("| **status** | {} |", s); }
    if fm.is_abstract == Some(true) { println!("| **abstract** | true |"); }
    if let Some(ref d) = fm.domain { println!("| **domain** | {} |", d); }
    if let Some(ref rd) = fm.req_domain { println!("| **reqDomain** | {} |", rd); }
    if let Some(sil) = fm.sil_level { println!("| **SIL** | {} |", sil); }
    if let Some(ref asil) = fm.asil_level { println!("| **ASIL** | {} |", asil); }
    if let Some(ref tl_) = fm.test_level { println!("| **testLevel** | {} |", tl_); }
    if let Some(ref mul) = fm.multiplicity { println!("| **multiplicity** | {} |", mul); }
    if let Some(ref dir) = fm.direction { println!("| **direction** | {} |", dir); }
    if let Some(ref s) = fm.breakdown_adr { println!("| **breakdownAdr** | {} |", s); }

    // Supertype / typedBy
    if let Some(ref v) = fm.supertype {
        let ss = yaml_strings(v);
        if !ss.is_empty() { println!("| **supertype** | {} |", ss.join(", ")); }
    }
    if let Some(ref v) = fm.typed_by {
        let ss = yaml_strings(v);
        if !ss.is_empty() { println!("| **typedBy** | {} |", ss.join(", ")); }
    }
    if let Some(ref subs) = fm.subsets {
        if !subs.is_empty() { println!("| **subsets** | {} |", subs.join(", ")); }
    }
    if let Some(ref df) = fm.derived_from {
        if !df.is_empty() { println!("| **derivedFrom** | {} |", df.join(", ")); }
    }
    if let Some(ref sat) = fm.satisfies {
        if !sat.is_empty() { println!("| **satisfies** | {} |", sat.join(", ")); }
    }
    if let Some(ref ver) = fm.verifies {
        if !ver.is_empty() { println!("| **verifies** | {} |", ver.join(", ")); }
    }
    if let Some(ref es) = fm.exhibits_states {
        if !es.is_empty() { println!("| **exhibitsStates** | {} |", es.join(", ")); }
    }
    if let Some(ref dk) = fm.diagram_kind { println!("| **diagramKind** | {} |", dk); }
    if let Some(ref sub) = fm.subject { println!("| **subject** | {} |", sub); }

    // Features table
    if let Some(ref feats) = fm.features {
        if !feats.is_empty() {
            println!();
            println!("## Features");
            println!();
            println!("| Name | Type | typedBy | Multiplicity | Direction |");
            println!("|---|---|---|---|---|");
            for feat in feats {
                let name = feat.get("name").and_then(|v| v.as_str()).unwrap_or("—");
                let ftype = feat.get("type").and_then(|v| v.as_str()).unwrap_or("—");
                let typed = feat.get("typedBy").or_else(|| feat.get("typed_by"))
                    .map(|v| yaml_strings(v).join(", "))
                    .unwrap_or_default();
                let mult = feat.get("multiplicity").and_then(|v| v.as_str()).unwrap_or("1");
                let dir = feat.get("direction").and_then(|v| v.as_str()).unwrap_or("—");
                println!("| {} | {} | {} | {} | {} |", name, ftype, typed, mult, dir);
            }
        }
    }

    // Doc
    let doc = elem.doc.trim();
    if !doc.is_empty() {
        println!();
        println!("## Documentation");
        println!();
        println!("{}", doc);
    }
}

// ── cmd: ls ──────────────────────────────────────────────────────────────────

pub fn cmd_ls(elements: &[RawElement], parent: &str) {
    let children = ns_children(elements, parent);
    if children.is_empty() {
        if parent.is_empty() {
            eprintln!("No top-level elements found.");
        } else {
            eprintln!("No children found for: {parent}");
        }
        return;
    }
    let header = if parent.is_empty() { "(root)".to_string() } else { parent.to_string() };
    println!("# {header}");
    println!();
    println!("| Name | Qualified Name | Type |");
    println!("|---|---|---|");
    for child in &children {
        let name = child
            .qualified_name
            .split("::")
            .last()
            .unwrap_or(&child.qualified_name);
        let type_str = tl(child.frontmatter.element_type.as_ref());
        let id_suffix = child
            .frontmatter
            .id
            .as_deref()
            .map(|id| format!(" `{id}`"))
            .unwrap_or_default();
        println!("| {}{} | {} | {} |", name, id_suffix, child.qualified_name, type_str);
    }
    println!();
    println!("{} element(s)", children.len());
}

// ── cmd: tree ────────────────────────────────────────────────────────────────

pub fn cmd_tree(elements: &[RawElement], root: &str) {
    let header = if root.is_empty() { "(root)" } else { root };
    println!("{}", header);
    print_tree_level(elements, root, "", true);
}

fn print_tree_level(elements: &[RawElement], parent: &str, prefix: &str, _is_root: bool) {
    let children = ns_children(elements, parent);
    let n = children.len();
    for (i, child) in children.iter().enumerate() {
        let last = i == n - 1;
        let connector = if last { "└──" } else { "├──" };
        let name = child
            .qualified_name
            .split("::")
            .last()
            .unwrap_or(&child.qualified_name);
        let type_str = tl(child.frontmatter.element_type.as_ref());
        let id_tag = child
            .frontmatter
            .id
            .as_deref()
            .map(|id| format!(" [{id}]"))
            .unwrap_or_default();
        println!("{}{} {} [{}]{}", prefix, connector, name, type_str, id_tag);
        let child_prefix = format!("{}{}   ", prefix, if last { " " } else { "│" });
        print_tree_level(elements, &child.qualified_name, &child_prefix, false);
    }
}

// ── cmd: find ────────────────────────────────────────────────────────────────

pub fn cmd_find(elements: &[RawElement], pattern: &str) {
    let mut scored: Vec<(u32, &RawElement)> = elements
        .iter()
        .map(|e| (fuzzy_score(e, pattern), e))
        .filter(|(s, _)| *s > 0)
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.qualified_name.cmp(&b.1.qualified_name)));
    scored.truncate(25);

    if scored.is_empty() {
        println!("No matches for `{pattern}`.");
        return;
    }

    println!("# Search: `{}`", pattern);
    println!();
    println!("| Score | Qualified Name | Type | Excerpt |");
    println!("|---|---|---|---|");
    for (score, elem) in &scored {
        let type_str = tl(elem.frontmatter.element_type.as_ref());
        // Prefer title or id as excerpt, fall back to doc
        let excerpt = elem
            .frontmatter
            .title
            .as_deref()
            .map(|t| t.to_string())
            .or_else(|| {
                elem.frontmatter
                    .id
                    .as_deref()
                    .map(|id| id.to_string())
            })
            .unwrap_or_else(|| doc_excerpt(&elem.doc, 80));
        println!("| {} | {} | {} | {} |", score, elem.qualified_name, type_str, excerpt);
    }
    println!();
    println!("{} match(es)", scored.len());
}

// ── cmd: trace ───────────────────────────────────────────────────────────────

pub fn cmd_trace(
    elements: &[RawElement],
    resolver: &Resolver,
    val: &ValidationResult,
    key: &str,
) {
    let Some(elem) = resolve(elements, resolver, key) else {
        eprintln!("Element not found: {key}");
        return;
    };
    let fm = &elem.frontmatter;

    let id = fm.id.as_deref().unwrap_or(&elem.qualified_name);
    let title = fm.title.as_deref().unwrap_or("(no title)");
    let status = fm.status.as_deref().unwrap_or("—");
    let req_domain = fm.req_domain.as_deref().unwrap_or("—");
    let sil = fm.sil_level.map(|v| v.to_string()).unwrap_or("—".into());
    let asil = fm.asil_level.as_deref().unwrap_or("—");
    let is_parent = val.derived_children.contains_key(id);

    println!("# Trace: {}", id);
    println!();
    println!("**Title:** {}  ", title);
    println!("**Type:** {} ({})  ", tl(fm.element_type.as_ref()), if is_parent { "parent" } else { "leaf" });
    println!("**Status:** {} · domain: {} · SIL: {} · ASIL: {}", status, req_domain, sil, asil);
    println!();

    // ── Parents (derivedFrom) ─────────────────────────────────────────────
    let parents = fm.derived_from.as_deref().unwrap_or(&[]);
    if !parents.is_empty() {
        println!("## Parents (`derivedFrom`)");
        println!();
        println!("| ID | Title | Status |");
        println!("|---|---|---|");
        for parent_ref in parents {
            if let Some(p) = resolve(elements, resolver, parent_ref) {
                let p_id = p.frontmatter.id.as_deref().unwrap_or(&p.qualified_name);
                let p_title = p.frontmatter.title.as_deref().unwrap_or("—");
                let p_status = p.frontmatter.status.as_deref().unwrap_or("—");
                println!("| {} | {} | {} |", p_id, p_title, p_status);
            } else {
                println!("| {} | (not found) | — |", parent_ref);
            }
        }
        println!();
    }

    // ── Breakdown ADR ─────────────────────────────────────────────────────
    if let Some(ref adr_ref) = fm.breakdown_adr {
        println!("## Breakdown ADR");
        println!();
        if let Some(adr) = resolve(elements, resolver, adr_ref) {
            let adr_id = adr.frontmatter.id.as_deref().unwrap_or(&adr.qualified_name);
            let adr_title = adr.frontmatter.title.as_deref().unwrap_or("—");
            let adr_status = adr.frontmatter.status.as_deref().unwrap_or("—");
            println!("- **{}** — {} (`{}`)", adr_id, adr_title, adr_status);
        } else {
            println!("- {} (not found)", adr_ref);
        }
        println!();
    }

    // ── Derived children ─────────────────────────────────────────────────
    if let Some(children) = val.derived_children.get(id) {
        println!("## Derived children");
        println!();
        println!("| ID | Title | Status | Domain |");
        println!("|---|---|---|---|");
        let mut sorted_children = children.clone();
        sorted_children.sort();
        for cid in &sorted_children {
            if let Some(c) = resolve(elements, resolver, cid) {
                let c_title = c.frontmatter.title.as_deref().unwrap_or("—");
                let c_status = c.frontmatter.status.as_deref().unwrap_or("—");
                let c_domain = c.frontmatter.req_domain.as_deref().unwrap_or("—");
                println!("| {} | {} | {} | {} |", cid, c_title, c_status, c_domain);
            } else {
                println!("| {} | (not found) | — | — |", cid);
            }
        }
        println!();
    }

    // ── Satisfied by ──────────────────────────────────────────────────────
    let satisfying: Vec<&RawElement> = elements
        .iter()
        .filter(|e| {
            e.frontmatter
                .satisfies
                .as_ref()
                .map(|s| s.iter().any(|r| r == id))
                .unwrap_or(false)
        })
        .collect();
    if satisfying.is_empty() {
        println!("## Satisfied by");
        println!();
        println!("_(none)_");
        println!();
    } else {
        println!("## Satisfied by");
        println!();
        println!("| Qualified Name | Type | Domain |");
        println!("|---|---|---|");
        for e in &satisfying {
            let type_str = tl(e.frontmatter.element_type.as_ref());
            let dom = e.frontmatter.domain.as_deref().unwrap_or("—");
            println!("| {} | {} | {} |", e.qualified_name, type_str, dom);
        }
        println!();
    }

    // ── Verified by ──────────────────────────────────────────────────────
    let verifiers = val.verified_by.get(id).cloned().unwrap_or_default();
    if verifiers.is_empty() {
        println!("## Verified by");
        println!();
        println!("_(none)_");
        println!();
    } else {
        println!("## Verified by");
        println!();
        println!("| TC ID | Name | Level | Gherkin Scenarios |");
        println!("|---|---|---|---|");
        let mut sorted_v = verifiers.clone();
        sorted_v.sort();
        for tc_id in &sorted_v {
            if let Some(tc) = resolve(elements, resolver, tc_id) {
                let name = tc
                    .frontmatter
                    .name
                    .as_deref()
                    .unwrap_or_else(|| tc.qualified_name.split("::").last().unwrap_or("—"));
                let level = tc.frontmatter.test_level.as_deref().unwrap_or("—");
                let scenarios = gherkin_count(&tc.doc);
                println!("| {} | {} | {} | {} |", tc_id, name, level, scenarios);
            } else {
                println!("| {} | (not found) | — | — |", tc_id);
            }
        }
        println!();
    }
}

// ── cmd: links ───────────────────────────────────────────────────────────────

pub fn cmd_links(elements: &[RawElement], resolver: &Resolver, key: &str) {
    let Some(elem) = resolve(elements, resolver, key) else {
        eprintln!("Element not found: {key}");
        return;
    };

    println!("# Links: {}", elem.qualified_name);
    println!();
    println!("**Type:** {} | **File:** {}", tl(elem.frontmatter.element_type.as_ref()), elem.file_path);
    println!();

    // Outbound
    let out = outbound_refs(elem);
    if out.is_empty() {
        println!("## Outbound relationships");
        println!();
        println!("_(none)_");
        println!();
    } else {
        println!("## Outbound relationships");
        println!();
        println!("| Relationship | Target | Target Type |");
        println!("|---|---|---|");
        for (rel, target) in &out {
            let ttype = resolve(elements, resolver, target)
                .map(|e| tl(e.frontmatter.element_type.as_ref()))
                .unwrap_or("(unresolved)");
            println!("| {} | {} | {} |", rel, target, ttype);
        }
        println!();
    }

    // Inbound
    let target_qn = &elem.qualified_name;
    let target_id = elem.frontmatter.id.as_deref();
    let mut inbound: Vec<(String, String)> = Vec::new(); // (source_qname, rel)
    for other in elements {
        if std::ptr::eq(other, elem) { continue; }
        for (rel, tgt) in outbound_refs(other) {
            if &tgt == target_qn || target_id == Some(tgt.as_str()) {
                inbound.push((other.qualified_name.clone(), rel));
            }
        }
    }
    inbound.sort();
    inbound.dedup();

    if inbound.is_empty() {
        println!("## Inbound relationships");
        println!();
        println!("_(none)_");
        println!();
    } else {
        println!("## Inbound relationships");
        println!();
        println!("| Source | Relationship | Source Type |");
        println!("|---|---|---|");
        for (src, rel) in &inbound {
            let stype = resolver
                .get(elements, src)
                .map(|e| tl(e.frontmatter.element_type.as_ref()))
                .unwrap_or("?");
            println!("| {} | {} | {} |", src, rel, stype);
        }
        println!();
    }
}

// ── cmd: why ─────────────────────────────────────────────────────────────────

pub fn cmd_why(
    elements: &[RawElement],
    resolver: &Resolver,
    val: &ValidationResult,
    key: &str,
) {
    let Some(elem) = resolve(elements, resolver, key) else {
        eprintln!("Element not found: {key}");
        return;
    };

    let satisfies = match &elem.frontmatter.satisfies {
        Some(v) if !v.is_empty() => v.clone(),
        _ => {
            println!("# Why: {}", elem.qualified_name);
            println!();
            println!("This element has no `satisfies` links.");
            return;
        }
    };

    println!("# Why: {}", elem.qualified_name);
    println!();
    println!("**Type:** {} · **Domain:** {}",
        tl(elem.frontmatter.element_type.as_ref()),
        elem.frontmatter.domain.as_deref().unwrap_or("—"));
    println!();

    println!("## Satisfied requirements");
    println!();
    println!("| ID | Title | Status | reqDomain | SIL | ASIL |");
    println!("|---|---|---|---|---|---|");
    let mut req_ids: Vec<String> = Vec::new();
    for req_ref in &satisfies {
        if let Some(req) = resolve(elements, resolver, req_ref) {
            let id = req.frontmatter.id.as_deref().unwrap_or(&req.qualified_name);
            let title = req.frontmatter.title.as_deref().unwrap_or("—");
            let status = req.frontmatter.status.as_deref().unwrap_or("—");
            let rd = req.frontmatter.req_domain.as_deref().unwrap_or("—");
            let sil = req.frontmatter.sil_level.map(|v| v.to_string()).unwrap_or("—".into());
            let asil = req.frontmatter.asil_level.as_deref().unwrap_or("—");
            println!("| {} | {} | {} | {} | {} | {} |", id, title, status, rd, sil, asil);
            req_ids.push(id.to_string());
        } else {
            println!("| {} | (not found) | — | — | — | — |", req_ref);
        }
    }
    println!();

    // Which TCs cover those requirements
    println!("## Verification coverage (via satisfied requirements)");
    println!();
    let mut tcs: Vec<(String, String, String, String)> = Vec::new(); // (tc_id, name, level, req_id)
    for req_id in &req_ids {
        if let Some(verifiers) = val.verified_by.get(req_id.as_str()) {
            for tc_id in verifiers {
                if let Some(tc) = resolve(elements, resolver, tc_id) {
                    let name = tc.frontmatter.name.as_deref()
                        .unwrap_or_else(|| tc.qualified_name.split("::").last().unwrap_or("—"))
                        .to_string();
                    let level = tc.frontmatter.test_level.as_deref().unwrap_or("—").to_string();
                    tcs.push((tc_id.clone(), name, level, req_id.clone()));
                }
            }
        }
    }
    if tcs.is_empty() {
        println!("_(no test cases found for satisfied requirements)_");
    } else {
        tcs.sort_by_key(|(id, _, _, _)| id.clone());
        println!("| TC ID | Name | Level | Covers |");
        println!("|---|---|---|---|");
        for (tc_id, name, level, req_id) in &tcs {
            println!("| {} | {} | {} | {} |", tc_id, name, level, req_id);
        }
    }
    println!();
}

// ── cmd: who-verifies ────────────────────────────────────────────────────────

pub fn cmd_who_verifies(
    elements: &[RawElement],
    resolver: &Resolver,
    val: &ValidationResult,
    key: &str,
) {
    let Some(elem) = resolve(elements, resolver, key) else {
        eprintln!("Element not found: {key}");
        return;
    };
    let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
    let title = elem.frontmatter.title.as_deref().unwrap_or("(no title)");

    println!("# Verification: {}", id);
    println!();
    println!("**Title:** {}", title);
    println!();

    let verifiers = val.verified_by.get(id).cloned().unwrap_or_default();
    if verifiers.is_empty() {
        println!("No test cases verify this requirement.");
        return;
    }

    println!("| TC ID | Name | Level | Gherkin Scenarios | Status |");
    println!("|---|---|---|---|---|");
    let mut sorted = verifiers.clone();
    sorted.sort();
    for tc_id in &sorted {
        if let Some(tc) = resolve(elements, resolver, tc_id) {
            let name = tc.frontmatter.name.as_deref()
                .unwrap_or_else(|| tc.qualified_name.split("::").last().unwrap_or("—"));
            let level = tc.frontmatter.test_level.as_deref().unwrap_or("—");
            let scenarios = gherkin_count(&tc.doc);
            let status = tc.frontmatter.status.as_deref().unwrap_or("—");
            println!("| {} | {} | {} | {} | {} |", tc_id, name, level, scenarios, status);
        } else {
            println!("| {} | (not found) | — | — | — |", tc_id);
        }
    }
    println!();
}

// ── cmd: refs ────────────────────────────────────────────────────────────────

pub fn cmd_refs(elements: &[RawElement], resolver: &Resolver, key: &str) {
    let Some(elem) = resolve(elements, resolver, key) else {
        eprintln!("Element not found: {key}");
        return;
    };
    let target_qn = &elem.qualified_name;
    let target_id = elem.frontmatter.id.as_deref();

    println!("# References to: {}", target_qn);
    println!();

    let mut rows: Vec<(String, String, String)> = Vec::new(); // (source_qname, rel, source_type)
    for other in elements {
        if std::ptr::eq(other, elem) { continue; }
        for (rel, tgt) in outbound_refs(other) {
            if &tgt == target_qn || target_id == Some(tgt.as_str()) {
                let stype = tl(other.frontmatter.element_type.as_ref()).to_string();
                rows.push((other.qualified_name.clone(), rel, stype));
            }
        }
    }
    rows.sort();
    rows.dedup();

    if rows.is_empty() {
        println!("No elements reference `{}`.", target_qn);
        return;
    }

    println!("| Source | Relationship | Type |");
    println!("|---|---|---|");
    for (src, rel, stype) in &rows {
        println!("| {} | {} | {} |", src, rel, stype);
    }
    println!();
    println!("{} reference(s)", rows.len());
}

// ── help ─────────────────────────────────────────────────────────────────────

pub fn print_help() {
    println!("Usage: syscribe <model_root> [command] [args...]");
    println!();
    println!("Commands:");
    println!("  (none)                    Full validation report (default)");
    println!("  show <qname|id>           Show element details and documentation");
    println!("  ls [qname]                List namespace children (default: root)");
    println!("  tree [qname]              Recursive namespace tree (default: root)");
    println!("  find <pattern>            Fuzzy search by name / ID / content");
    println!("  trace <qname|req-id>      Full traceability slice for a requirement");
    println!("  links <qname|id>          All outbound and inbound relationships");
    println!("  why <qname>               What requirements this element satisfies");
    println!("  who-verifies <req-id>     Which test cases cover a requirement");
    println!("  refs <qname|id>           What elements reference this element");
    println!();
    println!("Options:");
    println!("  --agent-instructions      Print the LLM authoring prompt");
    println!("  --help, -h                Show this help");
    println!();
    println!("Examples:");
    println!("  syscribe model/ find FlightController");
    println!("  syscribe model/ show UAV::Avionics::FlightController");
    println!("  syscribe model/ ls UAV::Avionics");
    println!("  syscribe model/ tree UAV");
    println!("  syscribe model/ trace REQ-UAV-FC-001");
    println!("  syscribe model/ links UAV::Avionics::FlightController");
    println!("  syscribe model/ why UAV::Avionics::FlightController");
    println!("  syscribe model/ who-verifies REQ-UAV-SAFE-001");
    println!("  syscribe model/ refs Interfaces::TelemetryPortDef");
}
