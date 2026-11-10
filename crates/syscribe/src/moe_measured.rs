//! Measured values for a MoE (GH #467, `REQ-TRS-MOEMEAS-001`): the value of a Measure of Effectiveness taken from
//! the measurements recorded for one Configuration (`ingest-results --format records --config <CONF>`), instead of
//! predicted from its `parameterBindings:`.
//!
//! A MoE names a measurement (`mg_moe_measurement`), how several recorded values combine (`mg_moe_aggregate`:
//! `mean`, `max`, `p95`, `last`) and the unit its bounds are written in (`mg_moe_unit`; absent, the unit of the
//! latest recorded value). Each recorded value is converted to that unit, so `"1.65 s"` and `"1700 ms"` combine.
//! A `fail` verdict on a record that carries the measurement, or a measurement that breaks its own `limit`, is a
//! knock-out for the cell, whatever the aggregate says.

use std::collections::BTreeMap;
use std::path::Path;
use syscribe_model::element::RawElement;
use syscribe_model::expr::{parse_value, to_unit, Quantity, Value};
use syscribe_model::result_records::ResultRecord;
use syscribe_model::results::{ResultsData, Verdict};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aggregate {
    Mean,
    Max,
    P95,
    Last,
}

impl Aggregate {
    pub fn parse(s: &str) -> Option<Aggregate> {
        match s {
            "mean" => Some(Aggregate::Mean),
            "max" => Some(Aggregate::Max),
            "p95" => Some(Aggregate::P95),
            "last" => Some(Aggregate::Last),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Aggregate::Mean => "mean",
            Aggregate::Max => "max",
            Aggregate::P95 => "p95",
            Aggregate::Last => "last",
        }
    }
}

/// One recorded value that went into a cell.
#[derive(Debug, Clone, PartialEq)]
pub struct Contribution {
    pub test_case: String,
    pub scenario: Option<String>,
    pub verdict: Verdict,
    pub environment: Option<String>,
    /// In the cell's unit.
    pub value: f64,
}

/// The measured value of one (MoE, Configuration) cell and the evidence behind it.
#[derive(Debug, Clone, PartialEq)]
pub struct Measured {
    pub value: f64,
    pub unit: String,
    pub aggregate: Aggregate,
    pub contributions: Vec<Contribution>,
    /// Why the recorded evidence knocks the cell out, whatever the value: one entry per failing record or
    /// broken limit.
    pub violations: Vec<String>,
}

/// The records of one Configuration read from a file outside the model (`trade-study --results`), and what the
/// file was, so the ranking can say exactly which evidence it used.
#[derive(Debug, Clone, Default)]
pub struct ExternalRun {
    /// `None` for a Configuration the directory held no file for: no evidence, not the sidecar's.
    pub path: Option<String>,
    /// `blake3:<hex>` of the file's contents.
    pub hash: Option<String>,
    /// Joined to the model's TestCases, so a record naming nothing is not here.
    pub records: Vec<ResultRecord>,
    /// Records the file held, and how many of them named no TestCase of the model.
    pub total: usize,
    pub unmatched: usize,
}

fn load_one(elems: &[RawElement], path: &Path) -> Result<ExternalRun, String> {
    let shown = path.display().to_string();
    let text = std::fs::read_to_string(path).map_err(|e| format!("--results '{shown}': {e}"))?;
    let artefact = syscribe_model::result_records::file_artefact(&shown)?;
    // The same parse and join as `ingest-results --format records`: strict validation, canonical TestCase ids.
    let mut data = ResultsData::parse_records(&text, &shown).map_err(|e| format!("--results '{shown}': {e}"))?;
    let total = data.count;
    data.join_test_cases(elems).map_err(|e| format!("--results '{shown}': {e}"))?;
    Ok(ExternalRun { path: Some(shown), hash: Some(artefact.hash), unmatched: data.unmatched.len(), records: data.records, total })
}

/// Read the `--results` arguments: a directory holding `<Configuration id>.json` for the compared Configurations
/// (one with no file there has no evidence — it does not fall back to the sidecar), and/or `<Configuration>=<file>`
/// for one (which wins over the directory). `cfgs` are the compared Configurations as (id, qualified name); the map
/// is keyed by id, else qualified name. A Configuration not in the map reads the sidecar, as without `--results`.
pub fn load_external(elems: &[RawElement], specs: &[String], cfgs: &[(Option<String>, String)]) -> Result<BTreeMap<String, ExternalRun>, String> {
    let key = |id: &Option<String>, q: &str| id.clone().unwrap_or_else(|| q.to_string());
    let (mut dir, mut files): (Option<&Path>, Vec<(String, &str)>) = (None, Vec::new());
    for s in specs {
        if let Some((c, f)) = s.split_once('=') {
            let k = cfgs.iter().find(|(id, q)| id.as_deref() == Some(c) || q == c).map(|(id, q)| key(id, q));
            let k = k.ok_or_else(|| format!("--results '{s}': '{c}' does not name a compared Configuration"))?;
            if files.iter().any(|(x, _)| *x == k) {
                return Err(format!("--results names Configuration '{c}' twice"));
            }
            files.push((k, f));
        } else if Path::new(s).is_dir() {
            if dir.replace(Path::new(s)).is_some() {
                return Err("--results: only one directory may be given".into());
            }
        } else {
            return Err(format!("--results '{s}' is not a directory; to read one file, name its Configuration: --results <Configuration>={s}"));
        }
    }
    let mut out = BTreeMap::new();
    if let Some(d) = dir {
        let mut found = false;
        for (id, q) in cfgs {
            let f = id.as_ref().map(|i| d.join(format!("{i}.json")));
            match f.filter(|f| f.is_file()) {
                Some(f) => {
                    out.insert(key(id, q), load_one(elems, &f)?);
                    found = true;
                }
                None => {
                    out.insert(key(id, q), ExternalRun::default());
                }
            }
        }
        if !found {
            return Err(format!("--results '{}' holds no <Configuration id>.json for any compared Configuration", d.display()));
        }
    }
    for (k, f) in files {
        out.insert(k, load_one(elems, Path::new(f))?);
    }
    Ok(out)
}

/// Why retained records are older than the model: the TestCase they were recorded for, or a requirement it verifies,
/// changed or vanished since they were ingested (the check `W091` makes for the sidecar). Empty when nothing changed.
/// Records read from a retained run keep the hashes of their ingest, so this is how a later model edit shows.
pub fn stale_reasons(elems: &[RawElement], records: &[ResultRecord]) -> Vec<String> {
    let lookup = syscribe_model::result_join::testcase_lookup(elems);
    let resolver = syscribe_model::resolver::Resolver::new(elems);
    let mut by_test: BTreeMap<&str, Vec<&ResultRecord>> = BTreeMap::new();
    for r in records {
        by_test.entry(r.test_case.as_str()).or_default().push(r);
    }
    let mut out = std::collections::BTreeSet::new();
    for (tc, rs) in by_test {
        match lookup(tc) {
            Some(el) => out.extend(syscribe_model::result_join::stale_reasons(el, rs.iter().copied(), elems, &resolver)),
            None => {
                out.insert(format!("the TestCase {tc} no longer exists"));
            }
        }
    }
    out.into_iter().collect()
}

/// The number and unit of a quantity written as `"4.2 ms"` or `"4.2ms"`; an empty unit is a bare number.
fn split_unit(text: &str) -> (&str, &str) {
    let t = text.trim();
    if let Some((n, u)) = t.split_once(char::is_whitespace) {
        return (n, u.trim());
    }
    for i in (1..=t.len()).rev().filter(|i| t.is_char_boundary(*i)) {
        if t[..i].parse::<f64>().is_ok() {
            return (&t[..i], &t[i..]);
        }
    }
    (t, "")
}

fn in_unit(q: &Quantity, unit: &str) -> Result<f64, String> {
    // `fraction` is how a ratio MoE spells "no unit" (`mg_moe_unit: fraction`).
    if unit.is_empty() || unit == "fraction" {
        return if q.dim == Quantity::scalar(0.0).dim { Ok(q.v) } else { Err("a quantity with a unit cannot be compared to a bound with none".into()) };
    }
    to_unit(q, unit).map_err(|e| e.to_string())
}

fn quantity(text: &str) -> Option<Quantity> {
    match parse_value(text)? {
        Value::Num(q) => Some(q),
        _ => None,
    }
}

/// Does `value` satisfy the limit `<op> <limit>`? `None` when it cannot be decided (different dimensions).
fn limit_holds(op: &str, value: &Quantity, limit: &Quantity) -> Option<bool> {
    if value.dim != limit.dim {
        return None;
    }
    let (v, l) = (value.v, limit.v);
    let eps = 1e-9 * l.abs().max(v.abs()).max(1e-300);
    Some(match op {
        "<" => v < l,
        "<=" => v <= l + eps,
        ">" => v > l,
        ">=" => v >= l - eps,
        "==" => (v - l).abs() <= eps,
        _ => return None,
    })
}

/// Nearest-rank percentile of an unsorted sample.
fn percentile(values: &[f64], p: f64) -> f64 {
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = ((p / 100.0) * v.len() as f64).ceil().max(1.0) as usize;
    v[rank.min(v.len()) - 1]
}

/// The measured value of `name` over `records` (the records of one Configuration), or `Ok(None)` when none carries
/// it. `Err` when the recorded values cannot be put in one unit.
pub fn measured(records: &[ResultRecord], name: &str, aggregate: Aggregate, unit: Option<&str>) -> Result<Option<Measured>, String> {
    // (record index, record, measurement) in the order recorded; `last` is the latest by finish time, then by order.
    let mut hits: Vec<(usize, &ResultRecord, &syscribe_model::result_records::Measurement)> = Vec::new();
    for (i, r) in records.iter().enumerate() {
        for m in r.measurements.iter().filter(|m| m.name == name) {
            hits.push((i, r, m));
        }
    }
    if hits.is_empty() {
        return Ok(None);
    }
    let stamp = |r: &ResultRecord| r.provenance.as_ref().and_then(|p| p.finished_at.clone().or_else(|| p.started_at.clone())).unwrap_or_default();
    let latest = hits.iter().max_by(|a, b| (stamp(a.1), a.0).cmp(&(stamp(b.1), b.0))).map(|h| h.2).expect("non-empty");
    let unit = unit.map(str::to_string).unwrap_or_else(|| split_unit(&latest.value).1.to_string());

    let mut contributions = Vec::new();
    let mut violations = Vec::new();
    for (_, r, m) in &hits {
        let q = quantity(&m.value).ok_or_else(|| format!("measurement '{name}' of {} is not a number: '{}'", r.test_case, m.value))?;
        let value = in_unit(&q, &unit).map_err(|e| format!("measurement '{name}' of {} ('{}'): {e}", r.test_case, m.value))?;
        let at = match &r.scenario {
            Some(s) => format!("{} / {s}", r.test_case),
            None => r.test_case.clone(),
        };
        if r.verdict == Verdict::Fail {
            violations.push(format!("{at} recorded a fail verdict"));
        }
        if let Some(l) = &m.limit {
            if let Some(lq) = quantity(&l.value) {
                if limit_holds(&l.op, &q, &lq) == Some(false) {
                    violations.push(format!("{at}: {name} {} breaks its own limit ({} {})", m.value, l.op, l.value));
                }
            }
        }
        contributions.push(Contribution {
            test_case: r.test_case.clone(),
            scenario: r.scenario.clone(),
            verdict: r.verdict,
            environment: r.provenance.as_ref().and_then(|p| p.environment.clone()),
            value,
        });
    }
    violations.dedup();
    let values: Vec<f64> = contributions.iter().map(|c| c.value).collect();
    let value = match aggregate {
        Aggregate::Mean => values.iter().sum::<f64>() / values.len() as f64,
        Aggregate::Max => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        Aggregate::P95 => percentile(&values, 95.0),
        Aggregate::Last => {
            let (i, _, _) = hits.iter().max_by(|a, b| (stamp(a.1), a.0).cmp(&(stamp(b.1), b.0))).expect("non-empty");
            let k = hits.iter().position(|h| h.0 == *i).expect("present");
            contributions[k].value
        }
    };
    Ok(Some(Measured { value, unit, aggregate, contributions, violations }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use syscribe_model::result_records::{Limit, Measurement, Provenance};

    fn rec(tc: &str, verdict: Verdict, finished: Option<&str>, ms: &[(&str, &str, Option<(&str, &str)>)]) -> ResultRecord {
        ResultRecord {
            test_case: tc.into(),
            scenario: None,
            verdict,
            rationale: None,
            steps: vec![],
            measurements: ms
                .iter()
                .map(|(n, v, l)| Measurement { name: (*n).into(), value: (*v).into(), limit: l.map(|(op, v)| Limit { op: op.into(), value: v.into() }) })
                .collect(),
            provenance: finished.map(|f| Provenance { finished_at: Some(f.into()), ..Default::default() }),
            tc_hash: None,
            req_hashes: Default::default(),
        }
    }

    fn sample() -> Vec<ResultRecord> {
        vec![
            rec("TC-A", Verdict::Pass, Some("2026-11-01T10:00:00Z"), &[("closeTime", "1600 ms", None)]),
            rec("TC-B", Verdict::Pass, Some("2026-11-03T10:00:00Z"), &[("closeTime", "1.8 s", None)]),
            rec("TC-C", Verdict::Pass, Some("2026-11-02T10:00:00Z"), &[("closeTime", "1700 ms", None), ("other", "1 V", None)]),
        ]
    }

    #[test]
    fn aggregates_combine_values_given_in_different_units() {
        let r = sample();
        let get = |a| measured(&r, "closeTime", a, Some("ms")).unwrap().unwrap();
        assert!((get(Aggregate::Mean).value - 1700.0).abs() < 1e-6);
        assert!((get(Aggregate::Max).value - 1800.0).abs() < 1e-6);
        assert!((get(Aggregate::P95).value - 1800.0).abs() < 1e-6, "nearest rank of 3 values");
        // The latest by finish time is TC-B, not the last in the list.
        assert!((get(Aggregate::Last).value - 1800.0).abs() < 1e-6);
        assert_eq!(get(Aggregate::Mean).contributions.len(), 3);
    }

    #[test]
    fn the_unit_defaults_to_the_latest_recorded_values_and_a_missing_measurement_is_none() {
        let r = sample();
        let m = measured(&r, "closeTime", Aggregate::Max, None).unwrap().unwrap();
        assert_eq!((m.unit.as_str(), (m.value * 1e6).round()), ("s", 1.8e6), "latest is 1.8 s");
        assert!(measured(&r, "noSuch", Aggregate::Max, None).unwrap().is_none());
    }

    #[test]
    fn a_wrong_dimension_is_an_error_not_a_silent_number() {
        let r = sample();
        assert!(measured(&r, "closeTime", Aggregate::Max, Some("V")).is_err());
        let mixed = vec![rec("TC-A", Verdict::Pass, None, &[("x", "1 ms", None)]), rec("TC-B", Verdict::Pass, None, &[("x", "2 V", None)])];
        assert!(measured(&mixed, "x", Aggregate::Max, Some("ms")).is_err());
    }

    #[test]
    fn a_fail_verdict_or_a_broken_limit_is_a_violation_whatever_the_value() {
        let failing = vec![rec("TC-A", Verdict::Fail, None, &[("closeTime", "1500 ms", None)])];
        let m = measured(&failing, "closeTime", Aggregate::Max, Some("ms")).unwrap().unwrap();
        assert_eq!(m.violations.len(), 1, "{:?}", m.violations);
        let limited = vec![rec("TC-A", Verdict::Pass, None, &[("closeTime", "2300 ms", Some(("<", "2 s")))])];
        let m = measured(&limited, "closeTime", Aggregate::Max, Some("ms")).unwrap().unwrap();
        assert!(m.violations.iter().any(|v| v.contains("breaks its own limit")), "{:?}", m.violations);
        let ok = vec![rec("TC-A", Verdict::Pass, None, &[("closeTime", "1650 ms", Some(("<", "2 s")))])];
        assert!(measured(&ok, "closeTime", Aggregate::Max, Some("ms")).unwrap().unwrap().violations.is_empty());
    }

    #[test]
    fn a_bare_number_has_no_unit_and_compares_with_a_unitless_bound() {
        let r = vec![rec("TC-A", Verdict::Pass, None, &[("ratio", "0.97", None)])];
        let m = measured(&r, "ratio", Aggregate::Last, None).unwrap().unwrap();
        assert_eq!((m.unit.as_str(), m.value), ("", 0.97));
        // A ratio MoE says `mg_moe_unit: fraction`, which is the same thing.
        assert_eq!(measured(&r, "ratio", Aggregate::Last, Some("fraction")).unwrap().unwrap().value, 0.97);
    }
}
