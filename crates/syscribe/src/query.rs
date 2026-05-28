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
        // Tier 4
        ElementType::FaultTree => "FaultTree",
        ElementType::FaultTreeGate => "FaultTreeGate",
        ElementType::FaultTreeEvent => "FaultTreeEvent",
        ElementType::FMEASheet => "FMEASheet",
        ElementType::FMEAEntry => "FMEAEntry",
        // TARA container
        ElementType::TARASheet => "TARASheet",
        // Tier 2
        ElementType::HazardousEvent => "HazardousEvent",
        ElementType::SafetyGoal => "SafetyGoal",
        ElementType::DamageScenario => "DamageScenario",
        ElementType::ThreatScenario => "ThreatScenario",
        ElementType::CybersecurityGoal => "CybersecurityGoal",
        ElementType::SecurityControl => "SecurityControl",
        ElementType::VulnerabilityReport => "VulnerabilityReport",
        _ => "Other",
    }
}

fn yaml_first_string(v: Option<&serde_yaml::Value>) -> Option<&str> {
    match v? {
        serde_yaml::Value::String(s) => Some(s.as_str()),
        serde_yaml::Value::Sequence(seq) => seq.first()?.as_str(),
        _ => None,
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

/// Resolve by exact qname, then exact stable ID, then fuzzy best-match.
fn resolve<'a>(elements: &'a [RawElement], resolver: &Resolver, key: &str) -> Option<&'a RawElement> {
    if let Some(e) = resolver.get(elements, key).or_else(|| resolver.get_by_id(elements, key)) {
        return Some(e);
    }
    // Fuzzy fallback: pick the single highest-scoring candidate.
    let mut best_score = 0u32;
    let mut best: Option<&RawElement> = None;
    let mut ambiguous = false;
    for elem in elements {
        let s = fuzzy_score(elem, key);
        if s > best_score {
            best_score = s;
            best = Some(elem);
            ambiguous = false;
        } else if s == best_score && s > 0 {
            ambiguous = true;
        }
    }
    if best_score == 0 {
        return None;
    }
    if ambiguous {
        eprintln!("Ambiguous match for `{key}` — use `find` to see all candidates.");
        return None;
    }
    let matched = best.unwrap();
    eprintln!("(matched: {})", matched.qualified_name);
    Some(matched)
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
            if e.qualified_name.is_empty() {
                return false; // root _index.md — not a tree node
            }
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
    if let Some(ref g) = fm.derived_from_security_goal { out.push(("derivedFromSecurityGoal".into(), g.clone())); }
    if let Some(ref g) = fm.derived_from_safety_goal { out.push(("derivedFromSafetyGoal".into(), g.clone())); }
    if let Some(ref afs) = fm.allocated_from {
        for s in afs { out.push(("allocatedFrom".into(), s.clone())); }
    }
    if let Some(ref ats) = fm.allocated_to {
        for s in ats { out.push(("allocatedTo".into(), s.clone())); }
    }
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
    if let Some(ref rk) = fm.requirement_kind { println!("| **requirementKind** | {} |", rk); }
    if let Some(ref rd) = fm.req_domain { println!("| **reqDomain** | {} |", rd); }
    if let Some(sil) = fm.sil_level { println!("| **SIL** | {} |", sil); }
    if let Some(ref asil) = fm.asil_level { println!("| **ASIL** | {} |", asil); }
    if let Some(ref dal) = fm.dal_level { println!("| **DAL** | {} |", dal); }
    if let Some(ref vm) = fm.verification_method { println!("| **verificationMethod** | {} |", vm); }
    if let Some(ref tl_) = fm.test_level { println!("| **testLevel** | {} |", tl_); }
    if let Some(ref ct) = fm.coverage_target { println!("| **coverageTarget** | {} |", ct); }
    if let Some(ref mul) = fm.multiplicity { println!("| **multiplicity** | {} |", mul); }
    if let Some(ref dir) = fm.direction { println!("| **direction** | {} |", dir); }
    if let Some(ref s) = fm.breakdown_adr { println!("| **breakdownAdr** | {} |", s); }
    if let Some(ref g) = fm.derived_from_security_goal { println!("| **derivedFromSecurityGoal** | {} |", g); }
    if let Some(ref g) = fm.derived_from_safety_goal { println!("| **derivedFromSafetyGoal** | {} |", g); }

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

    // ── Tier 4: FTA/FMEA fields ──────────────────────────────────────────
    if let Some(ref te) = fm.top_event { println!("| **topEvent** | {} |", te); }
    if let Some(ref mt) = fm.mission_time { println!("| **missionTime** | {} |", mt); }
    if let Some(ref gt) = fm.gate_type { println!("| **gateType** | {} |", gt); }
    if let Some(ref inputs) = fm.inputs {
        if !inputs.is_empty() { println!("| **inputs** | {} |", inputs.join(", ")); }
    }
    if let Some(ref ek) = fm.event_kind { println!("| **eventKind** | {} |", ek); }
    if let Some(fr) = fm.failure_rate { println!("| **failureRate** | {} |", fr); }
    if let Some(p) = fm.probability { println!("| **probability** | {} |", p); }
    if let Some(ref fm_) = fm.failure_mode { println!("| **failureMode** | {} |", fm_); }
    if let Some(ref eff) = fm.effect { println!("| **effect** | {} |", eff); }
    if let Some(ref cau) = fm.cause { println!("| **cause** | {} |", cau); }
    if let Some(s) = fm.fmea_severity { println!("| **fmeaSeverity** | {} |", s); }
    if let Some(o) = fm.occurrence { println!("| **occurrence** | {} |", o); }
    if let Some(d) = fm.detection { println!("| **detection** | {} |", d); }
    if let Some(rpn) = fm.rpn { println!("| **RPN** | {} |", rpn); }
    if let Some(ref ra) = fm.recommended_action { println!("| **recommendedAction** | {} |", ra); }

    // ── Tier 2: HARA fields ───────────────────────────────────────────────
    if let Some(ref s) = fm.severity { println!("| **severity** | {} |", s); }
    if let Some(ref e) = fm.exposure { println!("| **exposure** | {} |", e); }
    if let Some(ref c) = fm.controllability { println!("| **controllability** | {} |", c); }
    if let Some(ref os) = fm.operational_situation { println!("| **operationalSituation** | {} |", os); }
    // IEC 61508 risk graph
    if let Some(ref c) = fm.consequence { println!("| **consequence** | {} |", c); }
    if let Some(ref fe) = fm.freq_exposure { println!("| **freqExposure** | {} |", fe); }
    if let Some(ref av) = fm.avoidance { println!("| **avoidance** | {} |", av); }
    if let Some(ref dr) = fm.demand_rate { println!("| **demandRate** | {} |", dr); }
    if let Some(ref ss) = fm.safe_state { println!("| **safeState** | {} |", ss); }
    if let Some(ref ft) = fm.ftti { println!("| **ftti** | {} |", ft); }
    if let Some(ref pl) = fm.pl_level { println!("| **plLevel** | {} |", pl); }
    if let Some(ref hes) = fm.hazardous_events {
        if !hes.is_empty() { println!("| **hazardousEvents** | {} |", hes.join(", ")); }
    }

    // ── Tier 2: TARA fields ───────────────────────────────────────────────
    if let Some(ref ds) = fm.damage_severity { println!("| **damageSeverity** | {} |", ds); }
    if let Some(ref ic) = fm.impact_categories {
        if !ic.is_empty() { println!("| **impactCategories** | {} |", ic.join(", ")); }
    }
    if let Some(ref af) = fm.attack_feasibility { println!("| **attackFeasibility** | {} |", af); }
    if let Some(ref av) = fm.attack_vector { println!("| **attackVector** | {} |", av); }
    if let Some(ref dsc) = fm.damage_scenarios {
        if !dsc.is_empty() { println!("| **damageScenarios** | {} |", dsc.join(", ")); }
    }
    if let Some(ref cl) = fm.cal_level { println!("| **calLevel** | {} |", cl); }
    if let Some(ref sp) = fm.security_property { println!("| **securityProperty** | {} |", sp); }
    if let Some(ref ts) = fm.threat_scenarios {
        if !ts.is_empty() { println!("| **threatScenarios** | {} |", ts.join(", ")); }
    }
    if let Some(ref ct) = fm.control_type { println!("| **controlType** | {} |", ct); }
    if let Some(ref ig) = fm.implements_goals {
        if !ig.is_empty() { println!("| **implementsGoals** | {} |", ig.join(", ")); }
    }
    if let Some(score) = fm.cvss_score { println!("| **cvssScore** | {} |", score); }
    if let Some(ref cve) = fm.cve_id { println!("| **cveId** | {} |", cve); }
    if let Some(ref ae) = fm.affected_elements {
        if !ae.is_empty() { println!("| **affectedElements** | {} |", ae.join(", ")); }
    }
    if let Some(ref mb) = fm.mitigated_by {
        if !mb.is_empty() { println!("| **mitigatedBy** | {} |", mb.join(", ")); }
    }

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

pub fn cmd_untyped(elements: &[RawElement]) {
    let mut matches: Vec<&RawElement> = elements
        .iter()
        .filter(|e| e.frontmatter.element_type.is_none())
        .collect();

    if matches.is_empty() {
        println!("All elements have a type.");
        return;
    }

    matches.sort_by_key(|e| e.qualified_name.as_str());

    println!("| Qualified Name | File |");
    println!("|---|---|");
    for e in &matches {
        println!("| {} | {} |", e.qualified_name, e.file_path);
    }
    println!();
    println!("{} untyped element(s)", matches.len());
}

pub fn cmd_types(elements: &[RawElement]) {
    use std::collections::HashMap;
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for e in elements {
        let label = tl(e.frontmatter.element_type.as_ref());
        *counts.entry(label).or_insert(0) += 1;
    }
    let mut rows: Vec<(&str, usize)> = counts.into_iter().collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    println!("| Type | Count |");
    println!("|---|---|");
    for (label, count) in &rows {
        println!("| {} | {} |", label, count);
    }
}

pub fn cmd_list(elements: &[RawElement], type_filter: &str, scope: &str) {
    let type_filter_lc = type_filter.to_lowercase();
    let mut matches: Vec<&RawElement> = elements
        .iter()
        .filter(|e| {
            let label = tl(e.frontmatter.element_type.as_ref()).to_lowercase();
            label == type_filter_lc
        })
        .filter(|e| scope.is_empty() || e.qualified_name.starts_with(scope))
        .collect();

    if matches.is_empty() {
        let scope_note = if scope.is_empty() { String::new() } else { format!(" in `{scope}`") };
        println!("No `{type_filter}` elements found{scope_note}.");
        return;
    }

    matches.sort_by_key(|e| e.qualified_name.as_str());

    let scope_note = if scope.is_empty() { String::new() } else { format!(" in `{scope}`") };
    println!("# {} elements{} ({})", type_filter, scope_note, matches.len());
    println!();
    println!("| Qualified Name | Name / ID | Supertype / TypedBy | File |");
    println!("|---|---|---|---|");
    for e in &matches {
        let label = e.frontmatter.title
            .as_deref()
            .or_else(|| e.frontmatter.id.as_deref())
            .or_else(|| e.frontmatter.name.as_deref())
            .unwrap_or("—");
        let classifier = yaml_first_string(e.frontmatter.supertype.as_ref())
            .or_else(|| yaml_first_string(e.frontmatter.typed_by.as_ref()))
            .unwrap_or("—");
        println!("| {} | {} | {} | {} |", e.qualified_name, label, classifier, e.file_path);
    }
    println!();
}

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

    // ── Safety Goal (derivedFromSafetyGoal) ──────────────────────────────
    if let Some(ref sg_ref) = fm.derived_from_safety_goal {
        println!("## Safety Goal (`derivedFromSafetyGoal`)");
        println!();
        if let Some(sg) = resolve(elements, resolver, sg_ref) {
            let sg_id = sg.frontmatter.id.as_deref().unwrap_or(&sg.qualified_name);
            let sg_title = sg.frontmatter.title.as_deref().unwrap_or("—");
            let asil = sg.frontmatter.asil_level.as_deref()
                .or_else(|| sg.frontmatter.sil_level.map(|_| "SIL").unwrap_or_default().into())
                .unwrap_or("—");
            let pl = sg.frontmatter.pl_level.as_deref().unwrap_or("");
            let level = if !pl.is_empty() { format!("PL{}", pl) } else { asil.to_string() };
            println!("- **{}** — {} (`{}`)", sg_id, sg_title, level);
        } else {
            println!("- {} (not found)", sg_ref);
        }
        println!();
    }

    // ── Security Goal (derivedFromSecurityGoal) ──────────────────────────
    if let Some(ref csg_ref) = fm.derived_from_security_goal {
        println!("## Security Goal (`derivedFromSecurityGoal`)");
        println!();
        if let Some(csg) = resolve(elements, resolver, csg_ref) {
            let csg_id = csg.frontmatter.id.as_deref().unwrap_or(&csg.qualified_name);
            let csg_title = csg.frontmatter.title.as_deref().unwrap_or("—");
            let cal = csg.frontmatter.cal_level.as_deref().unwrap_or("—");
            let prop = csg.frontmatter.security_property.as_deref().unwrap_or("—");
            println!("- **{}** — {} (`{}` · {})", csg_id, csg_title, cal, prop);
        } else {
            println!("- {} (not found)", csg_ref);
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

pub fn cmd_validate(elements: &[RawElement], file_filter: Option<&str>, json: bool) {
    use syscribe_model::validator;
    use syscribe_model::validator::Severity;

    let result = validator::validate(elements);

    let findings: Vec<_> = result.findings.iter()
        .filter(|f| file_filter.map_or(true, |ff| f.file.contains(ff)))
        .collect();

    let errors: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Error).collect();
    let warnings: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Warning).collect();

    if json {
        let items: Vec<serde_json::Value> = findings.iter().map(|f| {
            serde_json::json!({
                "code": f.code,
                "severity": match f.severity { Severity::Error => "error", Severity::Warning => "warning" },
                "file": f.file,
                "message": f.message,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
        if !errors.is_empty() { std::process::exit(1); }
        return;
    }

    if findings.is_empty() {
        println!("0 errors, 0 warnings — model is valid.");
        return;
    }

    if !errors.is_empty() {
        println!("Errors ({}):", errors.len());
        println!();
        println!("| Code | File | Message |");
        println!("|---|---|---|");
        for f in &errors {
            println!("| {} | {} | {} |", f.code, f.file, f.message);
        }
        println!();
    }

    if !warnings.is_empty() {
        println!("Warnings ({}):", warnings.len());
        println!();
        println!("| Code | File | Message |");
        println!("|---|---|---|");
        for f in &warnings {
            println!("| {} | {} | {} |", f.code, f.file, f.message);
        }
        println!();
    }

    if !errors.is_empty() {
        std::process::exit(1);
    }
}

pub fn cmd_path_for(elements: &[RawElement], resolver: &Resolver, key: &str) {
    match resolve(elements, resolver, key) {
        Some(e) => println!("{}", e.file_path),
        None => {
            eprintln!("Element not found: {key}");
            std::process::exit(1);
        }
    }
}

pub fn cmd_check_ref(elements: &[RawElement], resolver: &Resolver, key: &str) {
    match resolver.resolve_ref(elements, key) {
        Some(e) => {
            let type_str = tl(e.frontmatter.element_type.as_ref());
            println!("resolved  {}", e.qualified_name);
            println!("type      {}", type_str);
            println!("file      {}", e.file_path);
        }
        None => {
            println!("unresolved: '{}' does not match any element qname or stable ID", key);
            std::process::exit(1);
        }
    }
}

pub fn cmd_next_id(elements: &[RawElement], prefix: &str) {
    let prefix_with_dash = format!("{}-", prefix.trim_end_matches('-'));
    let mut max_n: u32 = 0;
    let mut found_any = false;

    for e in elements {
        if let Some(id) = e.frontmatter.id.as_deref() {
            if let Some(suffix) = id.strip_prefix(&prefix_with_dash) {
                if let Ok(n) = suffix.parse::<u32>() {
                    found_any = true;
                    if n > max_n { max_n = n; }
                }
            }
        }
    }

    let next = max_n + 1;
    println!("{}{:03}", prefix_with_dash, next);

    if !found_any {
        eprintln!("(no existing IDs with prefix '{}' — starting from 001)", prefix_with_dash);
    }
}

pub fn cmd_template(type_name: &str) {
    let out = match type_name.to_lowercase().as_str() {
        "requirement" => r#"---
type: Requirement
id: REQ-PREFIX-001
title: "The system shall ..."
status: draft
requirementKind: system
reqDomain: system
verificationMethod: test
# silLevel: 1
# asilLevel: A
# dalLevel: B
# derivedFrom:
#   - REQ-PARENT-001
# breakdownAdr: ADR-XXX-001
# derivedFromSafetyGoal: SG-PREFIX-001
# derivedFromSecurityGoal: CSG-PREFIX-001
---

The system shall ...

## Rationale

Why this requirement exists.
"#,
        "testcase" => r#"---
type: TestCase
id: TC-PREFIX-001
title: "Verify that ..."
status: draft
testLevel: L5
coverageTarget: statement
verifies:
  - REQ-PREFIX-001
---

## Test Procedure

```gherkin
Feature: ...

  Scenario: Normal case
    Given ...
    When ...
    Then ...
```
"#,
        "adr" => r#"---
type: ADR
id: ADR-PREFIX-001
title: "Decision title"
status: proposed
---

## Context

What is the issue that motivates this decision?

## Decision

What was decided.

## Consequences

What are the results of this decision?
"#,
        "partdef" => r#"---
type: PartDef
name: MyPartDef
# supertype: SomePackage::SomeParent
# domain: system
# isAbstract: false
# features:
#   - name: myPort
#     type: Port
#     typedBy: Interfaces::MyPortDef
#     direction: in
#   - name: subPart
#     type: Part
#     typedBy: SomePackage::SomePartDef
---

Description of this part definition.
"#,
        "part" => r#"---
type: Part
name: myPart
typedBy: SomePackage::SomePartDef
# multiplicity: "1"
# domain: system
# satisfies:
#   - REQ-PREFIX-001
---

Description of this part usage.
"#,
        "portdef" => r#"---
type: PortDef
name: MyPortDef
# direction: in
# conjugates: Interfaces::MyPortDef
# features:
#   - name: signal
#     type: Attribute
#     typedBy: ScalarValues::Real
#     direction: in
# operations:
#   - name: read
#     returnType: ScalarValues::Boolean
---

Description of this port definition.
"#,
        "port" => r#"---
type: Port
name: myPort
typedBy: Interfaces::MyPortDef
# direction: in
# isConjugated: false
# multiplicity: "1"
---

Description of this port usage.
"#,
        "connectiondef" => r#"---
type: ConnectionDef
name: MyConnectionDef
# supertype: Interfaces::SomeConnectionDef
# ends:
#   - typedBy: Interfaces::MyPortDef
#     direction: in
#   - typedBy: Interfaces::MyPortDef
#     direction: out
---

Description of this connection definition.
"#,
        "connection" => r#"---
type: Connection
name: myConnection
typedBy: Interfaces::MyConnectionDef
connections:
  - from: partA::outPort
    to: partB::inPort
---
"#,
        "interfacedef" => r#"---
type: InterfaceDef
name: MyInterfaceDef
# features:
#   - name: dataIn
#     type: Port
#     typedBy: Interfaces::MyPortDef
#     direction: in
---

Description of this interface definition.
"#,
        "actiondef" => r#"---
type: ActionDef
name: MyActionDef
# supertype: Actions::SomeActionDef
# parameters:
#   - name: input
#     typedBy: ScalarValues::Real
#     direction: in
# returnType: ScalarValues::Real
---

Description of this action definition.
"#,
        "constraintdef" => r#"---
type: ConstraintDef
name: MyConstraintDef
# parameters:
#   - name: value
#     typedBy: ScalarValues::Real
# expression: "value > 0.0"
---

Description of this constraint definition.
"#,
        "calculationdef" => r#"---
type: CalculationDef
name: MyCalculationDef
# returnType: ScalarValues::Real
# parameters:
#   - name: input
#     typedBy: ScalarValues::Real
#     direction: in
---

Description of this calculation definition.
"#,
        "statedef" => r#"---
type: StateDef
name: MyStateDef
subStates:
  - name: Idle
  - name: Active
  - name: Fault
transitions:
  - from: Idle
    to: Active
    trigger: start
  - from: Active
    to: Fault
    trigger: error
  - from: Fault
    to: Idle
    trigger: reset
---

Description of this state machine.
"#,
        "flowdef" => r#"---
type: FlowDef
name: MyFlowDef
itemType: Items::MyItemDef
---

Description of this flow definition.
"#,
        "enumerationdef" => r#"---
type: EnumerationDef
name: MyEnumerationDef
values:
  - VALUE_ONE
  - VALUE_TWO
  - VALUE_THREE
---

Description of this enumeration.
"#,
        "usecasedef" => r#"---
type: UseCaseDef
name: MyUseCaseDef
actors:
  - Operator
# includes:
#   - UseCases::SomeOtherUseCase
---

Description of this use case.
"#,
        "requirementdef" => r#"---
type: RequirementDef
name: MyRequirementDef
# supertype: Requirements::SomeRequirementDef
---

Description of this requirement definition.
"#,
        "allocationdef" => r#"---
type: AllocationDef
name: MyAllocationDef
---

Description of this allocation definition.
"#,
        "allocation" => r#"---
type: Allocation
name: MyAllocation
allocatedFrom: Software::MySwComponent
allocatedTo: Hardware::MyHwComponent
---
"#,
        "viewdef" => r#"---
type: ViewDef
name: MyViewDef
# viewpoint: Viewpoints::SystemsEngineerViewpoint
# expose:
#   - SomePackage::SomeElement
---

Description of this view.
"#,
        "viewpointdef" => r#"---
type: ViewpointDef
name: MyViewpointDef
stakeholders:
  - Systems Engineer
concerns:
  - System structure
  - Interface definition
---

Description of this viewpoint.
"#,
        "metadatadef" => r#"---
type: MetadataDef
name: MyMetadataDef
annotates:
  - PartDef
# features:
#   - name: value
#     typedBy: ScalarValues::String
---

Description of this metadata definition.
"#,
        "package" => r#"---
type: Package
name: MyPackage
---

Description of this package.
"#,
        "featuredef" => r#"---
type: FeatureDef
name: MyFeatureDef
groupKind: optional
# cardinality: "1"
# parentFeature: Features::ParentFeatureDef
# excludes:
#   - Features::ConflictingFeatureDef
---

Description of this feature definition.
"#,
        "faulttree" => r#"---
type: FaultTree
id: FT-PREFIX-001
title: "Fault tree for [undesired top event]"
status: draft
topEvent: SG-PREFIX-001     # SafetyGoal this tree analyses
# missionTime: "1e9 h"
---

Describe the analysis scope and methodology.
"#,
        "faulttreegate" => r#"---
type: FaultTreeGate
id: FTG-PREFIX-001
title: "OR gate — [description]"
gateType: OR                # AND | OR | XOR | NOT | inhibit
inputs:
  - FTG-PREFIX-002          # child gate
  - FTE-PREFIX-001          # or leaf event
# probability: 1.2e-7       # optional; computed from inputs
---
"#,
        "faulttreeevent" => r#"---
type: FaultTreeEvent
id: FTE-PREFIX-001
title: "[Component] [failure description]"
eventKind: basic            # basic | undeveloped | house
# ref: Package::Component   # model element this failure belongs to
# failureRate: 1.0e-9       # failures per hour (basic events)
# probability: 1.0e-6
---
"#,
        "fmeasheet" => r#"---
type: FMEASheet
id: FMEA-PREFIX-001
title: "FMEA — [system or component name]"
status: draft
entries:
  - id: FM-PREFIX-001
    ref: Package::Component
    failureMode: "Loss of output signal"
    effect: "No command issued"
    cause: "Software exception in main loop"
    severity: 9             # 1–10
    occurrence: 3           # 1–10
    detection: 4            # 1–10
    # rpn: 108              # computed automatically if omitted
    recommendedAction: "Add watchdog monitor"
    # satisfies: REQ-PREFIX-001
  - id: FM-PREFIX-002
    ref: Package::SensorA
    failureMode: "Stuck-at-high output"
    effect: "False positive reading"
    cause: "Hardware fault"
    severity: 7
    occurrence: 2
    detection: 6
    recommendedAction: "Add redundant sensor cross-check"
---

## Scope

Describe the system boundary and assumptions for this FMEA.
"#,
        "fmeaentry" => {
            eprintln!("FMEAEntry elements are synthesised from FMEASheet entries — use `template FMEASheet` instead.");
            std::process::exit(1);
        }
        "tarasheet" => r#"---
type: TARASheet
id: TARA-PREFIX-001
title: "TARA — [system or asset name]"
status: draft
damageTable:
  - id: DS-PREFIX-001
    title: "Unauthorized [action] enables [damage]"
    damageSeverity: severe    # severe | major | moderate | negligible
    impactCategories:
      - safety                # safety | financial | operational | privacy
threatTable:
  - id: TS-PREFIX-001
    title: "Attacker [action] via [attack surface]"
    attackFeasibility: medium # high | medium | low | very_low
    attackVector: network     # network | adjacent | local | physical
    damageScenarios:
      - DS-PREFIX-001
goalTable:
  - id: CSG-PREFIX-001
    title: "Ensure [security property] of [asset]"
    calLevel: CAL3            # CAL1 | CAL2 | CAL3 | CAL4
    securityProperty: integrity # confidentiality | integrity | availability | authenticity
    threatScenarios:
      - TS-PREFIX-001
controlTable:
  - id: SC-PREFIX-001
    title: "Implement [control mechanism]"
    controlType: prevention   # prevention | detection | response | recovery
    implementsGoals:
      - CSG-PREFIX-001
---

## Scope

Describe the system boundary, assets in scope, and assumptions for this TARA.

## Methodology

Reference the threat modelling approach used (e.g. STRIDE, PASTA, TARA per ISO/SAE 21434).
"#,
        "hazardousevent" => r#"---
type: HazardousEvent
id: HE-PREFIX-001
title: "Loss of [function] during [operating scenario]"
status: draft
# ── ISO 26262 HARA parameters ──────────────────────────────────────────────
# severity: S3         # S0 no injury | S1 light | S2 severe | S3 life-threatening
# exposure: E4         # E0 incredibly unlikely … E4 high probability
# controllability: C2  # C0 controllable | C1 simply | C2 normally | C3 uncontrollable
# asilLevel: D         # derived from S×E×C; can also be set directly
# operationalSituation: "Vehicle traveling >80 km/h on curved road"
# ── IEC 61508 risk graph parameters (use instead of S/E/C for non-automotive) ──
# consequence: Cc      # Ca slight | Cb serious | Cc death of one | Cd death of several
# freqExposure: Fb     # Fa rare/unlikely | Fb frequent/likely
# avoidance: Pb        # Pa possible | Pb barely possible
# demandRate: W3       # W1 very slight | W2 slight | W3 relatively high
---

Describe the hazardous situation: what goes wrong, under what conditions, and what harm could result.

## Rationale

Why this event was identified in the HARA.
"#,
        "safetygoal" => r#"---
type: SafetyGoal
id: SG-PREFIX-001
title: "Prevent [hazard] to avoid [harm]"
status: draft
# ── Integrity level — choose one standard ─────────────────────────────────
asilLevel: D           # ISO 26262: A | B | C | D
# silLevel: 2          # IEC 61508: 1 | 2 | 3 | 4
# plLevel: d           # ISO 13849-1: a | b | c | d | e
# ──────────────────────────────────────────────────────────────────────────
# safeState: "Controlled stop with residual braking"
# ftti: "100ms"
hazardousEvents:
  - HE-PREFIX-001
---

The system shall avoid [hazard] in all driving situations.

## Rationale

Why this safety goal derives from the listed hazardous events.
"#,
        "damagescenario" => r#"---
type: DamageScenario
id: DS-PREFIX-001
title: "Unauthorized [action] enables [damage]"
status: draft
damageSeverity: severe    # severe | major | moderate | negligible
impactCategories:
  - safety                # safety | financial | operational | privacy
---

Describe what damage could occur and to whom.
"#,
        "threatscenario" => r#"---
type: ThreatScenario
id: TS-PREFIX-001
title: "Attacker [action] via [attack surface]"
status: draft
attackFeasibility: medium  # high | medium | low | very_low
attackVector: network      # network | adjacent | local | physical
damageScenarios:
  - DS-PREFIX-001
---

Describe how the threat could be realized and which damage scenarios it enables.
"#,
        "cybersecuritygoal" => r#"---
type: CybersecurityGoal
id: CSG-PREFIX-001
title: "Ensure [security property] of [asset]"
status: draft
calLevel: CAL3            # CAL1 | CAL2 | CAL3 | CAL4
securityProperty: integrity # confidentiality | integrity | availability | authenticity
threatScenarios:
  - TS-PREFIX-001
---

The [asset] shall maintain [security property] against the identified threat scenarios.
"#,
        "securitycontrol" => r#"---
type: SecurityControl
id: SC-PREFIX-001
title: "Implement [control mechanism]"
status: draft
controlType: prevention   # prevention | detection | response | recovery
implementsGoals:
  - CSG-PREFIX-001
# satisfies: REQ-SEC-PREFIX-001
---

Describe the security control mechanism, its scope, and how it implements the referenced cybersecurity goals.

## Architecture allocation

Record which component implements this control by adding `allocatedFrom:` to
the implementing `Part` or `PartDef` (the architecture element holds the
reference, keeping the link OSLC-compliant):

```yaml
# In Package/Component.md
type: PartDef
allocatedFrom: SC-PREFIX-001   # this component implements this security control
```
"#,
        "vulnerabilityreport" => r#"---
type: VulnerabilityReport
id: VR-PREFIX-001
title: "Stack buffer overflow in [component]"
status: open              # open | mitigated | accepted | resolved
# cvssScore: 8.1
# cveId: CVE-2024-12345
affectedElements:
  - Package::Component
mitigatedBy:
  - SC-PREFIX-001
---

## Summary

Describe the vulnerability, its root cause, and potential impact.

## Reproduction

Steps to reproduce (if applicable).

## Mitigation

How the vulnerability is being addressed.
"#,
        other => {
            eprintln!("Unknown type '{}'. Known types:", other);
            eprintln!("  Requirement, TestCase, ADR");
            eprintln!("  PartDef, Part, PortDef, Port, ConnectionDef, Connection");
            eprintln!("  InterfaceDef, ActionDef, ConstraintDef, CalculationDef");
            eprintln!("  StateDef, FlowDef, EnumerationDef, UseCaseDef");
            eprintln!("  RequirementDef, AllocationDef, Allocation");
            eprintln!("  ViewDef, ViewpointDef, MetadataDef, Package, FeatureDef");
            eprintln!("  HazardousEvent, SafetyGoal");
            eprintln!("  DamageScenario, ThreatScenario, CybersecurityGoal, SecurityControl, VulnerabilityReport");
            eprintln!("  FaultTree, FaultTreeGate, FaultTreeEvent, FMEASheet, TARASheet");
            std::process::exit(1);
        }
    };
    print!("{}", out);
}

pub fn print_help() {
    println!("Usage: syscribe <model_root> [command] [args...]");
    println!();
    println!("Commands:");
    println!("  (none)                         Full validation report (default)");
    println!("  validate [--json] [--file <f>] Validation findings only (errors + warnings)");
    println!("  types                          List all element types present in the model with counts");
    println!("  untyped                        List elements with no type: field set");
    println!("  list <type> [scope]            List all elements of a given type (optional namespace scope)");
    println!("  show <qname|id>                Show element details and documentation");
    println!("  ls [qname]                     List namespace children (default: root)");
    println!("  tree [qname]                   Recursive namespace tree (default: root)");
    println!("  find <pattern>                 Fuzzy search by name / ID / content");
    println!("  path-for <qname|id>            Print the file path for an element");
    println!("  check-ref <qname|id>           Verify a cross-reference resolves and show its type");
    println!("  next-id <id-prefix>            Print the next available stable ID for a prefix");
    println!("  template <type>                Print a ready-to-fill frontmatter skeleton for a type");
    println!("  trace <qname|req-id>           Full traceability slice for a requirement");
    println!("  links <qname|id>               All outbound and inbound relationships");
    println!("  why <qname>                    What requirements this element satisfies");
    println!("  who-verifies <req-id>          Which test cases cover a requirement");
    println!("  refs <qname|id>                What elements reference this element");
    println!();
    println!("Options:");
    println!("  --agent-instructions           Print the LLM authoring prompt");
    println!("  --help, -h                     Show this help");
    println!();
    println!("Examples:");
    println!("  syscribe model/ validate");
    println!("  syscribe model/ validate --json");
    println!("  syscribe model/ validate --file model/UAV/Avionics/FlightController.md");
    println!("  syscribe model/ list PartDef");
    println!("  syscribe model/ list PortDef UAV::Avionics");
    println!("  syscribe model/ path-for UAV::Avionics::FlightController");
    println!("  syscribe model/ check-ref Interfaces::TelemetryPortDef");
    println!("  syscribe model/ next-id REQ-UAV-FC");
    println!("  syscribe model/ template PartDef");
    println!("  syscribe model/ template Requirement");
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
