//! Mutation analysis of the parts of a composite (REQ-TRS-SYSMUT-001): every part's machine is mutated one fault at a time
//! (the operators of `sm_mut`) and each mutant is judged by running all the composite's scenarios with that one part's
//! machine replaced. Plants are instantiated afresh for every run, so composites with FMU parts work too.

use serde::Serialize;

use crate::sm_mut::{generate, trace_distance, Elem, MutantResult, Uncovered};
use crate::sm_run::{Coverage, Options};
use crate::sm_scen::{Scenario, Status};
use crate::system_run::SystemModel;
use crate::system_scen::run_system_scenario;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartMutReport {
    pub part: String,
    /// `<composite>.<part> (<machine>)`.
    pub machine: String,
    pub machine_hash: String,
    pub score: f64,
    pub killed: usize,
    pub survived: usize,
    pub equivalent_suspect: usize,
    pub mutants: Vec<MutantResult>,
    pub uncovered: Uncovered,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SysMutReport {
    pub composite: String,
    pub hash: String,
    pub scenarios: usize,
    pub score: f64,
    pub killed: usize,
    pub survived: usize,
    pub equivalent_suspect: usize,
    pub parts: Vec<PartMutReport>,
}

fn percent(killed: usize, survived: usize) -> f64 {
    if killed + survived == 0 {
        100.0
    } else {
        100.0 * killed as f64 / (killed + survived) as f64
    }
}

/// Run the analysis over every part with a machine, or only `only_part`. `Err` when a scenario does not pass on the
/// unmutated composite, or `only_part` names no part with a machine.
pub fn analyse(model: &SystemModel, scenarios: &[Scenario], opts: &Options, only_part: Option<&str>) -> Result<SysMutReport, String> {
    let base: Vec<_> = scenarios.iter().map(|s| run_system_scenario(model, s, opts)).collect();
    if let Some(v) = base.iter().find(|v| v.status != Status::Pass) {
        return Err(format!("scenario '{}' does not pass on the unmutated composite: {}", v.scenario, v.message.clone().unwrap_or_default()));
    }
    let wanted: Vec<usize> = model.spec.instances.iter().enumerate().filter(|(i, inst)| model.machines[*i].is_some() && only_part.is_none_or(|p| p == inst.name)).map(|(i, _)| i).collect();
    if wanted.is_empty() {
        return Err(match only_part {
            Some(p) => format!("'{p}' is not a part with a state machine of {}", model.spec.composite),
            None => format!("{} has no part with a state machine", model.spec.composite),
        });
    }
    let (mut killed_all, mut survived_all, mut equiv_all) = (0, 0, 0);
    let mut parts = Vec::new();
    for pi in wanted {
        let inst = &model.spec.instances[pi];
        let m = model.machines[pi].as_ref().expect("filtered");
        let mut cov = Coverage::default();
        for v in &base {
            if let Some(pc) = v.part_coverage.iter().find(|c| c.part == inst.name) {
                cov.merge(&pc.coverage);
            }
        }
        let covered = |e: Elem| match e {
            Elem::Transition(t) => cov.transitions.contains(&t) || cov.conds.contains_key(&(t, 0)) || cov.conds.contains_key(&(t, 1)),
            Elem::State(s) => cov.states.contains(&s),
        };
        let (mut killed, mut survived, mut equiv) = (0, 0, 0);
        let mut results = Vec::new();
        for mu in generate(m) {
            let mut mutant = model.clone();
            mutant.machines[pi] = Some(mu.machine.clone());
            let mut killer = None;
            let mut closest: Option<(usize, String)> = None;
            for (i, sc) in scenarios.iter().enumerate() {
                let v = run_system_scenario(&mutant, sc, opts);
                if v.status != Status::Pass {
                    killer = Some(sc.title.clone());
                    break;
                }
                let d = trace_distance(&v.trace, &base[i].trace);
                if d > 0 && closest.as_ref().is_none_or(|c| d > c.0) {
                    closest = Some((d, sc.title.clone()));
                }
            }
            let (fate, killed_by, closest) = match killer {
                Some(k) => {
                    killed += 1;
                    ("killed", Some(k), None)
                }
                None if closest.is_none() && !covered(mu.elem) => {
                    equiv += 1;
                    ("equivalentSuspect", None, None)
                }
                None => {
                    survived += 1;
                    ("survived", None, closest.map(|c| c.1))
                }
            };
            results.push(MutantResult { id: mu.id, operator: mu.operator, description: mu.description, origin: mu.origin, fate, killed_by, closest });
        }
        killed_all += killed;
        survived_all += survived;
        equiv_all += equiv;
        parts.push(PartMutReport {
            part: inst.name.clone(),
            machine: format!("{}.{} ({})", model.spec.composite, inst.name, m.machine),
            machine_hash: m.hash.clone(),
            score: percent(killed, survived),
            killed,
            survived,
            equivalent_suspect: equiv,
            mutants: results,
            uncovered: Uncovered { states: m.states.iter().filter(|s| !cov.states.contains(&s.id)).map(|s| s.path.clone()).collect(), transitions: m.transitions.iter().filter(|t| !cov.transitions.contains(&t.id)).map(|t| t.id).collect() },
        });
    }
    Ok(SysMutReport { composite: model.spec.composite.clone(), hash: model.hash.clone(), scenarios: scenarios.len(), score: percent(killed_all, survived_all), killed: killed_all, survived: survived_all, equivalent_suspect: equiv_all, parts })
}
