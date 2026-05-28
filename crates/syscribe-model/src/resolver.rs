use std::collections::HashMap;
use crate::element::{ElementType, RawElement};

static REQ_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static TC_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static CONF_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static ADR_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
// Tier 2 safety/security stable-ID patterns
static HE_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static SG_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static DS_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static TS_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static CSG_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static SC_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static VR_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

fn req_re() -> &'static regex::Regex {
    REQ_RE.get_or_init(|| regex::Regex::new(r"^REQ(-[A-Z0-9]{2,12})+-[0-9]{3}$").unwrap())
}

fn tc_re() -> &'static regex::Regex {
    TC_RE.get_or_init(|| regex::Regex::new(r"^TC(-[A-Z0-9]{2,12})+-[0-9]{3}$").unwrap())
}

fn conf_re() -> &'static regex::Regex {
    CONF_RE.get_or_init(|| regex::Regex::new(r"^CONF(-[A-Z0-9]{2,12})+-[0-9]{3}$").unwrap())
}

fn adr_re() -> &'static regex::Regex {
    ADR_RE.get_or_init(|| regex::Regex::new(r"^ADR(-[A-Z0-9]{2,12})+-[0-9]{3}$").unwrap())
}

fn he_re() -> &'static regex::Regex {
    HE_RE.get_or_init(|| regex::Regex::new(r"^HE(-[A-Z0-9]{2,12})+-[0-9]{3}$").unwrap())
}

fn sg_re() -> &'static regex::Regex {
    SG_RE.get_or_init(|| regex::Regex::new(r"^SG(-[A-Z0-9]{2,12})+-[0-9]{3}$").unwrap())
}

fn ds_re() -> &'static regex::Regex {
    DS_RE.get_or_init(|| regex::Regex::new(r"^DS(-[A-Z0-9]{2,12})+-[0-9]{3}$").unwrap())
}

fn ts_re() -> &'static regex::Regex {
    TS_RE.get_or_init(|| regex::Regex::new(r"^TS(-[A-Z0-9]{2,12})+-[0-9]{3}$").unwrap())
}

fn csg_re() -> &'static regex::Regex {
    CSG_RE.get_or_init(|| regex::Regex::new(r"^CSG(-[A-Z0-9]{2,12})+-[0-9]{3}$").unwrap())
}

fn sc_re() -> &'static regex::Regex {
    SC_RE.get_or_init(|| regex::Regex::new(r"^SC(-[A-Z0-9]{2,12})+-[0-9]{3}$").unwrap())
}

fn vr_re() -> &'static regex::Regex {
    VR_RE.get_or_init(|| regex::Regex::new(r"^VR(-[A-Z0-9]{2,12})+-[0-9]{3}$").unwrap())
}

/// Returns true when `s` matches any known stable-ID pattern.
pub fn is_stable_id(s: &str) -> bool {
    req_re().is_match(s)
        || tc_re().is_match(s)
        || conf_re().is_match(s)
        || adr_re().is_match(s)
        || he_re().is_match(s)
        || sg_re().is_match(s)
        || ds_re().is_match(s)
        || ts_re().is_match(s)
        || csg_re().is_match(s)
        || sc_re().is_match(s)
        || vr_re().is_match(s)
}

/// Returns true for HE-* IDs (HazardousEvent).
pub fn is_he_id(s: &str) -> bool { he_re().is_match(s) }
/// Returns true for SG-* IDs (SafetyGoal).
pub fn is_sg_id(s: &str) -> bool { sg_re().is_match(s) }
/// Returns true for DS-* IDs (DamageScenario).
pub fn is_ds_id(s: &str) -> bool { ds_re().is_match(s) }
/// Returns true for TS-* IDs (ThreatScenario).
pub fn is_ts_id(s: &str) -> bool { ts_re().is_match(s) }
/// Returns true for CSG-* IDs (CybersecurityGoal).
pub fn is_csg_id(s: &str) -> bool { csg_re().is_match(s) }
/// Returns true for SC-* IDs (SecurityControl).
pub fn is_sc_id(s: &str) -> bool { sc_re().is_match(s) }
/// Returns true for VR-* IDs (VulnerabilityReport).
pub fn is_vr_id(s: &str) -> bool { vr_re().is_match(s) }

/// Returns true for ADR-* IDs.
pub fn is_adr_id(s: &str) -> bool {
    adr_re().is_match(s)
}

/// Returns true for REQ-* IDs.
pub fn is_req_id(s: &str) -> bool {
    req_re().is_match(s)
}

/// Returns true for TC-* IDs.
pub fn is_tc_id(s: &str) -> bool {
    tc_re().is_match(s)
}

/// Returns true for CONF-* IDs.
pub fn is_conf_id(s: &str) -> bool {
    conf_re().is_match(s)
}

pub struct Resolver {
    /// Index by qualified name.
    pub by_qname: HashMap<String, usize>,
    /// Index by stable id field (native Requirement, TestCase, and Configuration only).
    pub by_id: HashMap<String, usize>,
}

impl Resolver {
    pub fn new(elements: &[RawElement]) -> Self {
        let mut by_qname = HashMap::new();
        let mut by_id = HashMap::new();

        for (i, e) in elements.iter().enumerate() {
            by_qname.insert(e.qualified_name.clone(), i);
            if let Some(ref id) = e.frontmatter.id {
                if is_stable_id(id) {
                    by_id.insert(id.clone(), i);
                }
            }
        }

        Self { by_qname, by_id }
    }

    pub fn get<'a>(&self, elements: &'a [RawElement], qname: &str) -> Option<&'a RawElement> {
        self.by_qname.get(qname).map(|&i| &elements[i])
    }

    pub fn get_by_id<'a>(&self, elements: &'a [RawElement], id: &str) -> Option<&'a RawElement> {
        self.by_id.get(id).map(|&i| &elements[i])
    }

    /// Resolve a cross-reference string from `verifies:` or `derivedFrom:`.
    /// Step 0: if it matches a stable-ID pattern (REQ-*, TC-*, CONF-*), look up by id.
    /// Otherwise fall back to qualified-name lookup.
    pub fn resolve_ref<'a>(&self, elements: &'a [RawElement], r: &str) -> Option<&'a RawElement> {
        if is_stable_id(r) {
            self.get_by_id(elements, r)
        } else {
            self.get(elements, r)
        }
    }

    /// True if `elem` is a native Requirement (type: Requirement with a REQ-* id).
    pub fn is_native_requirement(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::Requirement))
            && elem
                .frontmatter
                .id
                .as_deref()
                .map(is_req_id)
                .unwrap_or(false)
    }

    /// True if `elem` is a native TestCase (type: TestCase with a TC-* id).
    pub fn is_native_testcase(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::TestCase))
            && elem
                .frontmatter
                .id
                .as_deref()
                .map(is_tc_id)
                .unwrap_or(false)
    }

    /// True if `elem` is a FeatureDef.
    pub fn is_feature_def(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::FeatureDef))
    }

    /// True if `elem` is a Configuration.
    pub fn is_configuration(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::Configuration))
    }

    /// True if `elem` is an ADR.
    pub fn is_adr(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::ADR))
    }

    pub fn is_hazardous_event(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::HazardousEvent))
    }

    pub fn is_safety_goal(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::SafetyGoal))
    }

    pub fn is_damage_scenario(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::DamageScenario))
    }

    pub fn is_threat_scenario(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::ThreatScenario))
    }

    pub fn is_cybersecurity_goal(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::CybersecurityGoal))
    }

    pub fn is_security_control(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::SecurityControl))
    }

    pub fn is_vulnerability_report(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::VulnerabilityReport))
    }
}
