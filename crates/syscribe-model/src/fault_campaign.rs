//! The fault-injection campaign (REQ-TRS-FAULT-002): the safety invariants of a composite — its scenarios tagged
//! `@invariant` — run unchanged under sets of injected faults (single FMEA failure modes, minimal cut sets of a fault
//! tree), reporting which sets the design survives.

use serde::Serialize;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::sm_run::Options;
use crate::sm_scen::{Scenario, Status};
use crate::system_run::SystemModel;
use crate::system_scen::{run_system_scenario, run_system_scenario_faulted};

/// A set of faults injected together before the invariants run.
#[derive(Debug, Clone)]
pub struct FaultSet {
    pub ids: Vec<String>,
    /// Where the set comes from: `FMEA row`, `cut set of FT-1, order 2`, …
    pub origin: String,
}

impl FaultSet {
    pub fn label(&self) -> String {
        self.ids.join(" + ")
    }
}

#[derive(Debug, Serialize)]
pub struct Finding {
    pub scenario: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SetResult {
    pub label: String,
    pub origin: String,
    pub faults: Vec<String>,
    /// `survived`, `defeated` (an invariant fails) or `error` (the run could not be made).
    pub status: &'static str,
    pub defeats: Vec<Finding>,
    pub errors: Vec<Finding>,
}

#[derive(Debug, Serialize)]
pub struct NotInjectable {
    pub origin: String,
    pub events: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Campaign {
    pub composite: String,
    pub hash: String,
    pub invariants: usize,
    pub sets: Vec<SetResult>,
    pub not_injectable: Vec<NotInjectable>,
    /// The largest cut-set order that was run (with `--ft`); larger orders are not run.
    pub cut_set_order: Option<usize>,
    /// The fault tree has minimal cut sets above `cut_set_order`; they were left out of the campaign.
    pub cut_sets_truncated: bool,
    pub fmea: FmeaCoverage,
}

/// What the campaign exercised of the analyses.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FmeaCoverage {
    /// Failure modes of the composite's parts that have a fault model and were in a set that ran.
    pub exercised: Vec<String>,
    /// … that have one and were in no set (restricted away, or only in a cut set that could not run).
    pub not_exercised: Vec<String>,
    /// … that have no usable fault model (prose only, or a malformed `injection:`), so they cannot be exercised.
    pub without_model: Vec<String>,
}

/// Run the invariants under each set. `Err` when an invariant does not pass on the composite without faults.
pub fn run(model: &SystemModel, invariants: &[Scenario], opts: &Options, sets: &[FaultSet]) -> Result<Campaign, String> {
    for sc in invariants {
        let v = run_system_scenario(model, sc, opts);
        if v.status != Status::Pass {
            return Err(format!("the invariant '{}' does not hold without faults, so it cannot judge them: {}", sc.title, v.message.unwrap_or_default()));
        }
    }
    let mut results = Vec::new();
    for set in sets {
        let (mut defeats, mut errors) = (Vec::new(), Vec::new());
        for sc in invariants {
            let v = run_system_scenario_faulted(model, sc, opts, &set.ids);
            match v.status {
                Status::Pass => {}
                Status::Fail => defeats.push(Finding { scenario: sc.title.clone(), message: v.message.unwrap_or_default() }),
                Status::Error => errors.push(Finding { scenario: sc.title.clone(), message: v.message.unwrap_or_default() }),
            }
        }
        // a real defeat outranks an invariant that could not be evaluated in the same set
        let status = if !defeats.is_empty() {
            "defeated"
        } else if !errors.is_empty() {
            "error"
        } else {
            "survived"
        };
        results.push(SetResult { label: set.label(), origin: set.origin.clone(), faults: set.ids.clone(), status, defeats, errors });
    }
    Ok(Campaign { composite: model.spec.composite.clone(), hash: model.hash.clone(), invariants: invariants.len(), sets: results, not_injectable: Vec::new(), cut_set_order: None, cut_sets_truncated: false, fmea: FmeaCoverage::default() })
}

/// The FMEA rows and fault-tree events whose `ref` is, or specialises into, a definition of a part of the composite:
/// `(id, has a fault model)`.
pub fn analysis_rows(elements: &[RawElement], resolver: &Resolver, model: &SystemModel) -> Vec<(String, bool)> {
    let defs: Vec<String> = model
        .spec
        .instances
        .iter()
        .filter_map(|i| resolver.resolve_ref(elements, &i.def))
        .flat_map(|d| crate::system::chain(elements, resolver, d).into_iter().map(|c| c.qualified_name.clone()).collect::<Vec<_>>())
        .collect();
    let mut out = Vec::new();
    for e in elements {
        let fm = &e.frontmatter;
        let (id, target) = match fm.element_type {
            Some(ElementType::FMEAEntry) => (fm.id.clone(), fm.subject.clone()),
            Some(ElementType::FaultTreeEvent) => (fm.id.clone(), fm.event_ref.clone()),
            _ => continue,
        };
        let (Some(id), Some(target)) = (id, target) else { continue };
        if resolver.resolve_ref(elements, &target).is_some_and(|t| defs.contains(&t.qualified_name)) {
            out.push((id, fm.injection.as_ref().is_some_and(|i| crate::fault::parse_injection(i).is_ok())));
        }
    }
    out
}
