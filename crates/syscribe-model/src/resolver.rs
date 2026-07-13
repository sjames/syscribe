use std::collections::HashMap;
use std::sync::RwLock;
use crate::element::{ElementType, RawElement};

static REQ_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static TC_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static TP_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static CONF_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static ADR_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static RR_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static TRD_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static ZN_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static CD_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static CM_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
// Tier 2 safety/security stable-ID patterns
static HE_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
// Tier 4 TARA sheet stable-ID pattern
static TARA_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
// Tier 4 FTA/FMEA stable-ID patterns
static FT_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static FTG_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static FTE_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static FMEA_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static FM_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
// Tier 4 attack-path (ISO/SAE 21434 §15.7) stable-ID patterns
static AT_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static ATG_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static ATS_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
// GSN argument-layer stable-ID patterns (issue #20)
static ARG_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static AOU_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static SG_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static DS_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static TS_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static CSG_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static SC_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static VR_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
// Asset (ISO/SAE 21434 §15.3) stable-ID pattern (REQ-TRS-TYPE-017)
static ASSET_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
// FeatureDef optional stable-ID pattern (REQ-TRS-ID-006)
static FEAT_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
// Baseline stable-ID pattern (FEAT-style, no trailing numeric suffix; REQ-TRS-BL-001)
static BL_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

fn req_re() -> &'static regex::Regex {
    REQ_RE.get_or_init(|| regex::Regex::new(r"^REQ(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn tc_re() -> &'static regex::Regex {
    TC_RE.get_or_init(|| regex::Regex::new(r"^TC(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn tp_re() -> &'static regex::Regex {
    TP_RE.get_or_init(|| regex::Regex::new(r"^TP(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn conf_re() -> &'static regex::Regex {
    CONF_RE.get_or_init(|| regex::Regex::new(r"^CONF(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn adr_re() -> &'static regex::Regex {
    ADR_RE.get_or_init(|| regex::Regex::new(r"^ADR(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn rr_re() -> &'static regex::Regex {
    RR_RE.get_or_init(|| regex::Regex::new(r"^RR(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn trd_re() -> &'static regex::Regex {
    TRD_RE.get_or_init(|| regex::Regex::new(r"^TRD(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn zn_re() -> &'static regex::Regex {
    ZN_RE.get_or_init(|| regex::Regex::new(r"^ZN(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn cd_re() -> &'static regex::Regex {
    CD_RE.get_or_init(|| regex::Regex::new(r"^CD(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn cm_re() -> &'static regex::Regex {
    CM_RE.get_or_init(|| regex::Regex::new(r"^CM(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn he_re() -> &'static regex::Regex {
    HE_RE.get_or_init(|| regex::Regex::new(r"^HE(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn sg_re() -> &'static regex::Regex {
    SG_RE.get_or_init(|| regex::Regex::new(r"^SG(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn ds_re() -> &'static regex::Regex {
    DS_RE.get_or_init(|| regex::Regex::new(r"^DS(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn ts_re() -> &'static regex::Regex {
    TS_RE.get_or_init(|| regex::Regex::new(r"^TS(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn csg_re() -> &'static regex::Regex {
    CSG_RE.get_or_init(|| regex::Regex::new(r"^CSG(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn sc_re() -> &'static regex::Regex {
    SC_RE.get_or_init(|| regex::Regex::new(r"^SC(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn vr_re() -> &'static regex::Regex {
    VR_RE.get_or_init(|| regex::Regex::new(r"^VR(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn asset_re() -> &'static regex::Regex {
    ASSET_RE.get_or_init(|| regex::Regex::new(r"^ASSET(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

/// Returns true for ASSET-* IDs (REQ-TRS-TYPE-017).
pub fn is_asset_id(s: &str) -> bool {
    asset_re().is_match(s) || extra_matches("ASSET", s)
}

fn feat_re() -> &'static regex::Regex {
    // FEAT ids do NOT require a trailing numeric segment (unlike REQ/TC/…): a feature
    // may be `FEAT-ABS` or `FEAT-ABS-001`. Each segment is [A-Z0-9]{2,12}.
    FEAT_RE.get_or_init(|| regex::Regex::new(r"^FEAT(-[A-Z0-9]{2,12})+$").unwrap())
}

/// Returns true for FEAT-* IDs (optional FeatureDef stable id, REQ-TRS-ID-006).
pub fn is_feat_id(s: &str) -> bool {
    feat_re().is_match(s) || extra_matches("FEAT", s)
}

fn bl_re() -> &'static regex::Regex {
    // BL ids do NOT require a trailing numeric segment (FEAT-style, REQ-TRS-BL-001):
    // a baseline may be `BL-2026-07` or `BL-QUARTERLY-001`. Each segment is [A-Z0-9]{2,12}.
    BL_RE.get_or_init(|| regex::Regex::new(r"^BL(-[A-Z0-9]{2,12})+$").unwrap())
}

/// Returns true for BL-* IDs (Baseline, REQ-TRS-BL-001).
pub fn is_baseline_id(s: &str) -> bool {
    bl_re().is_match(s) || extra_matches("BL", s)
}

// ── Configurable additional stable-ID prefixes (REQ-TRS-ID-007) ──────────────────
//
// Each id-identified type has a fixed built-in prefix; a project may declare *extra*
// prefixes per type in `[ids.prefixes]` of `.syscribe.toml`. Extras are strictly
// additive (the built-in always stays valid) and pure identity (they affect only id
// recognition / id-based resolution). The registry below is installed once from the
// loaded config; when empty (the default) behaviour is identical to built-in only.

/// The id-identified element types, each paired with its built-in stable-ID prefix and
/// whether its ids require a trailing numeric suffix (every type except `FeatureDef`,
/// REQ-TRS-ID-006). Single source of truth: the type name is the `[ids.prefixes]` key,
/// the built-in prefix keys the runtime registry, and the suffix flag selects the
/// grammar used to compile an additional prefix.
pub const STABLE_ID_KINDS: &[(&str, &str, bool)] = &[
    ("Requirement", "REQ", true),
    ("TestCase", "TC", true),
    ("TestPlan", "TP", true),
    ("Configuration", "CONF", true),
    ("ADR", "ADR", true),
    ("ReviewRecord", "RR", true),
    ("TradeStudy", "TRD", true),
    ("Zone", "ZN", true),
    ("Conduit", "CD", true),
    ("ConfirmationMeasure", "CM", true),
    ("HazardousEvent", "HE", true),
    ("SafetyGoal", "SG", true),
    ("DamageScenario", "DS", true),
    ("ThreatScenario", "TS", true),
    ("CybersecurityGoal", "CSG", true),
    ("SecurityControl", "SC", true),
    ("VulnerabilityReport", "VR", true),
    ("Asset", "ASSET", true),
    ("TARASheet", "TARA", true),
    ("FaultTree", "FT", true),
    ("FaultTreeGate", "FTG", true),
    ("FaultTreeEvent", "FTE", true),
    ("FMEASheet", "FMEA", true),
    ("FMEAEntry", "FM", true),
    ("AttackTree", "AT", true),
    ("AttackTreeGate", "ATG", true),
    ("AttackStep", "ATS", true),
    ("Argument", "ARG", true),
    ("AssumptionOfUse", "AOU", true),
    ("FeatureDef", "FEAT", false),
    ("Baseline", "BL", false),
];

static PREFIX_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

/// True when `s` is a well-formed stable-ID prefix: uppercase, starting with a letter,
/// 2–12 characters (`^[A-Z][A-Z0-9]{1,11}$`) — the shape of every built-in prefix and
/// the rule a configured additional prefix must satisfy (REQ-TRS-ID-007).
pub fn is_valid_id_prefix(s: &str) -> bool {
    PREFIX_RE
        .get_or_init(|| regex::Regex::new(r"^[A-Z][A-Z0-9]{1,11}$").unwrap())
        .is_match(s)
}

/// True when `name` is an element-type name that carries a stable id (an accepted
/// `[ids.prefixes]` key).
pub fn is_stable_id_type_name(name: &str) -> bool {
    STABLE_ID_KINDS.iter().any(|(ty, _, _)| *ty == name)
}

/// Compiled additional stable-ID prefixes (REQ-TRS-ID-007), keyed by the **built-in**
/// prefix of the type they extend (e.g. `"REQ"`). Installed from `.syscribe.toml` via
/// [`set_extra_id_prefixes_by_type`]; empty (the default) means built-in prefixes only.
static EXTRA_PREFIXES: std::sync::OnceLock<RwLock<HashMap<String, Vec<regex::Regex>>>> =
    std::sync::OnceLock::new();

fn extra_registry() -> &'static RwLock<HashMap<String, Vec<regex::Regex>>> {
    EXTRA_PREFIXES.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Install the configured additional stable-ID prefixes, replacing any previous set.
/// `by_type` is keyed by element-type name (`[ids.prefixes]` form). Unknown type names
/// and prefixes failing [`is_valid_id_prefix`] are silently skipped here (the validator
/// reports them as `W046`); an empty map clears all extras.
pub fn set_extra_id_prefixes_by_type(by_type: &HashMap<String, Vec<String>>) {
    let mut compiled: HashMap<String, Vec<regex::Regex>> = HashMap::new();
    for (type_name, builtin, requires_suffix) in STABLE_ID_KINDS {
        let Some(extras) = by_type.get(*type_name) else { continue };
        let mut res = Vec::new();
        for p in extras {
            if !is_valid_id_prefix(p) {
                continue;
            }
            let pat = if *requires_suffix {
                format!(r"^{p}(-[A-Z0-9]{{2,12}})+-[0-9]{{3,}}$")
            } else {
                format!(r"^{p}(-[A-Z0-9]{{2,12}})+$")
            };
            if let Ok(re) = regex::Regex::new(&pat) {
                res.push(re);
            }
        }
        if !res.is_empty() {
            compiled.insert((*builtin).to_string(), res);
        }
    }
    if let Ok(mut w) = extra_registry().write() {
        *w = compiled;
    }
}

/// True when `s` matches any configured additional prefix registered under built-in
/// prefix `builtin` (REQ-TRS-ID-007).
fn extra_matches(builtin: &str, s: &str) -> bool {
    let Ok(map) = extra_registry().read() else { return false };
    map.get(builtin).is_some_and(|res| res.iter().any(|re| re.is_match(s)))
}

/// True when `s` matches any configured additional prefix for any type.
fn any_extra_matches(s: &str) -> bool {
    let Ok(map) = extra_registry().read() else { return false };
    map.values().any(|res| res.iter().any(|re| re.is_match(s)))
}

/// The **auto-imported** SysMLv2 standard-library packages whose membership is fully
/// known (spec §4.7). Type references to these resolve from the built-in inventory
/// without an `.md` file; an unknown member of one of them is a likely typo
/// (REQ-TRS-LIB-001). Import-only packages (`SI`, `ISQ`, …) are intentionally absent —
/// their membership is not enumerated, so they stay lenient.
pub const BUILTIN_TYPE_PACKAGES: &[(&str, &[&str])] = &[
    ("ScalarValues", &["Integer", "Real", "Natural", "Boolean", "String"]),
    ("Base", &["Anything", "DataValue"]),
];

/// Classification of a reference against the built-in type inventory.
pub enum BuiltinType {
    /// A recognised member of a known auto-imported built-in package.
    Member,
    /// A reference into a known built-in package, to a member it does not declare.
    UnknownMember {
        pkg: &'static str,
        member: String,
        known: &'static [&'static str],
    },
    /// Not a reference into a known auto-imported built-in package (in-model,
    /// import-only stdlib, or external).
    NotBuiltin,
}

/// Classify `s` against [`BUILTIN_TYPE_PACKAGES`] (REQ-TRS-LIB-001). Only a single
/// `Pkg::member` form is considered; deeper nesting or a bare package is `NotBuiltin`.
pub fn builtin_type_kind(s: &str) -> BuiltinType {
    for (pkg, members) in BUILTIN_TYPE_PACKAGES {
        if let Some(member) = s.strip_prefix(pkg).and_then(|r| r.strip_prefix("::")) {
            if member.is_empty() || member.contains("::") {
                return BuiltinType::NotBuiltin;
            }
            if members.contains(&member) {
                return BuiltinType::Member;
            }
            return BuiltinType::UnknownMember {
                pkg,
                member: member.to_string(),
                known: members,
            };
        }
    }
    BuiltinType::NotBuiltin
}

/// True when `s` names a recognised built-in standard-library type (REQ-TRS-LIB-001).
pub fn is_builtin_type(s: &str) -> bool {
    matches!(builtin_type_kind(s), BuiltinType::Member)
}

static BASIC_NAME_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

/// True when `s` is a SysMLv2 **basic name**: a letter or `_`, then letters, digits
/// or `_` (`^[A-Za-z_][A-Za-z0-9_]*$`). Hyphens, spaces and other punctuation are not
/// permitted — such names must be renamed (REQ-TRS-NAME-001 / `W042`).
pub fn is_basic_name(s: &str) -> bool {
    BASIC_NAME_RE
        .get_or_init(|| regex::Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap())
        .is_match(s)
}

/// Returns true when `s` matches any known stable-ID pattern.
pub fn is_stable_id(s: &str) -> bool {
    req_re().is_match(s)
        || tc_re().is_match(s)
        || tp_re().is_match(s)
        || asset_re().is_match(s)
        || conf_re().is_match(s)
        || adr_re().is_match(s)
        || rr_re().is_match(s)
        || trd_re().is_match(s)
        || zn_re().is_match(s)
        || cd_re().is_match(s)
        || cm_re().is_match(s)
        || he_re().is_match(s)
        || sg_re().is_match(s)
        || ds_re().is_match(s)
        || ts_re().is_match(s)
        || csg_re().is_match(s)
        || sc_re().is_match(s)
        || vr_re().is_match(s)
        || tara_re().is_match(s)
        || ft_re().is_match(s)
        || ftg_re().is_match(s)
        || fte_re().is_match(s)
        || fmea_re().is_match(s)
        || fm_re().is_match(s)
        || at_re().is_match(s)
        || atg_re().is_match(s)
        || ats_re().is_match(s)
        || arg_re().is_match(s)
        || aou_re().is_match(s)
        || feat_re().is_match(s)
        || bl_re().is_match(s)
        || any_extra_matches(s)
}

/// Returns true for HE-* IDs (HazardousEvent).
pub fn is_he_id(s: &str) -> bool { he_re().is_match(s) || extra_matches("HE", s) }
/// Returns true for SG-* IDs (SafetyGoal).
pub fn is_sg_id(s: &str) -> bool { sg_re().is_match(s) || extra_matches("SG", s) }
/// Returns true for DS-* IDs (DamageScenario).
pub fn is_ds_id(s: &str) -> bool { ds_re().is_match(s) || extra_matches("DS", s) }
/// Returns true for TS-* IDs (ThreatScenario).
pub fn is_ts_id(s: &str) -> bool { ts_re().is_match(s) || extra_matches("TS", s) }
/// Returns true for CSG-* IDs (CybersecurityGoal).
pub fn is_csg_id(s: &str) -> bool { csg_re().is_match(s) || extra_matches("CSG", s) }
/// Returns true for SC-* IDs (SecurityControl).
pub fn is_sc_id(s: &str) -> bool { sc_re().is_match(s) || extra_matches("SC", s) }
/// Returns true for VR-* IDs (VulnerabilityReport).
pub fn is_vr_id(s: &str) -> bool { vr_re().is_match(s) || extra_matches("VR", s) }

fn tara_re() -> &'static regex::Regex {
    TARA_RE.get_or_init(|| regex::Regex::new(r"^TARA(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

/// Returns true for TARA-* IDs (TARASheet).
pub fn is_tara_id(s: &str) -> bool { tara_re().is_match(s) || extra_matches("TARA", s) }

fn ft_re() -> &'static regex::Regex {
    FT_RE.get_or_init(|| regex::Regex::new(r"^FT(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}
fn ftg_re() -> &'static regex::Regex {
    FTG_RE.get_or_init(|| regex::Regex::new(r"^FTG(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}
fn fte_re() -> &'static regex::Regex {
    FTE_RE.get_or_init(|| regex::Regex::new(r"^FTE(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}
fn fmea_re() -> &'static regex::Regex {
    FMEA_RE.get_or_init(|| regex::Regex::new(r"^FMEA(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}
fn fm_re() -> &'static regex::Regex {
    FM_RE.get_or_init(|| regex::Regex::new(r"^FM(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

/// Returns true for FT-* IDs (FaultTree).
pub fn is_ft_id(s: &str) -> bool { ft_re().is_match(s) || extra_matches("FT", s) }
/// Returns true for FTG-* IDs (FaultTreeGate).
pub fn is_ftg_id(s: &str) -> bool { ftg_re().is_match(s) || extra_matches("FTG", s) }
/// Returns true for FTE-* IDs (FaultTreeEvent).
pub fn is_fte_id(s: &str) -> bool { fte_re().is_match(s) || extra_matches("FTE", s) }
/// Returns true for FMEA-* IDs (FMEASheet).
pub fn is_fmea_id(s: &str) -> bool { fmea_re().is_match(s) || extra_matches("FMEA", s) }
/// Returns true for FM-* IDs (FMEAEntry).
pub fn is_fm_id(s: &str) -> bool { fm_re().is_match(s) || extra_matches("FM", s) }

fn at_re() -> &'static regex::Regex {
    AT_RE.get_or_init(|| regex::Regex::new(r"^AT(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}
fn atg_re() -> &'static regex::Regex {
    ATG_RE.get_or_init(|| regex::Regex::new(r"^ATG(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}
fn ats_re() -> &'static regex::Regex {
    ATS_RE.get_or_init(|| regex::Regex::new(r"^ATS(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

fn arg_re() -> &'static regex::Regex {
    ARG_RE.get_or_init(|| regex::Regex::new(r"^ARG(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}
fn aou_re() -> &'static regex::Regex {
    AOU_RE.get_or_init(|| regex::Regex::new(r"^AOU(-[A-Z0-9]{2,12})+-[0-9]{3,}$").unwrap())
}

/// Returns true for ARG-* IDs (Argument).
pub fn is_arg_id(s: &str) -> bool { arg_re().is_match(s) || extra_matches("ARG", s) }
/// Returns true for AOU-* IDs (AssumptionOfUse).
pub fn is_aou_id(s: &str) -> bool { aou_re().is_match(s) || extra_matches("AOU", s) }

/// Returns true for AT-* IDs (AttackTree).
pub fn is_at_id(s: &str) -> bool { at_re().is_match(s) || extra_matches("AT", s) }
/// Returns true for ATG-* IDs (AttackTreeGate).
pub fn is_atg_id(s: &str) -> bool { atg_re().is_match(s) || extra_matches("ATG", s) }
/// Returns true for ATS-* IDs (AttackStep).
pub fn is_ats_id(s: &str) -> bool { ats_re().is_match(s) || extra_matches("ATS", s) }

/// Returns true for ADR-* IDs.
pub fn is_adr_id(s: &str) -> bool {
    adr_re().is_match(s) || extra_matches("ADR", s)
}

/// Returns true for RR-* IDs (ReviewRecord).
pub fn is_rr_id(s: &str) -> bool {
    rr_re().is_match(s) || extra_matches("RR", s)
}

/// Returns true for TRD-* IDs (TradeStudy).
pub fn is_trd_id(s: &str) -> bool {
    trd_re().is_match(s) || extra_matches("TRD", s)
}

/// Returns true for ZN-* IDs (Zone).
pub fn is_zn_id(s: &str) -> bool {
    zn_re().is_match(s) || extra_matches("ZN", s)
}

/// Returns true for CD-* IDs (Conduit).
pub fn is_cd_id(s: &str) -> bool {
    cd_re().is_match(s) || extra_matches("CD", s)
}

/// Returns true for CM-* IDs (ConfirmationMeasure).
pub fn is_cm_id(s: &str) -> bool {
    cm_re().is_match(s) || extra_matches("CM", s)
}

/// Returns true for REQ-* IDs.
pub fn is_req_id(s: &str) -> bool {
    req_re().is_match(s) || extra_matches("REQ", s)
}

/// Returns true for TC-* IDs.
pub fn is_tc_id(s: &str) -> bool {
    tc_re().is_match(s) || extra_matches("TC", s)
}

/// Returns true for TP-* IDs (TestPlan).
pub fn is_test_plan_id(s: &str) -> bool {
    tp_re().is_match(s) || extra_matches("TP", s)
}

/// Returns true for CONF-* IDs.
pub fn is_conf_id(s: &str) -> bool {
    conf_re().is_match(s) || extra_matches("CONF", s)
}

pub struct Resolver {
    /// Index by qualified name.
    pub by_qname: HashMap<String, usize>,
    /// Index by stable id field (native Requirement, TestCase, and Configuration only).
    pub by_id: HashMap<String, usize>,
    /// Index by `name:` display field.  Only the first occurrence is stored when
    /// names are not unique (display names are not required to be unique across the
    /// model, so this is a best-effort fallback for short-name references).
    pub by_name: HashMap<String, usize>,
}

impl Resolver {
    pub fn new(elements: &[RawElement]) -> Self {
        let mut by_qname = HashMap::new();
        let mut by_id = HashMap::new();
        let mut by_name = HashMap::new();

        for (i, e) in elements.iter().enumerate() {
            by_qname.insert(e.qualified_name.clone(), i);
            if let Some(ref id) = e.frontmatter.id {
                if is_stable_id(id) {
                    by_id.insert(id.clone(), i);
                }
            }
            if let Some(ref name) = e.frontmatter.name {
                by_name.entry(name.clone()).or_insert(i);
            }
        }

        Self { by_qname, by_id, by_name }
    }

    pub fn get<'a>(&self, elements: &'a [RawElement], qname: &str) -> Option<&'a RawElement> {
        self.by_qname.get(qname).map(|&i| &elements[i])
    }

    pub fn get_by_id<'a>(&self, elements: &'a [RawElement], id: &str) -> Option<&'a RawElement> {
        self.by_id.get(id).map(|&i| &elements[i])
    }

    /// Resolve a cross-reference string.
    /// Resolution order:
    ///   1. Stable-ID patterns (REQ-*, TC-*, CONF-*, …) → by_id index.
    ///   2. Qualified name (contains `::` or matches a known qname) → by_qname index.
    ///   3. Display name fallback → by_name index (covers short-name references such as
    ///      `supertype: HwBase` when the element's `name:` is "HwBase").
    pub fn resolve_ref<'a>(&self, elements: &'a [RawElement], r: &str) -> Option<&'a RawElement> {
        if is_stable_id(r) {
            return self.get_by_id(elements, r);
        }
        if let Some(elem) = self.get(elements, r) {
            return Some(elem);
        }
        // Fallback: try the display name index
        self.by_name.get(r).map(|&i| &elements[i])
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

    /// True if `elem` is a native TestPlan (type: TestPlan).
    pub fn is_test_plan(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::TestPlan))
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

    pub fn is_argument(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::Argument))
    }

    pub fn is_assumption_of_use(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::AssumptionOfUse))
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

    pub fn is_fault_tree_gate(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::FaultTreeGate))
    }

    pub fn is_fault_tree_event(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::FaultTreeEvent))
    }

    pub fn is_fmea_sheet(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::FMEASheet))
    }

    pub fn is_attack_tree_gate(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::AttackTreeGate))
    }

    pub fn is_attack_step(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::AttackStep))
    }

    pub fn is_tara_sheet(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::TARASheet))
    }

    pub fn is_confirmation_measure(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::ConfirmationMeasure))
    }

    pub fn is_asset(elem: &RawElement) -> bool {
        matches!(elem.frontmatter.element_type, Some(ElementType::Asset))
    }
}
