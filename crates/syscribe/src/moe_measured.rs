//! Measured values for a MoE (GH #467, `REQ-TRS-MOEMEAS-001`): the value of a Measure of Effectiveness taken from
//! the measurements recorded for one Configuration (`ingest-results --format records --config <CONF>`), instead of
//! predicted from its `parameterBindings:`.
//!
//! A MoE names a measurement (`mg_moe_measurement`), how several recorded values combine (`mg_moe_aggregate`:
//! `mean`, `max`, `p95`, `last`) and the unit its bounds are written in (`mg_moe_unit`; absent, the unit of the
//! latest recorded value). Each recorded value is converted to that unit, so `"1.65 s"` and `"1700 ms"` combine.
//! A `fail` verdict on a record that carries the measurement, or a measurement that breaks its own `limit`, is a
//! knock-out for the cell, whatever the aggregate says.

use syscribe_model::expr::{parse_value, to_unit, Quantity, Value};
use syscribe_model::result_records::ResultRecord;
use syscribe_model::results::Verdict;

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
