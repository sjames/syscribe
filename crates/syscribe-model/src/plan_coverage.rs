//! Required coverage declared on a `TestPlan` (`ADR-SYS-COVERAGE-001`, GH #500/#501).
//!
//! A plan may declare, next to `runsOn:` (the rig axis):
//!
//! - `conditions:` — named axes, each a list of named points (`{name, value?}` or a bare name);
//! - `minUnits:` — how many distinct units must pass at each required point (integer ≥ 1);
//! - `mode:` — `each` (default: every point of every axis on its own) or `product` (every
//!   combination of one point per axis);
//! - `points:` — an explicit list of required coordinate sets, each a map `axis: point`; it replaces
//!   `mode` as the definition of what is required, so giving both is an error.
//!
//! The fields are read from the frontmatter catch-all so they cost nothing on other types. This module
//! parses and checks them (`E607` malformed, `E608` a `points:` entry that names no declared axis or
//! point) and expands them to the list of required coordinate sets that the verdict (a later slice)
//! judges.

use std::collections::{HashMap, HashSet};

use serde_yaml::Value;

/// The frontmatter keys this module owns (exempt from `W047` on a `TestPlan`).
pub const KEYS: &[&str] = &["conditions", "minUnits", "mode", "points"];

/// How the declared axes combine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Each,
    Product,
}

/// One point of a condition axis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Point {
    pub name: String,
    pub value: Option<String>,
}

/// A coordinate: `(axis, point name)` pairs, one per constrained axis.
pub type Coordinate = Vec<(String, String)>;

/// A plan's parsed coverage declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coverage {
    /// Axes in declaration order.
    pub axes: Vec<(String, Vec<Point>)>,
    pub min_units: Option<u32>,
    pub mode: Mode,
    /// Explicit coordinate sets, when `points:` is given.
    pub points: Option<Vec<Coordinate>>,
}

/// True when the frontmatter carries any coverage key.
pub fn present(extra: &HashMap<String, Value>) -> bool {
    KEYS.iter().any(|k| extra.contains_key(*k))
}

fn is_basic(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some(f) if f.is_ascii_alphabetic() || f == '_') && c.all(|x| x.is_ascii_alphanumeric() || x == '_')
}

fn scalar(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Parse the coverage keys. `Ok(None)` when none is present; `Err` carries every problem found
/// (`E607` for the first group, `E608` for dangling `points:`), each as `(code, message)`.
pub fn parse(extra: &HashMap<String, Value>) -> Result<Option<Coverage>, Vec<(&'static str, String)>> {
    if !present(extra) {
        return Ok(None);
    }
    let mut errs: Vec<(&'static str, String)> = Vec::new();
    let mut axes: Vec<(String, Vec<Point>)> = Vec::new();

    match extra.get("conditions") {
        None => {}
        Some(Value::Mapping(m)) => {
            for (k, v) in m {
                let Some(axis) = k.as_str().filter(|a| is_basic(a)) else {
                    errs.push(("E607", format!("`conditions:` axis {k:?} is not a basic name (letters, digits, `_`)")));
                    continue;
                };
                let Value::Sequence(items) = v else {
                    errs.push(("E607", format!("`conditions.{axis}` must be a list of points")));
                    continue;
                };
                if items.is_empty() {
                    errs.push(("E607", format!("`conditions.{axis}` declares no point")));
                }
                let mut pts: Vec<Point> = Vec::new();
                let mut seen = HashSet::new();
                for it in items {
                    let (name, value) = match it {
                        Value::Mapping(pm) => {
                            let name = pm.get("name").and_then(scalar);
                            let value = pm.get("value").and_then(scalar);
                            for key in pm.keys() {
                                if !matches!(key.as_str(), Some("name" | "value")) {
                                    errs.push(("E607", format!("`conditions.{axis}` point has unknown key {key:?} (only `name`, `value`)")));
                                }
                            }
                            (name, value)
                        }
                        other => (scalar(other), None),
                    };
                    match name.filter(|n| !n.trim().is_empty()) {
                        None => errs.push(("E607", format!("`conditions.{axis}` has a point with no `name`"))),
                        Some(n) => {
                            if !seen.insert(n.clone()) {
                                errs.push(("E607", format!("`conditions.{axis}` declares point '{n}' twice")));
                            }
                            pts.push(Point { name: n, value });
                        }
                    }
                }
                axes.push((axis.to_string(), pts));
            }
        }
        Some(_) => errs.push(("E607", "`conditions:` must be a map of axis → list of points".into())),
    }

    let min_units = match extra.get("minUnits") {
        None => None,
        Some(Value::Number(n)) => match n.as_u64().filter(|u| *u >= 1 && *u <= u32::MAX as u64) {
            Some(u) => Some(u as u32),
            None => {
                errs.push(("E607", format!("`minUnits:` must be an integer ≥ 1, got {n}")));
                None
            }
        },
        Some(_) => {
            errs.push(("E607", "`minUnits:` must be an integer ≥ 1".into()));
            None
        }
    };

    let mode = match extra.get("mode") {
        None => Mode::Each,
        Some(Value::String(s)) if s == "each" => Mode::Each,
        Some(Value::String(s)) if s == "product" => Mode::Product,
        Some(_) => {
            errs.push(("E607", "`mode:` must be `each` or `product`".into()));
            Mode::Each
        }
    };

    let points = match extra.get("points") {
        None => None,
        Some(Value::Sequence(items)) => {
            let mut out: Vec<Coordinate> = Vec::new();
            for it in items {
                let Value::Mapping(m) = it else {
                    errs.push(("E607", "a `points:` entry must be a map of axis → point name".into()));
                    continue;
                };
                if m.is_empty() {
                    errs.push(("E607", "a `points:` entry names no axis".into()));
                    continue;
                }
                let mut coord: Coordinate = Vec::new();
                for (k, v) in m {
                    match (k.as_str(), scalar(v)) {
                        (Some(a), Some(p)) => {
                            match axes.iter().find(|(n, _)| n == a) {
                                None => errs.push(("E608", format!("`points:` names axis '{a}', which `conditions:` does not declare"))),
                                Some((_, pts)) if !pts.iter().any(|x| x.name == p) => {
                                    errs.push(("E608", format!("`points:` names point '{p}' on axis '{a}', which `conditions.{a}` does not declare")))
                                }
                                Some(_) => {}
                            }
                            coord.push((a.to_string(), p));
                        }
                        _ => errs.push(("E607", "a `points:` entry must map axis names to point names".into())),
                    }
                }
                out.push(coord);
            }
            Some(out)
        }
        Some(_) => {
            errs.push(("E607", "`points:` must be a list of maps".into()));
            None
        }
    };
    if extra.contains_key("points") && extra.contains_key("mode") {
        errs.push(("E607", "`points:` and `mode:` are exclusive — `points:` already says what is required".into()));
    }
    if extra.contains_key("mode") && axes.is_empty() {
        errs.push(("E607", "`mode:` needs `conditions:` with at least one axis".into()));
    }
    if extra.contains_key("points") && axes.is_empty() {
        errs.push(("E607", "`points:` needs `conditions:` with at least one axis".into()));
    }

    if errs.is_empty() {
        Ok(Some(Coverage { axes, min_units, mode, points }))
    } else {
        Err(errs)
    }
}

impl Coverage {
    /// The coordinate sets a plan requires a pass at. `points:` wins; otherwise `each` gives one
    /// single-axis coordinate per point and `product` the cartesian product of one point per axis.
    /// No `conditions:` gives no coordinates (only the rig and unit rules apply).
    pub fn required(&self) -> Vec<Coordinate> {
        if let Some(p) = &self.points {
            return p.clone();
        }
        match self.mode {
            Mode::Each => self
                .axes
                .iter()
                .flat_map(|(a, pts)| pts.iter().map(move |p| vec![(a.clone(), p.name.clone())]))
                .collect(),
            Mode::Product => {
                let mut acc: Vec<Coordinate> = vec![Vec::new()];
                for (a, pts) in &self.axes {
                    acc = acc
                        .iter()
                        .flat_map(|c| {
                            pts.iter().map(move |p| {
                                let mut n = c.clone();
                                n.push((a.clone(), p.name.clone()));
                                n
                            })
                        })
                        .collect();
                }
                if self.axes.is_empty() { Vec::new() } else { acc }
            }
        }
    }
}

/// Why a required cell has no pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GapReason {
    /// No usable record at this coordinate.
    Missing,
    /// A record at this coordinate failed (or did not conclude on every scenario).
    Failed,
    /// Passed, but on fewer distinct units than `minUnits:` asks for.
    Units { have: u32, need: u32 },
}

/// One required cell of the grid (test × rig × coordinate) that has no pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gap {
    pub test: String,
    pub env: Option<String>,
    pub coordinate: Coordinate,
    pub reason: GapReason,
}

impl Gap {
    /// `TC-X @ TE-1 [ambient=hot] (missing)`.
    pub fn describe(&self) -> String {
        let mut s = self.test.clone();
        if let Some(e) = &self.env {
            s.push_str(&format!(" @ {e}"));
        }
        if !self.coordinate.is_empty() {
            let c: Vec<String> = self.coordinate.iter().map(|(a, p)| format!("{a}={p}")).collect();
            s.push_str(&format!(" [{}]", c.join(", ")));
        }
        match &self.reason {
            GapReason::Missing => s.push_str(" (missing)"),
            GapReason::Failed => s.push_str(" (FAILED)"),
            GapReason::Units { have, need } => s.push_str(&format!(" ({have} of {need} units)")),
        }
        s
    }
}

/// The cells of the plan's grid that have no pass: for every member, every rig in `envs` (any rig when empty)
/// and every required coordinate, a record whose `condition` includes the coordinate must have passed, on at
/// least `minUnits` distinct named units when that is declared. A record of an `unhealthy` rig counts for
/// nothing. Empty when the plan declares nothing to judge.
pub fn gaps(
    cov: &Coverage,
    members: &[String],
    envs: &[String],
    unhealthy: &HashSet<String>,
    results: Option<&crate::results::ResultsData>,
) -> Vec<Gap> {
    use crate::results::Verdict;
    let mut coords = cov.required();
    if coords.is_empty() {
        if cov.min_units.is_none() {
            return Vec::new();
        }
        coords.push(Vec::new()); // only the unit rule: the unconditioned cell
    }
    let rigs: Vec<Option<&String>> = if envs.is_empty() { vec![None] } else { envs.iter().map(Some).collect() };
    let mut out = Vec::new();
    for tc in members {
        let recs: Vec<&crate::result_records::ResultRecord> = results.map(|r| r.records_for(tc).collect()).unwrap_or_default();
        for rig in &rigs {
            for coord in &coords {
                let matching: Vec<&&crate::result_records::ResultRecord> = recs
                    .iter()
                    .filter(|r| {
                        let renv = r.provenance.as_ref().and_then(|p| p.environment.as_ref());
                        renv.is_none_or(|e| !unhealthy.contains(e))
                            && rig.is_none_or(|e| renv == Some(e))
                            && coord.iter().all(|(a, p)| r.condition.get(a) == Some(p))
                    })
                    .collect();
                let reason = if matching.is_empty() {
                    Some(GapReason::Missing)
                } else if matching.iter().any(|r| r.verdict == Verdict::Fail) {
                    Some(GapReason::Failed)
                } else {
                    // a unit passes when every record of it passed
                    let mut by_unit: std::collections::BTreeMap<Option<&str>, bool> = std::collections::BTreeMap::new();
                    for r in &matching {
                        let u = r.provenance.as_ref().and_then(|p| p.unit.as_ref()).map(|u| u.id.as_str());
                        let e = by_unit.entry(u).or_insert(true);
                        *e = *e && r.verdict == Verdict::Pass;
                    }
                    match cov.min_units {
                        None => (!by_unit.values().any(|ok| *ok)).then_some(GapReason::Missing),
                        Some(need) => {
                            let have = by_unit.iter().filter(|(u, ok)| u.is_some() && **ok).count() as u32;
                            if have >= need {
                                None
                            } else if by_unit.values().any(|ok| *ok) {
                                Some(GapReason::Units { have, need })
                            } else {
                                Some(GapReason::Missing)
                            }
                        }
                    }
                };
                if let Some(reason) = reason {
                    out.push(Gap { test: tc.clone(), env: rig.map(|e| (*e).clone()), coordinate: coord.clone(), reason });
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn x(yaml: &str) -> HashMap<String, Value> {
        serde_yaml::from_str(yaml).unwrap()
    }
    fn codes(yaml: &str) -> Vec<&'static str> {
        match parse(&x(yaml)) {
            Err(e) => e.into_iter().map(|(c, _)| c).collect(),
            Ok(_) => vec![],
        }
    }
    const AMB: &str = "conditions:\n  ambient: [{name: cold, value: -20 degC}, {name: hot, value: 70 degC}]\n  supply: [low, high]\n";

    #[test]
    fn absent_is_none() {
        assert_eq!(parse(&x("a: 1")).unwrap(), None);
    }
    #[test]
    fn each_is_the_sum_of_points() {
        let c = parse(&x(AMB)).unwrap().unwrap();
        assert_eq!(c.mode, Mode::Each);
        assert_eq!(c.required().len(), 4);
        assert_eq!(c.axes[0].1[0].value.as_deref(), Some("-20 degC"));
        assert_eq!(c.axes[1].1[0].name, "low", "a bare string is a point name");
    }
    #[test]
    fn product_is_the_cartesian_product() {
        let c = parse(&x(&format!("{AMB}mode: product\n"))).unwrap().unwrap();
        let r = c.required();
        assert_eq!(r.len(), 4);
        assert!(r.iter().all(|c| c.len() == 2));
        assert!(r.contains(&vec![("ambient".into(), "hot".into()), ("supply".into(), "low".into())]));
    }
    #[test]
    fn product_of_three_axes_multiplies() {
        let c = parse(&x("conditions:\n  a: [x, y]\n  b: [p, q, r]\n  c: [m, n]\nmode: product\n")).unwrap().unwrap();
        assert_eq!(c.required().len(), 12);
    }
    #[test]
    fn explicit_points_are_the_requirement() {
        let c = parse(&x(&format!("{AMB}points:\n  - {{ambient: hot, supply: high}}\n  - {{ambient: cold}}\n"))).unwrap().unwrap();
        assert_eq!(c.required().len(), 2);
    }
    #[test]
    fn min_units_must_be_a_positive_integer() {
        assert_eq!(parse(&x("minUnits: 2")).unwrap().unwrap().min_units, Some(2));
        for bad in ["minUnits: 0", "minUnits: -1", "minUnits: 1.5", "minUnits: two", "minUnits: [2]"] {
            assert_eq!(codes(bad), vec!["E607"], "{bad}");
        }
    }
    #[test]
    fn malformed_conditions_are_e607() {
        for bad in [
            "conditions: [a]",
            "conditions:\n  ambient: hot",
            "conditions:\n  ambient: []",
            "conditions:\n  ambient: [{value: 1}]",
            "conditions:\n  ambient: [hot, hot]",
            "conditions:\n  ambient: [{name: hot, colour: red}]",
            "conditions:\n  'my axis': [hot]",
        ] {
            assert!(codes(bad).contains(&"E607"), "{bad}");
        }
    }
    #[test]
    fn mode_and_points_rules() {
        assert!(codes(&format!("{AMB}mode: both")).contains(&"E607"));
        assert!(codes("mode: product").contains(&"E607"), "mode needs axes");
        assert!(codes(&format!("{AMB}mode: each\npoints:\n  - {{ambient: hot}}")).contains(&"E607"), "exclusive");
        assert!(codes(&format!("{AMB}points: {{ambient: hot}}")).contains(&"E607"));
        assert!(codes(&format!("{AMB}points:\n  - hot")).contains(&"E607"));
    }
    #[test]
    fn dangling_points_are_e608() {
        assert_eq!(codes(&format!("{AMB}points:\n  - {{humidity: wet}}")), vec!["E608"]);
        assert_eq!(codes(&format!("{AMB}points:\n  - {{ambient: tepid}}")), vec!["E608"]);
    }
    #[test]
    fn no_axes_means_no_required_coordinates() {
        let c = parse(&x("minUnits: 3")).unwrap().unwrap();
        assert!(c.required().is_empty());
    }

    // ── gaps ────────────────────────────────────────────────────────────────

    use crate::result_records::{parse as parse_records, validate_all};
    use crate::results::ResultsData;

    fn data(json: &str) -> ResultsData {
        let mut v = parse_records(json).unwrap();
        validate_all(&mut v).unwrap();
        ResultsData::from_records(v, 0, "t")
    }
    fn rec(verdict: &str, cond: &str, env: &str, unit: &str) -> String {
        let c = if cond.is_empty() { String::new() } else { format!(r#","condition":{{"ambient":"{cond}"}}"#) };
        let u = if unit.is_empty() { String::new() } else { format!(r#","unit":{{"id":"{unit}"}}"#) };
        let e = if env.is_empty() { String::new() } else { format!(r#""environment":"{env}""#) };
        let comma = if !e.is_empty() && !u.is_empty() { "," } else { "" };
        format!(r#"{{"testCase":"T","verdict":"{verdict}","provenance":{{{e}{comma}{}}}{c}}}"#, u.trim_start_matches(','))
    }
    fn cov(yaml: &str) -> Coverage {
        parse(&x(yaml)).unwrap().unwrap()
    }
    fn g(c: &Coverage, envs: &[&str], recs: &[String]) -> Vec<Gap> {
        let envs: Vec<String> = envs.iter().map(|e| e.to_string()).collect();
        let d = (!recs.is_empty()).then(|| data(&format!("[{}]", recs.join(","))));
        gaps(c, &["T".to_string()], &envs, &HashSet::new(), d.as_ref())
    }
    const AX: &str = "conditions:\n  ambient: [cold, hot]\n";

    #[test]
    fn every_required_point_needs_a_pass() {
        let c = cov(AX);
        assert_eq!(g(&c, &[], &[rec("pass", "cold", "", ""), rec("pass", "hot", "", "")]), vec![]);
        let gaps = g(&c, &[], &[rec("pass", "cold", "", "")]);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].describe(), "T [ambient=hot] (missing)");
    }
    #[test]
    fn nothing_recorded_misses_every_cell() {
        let c = cov(AX);
        assert_eq!(gaps(&c, &["T".into()], &[], &HashSet::new(), None).len(), 2);
    }
    #[test]
    fn a_failure_at_a_point_is_a_failed_cell() {
        let gaps = g(&cov(AX), &[], &[rec("pass", "cold", "", ""), rec("fail", "hot", "", "")]);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].reason, GapReason::Failed);
    }
    #[test]
    fn a_result_that_did_not_conclude_is_not_a_pass() {
        for v in ["flaky", "blocked", "not-run", "inconclusive", "ignored"] {
            let gaps = g(&cov(AX), &[], &[rec("pass", "cold", "", ""), rec(v, "hot", "", "")]);
            assert_eq!(gaps.len(), 1, "{v}");
        }
    }
    #[test]
    fn an_unconditioned_pass_covers_no_point() {
        assert_eq!(g(&cov(AX), &[], &[rec("pass", "", "", "")]).len(), 2);
    }
    #[test]
    fn product_cells_need_every_combination() {
        let c = cov("conditions:\n  ambient: [cold, hot]\n  supply: [low, high]\nmode: product\n");
        let both = |a: &str, s: &str, v: &str| format!(r#"{{"testCase":"T","verdict":"{v}","condition":{{"ambient":"{a}","supply":"{s}"}}}}"#);
        let recs = [both("cold", "low", "pass"), both("cold", "high", "pass"), both("hot", "low", "pass")];
        let gaps = g(&c, &[], &recs);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].describe(), "T [ambient=hot, supply=high] (missing)");
    }
    #[test]
    fn each_rig_needs_its_own_pass() {
        let c = cov(AX);
        let recs = [rec("pass", "cold", "TE-1", ""), rec("pass", "hot", "TE-1", ""), rec("pass", "cold", "TE-2", "")];
        let gaps = g(&c, &["TE-1", "TE-2"], &recs);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].describe(), "T @ TE-2 [ambient=hot] (missing)");
    }
    #[test]
    fn a_retired_rigs_results_count_for_nothing() {
        let c = cov(AX);
        let recs = [rec("pass", "cold", "TE-1", ""), rec("pass", "hot", "TE-1", "")];
        let unhealthy: HashSet<String> = ["TE-1".to_string()].into();
        let gaps = gaps(&c, &["T".into()], &["TE-1".into()], &unhealthy, Some(&data(&format!("[{}]", recs.join(",")))));
        assert_eq!(gaps.len(), 2);
    }
    #[test]
    fn min_units_counts_distinct_named_units_that_passed() {
        let c = cov("conditions:\n  ambient: [hot]\nminUnits: 2\n");
        let one = g(&c, &[], &[rec("pass", "hot", "", "SN-1")]);
        assert_eq!(one[0].reason, GapReason::Units { have: 1, need: 2 });
        assert_eq!(one[0].describe(), "T [ambient=hot] (1 of 2 units)");
        assert!(g(&c, &[], &[rec("pass", "hot", "", "SN-1"), rec("pass", "hot", "", "SN-2")]).is_empty());
        // the same unit twice is one unit
        assert_eq!(g(&c, &[], &[rec("pass", "hot", "", "SN-1"), rec("pass", "hot", "", "SN-1")]).len(), 1);
        // a failing unit does not count, and an unnamed pass is no unit
        assert_eq!(g(&c, &[], &[rec("pass", "hot", "", "SN-1"), rec("fail", "hot", "", "SN-2")])[0].reason, GapReason::Failed);
        assert_eq!(g(&c, &[], &[rec("pass", "hot", "", "")])[0].reason, GapReason::Units { have: 0, need: 2 });
    }
    #[test]
    fn min_units_alone_judges_the_unconditioned_cell() {
        let c = cov("minUnits: 2");
        assert_eq!(g(&c, &[], &[]).len(), 1);
        assert!(g(&c, &[], &[rec("pass", "", "", "SN-1"), rec("pass", "", "", "SN-2")]).is_empty());
    }
}
