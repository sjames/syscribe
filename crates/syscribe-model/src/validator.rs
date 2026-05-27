use std::collections::{HashMap, HashSet};
use crate::element::{ElementType, RawElement};
use crate::resolver::{is_adr_id, is_conf_id, is_req_id, is_tc_id, Resolver};

/// A single validation finding.
#[derive(Debug, Clone)]
pub struct Finding {
    pub code: &'static str,
    pub file: String,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tag = match self.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARN",
        };
        write!(f, "[{}] {} {}: {}", tag, self.code, self.file, self.message)
    }
}

pub struct ValidationResult {
    pub findings: Vec<Finding>,
    /// verifiedBy[req_id] = list of tc ids that have status:active
    pub verified_by: HashMap<String, Vec<String>>,
    /// derived_children[req_id] = list of child req ids
    pub derived_children: HashMap<String, Vec<String>>,
}

impl ValidationResult {
    pub fn errors(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|f| f.severity == Severity::Error)
    }
    pub fn warnings(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|f| f.severity == Severity::Warning)
    }
}

/// Extract qualified name strings from a field that may be a YAML String or Sequence.
fn yaml_strings(v: &serde_yaml::Value) -> Vec<&str> {
    match v {
        serde_yaml::Value::String(s) => vec![s.as_str()],
        serde_yaml::Value::Sequence(seq) => seq.iter().filter_map(|x| x.as_str()).collect(),
        _ => vec![],
    }
}

/// Run all parse-time and model-time validation rules against a loaded element list.
pub fn validate(elements: &[RawElement]) -> ValidationResult {
    let mut findings: Vec<Finding> = Vec::new();
    let resolver = Resolver::new(elements);

    // ── Parse-time checks (per-element) ──────────────────────────────────────

    for elem in elements {
        let file = elem.file_path.clone();
        let fm = &elem.frontmatter;

        // E004: required fields for native elements
        if let Some(ElementType::TestCase) = &fm.element_type {
            if fm.id.is_none() {
                findings.push(error("E004", &file, "`id` is required on TestCase"));
            }
            if fm.title.is_none() {
                findings.push(error("E004", &file, "`title` is required on TestCase"));
            }
            if fm.status.is_none() {
                findings.push(error("E004", &file, "`status` is required on TestCase"));
            }
            if fm.test_level.is_none() {
                findings.push(error("E004", &file, "`testLevel` is required on TestCase"));
            }
            if fm.verifies.as_ref().map_or(true, |v| v.is_empty()) {
                findings.push(error("E013", &file, "`verifies` must have at least one entry on TestCase"));
            }
        }

        if let Some(ElementType::Requirement) = &fm.element_type {
            if let Some(ref id) = fm.id {
                if is_req_id(id) {
                    // native Requirement: check required fields
                    if fm.title.is_none() {
                        findings.push(error("E004", &file, "`title` is required on native Requirement"));
                    }
                    if fm.status.is_none() {
                        findings.push(error("E004", &file, "`status` is required on native Requirement"));
                    }
                }
            }
        }

        // E006: id pattern validation
        if let Some(ref id) = fm.id {
            let ty = &fm.element_type;
            let is_req = matches!(ty, Some(ElementType::Requirement));
            let is_tc = matches!(ty, Some(ElementType::TestCase));
            if is_req && !is_req_id(id) && !id.is_empty() {
                findings.push(error("E006", &file, &format!("`id` '{}' does not match REQ pattern", id)));
            }
            if is_tc && !is_tc_id(id) && !id.is_empty() {
                findings.push(error("E006", &file, &format!("`id` '{}' does not match TC pattern", id)));
            }
        }

        // E007: status enum
        if let Some(ref status) = fm.status {
            let ty = &fm.element_type;
            let is_tc = matches!(ty, Some(ElementType::TestCase));
            let is_req = matches!(ty, Some(ElementType::Requirement));
            if is_req {
                const REQ_STATUSES: &[&str] = &["draft", "review", "approved", "implemented", "verified"];
                if !REQ_STATUSES.contains(&status.as_str()) {
                    findings.push(error("E007", &file, &format!("unknown Requirement status '{}'", status)));
                }
            }
            if is_tc {
                const TC_STATUSES: &[&str] = &["draft", "review", "approved", "active", "retired"];
                if !TC_STATUSES.contains(&status.as_str()) {
                    findings.push(error("E007", &file, &format!("unknown TestCase status '{}'", status)));
                }
            }
        }

        // E008: testLevel
        if let Some(ref lvl) = fm.test_level {
            const LEVELS: &[&str] = &["L1", "L2", "L3", "L4", "L5"];
            if !LEVELS.contains(&lvl.as_str()) {
                findings.push(error("E008", &file, &format!("unknown testLevel '{}'", lvl)));
            }
        }

        // E009: silLevel 1–4
        if let Some(sil) = fm.sil_level {
            if !(1..=4).contains(&sil) {
                findings.push(error("E009", &file, &format!("silLevel {} out of range 1–4", sil)));
            }
        }

        // E010: asilLevel A–D
        if let Some(ref asil) = fm.asil_level {
            const ASIL: &[&str] = &["A", "B", "C", "D"];
            if !ASIL.contains(&asil.as_str()) {
                findings.push(error("E010", &file, &format!("unknown asilLevel '{}'", asil)));
            }
        }

        // E011: TestCase must have a gherkin block
        if matches!(fm.element_type, Some(ElementType::TestCase)) {
            if !elem.doc.contains("```gherkin") {
                findings.push(error("E011", &file, "TestCase body has no ```gherkin fenced block"));
            }
        }

        // E012: native Requirement normative text must be non-empty
        if let Some(ElementType::Requirement) = &fm.element_type {
            if fm.id.as_deref().map(is_req_id).unwrap_or(false) {
                let normative = normative_text(&elem.doc);
                if normative.trim().is_empty() {
                    findings.push(error("E012", &file, "Requirement normative text is empty"));
                }
            }
        }

        // E014: Scenario Outline without Examples table
        if matches!(fm.element_type, Some(ElementType::TestCase)) {
            check_scenario_outline_has_examples(&elem.doc, &file, &mut findings);
        }

        // E015: first gherkin block must have Feature: line
        if matches!(fm.element_type, Some(ElementType::TestCase)) {
            if !first_gherkin_has_feature(&elem.doc) {
                findings.push(error("E015", &file, "first ```gherkin block has no Feature: line"));
            }
        }

        // W001: normative text should contain "shall"
        if let Some(ElementType::Requirement) = &fm.element_type {
            if fm.id.as_deref().map(is_req_id).unwrap_or(false) {
                let normative = normative_text(&elem.doc);
                if !normative.contains("shall") {
                    findings.push(warning("W001", &file, "normative text contains no 'shall'"));
                }
            }
        }

        // W006: silLevel without asilLevel or vice versa
        match (&fm.sil_level, &fm.asil_level) {
            (Some(_), None) => findings.push(warning("W006", &file, "silLevel present without asilLevel")),
            (None, Some(_)) => findings.push(warning("W006", &file, "asilLevel present without silLevel")),
            _ => {}
        }

        // W004: sourceFile must exist
        if let Some(ref sf) = fm.source_file {
            if !std::path::Path::new(sf).exists() {
                findings.push(warning("W004", &file, &format!("sourceFile '{}' does not exist on disk", sf)));
            }
        }

        // E200: Configuration id must match CONF-* pattern
        if matches!(fm.element_type, Some(ElementType::Configuration)) {
            if let Some(ref id) = fm.id {
                if !is_conf_id(id) {
                    findings.push(error("E200", &file, &format!("`id` '{}' does not match CONF-* pattern", id)));
                }
            }
        }

        // E201: Configuration required fields
        if matches!(fm.element_type, Some(ElementType::Configuration)) {
            if fm.id.is_none() {
                findings.push(error("E201", &file, "`id` is required on Configuration"));
            }
            if fm.title.is_none() {
                findings.push(error("E201", &file, "`title` is required on Configuration"));
            }
            if fm.status.is_none() {
                findings.push(error("E201", &file, "`status` is required on Configuration"));
            }
            if fm.feature_model.is_none() {
                findings.push(error("E201", &file, "`featureModel` is required on Configuration"));
            }
        }

        // E300: ADR.id must match ADR-* pattern
        if matches!(fm.element_type, Some(ElementType::ADR)) {
            if let Some(ref id) = fm.id {
                if !is_adr_id(id) {
                    findings.push(error("E300", &file, &format!("`id` '{}' does not match ADR-* pattern", id)));
                }
            }
        }

        // E301: ADR required fields
        if matches!(fm.element_type, Some(ElementType::ADR)) {
            if fm.id.is_none() {
                findings.push(error("E301", &file, "`id` is required on ADR"));
            }
            if fm.title.is_none() {
                findings.push(error("E301", &file, "`title` is required on ADR"));
            }
            if fm.status.is_none() {
                findings.push(error("E301", &file, "`status` is required on ADR"));
            }
        }

        // E302: reqDomain enum validation
        if let Some(ref rd) = fm.req_domain {
            const DOMAINS: &[&str] = &["system", "hardware", "software"];
            if !DOMAINS.contains(&rd.as_str()) {
                findings.push(error("E302", &file, &format!("unknown reqDomain value '{}'", rd)));
            }
        }

        // E303: domain enum validation
        if let Some(ref d) = fm.domain {
            const DOMAINS: &[&str] = &["system", "hardware", "software"];
            if !DOMAINS.contains(&d.as_str()) {
                findings.push(error("E303", &file, &format!("unknown domain value '{}'", d)));
            }
        }

        // E304: ADR.status enum validation
        if matches!(fm.element_type, Some(ElementType::ADR)) {
            if let Some(ref status) = fm.status {
                const ADR_STATUSES: &[&str] = &["proposed", "accepted", "deprecated", "superseded"];
                if !ADR_STATUSES.contains(&status.as_str()) {
                    findings.push(error("E304", &file, &format!("unknown ADR status '{}'", status)));
                }
            }
        }

        // W304: isDeploymentPackage: true combined with domain: hardware
        if fm.is_deployment_package == Some(true) {
            if fm.domain.as_deref() == Some("hardware") {
                findings.push(warning("W304", &file, "`isDeploymentPackage: true` combined with `domain: hardware` — deployment packages must be software"));
            }
        }

        // ── Diagram checks (E4xx / W4xx) ─────────────────────────────────────

        if matches!(fm.element_type, Some(ElementType::Diagram)) {
            // W400: no diagramKind — rendering mode is ambiguous
            if fm.diagram_kind.is_none() {
                findings.push(warning("W400", &file, "Diagram element has no `diagramKind` — rendering mode ambiguous"));
            }
            // E400: Mermaid diagrams require a ```mermaid fenced block in the body
            if fm.diagram_kind.as_deref() == Some("Mermaid") && !elem.doc.contains("```mermaid") {
                findings.push(error("E400", &file, "`diagramKind: Mermaid` but body has no ```mermaid fenced block"));
            }
            // E401: PlantUML diagrams require a ```plantuml fenced block in the body
            if fm.diagram_kind.as_deref() == Some("PlantUML") && !elem.doc.contains("```plantuml") {
                findings.push(error("E401", &file, "`diagramKind: PlantUML` but body has no ```plantuml fenced block"));
            }
            // W408 / W409: validate %% ref: annotations inside Mermaid blocks.
            // Convention: `%% ref: QualifiedName` on any line within the ```mermaid block.
            // W408 fires for each annotation that doesn't resolve.
            // W409 fires when no annotations are present at all.
            if fm.diagram_kind.as_deref() == Some("Mermaid") {
                let mermaid_block = elem.doc.find("```mermaid").and_then(|start| {
                    let after_fence = start + "```mermaid".len();
                    elem.doc[after_fence..].find("```").map(|end| &elem.doc[after_fence..after_fence + end])
                });
                if let Some(block) = mermaid_block {
                    let mut ref_count = 0usize;
                    for line in block.lines() {
                        let trimmed = line.trim();
                        if let Some(ref_str) = trimmed.strip_prefix("%% ref:") {
                            let ref_str = ref_str.trim();
                            if !ref_str.is_empty() {
                                ref_count += 1;
                                if resolver.resolve_ref(elements, ref_str).is_none() {
                                    findings.push(warning(
                                        "W408",
                                        &file,
                                        &format!("Mermaid `%% ref:` annotation '{}' does not resolve to a known element", ref_str),
                                    ));
                                }
                            }
                        }
                    }
                    if ref_count == 0 {
                        findings.push(warning(
                            "W409",
                            &file,
                            "Mermaid diagram has no `%% ref:` annotations — add at least one to link diagram nodes to model elements",
                        ));
                    }
                }
            }
            // W401: subject must resolve to a known element
            if let Some(ref subj) = fm.subject {
                if resolver.resolve_ref(elements, subj).is_none() {
                    findings.push(warning(
                        "W401",
                        &file,
                        &format!("`subject` '{}' does not resolve to a known element", subj),
                    ));
                }
            }
            // W402: shapes ref must resolve; refs where any ancestor resolves are suppressed
            // (covers inline features at any depth, e.g. System::part::port::subport)
            let validate_shape_ref = |ref_str: &str, findings: &mut Vec<Finding>| {
                if resolver.resolve_ref(elements, ref_str).is_some() {
                    return;
                }
                let has_resolvable_ancestor = {
                    let mut seg = ref_str;
                    let mut found = false;
                    while let Some(pos) = seg.rfind("::") {
                        seg = &seg[..pos];
                        if resolver.resolve_ref(elements, seg).is_some() {
                            found = true;
                            break;
                        }
                    }
                    found
                };
                if !has_resolvable_ancestor {
                    findings.push(warning(
                        "W402",
                        &file,
                        &format!("shapes `ref` '{}' does not resolve to a known element", ref_str),
                    ));
                }
            };
            match fm.shapes.as_ref() {
                Some(serde_yaml::Value::Mapping(shapes_map)) => {
                    for shape_val in shapes_map.values() {
                        if let serde_yaml::Value::Mapping(attrs) = shape_val {
                            if let Some(serde_yaml::Value::String(ref_str)) =
                                attrs.get(&serde_yaml::Value::String("ref".into()))
                            {
                                validate_shape_ref(ref_str, &mut findings);
                            }
                        }
                    }
                }
                Some(serde_yaml::Value::Sequence(shapes_seq)) => {
                    for shape_val in shapes_seq {
                        if let serde_yaml::Value::Mapping(attrs) = shape_val {
                            if let Some(serde_yaml::Value::String(ref_str)) =
                                attrs.get(&serde_yaml::Value::String("ref".into()))
                            {
                                validate_shape_ref(ref_str, &mut findings);
                            }
                        }
                    }
                }
                _ => {}
            }
            // W403: edge source/target must reference a shape id defined in this diagram's shapes
            let shape_ids: HashSet<String> = match fm.shapes.as_ref() {
                Some(serde_yaml::Value::Mapping(map)) => {
                    map.keys().filter_map(|k| k.as_str().map(|s| s.to_string())).collect()
                }
                Some(serde_yaml::Value::Sequence(seq)) => seq
                    .iter()
                    .filter_map(|sh| {
                        if let serde_yaml::Value::Mapping(m) = sh {
                            m.get(&serde_yaml::Value::String("id".into()))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect(),
                _ => HashSet::new(),
            };
            if !shape_ids.is_empty() {
                let validate_edge = |edge_attrs: &serde_yaml::Mapping, findings: &mut Vec<Finding>| {
                    for field in &["source", "target"] {
                        if let Some(serde_yaml::Value::String(ref_str)) =
                            edge_attrs.get(&serde_yaml::Value::String((*field).into()))
                        {
                            if !shape_ids.contains(ref_str.as_str()) {
                                findings.push(warning(
                                    "W403",
                                    &file,
                                    &format!(
                                        "edge `{}` '{}' is not a defined shape id in this diagram",
                                        field, ref_str
                                    ),
                                ));
                            }
                        }
                    }
                };
                match fm.edges.as_ref() {
                    Some(serde_yaml::Value::Mapping(edges_map)) => {
                        for edge_val in edges_map.values() {
                            if let serde_yaml::Value::Mapping(attrs) = edge_val {
                                validate_edge(attrs, &mut findings);
                            }
                        }
                    }
                    Some(serde_yaml::Value::Sequence(edges_seq)) => {
                        for edge_val in edges_seq {
                            if let serde_yaml::Value::Mapping(attrs) = edge_val {
                                validate_edge(attrs, &mut findings);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // E402: companion SVG file must exist on disk
        // All paths are resolved relative to the .md file's parent directory.
        let md_dir = std::path::Path::new(&file)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        if fm.svg_mode.as_deref() == Some("companion") {
            let companion_path = if let Some(ref sf) = fm.svg_file {
                md_dir.join(sf)
            } else {
                // Default: same stem as the .md file, .svg extension
                std::path::Path::new(&file).with_extension("svg")
            };
            if !companion_path.exists() {
                findings.push(error(
                    "E402",
                    &file,
                    &format!("companion SVG file '{}' does not exist on disk", companion_path.display()),
                ));
            }
        } else if let Some(ref svg_file) = fm.svg_file {
            // svgFile set without svgMode: companion — still validate existence
            if !md_dir.join(svg_file).exists() {
                findings.push(error(
                    "E402",
                    &file,
                    &format!("`svgFile` '{}' does not exist on disk", svg_file),
                ));
            }
        }

        // W405: body must be consistent with svgMode
        if let Some(ref mode) = fm.svg_mode {
            match mode.as_str() {
                "companion" => {
                    if !elem.doc.contains("<img") {
                        findings.push(warning(
                            "W405",
                            &file,
                            "`svgMode: companion` but body contains no `<img` tag pointing to the SVG file",
                        ));
                    }
                }
                "inline" => {
                    if !elem.doc.contains("```svg") {
                        findings.push(warning(
                            "W405",
                            &file,
                            "`svgMode: inline` but body contains no fenced ```svg block",
                        ));
                    }
                }
                _ => {}
            }
        }

        // W406/W407: SVG id consistency — frontmatter shape/edge ids vs inline SVG
        // Only checked for inline mode (companion SVG is not loaded by the validator)
        if fm.svg_mode.as_deref().unwrap_or("inline") == "inline" {
            // Collect ids declared in shapes: and edges: frontmatter
            let fm_ids: HashSet<String> = {
                let mut ids = HashSet::new();
                let collect_map_keys = |map: &serde_yaml::Mapping, ids: &mut HashSet<String>| {
                    for k in map.keys() {
                        if let Some(s) = k.as_str() {
                            ids.insert(s.to_string());
                        }
                    }
                };
                let collect_seq_ids = |seq: &[serde_yaml::Value], ids: &mut HashSet<String>| {
                    for v in seq {
                        if let serde_yaml::Value::Mapping(m) = v {
                            if let Some(serde_yaml::Value::String(id)) =
                                m.get(&serde_yaml::Value::String("id".into()))
                            {
                                ids.insert(id.clone());
                            }
                        }
                    }
                };
                if let Some(s) = &fm.shapes {
                    match s {
                        serde_yaml::Value::Mapping(m) => collect_map_keys(m, &mut ids),
                        serde_yaml::Value::Sequence(seq) => collect_seq_ids(seq, &mut ids),
                        _ => {}
                    }
                }
                if let Some(e) = &fm.edges {
                    match e {
                        serde_yaml::Value::Mapping(m) => collect_map_keys(m, &mut ids),
                        serde_yaml::Value::Sequence(seq) => collect_seq_ids(seq, &mut ids),
                        _ => {}
                    }
                }
                ids
            };

            if !fm_ids.is_empty() || elem.doc.contains("```svg") {
                // Extract id="..." values from the inline SVG block
                let svg_ids: HashSet<String> = {
                    let mut ids = HashSet::new();
                    let mut remaining = elem.doc.as_str();
                    while let Some(pos) = remaining.find("id=\"") {
                        remaining = &remaining[pos + 4..];
                        if let Some(end) = remaining.find('"') {
                            ids.insert(remaining[..end].to_string());
                            remaining = &remaining[end + 1..];
                        } else {
                            break;
                        }
                    }
                    ids
                };

                // W406: frontmatter id with no matching SVG element
                for id in &fm_ids {
                    if !svg_ids.contains(id.as_str()) {
                        findings.push(warning(
                            "W406",
                            &file,
                            &format!("frontmatter shape/edge id '{}' has no matching `id` attribute in the inline SVG", id),
                        ));
                    }
                }
                // W407: SVG element id with no matching frontmatter entry
                for id in &svg_ids {
                    if !fm_ids.contains(id.as_str()) {
                        findings.push(warning(
                            "W407",
                            &file,
                            &format!("SVG element id '{}' has no matching entry in frontmatter `shapes`/`edges`", id),
                        ));
                    }
                }
            }
        }

        // ── Allocation cross-reference checks (E5xx) ─────────────────────────

        // E500/E501: features with type: Allocation must have resolvable allocatedFrom/allocatedTo
        if let Some(ref feats) = fm.features {
            for feat_val in feats {
                if let serde_yaml::Value::Mapping(ref feat) = *feat_val {
                    let feat_type = feat
                        .get(&serde_yaml::Value::String("type".into()))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if feat_type == "Allocation" {
                        if let Some(serde_yaml::Value::String(ref from_str)) =
                            feat.get(&serde_yaml::Value::String("allocatedFrom".into()))
                        {
                            if resolver.resolve_ref(elements, from_str).is_none() {
                                findings.push(error(
                                    "E500",
                                    &file,
                                    &format!("Allocation feature `allocatedFrom` '{}' does not resolve", from_str),
                                ));
                            }
                        }
                        if let Some(serde_yaml::Value::String(ref to_str)) =
                            feat.get(&serde_yaml::Value::String("allocatedTo".into()))
                        {
                            if resolver.resolve_ref(elements, to_str).is_none() {
                                findings.push(error(
                                    "E501",
                                    &file,
                                    &format!("Allocation feature `allocatedTo` '{}' does not resolve", to_str),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // E502/E503: top-level allocatedFrom/allocatedTo on Allocation elements must resolve
        if matches!(fm.element_type, Some(ElementType::Allocation)) {
            if let Some(ref af) = fm.allocated_from {
                if resolver.resolve_ref(elements, af).is_none() {
                    findings.push(error(
                        "E502",
                        &file,
                        &format!("`allocatedFrom` '{}' does not resolve to a known element", af),
                    ));
                }
            }
            if let Some(ref at_ref) = fm.allocated_to {
                if resolver.resolve_ref(elements, at_ref).is_none() {
                    findings.push(error(
                        "E503",
                        &file,
                        &format!("`allocatedTo` '{}' does not resolve to a known element", at_ref),
                    ));
                }
            }
        }

        // ── Structural cross-reference warnings (W5xx) ───────────────────────

        // W500: viewpoint on View must resolve to a ViewpointDef
        if matches!(fm.element_type, Some(ElementType::View)) {
            if let Some(ref vp) = fm.viewpoint {
                match resolver.resolve_ref(elements, vp) {
                    None => findings.push(warning(
                        "W500",
                        &file,
                        &format!("`viewpoint` '{}' does not resolve to any element", vp),
                    )),
                    Some(target)
                        if !matches!(
                            target.frontmatter.element_type,
                            Some(ElementType::ViewpointDef)
                        ) =>
                    {
                        findings.push(warning(
                            "W500",
                            &file,
                            &format!("`viewpoint` '{}' does not resolve to a ViewpointDef", vp),
                        ));
                    }
                    _ => {}
                }
            }
        }

        // W501: exhibitsStates entries must resolve to known elements
        if let Some(ref states) = fm.exhibits_states {
            for st in states {
                if resolver.resolve_ref(elements, st).is_none() {
                    findings.push(warning(
                        "W501",
                        &file,
                        &format!("`exhibitsStates` entry '{}' does not resolve to any known element", st),
                    ));
                }
            }
        }

        // W502: expose entries on View must resolve to known elements
        if matches!(fm.element_type, Some(ElementType::View)) {
            if let Some(ref expose_vals) = fm.expose {
                for exp_val in expose_vals {
                    let ref_str = match exp_val {
                        serde_yaml::Value::String(s) => Some(s.as_str()),
                        serde_yaml::Value::Mapping(map) => map
                            .get(&serde_yaml::Value::String("ref".into()))
                            .and_then(|v| v.as_str()),
                        _ => None,
                    };
                    if let Some(r) = ref_str {
                        if resolver.resolve_ref(elements, r).is_none() {
                            findings.push(warning(
                                "W502",
                                &file,
                                &format!("`expose` entry '{}' does not resolve to any known element", r),
                            ));
                        }
                    }
                }
            }
        }

        // W404: operation parameter typedBy / returnType doesn't resolve to a known element
        if let Some(ref ops) = fm.operations {
            for op_val in ops {
                if let serde_yaml::Value::Mapping(ref op) = *op_val {
                    if let Some(serde_yaml::Value::Sequence(ref params)) =
                        op.get(&serde_yaml::Value::String("parameters".into()))
                    {
                        for param_val in params {
                            if let serde_yaml::Value::Mapping(ref param) = *param_val {
                                if let Some(serde_yaml::Value::String(ref typed_by)) =
                                    param.get(&serde_yaml::Value::String("typedBy".into()))
                                {
                                    if resolver.resolve_ref(elements, typed_by).is_none() {
                                        findings.push(warning(
                                            "W404",
                                            &file,
                                            &format!(
                                                "operation parameter `typedBy` '{}' does not resolve to a known element",
                                                typed_by
                                            ),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    // also check returnType
                    if let Some(serde_yaml::Value::String(ref ret)) =
                        op.get(&serde_yaml::Value::String("returnType".into()))
                    {
                        if resolver.resolve_ref(elements, ret).is_none() {
                            findings.push(warning(
                                "W404",
                                &file,
                                &format!(
                                    "operation `returnType` '{}' does not resolve to a known element",
                                    ret
                                ),
                            ));
                        }
                    }
                }
            }
        }

        // ── Documentation completeness (W6xx) ─────────────────────────────────

        // W600: PartDef and Part elements should have non-empty documentation
        if matches!(
            fm.element_type,
            Some(ElementType::PartDef) | Some(ElementType::Part)
        ) && elem.doc.trim().is_empty()
        {
            findings.push(warning("W600", &file, "PartDef/Part has an empty documentation body"));
        }

        // W601: ActionDef and Action elements should have non-empty documentation
        if matches!(
            fm.element_type,
            Some(ElementType::ActionDef) | Some(ElementType::Action)
        ) && elem.doc.trim().is_empty()
        {
            findings.push(warning("W601", &file, "ActionDef/Action has an empty documentation body"));
        }
    }

    // ── Model-time checks (cross-element) ────────────────────────────────────

    // E101: duplicate id
    {
        let mut seen_ids: HashMap<&str, &str> = HashMap::new();
        for elem in elements {
            if let Some(ref id) = elem.frontmatter.id {
                if let Some(prev_file) = seen_ids.insert(id.as_str(), elem.file_path.as_str()) {
                    findings.push(error(
                        "E101",
                        &elem.file_path,
                        &format!("duplicate id '{}' (first seen in {})", id, prev_file),
                    ));
                }
            }
        }
    }

    // Build verified_by and derived_children reverse indices, and check E102–E105
    let mut verified_by: HashMap<String, Vec<String>> = HashMap::new();
    let mut derived_children: HashMap<String, Vec<String>> = HashMap::new();

    for elem in elements {
        let fm = &elem.frontmatter;

        // verifies: cross-reference check
        if let Some(ref vs) = fm.verifies {
            for v in vs {
                match resolver.resolve_ref(elements, v) {
                    None => findings.push(error(
                        "E102",
                        &elem.file_path,
                        &format!("unresolved verifies reference '{}'", v),
                    )),
                    Some(target) => {
                        // E104: target must be a native Requirement
                        if !Resolver::is_native_requirement(target) {
                            findings.push(error(
                                "E104",
                                &elem.file_path,
                                &format!("'{}' does not resolve to a native Requirement", v),
                            ));
                        } else if let Some(ref req_id) = target.frontmatter.id {
                            // Build reverse index
                            if let Some(ref tc_id) = elem.frontmatter.id {
                                verified_by
                                    .entry(req_id.clone())
                                    .or_default()
                                    .push(tc_id.clone());
                            }
                        }
                    }
                }
            }
        }

        // derivedFrom: cross-reference check
        if let Some(ref dfs) = fm.derived_from {
            for df in dfs {
                match resolver.resolve_ref(elements, df) {
                    None => findings.push(error(
                        "E103",
                        &elem.file_path,
                        &format!("unresolved derivedFrom reference '{}'", df),
                    )),
                    Some(target) => {
                        // E105: target must be a native Requirement
                        if !Resolver::is_native_requirement(target) {
                            findings.push(error(
                                "E105",
                                &elem.file_path,
                                &format!("'{}' does not resolve to a native Requirement", df),
                            ));
                        } else if let Some(ref parent_id) = target.frontmatter.id {
                            if let Some(ref child_id) = elem.frontmatter.id {
                                derived_children
                                    .entry(parent_id.clone())
                                    .or_default()
                                    .push(child_id.clone());
                            }
                        }
                    }
                }
            }
        }

        // E106: testFunctions[].scenario must match a Gherkin scenario title
        if let Some(ref fns) = fm.test_functions {
            let scenarios = extract_gherkin_scenarios(&elem.doc);
            for tf in fns {
                if let Some(serde_yaml::Value::Mapping(map)) = Some(tf) {
                    if let Some(serde_yaml::Value::String(scenario)) =
                        map.get(&serde_yaml::Value::String("scenario".into()))
                    {
                        if !scenarios.contains(scenario.as_str()) {
                            findings.push(error(
                                "E106",
                                &elem.file_path,
                                &format!("testFunctions scenario '{}' not found in Gherkin blocks", scenario),
                            ));
                        }
                    }
                }
            }
        }
    }

    // W002/W003: coverage checks for native Requirements
    for elem in elements {
        if !Resolver::is_native_requirement(elem) {
            continue;
        }
        let req_id = elem.frontmatter.id.as_deref().unwrap_or("");
        let status = elem.frontmatter.status.as_deref().unwrap_or("");
        let active_tcs: Vec<_> = verified_by
            .get(req_id)
            .map(|tcs| {
                tcs.iter()
                    .filter(|tc_id| {
                        resolver
                            .get_by_id(elements, tc_id)
                            .and_then(|e| e.frontmatter.status.as_deref())
                            == Some("active")
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        let is_parent = derived_children.get(req_id).map_or(false, |v| !v.is_empty());
        match status {
            // W002: leaf requirements at approved/implemented need an active TestCase.
            // Parent requirements (those with derivedChildren) are verified by
            // decomposition — all their leaf descendants carry the test coverage —
            // so W002 is suppressed for them.
            "approved" | "implemented" if active_tcs.is_empty() && !is_parent => {
                findings.push(warning(
                    "W002",
                    &elem.file_path,
                    &format!("Requirement '{}' (status: {}) has no active TestCase", req_id, status),
                ));
            }
            "verified" if active_tcs.is_empty() => {
                findings.push(warning(
                    "W003",
                    &elem.file_path,
                    &format!("Requirement '{}' has status: verified but no active TestCase covers it", req_id),
                ));
            }
            _ => {}
        }

        // W005: orphan (no derivedFrom and no derivedChildren)
        let has_parent = elem.frontmatter.derived_from.as_ref().map_or(false, |v| !v.is_empty());
        let has_children = derived_children.get(req_id).map_or(false, |v| !v.is_empty());
        if !has_parent && !has_children {
            findings.push(warning(
                "W005",
                &elem.file_path,
                &format!(
                    "Requirement '{}' has no derivedFrom and no derivedChildren — possible orphan",
                    req_id
                ),
            ));
        }
    }

    // E209: appliesWhen references must resolve to FeatureDef
    for elem in elements {
        if let Some(ref aw) = elem.frontmatter.applies_when {
            let refs = yaml_strings(aw);
            for r in refs {
                match resolver.resolve_ref(elements, r) {
                    None => findings.push(error(
                        "E209",
                        &elem.file_path,
                        &format!("unresolved appliesWhen reference '{}'", r),
                    )),
                    Some(target) if !Resolver::is_feature_def(target) => {
                        findings.push(error(
                            "E209",
                            &elem.file_path,
                            &format!("'{}' does not resolve to a FeatureDef", r),
                        ));
                    }
                    _ => {}
                }
            }
        }
    }

    // ── Traceability checks (§12) ─────────────────────────────────────────────

    // Build reverse index: satisfied_reqs[req_qname_or_id] = list of satisfying element qnames
    let mut satisfied_reqs: HashMap<String, Vec<String>> = HashMap::new();
    for elem in elements {
        if let Some(ref sat) = elem.frontmatter.satisfies {
            for s in sat {
                if let Some(target) = resolver.resolve_ref(elements, s) {
                    satisfied_reqs
                        .entry(target.qualified_name.clone())
                        .or_default()
                        .push(elem.qualified_name.clone());
                }
            }
        }
    }

    for elem in elements {
        let fm = &elem.frontmatter;

        // E310: native Requirement with derivedFrom must have breakdownAdr
        if Resolver::is_native_requirement(elem) {
            if fm.derived_from.as_ref().map_or(false, |v| !v.is_empty()) {
                if fm.breakdown_adr.is_none() {
                    findings.push(error(
                        "E310",
                        &elem.file_path,
                        "Requirement has `derivedFrom` but no `breakdownAdr`",
                    ));
                }
            }
        }

        // E311: breakdownAdr must resolve to an ADR
        if let Some(ref adr_ref) = fm.breakdown_adr {
            match resolver.resolve_ref(elements, adr_ref) {
                None => findings.push(error(
                    "E311",
                    &elem.file_path,
                    &format!("`breakdownAdr` '{}' cannot be resolved", adr_ref),
                )),
                Some(target) if !Resolver::is_adr(target) => {
                    findings.push(error(
                        "E311",
                        &elem.file_path,
                        &format!("`breakdownAdr` '{}' does not resolve to an ADR", adr_ref),
                    ));
                }
                // W303: breakdownAdr references a proposed ADR but requirement is approved or higher
                Some(target) => {
                    let req_status = fm.status.as_deref().unwrap_or("");
                    let adr_status = target.frontmatter.status.as_deref().unwrap_or("");
                    const APPROVED_OR_HIGHER: &[&str] = &["approved", "implemented", "verified"];
                    if adr_status == "proposed" && APPROVED_OR_HIGHER.contains(&req_status) {
                        findings.push(warning(
                            "W303",
                            &elem.file_path,
                            &format!(
                                "`breakdownAdr` '{}' is still `proposed` but Requirement has status '{}'",
                                adr_ref, req_status
                            ),
                        ));
                    }
                }
            }
        }

        // E312: a parent requirement (has derivedChildren) must not appear in any satisfies list
        if Resolver::is_native_requirement(elem) {
            let req_id = fm.id.as_deref().unwrap_or("");
            let is_parent = derived_children.get(req_id).map_or(false, |c| !c.is_empty());
            if is_parent {
                let qn = &elem.qualified_name;
                let in_satisfies = satisfied_reqs.contains_key(qn.as_str())
                    || (req_id != "" && satisfied_reqs.contains_key(req_id));
                if in_satisfies {
                    findings.push(error(
                        "E312",
                        &elem.file_path,
                        &format!("parent Requirement '{}' appears in a `satisfies:` list — only leaf requirements may be assigned", req_id),
                    ));
                }
            }
        }

        // E313: satisfies domain mismatch — architecture element domain vs requirement reqDomain
        if let Some(ref sat) = fm.satisfies {
            let elem_domain = fm.domain.as_deref().unwrap_or("system");
            for s in sat {
                if let Some(target) = resolver.resolve_ref(elements, s) {
                    if Resolver::is_native_requirement(target) {
                        let req_domain = target.frontmatter.req_domain.as_deref().unwrap_or("system");
                        if elem_domain != "system" && req_domain != "system" && elem_domain != req_domain {
                            findings.push(error(
                                "E313",
                                &elem.file_path,
                                &format!(
                                    "`satisfies` domain mismatch: element has `domain: {}` but requirement '{}' has `reqDomain: {}`",
                                    elem_domain, s, req_domain
                                ),
                            ));
                        }
                    }
                }
            }
        }

        // E315: cross-domain direct supertype/typedBy references
        let elem_domain = fm.domain.as_deref().unwrap_or("system");
        if elem_domain != "system" {
            for field_val in [fm.supertype.as_ref(), fm.typed_by.as_ref()].into_iter().flatten() {
                for r in yaml_strings(field_val) {
                    if let Some(target) = resolver.resolve_ref(elements, r) {
                        let target_domain = target.frontmatter.domain.as_deref().unwrap_or("system");
                        if target_domain != "system" && elem_domain != target_domain {
                            findings.push(error(
                                "E315",
                                &elem.file_path,
                                &format!(
                                    "cross-domain reference: `domain: {}` element references `domain: {}` element '{}' — use Allocation instead",
                                    elem_domain, target_domain, r
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    // E314: deployment packages must have at least one Allocation to a hardware element
    {
        // Build a set of (allocateFrom qname) → target domain for all Allocation elements
        let mut hw_alloc_targets: HashSet<String> = HashSet::new();
        for elem in elements {
            if !matches!(elem.frontmatter.element_type, Some(ElementType::Allocation)) {
                continue;
            }
            // allocated_from is the software side; allocated_to is the hardware side
            if let Some(ref to_ref) = elem.frontmatter.allocated_to {
                if let Some(target) = resolver.get(elements, to_ref) {
                    if target.frontmatter.domain.as_deref() == Some("hardware") {
                        if let Some(ref from_ref) = elem.frontmatter.allocated_from {
                            hw_alloc_targets.insert(from_ref.clone());
                        }
                    }
                }
            }
        }
        for elem in elements {
            if elem.frontmatter.is_deployment_package == Some(true) {
                if !hw_alloc_targets.contains(&elem.qualified_name) {
                    findings.push(error(
                        "E314",
                        &elem.file_path,
                        &format!(
                            "`isDeploymentPackage: true` element '{}' has no Allocation to a hardware element",
                            elem.qualified_name
                        ),
                    ));
                }
            }
        }
    }

    // W300/W301: leaf requirement coverage by satisfying architecture elements
    for elem in elements {
        if !Resolver::is_native_requirement(elem) {
            continue;
        }
        let req_id = elem.frontmatter.id.as_deref().unwrap_or("");
        let is_parent = derived_children.get(req_id).map_or(false, |c| !c.is_empty());
        if is_parent {
            continue; // only check leaf requirements
        }
        let status = elem.frontmatter.status.as_deref().unwrap_or("");
        let satisfiers = satisfied_reqs.get(&elem.qualified_name).map(|v| v.len()).unwrap_or(0);

        if matches!(status, "approved" | "implemented") && satisfiers == 0 {
            findings.push(warning(
                "W300",
                &elem.file_path,
                &format!("leaf Requirement '{}' (status: {}) has no satisfying architecture element", req_id, status),
            ));
        } else if satisfiers > 1 {
            findings.push(warning(
                "W301",
                &elem.file_path,
                &format!("leaf Requirement '{}' is satisfied by {} elements — only one expected", req_id, satisfiers),
            ));
        }

        // W302: leaf requirement still has reqDomain: system at implemented/verified
        if matches!(status, "implemented" | "verified") {
            let req_domain = elem.frontmatter.req_domain.as_deref().unwrap_or("system");
            if req_domain == "system" {
                findings.push(warning(
                    "W302",
                    &elem.file_path,
                    &format!("leaf Requirement '{}' (status: {}) still has `reqDomain: system` — refine to `hardware` or `software`", req_id, status),
                ));
            }
        }
    }

    ValidationResult {
        findings,
        verified_by,
        derived_children,
    }
}

fn error(code: &'static str, file: &str, msg: &str) -> Finding {
    Finding { code, file: file.to_string(), message: msg.to_string(), severity: Severity::Error }
}

fn warning(code: &'static str, file: &str, msg: &str) -> Finding {
    Finding { code, file: file.to_string(), message: msg.to_string(), severity: Severity::Warning }
}

/// Extract the normative text: everything before the first `##` heading.
fn normative_text(doc: &str) -> &str {
    doc.find("\n## ")
        .or_else(|| doc.find("\n# "))
        .map(|pos| &doc[..pos])
        .unwrap_or(doc)
}

/// Extract all scenario titles (Scenario: / Scenario Outline:) from Gherkin blocks.
fn extract_gherkin_scenarios(doc: &str) -> HashSet<&str> {
    let mut titles = HashSet::new();
    let mut in_gherkin = false;
    for line in doc.lines() {
        let trimmed = line.trim();
        if trimmed == "```gherkin" {
            in_gherkin = true;
            continue;
        }
        if in_gherkin && trimmed == "```" {
            in_gherkin = false;
            continue;
        }
        if in_gherkin {
            if let Some(rest) = trimmed.strip_prefix("Scenario:").or_else(|| {
                trimmed
                    .strip_prefix("Scenario Outline:")
                    .or_else(|| trimmed.strip_prefix("Scenario outline:"))
            }) {
                titles.insert(rest.trim());
            }
        }
    }
    titles
}

fn check_scenario_outline_has_examples(doc: &str, file: &str, findings: &mut Vec<Finding>) {
    let mut in_gherkin = false;
    let mut in_outline = false;
    for line in doc.lines() {
        let trimmed = line.trim();
        if trimmed == "```gherkin" {
            in_gherkin = true;
            continue;
        }
        if in_gherkin && trimmed == "```" {
            in_gherkin = false;
            in_outline = false;
            continue;
        }
        if in_gherkin {
            if trimmed.starts_with("Scenario Outline:") || trimmed.starts_with("Scenario outline:") {
                in_outline = true;
            } else if trimmed.starts_with("Examples:") {
                in_outline = false;
            } else if in_outline
                && (trimmed.starts_with("Scenario:")
                    || trimmed.starts_with("Scenario Outline:")
                    || trimmed == "```")
            {
                findings.push(error("E014", file, "Scenario Outline has no Examples: table"));
                in_outline = false;
            }
        }
    }
    if in_outline {
        findings.push(error("E014", file, "Scenario Outline has no Examples: table"));
    }
}

fn first_gherkin_has_feature(doc: &str) -> bool {
    let mut in_first = false;
    let mut found = false;
    for line in doc.lines() {
        let trimmed = line.trim();
        if !in_first && trimmed == "```gherkin" {
            in_first = true;
            continue;
        }
        if in_first {
            if trimmed == "```" {
                break;
            }
            if trimmed.starts_with("Feature:") {
                found = true;
                break;
            }
        }
    }
    !in_first || found // if no gherkin block, E011 will fire; don't double-report
}
