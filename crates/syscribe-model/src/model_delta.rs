//! What changed between two states of the model (GH #415, `REQ-TRS-DELTA-001`): the qualified names added,
//! changed and removed, and whether the feature model, any Configuration or the set of validation findings
//! changed. Used by the LSP server to tell a front end what to refresh after a reload; pure, so it is tested
//! without a server.

use std::collections::{BTreeMap, BTreeSet};

use crate::element::{ElementType, RawElement};
use crate::suspect::content_hash;
use crate::validator::Finding;

/// A comparable summary of one state of the model.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Snapshot {
    /// Qualified name -> full-content hash.
    elements: BTreeMap<String, String>,
    /// Qualified names of the feature-model elements (`FeatureDef`, `FeatureModel`).
    feature_model: BTreeSet<String>,
    /// Qualified names of the `Configuration`s.
    configurations: BTreeSet<String>,
    /// `code|file|message` of each finding.
    findings: BTreeSet<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Delta {
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub removed: Vec<String>,
    pub feature_model_changed: bool,
    /// Qualified names of Configurations that were added, changed or removed.
    pub configurations_changed: Vec<String>,
    pub validation_changed: bool,
}

impl Delta {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.changed.is_empty() && self.removed.is_empty() && !self.validation_changed
    }

    pub fn to_json(&self, sequence: u64) -> serde_json::Value {
        serde_json::json!({
            "schemaVersion": "1.0",
            "sequence": sequence,
            "added": self.added,
            "changed": self.changed,
            "removed": self.removed,
            "featureModelChanged": self.feature_model_changed,
            "configurationsChanged": self.configurations_changed,
            "validationChanged": self.validation_changed,
        })
    }
}

pub fn snapshot(elements: &[RawElement], findings: &[Finding]) -> Snapshot {
    let mut s = Snapshot::default();
    for e in elements {
        s.elements.insert(e.qualified_name.clone(), content_hash(e));
        match e.frontmatter.element_type {
            Some(ElementType::FeatureDef | ElementType::FeatureModel) => {
                s.feature_model.insert(e.qualified_name.clone());
            }
            Some(ElementType::Configuration) => {
                s.configurations.insert(e.qualified_name.clone());
            }
            _ => {}
        }
    }
    s.findings = findings.iter().map(|f| format!("{}|{}|{}", f.code, f.file, f.message)).collect();
    s
}

/// The difference `old` -> `new`, each list sorted.
pub fn delta(old: &Snapshot, new: &Snapshot) -> Delta {
    let mut d = Delta::default();
    for (q, h) in &new.elements {
        match old.elements.get(q) {
            None => d.added.push(q.clone()),
            Some(o) if o != h => d.changed.push(q.clone()),
            _ => {}
        }
    }
    d.removed = old.elements.keys().filter(|q| !new.elements.contains_key(*q)).cloned().collect();
    let touched: BTreeSet<&String> = d.added.iter().chain(&d.changed).chain(&d.removed).collect();
    d.feature_model_changed = touched.iter().any(|q| new.feature_model.contains(*q) || old.feature_model.contains(*q));
    d.configurations_changed = touched.iter().filter(|q| new.configurations.contains(**q) || old.configurations.contains(**q)).map(|q| (*q).clone()).collect();
    d.validation_changed = old.findings != new.findings;
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator::Severity;

    fn finding(code: &'static str, file: &str) -> Finding {
        Finding { code, file: file.to_string(), message: "m".into(), severity: Severity::Warning }
    }

    fn snap(parts: &[(&str, &str, Option<&str>)], findings: &[Finding]) -> Snapshot {
        let mut s = Snapshot::default();
        for (q, h, kind) in parts {
            s.elements.insert((*q).into(), (*h).into());
            match *kind {
                Some("feature") => {
                    s.feature_model.insert((*q).into());
                }
                Some("config") => {
                    s.configurations.insert((*q).into());
                }
                _ => {}
            }
        }
        s.findings = findings.iter().map(|f| format!("{}|{}|{}", f.code, f.file, f.message)).collect();
        s
    }

    #[test]
    fn unchanged_is_empty() {
        let a = snap(&[("A", "1", None)], &[finding("W1", "a.md")]);
        assert!(delta(&a, &a.clone()).is_empty());
    }

    #[test]
    fn added_changed_and_removed_are_sorted_and_classified() {
        let old = snap(&[("A", "1", None), ("B", "1", None), ("C", "1", None)], &[]);
        let new = snap(&[("A", "1", None), ("B", "2", None), ("D", "1", None), ("E", "1", None)], &[]);
        let d = delta(&old, &new);
        assert_eq!((d.added.as_slice(), d.changed.as_slice(), d.removed.as_slice()), (&["D".to_string(), "E".to_string()][..], &["B".to_string()][..], &["C".to_string()][..]));
        assert!(!d.feature_model_changed && d.configurations_changed.is_empty() && !d.validation_changed);
    }

    #[test]
    fn feature_model_and_configuration_changes_are_flagged_including_removals() {
        let old = snap(&[("F", "1", Some("feature")), ("C1", "1", Some("config")), ("C2", "1", Some("config"))], &[]);
        let new = snap(&[("F", "2", Some("feature")), ("C1", "1", Some("config"))], &[]);
        let d = delta(&old, &new);
        assert!(d.feature_model_changed);
        assert_eq!(d.configurations_changed, vec!["C2".to_string()], "only the removed one; C1 is untouched");
        // A change to an unrelated element flags neither.
        let d = delta(&snap(&[("X", "1", None)], &[]), &snap(&[("X", "2", None)], &[]));
        assert!(!d.feature_model_changed && d.configurations_changed.is_empty());
    }

    #[test]
    fn a_changed_finding_set_is_flagged_even_when_no_element_changed() {
        let a = snap(&[("A", "1", None)], &[finding("W1", "a.md")]);
        let b = snap(&[("A", "1", None)], &[finding("W1", "a.md"), finding("E2", "b.md")]);
        let d = delta(&a, &b);
        assert!(d.validation_changed && d.added.is_empty() && !d.is_empty());
        let json = d.to_json(7);
        assert_eq!(json["sequence"], 7);
        assert_eq!(json["validationChanged"], true);
    }
}
