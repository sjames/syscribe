use syscribe_model::{
    element::{ElementType, RawElement},
    resolver::Resolver,
    results::{FnVerdict, ResultsData},
    validator::ValidationResult,
};

// ── Executed-evidence verdict (issue #21) ──────────────────────────────────────

/// Aggregated executed-evidence verdict for a single TestCase.
///
/// Computed over the TestCase's `testFunctions[].function` references against an
/// ingested results sidecar:
///   * `Unknown` — the TestCase has no `testFunctions` (or no results are loaded);
///   * `Fail`    — any function's ingested verdict is `Fail`;
///   * `Pass`    — every function passed;
///   * `Unknown` — otherwise (some `Ignored`/`Missing`, none `Fail`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcVerdict {
    Pass,
    Fail,
    Unknown,
}

/// The `function` strings declared under a TestCase's `testFunctions:`.
fn tc_function_refs(tc: &RawElement) -> Vec<String> {
    let func_key = serde_yaml::Value::String("function".into());
    tc.frontmatter
        .test_functions
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter_map(|tf| match tf {
            serde_yaml::Value::Mapping(map) => match map.get(&func_key) {
                Some(serde_yaml::Value::String(f)) => Some(f.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// Aggregate a TestCase's ingested verdict. `None` results → `Unknown`.
pub fn tc_verdict(tc: &RawElement, results: Option<&ResultsData>) -> TcVerdict {
    let Some(results) = results else {
        return TcVerdict::Unknown;
    };
    let funcs = tc_function_refs(tc);
    if funcs.is_empty() {
        return TcVerdict::Unknown;
    }
    let mut all_pass = true;
    for f in &funcs {
        match results.verdict_for(f) {
            FnVerdict::Fail => return TcVerdict::Fail,
            FnVerdict::Pass => {}
            FnVerdict::Ignored | FnVerdict::Missing => all_pass = false,
        }
    }
    if all_pass {
        TcVerdict::Pass
    } else {
        TcVerdict::Unknown
    }
}

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
        ElementType::AttackTree => "AttackTree",
        ElementType::AttackTreeGate => "AttackTreeGate",
        ElementType::AttackStep => "AttackStep",
        ElementType::FMEASheet => "FMEASheet",
        ElementType::FMEAEntry => "FMEAEntry",
        // GSN argument layer (issue #20)
        ElementType::Argument => "Argument",
        ElementType::AssumptionOfUse => "AssumptionOfUse",
        // TARA container
        ElementType::TARASheet => "TARASheet",
        // Asset identification (ISO/SAE 21434 §15.3)
        ElementType::Asset => "Asset",
        // Tier 2
        ElementType::HazardousEvent => "HazardousEvent",
        ElementType::SafetyGoal => "SafetyGoal",
        ElementType::DamageScenario => "DamageScenario",
        ElementType::ThreatScenario => "ThreatScenario",
        ElementType::CybersecurityGoal => "CybersecurityGoal",
        ElementType::SecurityControl => "SecurityControl",
        ElementType::VulnerabilityReport => "VulnerabilityReport",
        ElementType::ConfirmationMeasure => "ConfirmationMeasure",
        // Previously fell through to "Other" (mislabelled in show/list) — GH #42 follow-up.
        ElementType::ConcernDef => "ConcernDef",
        ElementType::Concern => "Concern",
        ElementType::CaseDef => "CaseDef",
        ElementType::EventOccurrenceDef => "EventOccurrenceDef",
        ElementType::EventOccurrence => "EventOccurrence",
        ElementType::SuccessionDef => "SuccessionDef",
        ElementType::RenderingDef => "RenderingDef",
        ElementType::ExhibitState => "ExhibitState",
        ElementType::BindingConnector => "BindingConnector",
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

// ── Custom fields (GH #39) ─────────────────────────────────────────────────────

/// Render a single YAML *scalar* (string/number/bool/null) as its plain string
/// form. Non-scalars fall back to `serde_yaml`'s compact representation (used only
/// as a defensive default — well-shaped custom fields are scalars or lists thereof).
fn yaml_scalar_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::Null => "null".to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::String(s) => s.clone(),
        other => serde_yaml::to_string(other)
            .unwrap_or_default()
            .trim()
            .to_string(),
    }
}

/// Render a custom-field value for display: scalars inline, lists comma-joined.
fn custom_field_display(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::Sequence(items) => items
            .iter()
            .map(yaml_scalar_string)
            .collect::<Vec<_>>()
            .join(", "),
        other => yaml_scalar_string(other),
    }
}

/// A parsed `--where` predicate over the `custom.<key>` namespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomWhere {
    /// `custom.<key>` — the field is present (any value).
    Present { key: String },
    /// `custom.<key>=<val>` — scalar equals val, or any list element equals val.
    Eq { key: String, val: String },
    /// `custom.<key>=~<pat>` — regex (or substring fallback) match on the value's
    /// string form (scalar string, or the comma-joined list form).
    Regex { key: String, pat: String },
    /// `custom.<key>~=<val>` — list membership: the list contains val (also true
    /// when a scalar field equals val).
    Member { key: String, val: String },
}

/// Parse a `--where` argument. Operators are matched longest-first so the two-char
/// spellings (`=~`, `~=`) win over the one-char `=`. Returns `Err(message)` for an
/// unparseable predicate (missing `custom.` prefix, empty key).
///
/// Operator precedence (checked in this order): `=~`, then `~=`, then `=`, then the
/// bare presence form. `=~` uses the `regex` crate; an invalid regex pattern falls
/// back to a plain substring test (so any literal pattern still works).
pub fn parse_custom_where(arg: &str) -> Result<CustomWhere, String> {
    let body = arg.strip_prefix("custom.").ok_or_else(|| {
        format!("--where predicate must address the `custom.<key>` namespace: '{arg}'")
    })?;

    let mk_key = |k: &str| -> Result<String, String> {
        if k.is_empty() {
            Err(format!("--where predicate has an empty custom-field key: '{arg}'"))
        } else {
            Ok(k.to_string())
        }
    };

    // Longest operators first: `=~` and `~=` before `=`.
    if let Some((k, pat)) = body.split_once("=~") {
        return Ok(CustomWhere::Regex { key: mk_key(k)?, pat: pat.to_string() });
    }
    if let Some((k, val)) = body.split_once("~=") {
        return Ok(CustomWhere::Member { key: mk_key(k)?, val: val.to_string() });
    }
    if let Some((k, val)) = body.split_once('=') {
        return Ok(CustomWhere::Eq { key: mk_key(k)?, val: val.to_string() });
    }
    // Bare presence form.
    Ok(CustomWhere::Present { key: mk_key(body)? })
}

/// True when `elem`'s `custom_fields` satisfy the predicate. An element without the
/// named field never matches (including the presence form).
pub fn custom_field_matches(elem: &RawElement, pred: &CustomWhere) -> bool {
    let key = match pred {
        CustomWhere::Present { key }
        | CustomWhere::Eq { key, .. }
        | CustomWhere::Regex { key, .. }
        | CustomWhere::Member { key, .. } => key,
    };
    let Some(value) = elem.frontmatter.custom_fields.get(key) else {
        return false;
    };

    match pred {
        CustomWhere::Present { .. } => true,
        CustomWhere::Eq { val, .. } => match value {
            serde_yaml::Value::Sequence(items) => {
                items.iter().any(|x| &yaml_scalar_string(x) == val)
            }
            other => &yaml_scalar_string(other) == val,
        },
        CustomWhere::Regex { pat, .. } => {
            let hay = custom_field_display(value);
            match regex::Regex::new(pat) {
                Ok(re) => re.is_match(&hay),
                // Invalid regex → substring fallback so literal patterns still work.
                Err(_) => hay.contains(pat),
            }
        }
        CustomWhere::Member { val, .. } => match value {
            serde_yaml::Value::Sequence(items) => {
                items.iter().any(|x| &yaml_scalar_string(x) == val)
            }
            // A scalar field "contains" val iff it equals val.
            other => &yaml_scalar_string(other) == val,
        },
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
    if let Some(ref aw) = fm.applies_when {
        if let Ok(Some(expr)) = syscribe_model::variability::applies_when_expr(aw) {
            for op in expr.operands() {
                out.push(("appliesWhen".into(), op));
            }
        }
    }
    if let Some(ref s) = fm.breakdown_adr { out.push(("breakdownAdr".into(), s.clone())); }
    if let Some(ref g) = fm.derived_from_cybersecurity_goal { out.push(("derivedFromCybersecurityGoal".into(), g.clone())); }
    if let Some(ref g) = fm.derived_from_safety_goal { out.push(("derivedFromSafetyGoal".into(), g.clone())); }
    if let Some(ref ss) = fm.supports {
        for s in ss { out.push(("supports".into(), s.clone())); }
    }
    if let Some(ref ev) = fm.evidence {
        for s in ev { out.push(("evidence".into(), s.clone())); }
    }
    if let Some(ref at) = fm.applies_to {
        for s in at { out.push(("appliesTo".into(), s.clone())); }
    }
    if let Some(ref afs) = fm.allocated_from {
        for s in afs { out.push(("allocatedFrom".into(), s.clone())); }
    }
    if let Some(ref ats) = fm.allocated_to {
        for s in ats { out.push(("allocatedTo".into(), s.clone())); }
    }
    if let Some(ref es) = fm.exhibits_states {
        for s in es { out.push(("exhibitsStates".into(), s.clone())); }
    }
    if let Some(ref impls) = fm.implemented_by {
        for s in impls { out.push(("implementedBy".into(), s.clone())); }
    }
    if let Some(ref s) = fm.subject { out.push(("subject".into(), s.clone())); }
    if let Some(ref r) = fm.fmea_ref { out.push(("fmeaRef".into(), r.clone())); }
    if let Some(ref r) = fm.fta_ref { out.push(("ftaRef".into(), r.clone())); }
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

pub fn cmd_show(
    elements: &[RawElement],
    resolver: &Resolver,
    val: &ValidationResult,
    key: &str,
) {
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
    // Applied stereotypes (MetadataDef applications, REQ-TRS-META-001) as «Name» banners.
    {
        let apps = syscribe_model::element::metadata_applications(&fm.metadata);
        if !apps.is_empty() {
            let parts: Vec<String> = apps
                .iter()
                .map(|a| {
                    let nm = a.def.rsplit("::").next().unwrap_or(&a.def);
                    if a.values.is_empty() {
                        format!("«{}»", nm)
                    } else {
                        let vs: Vec<String> = a
                            .values
                            .iter()
                            .map(|(k, v)| format!("{}={}", k, custom_field_display(v)))
                            .collect();
                        format!("«{}» {}", nm, vs.join(", "))
                    }
                })
                .collect();
            println!("| **stereotypes** | {} |", parts.join(", "));
        }
    }
    if let Some(ref refs) = fm.ext_ref { println!("| **extRef** | {} |", refs.join(", ")); }
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
    if let Some(ref g) = fm.derived_from_cybersecurity_goal { println!("| **derivedFromCybersecurityGoal** | {} |", g); }
    if let Some(ref g) = fm.derived_from_safety_goal { println!("| **derivedFromSafetyGoal** | {} |", g); }
    if let Some(ref at) = fm.argument_type { println!("| **argumentType** | {} |", at); }
    if let Some(ref ss) = fm.supports { if !ss.is_empty() { println!("| **supports** | {} |", ss.join(", ")); } }
    if let Some(ref ev) = fm.evidence { if !ev.is_empty() { println!("| **evidence** | {} |", ev.join(", ")); } }
    if let Some(ref at) = fm.applies_to { if !at.is_empty() { println!("| **appliesTo** | {} |", at.join(", ")); } }
    if let Some(ref f) = fm.feature_model { println!("| **featureModel** | {} |", f); }
    if let Some(ref aw) = fm.applies_when {
        let aw_str = match aw {
            serde_yaml::Value::String(s) => s.clone(),
            other => yaml_strings(other).join(", "),
        };
        println!("| **appliesWhen** | {} |", aw_str);
    }

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

    // ── Tier 4: FTA/FMEA/APA fields ──────────────────────────────────────
    if let Some(ref te) = fm.top_event { println!("| **topEvent** | {} |", te); }
    if let Some(ref tr) = fm.threat_ref { println!("| **threatRef** | {} |", tr); }
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

    // Feature selections (Configuration §9.8) — show the parsed `features:` map so
    // a mis-authored configuration (e.g. legacy `selections:`) is visibly empty.
    if fm.element_type.as_ref() == Some(&ElementType::Configuration) {
        let sel = fm.feature_selections();
        println!();
        println!("## Feature selections");
        println!();
        if sel.is_empty() {
            println!("_(none parsed — selections must be a `features:` map of `<FeatureDef>: true/false`)_");
        } else {
            println!("| Feature | Selected |");
            println!("|---|---|");
            for (feat, on) in &sel {
                println!("| {} | {} |", feat, on);
            }
        }
    }

    // Features table (inline feature declarations — not Configuration selections)
    if fm.element_type.as_ref() != Some(&ElementType::Configuration) {
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
    }

    // Refined by (MagicGrid «refine», REQ-TRS-MG-001) — the use cases that
    // `refines:` this requirement. Keyed by stable id when present, else qname.
    let mg_key = fm.id.as_deref().unwrap_or(elem.qualified_name.as_str());
    if let Some(ucs) = val.refined_by.get(mg_key) {
        if !ucs.is_empty() {
            println!();
            println!("## Refined by");
            println!();
            println!("| Use Case | Type | Status |");
            println!("|---|---|---|");
            let mut sorted = ucs.clone();
            sorted.sort();
            for uc_ref in &sorted {
                if let Some(uc) = resolve(elements, resolver, uc_ref) {
                    let ty = tl(uc.frontmatter.element_type.as_ref());
                    let st = uc.frontmatter.status.as_deref().unwrap_or("—");
                    println!("| {} | {} | {} |", uc_ref, ty, st);
                } else {
                    println!("| {} | (not found) | — |", uc_ref);
                }
            }
        }
    }

    // Refined by — MoPs (MagicGrid mopRefinedBy, REQ-TRS-MG-008) — the
    // Measurements of Performance whose `mg_mop_refines:` names this MoE. Keyed by
    // stable id when present, else qname (matches the validator's index_key).
    let mop_key = fm.id.as_deref().unwrap_or(elem.qualified_name.as_str());
    if let Some(mops) = val.mop_refined_by.get(mop_key) {
        if !mops.is_empty() {
            println!();
            println!("## Refined by (MoP)");
            println!();
            println!("| MoP | Type | Status |");
            println!("|---|---|---|");
            let mut sorted = mops.clone();
            sorted.sort();
            for mop_ref in &sorted {
                if let Some(mop) = resolve(elements, resolver, mop_ref) {
                    let ty = tl(mop.frontmatter.element_type.as_ref());
                    let st = mop.frontmatter.status.as_deref().unwrap_or("—");
                    println!("| {} | {} | {} |", mop_ref, ty, st);
                } else {
                    println!("| {} | (not found) | — |", mop_ref);
                }
            }
        }
    }

    // Actor participation (MagicGrid actorIn, REQ-TRS-MG-002) — surfaced like the
    // requirement reverse indices: the use cases that name this part as an actor.
    // Keyed by stable id when present, else qualified name (matches the validator).
    let actor_key = fm.id.as_deref().unwrap_or(elem.qualified_name.as_str());
    if let Some(ucs) = val.actor_in.get(actor_key) {
        if !ucs.is_empty() {
            println!();
            println!("## Actor in");
            println!();
            println!("| Use Case | Type | Status |");
            println!("|---|---|---|");
            let mut sorted = ucs.clone();
            sorted.sort();
            for uc_ref in &sorted {
                if let Some(uc) = resolve(elements, resolver, uc_ref) {
                    let ty = tl(uc.frontmatter.element_type.as_ref());
                    let st = uc.frontmatter.status.as_deref().unwrap_or("—");
                    println!("| {} | {} | {} |", uc_ref, ty, st);
                } else {
                    println!("| {} | (not found) | — |", uc_ref);
                }
            }
        }
    }

    // Allocated from (REQ-TRS-ALLOC-001) — the derived reverse index: every
    // source element allocated to this target, aggregated over both authoring
    // forms (`allocatedTo`-on-source and the standalone `Allocation` element).
    // Keyed by stable id when present, else qname (matches the validator).
    let alloc_key = fm.id.as_deref().unwrap_or(elem.qualified_name.as_str());
    if let Some(srcs) = val.allocated_from.get(alloc_key) {
        if !srcs.is_empty() {
            println!();
            println!("## Allocated from");
            println!();
            println!("| Source | Type | Status |");
            println!("|---|---|---|");
            let mut sorted = srcs.clone();
            sorted.sort();
            for src_ref in &sorted {
                if let Some(src) = resolve(elements, resolver, src_ref) {
                    let ty = tl(src.frontmatter.element_type.as_ref());
                    let st = src.frontmatter.status.as_deref().unwrap_or("—");
                    println!("| {} | {} | {} |", src_ref, ty, st);
                } else {
                    println!("| {} | (not found) | — |", src_ref);
                }
            }
        }
    }

    // Derived fields (issue #60) — computed by the derive: block.
    if !elem.derived.is_empty() {
        println!();
        println!("## Derived Fields");
        println!();
        println!("| Field | Value |");
        println!("|---|---|");
        let mut keys: Vec<_> = elem.derived.keys().collect();
        keys.sort();
        for key in keys {
            if let Some(val) = elem.derived.get(key) {
                println!("| {} | {} |", key, custom_field_display(val));
            }
        }
    }

    // Custom fields (GH #39) — read-only labelled section; scalars inline, lists
    // comma-joined. Absent → no section. Keys render in sorted order (BTreeMap).
    if !fm.custom_fields.is_empty() {
        println!();
        println!("## Custom Fields");
        println!();
        println!("| Field | Value |");
        println!("|---|---|");
        for (key, value) in &fm.custom_fields {
            println!("| {} | {} |", key, custom_field_display(value));
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

pub fn cmd_ls(elements: &[RawElement], parent: &str, wheres: &[CustomWhere]) {
    let mut children = ns_children(elements, parent);
    // `--where` custom-field predicates (GH #39) — ANDed with each other.
    children.retain(|c| wheres.iter().all(|w| custom_field_matches(c, w)));
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

pub fn cmd_list(
    elements: &[RawElement],
    type_filter: &str,
    scope: &str,
    tags: &[&str],
    feature: Option<&str>,
    metadata: Option<&str>,
    status: Option<&str>,
    sil: Option<&str>,
    has_wcet: bool,
    wheres: &[CustomWhere],
    json: bool,
) {
    // `--feature <F>`: keep only elements whose `appliesWhen:` names F as an
    // operand. The feature must resolve to a known FeatureDef.
    if let Some(feat) = feature {
        let known = elements.iter().any(|e| {
            e.frontmatter.element_type.as_ref()
                == Some(&syscribe_model::element::ElementType::FeatureDef)
                && e.qualified_name == feat
        });
        if !known {
            eprintln!("Error: unknown feature '{}' (no matching FeatureDef)", feat);
            std::process::exit(1);
        }
    }
    // Effective appliesWhen honours transitive package conditioning (REQ-TRS-VAR-006).
    let lpkg = syscribe_model::variability::package_conditions(elements);
    let gates_feature = |e: &RawElement, feat: &str| -> bool {
        match syscribe_model::variability::effective_expr(e, &lpkg) {
            Some(expr) => expr.operands().iter().any(|o| o == feat),
            None => false,
        }
    };

    let type_filter_lc = type_filter.to_lowercase();
    let mut matches: Vec<&RawElement> = elements
        .iter()
        .filter(|e| {
            let label = tl(e.frontmatter.element_type.as_ref()).to_lowercase();
            label == type_filter_lc
        })
        .filter(|e| scope.is_empty() || e.qualified_name.starts_with(scope))
        // Multi-tag AND: all specified tags must be present (REQ-TRS-TAG-002).
        .filter(|e| {
            tags.is_empty() || {
                let elem_tags = e.frontmatter.tags.as_deref().unwrap_or(&[]);
                tags.iter().all(|t| elem_tags.iter().any(|x| x == t))
            }
        })
        .filter(|e| feature.is_none_or(|feat| gates_feature(e, feat)))
        // `--metadata <Def>`: keep only elements that apply that stereotype (MetadataDef),
        // matched by full ref or last `::` segment (REQ-TRS-META-001).
        .filter(|e| {
            metadata.is_none_or(|md| {
                syscribe_model::element::metadata_applications(&e.frontmatter.metadata)
                    .iter()
                    .any(|a| a.def == md || a.def.rsplit("::").next() == Some(md))
            })
        })
        // `--status <s>`: keep only elements whose `status:` equals s exactly.
        .filter(|e| status.is_none_or(|s| e.frontmatter.status.as_deref() == Some(s)))
        // `--sil <v>`: one flag covers SIL and ASIL — match when `silLevel`
        // (integer) stringifies to v OR `asilLevel` equals v.
        .filter(|e| {
            sil.is_none_or(|v| {
                e.frontmatter.sil_level.is_some_and(|n| n.to_string() == v)
                    || e.frontmatter.asil_level.as_deref() == Some(v)
            })
        })
        // `--has-wcet`: keep only elements that declare a non-empty `wcet:`.
        .filter(|e| !has_wcet || e.frontmatter.wcet.as_deref().is_some_and(|w| !w.trim().is_empty()))
        // `--where custom.<key>…` (GH #39): custom-field predicates, ANDed.
        .filter(|e| wheres.iter().all(|w| custom_field_matches(e, w)))
        .collect();

    matches.sort_by_key(|e| e.qualified_name.as_str());

    let is_testcase = type_filter_lc == "testcase";
    let is_aou = type_filter_lc == "assumptionofuse";

    // `--json`: emit a JSON array of the (filtered) elements. TestCase gets
    // extra fields a CI runner needs (REQ-TRS-OUT-014); other types get the
    // generic set.
    if json {
        let items: Vec<_> = matches
            .iter()
            .map(|e| {
                let mut obj = serde_json::json!({
                    "qualifiedName": e.qualified_name,
                    "type": tl(e.frontmatter.element_type.as_ref()),
                    "name": e.frontmatter.name,
                    "id": e.frontmatter.id,
                    "status": e.frontmatter.status,
                    "silLevel": e.frontmatter.sil_level,
                    "asilLevel": e.frontmatter.asil_level,
                    "wcet": e.frontmatter.wcet,
                });
                if is_testcase {
                    let verifies: Vec<&str> = e.frontmatter.verifies
                        .as_deref().unwrap_or(&[])
                        .iter().map(|v| v.as_str()).collect();
                    let tags_list: Vec<&str> = e.frontmatter.tags
                        .as_deref().unwrap_or(&[])
                        .iter().map(|s| s.as_str()).collect();
                    let tf = e.frontmatter.test_functions
                        .as_deref().unwrap_or(&[]).to_vec();
                    obj.as_object_mut().unwrap().extend([
                        ("testLevel".into(), serde_json::json!(e.frontmatter.test_level)),
                        ("verifies".into(), serde_json::json!(verifies)),
                        ("tags".into(), serde_json::json!(tags_list)),
                        ("sourceFile".into(), serde_json::json!(e.frontmatter.source_file)),
                        ("testFunctions".into(), serde_json::json!(tf)),
                    ]);
                } else if is_aou {
                    let applies_to: Vec<&str> = e.frontmatter.applies_to
                        .as_deref().unwrap_or(&[])
                        .iter().map(|s| s.as_str()).collect();
                    let body = if e.doc.trim().is_empty() {
                        serde_json::Value::Null
                    } else {
                        serde_json::json!(e.doc.trim())
                    };
                    obj.as_object_mut().unwrap().extend([
                        ("appliesTo".into(), serde_json::json!(applies_to)),
                        ("body".into(), body),
                    ]);
                }
                obj
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
        return;
    }

    if matches.is_empty() {
        let scope_note = if scope.is_empty() { String::new() } else { format!(" in `{scope}`") };
        println!("No `{type_filter}` elements found{scope_note}.");
        return;
    }

    let scope_note = if scope.is_empty() { String::new() } else { format!(" in `{scope}`") };
    println!("# {} elements{} ({})", type_filter, scope_note, matches.len());
    println!();

    if is_testcase {
        // TestCase-specific table: columns useful for test-execution planning
        // (REQ-TRS-OUT-014).
        println!("| ID | Name | Level | Status | Verifies | Tags |");
        println!("|---|---|---|---|---|---|");
        for e in &matches {
            let id = e.frontmatter.id.as_deref().unwrap_or("—");
            let name = e.frontmatter.name.as_deref().unwrap_or("—");
            let level = e.frontmatter.test_level.as_deref().unwrap_or("—");
            let status = e.frontmatter.status.as_deref().unwrap_or("—");
            let verifies = {
                let vs: Vec<&str> = e.frontmatter.verifies
                    .as_deref().unwrap_or(&[])
                    .iter().map(|v| v.as_str()).collect();
                if vs.is_empty() { "—".to_string() } else { vs.join(", ") }
            };
            let tags_col = {
                let ts: Vec<&str> = e.frontmatter.tags
                    .as_deref().unwrap_or(&[])
                    .iter().map(|s| s.as_str()).collect();
                if ts.is_empty() { "—".to_string() } else { ts.join(", ") }
            };
            println!("| {} | {} | {} | {} | {} | {} |",
                id, name, level, status, verifies, tags_col);
        }
    } else if is_aou {
        // AssumptionOfUse SRAC-oriented table (REQ-TRS-OUT-015).
        println!("| ID | Name | Applies To | Status |");
        println!("|---|---|---|---|");
        for e in &matches {
            let id = e.frontmatter.id.as_deref().unwrap_or("—");
            let name = e.frontmatter.name.as_deref().unwrap_or("—");
            let status = e.frontmatter.status.as_deref().unwrap_or("—");
            let applies = {
                let ts: Vec<&str> = e.frontmatter.applies_to
                    .as_deref().unwrap_or(&[])
                    .iter().map(|s| s.as_str()).collect();
                if ts.is_empty() { "—".to_string() } else { ts.join(", ") }
            };
            println!("| {} | {} | {} | {} |", id, name, applies, status);
        }
    } else {
        println!("| Qualified Name | Name / ID | Supertype / TypedBy | File |");
        println!("|---|---|---|---|");
        for e in &matches {
            let label = e.frontmatter.name
                .as_deref()
                .or_else(|| e.frontmatter.id.as_deref())
                .unwrap_or("—");
            let classifier = yaml_first_string(e.frontmatter.supertype.as_ref())
                .or_else(|| yaml_first_string(e.frontmatter.typed_by.as_ref()))
                .unwrap_or("—");
            println!("| {} | {} | {} | {} |", e.qualified_name, label, classifier, e.file_path);
        }
    }
    println!();
}

pub fn cmd_find(elements: &[RawElement], pattern: &str, wheres: &[CustomWhere]) {
    let mut scored: Vec<(u32, &RawElement)> = elements
        .iter()
        .map(|e| (fuzzy_score(e, pattern), e))
        .filter(|(s, _)| *s > 0)
        // `--where` custom-field predicates (GH #39) — ANDed with the fuzzy match.
        .filter(|(_, e)| wheres.iter().all(|w| custom_field_matches(e, w)))
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
        // Prefer name or id as excerpt, fall back to doc
        let excerpt = elem
            .frontmatter
            .name
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

/// Look up elements by an exact external reference (`extRef`). Prints every
/// element whose `extRef` contains `reference` exactly. Returns `true` when at
/// least one element matched (so the caller can exit non-zero on a miss).
pub fn cmd_extref(elements: &[RawElement], reference: &str, json: bool) -> bool {
    let matches: Vec<&RawElement> = elements
        .iter()
        .filter(|e| {
            e.frontmatter
                .ext_ref
                .as_ref()
                .is_some_and(|refs| refs.iter().any(|r| r == reference))
        })
        .collect();

    if json {
        let items: Vec<_> = matches
            .iter()
            .map(|e| {
                serde_json::json!({
                    "qualifiedName": e.qualified_name,
                    "type": tl(e.frontmatter.element_type.as_ref()),
                    "id": e.frontmatter.id,
                    "extRef": e.frontmatter.ext_ref,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
        return !matches.is_empty();
    }

    if matches.is_empty() {
        println!("No element declares extRef `{reference}`.");
        return false;
    }

    println!("# extRef: `{}`", reference);
    println!();
    println!("| Qualified Name | Type | id |");
    println!("|---|---|---|");
    for e in &matches {
        println!(
            "| {} | {} | {} |",
            e.qualified_name,
            tl(e.frontmatter.element_type.as_ref()),
            e.frontmatter.id.as_deref().unwrap_or("")
        );
    }
    println!();
    println!("{} match(es)", matches.len());
    true
}

// ── cmd: trace ───────────────────────────────────────────────────────────────

pub fn cmd_trace(
    elements: &[RawElement],
    resolver: &Resolver,
    val: &ValidationResult,
    key: &str,
    results: Option<&ResultsData>,
    linked_only: bool,
) {
    // Executed-evidence annotations apply only when a results sidecar is loaded
    // and the caller did not force the linked-only view (issue #21).
    let evidence = if linked_only { None } else { results };
    let Some(elem) = resolve(elements, resolver, key) else {
        eprintln!("Element not found: {key}");
        return;
    };
    let fm = &elem.frontmatter;

    let id = fm.id.as_deref().unwrap_or(&elem.qualified_name);
    let title = fm.name.as_deref().unwrap_or("(no name)");
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
                let p_title = p.frontmatter.name.as_deref().unwrap_or("—");
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
            let adr_title = adr.frontmatter.name.as_deref().unwrap_or("—");
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
            let sg_title = sg.frontmatter.name.as_deref().unwrap_or("—");
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
    if let Some(ref csg_ref) = fm.derived_from_cybersecurity_goal {
        println!("## Security Goal (`derivedFromCybersecurityGoal`)");
        println!();
        if let Some(csg) = resolve(elements, resolver, csg_ref) {
            let csg_id = csg.frontmatter.id.as_deref().unwrap_or(&csg.qualified_name);
            let csg_title = csg.frontmatter.name.as_deref().unwrap_or("—");
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
                let c_title = c.frontmatter.name.as_deref().unwrap_or("—");
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
                // Annotate the displayed id with the ingested verdict, when results
                // are present and not linked-only (issue #21).
                let annotated = match evidence {
                    Some(_) => match tc_verdict(tc, evidence) {
                        TcVerdict::Pass => format!("{tc_id} [pass]"),
                        TcVerdict::Fail => format!("{tc_id} [fail]"),
                        TcVerdict::Unknown => format!("{tc_id} [unknown]"),
                    },
                    None => tc_id.to_string(),
                };
                println!("| {} | {} | {} | {} |", annotated, name, level, scenarios);
            } else {
                println!("| {} | (not found) | — | — |", tc_id);
            }
        }
        println!();
    }

    // ── Refined by (MagicGrid «refine», REQ-TRS-MG-001) ───────────────────
    let refiners = val.refined_by.get(id).cloned().unwrap_or_default();
    if refiners.is_empty() {
        println!("## Refined by");
        println!();
        println!("_(none)_");
        println!();
    } else {
        println!("## Refined by");
        println!();
        println!("| Use Case | Type | Status |");
        println!("|---|---|---|");
        let mut sorted_r = refiners.clone();
        sorted_r.sort();
        for uc_ref in &sorted_r {
            if let Some(uc) = resolve(elements, resolver, uc_ref) {
                let ty = tl(uc.frontmatter.element_type.as_ref());
                let st = uc.frontmatter.status.as_deref().unwrap_or("—");
                println!("| {} | {} | {} |", uc_ref, ty, st);
            } else {
                println!("| {} | (not found) | — |", uc_ref);
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
            let title = req.frontmatter.name.as_deref().unwrap_or("—");
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
    let title = elem.frontmatter.name.as_deref().unwrap_or("(no name)");

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
        // The key is not an element qname/id. It may be a raw reference target —
        // e.g. an implementedBy source path or sourceFile. Report the model
        // elements that point at it. Paths are matched by exact value and by
        // path-prefix (a directory key matches files beneath it). Location
        // prefixes/schemes (`repo:`, `model:`, `file://`, `scheme://`) are
        // stripped on both sides so `refs firmware/x` finds `repo:firmware/x`.
        let bare = |s: &str| -> String {
            let s = s.trim();
            let s = if let Some(end) = s.find("://") {
                // file://path or scheme://host/path — keep the part after "://"
                &s[end + 3..]
            } else if let Some(rest) = s.strip_prefix("repo:") {
                rest
            } else if let Some(rest) = s.strip_prefix("model:") {
                rest
            } else {
                s
            };
            s.trim_start_matches('/').trim_end_matches('/').to_string()
        };
        let needle = bare(key);
        let mut rows: Vec<(String, String, String)> = Vec::new();
        for other in elements {
            for (rel, tgt) in outbound_refs(other) {
                let t = bare(&tgt);
                if t == needle || t.starts_with(&format!("{needle}/")) {
                    let stype = tl(other.frontmatter.element_type.as_ref()).to_string();
                    rows.push((other.qualified_name.clone(), rel, stype));
                }
            }
        }
        rows.sort();
        rows.dedup();
        println!("# References to: {key}");
        println!();
        if rows.is_empty() {
            println!("No elements reference `{key}`.");
            return;
        }
        println!("| Source | Relationship | Type |");
        println!("|---|---|---|");
        for (src, rel, stype) in &rows {
            println!("| {} | {} | {} |", src, rel, stype);
        }
        println!();
        println!("{} reference(s)", rows.len());
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

    // Computed inbound for a Configuration target: the TestCases that run in it,
    // i.e. whose appliesWhen is satisfied by this configuration's selections
    // (configuration-agnostic TestCases — no appliesWhen — run in every config).
    let mut runs_in: Vec<String> = Vec::new();
    if elem.frontmatter.element_type.as_ref() == Some(&ElementType::Configuration) {
        let feat_alias = syscribe_model::variability::feature_id_to_qname(elements);
        let sel = syscribe_model::variability::canon_selection(
            &elem.frontmatter.feature_selections(),
            &feat_alias,
        );
        let selected = |q: &str| sel.get(q).copied().unwrap_or(false);
        for other in elements {
            if other.frontmatter.element_type.as_ref() != Some(&ElementType::TestCase) {
                continue;
            }
            let runs = match other
                .frontmatter
                .applies_when
                .as_ref()
                .and_then(|aw| syscribe_model::variability::applies_when_expr(aw).ok().flatten())
                .map(|e| {
                    e.canonicalize(&|q: &str| {
                        syscribe_model::variability::canon_feature_ref(q, &feat_alias)
                    })
                }) {
                None => true,
                Some(expr) => expr.eval(&selected),
            };
            if runs {
                runs_in.push(
                    other
                        .frontmatter
                        .id
                        .clone()
                        .unwrap_or_else(|| other.qualified_name.clone()),
                );
            }
        }
        runs_in.sort();
    }

    if rows.is_empty() && runs_in.is_empty() {
        println!("No elements reference `{}`.", target_qn);
        return;
    }

    if !rows.is_empty() {
        println!("| Source | Relationship | Type |");
        println!("|---|---|---|");
        for (src, rel, stype) in &rows {
            println!("| {} | {} | {} |", src, rel, stype);
        }
        println!();
        println!("{} reference(s)", rows.len());
        println!();
    }

    if !runs_in.is_empty() {
        println!("## TestCases running in this configuration");
        println!();
        for id in &runs_in {
            println!("- {}", id);
        }
        println!();
    }
}

// ── help ─────────────────────────────────────────────────────────────────────

/// CI severity-gating options for `validate` (issue #3).
///
/// Controls which warning conditions cause a non-zero exit, on top of the
/// always-fatal `Error` findings.
#[derive(Debug, Clone, Default)]
pub struct GateOptions {
    /// Warning codes to treat as gate failures (e.g. `W004`, `W009`).
    pub deny: std::collections::HashSet<String>,
    /// Fail if the number of warnings exceeds this threshold.
    pub max_warnings: Option<usize>,
    /// Treat every warning as a gate failure.
    pub warnings_as_errors: bool,
}

impl GateOptions {
    /// Returns the findings that trip the gate. Warnings are gated by `--deny` (or
    /// all of them under `--warnings-as-errors`); informational findings are gated
    /// only when their code is explicitly listed in `--deny`.
    fn denied<'a>(
        &self,
        warnings: &[&'a syscribe_model::validator::Finding],
        infos: &[&'a syscribe_model::validator::Finding],
    ) -> Vec<&'a syscribe_model::validator::Finding> {
        let mut out: Vec<&syscribe_model::validator::Finding> = if self.warnings_as_errors {
            warnings.to_vec()
        } else {
            warnings.iter().filter(|f| self.deny.contains(f.code)).copied().collect()
        };
        out.extend(infos.iter().filter(|f| self.deny.contains(f.code)).copied());
        out
    }
}

/// Whether `elem` matches all scope fields declared on `profile` (REQ-TRS-OUT-012).
/// `sil` uses the same matching as `list --sil` (silLevel stringified OR asilLevel);
/// `status` is exact; `tag` is membership in `tags:`.
fn profile_scope_matches(profile: &syscribe_model::config::Profile, elem: &RawElement) -> bool {
    if let Some(sil) = &profile.sil {
        let hit = elem.frontmatter.sil_level.is_some_and(|n| n.to_string() == *sil)
            || elem.frontmatter.asil_level.as_deref() == Some(sil.as_str());
        if !hit {
            return false;
        }
    }
    if let Some(status) = &profile.status {
        if elem.frontmatter.status.as_deref() != Some(status.as_str()) {
            return false;
        }
    }
    if let Some(tag) = &profile.tag {
        let hit = elem
            .frontmatter
            .tags
            .as_ref()
            .is_some_and(|ts| ts.iter().any(|t| t == tag));
        if !hit {
            return false;
        }
    }
    true
}

/// Findings promoted to gate failures by a named `profile` (issue #18).
///
/// A finding is promoted when its code is in `profile.promote` AND (the profile
/// is unscoped, OR the element whose `file_path == finding.file` matches all the
/// profile's scope fields). A finding whose file maps to no element is not
/// promoted when any scope field is set.
pub fn profile_promoted<'a>(
    profile: &syscribe_model::config::Profile,
    elements: &[RawElement],
    findings: &[&'a syscribe_model::validator::Finding],
) -> Vec<&'a syscribe_model::validator::Finding> {
    let promote: std::collections::HashSet<&str> =
        profile.promote.iter().map(|s| s.as_str()).collect();
    findings
        .iter()
        .filter(|f| promote.contains(f.code))
        .filter(|f| {
            if profile.is_unscoped() {
                return true;
            }
            elements
                .iter()
                .find(|e| e.file_path == f.file)
                .is_some_and(|e| profile_scope_matches(profile, e))
        })
        .copied()
        .collect()
}

/// `feature-check`: holistic feature-model validation (§9), separate from the
/// per-element `validate` pass. With `--deep`, additionally runs the solver-backed
/// analyses (void/dead/core/false-optional/configuration-validity). Exit `0` when
/// there are no error-severity findings, `1` otherwise. Dormant (exit 0 with a
/// notice) when no FeatureDef.
pub fn cmd_feature_check(
    elements: &[RawElement],
    json: bool,
    deep: bool,
    count: bool,
    enumerate: bool,
    prove: Option<&str>,
    gate: &GateOptions,
) {
    use syscribe_model::feature_model;
    use syscribe_model::feature_model::EnumOutcome;
    use syscribe_model::validator::Severity;

    if !feature_model::has_feature_model(elements) {
        if json {
            println!("[]");
        } else {
            println!("No feature model present — nothing to check.");
        }
        return;
    }

    let mut findings = feature_model::check_feature_model(elements);
    // Parameter-binding validation (E203–E206/E222/W017) is shared with `validate`
    // so a product line checked holistically also gets range/binding enforcement (GH #14).
    findings.extend(syscribe_model::validator::parameter_binding_findings(elements));
    let deep_rep = if deep {
        Some(feature_model::check_feature_model_deep(elements))
    } else {
        None
    };
    let variants = if count || enumerate {
        Some(feature_model::enumerate_variants(elements, feature_model::MAX_ENUM))
    } else {
        None
    };
    // --prove: emit DIMACS CNF of each UNSAT finding (externally re-checkable).
    let proofs: Option<Vec<String>> = prove.map(|dir| {
        feature_model::write_proofs(elements, std::path::Path::new(dir)).unwrap_or_default()
    });
    if let Some(r) = &deep_rep {
        findings.extend(r.findings.iter().cloned());
    }
    let has_error = findings.iter().any(|f| f.severity == Severity::Error);

    // Gate evaluation (REQ-TRS-DISC-006): warnings (e.g. W024) gateable via --deny.
    let warn_refs: Vec<&syscribe_model::validator::Finding> = findings.iter().filter(|f| f.severity == Severity::Warning).collect();
    let info_refs: Vec<&syscribe_model::validator::Finding> = findings.iter().filter(|f| f.severity == Severity::Info).collect();
    let denied = gate.denied(&warn_refs, &info_refs);
    let over_max = gate.max_warnings.map_or(false, |m| warn_refs.len() > m);
    let gate_tripped = !denied.is_empty() || over_max;

    let sev = |s: &Severity| match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    };

    if json {
        let items: Vec<serde_json::Value> = findings
            .iter()
            .map(|f| {
                serde_json::json!({
                    "code": f.code,
                    "severity": sev(&f.severity),
                    "file": f.file,
                    "message": f.message,
                })
            })
            .collect();
        let mut doc = serde_json::Map::new();
        doc.insert("schemaVersion".into(), serde_json::json!("1.0"));
        doc.insert("findings".into(), serde_json::json!(items));
        if let Some(r) = &deep_rep {
            doc.insert("void".into(), serde_json::json!(r.void));
            doc.insert("deadFeatures".into(), serde_json::json!(r.dead));
            doc.insert("coreFeatures".into(), serde_json::json!(r.core));
            doc.insert("falseOptionalFeatures".into(), serde_json::json!(r.false_optional));
            doc.insert("invalidConfigurations".into(), serde_json::json!(r.invalid_configs));
            doc.insert("diagnoses".into(), serde_json::json!(r.diagnoses));
            if let Some(reason) = &r.skipped {
                doc.insert("deepSkipped".into(), serde_json::json!(reason));
            }
        }
        if let Some(v) = &variants {
            match v {
                EnumOutcome::Variants { configs, truncated } => {
                    if *truncated {
                        doc.insert("variantCount".into(), serde_json::json!({ "atLeast": configs.len() }));
                    } else {
                        doc.insert("variantCount".into(), serde_json::json!(configs.len()));
                    }
                    if enumerate {
                        doc.insert("variants".into(), serde_json::json!(configs));
                    }
                }
                EnumOutcome::Skipped(reason) => {
                    doc.insert("variantSkipped".into(), serde_json::json!(reason));
                }
                EnumOutcome::Dormant => {}
            }
        }
        if let Some(p) = &proofs {
            doc.insert("proofs".into(), serde_json::json!(p));
        }
        println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(doc)).unwrap());
    } else {
        if findings.is_empty() {
            println!("Feature model OK — 0 findings.");
        } else {
            println!("# Feature Model Check");
            println!();
            println!("| Code | File | Message |");
            println!("|---|---|---|");
            for f in &findings {
                println!("| {} | {} | {} |", f.code, f.file, f.message);
            }
            println!();
            let errs = findings.iter().filter(|f| f.severity == Severity::Error).count();
            let warns = findings.iter().filter(|f| f.severity == Severity::Warning).count();
            println!("{} error(s), {} warning(s)", errs, warns);
        }
        if let Some(r) = &deep_rep {
            println!();
            println!("## Deep analysis");
            if let Some(reason) = &r.skipped {
                println!("{}", reason);
            } else {
                println!("- void model: {}", r.void);
                println!("- dead features: {}", if r.dead.is_empty() { "none".into() } else { r.dead.join(", ") });
                println!("- core features: {}", if r.core.is_empty() { "none".into() } else { r.core.join(", ") });
                println!("- false-optional: {}", if r.false_optional.is_empty() { "none".into() } else { r.false_optional.join(", ") });
                println!("- invalid configurations: {}", if r.invalid_configs.is_empty() { "none".into() } else { r.invalid_configs.join(", ") });
                if !r.diagnoses.is_empty() {
                    let opts: Vec<String> = r.diagnoses.iter().map(|m| format!("relax {{{}}}", m.join(", "))).collect();
                    println!("- diagnoses (fixes): {}", opts.join(" | "));
                }
                println!("(deep analysis covers the Boolean feature layer only; parameter satisfiability is not checked)");
            }
        }
        if let Some(v) = &variants {
            println!();
            match v {
                EnumOutcome::Variants { configs, truncated } => {
                    if *truncated {
                        println!("Valid configurations: ≥ {} (truncated)", configs.len());
                    } else {
                        println!("Valid configurations: {}", configs.len());
                    }
                    if enumerate {
                        for (i, c) in configs.iter().enumerate() {
                            println!("  {}. {}", i + 1, if c.is_empty() { "(none)".into() } else { c.join(", ") });
                        }
                    }
                }
                EnumOutcome::Skipped(reason) => println!("{}", reason),
                EnumOutcome::Dormant => {}
            }
        }
        if let Some(p) = &proofs {
            println!();
            if p.is_empty() {
                println!("Proofs: none written (no UNSAT findings, or dormant/over-limit).");
            } else {
                println!("Proofs (DIMACS CNF, externally re-checkable as UNSAT): {}", p.join(", "));
            }
        }
    }

    if has_error {
        std::process::exit(1);
    }
    if gate_tripped {
        if !json {
            for line in gate_report_lines(&denied, over_max, warn_refs.len(), gate) {
                println!("{}", line);
            }
        }
        std::process::exit(2);
    }
}

/// `configure <Configuration>`: assisted configuration (REQ-TRS-FMA-008). Treats
/// the configuration's `features:` as a partial selection and reports
/// satisfiability + forced/free features. Exit `1` if the partial selection is
/// contradictory; dormant (exit 0) with no feature model.
pub fn cmd_configure(elements: &[RawElement], conf: &str, json: bool) {
    use syscribe_model::feature_model::{configure, ConfigureOutcome};
    match configure(elements, conf) {
        ConfigureOutcome::Dormant => {
            if json {
                println!("[]");
            } else {
                println!("No feature model present — nothing to configure.");
            }
        }
        ConfigureOutcome::NotFound => {
            eprintln!("Configuration not found: {conf}");
            std::process::exit(1);
        }
        ConfigureOutcome::Report { satisfiable, forced_true, forced_false, free, explanation } => {
            if json {
                let doc = serde_json::json!({
                    "satisfiable": satisfiable,
                    "forcedTrue": forced_true,
                    "forcedFalse": forced_false,
                    "free": free,
                    "explanation": explanation,
                });
                println!("{}", serde_json::to_string_pretty(&doc).unwrap());
            } else {
                println!("# Configure: {}", conf);
                println!();
                println!("- satisfiable: {}", satisfiable);
                if let Some(e) = &explanation {
                    println!("- conflict: {}", e);
                }
                if satisfiable {
                    let or_none = |v: &[String]| if v.is_empty() { "none".to_string() } else { v.join(", ") };
                    println!("- forced (true): {}", or_none(&forced_true));
                    println!("- forced (false): {}", or_none(&forced_false));
                    println!("- free: {}", or_none(&free));
                }
            }
            if !satisfiable {
                std::process::exit(1);
            }
        }
    }
}

/// Exit-code contract for `validate` (issue #3):
/// `0` clean · `1` Error-severity findings · `2` warnings tripped a gate.
pub fn cmd_validate(
    elements: &[RawElement],
    config: &syscribe_model::config::ValidateConfig,
    gate: &GateOptions,
    profile: Option<&syscribe_model::config::Profile>,
    file_filter: Option<&str>,
    json: bool,
) {
    use syscribe_model::validator;
    use syscribe_model::validator::Severity;

    let result = validator::validate_with_config(elements, config);

    let findings: Vec<_> = result.findings.iter()
        .filter(|f| file_filter.map_or(true, |ff| f.file.contains(ff)))
        .collect();

    let errors: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Error).copied().collect();
    let warnings: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Warning).copied().collect();
    let infos: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Info).copied().collect();

    // Gate evaluation (independent of output format). A selected `--profile`
    // contributes its promoted findings additively with `--deny`/etc.
    let mut denied = gate.denied(&warnings, &infos);
    if let Some(p) = profile {
        let mut candidates = warnings.clone();
        candidates.extend(infos.iter().copied());
        for f in profile_promoted(p, elements, &candidates) {
            if !denied.iter().any(|d| std::ptr::eq(*d, f)) {
                denied.push(f);
            }
        }
    }
    let over_max = gate.max_warnings.map_or(false, |m| warnings.len() > m);
    let gate_tripped = !denied.is_empty() || over_max;

    // Exit code: errors dominate (1), then gated warnings (2), else clean (0).
    let exit_code = if !errors.is_empty() {
        1
    } else if gate_tripped {
        2
    } else {
        0
    };

    if json {
        let items: Vec<serde_json::Value> = findings.iter().map(|f| {
            serde_json::json!({
                "code": f.code,
                "severity": match f.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Info => "info",
                },
                "file": f.file,
                "message": f.message,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
        if exit_code == 2 {
            for line in gate_report_lines(&denied, over_max, warnings.len(), gate) {
                eprintln!("{}", line);
            }
        }
        if exit_code != 0 {
            std::process::exit(exit_code);
        }
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

    if !infos.is_empty() {
        println!("Informational ({}):", infos.len());
        println!();
        println!("| Code | File | Message |");
        println!("|---|---|---|");
        for f in &infos {
            println!("| {} | {} | {} |", f.code, f.file, f.message);
        }
        println!();
    }

    if exit_code == 2 {
        for line in gate_report_lines(&denied, over_max, warnings.len(), gate) {
            println!("{}", line);
        }
    }

    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}

/// Stable display id for an element: `id:` when present, else qualified name.
fn cfg_id(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// Print a finding list (text or json) under the gate, returning the exit code
/// (0 clean · 1 errors · 2 gated warnings) WITHOUT exiting. Shared by the
/// configuration-lens validators.
fn print_findings_report(
    findings: &[syscribe_model::validator::Finding],
    gate: &GateOptions,
    json: bool,
) -> i32 {
    use syscribe_model::validator::Severity;
    let errors: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Error).collect();
    let warnings: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Warning).collect();
    let infos: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Info).collect();
    let denied = gate.denied(&warnings, &infos);
    let over_max = gate.max_warnings.map_or(false, |m| warnings.len() > m);
    let exit_code = if !errors.is_empty() {
        1
    } else if !denied.is_empty() || over_max {
        2
    } else {
        0
    };
    if json {
        let items: Vec<serde_json::Value> = findings
            .iter()
            .map(|f| {
                serde_json::json!({
                    "code": f.code,
                    "severity": match f.severity {
                        Severity::Error => "error",
                        Severity::Warning => "warning",
                        Severity::Info => "info",
                    },
                    "file": f.file,
                    "message": f.message,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
        return exit_code;
    }
    if findings.is_empty() {
        println!("0 errors, 0 warnings — variant is valid.");
        return exit_code;
    }
    let table = |label: &str, fs: &[&syscribe_model::validator::Finding]| {
        if fs.is_empty() {
            return;
        }
        println!("{} ({}):", label, fs.len());
        println!();
        println!("| Code | File | Message |");
        println!("|---|---|---|");
        for f in fs {
            println!("| {} | {} | {} |", f.code, f.file, f.message);
        }
        println!();
    };
    table("Errors", &errors);
    table("Warnings", &warnings);
    table("Informational", &infos);
    exit_code
}

/// `validate --config <C>`: full re-validation in the configuration lens.
pub fn cmd_validate_projected(
    elements: &[RawElement],
    config: &syscribe_model::config::ValidateConfig,
    gate: &GateOptions,
    json: bool,
    sel: &syscribe_model::projection::Selection,
) {
    let findings = syscribe_model::projection::validate_projected(elements, config, sel);
    let code = print_findings_report(&findings, gate, json);
    if code != 0 {
        std::process::exit(code);
    }
}

/// `validate --all-configs`: run the lens validation for every stored
/// Configuration and summarise per-variant; exit non-zero if any has errors.
pub fn cmd_validate_all_configs(
    elements: &[RawElement],
    config: &syscribe_model::config::ValidateConfig,
    json: bool,
) {
    use syscribe_model::validator::Severity;
    let mut configs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| e.frontmatter.element_type.as_ref() == Some(&ElementType::Configuration))
        .collect();
    configs.sort_by(|a, b| cfg_id(a).cmp(&cfg_id(b)));

    let mut any_error = false;
    let mut rows: Vec<(String, usize, usize)> = Vec::new();
    for cfg in &configs {
        let sel = cfg.frontmatter.feature_selections();
        let findings = syscribe_model::projection::validate_projected(elements, config, &sel);
        let errs = findings.iter().filter(|f| f.severity == Severity::Error).count();
        let warns = findings.iter().filter(|f| f.severity == Severity::Warning).count();
        if errs > 0 {
            any_error = true;
        }
        rows.push((cfg_id(cfg), errs, warns));
    }

    if json {
        let items: Vec<serde_json::Value> = rows
            .iter()
            .map(|(id, e, w)| serde_json::json!({ "configuration": id, "errors": e, "warnings": w }))
            .collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
    } else if rows.is_empty() {
        println!("No configurations to validate.");
    } else {
        println!("# Validate all configurations");
        println!();
        println!("| Configuration | Errors | Warnings |");
        println!("|---|---|---|");
        for (id, e, w) in &rows {
            println!("| {} | {} | {} |", id, e, w);
        }
    }
    if any_error {
        std::process::exit(1);
    }
}

/// `diff --config A --config B`: elements active in one variant but not the other.
pub fn cmd_diff(elements: &[RawElement], a: &str, b: &str, json: bool) {
    use std::collections::BTreeSet;
    use syscribe_model::projection::{project, resolve_selection, SelectionOutcome};

    let sel_of = |arg: &str| -> syscribe_model::projection::Selection {
        match resolve_selection(elements, arg) {
            SelectionOutcome::Resolved(s) => s,
            SelectionOutcome::Dormant => {
                eprintln!("No feature model present — nothing to diff.");
                std::process::exit(0);
            }
            SelectionOutcome::Error(m) => {
                eprintln!("{m}");
                std::process::exit(1);
            }
        }
    };
    let id_or_qn =
        |e: &RawElement| e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone());
    let set = |arg: &str| -> BTreeSet<String> {
        project(elements, &sel_of(arg)).iter().map(&id_or_qn).collect()
    };
    let sa = set(a);
    let sb = set(b);
    let only_a: Vec<&String> = sa.difference(&sb).collect();
    let only_b: Vec<&String> = sb.difference(&sa).collect();

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "onlyInA": only_a,
                "onlyInB": only_b,
            }))
            .unwrap()
        );
    } else {
        println!("# Variant diff: {} vs {}", a, b);
        println!();
        println!("Only in {} ({}):", a, only_a.len());
        for x in &only_a {
            println!("  + {}", x);
        }
        println!();
        println!("Only in {} ({}):", b, only_b.len());
        for x in &only_b {
            println!("  - {}", x);
        }
    }
}

/// Build human-readable lines explaining why the warning gate tripped.
fn gate_report_lines(
    denied: &[&syscribe_model::validator::Finding],
    over_max: bool,
    warn_total: usize,
    gate: &GateOptions,
) -> Vec<String> {
    let mut lines = Vec::new();
    if !denied.is_empty() {
        let label = if gate.warnings_as_errors {
            "all warnings promoted to errors (--warnings-as-errors)".to_string()
        } else {
            format!("denied warning codes ({})", sorted_codes(denied).join(", "))
        };
        lines.push(format!("Gate failure (exit 2): {} — {} finding(s).", label, denied.len()));
    }
    if over_max {
        lines.push(format!(
            "Gate failure (exit 2): warning count {} exceeds --max-warnings {}.",
            warn_total,
            gate.max_warnings.unwrap()
        ));
    }
    lines
}

fn sorted_codes(findings: &[&syscribe_model::validator::Finding]) -> Vec<String> {
    let mut codes: Vec<String> = findings.iter().map(|f| f.code.to_string()).collect();
    codes.sort();
    codes.dedup();
    codes
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
name: "The system shall ..."
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
# decompositionKind: independent   # ASIL D / SIL 4 decomposition: independent | redundant | diverse
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
name: "Verify that ..."
status: draft
testLevel: L5
# securityTestMethod: fuzz   # optional (ISO/SAE 21434 §13.3): fuzz | penetration_test |
#                            #   security_regression | vulnerability_scan | threat_modeling (W809 if other)
coverageTarget: statement
verifies:
  - REQ-PREFIX-001
# Each testFunctions[].scenario MUST match a `Scenario:` title below 1:1 (E106).
# sourceFile + function enable function-level traceability (W009).
# testFunctions:
#   - function: "mymod::tests::normal_case"   # last segment resolved in sourceFile
#     scenario: "Normal case"                 # must equal a Scenario: title below
# sourceFile: tests/my_tests.rs
---

## Test Procedure

```gherkin
Feature: ...

  Scenario: Normal case
    Given ...
    When ...
    Then ...
```

> Tip: run `syscribe scaffold-gherkin TC-PREFIX-001 --fix` to auto-insert any
> `Scenario:` blocks missing for the `testFunctions[].scenario` entries above.
"#,
        "testplan" => r#"---
type: TestPlan
id: TP-PREFIX-001
name: "..."
status: draft
scope: integration   # unit|smoke|integration|hil|certification|security|regression
# configurations: [CONF-PREFIX-001]   # the product variant(s); omit = config-agnostic
# demonstrates:                        # optional: goals/requirements this plan evidences
#   - SG-PREFIX-001
testCases:                             # explicit members (TC-*)
  - TC-PREFIX-001
# selection:                           # optional additive query, unioned with testCases
#   testLevels: [L3, L4]
#   domains: [software]
#   tags: [integration]
---

Objectives, entry/exit criteria, test environment and responsibilities.

Inspect with `syscribe -m model/ testplan TP-PREFIX-001`.
"#,
        "adr" => r#"---
type: ADR
id: ADR-PREFIX-001
name: "Decision title"
status: proposed
---

## Context

What is the issue that motivates this decision?

## Decision

What was decided.

## Consequences

What are the results of this decision?
"#,
        "zone" => r#"---
type: Zone
id: ZN-PREFIX-001
name: "Control Zone"
status: approved          # draft | review | approved | deprecated
targetSL: 3               # required Security Level 1-4 (IEC 62443-3-3)
# achievedSL: 2           # assessed SL (W950 if below targetSL)
members:                  # PartDef/Part qnames or stable IDs in this zone
  - Logical::PLCController
# rationale: "SL 3 due to potential for significant process disruption."
---

What this security zone groups and why this Security Level.
"#,
        "conduit" => r#"---
type: Conduit
id: CD-PREFIX-001
name: "ControlToField"
status: approved          # draft | review | approved | deprecated
fromZone: ZN-PREFIX-001
toZone: ZN-PREFIX-002
# achievedSL: 3           # SL of the conduit's boundary controls (W951 if below either zone)
# protocols: [Modbus/TCP, OPC-UA]
# implementedBy: [SC-FIREWALL-001]   # SecurityControl ids / architecture qnames
---

The communication channel between the two zones and its boundary controls.
"#,
        "reviewrecord" => r#"---
type: ReviewRecord
id: RR-PREFIX-001
name: "Software Architecture Review — Sprint N"
status: closed          # open | closed | waived
reviewType: design_review   # design_review | requirements_review | hazard_review |
                            #   test_readiness_review | inspection | walk_through
reviewDate: "2026-01-01"
# reviewedBy: [alice, bob]
# recordedAt: "https://github.com/<org>/<repo>/pull/<n>"   # thin pointer to the review
reviews:                # ≥1 element qnames / stable IDs covered by this review
  - REQ-PREFIX-001
# items:
#   - id: RID-001
#     description: "Action item description."
#     disposition: closed   # open | closed | not_applicable
#     closedBy: REQ-PREFIX-002
---

Summary of the review and its outcome (discussion lives in the linked `recordedAt` review).
"#,
        "tradestudy" => r#"---
type: TradeStudy
id: TRD-PREFIX-001
name: "..."
status: draft   # draft | review | complete
# objective: REQ-PREFIX-001     # the Requirement this study informs
# decision: ADR-PREFIX-001      # the ADR recording the selected alternative
criteria:
  - name: latency
    weight: 0.5          # relative weight in [0,1]; need not sum to 1 (tool normalises)
    direction: minimize  # maximize | minimize
    unit: ms
  - name: cost
    weight: 0.5
    direction: minimize
    unit: USD
alternatives:
  - name: OptionA
    # element: Physical::OptionA   # optional modeled element
  - name: OptionB
scores:
  - { alternative: OptionA, criterion: latency, score: 10 }
  - { alternative: OptionA, criterion: cost,    score: 5 }
  - { alternative: OptionB, criterion: latency, score: 20 }
  - { alternative: OptionB, criterion: cost,    score: 3 }
---

What this trade study evaluates and why.
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
# implementedBy: include/my_interface.h   # path to header/IDL/source that defines this contract
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
id: FEAT-MYFEATURE-001
name: MyFeatureDef
groupKind: optional
# cardinality: "1"
# parentFeature: Features::ParentFeatureDef
# excludes:
#   - Features::ConflictingFeatureDef
# parameters:
#   - name: myParam
#     type: integer
#     range: "1..100"
#     default: 10
#     buildVar: MY_PARAM   # emitted by build-config when feature is selected
# buildExports:            # optional: build variables driven by this feature
#   - var: ENABLE_MYFEATURE
#     whenSelected: 1      # value when selected (default: 1)
#     whenDeselected: 0    # omit this line to suppress the var when deselected
---

Description of this feature definition.
"#,
        "itemdef" => r#"---
type: ItemDef
name: MyItemDef
# supertype: Items::SomeItemDef
# features:
#   - name: mass
#     typedBy: ScalarValues::Real
---

Description of this item definition.
"#,
        "item" => r#"---
type: Item
name: myItem
typedBy: Items::MyItemDef
# multiplicity: "1"
---

Description of this item usage.
"#,
        "interface" => r#"---
type: Interface
name: myInterface
typedBy: Interfaces::MyInterfaceDef
# implementedBy: src/my_interface.c   # path to source artifact implementing this interface
---

Description of this interface usage.
"#,
        "action" => r#"---
type: Action
name: myAction
typedBy: Actions::MyActionDef
# subActions:
#   - name: step1
#     typedBy: Actions::Step1
---

Description of this action usage.
"#,
        "attributedef" => r#"---
type: AttributeDef
name: MyAttributeDef
# supertype: ScalarValues::Real
---

Description of this attribute definition.
"#,
        "configuration" => r#"---
type: Configuration
id: CONF-PREFIX-001
name: "My variant configuration"
status: draft
featureModel: Features::MyFeatureDef
# features: a map of <FeatureDef qualified name>: true/false (§9.8).
# This is the only selection syntax honored by `matrix` and appliesWhen eval.
features:
  Features::FeatureA: true
  Features::FeatureB: false
# parameterBindings:          # bind FeatureDef parameter values for this variant
#   Features::MyFeature.myParam: 42
# buildOverrides:             # optional: config-level build variable overrides
#   PRODUCT_VARIANT: "premium"  # applied last by build-config; wins on collision
#   VERSION_STRING: "2.0.0"
---

Description of this configuration.
"#,
        "verificationcasedef" => r#"---
type: VerificationCaseDef
name: MyVerificationCaseDef
# subject: SomePackage::SomeElement
# returnType: ScalarValues::Boolean
---

Description of this verification case definition.
"#,
        "verificationcase" => r#"---
type: VerificationCase
name: myVerificationCase
typedBy: Verification::MyVerificationCaseDef
# verifies:
#   - REQ-PREFIX-001
---

Description of this verification case.
"#,
        "analysiscasedef" => r#"---
type: AnalysisCaseDef
name: MyAnalysisCaseDef
# subject: SomePackage::SomeElement
# returnType: ScalarValues::Real
---

Description of this analysis case definition.
"#,
        "analysiscase" => r#"---
type: AnalysisCase
name: myAnalysisCase
typedBy: Analysis::MyAnalysisCaseDef
---

Description of this analysis case usage.
"#,
        "diagram" => r#"---
type: Diagram
name: MyDiagram
diagramKind: IBD
subject: SomePackage::SomeElement
# svgMode: companion      # for Syscribe native SVG rendering
# svgFile: ./MyDiagram.svg
# pumlMode: companion      # for PlantUML source generation (syscribe plantuml)
# pumlFile: ./MyDiagram.puml
shapes:
  - id: shape-a
    ref: SomePackage::PartA
edges: []
---

Describe the purpose and scope of this diagram.

<!-- When using pumlMode: companion, add an img tag pointing to the anticipated SVG:
<img src="./MyDiagram.svg" alt="MyDiagram" width="100%"/>
-->
"#,
        "view" => r#"---
type: View
name: myView
typedBy: Views::MyViewDef
expose:
  - SomePackage::SomeElement
---

Description of this view.
"#,
        "metadata" => r#"---
type: Metadata
name: myMetadata
typedBy: MetadataTypes::MyMetadataDef
# value: "..."
---
"#,
        "calculation" => r#"---
type: Calculation
name: myCalculation
typedBy: Calculations::MyCalculationDef
---

Description of this calculation usage.
"#,
        "constraint" => r#"---
type: Constraint
name: myConstraint
typedBy: Constraints::MyConstraintDef
---
"#,
        "librarypackage" => r#"---
type: LibraryPackage
name: MyLibraryPackage
---

Description of this library package.
"#,
        "namespace" => r#"---
type: Namespace
name: MyNamespace
---

Description of this namespace.
"#,
        "dependency" => r#"---
type: Dependency
name: myDependency
# dependsOn: SomePackage::SomeElement
---
"#,
        "usecase" => r#"---
type: UseCase
name: myUseCase
typedBy: UseCases::MyUseCaseDef
actors:
  - Operator
---

Description of this use case usage.
"#,
        "state" => r#"---
type: State
name: myState
typedBy: States::MyStateDef
---
"#,
        "enumeration" => r#"---
type: Enumeration
name: myEnumeration
typedBy: Enumerations::MyEnumerationDef
---
"#,
        "faulttree" => r#"---
type: FaultTree
id: FT-PREFIX-001
name: "Fault tree for [undesired top event]"
status: draft
topEvent: SG-PREFIX-001     # SafetyGoal this tree analyses
# missionTime: "1e9 h"
---

Describe the analysis scope and methodology.

# Directory layout — gates and events must be nested UNDER the FaultTree:
#
#   Safety/FTA/
#     FT-PREFIX-001.md              ← this file
#     FT-PREFIX-001/
#       FTG-PREFIX-001.md           ← top gate
#       FTG-PREFIX-002.md
#       FTE-PREFIX-001.md
#
# W900 fires if no FaultTreeGate or FaultTreeEvent are found as children
# (i.e. qualified names starting with Safety::FTA::FT-PREFIX-001::).
"#,
        "faulttreegate" => r#"---
type: FaultTreeGate
id: FTG-PREFIX-001
name: "OR gate — [description]"
gateType: OR                # AND | OR | XOR | NOT | inhibit
inputs:
  - FTG-PREFIX-002          # child gate (id or qname)
  - FTE-PREFIX-001          # or leaf event (id or qname)
# probability: 1.2e-7       # optional; computed from inputs
---

# Place this file inside the FaultTree's subdirectory:
#   Safety/FTA/FT-PREFIX-001/FTG-PREFIX-001.md
"#,
        "faulttreeevent" => r#"---
type: FaultTreeEvent
id: FTE-PREFIX-001
name: "[Component] [failure description]"
eventKind: basic            # basic | undeveloped | house
# ref: Package::Component   # model element this failure belongs to
# failureRate: 1.0e-9       # failures per hour (basic events)
# probability: 1.0e-6
---

# Place this file inside the FaultTree's subdirectory:
#   Safety/FTA/FT-PREFIX-001/FTE-PREFIX-001.md
"#,
        "attacktree" => r#"---
type: AttackTree
id: AT-PREFIX-001
name: "Attack tree for [ThreatScenario]"
status: draft
threatRef: TS-PREFIX-001     # ThreatScenario this tree substantiates (E917)
---

Describe the attack path analysis scope (ISO/SAE 21434 §15.7).

# Directory layout — gates and steps must be nested UNDER the AttackTree:
#
#   Security/Attacks/
#     AT-PREFIX-001.md              ← this file
#     AT-PREFIX-001/
#       ATG-PREFIX-001.md           ← root gate
#       ATG-PREFIX-002.md
#       ATS-PREFIX-001.md           ← leaf step
#
# W036 fires if no AttackTreeGate or AttackStep are found as children.
# W035 fires if the rolled-up feasibility disagrees with the ThreatScenario.
"#,
        "attacktreegate" => r#"---
type: AttackTreeGate
id: ATG-PREFIX-001
name: "OR gate — [description]"
gateType: OR                # AND (sequential path) | OR (alternatives)
inputs:
  - ATG-PREFIX-002          # child gate (id or qname)
  - ATS-PREFIX-001          # or leaf step (id or qname)
---

# Place this file inside the AttackTree's subdirectory:
#   Security/Attacks/AT-PREFIX-001/ATG-PREFIX-001.md
# Roll-up: AND = min of inputs (weakest link); OR = max of inputs (easiest path).
"#,
        "attackstep" => r#"---
type: AttackStep
id: ATS-PREFIX-001
name: "[Attacker action / sub-goal]"
attackFeasibility: medium   # high | medium | low | very_low
---

# Place this file inside the AttackTree's subdirectory:
#   Security/Attacks/AT-PREFIX-001/ATS-PREFIX-001.md
"#,
        "fmeasheet" => r#"---
type: FMEASheet
id: FMEA-PREFIX-001
name: "FMEA — [system or component name]"
status: draft
entries:
  - id: FM-PREFIX-001
    ref: Package::Component
    failureMode: "Loss of output signal"
    effect: "No command issued"
    cause: "Software exception in main loop"
    fmeaSeverity: 9         # 1–10
    occurrence: 3           # 1–10
    detection: 4            # 1–10
    # rpn: 108              # computed automatically as S×O×D if omitted
    recommendedAction: "Add watchdog monitor"
    # satisfies: REQ-PREFIX-001
  - id: FM-PREFIX-002
    ref: Package::SensorA
    failureMode: "Stuck-at-high output"
    effect: "False positive reading"
    cause: "Hardware fault"
    fmeaSeverity: 7
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
name: "TARA — [system or asset name]"
status: draft
damageTable:
  - id: DS-PREFIX-001
    name: "Unauthorized [action] enables [damage]"
    damageSeverity: severe    # severe | major | moderate | negligible
    impactCategories:
      - safety                # safety | financial | operational | privacy
threatTable:
  - id: TS-PREFIX-001
    name: "Attacker [action] via [attack surface]"
    attackFeasibility: medium # high | medium | low | very_low
    attackVector: network     # network | adjacent | local | physical
    damageScenarios:
      - DS-PREFIX-001
goalTable:
  - id: CSG-PREFIX-001
    name: "Ensure [security property] of [asset]"
    calLevel: CAL3            # CAL1 | CAL2 | CAL3 | CAL4
    securityProperty: integrity # confidentiality | integrity | availability | authenticity
    threatScenarios:
      - TS-PREFIX-001
controlTable:
  - id: SC-PREFIX-001
    name: "Implement [control mechanism]"
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
name: "Loss of [function] during [operating scenario]"
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
name: "Prevent [hazard] to avoid [harm]"
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
        "asset" => r#"---
type: Asset
id: ASSET-PREFIX-001
name: "What is being protected"
status: draft
cybersecurityProperties:  # confidentiality | integrity | availability | authenticity
  - integrity
assetOwner: Pkg::Subpkg::Component   # optional: owning architecture element (qname/id)
relatedSafetyGoal: SG-PREFIX-001     # optional: SafetyGoal ref for safety<->security co-engineering
---

Describe the asset (ISO/SAE 21434 §15.3) and why its cybersecurity properties matter.
"#,
        "damagescenario" => r#"---
type: DamageScenario
id: DS-PREFIX-001
name: "Unauthorized [action] enables [damage]"
status: draft
damageSeverity: severe    # severe | major | moderate | negligible
impactCategories:
  - safety                # safety | financial | operational | privacy
assets:                   # optional: Asset(s) this scenario damages (§15.3 -> §15.4 trace)
  - ASSET-PREFIX-001
---

Describe what damage could occur and to whom.
"#,
        "threatscenario" => r#"---
type: ThreatScenario
id: TS-PREFIX-001
name: "Attacker [action] via [attack surface]"
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
name: "Ensure [security property] of [asset]"
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
name: "Implement [control mechanism]"
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
        "confirmationmeasure" => r#"---
type: ConfirmationMeasure
id: CM-PREFIX-001
name: "Independent functional-safety assessment of [work product]"
status: planned          # planned | in_progress | completed
measureType: functional_safety_assessment
# measureType options:
#   confirmation_review | functional_safety_audit |
#   functional_safety_assessment | cybersecurity_assessment
independenceLevel: I3    # I1 | I2 | I3
confirms:
  - SG-PREFIX-001        # the confirmed work product(s) (any model element ref)
---

Record the confirmation measure: who performs it, with what independence (I1–I3),
and which work products it confirms (ISO 26262-2 §6 / ISO/SAE 21434 §7).

An ASIL D SafetyGoal/Requirement requires an I3 `functional_safety_assessment`;
a CAL4 CybersecurityGoal requires an I3 `cybersecurity_assessment` (else W039).
"#,
        "vulnerabilityreport" => r#"---
type: VulnerabilityReport
id: VR-PREFIX-001
name: "Stack buffer overflow in [component]"
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
        "argument" => r#"---
type: Argument
id: ARG-PREFIX-001
name: "Argue that [claim]"
status: draft
argumentType: strategy   # claim | strategy | solution
supports: SG-PREFIX-001  # the SafetyGoal or parent Argument this argues for
evidence:                # Requirement / TestCase / sub-Argument / AssumptionOfUse refs
  - REQ-PREFIX-001
  - TC-PREFIX-001
---

A GSN node in the safety case. Describe the claim/strategy and how the listed
evidence discharges it. Render the full tree with `syscribe safety-case`.
"#,
        "assumptionofuse" => r#"---
type: AssumptionOfUse
id: AOU-PREFIX-001
name: "Integrator provides [application condition]"
status: draft
appliesTo:
  - SG-PREFIX-001  # SafetyGoal / Argument / Requirement this SRAC constrains
---

A safety-related application condition (SRAC): a constraint the integrator must
honour for the referenced goal/argument/requirement to hold.
"#,
        other => {
            eprintln!("Unknown type '{}'. Known types:", other);
            eprintln!("  Native elements:  Requirement, TestCase, TestPlan, ADR");
            eprintln!("  Structural:       PartDef, Part, ItemDef, Item");
            eprintln!("  Interfaces:       PortDef, Port, InterfaceDef, Interface");
            eprintln!("  Connections:      ConnectionDef, Connection");
            eprintln!("  Actions:          ActionDef, Action");
            eprintln!("  Attributes:       AttributeDef, EnumerationDef, Enumeration");
            eprintln!("  Calculations:     CalculationDef, Calculation");
            eprintln!("  Constraints:      ConstraintDef, Constraint");
            eprintln!("  States:           StateDef, State");
            eprintln!("  Use cases:        UseCaseDef, UseCase");
            eprintln!("  Flows:            FlowDef");
            eprintln!("  Requirements:     RequirementDef");
            eprintln!("  Verification:     VerificationCaseDef, VerificationCase");
            eprintln!("  Analysis:         AnalysisCaseDef, AnalysisCase");
            eprintln!("  Allocation:       AllocationDef, Allocation");
            eprintln!("  Views:            ViewDef, View, ViewpointDef, Diagram");
            eprintln!("  Metadata:         MetadataDef, Metadata");
            eprintln!("  Packages:         Package, LibraryPackage, Namespace");
            eprintln!("  PLE:              FeatureDef, Configuration");
            eprintln!("  Misc:             Dependency");
            eprintln!("  Safety (HARA):    HazardousEvent, SafetyGoal");
            eprintln!("  Security (TARA):  Asset, DamageScenario, ThreatScenario, CybersecurityGoal,");
            eprintln!("                    SecurityControl, VulnerabilityReport, TARASheet");
            eprintln!("  FTA:              FaultTree, FaultTreeGate, FaultTreeEvent");
            eprintln!("  FMEA:             FMEASheet");
            eprintln!("  APA:              AttackTree, AttackTreeGate, AttackStep");
            eprintln!("  Confirmation:     ConfirmationMeasure");
            eprintln!("  Safety case (GSN): Argument, AssumptionOfUse");
            std::process::exit(1);
        }
    };
    print!("{}", out);
}

pub fn print_help() {
    println!("Usage: syscribe [-m <root>] [command] [args...]");
    println!();
    println!("Model root (priority order):");
    println!("  -m / --model <path>            Explicit flag");
    println!("  SYSCRIBE_MODEL=<path>          Environment variable");
    println!("  .syscribe.toml                 Auto-discovered by walking up from the current dir");
    println!("  model/                         Default fallback");
    println!();
    println!("Commands:");
    println!("  (none) / report                Full validation report (default)");
    println!("  validate [--json] [--file <f>] Validation findings only (errors + warnings)");
    println!("           [--deny <CODES>]      Treat the listed warning codes as gate failures (exit 2)");
    println!("           [--max-warnings <N>]  Fail (exit 2) when warnings exceed N");
    println!("           [--warnings-as-errors] Treat every warning as a gate failure (exit 2)");
    println!("           [--profile <name>]    Apply a named [profiles.<name>] gate from .syscribe.toml (exit 2; exit 1 if undefined)");
    println!("           [--results <f>]       Ingest test results for this run (W010), without writing the sidecar");
    println!("           [--fetch-remote]      Run the .syscribe.toml [remote] download hook to fetch & verify remote sourceFiles");
    println!("  ingest-results [--format cargo-json|junit] <file>");
    println!("                                 Parse test results into .syscribe/results.json (enables W010)");
    println!("  export [--ndjson]              Structured model graph as JSON (default) or NDJSON");
    println!("  types                          List all element types present in the model with counts");
    println!("  untyped                        List elements with no type: field set");
    println!("  list <type> [scope] [--tag <t>] List elements of a type (optional namespace scope; --tag filters by tags:)");
    println!("       [--feature <F>]           Keep only elements whose appliesWhen names FeatureDef F as an operand");
    println!("       [--status <s>]            Keep only elements whose status: equals s");
    println!("       [--sil <v>]               Keep only elements whose silLevel stringifies to v OR asilLevel equals v");
    println!("       [--has-wcet]              Keep only elements that declare a non-empty wcet:");
    println!("       [--json]                  Emit a JSON array (qualifiedName,type,name,id,status,silLevel,asilLevel,wcet)");
    println!("  matrix [--json] [--tag <t>]    Requirement × Configuration coverage matrix (cells: covered/gap/N-A)");
    println!("                                 Columns are Configuration elements; --json emits the grid; --tag filters rows.");
    println!("       [--status <s>]            Restrict rows to requirements whose status: equals s");
    println!("       [--gaps-only]             Drop fully-covered and all-N/A rows; keep only rows with a gap cell");
    println!("       [--linked-only]           Ignore ingested results: covered cells stay ✓ (today's linked-only view)");
    println!("                                 With a .syscribe/results.json sidecar (default): ✓ covered+passing, ▣ covered, not passing");
    println!("                                 A per-config + overall coverage-% footer is printed (coverage object in --json).");
    println!("                                 With no feature model, falls back to a flat requirement/testcase view.");
    println!("  matrix --features [--json]     Feature × Configuration selection grid (cell ✓ where selected true)");
    println!("  matrix --allocations [--json]  Allocation source × target matrix; rollup of unallocated sources /");
    println!("                                 unused targets; logical→physical partition when mg_layer is present.");
    println!("  magicgrid [--json]             MagicGrid B/W/S × 1-4 cell report over mg_cell; flags empty cells.");
    println!("  trade-study [--json]           MoE-weighted trade study: mg_moe rows × Configuration columns, each");
    println!("       [--config <id|qname> ...] cell VALUE (SCORE); rollup marks WINNER / FAIL. --config restricts columns.");
    println!("  features [--json]              Feature-model overview: every FeatureDef with groupKind, requires/excludes,");
    println!("                                 parameters and a 'selected in N/M' rollup. Notice + exit 0 with no feature model.");
    println!("  feature <qname|name> [--json]  Single-feature card: Gates (elements gating on it), Selected in (configs),");
    println!("                                 groupKind, requires/excludes, parameters. Errors on a non-FeatureDef arg.");
    println!("  why-active <qname|id> --config <CONF|features> [--json]");
    println!("                                 Explain an element's activation under a configuration: appliesWhen, the");
    println!("                                 referenced feature selections, and Verdict: active|inactive|always active.");
    println!("  feature-check [--json] [--deep] Holistic feature-model validation (requires/excludes, dead features,");
    println!("                                 orphan features W024, ...). [--deny <CODES>] gates warnings (exit 2).");
    println!("                                 derivedFrom cycles, bindTo ranges, parameterConstraints). Separate from");
    println!("                                 `validate`; exit 0 if no errors, 1 otherwise; dormant with no feature model.");
    println!("                                 --deep adds SAT-backed analysis: void model, dead/core/false-optional");
    println!("                                 features, full-semantics config validity, and void-model diagnoses.");
    println!("                                 --count / --enumerate report the number of valid configurations.");
    println!("  configure <Configuration> [--json]  Assisted configuration: from a partial selection, report");
    println!("                                 satisfiability + forced/free features (exit 1 if contradictory).");
    println!("  build-config --config <id>     Materialise a configuration's selection as build flags. --format <fmt>");
    println!("       [--format <fmt>] [--prefix <p>] [--no-validate]  (default json); --prefix names the output; --no-validate");
    println!("  build-config --all-configs     skips the per-variant validation. --all-configs emits every Configuration.");
    println!("       [--format <fmt>] [--prefix <p>] [--no-validate]");
    println!("  diff --config <A> --config <B> Elements active in one configuration but not the other");
    println!("  audit [--json] [--profile <p>] Safety-readiness dashboard: requirement status split (overall +");
    println!("                                 per package), SIL/ASIL distribution, per-configuration coverage %,");
    println!("                                 orphans, and a PASS/FAIL verdict. Reuses validate + matrix coverage +");
    println!("                                 #18 profiles. Exit 0 PASS · 2 FAIL (any error, any W306, or — under");
    println!("                                 --profile <p> — any finding that profile promotes); exit 1 if <p> undefined.");
    println!();
    println!("Configuration lens (§9 projection; inert when no feature model):");
    println!("  --config <CONF|features>       On validate/list/export: project onto a configuration (stored id/qname");
    println!("                                 or ad-hoc 'Features::A,Features::B'). validate --config certifies the");
    println!("                                 variant and flags escaping refs (E226 structural / W019 traceability).");
    println!("  validate --all-configs         Validate every stored Configuration; exit non-zero if any has errors.");
    println!("  show <qname|id>                Show element details and documentation");
    println!("  ls [qname]                     List namespace children (default: root)");
    println!("  tree [qname]                   Recursive namespace tree (default: root)");
    println!("  find <pattern>                 Fuzzy search by name / ID / content");
    println!("  extref <ref> [--json]          Find elements by external reference (extRef)");
    println!("  path-for <qname|id>            Print the file path for an element");
    println!("  check-ref <qname|id>           Verify a cross-reference resolves and show its type");
    println!("  next-id <id-prefix>            Print the next available stable ID for a prefix");
    println!("  template <type>                Print a ready-to-fill frontmatter skeleton for a type");
    println!("  scaffold-gherkin <TC> [--fix]  Generate/align Gherkin Scenario blocks from testFunctions");
    println!("  applies-when <qname|id>        Show / set / clear an element's appliesWhen: gate. No flag = read-only");
    println!("       [--set \"<expr>\" | --clear] [--dry-run] [--json]  (own + effective); --set writes, --clear removes.");
    println!("  move <src> <dest> [--dry-run]  Move an element/package to a new qname, rewriting all references");
    println!("  trace <qname|req-id>           Full traceability slice for a requirement");
    println!("        [--linked-only]          Ignore ingested results (default annotates verifying TCs with [pass]/[fail]/[unknown])");
    println!("  links <qname|id>               All outbound and inbound relationships");
    println!("  connectivity <qname|id>        Element-rooted transitive subgraph: reachable elements + the");
    println!("        [--format text|dot|json]  connections between them. text (default) is an indented tree;");
    println!("        [--depth N]               json emits {{root,nodes,edges}}; dot emits styled Graphviz.");
    println!("        [--kinds <csv>]           Edge kinds to follow (default connection,flow,binding,succession,");
    println!("        [--undirected]            contains,typedBy — so the model-root element dumps the whole model).");
    println!("                                 --depth bounds hops; --undirected walks edges both ways.");
    println!("  why <qname>                    What requirements this element satisfies");
    println!("  who-verifies <req-id>          Which test cases cover a requirement");
    println!("  verification-depth [--sil <v>] [--status <s>] [--min-levels N] [--json]");
    println!("                                 Per-requirement distinct verification levels + depth flag (gate with --min-levels)");
    println!("                                 (composes with the --plan/--config lenses)");
    println!("  testplan [<TP-id>] [--json]    TestPlan surface: list every TestPlan, or detail one (resolved members,");
    println!("                                 in-scope requirements, per-config coverage grid, rolled-up verdict).");
    println!("  refs <qname|id>                What elements reference this element");
    println!("                                 (for a Configuration: also the TestCases that run in it)");
    println!("  co-analysis [--json]           Safety↔security co-engineering view (ISO 26262 ⇄ ISO/SAE 21434):");
    println!("                                 each SafetyGoal/HazardousEvent with the cyber ThreatScenarios that");
    println!("                                 can violate it (via DamageScenario.hazardRef) + safety-tagged");
    println!("                                 DamageScenarios with no hazardRef (the W030 gaps).");
    println!("  cyber-risk [--json]            ISO/SAE 21434 risk determination: each ThreatScenario with its");
    println!("                                 damage severity, attack feasibility, computed risk level (low|medium|");
    println!("                                 high|critical), riskTreatment, addressed-by-goal, and a flag");
    println!("                                 (untreated = trips W031, else ok/unknown).");
    println!("  metrics [--json]               Quantitative HW safety metrics (ISO 26262-5 §8-9): per SafetyGoal,");
    println!("                                 computed SPFM / LFM / PMHF from FaultTreeEvent failureRate +");
    println!("                                 diagnosticCoverage, with pass/fail vs ASIL/SIL target. Opt-in:");
    println!("                                 goals without diagnosticCoverage show n/a (gate via --deny W033).");
    println!("  safety-case [<SG-id>] [--json] GSN safety-argument tree: per SafetyGoal, the Arguments (claim/strategy/");
    println!("                                 solution) that support it, their evidence (Requirements/TestCases with");
    println!("                                 ingested verdict), the AssumptionOfUse (SRAC) nodes, plus the implicit");
    println!("                                 SafetyGoal→Requirement→TestCase fold-in (works without Argument nodes).");
    println!("  fmea report [--fmea-sheet <id>] [--json]");
    println!("                                 FMEA worksheet rollup: per FMEAEntry the S/O/D ratings, computed RPN, and");
    println!("                                 actions; --fmea-sheet restricts to one sheet, --json emits the document.");
    println!("  fault-tree render <FaultTree-id>");
    println!("                                 Render a FaultTree as an indented gate/event tree (AND/OR, λ, DC).");
    println!();
    println!("Architecture analysis:");
    println!("  n2 [<qname>] [--depth N]       N² interface matrix (§16): in-scope PartDef/Part on the diagonal, the");
    println!("     [--format text|html|json]   interfaces directed R→C off-diagonal. No qname = whole model; a qname =");
    println!("     [--interfaces-only]         that composite's subpart types, expanded --depth levels (default 1).");
    println!("     [--allocations]             --allocations adds allocatedTo edges; --interfaces-only drops empty parts.");
    println!("  impact <qname|id>              Change-impact traversal (§17): every reachable node, its hop distance,");
    println!("     [--direction downstream|upstream|both]  and the connecting edge kind. downstream = reverse links,");
    println!("     [--depth N] [--kinds <csv>] upstream = forward links, both = union. --depth bounds hops; cycle-safe.");
    println!("     [--format text|json|dot]");
    println!("  behavioral-coverage [<scope>]  How completely active TestCases exercise behavioral elements (§20).");
    println!("     [--depth N] [--uncovered-only]  --include-planned adds draft/review/approved tests in a column;");
    println!("     [--include-planned]         --uncovered-only filters; --format text|json with a coverage %.");
    println!("     [--format text|json]");
    println!();
    println!("IEC 62443 / security & supply chain:");
    println!("  zones [--coverage] [--json]    Zone elements (§13) with targetSL/achievedSL, members, SL-gap status.");
    println!("                                 --coverage adds a Zone × SecurityControl cross-table.");
    println!("  conduits [--json]              Conduit elements with from/to zones, achievedSL, and SL-boundary check.");
    println!("  sbom [--format cyclonedx|spdx] Software Bill of Materials (§18) from implementedBy: links. --include-tests");
    println!("       [--config <C>] [--scope <qname>]  adds TestCase.sourceFile components; --config projects a variant;");
    println!("       [--include-tests] [--output <f>]  --output writes a file (default stdout).");
    println!("  export-reqif [--output <f>]    ReqIF 1.2 export (§21) of Requirement elements + packages for DOORS Next /");
    println!("       [--scope <qname>] [--config <C>]  Jama / Polarion. --include-tests adds TEST_CASE + VERIFIED_BY;");
    println!("       [--include-tests] [--zip]   --zip writes .reqifz; --output sets the path.");
    println!();
    println!("Reviews & multi-repo:");
    println!("  reviews [<qname>] [--open-only] ReviewRecord surface (§19): list reviews + coverage. <qname> filters to");
    println!("       [--coverage] [--json]       reviews covering an element; --open-only = open action items;");
    println!("                                 --coverage = native-Requirement review-coverage cross-table.");
    println!("  review <RR-id> [--json]        Show one ReviewRecord in full.");
    println!("  repos [list] [--json]          Multi-repo composition (§14): list configured peer repos (path, ref,");
    println!("  repos status [--json]          on-disk + sync status). status exits 2 on ref drift; sync runs git fetch +");
    println!("  repos sync [--all | <alias>]   checkout <ref> for pinned repos (--all or a single alias).");
    println!();
    println!("Diagrams:");
    println!("  render <diagram_path>          Render a Diagram element to stdout (Mermaid or SVG per diagramKind).");
    println!("  diagram list [--type <T>] [--namespace <NS>]");
    println!("                                 List candidate elements for diagram generation (type / namespace filters).");
    println!("  diagram render <qname>         Render one element's diagram. --view full|ports|features|compact|name|");
    println!("       [--output <f>] [--view <preset>]  requirement; --include-ports/--include-features <csv>; --min-width N.");
    println!("  diagram measure <qnames>       Print computed box dimensions for a comma-separated list of qnames.");
    println!("  diagram compose <layout.json|qname>  Compose a multi-element SVG from a layout file or Diagram element.");
    println!("       [--output <f>] [--kind bdd|ibd|arch] [--emit-placement]");
    println!("  diagram layout <placement.json|->    Resolve a placement file into a final layout (--compose pipes to SVG).");
    println!("       [--output <f>] [--compose] [--kind bdd|ibd|arch] [--svg <f>]");
    println!("  diagram seq <qname> [--output <f>]   Render a Sequence Diagram element to SVG.");
    println!("  diagram req <root> [--depth N] Render a requirement-breakdown tree. --show-verify adds TestCases,");
    println!("       [--show-verify] [--show-satisfy] [--output <f>]  --show-satisfy adds satisfying architecture.");
    println!("  plantuml [<qname>] [--output <f>|-] [--dry-run]");
    println!("                                 Generate PlantUML .puml from Diagram elements (batch, or single qname).");
    println!("  plantuml render [--jar <path>] [--dry-run]");
    println!("                                 Render companion .puml files to .svg via PlantUML (--jar / PLANTUML_JAR / PATH).");
    println!();
    println!("Documentation & extension hygiene:");
    println!("  lint-docs <path>... [--json]   Scan external .md/.svg for stale element references (W099–W102).");
    println!("  scripts list [--json]          Enumerate registered Rhai extension commands/checks.");
    println!("  scripts run <command> [--json] Invoke a registered extension command and print its result.");
    println!("  scripts validate [--deny <CODES>] [--max-warnings <N>] [--warnings-as-errors] [--json]");
    println!("                                 Run every registered check (separate from the built-in validate).");
    println!();
    println!("Variability (§9, opt-in — dormant unless a FeatureDef is linked):");
    println!("  appliesWhen: <expr>            On any element/TestCase: a boolean expression over FeatureDef QNames");
    println!("                                 (and / or / not / parentheses; a bare QName or list[AND] also work).");
    println!("                                 A TestCase runs in a Configuration iff its appliesWhen holds there.");
    println!("                                 'matrix' and the W015 per-config coverage rule build on this.");
    println!();
    println!("Spec browser (no model root required):");
    println!("  spec                           Table of contents for the format spec");
    println!("  spec types                     Element type inventory and native type schemas");
    println!("  spec fields                    Complete frontmatter field reference");
    println!("  spec namespace                 Directory conventions, cross-refs, multiplicity");
    println!("  spec validation                All validation rule codes");
    println!("  spec traceability              Traceability rules R-001–R-007");
    println!("  spec safety                    Safety/security analysis elements (HARA/TARA/FTA/FMEA)");
    println!();
    println!("Exit codes (validate):");
    println!("  0                              No errors and no gate failures");
    println!("  1                              One or more Error-severity findings, or an undefined --profile name");
    println!("  2                              Warnings tripped a gate (--deny / --max-warnings / --warnings-as-errors / --profile)");
    println!();
    println!("Options:");
    println!("  -m, --model <path>             Model root directory");
    println!("  --agent-instructions [topic]   Print the LLM authoring prompt; topic 'magicgrid' teaches MagicGrid modeling");
    println!("  --version, -V                  Print the tool version (also `syscribe version`)");
    println!("  --help, -h                     Show this help");
    println!();
    println!("Examples:");
    println!("  syscribe -m model/ validate");
    println!("  syscribe -m model/ validate --json");
    println!("  syscribe -m model/ validate --file model/UAV/Avionics/FlightController.md");
    println!("  syscribe -m model/ list PartDef");
    println!("  syscribe -m model/ list PortDef UAV::Avionics");
    println!("  syscribe -m model/ list Requirement --tag smoke");
    println!("  syscribe -m model/ matrix");
    println!("  syscribe -m model/ matrix --json --tag safety");
    println!("  syscribe -m model/ path-for UAV::Avionics::FlightController");
    println!("  syscribe -m model/ check-ref Interfaces::TelemetryPortDef");
    println!("  syscribe -m model/ next-id REQ-UAV-FC");
    println!("  syscribe -m model/ template PartDef");
    println!("  syscribe -m model/ template Requirement");
    println!("  syscribe -m model/ find FlightController");
    println!("  syscribe -m model/ show UAV::Avionics::FlightController");
    println!("  syscribe -m model/ ls UAV::Avionics");
    println!("  syscribe -m model/ tree UAV");
    println!("  syscribe -m model/ trace REQ-UAV-FC-001");
    println!("  syscribe -m model/ links UAV::Avionics::FlightController");
    println!("  syscribe -m model/ why UAV::Avionics::FlightController");
    println!("  syscribe -m model/ who-verifies REQ-UAV-SAFE-001");
    println!("  syscribe -m model/ refs Interfaces::TelemetryPortDef");
    println!("  SYSCRIBE_MODEL=model/ syscribe validate");
    println!();
    println!("Detailed help: `syscribe help <command>` or `syscribe <command> --help` (e.g. `syscribe help audit`).");
}

#[cfg(test)]
mod custom_where_tests {
    use super::*;

    fn elem_with(fields: &[(&str, serde_yaml::Value)]) -> RawElement {
        let mut fm = syscribe_model::element::RawFrontmatter::default();
        for (k, v) in fields {
            fm.custom_fields.insert(k.to_string(), v.clone());
        }
        RawElement {
            qualified_name: "Test::Elem".to_string(),
            file_path: "Test/Elem.md".to_string(),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None,
            derived: std::collections::HashMap::new(),
            derive_findings: Vec::new(),
        }
    }

    fn s(v: &str) -> serde_yaml::Value {
        serde_yaml::Value::String(v.to_string())
    }
    fn list(vs: &[&str]) -> serde_yaml::Value {
        serde_yaml::Value::Sequence(vs.iter().map(|x| s(x)).collect())
    }

    #[test]
    fn parse_precedence_longest_first() {
        // `=~` wins over `=`
        assert_eq!(
            parse_custom_where("custom.k=~pat").unwrap(),
            CustomWhere::Regex { key: "k".into(), pat: "pat".into() }
        );
        // `~=` wins over `=`
        assert_eq!(
            parse_custom_where("custom.k~=v").unwrap(),
            CustomWhere::Member { key: "k".into(), val: "v".into() }
        );
        // bare `=`
        assert_eq!(
            parse_custom_where("custom.k=v").unwrap(),
            CustomWhere::Eq { key: "k".into(), val: "v".into() }
        );
        // presence
        assert_eq!(
            parse_custom_where("custom.k").unwrap(),
            CustomWhere::Present { key: "k".into() }
        );
    }

    #[test]
    fn parse_errors_on_bad_input() {
        assert!(parse_custom_where("supplier=Bosch").is_err()); // missing custom. prefix
        assert!(parse_custom_where("custom.=v").is_err()); // empty key
        assert!(parse_custom_where("custom.").is_err()); // empty key, presence
    }

    #[test]
    fn exact_scalar_and_list() {
        let e = elem_with(&[("supplier", s("Bosch")), ("nums", list(&["A-1", "A-2"]))]);
        let p = parse_custom_where("custom.supplier=Bosch").unwrap();
        assert!(custom_field_matches(&e, &p));
        let p = parse_custom_where("custom.supplier=Conti").unwrap();
        assert!(!custom_field_matches(&e, &p));
        // exact on a list: any element equals
        let p = parse_custom_where("custom.nums=A-2").unwrap();
        assert!(custom_field_matches(&e, &p));
    }

    #[test]
    fn regex_and_substring_fallback() {
        let e = elem_with(&[("cc", s("PWT-4471"))]);
        // substring via regex
        assert!(custom_field_matches(&e, &parse_custom_where("custom.cc=~PWT").unwrap()));
        // anchored regex
        assert!(custom_field_matches(&e, &parse_custom_where("custom.cc=~^PWT-\\d+$").unwrap()));
        assert!(!custom_field_matches(&e, &parse_custom_where("custom.cc=~^XXX").unwrap()));
        // invalid regex falls back to substring — literal "(" present? no, so false
        assert!(!custom_field_matches(&e, &parse_custom_where("custom.cc=~(").unwrap()));
    }

    #[test]
    fn member_list_and_scalar() {
        let e = elem_with(&[("nums", list(&["A-1001", "A-1002"])), ("one", s("solo"))]);
        assert!(custom_field_matches(&e, &parse_custom_where("custom.nums~=A-1001").unwrap()));
        assert!(!custom_field_matches(&e, &parse_custom_where("custom.nums~=A-9999").unwrap()));
        // scalar membership = equality
        assert!(custom_field_matches(&e, &parse_custom_where("custom.one~=solo").unwrap()));
    }

    #[test]
    fn presence_and_absence() {
        let e = elem_with(&[("supplier", s("Bosch"))]);
        assert!(custom_field_matches(&e, &parse_custom_where("custom.supplier").unwrap()));
        // absent field never matches, including presence
        assert!(!custom_field_matches(&e, &parse_custom_where("custom.missing").unwrap()));
        assert!(!custom_field_matches(&e, &parse_custom_where("custom.missing=x").unwrap()));
    }

    #[test]
    fn number_scalar_renders_and_matches() {
        let e = elem_with(&[("reviewCycle", serde_yaml::Value::Number(3.into()))]);
        assert!(custom_field_matches(&e, &parse_custom_where("custom.reviewCycle=3").unwrap()));
        assert_eq!(custom_field_display(&serde_yaml::Value::Number(3.into())), "3");
    }
}
