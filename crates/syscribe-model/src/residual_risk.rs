//! Campaign evidence and residual-risk decisions (GH #341): `inject --record` keeps each fault-injection campaign in
//! `<model_root>/.syscribe/campaigns.json`, and a `type: ResidualRisk` element (`RISK-*`) records what was decided
//! about a fault set the design does not survive. The validator ties the two together: a defeat in the latest
//! recorded campaign that no decision names is `W940`, a decision the campaign contradicts is `W941`, a decision
//! taken on a different composite is `W942`, an expired one `W943`, and a recorded campaign the composite has since
//! moved away from `W944`.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::element::{ElementType, RawElement};
use crate::fault_campaign::Campaign;
use crate::resolver::Resolver;
use crate::sm_scen::Scenario;

/// The decision statuses of a `ResidualRisk`.
pub const STATUSES: &[&str] = &["proposed", "accepted", "mitigated", "rejected"];

/// How many records each campaign (subject and configuration) keeps.
pub const HISTORY: usize = 20;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvariantRecord {
    pub scenario: String,
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindingRecord {
    pub scenario: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetRecord {
    pub label: String,
    pub origin: String,
    pub faults: Vec<String>,
    /// `survived`, `defeated` or `error`.
    pub status: String,
    /// Every fault of the set is in a part that is not the product (`role: environment`/`external`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub environment: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub defeats: Vec<FindingRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<FindingRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotRunRecord {
    pub origin: String,
    pub events: Vec<String>,
    pub reason: String,
}

/// One recorded campaign. Everything but `recorded_at_unix` is a function of the model, so two runs on the same
/// model record the same thing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignRecord {
    /// The run id (`--run`); absent for an unnamed run, which replaces the previous unnamed one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    /// What `inject` was given: a TestCase id or the composite.
    pub subject: String,
    pub composite: String,
    /// The Configuration (`--config`) the campaign ran on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    pub composite_hash: String,
    pub invariants: Vec<InvariantRecord>,
    pub sets: Vec<SetRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub not_run: Vec<NotRunRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cut_set_order: Option<usize>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub cut_sets_truncated: bool,
    /// The syscribe version that ran the campaign.
    pub engine: String,
    pub recorded_at_unix: u64,
}

impl CampaignRecord {
    pub fn from_campaign(subject: &str, run: Option<&str>, config: Option<&str>, campaign: &Campaign, invariants: &[Scenario], now: u64) -> Self {
        CampaignRecord {
            run: run.map(str::to_string),
            subject: subject.to_string(),
            composite: campaign.composite.clone(),
            config: config.map(str::to_string),
            composite_hash: campaign.hash.clone(),
            invariants: invariants.iter().map(|s| InvariantRecord { scenario: s.title.clone(), hash: scenario_hash(s) }).collect(),
            sets: campaign
                .sets
                .iter()
                .map(|s| SetRecord {
                    label: s.label.clone(),
                    origin: s.origin.clone(),
                    faults: s.faults.clone(),
                    status: s.status.to_string(),
                    environment: s.environment,
                    defeats: s.defeats.iter().map(|d| FindingRecord { scenario: d.scenario.clone(), message: d.message.clone() }).collect(),
                    errors: s.errors.iter().map(|d| FindingRecord { scenario: d.scenario.clone(), message: d.message.clone() }).collect(),
                })
                .collect(),
            not_run: campaign.not_injectable.iter().map(|n| NotRunRecord { origin: n.origin.clone(), events: n.events.clone(), reason: n.reason.clone() }).collect(),
            cut_set_order: campaign.cut_set_order,
            cut_sets_truncated: campaign.cut_sets_truncated,
            engine: env!("CARGO_PKG_VERSION").to_string(),
            recorded_at_unix: now,
        }
    }

    /// The campaign's key in the sidecar: one history per subject and configuration.
    pub fn key(&self) -> String {
        key(&self.subject, self.config.as_deref())
    }

    /// The record without its timestamp, for comparing two runs.
    pub fn content(&self) -> CampaignRecord {
        CampaignRecord { recorded_at_unix: 0, ..self.clone() }
    }

    pub fn count(&self, status: &str) -> usize {
        self.sets.iter().filter(|s| s.status == status).count()
    }
}

/// The sidecar key of a campaign.
pub fn key(subject: &str, config: Option<&str>) -> String {
    match config {
        Some(c) => format!("{subject}@{c}"),
        None => subject.to_string(),
    }
}

/// A stable hash of an invariant scenario: its title, tags and steps.
pub fn scenario_hash(s: &Scenario) -> String {
    let steps: Vec<String> = s.steps.iter().map(|st| format!("{:?} {}", st.kw, st.text)).collect();
    let text = serde_json::json!({"title": s.title, "tags": s.tags, "steps": steps}).to_string();
    blake3::hash(text.as_bytes()).to_hex().to_string()
}

pub fn sidecar(model_root: &Path) -> PathBuf {
    model_root.join(".syscribe").join("campaigns.json")
}

/// The recorded campaigns by key, the latest first.
pub fn load(model_root: &Path) -> BTreeMap<String, Vec<CampaignRecord>> {
    std::fs::read_to_string(sidecar(model_root)).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

/// Record a campaign: it goes first in its history, replacing an earlier record of the same run id (or the earlier
/// unnamed record), and the history keeps the latest [`HISTORY`] records.
pub fn record(model_root: &Path, rec: CampaignRecord) -> std::io::Result<PathBuf> {
    let mut all = load(model_root);
    let list = all.entry(rec.key()).or_default();
    list.retain(|r| r.run != rec.run);
    list.insert(0, rec);
    list.truncate(HISTORY);
    let path = sidecar(model_root);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(&all).expect("serialises") + "\n")?;
    Ok(path)
}

/// The difference between two records of a campaign: sets whose verdict changed, and sets only one of them ran.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDiff {
    /// `(set, before, after)`.
    pub changed: Vec<(String, String, String)>,
    pub added: Vec<(String, String)>,
    pub removed: Vec<(String, String)>,
    pub composite_changed: bool,
    pub invariants_changed: bool,
}

pub fn diff(a: &CampaignRecord, b: &CampaignRecord) -> RecordDiff {
    let by = |r: &CampaignRecord| r.sets.iter().map(|s| (s.label.clone(), s.status.clone())).collect::<BTreeMap<_, _>>();
    let (ma, mb) = (by(a), by(b));
    let mut d = RecordDiff { composite_changed: a.composite_hash != b.composite_hash, invariants_changed: a.invariants != b.invariants, ..Default::default() };
    for (k, va) in &ma {
        match mb.get(k) {
            Some(vb) if vb != va => d.changed.push((k.clone(), va.clone(), vb.clone())),
            Some(_) => {}
            None => d.removed.push((k.clone(), va.clone())),
        }
    }
    for (k, vb) in &mb {
        if !ma.contains_key(k) {
            d.added.push((k.clone(), vb.clone()));
        }
    }
    d
}

/// The fault ids a `ResidualRisk` names, resolved to the ids of the FMEA rows / fault-tree events (a qualified name
/// resolves to the element's id), sorted.
pub fn fault_set_of(elements: &[RawElement], resolver: &Resolver, risk: &RawElement) -> Vec<String> {
    let mut ids: Vec<String> = risk
        .frontmatter
        .fault_set
        .iter()
        .flatten()
        .map(|r| resolver.resolve_ref(elements, r).and_then(|e| e.frontmatter.id.clone()).unwrap_or_else(|| r.clone()))
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

fn sorted(v: &[String]) -> Vec<String> {
    let mut v = v.to_vec();
    v.sort();
    v.dedup();
    v
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    if !(b.len() == 10 && b.iter().enumerate().all(|(i, c)| if i == 4 || i == 7 { *c == b'-' } else { c.is_ascii_digit() })) {
        return false;
    }
    let (y, m, d): (u32, u32, u32) = (s[0..4].parse().unwrap_or(0), s[5..7].parse().unwrap_or(0), s[8..10].parse().unwrap_or(0));
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let dim = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        _ => 0,
    };
    d >= 1 && d <= dim
}

/// Today's date (UTC) as `YYYY-MM-DD`.
pub fn today() -> String {
    let days = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() / 86_400).unwrap_or(0) as i64;
    // days since 1970-01-01 → civil date (Howard Hinnant's algorithm)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}")
}

/// The checks of one `ResidualRisk` on its own: `E942` (malformed), `E943` (a reference that resolves to nothing), `E944` (a
/// reference to the wrong kind of element), `W943` (expired). `users` is the `[users]` roster (empty: not checked).
pub fn element_findings(elements: &[RawElement], resolver: &Resolver, risk: &RawElement, users: &std::collections::HashMap<String, String>, today: &str) -> Vec<(&'static str, String)> {
    let fm = &risk.frontmatter;
    let mut out: Vec<(&'static str, String)> = Vec::new();
    for (field, present) in [("id", fm.id.is_some()), ("name", fm.name.is_some()), ("status", fm.status.is_some()), ("faultSet", fm.fault_set.as_ref().is_some_and(|v| !v.is_empty()))] {
        if !present {
            out.push(("E942", format!("`{field}` is required on ResidualRisk")));
        }
    }
    if let Some(id) = &fm.id {
        if !crate::resolver::is_risk_id(id) {
            out.push(("E942", format!("`id` '{id}' does not match the RISK-* pattern")));
        }
    }
    if let Some(s) = fm.status.as_deref() {
        if !STATUSES.contains(&s) {
            out.push(("E942", format!("ResidualRisk.status '{s}' must be one of {}", STATUSES.join(", "))));
        }
        if s == "accepted" && fm.approved_by.is_none() {
            out.push(("E942", "an accepted ResidualRisk names who accepted it in `approvedBy`".into()));
        }
        if s == "mitigated" && fm.mitigated_by.as_ref().is_none_or(|v| v.is_empty()) {
            out.push(("E942", "a mitigated ResidualRisk names the mitigation in `mitigatedBy`".into()));
        }
    }
    for (field, v) in [("reviewDate", &fm.review_date), ("expires", &fm.expires)] {
        if let Some(v) = v {
            if !is_date(v) {
                out.push(("E942", format!("ResidualRisk.{field} '{v}' must be a YYYY-MM-DD date")));
            }
        }
    }
    if let Some(who) = &fm.approved_by {
        if !crate::resolver::is_valid_username(who) {
            out.push(("E942", format!("ResidualRisk.approvedBy '{who}' is not a valid username (lowercase, starting with a letter or underscore, then lowercase letters/digits/underscore/hyphen, max 32 chars)")));
        } else if !users.is_empty() && !users.contains_key(who) {
            out.push(("E942", format!("ResidualRisk.approvedBy '{who}' is not in the [users] roster of .syscribe.toml")));
        }
    }
    if let Some(h) = &fm.campaign_hash {
        if h.len() < 12 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
            out.push(("E942", format!("ResidualRisk.campaignHash '{h}' must be the composite hash `inject` prints (at least 12 hex digits)")));
        }
    }
    // `E943` a reference that resolves to nothing, `E944` one that resolves to the wrong kind of element
    let kind = |r: &str, ok: &dyn Fn(&ElementType) -> bool| -> Option<(&'static str, String)> {
        match resolver.resolve_ref(elements, r) {
            None => Some(("E943", format!("'{r}' does not resolve to any model element"))),
            Some(e) if e.frontmatter.element_type.as_ref().is_some_and(ok) => None,
            Some(e) => Some(("E944", format!("'{r}' is a {}", e.frontmatter.element_type.as_ref().map(|t| t.name()).unwrap_or("element of no type")))),
        }
    };
    for r in fm.fault_set.iter().flatten() {
        if let Some((code, m)) = kind(r, &|t| matches!(t, ElementType::FMEAEntry | ElementType::FaultTreeEvent)) {
            out.push((code, format!("faultSet entry {m}; it must be an FMEA row (FMEAEntry) or a fault-tree event (FaultTreeEvent)")));
        }
    }
    for r in fm.goal.iter().flatten() {
        if let Some((code, m)) = kind(r, &|t| matches!(t, ElementType::SafetyGoal)) {
            out.push((code, format!("goal {m}; it must be a SafetyGoal")));
        }
    }
    for r in fm.mitigated_by.iter().flatten() {
        if let Some((code, m)) = kind(r, &|t| matches!(t, ElementType::Requirement | ElementType::SafetyMechanism)) {
            out.push((code, format!("mitigatedBy {m}; it must be a Requirement or a SafetyMechanism")));
        }
    }
    if let Some(e) = fm.expires.as_deref().filter(|e| is_date(e)) {
        if e < today && !matches!(fm.status.as_deref(), Some("rejected") | Some("mitigated")) {
            out.push(("W943", format!("the decision expired on {e}: review it and set a new `expires`")));
        }
    }
    out
}

/// The checks that tie decisions to the latest recorded campaigns (`W940`, `W941`, `W942`, `W944`), as
/// `(code, file, message)`. `config` is the Configuration id being validated (`None`: the base model, which is judged
/// by the campaigns recorded without `--config`). Dormant without recorded campaigns.
pub fn campaign_findings(elements: &[RawElement], resolver: &Resolver, model_root: &Path, config: Option<&str>) -> Vec<(&'static str, String, String)> {
    let all = load(model_root);
    let latest: Vec<&CampaignRecord> = all.values().filter_map(|l| l.first()).filter(|r| r.config.as_deref() == config).collect();
    let mut out = Vec::new();
    if latest.is_empty() {
        return out;
    }
    // the composite as it is now, built once per composite
    let mut current: BTreeMap<String, Option<String>> = BTreeMap::new();
    for r in &latest {
        current.entry(r.composite.clone()).or_insert_with(|| crate::system_run::SystemModel::build_in(elements, &r.composite, model_root).ok().map(|m| m.hash));
    }
    let file_of = |q: &str| resolver.get(elements, q).map(|e| e.file_path.clone()).unwrap_or_default();
    let risks: Vec<(&RawElement, Vec<String>)> = elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::ResidualRisk)))
        .map(|e| (e, fault_set_of(elements, resolver, e)))
        .collect();
    let label = |e: &RawElement| e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone());
    for r in &latest {
        let run = r.run.as_deref().map(|x| format!(" (run {x})")).unwrap_or_default();
        if let Some(Some(now)) = current.get(&r.composite) {
            if *now != r.composite_hash {
                out.push(("W944", file_of(&r.composite), format!(
                    "the recorded campaign of {}{run} ran on composite hash {} but {} now hashes to {}: its verdicts may no longer hold; run `syscribe inject {} --record` again",
                    r.subject, &r.composite_hash[..12.min(r.composite_hash.len())], r.composite, &now[..12.min(now.len())], r.subject)));
            }
        }
        for s in r.sets.iter().filter(|s| s.status == "defeated" && !s.environment) {
            let set = sorted(&s.faults);
            let named: Vec<&&RawElement> = risks.iter().filter(|(_, f)| *f == set).map(|(e, _)| e).collect();
            let deciding = named.iter().any(|e| matches!(e.frontmatter.status.as_deref(), Some("proposed") | Some("accepted")));
            let mitigated = named.iter().any(|e| e.frontmatter.status.as_deref() == Some("mitigated"));
            if deciding || mitigated {
                continue; // a mitigated decision on a set that still defeats is W941 below
            }
            let failing: Vec<&str> = s.defeats.iter().map(|d| d.scenario.as_str()).collect();
            let rejected: Vec<String> = named.iter().map(|e| label(e)).collect();
            out.push(("W940", file_of(&r.composite), format!(
                "fault set {} ({}) defeats {} in the recorded campaign of {}{run} and no ResidualRisk accepts it{}: fix the design or record the decision (`type: ResidualRisk`, `faultSet: [{}]`)",
                s.label, s.origin, failing.iter().map(|f| format!("'{f}'")).collect::<Vec<_>>().join(", "), r.subject,
                if rejected.is_empty() { String::new() } else { format!(" ({} rejected it)", rejected.join(", ")) },
                set.join(", "))));
        }
    }
    for (risk, set) in &risks {
        if set.is_empty() {
            continue;
        }
        let status = risk.frontmatter.status.as_deref().unwrap_or("");
        let hits: Vec<(&CampaignRecord, &SetRecord)> = latest.iter().flat_map(|r| r.sets.iter().filter(|s| sorted(&s.faults) == *set).map(move |s| (*r, s))).collect();
        let defeated: Vec<&str> = hits.iter().filter(|(_, s)| s.status == "defeated").map(|(r, _)| r.subject.as_str()).collect();
        let survived: Vec<&str> = hits.iter().filter(|(_, s)| s.status == "survived").map(|(r, _)| r.subject.as_str()).collect();
        match status {
            "proposed" | "accepted" if defeated.is_empty() && !survived.is_empty() => out.push(("W941", risk.file_path.clone(), format!(
                "the design now survives fault set {} in the recorded campaign of {}: the decision has nothing left to accept; set `status: mitigated` (naming the mitigation) or remove it",
                set.join(" + "), survived.join(", ")))),
            "mitigated" if !defeated.is_empty() => out.push(("W941", risk.file_path.clone(), format!(
                "status is mitigated but fault set {} still defeats the invariants in the recorded campaign of {}",
                set.join(" + "), defeated.join(", ")))),
            _ => {}
        }
        if let Some(h) = risk.frontmatter.campaign_hash.as_deref().filter(|h| h.len() >= 12) {
            let composites: BTreeSet<&str> = hits.iter().map(|(r, _)| r.composite.as_str()).collect();
            let now: Vec<(&str, &String)> = composites.iter().filter_map(|c| current.get(*c).and_then(|h| h.as_ref()).map(|h| (*c, h))).collect();
            if !now.is_empty() && !now.iter().any(|(_, n)| n.starts_with(&h.to_ascii_lowercase())) {
                out.push(("W942", risk.file_path.clone(), format!(
                    "the decision was taken on composite hash {h}, but {}: review it against the current campaign and update `campaignHash`",
                    now.iter().map(|(c, n)| format!("{c} now hashes to {}", &n[..12])).collect::<Vec<_>>().join(", "))));
            }
        }
    }
    out
}

/// What the safety case and `audit` show of the latest recorded campaigns.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub campaigns: Vec<CampaignSummary>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignSummary {
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    pub composite: String,
    pub sets: usize,
    pub survived: usize,
    pub defeated: usize,
    pub errors: usize,
    /// Defeated sets a `ResidualRisk` (proposed, accepted or mitigated) names: `(set, risk ids)`.
    pub explained: Vec<(String, Vec<String>)>,
    /// Defeated sets no decision names.
    pub unexplained: Vec<String>,
}

/// The Configuration id `config` names (an id or a qualified name), as campaign records key it.
pub fn config_id(elements: &[RawElement], config: &str) -> String {
    elements
        .iter()
        .find(|e| matches!(e.frontmatter.element_type, Some(ElementType::Configuration)) && (e.frontmatter.id.as_deref() == Some(config) || e.qualified_name == config))
        .map(|e| e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone()))
        .unwrap_or_else(|| config.to_string())
}

/// The latest recorded campaign of each subject recorded on `config` (`None`: recorded without `--config`).
pub fn summary(elements: &[RawElement], resolver: &Resolver, model_root: &Path, config: Option<&str>) -> Summary {
    let config = config.map(|c| config_id(elements, c));
    let risks: Vec<(String, String, Vec<String>)> = elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::ResidualRisk)))
        .map(|e| (e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone()), e.frontmatter.status.clone().unwrap_or_default(), fault_set_of(elements, resolver, e)))
        .collect();
    let mut campaigns = Vec::new();
    for list in load(model_root).values() {
        let Some(r) = list.first().filter(|r| r.config == config) else { continue };
        let (mut explained, mut unexplained) = (Vec::new(), Vec::new());
        for s in r.sets.iter().filter(|s| s.status == "defeated" && !s.environment) {
            let set = sorted(&s.faults);
            let by: Vec<String> = risks.iter().filter(|(_, st, f)| *f == set && st != "rejected").map(|(id, _, _)| id.clone()).collect();
            if by.is_empty() {
                unexplained.push(s.label.clone());
            } else {
                explained.push((s.label.clone(), by));
            }
        }
        campaigns.push(CampaignSummary {
            subject: r.subject.clone(),
            config: r.config.clone(),
            composite: r.composite.clone(),
            sets: r.sets.len(),
            survived: r.count("survived"),
            defeated: r.count("defeated"),
            errors: r.count("error"),
            explained,
            unexplained,
        });
    }
    Summary { campaigns }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates() {
        assert!(is_date("2026-02-28"));
        assert!(!is_date("2026-02-30"));
        assert!(!is_date("2026-2-3"));
        let t = today();
        assert!(is_date(&t), "{t}");
        assert!(t.as_str() > "2025-01-01");
    }

    #[test]
    fn record_keeps_history_and_replaces_runs() {
        let dir = std::env::temp_dir().join(format!("syscribe-rr-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let rec = |run: Option<&str>, status: &str, t: u64| CampaignRecord {
            run: run.map(str::to_string),
            subject: "TC-1".into(),
            composite: "C".into(),
            config: None,
            composite_hash: "ab".repeat(32),
            invariants: vec![],
            sets: vec![SetRecord { label: "F1".into(), origin: "single fault F1".into(), faults: vec!["F1".into()], status: status.into(), environment: false, defeats: vec![], errors: vec![] }],
            not_run: vec![],
            cut_set_order: None,
            cut_sets_truncated: false,
            engine: "x".into(),
            recorded_at_unix: t,
        };
        record(&dir, rec(None, "defeated", 1)).unwrap();
        record(&dir, rec(None, "survived", 2)).unwrap();
        record(&dir, rec(Some("r1"), "defeated", 3)).unwrap();
        let all = load(&dir);
        let l = &all["TC-1"];
        assert_eq!(l.len(), 2, "the second unnamed run replaces the first");
        assert_eq!(l[0].run.as_deref(), Some("r1"));
        let d = diff(&l[1], &l[0]);
        assert_eq!(d.changed, vec![("F1".to_string(), "survived".to_string(), "defeated".to_string())]);
        assert!(!d.composite_changed);
        assert_eq!(l[0].content(), rec(Some("r1"), "defeated", 0));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
