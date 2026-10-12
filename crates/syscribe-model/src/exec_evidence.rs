//! Executable evidence (GH #333, #344): one engine behind two views. For each requirement, the executable scenarios
//! that verify it, what they exercised of the machines that carry it, how strong those scenarios are (mutation score)
//! and which requirements have only descriptive tests or none; for each architecture element, what models it, the
//! scenarios that exercise it and how its requirements are verified. `requirement-coverage` and `exec-coverage` print
//! the two views of the same [`Evidence`], so their numbers cannot disagree.
//!
//! A TestCase is *executable* when it is `active`, names a `subject:` that is a state machine, an action or a composite
//! part, and has Gherkin scenarios. A scenario verifies the TestCase's `verifies:` requirements, narrowed by its
//! `@verifies` tags and `# verifies:` step comments when it has any. It is evidence for a requirement only when its
//! subject exercises an architecture element that satisfies the requirement (the check `W992` makes per TestCase).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::element::{ElementType, RawElement};
use crate::models::{self, Exercised};
use crate::resolver::Resolver;
use crate::sm_run::{Coverage, Options};
use crate::sm_scen::{extract_scenarios, Scenario, Status};

/// Covered out of total.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct Count {
    pub covered: usize,
    pub total: usize,
}

impl Count {
    /// The percentage covered; 100 when there is nothing to cover.
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            100.0 * self.covered as f64 / self.total as f64
        }
    }
}

/// What a set of scenarios exercised of one machine.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineCoverage {
    /// The subject the scenarios ran on (a composite or the machine itself).
    pub subject: String,
    /// The part of the composite; `None` when the subject is the machine.
    pub part: Option<String>,
    pub machine: String,
    pub scenarios: usize,
    pub states: Count,
    pub transitions: Count,
    /// Guard conditions (MC/DC).
    pub guards: Count,
    pub timers: Count,
}

/// A mutation score of one machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationScore {
    /// The subject whose scenarios judged the mutants.
    #[serde(default)]
    pub subject: String,
    pub machine: String,
    pub machine_hash: String,
    pub score: f64,
    pub killed: usize,
    pub survived: usize,
    /// The machine changed since the score was recorded.
    #[serde(default)]
    pub stale: bool,
    /// `recorded` (by `mutate --record`) or `computed` (by `--mutate`).
    #[serde(default)]
    pub source: String,
}

/// One executable scenario that ran.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioRun {
    pub test_case: String,
    pub scenario: String,
    pub subject: String,
    /// `pass`, `fail`, `error`, or `not-run` (a scenario with plants).
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The requirements (ids) it verifies, after narrowing by its tags.
    pub verifies: Vec<String>,
}

/// A scenario named for a requirement.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioRef {
    pub test_case: String,
    pub scenario: String,
    pub status: String,
    pub subject: String,
}

/// A descriptive (non-executable) TestCase.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptiveTest {
    pub test_case: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
}

/// How a requirement is verified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReqVerdict {
    /// Passing executable scenarios exercise what satisfies it, above the thresholds.
    Executable,
    /// Executable scenarios exist but one fails, or their coverage or mutation score is below a threshold.
    ExecutableWeak,
    /// Only descriptive TestCases verify it.
    DescriptiveOnly,
    /// Nothing verifies it.
    Unverified,
    /// Verified by inspection or analysis, or a parent whose children carry the evidence.
    NotApplicable,
}

impl ReqVerdict {
    pub fn label(self) -> &'static str {
        match self {
            ReqVerdict::Executable => "executable",
            ReqVerdict::ExecutableWeak => "executable-weak",
            ReqVerdict::DescriptiveOnly => "descriptive-only",
            ReqVerdict::Unverified => "unverified",
            ReqVerdict::NotApplicable => "not-applicable",
        }
    }

    /// Executable evidence, strong or weak.
    pub fn is_executable(self) -> bool {
        matches!(self, ReqVerdict::Executable | ReqVerdict::ExecutableWeak)
    }
}

/// The evidence for one requirement.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementEvidence {
    pub id: String,
    pub qualified_name: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_method: Option<String>,
    pub verdict: ReqVerdict,
    /// Why the verdict is weak or not applicable.
    pub reasons: Vec<String>,
    pub descriptive: Vec<DescriptiveTest>,
    /// The executable scenarios that verify it and exercise what satisfies it.
    pub scenarios: Vec<ScenarioRef>,
    /// Scenarios that verify it but whose subject exercises nothing that satisfies it.
    pub claimed_not_exercised: Vec<ScenarioRef>,
    pub satisfiers: Vec<String>,
    /// What its scenarios exercised of the machines that carry it.
    pub coverage: Vec<MachineCoverage>,
    pub mutation: Vec<MutationScore>,
}

/// How the requirements an element satisfies are verified.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementSplit {
    /// Executable evidence that exercises this element.
    pub executable: Vec<String>,
    /// Executable evidence that exercises another element satisfying it, not this one.
    pub executable_elsewhere: Vec<String>,
    pub descriptive_only: Vec<String>,
    pub unverified: Vec<String>,
    pub not_applicable: Vec<String>,
}

/// The evidence for one architecture element.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementEvidence {
    pub qualified_name: String,
    pub domain: String,
    /// It carries a state machine itself (`exhibitsStates:`).
    pub executable_itself: bool,
    /// The executable parts that declare `models:` it.
    pub modelled_by: Vec<String>,
    /// The `models:` correspondence findings (`E991`, `W990`, `W991`) that name it.
    pub correspondence: Vec<String>,
    /// The executable scenarios whose subject exercises it.
    pub scenarios: usize,
    /// Those of them that passed.
    pub passing: usize,
    pub mutation: Vec<MutationScore>,
    pub requirements: RequirementSplit,
}

impl ElementEvidence {
    /// Modelled: it carries a machine, or some executable part models it.
    pub fn modelled(&self) -> bool {
        self.executable_itself || !self.modelled_by.is_empty()
    }
}

/// Everything the two views show.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<String>,
    pub scenarios: Vec<ScenarioRun>,
    pub requirements: Vec<RequirementEvidence>,
    pub elements: Vec<ElementEvidence>,
    /// Subjects that could not be built or lowered, with why.
    pub errors: Vec<String>,
}

/// Thresholds below which executable evidence is weak.
#[derive(Debug, Clone, Default)]
pub struct Thresholds {
    pub min_states: Option<f64>,
    pub min_transitions: Option<f64>,
    pub min_guards: Option<f64>,
    pub min_score: Option<f64>,
}

/// How to compute the evidence.
pub struct Request<'a> {
    pub model_root: &'a Path,
    pub configuration: Option<String>,
    /// Compute mutation scores instead of reading the recorded ones.
    pub mutate: bool,
    pub thresholds: Thresholds,
}

fn tc_id(tc: &RawElement) -> String {
    tc.frontmatter.id.clone().unwrap_or_else(|| tc.qualified_name.clone())
}

fn req_id(r: &RawElement) -> String {
    r.frontmatter.id.clone().unwrap_or_else(|| r.qualified_name.clone())
}

fn is_requirement(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::Requirement))
}

fn is_part(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::Part) | Some(ElementType::PartDef))
}

fn is_action(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::ActionDef) | Some(ElementType::Action))
}

fn is_machine(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::StateDef) | Some(ElementType::State))
}

/// The explicit `subject:` of an executable TestCase: a state machine, an action or a composite part. Inference from
/// `satisfies:` is left out on purpose: a descriptive TestCase would otherwise count as executable.
pub fn executable_subject<'a>(elements: &'a [RawElement], resolver: &Resolver, tc: &RawElement) -> Option<&'a RawElement> {
    let s = tc.frontmatter.subject.as_deref()?;
    // a part counts only when it is a composite with an executable part; a plain architecture part is not
    resolver.resolve_ref(elements, s).filter(|e| is_machine(e) || is_action(e) || (is_part(e) && crate::system::resolve(elements, resolver, e).0.is_some()))
}

fn is_active(e: &RawElement) -> bool {
    e.frontmatter.status.as_deref() == Some("active")
}

/// The requirements a TestCase verifies, by qualified name, in order.
fn verified(elements: &[RawElement], resolver: &Resolver, tc: &RawElement) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for r in tc.frontmatter.verifies.iter().flatten() {
        if let Some(req) = resolver.resolve_ref(elements, r).filter(|e| is_requirement(e)) {
            if !out.contains(&req.qualified_name) {
                out.push(req.qualified_name.clone());
            }
        }
    }
    out
}

/// The requirements a scenario verifies: its claims that are among the TestCase's, or all of the TestCase's.
/// The requirements a scenario verifies: its claims that are among the TestCase's `verifies:`; all of the TestCase's
/// when it claims none of them (an invalid claim is ignored, `W994`). A claim of a requirement the TestCase lists that
/// is not in the model (left out of this Configuration) still narrows the scenario.
fn scenario_verifies(elements: &[RawElement], resolver: &Resolver, tc: &RawElement, sc: &Scenario, tc_reqs: &[String]) -> Vec<String> {
    let listed = |r: &str| tc.frontmatter.verifies.iter().flatten().any(|v| v == r);
    let mut narrowed = false;
    let mut claims: Vec<String> = Vec::new();
    for r in sc.claimed() {
        match resolver.resolve_ref(elements, &r).map(|e| e.qualified_name.clone()).filter(|q| tc_reqs.contains(q)) {
            Some(q) => {
                narrowed = true;
                if !claims.contains(&q) {
                    claims.push(q);
                }
            }
            None if listed(&r) && resolver.resolve_ref(elements, &r).is_none() => narrowed = true,
            None => {}
        }
    }
    if narrowed {
        claims
    } else {
        tc_reqs.to_vec()
    }
}

/// `W994`/`W995` for one TestCase's `@verifies` tags and `# verifies:` comments (GH #344).
pub fn tag_findings(elements: &[RawElement], resolver: &Resolver, tc: &RawElement) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    if !tc.doc.to_ascii_lowercase().contains("verifies") {
        return out;
    }
    let scenarios = extract_scenarios(&tc.doc);
    if scenarios.iter().all(|s| s.claimed().is_empty()) {
        return out;
    }
    let id = tc_id(tc);
    let reqs = verified(elements, resolver, tc);
    let mut reported: BTreeSet<String> = BTreeSet::new();
    for sc in &scenarios {
        for r in sc.claimed() {
            let ok = resolver.resolve_ref(elements, &r).filter(|e| is_requirement(e)).is_some_and(|e| reqs.contains(&e.qualified_name))
                // listed but not in the model (inactive in this Configuration): W019's business, not a bad claim
                || (resolver.resolve_ref(elements, &r).is_none() && tc.frontmatter.verifies.iter().flatten().any(|v| *v == r));
            if !ok && reported.insert(r.clone()) {
                out.push(("W994", format!("{id}: scenario '{}' claims {r} (by `@verifies` or `# verifies:`), which is not a requirement this TestCase `verifies:`; the claim is ignored (a scenario with no other claim verifies all of them)", sc.outline.as_deref().unwrap_or(&sc.title))));
            }
        }
    }
    if scenarios.iter().all(|s| !s.claimed().is_empty()) {
        let claimed: BTreeSet<String> = scenarios.iter().flat_map(|s| scenario_verifies(elements, resolver, tc, s, &reqs)).collect();
        for q in reqs.iter().filter(|q| !claimed.contains(*q)) {
            let r = resolver.get(elements, q).map(req_id).unwrap_or_else(|| q.clone());
            out.push(("W995", format!("{id} verifies {r}, but every scenario is tagged and none claims it: no scenario is evidence for it")));
        }
    }
    out
}

/// A recorded mutation score (`mutate --record`), in `<model_root>/.syscribe/mutation.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationRecord {
    /// The subject `mutate` ran on (a machine, a composite or an action).
    #[serde(default)]
    pub subject: String,
    pub machine: String,
    pub machine_hash: String,
    pub score: f64,
    pub killed: usize,
    pub survived: usize,
    pub recorded_at_unix: u64,
}

/// The mutation sidecar's path.
pub fn mutation_sidecar(model_root: &Path) -> PathBuf {
    model_root.join(".syscribe").join("mutation.json")
}

/// The recorded scores, by machine: one per machine hash (a machine projected onto two Configurations hashes
/// differently, so each keeps its own), the latest first.
pub fn load_mutation(model_root: &Path) -> BTreeMap<String, Vec<MutationRecord>> {
    std::fs::read_to_string(mutation_sidecar(model_root)).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

/// Record scores, replacing the earlier one of the same machine, subject and hash.
pub fn record_mutation(model_root: &Path, records: Vec<MutationRecord>) -> std::io::Result<PathBuf> {
    let mut all = load_mutation(model_root);
    for r in records {
        let list = all.entry(r.machine.clone()).or_default();
        list.retain(|x| x.machine_hash != r.machine_hash || x.subject != r.subject);
        list.insert(0, r);
    }
    let path = mutation_sidecar(model_root);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(&all).expect("serialises"))?;
    Ok(path)
}

/// Sum the per-part results of one machine (a machine used by several parts, as two relays) into one score, so
/// `mutate --record` and `--mutate` agree. `(machine, hash, killed, survived)`; environment parts are left out.
pub fn aggregate_parts(parts: impl IntoIterator<Item = (String, String, usize, usize)>) -> Vec<(String, String, usize, usize)> {
    let mut by: BTreeMap<String, (String, usize, usize)> = BTreeMap::new();
    for (m, h, k, s) in parts {
        let e = by.entry(m).or_insert((h, 0, 0));
        e.1 += k;
        e.2 += s;
    }
    by.into_iter().filter(|(_, (_, k, s))| k + s > 0).map(|(m, (h, k, s))| (m, h, k, s)).collect()
}

/// A score in percent.
pub fn score_of(killed: usize, survived: usize) -> f64 {
    if killed + survived == 0 {
        100.0
    } else {
        100.0 * killed as f64 / (killed + survived) as f64
    }
}

/// One executable subject, built once.
enum Built {
    Machine(crate::ir::Machine),
    Composite(crate::system_run::SystemModel),
    Action(crate::act_ir::ActionProgram),
    Failed(String),
}

/// The coverage merged so far for one (subject, part) and the scenarios that went into it.
struct Merged {
    machine: String,
    coverage: Coverage,
    scenarios: usize,
}

fn coverage_of(built: &Built, part: Option<&str>, merged: &Merged, subject: &str) -> Option<MachineCoverage> {
    let m = match (built, part) {
        (Built::Machine(m), None) => m,
        (Built::Composite(sm), Some(p)) => sm.spec.instance(p).and_then(|i| sm.machines[i].as_ref())?,
        _ => return None,
    };
    let rep = crate::sm_cov::report(m, &merged.coverage, merged.scenarios, None, &[]);
    Some(MachineCoverage {
        subject: subject.to_string(),
        part: part.map(String::from),
        machine: merged.machine.clone(),
        scenarios: merged.scenarios,
        states: Count { covered: rep.states.covered, total: rep.states.total },
        transitions: Count { covered: rep.transitions.covered, total: rep.transitions.total },
        guards: Count { covered: rep.guards.covered, total: rep.guards.total },
        timers: Count { covered: rep.timers.covered, total: rep.timers.total },
    })
}

/// The machines (qualified names) a part definition carries along its definition chain.
fn machines_of(elements: &[RawElement], resolver: &Resolver, def: &RawElement) -> Vec<String> {
    crate::system::chain(elements, resolver, def)
        .iter()
        .flat_map(|d| d.frontmatter.exhibits_states.iter().flatten())
        .filter_map(|st| resolver.resolve_ref(elements, st))
        .map(|m| m.qualified_name.clone())
        .collect()
}

/// Compute the evidence over `elements` (already projected for a configuration, if any).
pub fn compute(elements: &[RawElement], req: &Request) -> Evidence {
    let resolver = Resolver::new(elements);
    let satisfiers = models::satisfiers(elements, &resolver);
    let opts = Options::default();
    let mut built: HashMap<String, Built> = HashMap::new();
    let mut reach: HashMap<String, Exercised> = HashMap::new();
    let mut runs: Vec<ScenarioRun> = Vec::new();
    // requirement qname → (exercising scenarios, claimed-not-exercised, descriptive tests)
    let mut exercising: HashMap<String, Vec<(usize, String)>> = HashMap::new();
    let mut not_exercised: HashMap<String, Vec<ScenarioRef>> = HashMap::new();
    let mut descriptive: HashMap<String, Vec<DescriptiveTest>> = HashMap::new();
    // (requirement, subject, part) → merged coverage of the scenarios that verify the requirement
    let mut merged: BTreeMap<(String, String, Option<String>), Merged> = BTreeMap::new();
    let mut errors: Vec<String> = Vec::new();
    // the scenarios of each subject, for `--mutate`
    let mut by_subject: BTreeMap<String, Vec<Scenario>> = BTreeMap::new();
    // the part usages (`features:` entries) of each composite subject, by name
    let mut usages: HashMap<String, HashMap<String, serde_yaml::Mapping>> = HashMap::new();

    for tc in elements.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::TestCase)) && is_active(e)) {
        let reqs = verified(elements, &resolver, tc);
        if reqs.is_empty() {
            continue;
        }
        let scenarios = extract_scenarios(&tc.doc);
        let subject = executable_subject(elements, &resolver, tc).filter(|_| !scenarios.is_empty());
        let Some(subject) = subject else {
            for q in &reqs {
                descriptive.entry(q.clone()).or_default().push(DescriptiveTest { test_case: tc_id(tc), level: tc.frontmatter.test_level.clone() });
            }
            continue;
        };
        let sq = subject.qualified_name.clone();
        let plants = crate::plant_cfg::declares_plants(&tc.frontmatter);
        if !built.contains_key(&sq) {
            let b = if is_part(subject) {
                match crate::system_run::SystemModel::build_in(elements, &sq, req.model_root) {
                    Ok(m) => Built::Composite(m),
                    Err(e) => Built::Failed(e.join("; ")),
                }
            } else if is_action(subject) {
                match crate::act_ir::lower_action(elements, &sq) {
                    Ok(p) => Built::Action(p),
                    Err(d) => Built::Failed(d.iter().map(|(c, m)| format!("{c}: {m}")).collect::<Vec<_>>().join("; ")),
                }
            } else {
                match crate::ir::lower(elements, &sq) {
                    Ok(m) => Built::Machine(m),
                    Err(d) => Built::Failed(d.iter().map(|(c, m)| format!("{c}: {m}")).collect::<Vec<_>>().join("; ")),
                }
            };
            if let Built::Failed(why) = &b {
                let hint = if why.contains("{{Features::") && req.configuration.is_none() { " (it uses feature parameters: give --config)" } else { "" };
                errors.push(format!("{sq}{hint}: {why}"));
            }
            built.insert(sq.clone(), b);
            let r = if is_part(subject) { models::exercised_by(elements, &resolver, subject) } else { models::exercised_by_machine(elements, &resolver, subject) };
            reach.insert(sq.clone(), r);
            if is_part(subject) {
                let view = crate::system::composite_view(elements, &resolver, subject);
                let named = view.features.iter().filter_map(|f| f.as_mapping()).filter_map(|m| Some((m.get("name")?.as_str()?.to_string(), m.clone()))).collect();
                usages.insert(sq.clone(), named);
            }
        }
        let b = &built[&sq];
        let r = &reach[&sq];
        for sc in &scenarios {
            let v = match b {
                _ if plants => None,
                Built::Machine(m) => Some(crate::sm_scen::run_scenario(m, sc, &opts)),
                Built::Composite(m) => Some(crate::system_scen::run_system_scenario(m, sc, &opts)),
                Built::Action(p) => Some(crate::act_scen::run_action_scenario(p, sc, &crate::act_run::ActOptions::default())),
                Built::Failed(_) => None,
            };
            let (status, message) = match (&v, b) {
                (Some(v), _) => (
                    match v.status {
                        Status::Pass => "pass",
                        Status::Fail => "fail",
                        Status::Error => "error",
                    },
                    v.message.clone(),
                ),
                (None, Built::Failed(why)) => ("error", Some(why.clone())),
                (None, _) => ("not-run", Some("the scenario uses plants, which this report does not co-simulate".to_string())),
            };
            let sv = scenario_verifies(elements, &resolver, tc, sc, &reqs);
            let idx = runs.len();
            runs.push(ScenarioRun {
                test_case: tc_id(tc),
                scenario: sc.title.clone(),
                subject: sq.clone(),
                status: status.to_string(),
                message,
                verifies: sv.iter().map(|q| resolver.get(elements, q).map(req_id).unwrap_or_else(|| q.clone())).collect(),
            });
            if !plants {
                by_subject.entry(sq.clone()).or_default().push(sc.clone());
            }
            for q in &sv {
                let sat = satisfiers.get(q).cloned().unwrap_or_default();
                if !models::evidences(elements, &resolver, r, q, &sat) {
                    not_exercised.entry(q.clone()).or_default().push(ScenarioRef { test_case: tc_id(tc), scenario: sc.title.clone(), status: status.to_string(), subject: sq.clone() });
                    continue;
                }
                exercising.entry(q.clone()).or_default().push((idx, status.to_string()));
                let Some(v) = &v else { continue };
                match b {
                    Built::Machine(m) => {
                        let e = merged.entry((q.clone(), sq.clone(), None)).or_insert_with(|| Merged { machine: m.machine.clone(), coverage: Coverage::default(), scenarios: 0 });
                        e.coverage.merge(&v.coverage);
                        e.scenarios += 1;
                    }
                    Built::Composite(sm) => {
                        let whole = sat.is_empty() || models::carries(elements, &resolver, subject, None, &sat);
                        for (i, inst) in sm.spec.instances.iter().enumerate() {
                            let Some(m) = sm.machines[i].as_ref() else { continue };
                            if inst.environment {
                                continue;
                            }
                            // a part carries the requirement by what its usage (else its definition) is or models; every
                            // product part does when the subject itself satisfies it
                            let usage = usages.get(&sq).and_then(|u| u.get(&inst.name));
                            let carries = whole || resolver.get(elements, &inst.def).is_some_and(|d| models::carries(elements, &resolver, d, usage, &sat));
                            if !carries {
                                continue;
                            }
                            let Some(pc) = v.part_coverage.iter().find(|c| c.part == inst.name) else { continue };
                            let e = merged.entry((q.clone(), sq.clone(), Some(inst.name.clone()))).or_insert_with(|| Merged { machine: m.machine.clone(), coverage: Coverage::default(), scenarios: 0 });
                            e.coverage.merge(&pc.coverage);
                            e.scenarios += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // mutation scores, by (subject, machine): a score depends on the scenarios it was judged by, so one machine run
    // as a machine subject and as a part of a composite has a score for each; computed now, or recorded and checked
    // against the machine's current hash
    let mut scores: BTreeMap<(String, String), MutationScore> = BTreeMap::new();
    let score = |subject: &str, machine: String, hash: String, killed: usize, survived: usize, stale: bool, source: &str| MutationScore {
        subject: subject.to_string(),
        machine,
        machine_hash: hash,
        score: score_of(killed, survived),
        killed,
        survived,
        stale,
        source: source.into(),
    };
    if req.mutate {
        for (sq, scs) in &by_subject {
            match &built[sq] {
                Built::Machine(m) => {
                    if let Ok(rep) = crate::sm_mut::analyse(m, scs, &opts).map_err(|_| ()).and_then(|r| if r.killed + r.survived > 0 { Ok(r) } else { Err(()) }) {
                        scores.insert((sq.clone(), m.machine.clone()), score(sq, m.machine.clone(), rep.machine_hash, rep.killed, rep.survived, false, "computed"));
                    }
                }
                Built::Composite(sm) => {
                    if let Ok(rep) = crate::system_mut::analyse(sm, scs, &opts, None) {
                        let parts = rep.parts.iter().filter_map(|p| {
                            let i = sm.spec.instance(&p.part)?;
                            let m = sm.machines[i].as_ref().filter(|_| !sm.spec.instances[i].environment)?;
                            Some((m.machine.clone(), p.machine_hash.clone(), p.killed, p.survived))
                        });
                        for (m, h, k, sv) in aggregate_parts(parts) {
                            scores.insert((sq.clone(), m.clone()), score(sq, m, h, k, sv, false, "computed"));
                        }
                    }
                }
                _ => {}
            }
        }
    } else {
        for (machine, recs) in load_mutation(req.model_root) {
            // per subject, the record of the machine as it is now; else the latest one, stale (or unchecked: it does
            // not lower)
            let current = crate::ir::lower(elements, &machine).ok().map(|m| m.hash);
            let subjects: BTreeSet<String> = recs.iter().map(|r| r.subject.clone()).collect();
            for subject in subjects {
                let mine: Vec<&MutationRecord> = recs.iter().filter(|r| r.subject == subject).collect();
                let Some(rec) = current.as_ref().and_then(|h| mine.iter().find(|r| &r.machine_hash == h)).or(mine.first()) else { continue };
                let stale = current.as_ref() != Some(&rec.machine_hash);
                scores.insert((subject.clone(), machine.clone()), score(&subject, machine.clone(), rec.machine_hash.clone(), rec.killed, rec.survived, stale, "recorded"));
            }
        }
    }
    let score_for = |subject: &str, machine: &str| scores.get(&(subject.to_string(), machine.to_string()));

    // requirements
    let th = &req.thresholds;
    let mut requirements: Vec<RequirementEvidence> = Vec::new();
    for r in elements.iter().filter(|e| is_requirement(e)) {
        let q = &r.qualified_name;
        let ex = exercising.get(q).cloned().unwrap_or_default();
        let coverage: Vec<MachineCoverage> = merged
            .range((q.clone(), String::new(), None)..)
            .take_while(|((rq, ..), _)| rq == q)
            .filter_map(|((_, s, p), m)| coverage_of(&built[s], p.as_deref(), m, s))
            .collect();
        let mutation: Vec<MutationScore> = coverage.iter().filter_map(|c| score_for(&c.subject, &c.machine).map(|m| ((m.subject.clone(), m.machine.clone()), m.clone()))).collect::<BTreeMap<(String, String), MutationScore>>().into_values().collect();
        let vm = r.frontmatter.verification_method.clone();
        let mut reasons: Vec<String> = Vec::new();
        let verdict = if matches!(vm.as_deref(), Some("inspection") | Some("analysis")) && ex.is_empty() {
            reasons.push(format!("verified by {}", vm.as_deref().unwrap_or_default()));
            ReqVerdict::NotApplicable
        } else if !ex.is_empty() {
            for (i, st) in &ex {
                let run = &runs[*i];
                let why = match &built[&run.subject] {
                    Built::Failed(_) => format!("{} could not be built, so its scenarios did not run", run.subject),
                    _ if st == "not-run" => format!("scenario '{}' of {} uses plants and was not run", run.scenario, run.test_case),
                    _ => format!("scenario '{}' of {} is {st}", run.scenario, run.test_case),
                };
                if st != "pass" && !reasons.contains(&why) {
                    reasons.push(why);
                }
            }
            for c in &coverage {
                let who = match &c.part {
                    Some(p) => format!("{}.{p}", c.subject),
                    None => c.machine.clone(),
                };
                for (what, got, min) in [("states", c.states.percent(), th.min_states), ("transitions", c.transitions.percent(), th.min_transitions), ("guard conditions", c.guards.percent(), th.min_guards)] {
                    if let Some(min) = min.filter(|m| got + 1e-9 < *m) {
                        reasons.push(format!("{who}: {what} {got:.0}% < {min}%"));
                    }
                }
            }
            let coverage_gate = th.min_states.is_some() || th.min_transitions.is_some() || th.min_guards.is_some();
            if coverage.is_empty() && (coverage_gate || th.min_score.is_some()) {
                // an action subject, or no part of a composite carries it: a threshold cannot be checked, so it is not met
                reasons.push("no state-machine coverage was measured for it, so the thresholds cannot be checked".into());
            }
            if let Some(min) = th.min_score {
                for m in &mutation {
                    if m.score + 1e-9 < min {
                        reasons.push(format!("{}: mutation score {:.0}% < {min}%", m.machine, m.score));
                    }
                }
                for c in coverage.iter().filter(|c| score_for(&c.subject, &c.machine).is_none()) {
                    let why = format!("{}: no mutation score from the scenarios of {} (run `mutate {} --record` or use --mutate)", c.machine, c.subject, c.subject);
                    if !reasons.contains(&why) {
                        reasons.push(why);
                    }
                }
            }
            if reasons.is_empty() {
                ReqVerdict::Executable
            } else {
                ReqVerdict::ExecutableWeak
            }
        } else {
            if let Some(n) = not_exercised.get(q) {
                reasons.push(format!("{} executable scenario(s) claim it, but their subject exercises nothing that satisfies it", n.len()));
            }
            if descriptive.contains_key(q) {
                ReqVerdict::DescriptiveOnly
            } else {
                ReqVerdict::Unverified
            }
        };
        requirements.push(RequirementEvidence {
            id: req_id(r),
            qualified_name: q.clone(),
            name: r.frontmatter.name.clone().unwrap_or_default(),
            verification_method: vm,
            verdict,
            reasons,
            descriptive: descriptive.get(q).cloned().unwrap_or_default(),
            scenarios: ex.iter().map(|(i, st)| ScenarioRef { test_case: runs[*i].test_case.clone(), scenario: runs[*i].scenario.clone(), status: st.clone(), subject: runs[*i].subject.clone() }).collect(),
            claimed_not_exercised: not_exercised.get(q).cloned().unwrap_or_default(),
            satisfiers: satisfiers.get(q).cloned().unwrap_or_default(),
            coverage,
            mutation,
        });
    }
    // a parent with no executable evidence of its own needs none when every child has it or needs none (GH #344)
    let mut children: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, r) in requirements.iter().enumerate() {
        let Some(e) = resolver.get(elements, &r.qualified_name) else { continue };
        for p in e.frontmatter.derived_from.iter().flatten() {
            if let Some(pe) = resolver.resolve_ref(elements, p).filter(|x| is_requirement(x)) {
                children.entry(pe.qualified_name.clone()).or_default().push(i);
            }
        }
    }
    loop {
        let mut changed = false;
        for i in 0..requirements.len() {
            let r = &requirements[i];
            if !matches!(r.verdict, ReqVerdict::DescriptiveOnly | ReqVerdict::Unverified) {
                continue;
            }
            let Some(ch) = children.get(&r.qualified_name) else { continue };
            if ch.iter().all(|c| matches!(requirements[*c].verdict, ReqVerdict::Executable | ReqVerdict::NotApplicable)) {
                requirements[i].verdict = ReqVerdict::NotApplicable;
                requirements[i].reasons = vec!["a parent requirement: every child has executable evidence or needs none".into()];
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for i in 0..requirements.len() {
        if !matches!(requirements[i].verdict, ReqVerdict::DescriptiveOnly | ReqVerdict::Unverified) {
            continue;
        }
        let Some(ch) = children.get(&requirements[i].qualified_name) else { continue };
        let short: Vec<String> = ch.iter().map(|c| &requirements[*c]).filter(|c| !matches!(c.verdict, ReqVerdict::Executable | ReqVerdict::NotApplicable)).map(|c| format!("{} ({})", c.id, c.verdict.label())).collect();
        requirements[i].reasons.push(format!("a parent requirement whose children lack executable evidence: {}", short.join(", ")));
    }
    requirements.sort_by(|a, b| a.id.cmp(&b.id));

    // architecture elements
    let by_req: HashMap<&str, &RequirementEvidence> = requirements.iter().map(|r| (r.qualified_name.as_str(), r)).collect();
    let modelled_by = models::modelled_by(elements, &resolver);
    let findings = models::findings(elements, &resolver);
    let mut elements_out: Vec<ElementEvidence> = Vec::new();
    for e in elements.iter().filter(|e| is_part(e)) {
        let domain = crate::system::chain(elements, &resolver, e).iter().find_map(|d| d.frontmatter.domain.clone());
        let Some(domain) = domain.filter(|d| d == "software" || d == "hardware") else { continue };
        let q = &e.qualified_name;
        let named = |msg: &str| msg.contains(&format!("'{q}'")) || msg.contains(&format!("'{q}.")) || msg.contains(&format!("of '{q}'"));
        let correspondence: Vec<String> = findings.iter().filter(|(c, _, m)| matches!(*c, "E991" | "W990" | "W991") && named(m)).map(|(c, _, m)| format!("{c}: {m}")).collect();
        let keys: Vec<String> = std::iter::once(q.clone()).collect();
        let reaches = |subject: &str| reach.get(subject).is_some_and(|r| keys.iter().any(|k| r.reaches(k)));
        let scenarios = runs.iter().filter(|s| reaches(&s.subject)).count();
        let passing = runs.iter().filter(|s| s.status == "pass" && reaches(&s.subject)).count();
        let mut carriers: Vec<String> = machines_of(elements, &resolver, e);
        for p in modelled_by.get(q).into_iter().flatten() {
            if let Some(pe) = resolver.get(elements, p) {
                carriers.extend(machines_of(elements, &resolver, pe));
            }
        }
        carriers.sort();
        carriers.dedup();
        let mutation: Vec<MutationScore> = scores.values().filter(|m| carriers.contains(&m.machine)).cloned().collect();
        let mut split = RequirementSplit::default();
        for rq in e.frontmatter.satisfies.iter().flatten().filter_map(|r| resolver.resolve_ref(elements, r)) {
            let Some(ev) = by_req.get(rq.qualified_name.as_str()) else { continue };
            let here = ev.scenarios.iter().any(|s| reaches(&s.subject));
            let list = match ev.verdict {
                ReqVerdict::Executable | ReqVerdict::ExecutableWeak if here => &mut split.executable,
                // the evidence exercises another element that satisfies it, not this one
                ReqVerdict::Executable | ReqVerdict::ExecutableWeak => &mut split.executable_elsewhere,
                ReqVerdict::DescriptiveOnly => &mut split.descriptive_only,
                ReqVerdict::Unverified => &mut split.unverified,
                ReqVerdict::NotApplicable => &mut split.not_applicable,
            };
            if !list.contains(&ev.id) {
                list.push(ev.id.clone());
            }
        }
        elements_out.push(ElementEvidence {
            qualified_name: q.clone(),
            domain,
            executable_itself: !machines_of(elements, &resolver, e).is_empty(),
            modelled_by: modelled_by.get(q).cloned().unwrap_or_default(),
            correspondence,
            scenarios,
            passing,
            mutation,
            requirements: split,
        });
    }
    elements_out.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));

    Evidence { configuration: req.configuration.clone(), scenarios: runs, requirements, elements: elements_out, errors }
}
