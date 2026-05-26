use std::collections::HashMap;
use crate::element::{ElementType, RawElement};

static REQ_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static TC_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static CONF_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static ADR_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

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

/// Returns true when `s` matches the REQ-*, TC-*, CONF-*, or ADR-* stable-ID patterns.
pub fn is_stable_id(s: &str) -> bool {
    req_re().is_match(s) || tc_re().is_match(s) || conf_re().is_match(s) || adr_re().is_match(s)
}

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
}
