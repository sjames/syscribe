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
}
