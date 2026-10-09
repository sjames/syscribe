//! `[cyber]` configuration for ISO/SAE 21434 risk and CAL determination (GH #222).
//!
//! Default (no `[cyber]` table) is the historical `simple` method implemented in
//! [`crate::risk`]: `score = impactRank + feasibilityRank`, risk→CAL 1:1. The
//! `.syscribe.toml` table lets a project:
//!
//! - pick `method = "simple" | "annex"`;
//! - override cells of the impact × feasibility risk matrix (`[cyber.risk_matrix]`)
//!   and the numeric-value → level map (`[cyber.risk_levels]`);
//! - override cells of the impact × attack-vector CAL table (`[cyber.cal_table]`)
//!   and the risk-level → CAL map (`[cyber.cal_by_risk]`);
//! - re-tune attack-potential scoring (`[cyber.attack_potential]`: per-factor
//!   `label = points` tables and `thresholds`).
//!
//! **The built-in `annex` tables are EXAMPLE tables modelled on the informative
//! annexes of ISO/SAE 21434 (attack-potential scoring, risk matrix, CAL from impact
//! and attack vector) as best recalled. They are NOT normative and this repository
//! does not contain the standard: verify every value against your project's copy
//! and override via `[cyber]` where it differs.**
//!
//! A malformed entry never aborts: it is recorded as a defect (reported as `W640`)
//! and that entry alone falls back to the default.

use crate::element::RawFrontmatter;
use crate::risk::{level_from_score, Risk, RiskLevel};
use std::collections::BTreeMap;
use std::path::Path;

/// Risk-determination method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskMethod {
    /// `impactRank + feasibilityRank` rank-sum (historical default).
    Simple,
    /// Example tables modelled on the ISO/SAE 21434 informative annexes.
    Annex,
}

impl RiskMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            RiskMethod::Simple => "simple",
            RiskMethod::Annex => "annex",
        }
    }
}

/// Impact keys, rank 0..3.
pub const IMPACTS: [&str; 4] = ["negligible", "moderate", "major", "severe"];
/// Feasibility keys, rank 0..3.
pub const FEASIBILITIES: [&str; 4] = ["very_low", "low", "medium", "high"];
/// Attack-vector keys, index 0..3.
pub const VECTORS: [&str; 4] = ["network", "adjacent", "local", "physical"];
const RISK_NAMES: [&str; 4] = ["low", "medium", "high", "critical"];

/// The five attack-potential factors, in fixed order (YAML field names).
pub const AP_FACTORS: [&str; 5] = [
    "elapsedTime",
    "expertise",
    "knowledge",
    "windowOfOpportunity",
    "equipment",
];

type Matrix = [[(Option<u8>, RiskLevel); 4]; 4];

/// One attack-potential factor table: label → points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactorTable {
    pub entries: Vec<(String, u32)>,
}

impl FactorTable {
    fn new(e: &[(&str, u32)]) -> Self {
        FactorTable { entries: e.iter().map(|(l, p)| (l.to_string(), *p)).collect() }
    }
    fn lookup(&self, label: &str) -> Option<u32> {
        let n = norm(label);
        self.entries.iter().find(|(l, _)| norm(l) == n).map(|(_, p)| *p)
    }
    fn labels(&self) -> String {
        self.entries.iter().map(|(l, _)| l.as_str()).collect::<Vec<_>>().join(", ")
    }
}

/// Attack-potential scoring: factor tables plus the sum → feasibility thresholds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttackPotential {
    pub tables: [FactorTable; 5],
    /// Sum ≤ `high_max` → `high`; ≤ `medium_max` → `medium`; ≤ `low_max` → `low`;
    /// above → `very_low`.
    pub high_max: u32,
    pub medium_max: u32,
    pub low_max: u32,
}

impl AttackPotential {
    /// EXAMPLE defaults modelled on the ISO/SAE 21434 informative attack-potential
    /// annex (verify against your copy).
    pub fn example() -> Self {
        AttackPotential {
            tables: [
                FactorTable::new(&[
                    ("up_to_1_day", 0),
                    ("up_to_1_week", 1),
                    ("up_to_1_month", 4),
                    ("up_to_6_months", 17),
                    ("over_6_months", 19),
                ]),
                FactorTable::new(&[
                    ("layman", 0),
                    ("proficient", 3),
                    ("expert", 6),
                    ("multiple_experts", 8),
                ]),
                FactorTable::new(&[
                    ("public", 0),
                    ("restricted", 3),
                    ("confidential", 7),
                    ("strictly_confidential", 11),
                ]),
                FactorTable::new(&[
                    ("unlimited", 0),
                    ("easy", 1),
                    ("moderate", 4),
                    ("difficult", 10),
                ]),
                FactorTable::new(&[
                    ("standard", 0),
                    ("specialized", 4),
                    ("bespoke", 7),
                    ("multiple_bespoke", 9),
                ]),
            ],
            high_max: 13,
            medium_max: 19,
            low_max: 24,
        }
    }

    /// Feasibility rank (0 very_low .. 3 high) for a points sum.
    pub fn rank_for_points(&self, sum: u32) -> u8 {
        if sum <= self.high_max {
            3
        } else if sum <= self.medium_max {
            2
        } else if sum <= self.low_max {
            1
        } else {
            0
        }
    }
}

/// Parsed `[cyber]` configuration. `Default` = the historical `simple` method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CyberConfig {
    method: RiskMethod,
    /// `[impact][feasibility]` → (optional numeric value, level).
    matrix: Matrix,
    /// `[impact][vector]` → CAL rank, `None` = no table cell (fall back to risk→CAL).
    cal_table: [[Option<u8>; 4]; 4],
    /// risk level (low..critical) → CAL rank.
    cal_by_risk: [u8; 4],
    pub attack_potential: AttackPotential,
    customized: bool,
    defects: Vec<String>,
}

/// Why an attack-potential score is not computable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApError {
    /// Some, but not all, of the five factor fields are present.
    Incomplete(Vec<&'static str>),
    /// A factor value is neither a known label nor a non-negative integer.
    Invalid { factor: &'static str, value: String, allowed: String },
}

fn norm(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_lowercase()
}

fn simple_matrix() -> Matrix {
    let mut m = [[(None, RiskLevel::Low); 4]; 4];
    for (i, row) in m.iter_mut().enumerate() {
        for (f, cell) in row.iter_mut().enumerate() {
            *cell = (None, level_from_score((i + f) as u8));
        }
    }
    m
}

/// EXAMPLE annex risk values (1..5), rows negligible..severe, cols very_low..high.
const ANNEX_VALUES: [[u8; 4]; 4] = [[1, 1, 1, 1], [1, 2, 2, 3], [1, 2, 3, 4], [2, 3, 4, 5]];
/// EXAMPLE annex CAL (1..4) by impact × vector (network, adjacent, local, physical).
const ANNEX_CAL: [[u8; 4]; 4] = [[2, 2, 1, 1], [3, 3, 2, 1], [4, 3, 2, 1], [4, 4, 3, 2]];

fn default_value_level(v: u8) -> Option<RiskLevel> {
    match v {
        1 | 2 => Some(RiskLevel::Low),
        3 => Some(RiskLevel::Medium),
        4 => Some(RiskLevel::High),
        5 => Some(RiskLevel::Critical),
        _ => None,
    }
}

impl Default for CyberConfig {
    fn default() -> Self {
        CyberConfig {
            method: RiskMethod::Simple,
            matrix: simple_matrix(),
            cal_table: [[None; 4]; 4],
            cal_by_risk: [1, 2, 3, 4],
            attack_potential: AttackPotential::example(),
            customized: false,
            defects: Vec::new(),
        }
    }
}

fn parse_level(s: &str) -> Option<RiskLevel> {
    match norm(s).as_str() {
        "low" => Some(RiskLevel::Low),
        "medium" => Some(RiskLevel::Medium),
        "high" => Some(RiskLevel::High),
        "critical" => Some(RiskLevel::Critical),
        _ => None,
    }
}

fn parse_cal(s: &str) -> Option<u8> {
    crate::risk::cal_rank(&s.to_ascii_uppercase())
}

fn index_of(keys: &[&str], k: &str) -> Option<usize> {
    let n = norm(k);
    keys.iter().position(|x| norm(x) == n)
}

fn nonneg(v: &toml::Value) -> Option<u32> {
    v.as_integer().and_then(|n| u32::try_from(n).ok())
}

impl CyberConfig {
    /// Load `[cyber]` from `<model_root>/.syscribe.toml`. Absent file/table → default.
    pub fn load(model_root: &Path) -> Self {
        match std::fs::read_to_string(model_root.join(".syscribe.toml")) {
            Ok(text) => Self::from_toml_str(&text),
            Err(_) => Self::default(),
        }
    }

    /// Parse the `[cyber]` table out of a whole `.syscribe.toml` text.
    pub fn from_toml_str(text: &str) -> Self {
        let Ok(root) = text.parse::<toml::Table>() else { return Self::default() };
        match root.get("cyber") {
            None => Self::default(),
            Some(toml::Value::Table(t)) => Self::from_table(t),
            Some(_) => {
                let mut c = Self::default();
                c.defects.push("[cyber] must be a table — ignored".to_string());
                c
            }
        }
    }

    fn from_table(t: &toml::Table) -> Self {
        let mut c = Self::default();
        let mut keys: Vec<&String> = t.keys().collect();
        keys.sort();

        // `method` first: it selects the base tables the overrides merge into.
        if let Some(v) = t.get("method") {
            match v.as_str().map(norm).as_deref() {
                Some("simple") => {}
                Some("annex") => {
                    c.method = RiskMethod::Annex;
                    c.customized = true;
                    for (i, row) in ANNEX_VALUES.iter().enumerate() {
                        for (f, v) in row.iter().enumerate() {
                            c.matrix[i][f] =
                                (Some(*v), default_value_level(*v).unwrap_or(RiskLevel::Low));
                        }
                    }
                    for (i, row) in ANNEX_CAL.iter().enumerate() {
                        for (j, v) in row.iter().enumerate() {
                            c.cal_table[i][j] = Some(*v);
                        }
                    }
                }
                _ => c.defects.push(format!(
                    "[cyber] method {} must be \"simple\" or \"annex\" — using \"simple\"",
                    v
                )),
            }
        }

        // `risk_levels` (value → level) is read before the matrix cells that use it.
        let mut value_levels: BTreeMap<u8, RiskLevel> =
            (1..=5).filter_map(|v| default_value_level(v).map(|l| (v, l))).collect();
        let mut relevel = false;
        for k in &keys {
            if norm(k) != "risklevels" {
                continue;
            }
            let Some(tb) = t[*k].as_table() else {
                c.defects.push("[cyber] risk_levels must be a table — ignored".into());
                continue;
            };
            let mut ks: Vec<&String> = tb.keys().collect();
            ks.sort();
            for k in ks {
                match (k.parse::<u8>(), tb[k].as_str().and_then(parse_level)) {
                    (Ok(n), Some(l)) if n >= 1 => {
                        value_levels.insert(n, l);
                        relevel = true;
                    }
                    _ => c.defects.push(format!(
                        "[cyber.risk_levels] entry '{} = {}' must map an integer value >= 1 to low|medium|high|critical — ignored",
                        k, tb[k]
                    )),
                }
            }
        }
        if relevel {
            c.customized = true;
            for row in c.matrix.iter_mut() {
                for cell in row.iter_mut() {
                    if let Some(l) = cell.0.and_then(|v| value_levels.get(&v)) {
                        cell.1 = *l;
                    }
                }
            }
        }

        for k in keys {
            let v = &t[k];
            match norm(k).as_str() {
                "method" | "risklevels" => {}
                "riskmatrix" => c.apply_risk_matrix(v, &value_levels),
                "caltable" => c.apply_cal_table(v),
                "calbyrisk" => c.apply_cal_by_risk(v),
                "attackpotential" => c.apply_attack_potential(v),
                _ => c.defects.push(format!("[cyber] unknown key '{}' — ignored", k)),
            }
        }
        c
    }

    fn apply_risk_matrix(&mut self, v: &toml::Value, levels: &BTreeMap<u8, RiskLevel>) {
        let Some(tb) = v.as_table() else {
            self.defects
                .push("[cyber.risk_matrix] must be a table of impact rows — ignored".into());
            return;
        };
        for (ik, iv) in tb {
            let Some(i) = index_of(&IMPACTS, ik) else {
                self.defects.push(format!(
                    "[cyber.risk_matrix] unknown impact '{}' (expected {}) — row ignored",
                    ik,
                    IMPACTS.join(", ")
                ));
                continue;
            };
            let Some(row) = iv.as_table() else {
                self.defects.push(format!(
                    "[cyber.risk_matrix.{}] must be a table of feasibility cells — row ignored",
                    ik
                ));
                continue;
            };
            for (fk, cv) in row {
                let Some(f) = index_of(&FEASIBILITIES, fk) else {
                    self.defects.push(format!(
                        "[cyber.risk_matrix.{}] unknown feasibility '{}' (expected {}) — cell ignored",
                        ik,
                        fk,
                        FEASIBILITIES.join(", ")
                    ));
                    continue;
                };
                let cell = match cv {
                    toml::Value::Integer(n) if (1..=255).contains(n) => {
                        levels.get(&(*n as u8)).map(|l| (Some(*n as u8), *l))
                    }
                    toml::Value::String(s) => parse_level(s).map(|l| (None, l)),
                    _ => None,
                };
                match cell {
                    Some(cell) => {
                        self.matrix[i][f] = cell;
                        self.customized = true;
                    }
                    None => self.defects.push(format!(
                        "[cyber.risk_matrix.{}] cell '{} = {}' must be low|medium|high|critical or an integer risk value mapped in [cyber.risk_levels] — cell ignored",
                        ik, fk, cv
                    )),
                }
            }
        }
    }

    fn apply_cal_table(&mut self, v: &toml::Value) {
        let Some(tb) = v.as_table() else {
            self.defects
                .push("[cyber.cal_table] must be a table of impact rows — ignored".into());
            return;
        };
        for (ik, iv) in tb {
            let Some(i) = index_of(&IMPACTS, ik) else {
                self.defects.push(format!(
                    "[cyber.cal_table] unknown impact '{}' (expected {}) — row ignored",
                    ik,
                    IMPACTS.join(", ")
                ));
                continue;
            };
            let Some(row) = iv.as_table() else {
                self.defects.push(format!(
                    "[cyber.cal_table.{}] must be a table of attack-vector cells — row ignored",
                    ik
                ));
                continue;
            };
            for (vk, cv) in row {
                let Some(j) = index_of(&VECTORS, vk) else {
                    self.defects.push(format!(
                        "[cyber.cal_table.{}] unknown attack vector '{}' (expected {}) — cell ignored",
                        ik,
                        vk,
                        VECTORS.join(", ")
                    ));
                    continue;
                };
                match cv.as_str().and_then(parse_cal) {
                    Some(cal) => {
                        self.cal_table[i][j] = Some(cal);
                        self.customized = true;
                    }
                    None => self.defects.push(format!(
                        "[cyber.cal_table.{}] cell '{} = {}' must be CAL1..CAL4 — cell ignored",
                        ik, vk, cv
                    )),
                }
            }
        }
    }

    fn apply_cal_by_risk(&mut self, v: &toml::Value) {
        let Some(tb) = v.as_table() else {
            self.defects.push(
                "[cyber.cal_by_risk] must be a table (low|medium|high|critical = \"CALn\") — ignored"
                    .into(),
            );
            return;
        };
        for (rk, cv) in tb {
            let Some(r) = index_of(&RISK_NAMES, rk) else {
                self.defects.push(format!(
                    "[cyber.cal_by_risk] unknown risk level '{}' (expected low, medium, high, critical) — entry ignored",
                    rk
                ));
                continue;
            };
            match cv.as_str().and_then(parse_cal) {
                Some(cal) => {
                    self.cal_by_risk[r] = cal;
                    self.customized = true;
                }
                None => self.defects.push(format!(
                    "[cyber.cal_by_risk] '{} = {}' must be CAL1..CAL4 — entry ignored",
                    rk, cv
                )),
            }
        }
    }

    fn apply_attack_potential(&mut self, v: &toml::Value) {
        let Some(tb) = v.as_table() else {
            self.defects.push("[cyber.attack_potential] must be a table — ignored".into());
            return;
        };
        for (k, val) in tb {
            if norm(k) == "thresholds" {
                self.apply_thresholds(val);
                continue;
            }
            let Some(fi) = index_of(&AP_FACTORS, k) else {
                self.defects.push(format!(
                    "[cyber.attack_potential] unknown key '{}' (expected {} or thresholds) — ignored",
                    k,
                    AP_FACTORS.join(", ")
                ));
                continue;
            };
            let Some(ft) = val.as_table() else {
                self.defects.push(format!(
                    "[cyber.attack_potential.{}] must be a table of label = points — ignored",
                    k
                ));
                continue;
            };
            let mut entries: Vec<(String, u32)> = Vec::new();
            let mut ok = true;
            for (label, pv) in ft {
                match nonneg(pv) {
                    Some(n) if !norm(label).is_empty() => entries.push((label.clone(), n)),
                    _ => {
                        ok = false;
                        self.defects.push(format!(
                            "[cyber.attack_potential.{}] '{} = {}' must be a non-negative integer — table ignored",
                            k, label, pv
                        ));
                    }
                }
            }
            if ok && entries.is_empty() {
                self.defects
                    .push(format!("[cyber.attack_potential.{}] is empty — table ignored", k));
            } else if ok {
                self.attack_potential.tables[fi] = FactorTable { entries };
                self.customized = true;
            }
        }
    }

    fn apply_thresholds(&mut self, v: &toml::Value) {
        let Some(tb) = v.as_table() else {
            self.defects
                .push("[cyber.attack_potential.thresholds] must be a table — ignored".into());
            return;
        };
        let mut new = [
            self.attack_potential.high_max,
            self.attack_potential.medium_max,
            self.attack_potential.low_max,
        ];
        let mut touched = false;
        for (k, val) in tb {
            let idx = match norm(k).as_str() {
                "highmax" => 0,
                "mediummax" => 1,
                "lowmax" => 2,
                _ => {
                    self.defects.push(format!(
                        "[cyber.attack_potential.thresholds] unknown key '{}' (expected high_max, medium_max, low_max) — ignored",
                        k
                    ));
                    continue;
                }
            };
            match nonneg(val) {
                Some(n) => {
                    new[idx] = n;
                    touched = true;
                }
                None => self.defects.push(format!(
                    "[cyber.attack_potential.thresholds] '{} = {}' must be a non-negative integer — ignored",
                    k, val
                )),
            }
        }
        if !touched {
            return;
        }
        if new[0] < new[1] && new[1] < new[2] {
            self.attack_potential.high_max = new[0];
            self.attack_potential.medium_max = new[1];
            self.attack_potential.low_max = new[2];
            self.customized = true;
        } else {
            self.defects.push(
                "[cyber.attack_potential.thresholds] must satisfy high_max < medium_max < low_max — thresholds ignored"
                    .into(),
            );
        }
    }

    /// `true` when nothing in `[cyber]` took effect (output identical to pre-#222).
    pub fn is_default(&self) -> bool {
        !self.customized
    }

    pub fn method(&self) -> RiskMethod {
        self.method
    }

    /// Defect messages for `W640` (the entries that were ignored).
    pub fn defects(&self) -> &[String] {
        &self.defects
    }

    /// Risk for impact rank × feasibility rank (each 0..=3).
    pub fn risk_cell(&self, impact: u8, feas: u8) -> Risk {
        let (value, level) = self.matrix[(impact as usize).min(3)][(feas as usize).min(3)];
        Risk { level, value }
    }

    /// Expected CAL rank: the table cell for (impact, vector) when present, else the
    /// risk → CAL map.
    pub fn cal_for(&self, impact: u8, vector: Option<&str>, level: RiskLevel) -> u8 {
        if let Some(j) = vector.and_then(|v| index_of(&VECTORS, v)) {
            if let Some(c) = self.cal_table[(impact as usize).min(3)][j] {
                return c;
            }
        }
        self.cal_by_risk[level.rank() as usize]
    }

    /// Attack-potential score of a threat/attack step.
    ///
    /// `None` when none of the five factor fields is present; `Some(Err)` when the
    /// set is incomplete or a value is invalid; `Some(Ok(rank))` otherwise.
    pub fn attack_potential_feasibility(&self, fm: &RawFrontmatter) -> Option<Result<u8, ApError>> {
        self.attack_potential_points(fm)
            .map(|r| r.map(|sum| self.attack_potential.rank_for_points(sum)))
    }

    /// Summed attack-potential points (see [`Self::attack_potential_feasibility`]).
    pub fn attack_potential_points(&self, fm: &RawFrontmatter) -> Option<Result<u32, ApError>> {
        let fields = [
            &fm.elapsed_time,
            &fm.expertise,
            &fm.knowledge,
            &fm.window_of_opportunity,
            &fm.equipment,
        ];
        if fields.iter().all(|f| f.is_none()) {
            return None;
        }
        let missing: Vec<&'static str> = fields
            .iter()
            .zip(AP_FACTORS)
            .filter(|(f, _)| f.is_none())
            .map(|(_, n)| n)
            .collect();
        if !missing.is_empty() {
            return Some(Err(ApError::Incomplete(missing)));
        }
        let mut sum = 0u32;
        for (i, f) in fields.iter().enumerate() {
            let Some(v) = f.as_ref() else { continue };
            match self.factor_points(i, v) {
                Ok(p) => sum = sum.saturating_add(p),
                Err(e) => return Some(Err(e)),
            }
        }
        Some(Ok(sum))
    }

    /// Points for one factor value (label or non-negative integer).
    pub fn factor_points(&self, idx: usize, v: &serde_yaml::Value) -> Result<u32, ApError> {
        let table = &self.attack_potential.tables[idx];
        let shown = match v {
            serde_yaml::Value::String(s) => s.clone(),
            other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
        };
        let pts = match v {
            serde_yaml::Value::Number(n) => n.as_u64().and_then(|n| u32::try_from(n).ok()),
            serde_yaml::Value::String(s) => table.lookup(s),
            _ => None,
        };
        pts.ok_or_else(|| ApError::Invalid {
            factor: AP_FACTORS[idx],
            value: shown,
            allowed: format!("{} or a non-negative integer", table.labels()),
        })
    }

    /// One-line description of the active configuration for report headers.
    pub fn describe(&self) -> String {
        let mut s = format!("method={}", self.method.as_str());
        if self.method == RiskMethod::Annex {
            s.push_str(
                " (EXAMPLE tables modelled on the ISO/SAE 21434 informative annexes — not normative; verify against your copy of the standard)",
            );
        } else {
            s.push_str(" with [cyber] overrides");
        }
        s
    }
}

#[cfg(test)]
#[path = "cyber_config_tests.rs"]
mod tests;
