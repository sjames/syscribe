//! The normalised result record (REQ-TRS-RESREC-001): a verdict, optional measurements and provenance for one
//! TestCase or one of its Gherkin scenarios, as read by `ingest-results --format records`.
//!
//! Everything here is validation. A malformed record is a hard error naming the record, so a hand- or
//! tool-written report cannot be silently swallowed into a smaller result set.

use serde::{Deserialize, Serialize};

use crate::expr::{parse_value, Value};
use crate::results::{one_line, Verdict};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResultRecord {
    /// A TestCase id, qualified name or `extRef` value (joined by REQ-TRS-RESJOIN-001).
    pub test_case: String,
    /// A `Scenario:` title of that TestCase; absent means the verdict of the whole TestCase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario: Option<String>,
    pub verdict: Verdict,
    /// Required (non-empty) for `waived`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    /// Opaque, whatever the run did; kept as given.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub measurements: Vec<Measurement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
    /// The condition point(s) this run exercised: axis → point name (`{"ambient": "hot"}`), the coordinates a
    /// `TestPlan`'s `conditions:` declare (ADR-SYS-COVERAGE-001). Part of the record's key, so results at
    /// different conditions never replace each other. Absent means unconditioned.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub condition: std::collections::BTreeMap<String, String>,
    /// Set by the tool when the record is joined to its TestCase (REQ-TRS-RESJOIN-001): the hash of that TestCase
    /// as it was. Whatever a report says here is discarded; a producer cannot vouch for freshness.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tc_hash: Option<String>,
    /// The same for each requirement the TestCase verifies, keyed by its canonical reference.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub req_hashes: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Measurement {
    pub name: String,
    /// A quantity in the expression language, with its unit (`"4.2 ms"`).
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<Limit>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limit {
    /// One of `<`, `<=`, `>`, `>=`, `==`.
    pub op: String,
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Provenance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<Build>,
    /// A `TestEnvironment` id or qualified name; stored by id once resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    /// The unit under test (GH #501): which serial or sample this run was on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<Unit>,
    /// The program that produced the report, as it names itself (`Robot 7.4.2`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    /// Referenced by URI and hash, never copied.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artefacts: Vec<Artefact>,
}

/// The unit (serial, sample) a record was measured on. Units are not model elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    /// Free text: a serial number or sample id.
    pub id: String,
    /// The part definition the unit is an instance of; resolved at ingest and stored by qualified name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lot: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Build {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artefact {
    pub uri: String,
    /// `sha256:<64 hex>` or `blake3:<64 hex>`.
    pub hash: String,
}

/// What identifies a record: test, scenario, environment, condition, unit.
pub type RecordKey = (String, Option<String>, Option<String>, Vec<(String, String)>, Option<String>);

impl ResultRecord {
    /// The key a later ingest supersedes: test, scenario, environment, condition and unit (ADR-SYS-COVERAGE-001),
    /// so a result at another condition, or on another unit, is a different result rather than a repeat.
    pub fn key(&self) -> RecordKey {
        (
            self.test_case.clone(),
            self.scenario.clone(),
            self.provenance.as_ref().and_then(|p| p.environment.clone()),
            self.condition.iter().map(|(a, p)| (a.clone(), p.clone())).collect(),
            self.provenance.as_ref().and_then(|p| p.unit.as_ref().map(|u| u.id.clone())),
        )
    }
}

/// How bad a verdict is when several apply to one test: used to combine results from several rigs and sources.
pub fn badness(v: Verdict) -> u8 {
    match v {
        Verdict::Fail => 7,
        Verdict::Flaky => 6,
        Verdict::Blocked => 5,
        Verdict::Inconclusive => 4,
        Verdict::NotRun => 3,
        Verdict::Ignored => 2,
        Verdict::Waived => 1,
        Verdict::Pass => 0,
    }
}

/// The worse of two verdicts.
pub fn worst(a: Verdict, b: Verdict) -> Verdict {
    if badness(b) > badness(a) {
        b
    } else {
        a
    }
}

/// Parse and validate a records file. Errors name the offending record (zero-based index and test case).
pub fn parse(text: &str) -> Result<Vec<ResultRecord>, String> {
    // A byte-order mark is what several Windows tools write in front of JSON.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let value: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("records: not valid JSON: {e}"))?;
    let serde_json::Value::Array(items) = value else {
        return Err("records: expected a JSON array of records".to_string());
    };
    if items.is_empty() {
        return Err("records: no records found (empty array)".to_string());
    }
    let mut out = Vec::with_capacity(items.len());
    for (i, item) in items.into_iter().enumerate() {
        let label = item.get("testCase").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let mut rec: ResultRecord = serde_json::from_value(item).map_err(|e| format!("records: record {i} ('{label}'): {e}"))?;
        validate(&mut rec).map_err(|e| format!("records: record {i} ('{}'): {e}", rec.test_case))?;
        out.push(rec);
    }
    Ok(out)
}

/// An artefact reference for a local file: its `file://` URI and the blake3 hash of its contents. The file is read to
/// be hashed and never copied.
pub fn file_artefact(path: &str) -> Result<Artefact, String> {
    let abs = std::fs::canonicalize(path).map_err(|e| format!("artefact '{path}' cannot be read: {e}"))?;
    if !abs.is_file() {
        return Err(format!("artefact '{path}' is not a file; only a file can be referenced by its hash"));
    }
    let bytes = std::fs::read(&abs).map_err(|e| format!("artefact '{path}' cannot be read: {e}"))?;
    Ok(Artefact { uri: format!("file://{}", abs.display()), hash: format!("blake3:{}", blake3::hash(&bytes).to_hex()) })
}

/// Keep the last record of each (test case, scenario, environment) key, in the order of those last occurrences: one
/// file is one run, and a repeated key there is a retry or a correction, never two results.
pub fn keep_last_per_key(records: Vec<ResultRecord>) -> Vec<ResultRecord> {
    let mut last: std::collections::HashMap<RecordKey, usize> = std::collections::HashMap::new();
    for (i, r) in records.iter().enumerate() {
        last.insert(r.key(), i);
    }
    records.into_iter().enumerate().filter(|(i, r)| last.get(&r.key()) == Some(i)).map(|(_, r)| r).collect()
}

/// Validate records an adapter built, with the checks a records file gets, so no source can store what the record
/// format would refuse.
pub fn validate_all(records: &mut [ResultRecord]) -> Result<(), String> {
    for (i, r) in records.iter_mut().enumerate() {
        let label = r.test_case.clone();
        validate(r).map_err(|e| format!("records: record {i} ('{label}'): {e}"))?;
    }
    Ok(())
}

fn validate(r: &mut ResultRecord) -> Result<(), String> {
    r.tc_hash = None;
    r.req_hashes.clear();
    r.test_case = one_line(&r.test_case);
    if r.test_case.is_empty() {
        return Err("empty or missing testCase".to_string());
    }
    if let Some(s) = &mut r.scenario {
        *s = one_line(s);
        if s.is_empty() {
            return Err("scenario is empty (leave it out for a whole-TestCase verdict)".to_string());
        }
    }
    if let Some(t) = &mut r.rationale {
        *t = one_line(t);
        if t.is_empty() {
            r.rationale = None;
        }
    }
    if r.verdict == Verdict::Waived && r.rationale.is_none() {
        return Err("a waived verdict needs a non-empty rationale".to_string());
    }
    for m in &mut r.measurements {
        m.name = one_line(&m.name);
        check_measurement(m, r.verdict)?;
    }
    if let Some(p) = &mut r.provenance {
        check_provenance(p)?;
    }
    let mut cleaned = std::collections::BTreeMap::new();
    for (axis, point) in std::mem::take(&mut r.condition) {
        let (axis, point) = (one_line(&axis), one_line(&point));
        let basic = axis.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && axis.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !basic {
            return Err(format!("condition axis '{axis}' is not a basic name (letters, digits, `_`)"));
        }
        if point.is_empty() {
            return Err(format!("condition '{axis}' has an empty point"));
        }
        cleaned.insert(axis, point);
    }
    r.condition = cleaned;
    Ok(())
}

fn quantity(text: &str, what: &str) -> Result<crate::expr::Quantity, String> {
    match parse_value(text) {
        Some(Value::Num(q)) => Ok(q),
        Some(_) => Err(format!("{what} '{text}' is not a number or quantity")),
        None => Err(format!("{what} '{text}' is not a quantity (write it with its unit, for example \"4.2 ms\")")),
    }
}

fn check_measurement(m: &Measurement, verdict: Verdict) -> Result<(), String> {
    if m.name.is_empty() {
        return Err("a measurement has no name".to_string());
    }
    let value = quantity(&m.value, &format!("measurement '{}' value", m.name))?;
    let Some(limit) = &m.limit else { return Ok(()) };
    if !["<", "<=", ">", ">=", "=="].contains(&limit.op.as_str()) {
        return Err(format!("measurement '{}': unknown limit operator '{}' (expected < <= > >= ==)", m.name, limit.op));
    }
    let bound = quantity(&limit.value, &format!("measurement '{}' limit", m.name))?;
    if value.dim != bound.dim {
        return Err(format!("measurement '{}': value '{}' and limit '{}' differ in dimension", m.name, m.value, limit.value));
    }
    let holds = match limit.op.as_str() {
        "<" => value.v < bound.v,
        "<=" => value.v <= bound.v,
        ">" => value.v > bound.v,
        ">=" => value.v >= bound.v,
        _ => (value.v - bound.v).abs() <= 1e-9 * bound.v.abs().max(value.v.abs()).max(f64::MIN_POSITIVE),
    };
    if !holds && verdict == Verdict::Pass {
        return Err(format!(
            "measurement '{}' is {} but its limit is {} {}: a pass cannot violate its own limit",
            m.name, m.value, limit.op, limit.value
        ));
    }
    Ok(())
}

fn check_provenance(p: &mut Provenance) -> Result<(), String> {
    for (field, v) in [("startedAt", &p.started_at), ("finishedAt", &p.finished_at)] {
        if let Some(t) = v {
            if !is_rfc3339(t) {
                return Err(format!("{field} '{t}' is not an RFC 3339 timestamp (for example 2026-10-01T08:00:00Z)"));
            }
        }
    }
    if let Some(o) = &mut p.operator {
        *o = one_line(o);
    }
    if let Some(t) = &mut p.tool {
        *t = one_line(t);
    }
    if let Some(u) = &mut p.unit {
        u.id = one_line(&u.id);
        if u.id.is_empty() {
            return Err("unit has no id".to_string());
        }
        for v in [&mut u.of, &mut u.lot].into_iter().flatten() {
            *v = one_line(v);
            if v.is_empty() {
                return Err("unit.of / unit.lot is empty".to_string());
            }
        }
    }
    if let Some(e) = &mut p.environment {
        *e = one_line(e);
        if e.is_empty() {
            return Err("environment is empty".to_string());
        }
    }
    if let Some(b) = &mut p.build {
        for v in [&mut b.commit, &mut b.version].into_iter().flatten() {
            *v = one_line(v);
        }
    }
    for a in &mut p.artefacts {
        a.uri = one_line(&a.uri);
        if a.uri.is_empty() {
            return Err("an artefact has no uri".to_string());
        }
        let ok = ["sha256:", "blake3:"].iter().any(|alg| a.hash.strip_prefix(alg).is_some_and(|h| h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit())));
        if !ok {
            return Err(format!("artefact '{}': hash '{}' must be sha256:<64 hex digits> or blake3:<64 hex digits>", a.uri, a.hash));
        }
    }
    Ok(())
}

/// `YYYY-MM-DDTHH:MM:SS[.fraction](Z|+HH:MM|-HH:MM)`, with calendar ranges checked.
pub fn is_rfc3339(t: &str) -> bool {
    let b = t.as_bytes();
    let num = |from: usize, len: usize| -> Option<u32> {
        let s = t.get(from..from + len)?;
        s.chars().all(|c| c.is_ascii_digit()).then(|| s.parse().ok()).flatten()
    };
    if b.len() < 20 || b[4] != b'-' || b[7] != b'-' || !(b[10] == b'T' || b[10] == b't') || b[13] != b':' || b[16] != b':' {
        return false;
    }
    let (Some(y), Some(mo), Some(d), Some(h), Some(mi), Some(s)) = (num(0, 4), num(5, 2), num(8, 2), num(11, 2), num(14, 2), num(17, 2)) else {
        return false;
    };
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let dim = match mo {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 28 + leap as u32,
        _ => return false,
    };
    if d < 1 || d > dim || h > 23 || mi > 59 || s > 60 {
        return false;
    }
    let mut i = 19;
    if b[i] == b'.' {
        let start = i + 1;
        i = start;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == start {
            return false;
        }
    }
    match t.get(i..) {
        Some("Z") | Some("z") => true,
        Some(z) if z.len() == 6 && (z.starts_with('+') || z.starts_with('-')) && z.as_bytes()[3] == b':' => {
            matches!((z[1..3].parse::<u32>(), z[4..6].parse::<u32>()), (Ok(h), Ok(m)) if h < 24 && m < 60)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339_accepts_and_rejects() {
        for ok in ["2026-10-01T08:00:00Z", "2026-10-01T08:00:05.250+02:00", "2024-02-29T23:59:59-07:30"] {
            assert!(is_rfc3339(ok), "{ok}");
        }
        for bad in ["yesterday", "2026-10-01", "2026-13-01T00:00:00Z", "2025-02-29T00:00:00Z", "2026-10-01T24:00:00Z", "2026-10-01T08:00:00", "2026-10-01T08:00:00.Z", "2026-10-01T08:00:00+0200"] {
            assert!(!is_rfc3339(bad), "{bad}");
        }
    }

    #[test]
    fn worst_orders_fail_over_everything_and_pass_last() {
        assert_eq!(worst(Verdict::Pass, Verdict::Fail), Verdict::Fail);
        assert_eq!(worst(Verdict::Waived, Verdict::Pass), Verdict::Waived);
        assert_eq!(worst(Verdict::Blocked, Verdict::Waived), Verdict::Blocked);
        assert_eq!(worst(Verdict::Flaky, Verdict::Blocked), Verdict::Flaky);
    }

    #[test]
    fn a_repeated_key_keeps_only_the_last_record_in_the_order_of_last_occurrences() {
        let recs = parse(
            r#"[{"testCase":"A","verdict":"fail"},{"testCase":"B","verdict":"pass"},{"testCase":"A","verdict":"pass"},
                {"testCase":"A","scenario":"s","verdict":"fail"},{"testCase":"B","provenance":{"environment":"E"},"verdict":"fail"}]"#,
        )
        .unwrap();
        let kept = keep_last_per_key(recs);
        let shown: Vec<_> = kept.iter().map(|r| format!("{}/{:?}/{:?}", r.test_case, r.scenario, r.verdict)).collect();
        assert_eq!(shown, ["B/None/Pass", "A/None/Pass", "A/Some(\"s\")/Fail", "B/None/Fail"], "{shown:?}");
    }

    #[test]
    fn a_byte_order_mark_is_skipped() {
        assert_eq!(parse("\u{feff}[{\"testCase\":\"A\",\"verdict\":\"pass\"}]").unwrap().len(), 1);
    }

    #[test]
    fn unknown_fields_are_rejected_so_typos_do_not_pass() {
        let e = parse(r#"[{"testCase":"TC-1","verdict":"pass","verdit":"fail"}]"#).unwrap_err();
        assert!(e.contains("verdit"), "{e}");
    }

    fn one(j: &str) -> Result<Vec<ResultRecord>, String> {
        let mut v = parse(j)?;
        validate_all(&mut v)?;
        Ok(v)
    }

    #[test]
    fn condition_and_unit_are_read_and_key_the_record() {
        let v = one(r#"[{"testCase":"T","verdict":"pass","condition":{"ambient":"hot"},"provenance":{"unit":{"id":"SN-1","of":"P::Heater","lot":"L7"}}}]"#).unwrap();
        assert_eq!(v[0].condition["ambient"], "hot");
        assert_eq!(v[0].provenance.as_ref().unwrap().unit.as_ref().unwrap().id, "SN-1");
        assert_eq!(v[0].key().3, vec![("ambient".to_string(), "hot".to_string())]);
        assert_eq!(v[0].key().4.as_deref(), Some("SN-1"));
    }

    #[test]
    fn another_condition_or_unit_is_another_result_not_a_repeat() {
        let recs = one(
            r#"[{"testCase":"T","verdict":"pass","condition":{"ambient":"cold"}},
                {"testCase":"T","verdict":"fail","condition":{"ambient":"hot"}},
                {"testCase":"T","verdict":"pass","provenance":{"unit":{"id":"SN-1"}}},
                {"testCase":"T","verdict":"pass","provenance":{"unit":{"id":"SN-2"}}},
                {"testCase":"T","verdict":"pass"},
                {"testCase":"T","verdict":"fail"}]"#,
        )
        .unwrap();
        let kept = keep_last_per_key(recs);
        assert_eq!(kept.len(), 5, "only the two unconditioned, unit-less records are one key");
        assert_eq!(kept.iter().filter(|r| r.condition.is_empty() && r.provenance.is_none()).next().unwrap().verdict, Verdict::Fail, "the later one wins");
    }

    #[test]
    fn malformed_condition_or_unit_is_rejected() {
        for bad in [
            r#"[{"testCase":"T","verdict":"pass","condition":{"my axis":"hot"}}]"#,
            r#"[{"testCase":"T","verdict":"pass","condition":{"ambient":""}}]"#,
            r#"[{"testCase":"T","verdict":"pass","condition":{"ambient":1}}]"#,
            r#"[{"testCase":"T","verdict":"pass","provenance":{"unit":{"id":""}}}]"#,
            r#"[{"testCase":"T","verdict":"pass","provenance":{"unit":{"of":"P"}}}]"#,
            r#"[{"testCase":"T","verdict":"pass","provenance":{"unit":{"id":"S","colour":"red"}}}]"#,
        ] {
            assert!(one(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn records_without_the_new_fields_still_read_and_round_trip_unchanged() {
        let v = one(r#"[{"testCase":"T","verdict":"pass"}]"#).unwrap();
        assert!(v[0].condition.is_empty());
        let j = serde_json::to_string(&v[0]).unwrap();
        assert!(!j.contains("condition") && !j.contains("unit"), "{j}");
    }
}
