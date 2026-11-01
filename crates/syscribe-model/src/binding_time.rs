//! Binding time — the lifecycle stage at which a feature selection or parameter
//! value is resolved (`ADR-SYS-BT-001`, `REQ-TRS-BT-001..005`; extends
//! `REQ-TRS-PARAM-004`).
//!
//! * [`BindingTimeVocab`] is the ordered stage vocabulary: the built-in
//!   `compile < load < runtime`, refinable by `[bindingTimes.<name>]` tables in
//!   `<model_root>/.syscribe.toml` (the `[linkTypes]` pattern). A malformed entry
//!   is ignored whole and reported as `W750`; the rest of the vocabulary stays in
//!   force. A project can insert stages (`after:` / `before:` a neighbour) but can
//!   never rename or reorder the three defaults.
//! * [`BindingTime`] is the one type that owns parsing, ordering and comparison.
//!   Every consumer (validator, feature model, placeholders, discovery) goes
//!   through it; none compares against a stage-name literal or a rank number.
//! * [`install`] / [`active`] hold the loaded model's vocabulary for consumers that
//!   carry no [`crate::config::ValidateConfig`], installed by
//!   `ValidateConfig::with_model_root` like [`crate::link_types::install`].

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, OnceLock, RwLock};

/// The stage a running system resolves by itself. It is the one stage with
/// behaviour of its own (`W027`, the `W017`/`W248` carve-outs, `W246`).
const RUNTIME: &str = "runtime";

/// One declared stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage {
    pub name: String,
    pub description: Option<String>,
    /// Free label for who resolves values at this stage (integrator, installer, system, …).
    pub binder: Option<String>,
}

fn builtin(name: &str, description: &str, binder: Option<&str>) -> Stage {
    Stage {
        name: name.to_string(),
        description: Some(description.to_string()),
        binder: binder.map(str::to_string),
    }
}

/// The built-in stages, earliest first.
fn default_stages() -> Vec<Stage> {
    vec![
        builtin("compile", "build / code generation", None),
        builtin("load", "deployment / installation / startup", None),
        builtin(RUNTIME, "dynamically, while the system executes", Some("system")),
    ]
}

/// The ordered binding-time vocabulary of a model, plus the `W750` defect
/// messages for `[bindingTimes]` entries that were ignored.
#[derive(Debug, Clone)]
pub struct BindingTimeVocab {
    stages: Vec<Stage>,
    defects: Vec<String>,
    /// A `[bindingTimes]` table was present (even if every entry in it was invalid).
    declared: bool,
}

impl Default for BindingTimeVocab {
    fn default() -> Self {
        Self { stages: default_stages(), defects: Vec::new(), declared: false }
    }
}

/// Keys accepted on a `[bindingTimes.<name>]` entry.
const KNOWN_KEYS: &[&str] = &["description", "binder", "after", "before"];

/// One not-yet-placed `[bindingTimes.<name>]` entry.
struct Pending {
    stage: Stage,
    after: Option<String>,
    before: Option<String>,
}

fn is_basic_name(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some(f) if f.is_ascii_alphabetic() || f == '_')
        && c.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl BindingTimeVocab {
    /// Load `[bindingTimes]` from `<model_root>/.syscribe.toml`. The built-in
    /// vocabulary when the file is absent, unparseable, or has no such table.
    pub fn load(model_root: &Path) -> Self {
        std::fs::read_to_string(model_root.join(".syscribe.toml"))
            .map(|t| Self::from_toml_str(&t))
            .unwrap_or_default()
    }

    /// Parse the `[bindingTimes]` table out of a whole `.syscribe.toml` text.
    pub fn from_toml_str(text: &str) -> Self {
        let Ok(doc) = text.parse::<toml::Table>() else { return Self::default() };
        let Some(raw) = doc.get("bindingTimes") else { return Self::default() };
        let mut vocab = Self { declared: true, ..Self::default() };
        let Some(table) = raw.as_table() else {
            vocab.defects.push("[bindingTimes] must be a table of `[bindingTimes.<name>]` entries — table ignored".to_string());
            return vocab;
        };

        // Pass 1: each entry on its own — name, keys, value types, self-reference.
        let mut pending: BTreeMap<String, Pending> = BTreeMap::new();
        let mut ignored: Vec<String> = Vec::new();
        let mut placed: BTreeMap<String, Stage> = BTreeMap::new();
        for (name, value) in table {
            match Self::parse_entry(name, value) {
                Ok(p) => {
                    pending.insert(name.clone(), p);
                }
                Err(d) => {
                    vocab.defects.push(format!("[bindingTimes.{name}] {d} — entry ignored"));
                    ignored.push(name.clone());
                }
            }
        }

        // Pass 2: place entries relative to a neighbour already in the order, until
        // no further entry can be placed. Sorted by name, so the result is deterministic.
        loop {
            let ready: Vec<String> = pending
                .iter()
                .filter(|(_, p)| {
                    [&p.after, &p.before].into_iter().flatten().all(|n| vocab.rank_of(n).is_some())
                })
                .map(|(n, _)| n.clone())
                .collect();
            if ready.is_empty() {
                break;
            }
            // Where each ready entry goes, against the order as it stands at the start
            // of the pass. Entries that would share a gap have no defensible relative
            // position, so none of them is placed.
            let mut slots: BTreeMap<usize, Vec<String>> = BTreeMap::new();
            for name in ready {
                let Some(p) = pending.remove(&name) else { continue };
                let after = p.after.as_deref().and_then(|n| vocab.rank_of(n));
                let before = p.before.as_deref().and_then(|n| vocab.rank_of(n));
                let at = match (after, before) {
                    (Some(a), Some(b)) if a >= b => {
                        vocab.defects.push(format!(
                            "[bindingTimes.{name}] `after: {}` is not earlier than `before: {}` — entry ignored",
                            p.after.as_deref().unwrap_or_default(),
                            p.before.as_deref().unwrap_or_default()
                        ));
                        ignored.push(name);
                        continue;
                    }
                    (Some(a), _) => a + 1,
                    (None, Some(b)) => b,
                    (None, None) => unreachable!("pass 1 rejects an entry with neither neighbour"),
                };
                slots.entry(at).or_default().push(name.clone());
                placed.insert(name, p.stage);
            }
            let mut inserts: Vec<(usize, Stage)> = Vec::new();
            for (at, names) in slots {
                if let [only] = names.as_slice() {
                    if let Some(stage) = placed.remove(only) {
                        inserts.push((at, stage));
                    }
                    continue;
                }
                let gap = format!(
                    "between {} and {}",
                    vocab.stages.get(at.wrapping_sub(1)).map_or("the start".to_string(), |s| format!("'{}'", s.name)),
                    vocab.stages.get(at).map_or("the end".to_string(), |s| format!("'{}'", s.name)),
                );
                for name in &names {
                    placed.remove(name);
                    let others: Vec<String> = names.iter().filter(|n| *n != name).map(|n| format!("'{n}'")).collect();
                    vocab.defects.push(format!(
                        "[bindingTimes.{name}] has the same position as {} ({gap}) — order them by naming one as the other's `after:`/`before:` — entry ignored",
                        others.join(", ")
                    ));
                    ignored.push(name.clone());
                }
            }
            // Highest gap first, so earlier insertions do not shift later indexes.
            inserts.sort_by(|a, b| b.0.cmp(&a.0));
            for (at, stage) in inserts {
                vocab.stages.insert(at, stage);
            }
        }

        // Whatever is left names a neighbour that never became part of the order.
        for (name, p) in pending {
            let reason = [&p.after, &p.before]
                .into_iter()
                .flatten()
                .find(|n| vocab.rank_of(n).is_none())
                .map(|n| {
                    if ignored.contains(n) {
                        format!("its neighbour '{n}' is itself ignored")
                    } else if table.contains_key(n.as_str()) {
                        format!("it is part of a cyclic order through '{n}'")
                    } else {
                        format!("its neighbour '{n}' is not a declared stage")
                    }
                })
                .unwrap_or_else(|| "its position cannot be resolved".to_string());
            vocab.defects.push(format!("[bindingTimes.{name}] {reason} — entry ignored"));
        }
        vocab.defects.sort();
        vocab
    }

    /// Validate one entry in isolation. `Err` is the defect text (without the
    /// `[bindingTimes.<name>]` prefix).
    fn parse_entry(name: &str, value: &toml::Value) -> Result<Pending, String> {
        if !is_basic_name(name) {
            return Err(format!("name '{name}' is not a basic name (letters, digits and `_`, not starting with a digit)"));
        }
        if default_stages().iter().any(|s| s.name == name) {
            return Err(format!("redeclares the built-in stage '{name}'"));
        }
        let Some(t) = value.as_table() else {
            return Err("must be a table".to_string());
        };
        let mut stage = Stage { name: name.to_string(), description: None, binder: None };
        let (mut after, mut before) = (None, None);
        for (k, v) in t {
            if !KNOWN_KEYS.contains(&k.as_str()) {
                return Err(format!("unknown key '{k}' (expected one of {})", KNOWN_KEYS.join(", ")));
            }
            let Some(s) = v.as_str() else {
                return Err(format!("`{k}` must be a string"));
            };
            match k.as_str() {
                "description" => stage.description = Some(s.to_string()),
                "binder" => stage.binder = Some(s.to_string()),
                "after" => after = Some(s.to_string()),
                _ => before = Some(s.to_string()),
            }
        }
        if after.is_none() && before.is_none() {
            return Err("has neither `after:` nor `before:`, so its position is undefined".to_string());
        }
        if after.as_deref() == Some(name) || before.as_deref() == Some(name) {
            return Err("names itself as its neighbour".to_string());
        }
        Ok(Pending { stage, after, before })
    }

    /// The effective stages, earliest first.
    pub fn stages(&self) -> &[Stage] {
        &self.stages
    }

    /// The `W750` messages for ignored `[bindingTimes]` entries.
    pub fn defects(&self) -> &[String] {
        &self.defects
    }

    /// A `[bindingTimes]` table is present (possibly with only invalid entries).
    pub fn is_declared(&self) -> bool {
        self.declared
    }

    /// Position of a stage in the order (0 = earliest).
    pub fn rank_of(&self, name: &str) -> Option<usize> {
        self.stages.iter().position(|s| s.name == name)
    }

    /// The stage names, earliest first, as `a, b, c` (for diagnostics).
    pub fn names_hint(&self) -> String {
        self.stages.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", ")
    }

    /// `binding-times --json` / the MCP `binding_times` tool payload.
    pub fn to_json(&self) -> serde_json::Value {
        let stages: Vec<serde_json::Value> = self
            .stages
            .iter()
            .enumerate()
            .map(|(i, s)| {
                serde_json::json!({
                    "name": s.name,
                    "order": i,
                    "description": s.description,
                    "binder": s.binder,
                    "builtin": default_stages().iter().any(|d| d.name == s.name),
                })
            })
            .collect();
        serde_json::json!({ "bindingTimes": stages })
    }

    /// The `## Project binding times` section appended to `--agent-instructions`
    /// when a model root refines the vocabulary. `None` for the built-in vocabulary,
    /// so nothing is appended.
    pub fn agent_instructions_section(&self) -> Option<String> {
        if self.stages == default_stages() {
            return None;
        }
        let mut s = String::from("\n\n## Project binding times\n\n");
        s.push_str(
            "This model declares its binding-time stages in `.syscribe.toml`. A `bindingTime:` \
             value must be one of these, earliest first — never invent a stage; an unknown one is `E230`.\n\n",
        );
        for (i, st) in self.stages.iter().enumerate() {
            s.push_str(&format!("{}. `{}`", i + 1, st.name));
            if let Some(d) = &st.description {
                s.push_str(&format!(" — {d}"));
            }
            if let Some(b) = &st.binder {
                s.push_str(&format!(" (bound by: {b})"));
            }
            s.push('\n');
        }
        Some(s)
    }
}

static ACTIVE: OnceLock<RwLock<Arc<BindingTimeVocab>>> = OnceLock::new();

/// Install the active vocabulary (the loaded model's), used by [`BindingTime`] for
/// consumers that carry no [`crate::config::ValidateConfig`]. An absent
/// `[bindingTimes]` installs the built-in one, clearing any previous.
pub fn install(vocab: &BindingTimeVocab) {
    let lock = ACTIVE.get_or_init(|| RwLock::new(Arc::new(BindingTimeVocab::default())));
    if let Ok(mut w) = lock.write() {
        *w = Arc::new(vocab.clone());
    }
}

/// The active vocabulary (the built-in one until another is installed).
pub fn active() -> Arc<BindingTimeVocab> {
    ACTIVE
        .get()
        .and_then(|l| l.read().ok().map(|r| r.clone()))
        .unwrap_or_default()
}

/// A binding-time stage: an index into the active vocabulary, ordered earliest to
/// latest. The one place that parses, orders and compares stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingTime(u16);

impl BindingTime {
    /// The stage named `name` in the active vocabulary.
    pub fn parse(name: &str) -> Option<Self> {
        Self::parse_in(&active(), name)
    }

    /// The stage named `name` in `vocab`.
    pub fn parse_in(vocab: &BindingTimeVocab, name: &str) -> Option<Self> {
        vocab.rank_of(name).and_then(|r| u16::try_from(r).ok()).map(Self)
    }

    /// Position in the order (0 = earliest).
    pub fn rank(self) -> usize {
        self.0 as usize
    }

    /// The stage's name in the active vocabulary.
    pub fn name(self) -> String {
        active().stages().get(self.rank()).map(|s| s.name.clone()).unwrap_or_default()
    }

    /// The stage the running system resolves itself.
    pub fn is_runtime(self) -> bool {
        active().stages().get(self.rank()).is_some_and(|s| s.name == RUNTIME)
    }

    /// The binding time declared by a `bindingTime:` value, if it names a stage.
    pub fn from_yaml(v: Option<&serde_yaml::Value>) -> Option<Self> {
        v.and_then(|v| v.as_str()).and_then(Self::parse)
    }
}

/// The range of stages over which a feature's selection or a parameter's value may be
/// bound (`REQ-TRS-BT-006`): `bindingTime:` is the earliest, `latestBindingTime:` the
/// latest. A single `bindingTime:` is the degenerate window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingWindow {
    pub earliest: BindingTime,
    pub latest: BindingTime,
}

/// A defect found while reading a window off a declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowIssue {
    /// `field` (`bindingTime` or `latestBindingTime`) names a stage the vocabulary lacks.
    UnknownStage { field: &'static str, value: String },
    /// `latestBindingTime:` is earlier than `bindingTime:`.
    Inverted { earliest: String, latest: String },
}

impl BindingWindow {
    /// Read a window off `bindingTime:` / `latestBindingTime:`. Neither field means
    /// *unspecified* (`None`, exempt from every binding-time check). A field naming an
    /// unknown stage is reported and dropped, and the window is read from what is left.
    /// `latestBindingTime:` alone means "no earlier than the first stage, no later than
    /// `latest`".
    pub fn read(earliest: Option<&str>, latest: Option<&str>) -> (Option<Self>, Vec<WindowIssue>) {
        Self::read_in(&active(), earliest, latest)
    }

    pub fn read_in(
        vocab: &BindingTimeVocab,
        earliest: Option<&str>,
        latest: Option<&str>,
    ) -> (Option<Self>, Vec<WindowIssue>) {
        let mut issues = Vec::new();
        let mut one = |field: &'static str, v: Option<&str>| {
            let v = v?;
            let bt = BindingTime::parse_in(vocab, v);
            if bt.is_none() {
                issues.push(WindowIssue::UnknownStage { field, value: v.to_string() });
            }
            bt
        };
        let (e, l) = (one("bindingTime", earliest), one("latestBindingTime", latest));
        let window = match (e, l) {
            (None, None) => None,
            (Some(e), None) => Some(Self { earliest: e, latest: e }),
            (None, Some(l)) => Some(Self { earliest: BindingTime(0), latest: l }),
            (Some(e), Some(l)) if l < e => {
                issues.push(WindowIssue::Inverted {
                    earliest: vocab.stages()[e.rank()].name.clone(),
                    latest: vocab.stages()[l.rank()].name.clone(),
                });
                None
            }
            (Some(e), Some(l)) => Some(Self { earliest: e, latest: l }),
        };
        (window, issues)
    }

    /// `true` when `t` lies in the window.
    pub fn contains(self, t: BindingTime) -> bool {
        self.earliest <= t && t <= self.latest
    }

    /// `true` when this window closes before `other` opens — so a value in this window
    /// cannot be bound at or after anything in `other`.
    pub fn closes_before_opens(self, other: Self) -> bool {
        self.latest < other.earliest
    }

    /// `compile` for a single stage, `compile..load` for a span (active vocabulary).
    pub fn describe(self) -> String {
        if self.earliest == self.latest {
            self.earliest.name()
        } else {
            format!("{}..{}", self.earliest.name(), self.latest.name())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &BindingTimeVocab) -> Vec<&str> {
        v.stages().iter().map(|s| s.name.as_str()).collect()
    }

    #[test]
    fn default_vocabulary_is_compile_load_runtime() {
        let v = BindingTimeVocab::default();
        assert_eq!(names(&v), ["compile", "load", "runtime"]);
        assert!(!v.is_declared() && v.defects().is_empty());
        assert_eq!(BindingTimeVocab::from_toml_str("[ids]\nmax_digits = 5\n").stages().len(), 3);
    }

    #[test]
    fn declared_stage_is_merged_between_neighbours() {
        let v = BindingTimeVocab::from_toml_str(
            "[bindingTimes.deploy]\ndescription = \"install\"\nbinder = \"installer\"\nafter = \"compile\"\nbefore = \"load\"\n",
        );
        assert_eq!(names(&v), ["compile", "deploy", "load", "runtime"]);
        assert!(v.defects().is_empty());
        assert_eq!(v.stages()[1].binder.as_deref(), Some("installer"));
    }

    #[test]
    fn single_neighbour_places_adjacent_and_chains_resolve() {
        let v = BindingTimeVocab::from_toml_str(
            "[bindingTimes.design]\nbefore = \"compile\"\n[bindingTimes.link]\nafter = \"compile\"\n[bindingTimes.startup]\nafter = \"load\"\n[bindingTimes.boot]\nafter = \"startup\"\n",
        );
        assert_eq!(names(&v), ["design", "compile", "link", "load", "startup", "boot", "runtime"]);
        assert!(v.defects().is_empty(), "{:?}", v.defects());
    }

    #[test]
    fn stages_sharing_a_gap_are_both_rejected_unless_ordered() {
        // Same `after`, and `after`/`before` naming the same gap from two sides.
        let v = BindingTimeVocab::from_toml_str(
            "[bindingTimes.a]\nafter = \"compile\"\n[bindingTimes.b]\nafter = \"compile\"\n\
             [bindingTimes.c]\nbefore = \"runtime\"\n[bindingTimes.d]\nafter = \"load\"\n",
        );
        assert_eq!(names(&v), ["compile", "load", "runtime"], "{:?}", v.defects());
        let d = v.defects().join("\n");
        assert!(d.contains("[bindingTimes.a]") && d.contains("same position as 'b'") && d.contains("between 'compile' and 'load'"), "{d}");
        assert!(d.contains("[bindingTimes.b]") && d.contains("same position as 'a'"), "{d}");
        assert!(d.contains("[bindingTimes.c]") && d.contains("[bindingTimes.d]") && d.contains("between 'load' and 'runtime'"), "{d}");
        assert_eq!(v.defects().len(), 4, "{d}");

        // Naming the other stage resolves it, in either direction.
        let v = BindingTimeVocab::from_toml_str(
            "[bindingTimes.a]\nafter = \"compile\"\n[bindingTimes.b]\nafter = \"a\"\n",
        );
        assert_eq!(names(&v), ["compile", "a", "b", "load", "runtime"]);
        assert!(v.defects().is_empty(), "{:?}", v.defects());
        let v = BindingTimeVocab::from_toml_str(
            "[bindingTimes.a]\nafter = \"compile\"\n[bindingTimes.b]\nafter = \"compile\"\nbefore = \"a\"\n",
        );
        assert_eq!(names(&v), ["compile", "b", "a", "load", "runtime"]);

        // Distinct gaps in one pass still insert correctly.
        let v = BindingTimeVocab::from_toml_str(
            "[bindingTimes.x]\nbefore = \"compile\"\n[bindingTimes.y]\nafter = \"compile\"\n[bindingTimes.z]\nafter = \"load\"\n",
        );
        assert_eq!(names(&v), ["x", "compile", "y", "load", "z", "runtime"]);
        assert!(v.defects().is_empty());
    }

    #[test]
    fn unplaceable_cyclic_unknown_and_self_entries_are_ignored_whole() {
        let v = BindingTimeVocab::from_toml_str(
            "[bindingTimes.loose]\ndescription = \"x\"\n\
             [bindingTimes.a]\nafter = \"b\"\n[bindingTimes.b]\nafter = \"a\"\n\
             [bindingTimes.ghost]\nafter = \"nowhere\"\n\
             [bindingTimes.me]\nafter = \"me\"\n\
             [bindingTimes.flip]\nafter = \"load\"\nbefore = \"compile\"\n",
        );
        assert_eq!(names(&v), ["compile", "load", "runtime"], "defaults unchanged");
        let d = v.defects().join("\n");
        assert!(d.contains("loose") && d.contains("neither"), "{d}");
        assert!(d.contains("[bindingTimes.a]") && d.contains("cyclic"), "{d}");
        assert!(d.contains("ghost") && d.contains("not a declared stage"), "{d}");
        assert!(d.contains("me") && d.contains("itself"), "{d}");
        assert!(d.contains("flip") && d.contains("not earlier"), "{d}");
        assert_eq!(v.defects().len(), 6, "{d}");
        assert!(v.is_declared());
    }

    #[test]
    fn unknown_key_wrong_type_default_redeclaration_and_bad_name_are_ignored() {
        let v = BindingTimeVocab::from_toml_str(
            "[bindingTimes.x]\nafter = \"compile\"\nmystery = \"y\"\n\
             [bindingTimes.y]\nafter = 3\n\
             [bindingTimes.load]\nafter = \"compile\"\n\
             [\"bindingTimes\".\"bad-name\"]\nafter = \"compile\"\n",
        );
        assert_eq!(names(&v), ["compile", "load", "runtime"]);
        assert_eq!(v.defects().len(), 4, "{:?}", v.defects());
    }

    #[test]
    fn entry_depending_on_an_ignored_entry_is_ignored_too() {
        let v = BindingTimeVocab::from_toml_str(
            "[bindingTimes.bad]\nmystery = \"y\"\n[bindingTimes.child]\nafter = \"bad\"\n",
        );
        assert_eq!(names(&v), ["compile", "load", "runtime"]);
        assert!(v.defects().iter().any(|d| d.contains("child") && d.contains("itself ignored")), "{:?}", v.defects());
    }

    #[test]
    fn binding_time_orders_by_vocabulary_position() {
        let v = BindingTimeVocab::from_toml_str("[bindingTimes.deploy]\nafter = \"compile\"\n");
        let c = BindingTime::parse_in(&v, "compile").unwrap();
        let d = BindingTime::parse_in(&v, "deploy").unwrap();
        let l = BindingTime::parse_in(&v, "load").unwrap();
        assert!(c < d && d < l);
        assert_eq!(BindingTime::parse_in(&v, "bogus"), None);
    }

    #[test]
    fn json_and_agent_section() {
        let d = BindingTimeVocab::default();
        assert!(d.agent_instructions_section().is_none());
        let j = d.to_json();
        assert_eq!(j["bindingTimes"][0]["name"], "compile");
        assert_eq!(j["bindingTimes"][2]["builtin"], true);
        let v = BindingTimeVocab::from_toml_str("[bindingTimes.deploy]\nafter = \"compile\"\nbinder = \"installer\"\n");
        let s = v.agent_instructions_section().unwrap();
        assert!(s.contains("## Project binding times") && s.contains("`deploy`") && s.contains("installer"));
        assert_eq!(v.to_json()["bindingTimes"][1]["builtin"], false);
    }

    #[test]
    fn windows_read_validate_and_compare() {
        let v = BindingTimeVocab::default();
        let r = |e, l| BindingWindow::read_in(&v, e, l);
        assert_eq!(r(None, None), (None, vec![]));
        let (w, i) = r(Some("load"), None);
        let w = w.unwrap();
        assert!(i.is_empty() && w.earliest == w.latest && w.earliest.rank() == 1);
        let (w, i) = r(Some("compile"), Some("load"));
        let w = w.unwrap();
        assert!(i.is_empty() && w.contains(BindingTime(0)) && w.contains(BindingTime(1)) && !w.contains(BindingTime(2)));
        // latest alone starts at the first stage
        let (w, _) = r(None, Some("load"));
        assert_eq!(w.unwrap().earliest.rank(), 0);
        // inverted: reported, no window
        let (w, i) = r(Some("load"), Some("compile"));
        assert!(w.is_none());
        assert_eq!(i, vec![WindowIssue::Inverted { earliest: "load".into(), latest: "compile".into() }]);
        // unknown stage: reported, dropped, window from what is left
        let (w, i) = r(Some("bogus"), Some("load"));
        assert_eq!(i, vec![WindowIssue::UnknownStage { field: "bindingTime", value: "bogus".into() }]);
        assert_eq!(w.unwrap().earliest.rank(), 0);
        let (w, i) = r(Some("compile"), Some("nope"));
        assert_eq!(i.len(), 1);
        assert_eq!(w.unwrap().latest.rank(), 0);
        // ordering predicate
        let early = BindingWindow { earliest: BindingTime(0), latest: BindingTime(0) };
        let late = BindingWindow { earliest: BindingTime(1), latest: BindingTime(2) };
        assert!(early.closes_before_opens(late) && !late.closes_before_opens(early));
        let span = BindingWindow { earliest: BindingTime(0), latest: BindingTime(1) };
        assert!(!span.closes_before_opens(late), "overlap is feasible");
    }
}
