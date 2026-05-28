use std::collections::{HashMap, HashSet};
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;
use crate::element::{ElementType, RawElement};
use crate::graph::EdgeKind;
use crate::resolver::{
    is_adr_id, is_conf_id, is_csg_id, is_ds_id, is_fm_id, is_fmea_id, is_ft_id, is_fte_id,
    is_ftg_id, is_he_id, is_req_id, is_sc_id, is_sg_id, is_tara_id, is_tc_id, is_ts_id,
    is_vr_id, Resolver,
};

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

/// Resolve a relative `href` path against a base directory into a normalised path string.
/// Handles `..` and `.` segments without touching the filesystem.
fn normalize_relative_path(base_dir: &str, href: &str) -> String {
    use std::path::Component;
    let combined = std::path::Path::new(base_dir).join(href);
    let mut parts: Vec<String> = Vec::new();
    for component in combined.components() {
        match component {
            Component::ParentDir => { parts.pop(); }
            Component::CurDir => {}
            Component::Normal(s) => parts.push(s.to_string_lossy().into_owned()),
            Component::RootDir => parts.clear(),
            Component::Prefix(_) => {}
        }
    }
    parts.join("/")
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

        // E019: dalLevel A–E
        if let Some(ref dal) = fm.dal_level {
            const DAL: &[&str] = &["A", "B", "C", "D", "E"];
            if !DAL.contains(&dal.as_str()) {
                findings.push(error("E019", &file, &format!("unknown dalLevel '{}' — must be A, B, C, D, or E", dal)));
            }
        }

        // E020: verificationMethod enum
        if let Some(ref vm) = fm.verification_method {
            const METHODS: &[&str] = &["test", "inspection", "analysis", "demonstration"];
            if !METHODS.contains(&vm.as_str()) {
                findings.push(error("E020", &file, &format!("unknown verificationMethod '{}' — must be test, inspection, analysis, or demonstration", vm)));
            }
        }

        // E021: coverageTarget enum
        if let Some(ref ct) = fm.coverage_target {
            const TARGETS: &[&str] = &["statement", "branch", "MCDC"];
            if !TARGETS.contains(&ct.as_str()) {
                findings.push(error("E021", &file, &format!("unknown coverageTarget '{}' — must be statement, branch, or MCDC", ct)));
            }
        }

        // E022: requirementKind enum
        if let Some(ref rk) = fm.requirement_kind {
            const KINDS: &[&str] = &["stakeholder", "system", "software", "hardware"];
            if !KINDS.contains(&rk.as_str()) {
                findings.push(error("E022", &file, &format!("unknown requirementKind '{}' — must be stakeholder, system, software, or hardware", rk)));
            }
        }

        // W701: Requirement with asilLevel B/C/D should have verificationMethod
        if let Some(ElementType::Requirement) = &fm.element_type {
            if let Some(ref asil) = fm.asil_level {
                if matches!(asil.as_str(), "B" | "C" | "D") && fm.verification_method.is_none() {
                    findings.push(warning(
                        "W701",
                        &file,
                        &format!("Requirement with asilLevel: {} has no verificationMethod — add test, inspection, analysis, or demonstration", asil),
                    ));
                }
            }
        }

        // W807: security requirement (derivedFromSecurityGoal set) should have verificationMethod
        if matches!(fm.element_type, Some(ElementType::Requirement))
            && fm.derived_from_security_goal.is_some()
            && fm.verification_method.is_none()
        {
            findings.push(warning(
                "W807",
                &file,
                "security Requirement (derivedFromSecurityGoal set) has no verificationMethod — add test, inspection, analysis, or demonstration",
            ));
        }

        // W703: asilLevel and dalLevel both present — these are different standards
        if fm.asil_level.is_some() && fm.dal_level.is_some() {
            findings.push(warning(
                "W703",
                &file,
                "both asilLevel (ISO 26262) and dalLevel (DO-178C) are set — these are different standards; validate under one or document the mapping",
            ));
        }

        // ── Tier 2: HazardousEvent (E800-E804) ───────────────────────────────
        if matches!(fm.element_type, Some(ElementType::HazardousEvent)) {
            // E800: required fields
            if fm.id.is_none() { findings.push(error("E800", &file, "`id` is required on HazardousEvent")); }
            if fm.title.is_none() { findings.push(error("E800", &file, "`title` is required on HazardousEvent")); }
            if fm.status.is_none() { findings.push(error("E800", &file, "`status` is required on HazardousEvent")); }
            // E804: id pattern
            if let Some(ref id) = fm.id {
                if !is_he_id(id) {
                    findings.push(error("E804", &file, &format!("`id` '{}' does not match HE-* pattern", id)));
                }
            }
            // E801: severity S0-S3
            if let Some(ref s) = fm.severity {
                if !["S0","S1","S2","S3"].contains(&s.as_str()) {
                    findings.push(error("E801", &file, &format!("HazardousEvent.severity '{}' must be S0, S1, S2, or S3", s)));
                }
            }
            // E802: exposure E0-E4
            if let Some(ref e) = fm.exposure {
                if !["E0","E1","E2","E3","E4"].contains(&e.as_str()) {
                    findings.push(error("E802", &file, &format!("HazardousEvent.exposure '{}' must be E0–E4", e)));
                }
            }
            // E803: controllability C0-C3
            if let Some(ref c) = fm.controllability {
                if !["C0","C1","C2","C3"].contains(&c.as_str()) {
                    findings.push(error("E803", &file, &format!("HazardousEvent.controllability '{}' must be C0, C1, C2, or C3", c)));
                }
            }
            // E833: IEC 61508 consequence Ca-Cd
            if let Some(ref c) = fm.consequence {
                if !["Ca","Cb","Cc","Cd"].contains(&c.as_str()) {
                    findings.push(error("E833", &file, &format!("HazardousEvent.consequence '{}' must be Ca, Cb, Cc, or Cd (IEC 61508 risk graph)", c)));
                }
            }
            // E834: IEC 61508 freqExposure Fa/Fb
            if let Some(ref fe) = fm.freq_exposure {
                if !["Fa","Fb"].contains(&fe.as_str()) {
                    findings.push(error("E834", &file, &format!("HazardousEvent.freqExposure '{}' must be Fa or Fb (IEC 61508 risk graph)", fe)));
                }
            }
            // E835: IEC 61508 avoidance Pa/Pb
            if let Some(ref av) = fm.avoidance {
                if !["Pa","Pb"].contains(&av.as_str()) {
                    findings.push(error("E835", &file, &format!("HazardousEvent.avoidance '{}' must be Pa or Pb (IEC 61508 risk graph)", av)));
                }
            }
            // E836: IEC 61508 demandRate W1-W3
            if let Some(ref dr) = fm.demand_rate {
                if !["W1","W2","W3"].contains(&dr.as_str()) {
                    findings.push(error("E836", &file, &format!("HazardousEvent.demandRate '{}' must be W1, W2, or W3 (IEC 61508 risk graph)", dr)));
                }
            }
        }

        // ── Tier 2: SafetyGoal (E805-E806, E837) ─────────────────────────────
        if matches!(fm.element_type, Some(ElementType::SafetyGoal)) {
            if fm.id.is_none() { findings.push(error("E805", &file, "`id` is required on SafetyGoal")); }
            if fm.title.is_none() { findings.push(error("E805", &file, "`title` is required on SafetyGoal")); }
            if fm.status.is_none() { findings.push(error("E805", &file, "`status` is required on SafetyGoal")); }
            if let Some(ref id) = fm.id {
                if !is_sg_id(id) {
                    findings.push(error("E806", &file, &format!("`id` '{}' does not match SG-* pattern", id)));
                }
            }
            // E837: plLevel enum (ISO 13849-1)
            if let Some(ref pl) = fm.pl_level {
                if !["a","b","c","d","e"].contains(&pl.as_str()) {
                    findings.push(error("E837", &file, &format!("SafetyGoal.plLevel '{}' must be a, b, c, d, or e (ISO 13849-1)", pl)));
                }
            }
            // W801: SafetyGoal should carry an integrity level (asilLevel, silLevel, or plLevel)
            if fm.asil_level.is_none() && fm.sil_level.is_none() && fm.pl_level.is_none() {
                findings.push(warning("W801", &file, "SafetyGoal has no integrity level — set asilLevel (ISO 26262), silLevel (IEC 61508), or plLevel (ISO 13849-1)"));
            }
        }

        // ── Tier 2: DamageScenario (E807-E810) ───────────────────────────────
        if matches!(fm.element_type, Some(ElementType::DamageScenario)) {
            if fm.id.is_none() { findings.push(error("E807", &file, "`id` is required on DamageScenario")); }
            if fm.title.is_none() { findings.push(error("E807", &file, "`title` is required on DamageScenario")); }
            if fm.status.is_none() { findings.push(error("E807", &file, "`status` is required on DamageScenario")); }
            if let Some(ref id) = fm.id {
                if !is_ds_id(id) {
                    findings.push(error("E808", &file, &format!("`id` '{}' does not match DS-* pattern", id)));
                }
            }
            // E809: damageSeverity enum
            if let Some(ref s) = fm.damage_severity {
                if !["severe","major","moderate","negligible"].contains(&s.as_str()) {
                    findings.push(error("E809", &file, &format!("DamageScenario.damageSeverity '{}' must be severe, major, moderate, or negligible", s)));
                }
            }
            // E810: impactCategories enum
            if let Some(ref cats) = fm.impact_categories {
                for cat in cats {
                    if !["safety","financial","operational","privacy"].contains(&cat.as_str()) {
                        findings.push(error("E810", &file, &format!("DamageScenario.impactCategories '{}' must be safety, financial, operational, or privacy", cat)));
                    }
                }
            }
        }

        // ── Tier 2: ThreatScenario (E811-E814) ───────────────────────────────
        if matches!(fm.element_type, Some(ElementType::ThreatScenario)) {
            if fm.id.is_none() { findings.push(error("E811", &file, "`id` is required on ThreatScenario")); }
            if fm.title.is_none() { findings.push(error("E811", &file, "`title` is required on ThreatScenario")); }
            if fm.status.is_none() { findings.push(error("E811", &file, "`status` is required on ThreatScenario")); }
            if let Some(ref id) = fm.id {
                if !is_ts_id(id) {
                    findings.push(error("E812", &file, &format!("`id` '{}' does not match TS-* pattern", id)));
                }
            }
            // E813: attackFeasibility enum
            if let Some(ref f) = fm.attack_feasibility {
                if !["high","medium","low","very_low"].contains(&f.as_str()) {
                    findings.push(error("E813", &file, &format!("ThreatScenario.attackFeasibility '{}' must be high, medium, low, or very_low", f)));
                }
            }
            // E814: attackVector enum
            if let Some(ref v) = fm.attack_vector {
                if !["network","adjacent","local","physical"].contains(&v.as_str()) {
                    findings.push(error("E814", &file, &format!("ThreatScenario.attackVector '{}' must be network, adjacent, local, or physical", v)));
                }
            }
        }

        // ── Tier 2: CybersecurityGoal (E815-E818) ────────────────────────────
        if matches!(fm.element_type, Some(ElementType::CybersecurityGoal)) {
            if fm.id.is_none() { findings.push(error("E815", &file, "`id` is required on CybersecurityGoal")); }
            if fm.title.is_none() { findings.push(error("E815", &file, "`title` is required on CybersecurityGoal")); }
            if fm.status.is_none() { findings.push(error("E815", &file, "`status` is required on CybersecurityGoal")); }
            if let Some(ref id) = fm.id {
                if !is_csg_id(id) {
                    findings.push(error("E816", &file, &format!("`id` '{}' does not match CSG-* pattern", id)));
                }
            }
            // E817: securityProperty enum
            if let Some(ref sp) = fm.security_property {
                if !["confidentiality","integrity","availability","authenticity"].contains(&sp.as_str()) {
                    findings.push(error("E817", &file, &format!("CybersecurityGoal.securityProperty '{}' must be confidentiality, integrity, availability, or authenticity", sp)));
                }
            }
            // E818: calLevel enum
            if let Some(ref cl) = fm.cal_level {
                if !["CAL1","CAL2","CAL3","CAL4"].contains(&cl.as_str()) {
                    findings.push(error("E818", &file, &format!("CybersecurityGoal.calLevel '{}' must be CAL1, CAL2, CAL3, or CAL4", cl)));
                }
            }
        }

        // ── Tier 2: SecurityControl (E819-E821) ──────────────────────────────
        if matches!(fm.element_type, Some(ElementType::SecurityControl)) {
            if fm.id.is_none() { findings.push(error("E819", &file, "`id` is required on SecurityControl")); }
            if fm.title.is_none() { findings.push(error("E819", &file, "`title` is required on SecurityControl")); }
            if fm.status.is_none() { findings.push(error("E819", &file, "`status` is required on SecurityControl")); }
            if let Some(ref id) = fm.id {
                if !is_sc_id(id) {
                    findings.push(error("E820", &file, &format!("`id` '{}' does not match SC-* pattern", id)));
                }
            }
            // E821: controlType enum
            if let Some(ref ct) = fm.control_type {
                if !["prevention","detection","response","recovery"].contains(&ct.as_str()) {
                    findings.push(error("E821", &file, &format!("SecurityControl.controlType '{}' must be prevention, detection, response, or recovery", ct)));
                }
            }
        }

        // ── Tier 2: VulnerabilityReport (E822-E824) ──────────────────────────
        if matches!(fm.element_type, Some(ElementType::VulnerabilityReport)) {
            if fm.id.is_none() { findings.push(error("E822", &file, "`id` is required on VulnerabilityReport")); }
            if fm.title.is_none() { findings.push(error("E822", &file, "`title` is required on VulnerabilityReport")); }
            if fm.status.is_none() { findings.push(error("E822", &file, "`status` is required on VulnerabilityReport")); }
            if let Some(ref id) = fm.id {
                if !is_vr_id(id) {
                    findings.push(error("E823", &file, &format!("`id` '{}' does not match VR-* pattern", id)));
                }
            }
            // E824: cvssScore 0.0-10.0
            if let Some(score) = fm.cvss_score {
                if !(0.0..=10.0).contains(&score) {
                    findings.push(error("E824", &file, &format!("VulnerabilityReport.cvssScore {} is out of range 0.0–10.0", score)));
                }
            }
            // W803: open vulnerability reports draw attention
            if fm.status.as_deref() == Some("open") {
                findings.push(warning("W803", &file, "VulnerabilityReport has status: open — ensure it is being tracked and mitigated"));
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

        // W006: silLevel and asilLevel both set — incompatible standards
        if fm.sil_level.is_some() && fm.asil_level.is_some() {
            findings.push(warning("W006", &file,
                "both silLevel (IEC 61508) and asilLevel (ISO 26262) are set — these are incompatible standards; use only one"));
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
            // Suppressed for companion SVGs: svgMode: companion already specifies how to display the diagram.
            if fm.diagram_kind.is_none() && fm.svg_mode.as_deref() != Some("companion") {
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
            // W408–W410: validate %% annotations inside Mermaid blocks.
            //   W408: `%% ref: QN` — QN doesn't resolve
            //   W409: no `%% ref:` annotations at all
            //   W410: `%% link: NodeId QN` — QN doesn't resolve
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
                        } else if let Some(rest) = trimmed.strip_prefix("%% link:") {
                            // Format: %% link: NodeId QualifiedName
                            let qn = rest.trim().splitn(2, ' ').nth(1).map(|s| s.trim()).unwrap_or("");
                            if !qn.is_empty() && resolver.resolve_ref(elements, qn).is_none() {
                                findings.push(warning(
                                    "W410",
                                    &file,
                                    &format!("Mermaid `%% link:` '{}' does not resolve to a known element", qn),
                                ));
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
            // W411: shapes `link:` must resolve to a known element.
            // Accepts `link: QualifiedName` (string) or `link: true` (reuses the shape's ref: value).
            // W412: href="..." attributes found directly in an SVG body must resolve to model elements.
            // Both prevent links rotting silently when elements are renamed or deleted.
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
            let validate_shape_link = |attrs: &serde_yaml::Mapping, findings: &mut Vec<Finding>| {
                let link_qn: Option<&str> = match attrs.get(&serde_yaml::Value::String("link".into())) {
                    Some(serde_yaml::Value::String(s)) if !s.is_empty() => Some(s.as_str()),
                    Some(serde_yaml::Value::Bool(true)) => attrs
                        .get(&serde_yaml::Value::String("ref".into()))
                        .and_then(|v| v.as_str()),
                    _ => None,
                };
                if let Some(qn) = link_qn {
                    if resolver.resolve_ref(elements, qn).is_none() {
                        findings.push(warning(
                            "W411",
                            &file,
                            &format!("shapes `link` '{}' does not resolve to a known element", qn),
                        ));
                    }
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
                            validate_shape_link(attrs, &mut findings);
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
                            validate_shape_link(attrs, &mut findings);
                        }
                    }
                }
                _ => {}
            }
            // W412: href="..." in the SVG fenced block must resolve to a known model element.
            // Only relative paths (not http/https/# anchors) are checked.
            if elem.doc.contains("```svg") {
                let svg_block = elem.doc.find("```svg").and_then(|start| {
                    let after = start + "```svg".len();
                    elem.doc[after..].find("```").map(|end| &elem.doc[after..after + end])
                });
                if let Some(svg) = svg_block {
                    let diagram_dir = std::path::Path::new(&file)
                        .parent()
                        .unwrap_or(std::path::Path::new("."))
                        .to_string_lossy()
                        .into_owned();
                    let href_re = regex::Regex::new(r#"href="([^"]+)""#).unwrap();
                    for cap in href_re.captures_iter(svg) {
                        let href = &cap[1];
                        // Skip external and anchor-only links
                        if href.starts_with("http://")
                            || href.starts_with("https://")
                            || href.starts_with('#')
                            || href.starts_with('/')
                        {
                            continue;
                        }
                        let resolved = normalize_relative_path(&diagram_dir, href);
                        if !elements.iter().any(|e| e.file_path == resolved) {
                            findings.push(warning(
                                "W412",
                                &file,
                                &format!("SVG `href` '{}' (resolved: '{}') does not match any model element file", href, resolved),
                            ));
                        }
                    }
                }
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

        // E502/E503: allocatedFrom/allocatedTo must each resolve on any element that sets them
        if let Some(ref afs) = fm.allocated_from {
            for af in afs {
                if resolver.resolve_ref(elements, af).is_none() {
                    findings.push(error(
                        "E502",
                        &file,
                        &format!("`allocatedFrom` '{}' does not resolve to a known element", af),
                    ));
                }
            }
        }
        if let Some(ref ats) = fm.allocated_to {
            for at_ref in ats {
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

        // ── Tier 4: FaultTree (E900-E902) ────────────────────────────────────
        if matches!(fm.element_type, Some(ElementType::FaultTree)) {
            if fm.id.is_none() { findings.push(error("E900", &file, "`id` is required on FaultTree")); }
            if fm.title.is_none() { findings.push(error("E900", &file, "`title` is required on FaultTree")); }
            if fm.status.is_none() { findings.push(error("E900", &file, "`status` is required on FaultTree")); }
            if fm.top_event.is_none() { findings.push(error("E900", &file, "`topEvent` is required on FaultTree — reference a SafetyGoal")); }
            if let Some(ref id) = fm.id {
                if !is_ft_id(id) {
                    findings.push(error("E901", &file, &format!("`id` '{}' does not match FT-* pattern", id)));
                }
            }
        }

        // ── Tier 4: FaultTreeGate (E903-E906) ────────────────────────────────
        if matches!(fm.element_type, Some(ElementType::FaultTreeGate)) {
            if fm.id.is_none() { findings.push(error("E903", &file, "`id` is required on FaultTreeGate")); }
            if fm.title.is_none() { findings.push(error("E903", &file, "`title` is required on FaultTreeGate")); }
            if fm.gate_type.is_none() { findings.push(error("E903", &file, "`gateType` is required on FaultTreeGate")); }
            if let Some(ref id) = fm.id {
                if !is_ftg_id(id) {
                    findings.push(error("E904", &file, &format!("`id` '{}' does not match FTG-* pattern", id)));
                }
            }
            // E905: gateType enum
            if let Some(ref gt) = fm.gate_type {
                if !["AND","OR","XOR","NOT","inhibit"].contains(&gt.as_str()) {
                    findings.push(error("E905", &file, &format!("FaultTreeGate.gateType '{}' must be AND, OR, XOR, NOT, or inhibit", gt)));
                }
            }
            // W901: gate with no inputs is a dead end
            if fm.inputs.as_ref().map_or(true, |v| v.is_empty()) {
                findings.push(warning("W901", &file, "FaultTreeGate has no `inputs` — it contributes nothing to the fault tree"));
            }
        }

        // ── Tier 4: FaultTreeEvent (E907-E910) ───────────────────────────────
        if matches!(fm.element_type, Some(ElementType::FaultTreeEvent)) {
            if fm.id.is_none() { findings.push(error("E907", &file, "`id` is required on FaultTreeEvent")); }
            if fm.title.is_none() { findings.push(error("E907", &file, "`title` is required on FaultTreeEvent")); }
            if fm.event_kind.is_none() { findings.push(error("E907", &file, "`eventKind` is required on FaultTreeEvent")); }
            if let Some(ref id) = fm.id {
                if !is_fte_id(id) {
                    findings.push(error("E908", &file, &format!("`id` '{}' does not match FTE-* pattern", id)));
                }
            }
            // E909: eventKind enum
            if let Some(ref ek) = fm.event_kind {
                if !["basic","undeveloped","house"].contains(&ek.as_str()) {
                    findings.push(error("E909", &file, &format!("FaultTreeEvent.eventKind '{}' must be basic, undeveloped, or house", ek)));
                }
            }
        }

        // ── Tier 4: FMEASheet (E911-E912) ────────────────────────────────────
        if matches!(fm.element_type, Some(ElementType::FMEASheet)) {
            if fm.id.is_none() { findings.push(error("E911", &file, "`id` is required on FMEASheet")); }
            if fm.title.is_none() { findings.push(error("E911", &file, "`title` is required on FMEASheet")); }
            if fm.status.is_none() { findings.push(error("E911", &file, "`status` is required on FMEASheet")); }
            if let Some(ref id) = fm.id {
                if !is_fmea_id(id) {
                    findings.push(error("E912", &file, &format!("`id` '{}' does not match FMEA-* pattern", id)));
                }
            }
            // W902: empty sheet
            if fm.entries.as_ref().map_or(true, |v| v.is_empty()) {
                findings.push(warning("W902", &file, "FMEASheet has no `entries` — add at least one failure mode row"));
            }
        }

        // ── Tier 4: FMEAEntry (E913-E914, W903-W904) — synthesised by walker ─
        if matches!(fm.element_type, Some(ElementType::FMEAEntry)) {
            if let Some(ref id) = fm.id {
                if !is_fm_id(id) {
                    findings.push(error("E913", &file, &format!("FMEAEntry `id` '{}' does not match FM-* pattern", id)));
                }
            }
            // E914: severity / occurrence / detection range 1–10
            for (label, val) in [
                ("fmeaSeverity", fm.fmea_severity),
                ("occurrence", fm.occurrence),
                ("detection", fm.detection),
            ] {
                if let Some(v) = val {
                    if !(1..=10).contains(&v) {
                        findings.push(error("E914", &file, &format!("FMEAEntry.{} {} is out of range 1–10", label, v)));
                    }
                }
            }
            // W903: high-RPN entry without a recommended action
            if let Some(rpn) = fm.rpn {
                if rpn > 100 && fm.recommended_action.is_none() {
                    findings.push(warning("W903", &file, &format!("FMEAEntry RPN {} > 100 but has no `recommendedAction`", rpn)));
                }
            }
        }

        // ── Tier 4: TARASheet (E940-E941, W905) ─────────────────────────────────
        if matches!(fm.element_type, Some(ElementType::TARASheet)) {
            if fm.id.is_none() { findings.push(error("E940", &file, "`id` is required on TARASheet")); }
            if fm.title.is_none() { findings.push(error("E940", &file, "`title` is required on TARASheet")); }
            if fm.status.is_none() { findings.push(error("E940", &file, "`status` is required on TARASheet")); }
            if let Some(ref id) = fm.id {
                if !is_tara_id(id) {
                    findings.push(error("E941", &file, &format!("`id` '{}' does not match TARA-* pattern", id)));
                }
            }
            // W905: empty sheet — all four tables absent or empty
            let all_empty = fm.damage_table.as_ref().map_or(true, |v| v.is_empty())
                && fm.threat_table.as_ref().map_or(true, |v| v.is_empty())
                && fm.goal_table.as_ref().map_or(true, |v| v.is_empty())
                && fm.control_table.as_ref().map_or(true, |v| v.is_empty());
            if all_empty {
                findings.push(warning("W905", &file, "TARASheet has no rows in any section table — add damageTable, threatTable, goalTable, or controlTable entries"));
            }
        }

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

        // W702: asilLevel: D requirement must have at least one active L5 (HIL) TestCase
        if elem.frontmatter.asil_level.as_deref() == Some("D") && !active_tcs.is_empty() {
            let has_l5 = active_tcs.iter().any(|tc_id| {
                resolver
                    .get_by_id(elements, tc_id)
                    .and_then(|e| e.frontmatter.test_level.as_deref())
                    == Some("L5")
            });
            if !has_l5 {
                findings.push(warning(
                    "W702",
                    &elem.file_path,
                    &format!("Requirement '{}' has asilLevel: D but no active TestCase at testLevel: L5 (HIL) — ISO 26262-6 §9 requires hardware-in-the-loop testing for ASIL D", req_id),
                ));
            }
        }

        // W305: parent requirement must have at least one active integration-level TestCase
        // (L3 system test, L4 system integration test, or L5 HIL/acceptance).
        // Leaf-level test cases (L1/L2) on derived requirements are not sufficient to
        // verify the emergent, composed behaviour expressed by the parent.
        if is_parent && matches!(status, "approved" | "implemented" | "verified") {
            let has_integration_tc = active_tcs.iter().any(|tc_id| {
                resolver
                    .get_by_id(elements, tc_id)
                    .and_then(|e| e.frontmatter.test_level.as_deref())
                    .map_or(false, |lvl| matches!(lvl, "L3" | "L4" | "L5"))
            });
            if !has_integration_tc {
                findings.push(warning(
                    "W305",
                    &elem.file_path,
                    &format!(
                        "parent Requirement '{}' (status: {}) has no active system integration TestCase (testLevel: L3, L4, or L5)",
                        req_id, status
                    ),
                ));
            }
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

    // W007: *Def element never used as supertype: or typedBy: anywhere in the model.
    // Scans top-level fields AND typedBy inside features/connections/performs sub-objects
    // and exhibitsStates lists, so that elements referenced only in those positions are
    // not incorrectly flagged.
    {
        let mut referenced_defs: HashSet<String> = HashSet::new();
        for elem in elements.iter() {
            let fm = &elem.frontmatter;

            // Top-level supertype and typedBy
            for field in [fm.supertype.as_ref(), fm.typed_by.as_ref()].into_iter().flatten() {
                for s in yaml_strings(field) {
                    if let Some(target) = resolver.resolve_ref(elements, s) {
                        referenced_defs.insert(target.qualified_name.clone());
                    }
                }
            }

            // exhibitsStates: Vec<String> — direct qualified name references
            for s in fm.exhibits_states.iter().flatten() {
                if let Some(target) = resolver.resolve_ref(elements, s) {
                    referenced_defs.insert(target.qualified_name.clone());
                }
            }

            // features, connections, performs, flow_connections, etc. —
            // scan typedBy inside each mapping entry (and nested ports sub-key)
            for list in [
                fm.features.as_deref(),
                fm.connections.as_deref(),
                fm.flow_connections.as_deref(),
                fm.binding_connections.as_deref(),
                fm.succession_connections.as_deref(),
                fm.performs.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                collect_typed_by_refs(list, elements, &resolver, &mut referenced_defs);
            }
        }
        for elem in elements {
            if is_type_def(elem) {
                if !referenced_defs.contains(&elem.qualified_name) {
                    findings.push(warning(
                        "W007",
                        &elem.file_path,
                        &format!(
                            "'{}' is defined but never used as a supertype or type",
                            elem.qualified_name
                        ),
                    ));
                }
            }
        }
    }

    // W008: element has no type: field (untyped)
    for elem in elements {
        if elem.frontmatter.element_type.is_none() {
            findings.push(warning(
                "W008",
                &elem.file_path,
                &format!("'{}' has no type: field — element will be ignored by most commands", elem.qualified_name),
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

    // ── Tier 2 cross-reference checks (E825-E830) ────────────────────────────

    // Build reverse index: csg_implemented_by[csg_id_or_qn] — used for W802
    let mut csg_implemented: HashSet<String> = HashSet::new();
    // Build reverse index: he_referenced_by[he_id_or_qn] — used for W800
    let mut he_referenced: HashSet<String> = HashSet::new();
    // Build reverse index: csg_derived_reqs[csg_id_or_qn] — used for W804
    let mut csg_derived_reqs: HashSet<String> = HashSet::new();
    // Build reverse index: sg_derived_reqs[sg_id_or_qn] — used for W805
    let mut sg_derived_reqs: HashSet<String> = HashSet::new();

    for elem in elements {
        let fm = &elem.frontmatter;

        // E825: SafetyGoal.hazardousEvents must each resolve to a HazardousEvent
        if matches!(fm.element_type, Some(ElementType::SafetyGoal)) {
            if let Some(ref refs) = fm.hazardous_events {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E825", &elem.file_path,
                            &format!("`hazardousEvents` '{}' does not resolve to any element", r))),
                        Some(target) if !Resolver::is_hazardous_event(target) => {
                            findings.push(error("E825", &elem.file_path,
                                &format!("`hazardousEvents` '{}' does not resolve to a HazardousEvent", r)));
                        }
                        Some(target) => {
                            he_referenced.insert(target.qualified_name.clone());
                            if let Some(ref id) = target.frontmatter.id { he_referenced.insert(id.clone()); }
                        }
                    }
                }
            }
        }

        // E826: ThreatScenario.damageScenarios must each resolve to a DamageScenario
        if matches!(fm.element_type, Some(ElementType::ThreatScenario)) {
            if let Some(ref refs) = fm.damage_scenarios {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E826", &elem.file_path,
                            &format!("`damageScenarios` '{}' does not resolve to any element", r))),
                        Some(target) if !Resolver::is_damage_scenario(target) => {
                            findings.push(error("E826", &elem.file_path,
                                &format!("`damageScenarios` '{}' does not resolve to a DamageScenario", r)));
                        }
                        _ => {}
                    }
                }
            }
        }

        // E827: CybersecurityGoal.threatScenarios must each resolve to a ThreatScenario
        if matches!(fm.element_type, Some(ElementType::CybersecurityGoal)) {
            if let Some(ref refs) = fm.threat_scenarios {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E827", &elem.file_path,
                            &format!("`threatScenarios` '{}' does not resolve to any element", r))),
                        Some(target) if !Resolver::is_threat_scenario(target) => {
                            findings.push(error("E827", &elem.file_path,
                                &format!("`threatScenarios` '{}' does not resolve to a ThreatScenario", r)));
                        }
                        _ => {}
                    }
                }
            }
        }

        // E828: SecurityControl.implementsGoals must each resolve to a CybersecurityGoal
        if matches!(fm.element_type, Some(ElementType::SecurityControl)) {
            if let Some(ref refs) = fm.implements_goals {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E828", &elem.file_path,
                            &format!("`implementsGoals` '{}' does not resolve to any element", r))),
                        Some(target) if !Resolver::is_cybersecurity_goal(target) => {
                            findings.push(error("E828", &elem.file_path,
                                &format!("`implementsGoals` '{}' does not resolve to a CybersecurityGoal", r)));
                        }
                        Some(target) => {
                            csg_implemented.insert(target.qualified_name.clone());
                            if let Some(ref id) = target.frontmatter.id { csg_implemented.insert(id.clone()); }
                        }
                    }
                }
            }
        }

        // E829: VulnerabilityReport.mitigatedBy must each resolve to a SecurityControl
        if matches!(fm.element_type, Some(ElementType::VulnerabilityReport)) {
            if let Some(ref refs) = fm.mitigated_by {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E829", &elem.file_path,
                            &format!("`mitigatedBy` '{}' does not resolve to any element", r))),
                        Some(target) if !Resolver::is_security_control(target) => {
                            findings.push(error("E829", &elem.file_path,
                                &format!("`mitigatedBy` '{}' does not resolve to a SecurityControl", r)));
                        }
                        _ => {}
                    }
                }
            }
            // E830: affectedElements must resolve to known model elements
            if let Some(ref refs) = fm.affected_elements {
                for r in refs {
                    if resolver.resolve_ref(elements, r).is_none() {
                        findings.push(error("E830", &elem.file_path,
                            &format!("`affectedElements` '{}' does not resolve to any element", r)));
                    }
                }
            }
        }

        // E831: derivedFromSecurityGoal must resolve to a CybersecurityGoal
        if let Some(ref goal_ref) = fm.derived_from_security_goal {
            match resolver.resolve_ref(elements, goal_ref) {
                None => findings.push(error("E831", &elem.file_path,
                    &format!("`derivedFromSecurityGoal` '{}' does not resolve to any element", goal_ref))),
                Some(target) if !Resolver::is_cybersecurity_goal(target) => {
                    findings.push(error("E831", &elem.file_path,
                        &format!("`derivedFromSecurityGoal` '{}' does not resolve to a CybersecurityGoal", goal_ref)));
                }
                Some(target) => {
                    csg_derived_reqs.insert(target.qualified_name.clone());
                    if let Some(ref id) = target.frontmatter.id { csg_derived_reqs.insert(id.clone()); }
                }
            }
        }

        // E832: derivedFromSafetyGoal must resolve to a SafetyGoal
        if let Some(ref goal_ref) = fm.derived_from_safety_goal {
            match resolver.resolve_ref(elements, goal_ref) {
                None => findings.push(error("E832", &elem.file_path,
                    &format!("`derivedFromSafetyGoal` '{}' does not resolve to any element", goal_ref))),
                Some(target) if !Resolver::is_safety_goal(target) => {
                    findings.push(error("E832", &elem.file_path,
                        &format!("`derivedFromSafetyGoal` '{}' does not resolve to a SafetyGoal", goal_ref)));
                }
                Some(target) => {
                    sg_derived_reqs.insert(target.qualified_name.clone());
                    if let Some(ref id) = target.frontmatter.id { sg_derived_reqs.insert(id.clone()); }
                }
            }
        }

    }

    // ── Tier 4 cross-reference checks ────────────────────────────────────────

    for elem in elements {
        let fm = &elem.frontmatter;

        // E902: FaultTree.topEvent must resolve to a SafetyGoal
        if matches!(fm.element_type, Some(ElementType::FaultTree)) {
            if let Some(ref te) = fm.top_event {
                match resolver.resolve_ref(elements, te) {
                    None => findings.push(error("E902", &elem.file_path,
                        &format!("`topEvent` '{}' does not resolve to any element", te))),
                    Some(target) if !Resolver::is_safety_goal(target) => {
                        findings.push(error("E902", &elem.file_path,
                            &format!("`topEvent` '{}' does not resolve to a SafetyGoal", te)));
                    }
                    _ => {}
                }
            }
        }

        // E906: FaultTreeGate.inputs must each resolve to a FaultTreeGate or FaultTreeEvent
        if matches!(fm.element_type, Some(ElementType::FaultTreeGate)) {
            if let Some(ref inputs) = fm.inputs {
                for r in inputs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E906", &elem.file_path,
                            &format!("`inputs` '{}' does not resolve to any element", r))),
                        Some(target)
                            if !Resolver::is_fault_tree_gate(target)
                                && !Resolver::is_fault_tree_event(target) =>
                        {
                            findings.push(error("E906", &elem.file_path,
                                &format!("`inputs` '{}' is not a FaultTreeGate or FaultTreeEvent", r)));
                        }
                        _ => {}
                    }
                }
            }
        }

        // W904: FMEAEntry.ref (subject) should resolve to a known element
        if matches!(fm.element_type, Some(ElementType::FMEAEntry)) {
            if let Some(ref r) = fm.subject {
                if resolver.resolve_ref(elements, r).is_none() {
                    findings.push(warning("W904", &elem.file_path,
                        &format!("FMEAEntry `ref` '{}' does not resolve to a known element", r)));
                }
            }
        }
    }

    // W900: FaultTree with no FaultTreeGate or FaultTreeEvent children
    for elem in elements {
        if !matches!(elem.frontmatter.element_type, Some(ElementType::FaultTree)) {
            continue;
        }
        let prefix = format!("{}::", elem.qualified_name);
        let has_children = elements.iter().any(|e| {
            e.qualified_name.starts_with(&prefix)
                && matches!(
                    e.frontmatter.element_type,
                    Some(ElementType::FaultTreeGate) | Some(ElementType::FaultTreeEvent)
                )
        });
        if !has_children {
            let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
            findings.push(warning("W900", &elem.file_path,
                &format!("FaultTree '{}' has no FaultTreeGate or FaultTreeEvent children", id)));
        }
    }

    // W800: HazardousEvent not referenced by any SafetyGoal
    for elem in elements {
        if Resolver::is_hazardous_event(elem) {
            let referenced = he_referenced.contains(&elem.qualified_name)
                || elem.frontmatter.id.as_ref().map_or(false, |id| he_referenced.contains(id));
            if !referenced {
                let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
                findings.push(warning("W800", &elem.file_path,
                    &format!("HazardousEvent '{}' is not referenced by any SafetyGoal.hazardousEvents", id)));
            }
        }
    }

    // W802: CybersecurityGoal not implemented by any SecurityControl
    for elem in elements {
        if Resolver::is_cybersecurity_goal(elem) {
            let implemented = csg_implemented.contains(&elem.qualified_name)
                || elem.frontmatter.id.as_ref().map_or(false, |id| csg_implemented.contains(id));
            if !implemented {
                let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
                findings.push(warning("W802", &elem.file_path,
                    &format!("CybersecurityGoal '{}' is not implemented by any SecurityControl.implementsGoals", id)));
            }
        }
    }

    // W804: CybersecurityGoal not referenced by any Requirement via derivedFromSecurityGoal
    for elem in elements {
        if Resolver::is_cybersecurity_goal(elem) {
            let has_req = csg_derived_reqs.contains(&elem.qualified_name)
                || elem.frontmatter.id.as_ref().map_or(false, |id| csg_derived_reqs.contains(id));
            if !has_req {
                let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
                findings.push(warning("W804", &elem.file_path,
                    &format!("CybersecurityGoal '{}' has no Requirement with `derivedFromSecurityGoal` pointing to it", id)));
            }
        }
    }

    // W805: SafetyGoal not referenced by any Requirement via derivedFromSafetyGoal
    for elem in elements {
        if Resolver::is_safety_goal(elem) {
            let has_req = sg_derived_reqs.contains(&elem.qualified_name)
                || elem.frontmatter.id.as_ref().map_or(false, |id| sg_derived_reqs.contains(id));
            if !has_req {
                let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
                findings.push(warning("W805", &elem.file_path,
                    &format!("SafetyGoal '{}' has no Requirement with `derivedFromSafetyGoal` pointing to it", id)));
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
            if let Some(ref to_refs) = elem.frontmatter.allocated_to {
                for to_ref in to_refs {
                    if let Some(target) = resolver.get(elements, to_ref) {
                        if target.frontmatter.domain.as_deref() == Some("hardware") {
                            if let Some(ref from_refs) = elem.frontmatter.allocated_from {
                                for from_ref in from_refs {
                                    hw_alloc_targets.insert(from_ref.clone());
                                }
                            }
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

    // E016/E017/E018: cycle detection in supertype, derivedFrom, and subsets graphs
    {
        let (full_graph, node_idx) = crate::graph::build_graph(elements);
        // Map NodeIndex back to file path for error reporting.
        let idx_to_file: HashMap<petgraph::graph::NodeIndex, &str> = node_idx
            .iter()
            .map(|(qn, &ni)| {
                let file = elements
                    .iter()
                    .find(|e| &e.qualified_name == qn)
                    .map(|e| e.file_path.as_str())
                    .unwrap_or(qn.as_str());
                (ni, file)
            })
            .collect();

        let checks: &[(&str, EdgeKind, &str)] = &[
            ("E016", EdgeKind::Supertype, "supertype cycle detected"),
            ("E017", EdgeKind::DerivedFrom, "derivedFrom cycle detected"),
            ("E018", EdgeKind::Subsets, "subsets cycle detected"),
        ];

        for (code, kind, label) in checks {
            let mut sub: DiGraph<petgraph::graph::NodeIndex, ()> = DiGraph::new();
            let mut sub_nodes: HashMap<petgraph::graph::NodeIndex, petgraph::graph::NodeIndex> =
                HashMap::new();

            for edge in full_graph.edge_references() {
                if edge.weight() == kind {
                    let src_orig = edge.source();
                    let dst_orig = edge.target();
                    let src = *sub_nodes
                        .entry(src_orig)
                        .or_insert_with(|| sub.add_node(src_orig));
                    let dst = *sub_nodes
                        .entry(dst_orig)
                        .or_insert_with(|| sub.add_node(dst_orig));
                    sub.add_edge(src, dst, ());
                }
            }

            if let Err(cycle) = toposort(&sub, None) {
                let orig_ni = sub[cycle.node_id()];
                let file = idx_to_file.get(&orig_ni).copied().unwrap_or("unknown");
                let qname = &full_graph[orig_ni];
                findings.push(error(
                    code,
                    file,
                    &format!("{} involving '{}'", label, qname),
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

/// Recursively scan a list of YAML mappings for `typedBy:` string values and resolve them
/// into qualified names added to `out`. Also descends into `ports:` sub-lists.
fn collect_typed_by_refs(
    list: &[serde_yaml::Value],
    elements: &[RawElement],
    resolver: &Resolver,
    out: &mut HashSet<String>,
) {
    let key_typed_by = serde_yaml::Value::String("typedBy".into());
    let key_ports = serde_yaml::Value::String("ports".into());
    for item in list {
        if let serde_yaml::Value::Mapping(map) = item {
            if let Some(v) = map.get(&key_typed_by) {
                for s in yaml_strings(v) {
                    if let Some(target) = resolver.resolve_ref(elements, s) {
                        out.insert(target.qualified_name.clone());
                    }
                }
            }
            // Recurse into nested ports: sub-key
            if let Some(serde_yaml::Value::Sequence(ports)) = map.get(&key_ports) {
                collect_typed_by_refs(ports, elements, resolver, out);
            }
        }
    }
}

/// Returns true for element types that are definitions and must be used by at least one usage.
fn is_type_def(elem: &RawElement) -> bool {
    matches!(
        elem.frontmatter.element_type,
        Some(
            ElementType::PartDef
            | ElementType::ItemDef
            | ElementType::AttributeDef
            | ElementType::PortDef
            | ElementType::ConnectionDef
            | ElementType::InterfaceDef
            | ElementType::ActionDef
            | ElementType::ConstraintDef
            | ElementType::RequirementDef
            | ElementType::CalculationDef
            | ElementType::StateDef
            | ElementType::FlowDef
            | ElementType::UseCaseDef
            | ElementType::ViewpointDef
            | ElementType::ViewDef
            | ElementType::AllocationDef
        )
    )
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
