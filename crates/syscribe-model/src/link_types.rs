//! User-defined link types (ADR-SYS-LINKTYPE-001; REQ-TRS-LINKTYPE-000..012).
//!
//! A project declares its own relationship vocabulary in `[linkTypes.<name>]`
//! tables of `<model_root>/.syscribe.toml` and authors instances under one
//! namespaced frontmatter field, `links:` — a map from a declared link-type name
//! to a reference or list of references, resolved exactly like `satisfies:`.
//!
//! This module owns:
//!
//! * the **declaration** side — [`LinkTypeRegistry`] parses and checks the
//!   `[linkTypes]` table (structural defects become `W630` messages and the entry
//!   is ignored as a whole; an unknown key is `W630` but the entry stays usable);
//! * the **instance** side — [`parse_links`] reads an element's raw `links:` value
//!   (shape errors are the validator's `E631`), [`declared_links`] yields the
//!   well-formed, declared entries;
//! * the **extends/relax** mechanism — [`effective_view`] presents every instance
//!   of a type that `extends` a built-in trace link to the validator *as if* it
//!   were authored in the base field, on a private clone of the element list that
//!   lives only inside a validation run. The clone is never written anywhere:
//!   every mutating command (`set`, MCP `update_element`/`apply_changes`, `mv`,
//!   `suspect accept`, …) works from the walker's untouched elements or the file
//!   text, so an extending link can never be rewritten into its base field
//!   (REQ-TRS-LINKTYPE-006). [`Provenance`] remembers which base-field entries
//!   were contributed by which link type so a rule can ask, per
//!   `(source, entry, code)`, whether the code is relaxed or the entry is withheld
//!   from the reverse index (`coverage = false`);
//! * **traversal** — [`follow`] walks one named link (custom type, its inverse, or
//!   a built-in link/reverse-index name) one hop or transitively
//!   (REQ-TRS-LINKTYPE-007), and [`link_types_json`] renders the vocabulary
//!   (REQ-TRS-LINKTYPE-008);
//! * two small process-wide registries — an append-only string interner so a
//!   custom link can travel through `Copy` types such as
//!   [`crate::graph::EdgeKind::Custom`] and `&'static str` link-kind labels, and
//!   the *active* registry installed by [`crate::config::ValidateConfig::with_model_root`]
//!   (mirroring how `[ids.prefixes]` is installed into the resolver) for
//!   consumers that do not carry a config (graph building, suspect scanning).

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::sync::{Arc, OnceLock, RwLock};

use crate::element::{ElementType, RawElement, RawFrontmatter};
use crate::resolver::Resolver;

// ── Names ─────────────────────────────────────────────────────────────────────

static NAME_RE: OnceLock<regex::Regex> = OnceLock::new();

/// True when `s` is a well-formed link-type or inverse name: lowerCamel,
/// `^[a-z][A-Za-z0-9]*$` (REQ-TRS-LINKTYPE-001).
pub fn is_valid_link_type_name(s: &str) -> bool {
    NAME_RE
        .get_or_init(|| regex::Regex::new(r"^[a-z][A-Za-z0-9]*$").unwrap())
        .is_match(s)
}

/// Built-in link, reverse-index and traversal names a declared link type **or**
/// its `inverse` may not reuse (REQ-TRS-LINKTYPE-001): the generic structural and
/// trace edge kinds of [`crate::graph::EdgeKind`], the reverse indices surfaced by
/// `impact`/`follow`/`show`, and the `links` field itself.
pub const RESERVED_LINK_NAMES: &[&str] = &[
    // forward links / structural edges
    "contains",
    "supertype",
    "typedBy",
    "subsets",
    "redefines",
    "verifies",
    "derivedFrom",
    "satisfies",
    "refines",
    "allocatedTo",
    "allocatedFrom",
    "conditionalOn",
    "appliesWhen",
    "planningParent",
    "planningBlockedBy",
    "connection",
    "flow",
    "binding",
    "succession",
    "featureTyped",
    // reverse indices
    "satisfiedBy",
    "verifiedBy",
    "derivedChildren",
    "refinedBy",
    "specializedBy",
    "safetyGoalChildren",
    // the instance field
    "links",
];

/// Whether `name` collides with a built-in name for a link-type **name**. A type
/// name also becomes a graph edge-kind name (`impact`/`connectivity --kinds`), so
/// it must additionally avoid every domain-specific [`crate::graph::EdgeKind`]
/// name (`mitigatedBy`, `topEvent`, …). An `inverse` is only ever a traversal /
/// display label, so it is checked against [`RESERVED_LINK_NAMES`] alone.
///
/// Compared **case-insensitively**: `connectivity --kinds` matches kind names
/// ignoring case, so a type named `mitigatedby` would otherwise be accepted yet
/// shadowed by the built-in `mitigatedBy` edge kind.
fn collides_with_builtin_type_name(name: &str) -> bool {
    is_reserved_name(name) || crate::graph::EdgeKind::BUILTIN.iter().any(|k| k.name().eq_ignore_ascii_case(name))
}

/// Whether `name` equals a [`RESERVED_LINK_NAMES`] entry, ignoring case.
fn is_reserved_name(name: &str) -> bool {
    RESERVED_LINK_NAMES.iter().any(|r| r.eq_ignore_ascii_case(name))
}

// ── Built-in bases a link type may extend ─────────────────────────────────────

/// A built-in trace link a user-defined type may `extends` (REQ-TRS-LINKTYPE-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BaseLink {
    Satisfies,
    Verifies,
    DerivedFrom,
    Refines,
}

impl BaseLink {
    pub const ALL: [BaseLink; 4] =
        [BaseLink::Satisfies, BaseLink::Verifies, BaseLink::DerivedFrom, BaseLink::Refines];

    pub fn parse(s: &str) -> Option<Self> {
        BaseLink::ALL.into_iter().find(|b| b.name() == s)
    }

    /// The base field's frontmatter name.
    pub fn name(self) -> &'static str {
        match self {
            BaseLink::Satisfies => "satisfies",
            BaseLink::Verifies => "verifies",
            BaseLink::DerivedFrom => "derivedFrom",
            BaseLink::Refines => "refines",
        }
    }

    /// The link-scoped rules of this base that `relax` may name (the ADR table).
    pub fn relaxable(self) -> &'static [&'static str] {
        match self {
            BaseLink::Satisfies => &["E312", "E313"],
            BaseLink::Verifies => &["E104"],
            BaseLink::DerivedFrom => &["E105", "E310", "W303"],
            BaseLink::Refines => &["E316"],
        }
    }
}

// ── Cardinality ───────────────────────────────────────────────────────────────

/// A declared per-source target-count bound: `N`, `N..M` or `N..*`. The default
/// is `0..*` (no bound).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cardinality {
    pub min: usize,
    /// `None` = unbounded (`*`).
    pub max: Option<usize>,
}

impl Cardinality {
    /// Parse `N`, `N..M` (with `M >= N`) or `N..*`. `None` on anything else.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let num = |t: &str| -> Option<usize> {
            let t = t.trim();
            if t.is_empty() || !t.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            t.parse().ok()
        };
        match s.split_once("..") {
            Some((a, b)) => {
                let min = num(a)?;
                if b.trim() == "*" {
                    return Some(Cardinality { min, max: None });
                }
                let max = num(b)?;
                (max >= min).then_some(Cardinality { min, max: Some(max) })
            }
            None => num(s).map(|n| Cardinality { min: n, max: Some(n) }),
        }
    }
}

impl std::fmt::Display for Cardinality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.max {
            None => write!(f, "{}..*", self.min),
            Some(m) if m == self.min => write!(f, "{}", m),
            Some(m) => write!(f, "{}..{}", self.min, m),
        }
    }
}

// ── Declaration ───────────────────────────────────────────────────────────────

/// One valid `[linkTypes.<name>]` declaration (REQ-TRS-LINKTYPE-001).
#[derive(Debug, Clone, PartialEq)]
pub struct LinkTypeDecl {
    pub name: String,
    pub description: Option<String>,
    pub inverse: Option<String>,
    /// `None` = any source type.
    pub source_types: Option<Vec<String>>,
    /// `None` = any target type.
    pub target_types: Option<Vec<String>>,
    pub cardinality: Cardinality,
    pub acyclic: bool,
    pub suspect: bool,
    pub extends: Option<BaseLink>,
    pub relax: Vec<String>,
    pub coverage: bool,
}

impl LinkTypeDecl {
    /// Label for an inbound instance: the declared `inverse`, else `<name> (inbound)`
    /// (REQ-TRS-LINKTYPE-009).
    pub fn inbound_label(&self) -> String {
        self.inverse.clone().unwrap_or_else(|| format!("{} (inbound)", self.name))
    }

    /// Whether this type relaxes `code` for its instances (REQ-TRS-LINKTYPE-006).
    pub fn relaxes(&self, code: &str) -> bool {
        self.relax.iter().any(|c| c == code)
    }

    /// Whether an element of type `et` may hold an instance (`E633`).
    pub fn permits_source(&self, et: Option<&ElementType>) -> bool {
        permits(&self.source_types, et)
    }

    /// Whether an element of type `et` may be a target (`E634`).
    pub fn permits_target(&self, et: Option<&ElementType>) -> bool {
        permits(&self.target_types, et)
    }

    /// Whether an element of type `et` is in scope for the lower-bound check
    /// (`W631`): only when `sourceTypes` is declared and lists the type.
    pub fn requires_links_on(&self, et: Option<&ElementType>) -> bool {
        self.cardinality.min > 0 && self.source_types.is_some() && permits(&self.source_types, et)
    }
}

fn permits(list: &Option<Vec<String>>, et: Option<&ElementType>) -> bool {
    match list {
        None => true,
        Some(types) => {
            types.contains(&element_type_name(et))
        }
    }
}

/// The `type:` name of an element type, as authored (`PartDef`, `Requirement`, …).
pub fn element_type_name(et: Option<&ElementType>) -> String {
    match et {
        Some(t) => format!("{:?}", t),
        None => "(untyped)".to_string(),
    }
}

/// True when `name` is a known element-type name (a `type:` value the format
/// recognises). Uses `ElementType`'s own deserializer so the list can never drift.
pub fn is_known_element_type(name: &str) -> bool {
    match serde_yaml::from_value::<ElementType>(serde_yaml::Value::String(name.to_string())) {
        Ok(ElementType::Unknown) | Err(_) => false,
        Ok(_) => true,
    }
}

/// The declared link-type vocabulary of a model (REQ-TRS-LINKTYPE-001): the valid
/// entries (sorted by name) plus the `W630` defect messages for the rest.
#[derive(Debug, Clone, Default)]
pub struct LinkTypeRegistry {
    types: Vec<LinkTypeDecl>,
    defects: Vec<String>,
    /// A `[linkTypes]` table was present (even if every entry in it was invalid).
    declared: bool,
}

/// Keys accepted on a `[linkTypes.<name>]` entry, in their canonical camelCase form.
const KNOWN_KEYS: &[&str] = &[
    "description",
    "inverse",
    "sourceTypes",
    "targetTypes",
    "cardinality",
    "acyclic",
    "suspect",
    "extends",
    "relax",
    "coverage",
];

/// Canonicalise an entry key: snake_case → camelCase (`source_types` → `sourceTypes`).
fn canonical_key(k: &str) -> String {
    let mut out = String::with_capacity(k.len());
    let mut upper = false;
    for c in k.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// Raw per-entry view collected in the first pass, before cross-entry checks.
struct RawEntry {
    decl: LinkTypeDecl,
    defects: Vec<String>,
}

impl LinkTypeRegistry {
    /// Load `[linkTypes]` from `<model_root>/.syscribe.toml`. Empty (not declared)
    /// when the file is absent, unparseable, or has no such table — the feature is
    /// dormant, same posture as every other config-gated table.
    pub fn load(model_root: &Path) -> Self {
        std::fs::read_to_string(model_root.join(".syscribe.toml"))
            .map(|t| Self::from_toml_str(&t))
            .unwrap_or_default()
    }

    /// Parse the `[linkTypes]` table out of a whole `.syscribe.toml` text.
    pub fn from_toml_str(text: &str) -> Self {
        let Ok(root) = toml::from_str::<toml::Table>(text) else {
            return Self::default();
        };
        match root.get("linkTypes").or_else(|| root.get("link_types")) {
            Some(toml::Value::Table(t)) => Self::from_table(t),
            Some(_) => Self {
                types: Vec::new(),
                defects: vec!["[linkTypes] must be a table of `[linkTypes.<name>]` entries — ignored".to_string()],
                declared: true,
            },
            None => Self::default(),
        }
    }

    /// Parse and check a `[linkTypes]` table.
    pub fn from_table(table: &toml::Table) -> Self {
        let mut defects: Vec<String> = Vec::new();
        let mut entries: Vec<RawEntry> = Vec::new();

        let mut names: Vec<&String> = table.keys().collect();
        names.sort();
        for name in names {
            let value = &table[name.as_str()];
            let mut entry = RawEntry {
                decl: LinkTypeDecl {
                    name: name.clone(),
                    description: None,
                    inverse: None,
                    source_types: None,
                    target_types: None,
                    cardinality: Cardinality::default(),
                    acyclic: false,
                    suspect: true,
                    extends: None,
                    relax: Vec::new(),
                    coverage: true,
                },
                defects: Vec::new(),
            };
            let toml::Value::Table(t) = value else {
                entry.defects.push("must be a table of keys".to_string());
                entries.push(entry);
                continue;
            };
            parse_entry(name, t, &mut entry, &mut defects);
            entries.push(entry);
        }

        // Cross-entry checks: an inverse must not reuse another declared type's name
        // or another type's inverse. Checked against every declared name — including
        // entries that are themselves invalid — so the verdict does not depend on
        // which of two colliding entries happens to be broken.
        // Names are compared case-insensitively (keys lowercased), for the same
        // reason as the built-in check: `--kinds` matching ignores case.
        let all_names: HashMap<String, String> =
            entries.iter().map(|e| (e.decl.name.to_ascii_lowercase(), e.decl.name.clone())).collect();
        let mut inverse_owners: HashMap<String, Vec<String>> = HashMap::new();
        for e in &entries {
            if let Some(inv) = &e.decl.inverse {
                inverse_owners.entry(inv.to_ascii_lowercase()).or_default().push(e.decl.name.clone());
            }
        }
        // Two declared type names differing only in case.
        let mut by_lower: HashMap<String, Vec<String>> = HashMap::new();
        for e in &entries {
            by_lower.entry(e.decl.name.to_ascii_lowercase()).or_default().push(e.decl.name.clone());
        }
        for e in &mut entries {
            if let Some(same) = by_lower.get(&e.decl.name.to_ascii_lowercase()) {
                let others: Vec<&str> = same.iter().filter(|o| **o != e.decl.name).map(|s| s.as_str()).collect();
                if !others.is_empty() {
                    e.defects.push(format!(
                        "name '{}' differs only in case from declared link type(s) {}",
                        e.decl.name,
                        others.join(", ")
                    ));
                }
            }
            let Some(inv) = e.decl.inverse.clone() else { continue };
            if inv.eq_ignore_ascii_case(&e.decl.name) {
                e.defects.push(format!("inverse '{inv}' is the same as the type name"));
            } else if let Some(other) = all_names.get(&inv.to_ascii_lowercase()) {
                e.defects.push(format!("inverse '{inv}' collides with the declared link type '{other}'"));
            }
            if let Some(owners) = inverse_owners.get(&inv.to_ascii_lowercase()) {
                let others: Vec<&String> = owners.iter().filter(|o| **o != e.decl.name).collect();
                if !others.is_empty() {
                    let list: Vec<&str> = others.iter().map(|s| s.as_str()).collect();
                    e.defects.push(format!(
                        "inverse '{inv}' is also the inverse of link type(s) {}",
                        list.join(", ")
                    ));
                }
            }
        }

        let mut types = Vec::new();
        for e in entries {
            if e.defects.is_empty() {
                types.push(e.decl);
            } else {
                for d in e.defects {
                    defects.push(format!("[linkTypes.{}] {} — entry ignored", e.decl.name, d));
                }
            }
        }
        types.sort_by(|a, b| a.name.cmp(&b.name));
        Self { types, defects, declared: true }
    }

    /// Every valid declaration, sorted by name.
    pub fn types(&self) -> &[LinkTypeDecl] {
        &self.types
    }

    /// The `W630` messages (structural defects and unknown keys).
    pub fn defects(&self) -> &[String] {
        &self.defects
    }

    /// No valid link type is declared.
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// A `[linkTypes]` table is present (possibly with only invalid entries).
    pub fn is_declared(&self) -> bool {
        self.declared
    }

    pub fn get(&self, name: &str) -> Option<&LinkTypeDecl> {
        self.types.iter().find(|t| t.name == name)
    }

    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.types.iter().position(|t| t.name == name)
    }

    /// The type whose declared `inverse` is `name`.
    pub fn by_inverse(&self, name: &str) -> Option<&LinkTypeDecl> {
        self.types.iter().find(|t| t.inverse.as_deref() == Some(name))
    }

    /// True when any valid type `extends` a built-in base.
    pub fn has_extensions(&self) -> bool {
        self.types.iter().any(|t| t.extends.is_some())
    }

    /// Human list of the declared types for `E630` (REQ-TRS-LINKTYPE-002/012).
    pub fn declared_names_hint(&self) -> String {
        if self.types.is_empty() {
            if self.declared {
                "no valid link types are declared (see W630 on .syscribe.toml) — declare one with a `[linkTypes.<name>]` table in .syscribe.toml".to_string()
            } else {
                "no link types are declared — add a `[linkTypes.<name>]` table to .syscribe.toml to declare one".to_string()
            }
        } else {
            let names: Vec<&str> = self.types.iter().map(|t| t.name.as_str()).collect();
            format!(
                "declared link types: {} (declare new ones with a `[linkTypes.<name>]` table in .syscribe.toml)",
                names.join(", ")
            )
        }
    }
}

/// Parse one `[linkTypes.<name>]` table's keys into `entry`; defects go on the
/// entry (ignoring it), unknown-key warnings straight to `warnings` (entry kept).
fn parse_entry(name: &str, t: &toml::Table, entry: &mut RawEntry, warnings: &mut Vec<String>) {
    if !is_valid_link_type_name(name) {
        entry.defects.push(format!(
            "name '{name}' is not a valid link-type name (expected lowerCamel `^[a-z][A-Za-z0-9]*$`)"
        ));
    } else if collides_with_builtin_type_name(name) {
        entry.defects.push(format!("name '{name}' collides with a built-in link/field name"));
    }

    let mut seen: HashSet<String> = HashSet::new();
    let mut relax_given = false;
    let mut coverage_given = false;
    let mut keys: Vec<&String> = t.keys().collect();
    keys.sort();
    for raw_key in keys {
        let v = &t[raw_key.as_str()];
        let key = canonical_key(raw_key);
        if !KNOWN_KEYS.contains(&key.as_str()) {
            warnings.push(format!("[linkTypes.{name}] unknown key '{raw_key}' — key ignored"));
            continue;
        }
        if !seen.insert(key.clone()) {
            entry.defects.push(format!("key '{key}' is given more than once (camelCase and snake_case)"));
            continue;
        }
        let d = &mut entry.decl;
        match key.as_str() {
            "description" => match v.as_str() {
                Some(s) => d.description = Some(s.to_string()),
                None => entry.defects.push("`description` must be a string".to_string()),
            },
            "inverse" => match v.as_str() {
                Some(s) if !is_valid_link_type_name(s) => entry.defects.push(format!(
                    "inverse '{s}' is not a valid link-type name (expected lowerCamel `^[a-z][A-Za-z0-9]*$`)"
                )),
                Some(s) if is_reserved_name(s) => entry
                    .defects
                    .push(format!("inverse '{s}' collides with a built-in link/reverse-index name")),
                Some(s) => d.inverse = Some(s.to_string()),
                None => entry.defects.push("`inverse` must be a string".to_string()),
            },
            "sourceTypes" | "targetTypes" => {
                let list = string_list(v);
                match list {
                    None => entry.defects.push(format!("`{key}` must be a list of element-type names")),
                    Some(list) => {
                        for ty in &list {
                            if !is_known_element_type(ty) {
                                entry.defects.push(format!("`{key}` names unknown element type '{ty}'"));
                            }
                        }
                        if key == "sourceTypes" {
                            d.source_types = Some(list);
                        } else {
                            d.target_types = Some(list);
                        }
                    }
                }
            }
            "cardinality" => {
                let text = match v {
                    toml::Value::String(s) => Some(s.clone()),
                    toml::Value::Integer(i) => Some(i.to_string()),
                    _ => None,
                };
                match text.as_deref().and_then(Cardinality::parse) {
                    Some(c) => d.cardinality = c,
                    None => entry.defects.push(format!(
                        "cardinality '{}' is not parseable (expected `N`, `N..M` or `N..*`)",
                        text.unwrap_or_else(|| v.to_string())
                    )),
                }
            }
            "acyclic" | "suspect" | "coverage" => match v.as_bool() {
                Some(b) => match key.as_str() {
                    "acyclic" => d.acyclic = b,
                    "suspect" => d.suspect = b,
                    _ => {
                        d.coverage = b;
                        coverage_given = true;
                    }
                },
                None => entry.defects.push(format!("`{key}` must be a boolean")),
            },
            "extends" => match v.as_str() {
                Some(s) => match BaseLink::parse(s) {
                    Some(b) => d.extends = Some(b),
                    None => entry.defects.push(format!(
                        "extends '{s}' is not a supported base (satisfies, verifies, derivedFrom, refines)"
                    )),
                },
                None => entry.defects.push("`extends` must be a string".to_string()),
            },
            "relax" => match string_list(v) {
                Some(list) => {
                    d.relax = list;
                    relax_given = true;
                }
                None => entry.defects.push("`relax` must be a list of finding codes".to_string()),
            },
            _ => {}
        }
    }

    let d = &entry.decl;
    if d.cardinality.min > 0 && d.source_types.is_none() {
        entry.defects.push(format!(
            "cardinality '{}' has a non-zero lower bound but no `sourceTypes` to scope it",
            d.cardinality
        ));
    }
    let has_extends_key = t.keys().any(|k| canonical_key(k) == "extends");
    match d.extends {
        None if !has_extends_key => {
            if relax_given {
                entry.defects.push("`relax` requires `extends`".to_string());
            }
            if coverage_given {
                entry.defects.push("`coverage` requires `extends`".to_string());
            }
        }
        None => {}
        Some(base) => {
            let bad: Vec<&String> = d.relax.iter().filter(|c| !base.relaxable().contains(&c.as_str())).collect();
            if !bad.is_empty() {
                let bad: Vec<&str> = bad.iter().map(|s| s.as_str()).collect();
                entry.defects.push(format!(
                    "relax code(s) {} are not relaxable for base '{}' (relaxable: {})",
                    bad.join(", "),
                    base.name(),
                    base.relaxable().join(", ")
                ));
            }
        }
    }
}

/// A TOML value as a list of strings (`["A", "B"]`); a bare string is accepted
/// as a one-element list. `None` when any member is not a string.
fn string_list(v: &toml::Value) -> Option<Vec<String>> {
    match v {
        toml::Value::String(s) => Some(vec![s.clone()]),
        toml::Value::Array(a) => a.iter().map(|x| x.as_str().map(str::to_string)).collect(),
        _ => None,
    }
}

// ── Process-wide registries ───────────────────────────────────────────────────

static INTERNED: OnceLock<RwLock<Vec<&'static str>>> = OnceLock::new();

/// Intern `s`, returning a stable small index. Append-only and idempotent: a
/// given string is leaked at most once per process, so the set is bounded by the
/// number of distinct link-type names/labels ever loaded.
pub fn intern(s: &str) -> u16 {
    let lock = INTERNED.get_or_init(|| RwLock::new(Vec::new()));
    if let Ok(r) = lock.read() {
        if let Some(i) = r.iter().position(|x| *x == s) {
            return i as u16;
        }
    }
    let Ok(mut w) = lock.write() else { return u16::MAX };
    if let Some(i) = w.iter().position(|x| *x == s) {
        return i as u16;
    }
    if w.len() >= u16::MAX as usize {
        return u16::MAX;
    }
    w.push(Box::leak(s.to_string().into_boxed_str()));
    (w.len() - 1) as u16
}

/// The string interned at `i` (`"?"` for an unknown index).
pub fn interned(i: u16) -> &'static str {
    INTERNED
        .get()
        .and_then(|l| l.read().ok().and_then(|r| r.get(i as usize).copied()))
        .unwrap_or("?")
}

/// `s` as a `&'static str`, via the interner.
pub fn static_str(s: &str) -> &'static str {
    interned(intern(s))
}

static ACTIVE: OnceLock<RwLock<Arc<LinkTypeRegistry>>> = OnceLock::new();

/// Install the active link-type registry (the loaded model's vocabulary), used by
/// consumers that carry no [`crate::config::ValidateConfig`]: graph building
/// (`EdgeKind::Custom`), suspect-link scanning, `impact`. Installed by
/// `ValidateConfig::with_model_root`, mirroring `[ids.prefixes]`.
pub fn install(reg: &LinkTypeRegistry) {
    let lock = ACTIVE.get_or_init(|| RwLock::new(Arc::new(LinkTypeRegistry::default())));
    if let Ok(mut w) = lock.write() {
        *w = Arc::new(reg.clone());
    }
}

/// The active link-type registry (empty until one is installed).
pub fn active() -> Arc<LinkTypeRegistry> {
    ACTIVE
        .get()
        .and_then(|l| l.read().ok().map(|r| r.clone()))
        .unwrap_or_default()
}

// ── Instances (`links:`) ──────────────────────────────────────────────────────

/// One key of an element's `links:` map.
#[derive(Debug, Clone)]
pub struct LinkEntry {
    /// The link-type name as authored (rendered, when the YAML key was not a string).
    pub key: String,
    /// `false` when the YAML key was not a string.
    pub key_ok: bool,
    /// The authored targets, or `None` when the value is not a string or a list of
    /// strings (`E631`).
    pub targets: Option<Vec<String>>,
}

/// The parsed shape of an element's `links:` field.
#[derive(Debug, Clone)]
pub enum LinksField {
    Absent,
    /// `links:` is present but not a mapping (`E631`).
    NotAMapping,
    Entries(Vec<LinkEntry>),
}

/// Read an element's raw `links:` value (REQ-TRS-LINKTYPE-002).
pub fn parse_links(fm: &RawFrontmatter) -> LinksField {
    match &fm.links {
        None | Some(serde_yaml::Value::Null) => LinksField::Absent,
        Some(serde_yaml::Value::Mapping(m)) => {
            let mut out = Vec::new();
            for (k, v) in m {
                let (key, key_ok) = match k.as_str() {
                    Some(s) => (s.to_string(), true),
                    None => (
                        serde_yaml::to_string(k).unwrap_or_default().trim().to_string(),
                        false,
                    ),
                };
                let targets = match v {
                    serde_yaml::Value::Null => Some(Vec::new()),
                    serde_yaml::Value::String(s) => Some(vec![s.clone()]),
                    serde_yaml::Value::Sequence(seq) => {
                        seq.iter().map(|x| x.as_str().map(str::to_string)).collect()
                    }
                    _ => None,
                };
                out.push(LinkEntry { key, key_ok, targets });
            }
            LinksField::Entries(out)
        }
        Some(_) => LinksField::NotAMapping,
    }
}

/// The well-formed entries of `fm.links` whose key is a valid declared type, as
/// `(registry index, targets)` in authored order.
pub fn declared_links(fm: &RawFrontmatter, reg: &LinkTypeRegistry) -> Vec<(usize, Vec<String>)> {
    let LinksField::Entries(entries) = parse_links(fm) else { return Vec::new() };
    entries
        .into_iter()
        .filter(|e| e.key_ok)
        .filter_map(|e| Some((reg.index_of(&e.key)?, e.targets?)))
        .collect()
}

/// Number of authored instances of each declared type across the model.
pub fn instance_counts(elements: &[RawElement], reg: &LinkTypeRegistry) -> HashMap<String, usize> {
    let mut out: HashMap<String, usize> = HashMap::new();
    for e in elements {
        for (i, targets) in declared_links(&e.frontmatter, reg) {
            *out.entry(reg.types()[i].name.clone()).or_default() += targets.len();
        }
    }
    out
}

/// One resolved custom-link instance.
#[derive(Debug, Clone)]
pub struct LinkEdge {
    /// Registry index of the link type.
    pub type_idx: usize,
    /// Element index (into the slice passed in) of the source.
    pub source: usize,
    /// Element index of the resolved target.
    pub target: usize,
    /// The target reference exactly as authored.
    pub target_ref: String,
}

/// Every resolvable instance of every declared type, in element order.
pub fn resolved_edges(elements: &[RawElement], resolver: &Resolver, reg: &LinkTypeRegistry) -> Vec<LinkEdge> {
    let mut out = Vec::new();
    if reg.is_empty() {
        return out;
    }
    for (si, e) in elements.iter().enumerate() {
        for (ti, targets) in declared_links(&e.frontmatter, reg) {
            for t in targets {
                if let Some(tgt) = resolver.resolve_ref(elements, &t) {
                    if let Some(&tix) = resolver.by_qname.get(&tgt.qualified_name) {
                        out.push(LinkEdge { type_idx: ti, source: si, target: tix, target_ref: t });
                    }
                }
            }
        }
    }
    out
}

/// A stable display label: the stable id when present, else the qualified name.
pub fn label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

// ── extends / relax: the validation-time effective view ───────────────────────

/// Which base-field entries of a source were contributed by which link type.
/// Keyed by `(source qname, base)` → `(authored length of the base field, registry
/// index of the type behind each appended entry)`; an entry at index `i >= len`
/// is the `(i - len)`-th contributed one.
#[derive(Debug, Clone, Default)]
pub struct Provenance {
    entries: HashMap<(String, BaseLink), (usize, Vec<usize>)>,
    types: Vec<LinkTypeDecl>,
}

impl Provenance {
    /// The link type that contributed entry `index` of `qname`'s `base` field, or
    /// `None` for an authored (built-in) entry.
    pub fn decl(&self, qname: &str, base: BaseLink, index: usize) -> Option<&LinkTypeDecl> {
        let (len, tys) = self.entries.get(&(qname.to_string(), base))?;
        let k = index.checked_sub(*len)?;
        tys.get(k).and_then(|&i| self.types.get(i))
    }

    /// Whether `code` is relaxed for entry `index` (never for a built-in entry).
    pub fn relaxed(&self, qname: &str, base: BaseLink, index: usize, code: &str) -> bool {
        self.decl(qname, base, index).is_some_and(|d| d.relaxes(code))
    }

    /// Whether entry `index` feeds the base's reverse index (`coverage`, default true).
    pub fn in_reverse_index(&self, qname: &str, base: BaseLink, index: usize) -> bool {
        self.decl(qname, base, index).is_none_or(|d| d.coverage)
    }

    /// The link-type name that contributed entry `index` of `qname`'s `base`
    /// field, for labelling it in a report; `None` for an authored entry.
    pub fn via(&self, qname: &str, base: BaseLink, index: usize) -> Option<&str> {
        self.decl(qname, base, index).map(|d| d.name.as_str())
    }

    /// Link-type name behind a contributed `base` entry of `qname` whose target is
    /// `target` as authored, for reports that only hold the (source, target) pair.
    pub fn via_target(&self, qname: &str, base: BaseLink, field: &[String], target: &str) -> Option<&str> {
        let (len, _) = self.entries.get(&(qname.to_string(), base))?;
        field
            .iter()
            .enumerate()
            .skip(*len)
            .find(|(_, t)| t.as_str() == target)
            .and_then(|(i, _)| self.via(qname, base, i))
    }

    /// Whether entry `index` was contributed by a link type (not authored).
    pub fn is_contributed(&self, qname: &str, base: BaseLink, index: usize) -> bool {
        self.decl(qname, base, index).is_some()
    }

    /// Element-level relaxation (`E310`, `W303`): true when the element's `base`
    /// field has `len > 0` entries and **every** one comes from a type relaxing
    /// `code` — a single built-in or non-relaxing entry keeps the rule in force.
    pub fn all_relaxed(&self, qname: &str, base: BaseLink, len: usize, code: &str) -> bool {
        len > 0 && (0..len).all(|i| self.relaxed(qname, base, i, code))
    }
}

/// A private, validation-only copy of the element list in which every resolved
/// instance of a type that `extends` a built-in base has been appended to that
/// base field, plus the [`Provenance`] of the appended entries.
pub struct EffectiveView {
    pub elements: Vec<RawElement>,
    pub provenance: Provenance,
}

/// Build the effective view (REQ-TRS-LINKTYPE-006), or `None` when no declared
/// type extends a base or no element uses one — in which case validation runs on
/// the authored elements unchanged (purely additive). Only **resolved** targets
/// are contributed: an unresolved one is reported once, as `E632`, never again as
/// the base's own unresolved-reference code. The authored target string is what
/// gets appended, so base-rule messages quote it verbatim.
pub fn effective_view(elements: &[RawElement], reg: &LinkTypeRegistry) -> Option<EffectiveView> {
    build_view(elements, reg, false)
}

/// The **reporting** view of the model (REQ-TRS-LINKTYPE-006): for read-only
/// query/report commands answering "what satisfies / verifies / derives from /
/// refines X". Like [`effective_view`], but only instances of `coverage = true`
/// extending types are appended to their base field — a report's base-link list
/// means "counts as satisfying/verifying", which a `coverage = false` link does
/// not. `Borrowed` (zero cost, byte-identical output) when no extending type is
/// in use. Never feed this to the validator (it would re-append the same links as
/// if authored, defeating `relax`), to content hashing, to an export that
/// serializes authored frontmatter, or to any write path.
pub fn coverage_view<'a>(elements: &'a [RawElement], reg: &LinkTypeRegistry) -> std::borrow::Cow<'a, [RawElement]> {
    coverage_view_with_provenance(elements, reg).0
}

/// [`coverage_view`] plus the [`Provenance`] of the appended entries, so a report
/// can label a contributed entry with its link type (`A::A1 (via partSat)`, see
/// [`Provenance::via`]).
pub fn coverage_view_with_provenance<'a>(
    elements: &'a [RawElement],
    reg: &LinkTypeRegistry,
) -> (std::borrow::Cow<'a, [RawElement]>, Provenance) {
    match build_view(elements, reg, true) {
        Some(v) => (std::borrow::Cow::Owned(v.elements), v.provenance),
        None => (std::borrow::Cow::Borrowed(elements), Provenance::default()),
    }
}

/// Report labelling (REQ-TRS-LINKTYPE-006): when `source` reaches `target` along
/// `base` **only** through a `coverage = true` user-defined link extending it, the
/// name of that link type (for a `X (via partSat)` label); `None` when an authored
/// base-field entry already resolves to `target`, or no extending link does.
/// Works on the authored elements — no view needed.
pub fn via_extension<'r>(
    elements: &[RawElement],
    resolver: &Resolver,
    reg: &'r LinkTypeRegistry,
    source: &RawElement,
    base: BaseLink,
    target: &RawElement,
) -> Option<&'r str> {
    if !reg.has_extensions() {
        return None;
    }
    let hits = |r: &str| resolver.resolve_ref(elements, r).is_some_and(|t| t.qualified_name == target.qualified_name);
    if base_field(&source.frontmatter, base).is_some_and(|f| f.iter().any(|r| hits(r))) {
        return None;
    }
    declared_links(&source.frontmatter, reg).into_iter().find_map(|(ti, targets)| {
        let d = &reg.types()[ti];
        (d.extends == Some(base) && d.coverage && targets.iter().any(|t| hits(t))).then_some(d.name.as_str())
    })
}

/// The authored `base` field of `fm`.
fn base_field(fm: &RawFrontmatter, base: BaseLink) -> Option<&Vec<String>> {
    match base {
        BaseLink::Satisfies => fm.satisfies.as_ref(),
        BaseLink::Verifies => fm.verifies.as_ref(),
        BaseLink::DerivedFrom => fm.derived_from.as_ref(),
        BaseLink::Refines => fm.refines.as_ref(),
    }
}

/// Shared builder behind [`effective_view`] (every extending instance) and
/// [`coverage_view`] (`coverage_only`: only `coverage = true` ones).
fn build_view(elements: &[RawElement], reg: &LinkTypeRegistry, coverage_only: bool) -> Option<EffectiveView> {
    if !reg.has_extensions() {
        return None;
    }
    let resolver = Resolver::new(elements);
    let mut contributions: Vec<(usize, BaseLink, usize, String)> = Vec::new(); // (elem, base, type, target)
    for (ei, e) in elements.iter().enumerate() {
        for (ti, targets) in declared_links(&e.frontmatter, reg) {
            let Some(base) = reg.types()[ti].extends else { continue };
            if coverage_only && !reg.types()[ti].coverage {
                continue;
            }
            for t in targets {
                // Reporting view: a target the base field already lists (authored,
                // or appended by an earlier pass) is not listed twice — which also
                // makes the view idempotent, so nested report helpers may apply it
                // again safely. The validator's view keeps the twin: it carries
                // the per-entry relax/coverage provenance.
                if coverage_only && base_field(&e.frontmatter, base).is_some_and(|f| f.contains(&t)) {
                    continue;
                }
                if coverage_only
                    && contributions.iter().any(|(ci, cb, _, ct)| *ci == ei && *cb == base && *ct == t)
                {
                    continue;
                }
                if resolver.resolve_ref(elements, &t).is_some() {
                    contributions.push((ei, base, ti, t));
                }
            }
        }
    }
    if contributions.is_empty() {
        return None;
    }
    let mut out = elements.to_vec();
    let mut prov = Provenance { entries: HashMap::new(), types: reg.types().to_vec() };
    for (ei, base, ti, target) in contributions {
        let elem = &mut out[ei];
        let field = match base {
            BaseLink::Satisfies => &mut elem.frontmatter.satisfies,
            BaseLink::Verifies => &mut elem.frontmatter.verifies,
            BaseLink::DerivedFrom => &mut elem.frontmatter.derived_from,
            BaseLink::Refines => &mut elem.frontmatter.refines,
        };
        let authored_len = field.as_ref().map_or(0, |v| v.len());
        let slot = prov
            .entries
            .entry((elem.qualified_name.clone(), base))
            .or_insert((authored_len, Vec::new()));
        field.get_or_insert_with(Vec::new).push(target);
        slot.1.push(ti);
    }
    Some(EffectiveView { elements: out, provenance: prov })
}

// ── Traversal: `follow` (REQ-TRS-LINKTYPE-007) ────────────────────────────────

/// A built-in field `follow` can walk (the built-in field only — never the
/// custom extensions of it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuiltinField {
    Satisfies,
    Verifies,
    DerivedFrom,
    Refines,
    Supertype,
    TypedBy,
    AllocatedTo,
}

/// Built-in forward names and their reverse-index names (`typedBy` has none).
const BUILTIN_FOLLOW: &[(&str, Option<&str>, BuiltinField)] = &[
    ("satisfies", Some("satisfiedBy"), BuiltinField::Satisfies),
    ("verifies", Some("verifiedBy"), BuiltinField::Verifies),
    ("derivedFrom", Some("derivedChildren"), BuiltinField::DerivedFrom),
    ("refines", Some("refinedBy"), BuiltinField::Refines),
    ("supertype", Some("specializedBy"), BuiltinField::Supertype),
    ("typedBy", None, BuiltinField::TypedBy),
    ("allocatedTo", Some("allocatedFrom"), BuiltinField::AllocatedTo),
];

#[derive(Debug, Clone, Copy)]
enum LinkSelector {
    Custom(usize),
    Builtin(BuiltinField),
}

/// Resolve a link name to `(selector, is_reverse_name)`.
fn resolve_link_name(name: &str, reg: &LinkTypeRegistry) -> Option<(LinkSelector, bool)> {
    if let Some(i) = reg.index_of(name) {
        return Some((LinkSelector::Custom(i), false));
    }
    if let Some(i) = reg.types().iter().position(|t| t.inverse.as_deref() == Some(name)) {
        return Some((LinkSelector::Custom(i), true));
    }
    for (fwd, rev, f) in BUILTIN_FOLLOW {
        if *fwd == name {
            return Some((LinkSelector::Builtin(*f), false));
        }
        if *rev == Some(name) {
            return Some((LinkSelector::Builtin(*f), true));
        }
    }
    None
}

/// Every name `follow` accepts: declared types and inverses, then the built-ins.
pub fn available_link_names(reg: &LinkTypeRegistry) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for t in reg.types() {
        out.push(t.name.clone());
        if let Some(inv) = &t.inverse {
            out.push(inv.clone());
        }
    }
    for (fwd, rev, _) in BUILTIN_FOLLOW {
        out.push(fwd.to_string());
        if let Some(rev) = rev {
            out.push(rev.to_string());
        }
    }
    out
}

fn yaml_refs(v: &Option<serde_yaml::Value>) -> Vec<String> {
    match v {
        Some(serde_yaml::Value::String(s)) => vec![s.clone()],
        Some(serde_yaml::Value::Sequence(seq)) => seq.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
        _ => Vec::new(),
    }
}

/// Forward target element indices of `e` along `sel`.
fn forward_targets(
    sel: LinkSelector,
    e: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    reg: &LinkTypeRegistry,
) -> Vec<usize> {
    let fm = &e.frontmatter;
    let refs: Vec<String> = match sel {
        LinkSelector::Custom(ti) => declared_links(fm, reg)
            .into_iter()
            .filter(|(i, _)| *i == ti)
            .flat_map(|(_, ts)| ts)
            .collect(),
        LinkSelector::Builtin(BuiltinField::Satisfies) => fm.satisfies.clone().unwrap_or_default(),
        LinkSelector::Builtin(BuiltinField::Verifies) => fm.verifies.clone().unwrap_or_default(),
        LinkSelector::Builtin(BuiltinField::DerivedFrom) => fm.derived_from.clone().unwrap_or_default(),
        LinkSelector::Builtin(BuiltinField::Refines) => fm.refines.clone().unwrap_or_default(),
        LinkSelector::Builtin(BuiltinField::Supertype) => yaml_refs(&fm.supertype),
        LinkSelector::Builtin(BuiltinField::TypedBy) => {
            // typedBy is resolved through the enclosing-package scope chain.
            return yaml_refs(&fm.typed_by)
                .iter()
                .filter_map(|r| resolver.resolve_scoped_ref(elements, &e.qualified_name, r))
                .filter_map(|t| resolver.by_qname.get(&t.qualified_name).copied())
                .collect();
        }
        LinkSelector::Builtin(BuiltinField::AllocatedTo) => fm.allocated_to.clone().unwrap_or_default(),
    };
    refs.iter()
        .filter_map(|r| resolver.resolve_ref(elements, r))
        .filter_map(|t| resolver.by_qname.get(&t.qualified_name).copied())
        .collect()
}

/// One element reached by [`follow`].
#[derive(Debug, Clone)]
pub struct FollowHit {
    pub qname: String,
    pub id: Option<String>,
    pub type_name: String,
    pub name: Option<String>,
    /// Hop distance from the start (1 = direct).
    pub depth: usize,
    /// Label (id, else qname) of the element it was reached from.
    pub from: String,
}

/// The result of a [`follow`] traversal.
#[derive(Debug, Clone)]
pub struct FollowResult {
    /// Label (id, else qname) of the start element.
    pub start: String,
    pub start_qname: String,
    /// The link name as requested.
    pub link: String,
    /// `"forward"` or `"reverse"`.
    pub direction: &'static str,
    pub hits: Vec<FollowHit>,
    /// Every traversed edge as `(from label, to label)`, for DOT output.
    pub edges: Vec<(String, String)>,
}

/// Walk `link` from `start` (REQ-TRS-LINKTYPE-007). `link` is a declared type
/// (forward), its inverse (reverse), or a built-in link / reverse-index name;
/// `reverse` flips the direction. `max_depth` bounds the hops (`Some(1)` = one
/// hop, `None` = to a fixed point); each element is reported once, at its
/// shortest hop distance, and the start itself is never reported. Cycles
/// terminate. `Err` carries the unknown-link message listing available names.
pub fn follow(
    elements: &[RawElement],
    resolver: &Resolver,
    reg: &LinkTypeRegistry,
    start: &RawElement,
    link: &str,
    reverse: bool,
    max_depth: Option<usize>,
) -> Result<FollowResult, String> {
    let Some((sel, is_rev_name)) = resolve_link_name(link, reg) else {
        return Err(format!(
            "unknown link '{}'. Available links: {}",
            link,
            available_link_names(reg).join(", ")
        ));
    };
    let backward = is_rev_name ^ reverse;

    // Adjacency in the requested direction, over element indices.
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); elements.len()];
    for (i, e) in elements.iter().enumerate() {
        for t in forward_targets(sel, e, elements, resolver, reg) {
            if backward {
                adj[t].push(i);
            } else {
                adj[i].push(t);
            }
        }
    }
    for list in &mut adj {
        list.sort_by(|a, b| elements[*a].qualified_name.cmp(&elements[*b].qualified_name));
        list.dedup();
    }

    let Some(&root) = resolver.by_qname.get(&start.qualified_name) else {
        return Err(format!("element '{}' is not in the model", start.qualified_name));
    };
    let mut seen: HashSet<usize> = HashSet::from([root]);
    let mut queue: VecDeque<(usize, usize)> = VecDeque::from([(root, 0usize)]);
    let mut hits = Vec::new();
    let mut edges = Vec::new();
    while let Some((cur, d)) = queue.pop_front() {
        if max_depth.is_some_and(|m| d >= m) {
            continue;
        }
        for &n in &adj[cur] {
            edges.push((label(&elements[cur]), label(&elements[n])));
            if seen.insert(n) {
                let ne = &elements[n];
                hits.push(FollowHit {
                    qname: ne.qualified_name.clone(),
                    id: ne.frontmatter.id.clone(),
                    type_name: element_type_name(ne.frontmatter.element_type.as_ref()),
                    name: ne.frontmatter.name.clone(),
                    depth: d + 1,
                    from: label(&elements[cur]),
                });
                queue.push_back((n, d + 1));
            }
        }
    }
    edges.sort();
    edges.dedup();
    Ok(FollowResult {
        start: label(start),
        start_qname: start.qualified_name.clone(),
        link: link.to_string(),
        direction: if backward { "reverse" } else { "forward" },
        hits,
        edges,
    })
}

/// `follow --format json` / the MCP `follow` tool payload.
pub fn follow_json(r: &FollowResult) -> serde_json::Value {
    let results: Vec<serde_json::Value> = r
        .hits
        .iter()
        .map(|h| {
            serde_json::json!({
                "qname": h.qname,
                "id": h.id,
                "type": h.type_name,
                "name": h.name,
                "depth": h.depth,
                "from": h.from,
            })
        })
        .collect();
    serde_json::json!({
        "start": r.start,
        "link": r.link,
        "direction": r.direction,
        "results": results,
    })
}

/// `link-types --json` / the MCP `link_types` tool payload (REQ-TRS-LINKTYPE-008).
pub fn link_types_json(elements: &[RawElement], reg: &LinkTypeRegistry) -> serde_json::Value {
    let counts = instance_counts(elements, reg);
    let types: Vec<serde_json::Value> = reg
        .types()
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "inverse": t.inverse,
                "extends": t.extends.map(|b| b.name()),
                "relax": t.relax,
                "coverage": t.coverage,
                "sourceTypes": t.source_types,
                "targetTypes": t.target_types,
                "cardinality": t.cardinality.to_string(),
                "acyclic": t.acyclic,
                "suspect": t.suspect,
                "count": counts.get(&t.name).copied().unwrap_or(0),
            })
        })
        .collect();
    serde_json::json!({ "linkTypes": types })
}

/// The `## Project link types` section appended to `--agent-instructions` when a
/// model root declares link types (REQ-TRS-LINKTYPE-012). `None` when none are
/// declared, so nothing is appended.
pub fn agent_instructions_section(reg: &LinkTypeRegistry) -> Option<String> {
    if reg.is_empty() {
        return None;
    }
    let mut s = String::new();
    s.push_str("\n\n## Project link types\n\n");
    s.push_str(
        "This model declares the following link types in `.syscribe.toml`. Author them under \
         `links:` on the source element (the element holding the entry is the source, §12.1). \
         Use only these names — an undeclared key is `E630`.\n",
    );
    for t in reg.types() {
        s.push_str(&format!("\n### `{}`\n\n", t.name));
        if let Some(d) = &t.description {
            s.push_str(&format!("- Description: {}\n", d));
        }
        let src = t.source_types.as_ref().map_or("any type".to_string(), |v| v.join(", "));
        let tgt = t.target_types.as_ref().map_or("any type".to_string(), |v| v.join(", "));
        s.push_str(&format!("- Direction: {} → {}", src, tgt));
        match &t.inverse {
            Some(inv) => s.push_str(&format!(" (inverse: `{}`)\n", inv)),
            None => s.push('\n'),
        }
        s.push_str(&format!("- Cardinality (targets per source): {}\n", t.cardinality));
        s.push_str(&format!("- Acyclic: {}\n", if t.acyclic { "yes" } else { "no" }));
        if let Some(base) = t.extends {
            let relax = if t.relax.is_empty() { "none".to_string() } else { t.relax.join(", ") };
            s.push_str(&format!(
                "- Extends: `{}` (relaxed codes: {}; counts toward coverage: {})\n",
                base.name(),
                relax,
                if t.coverage { "yes" } else { "no" }
            ));
        }
        s.push_str(&format!("- Example:\n\n  ```yaml\n  links:\n    {}: [<target-id>]\n  ```\n", t.name));
    }
    Some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg(toml_text: &str) -> LinkTypeRegistry {
        LinkTypeRegistry::from_toml_str(toml_text)
    }

    fn defect_for(r: &LinkTypeRegistry, name: &str) -> bool {
        r.defects().iter().any(|d| d.contains(&format!("[linkTypes.{name}]")))
    }

    #[test]
    fn no_table_is_not_declared_and_empty() {
        let r = reg("[ids]\nmax_digits = 4\n");
        assert!(!r.is_declared());
        assert!(r.is_empty());
        assert!(r.defects().is_empty());
    }

    #[test]
    fn well_formed_entries_parse_with_defaults_and_both_key_cases() {
        let r = reg(
            "[linkTypes.mitigates]\ndescription = \"d\"\ninverse = \"mitigatedBy\"\nsource_types = [\"PartDef\"]\n\
             targetTypes = [\"Requirement\"]\ncardinality = \"0..*\"\n\n\
             [linkTypes.partiallySatisfies]\nextends = \"satisfies\"\nrelax = [\"E313\"]\ncoverage = false\n",
        );
        assert!(r.defects().is_empty(), "{:?}", r.defects());
        let m = r.get("mitigates").unwrap();
        assert_eq!(m.inverse.as_deref(), Some("mitigatedBy"));
        assert_eq!(m.source_types, Some(vec!["PartDef".to_string()]));
        assert!(m.suspect && !m.acyclic && m.coverage);
        let p = r.get("partiallySatisfies").unwrap();
        assert_eq!(p.extends, Some(BaseLink::Satisfies));
        assert!(!p.coverage && p.relaxes("E313"));
    }

    #[test]
    fn each_structural_defect_is_w630_and_ignores_the_entry() {
        let r = reg(
            "[linkTypes.Bad-Name]\n[linkTypes.satisfies]\n[linkTypes.alpha]\ninverse = \"verifiedBy\"\n\
             [linkTypes.badCard]\ncardinality = \"x..y\"\n[linkTypes.lowerNoSource]\ncardinality = \"1..*\"\n\
             [linkTypes.unknownType]\nsourceTypes = [\"NoSuchType\"]\n[linkTypes.badBase]\nextends = \"allocatedTo\"\n\
             [linkTypes.relaxNoExtends]\nrelax = [\"E313\"]\n[linkTypes.coverageNoExtends]\ncoverage = false\n\
             [linkTypes.badRelax]\nextends = \"satisfies\"\nrelax = [\"E104\"]\n[linkTypes.dupInverse]\ninverse = \"alpha\"\n\
             [linkTypes.invA]\ninverse = \"shared\"\n[linkTypes.invB]\ninverse = \"shared\"\n\
             [linkTypes.backwards]\ncardinality = \"3..1\"\n[linkTypes.topEvent]\n",
        );
        for n in [
            "Bad-Name", "satisfies", "alpha", "badCard", "lowerNoSource", "unknownType", "badBase",
            "relaxNoExtends", "coverageNoExtends", "badRelax", "dupInverse", "invA", "invB", "backwards", "topEvent",
        ] {
            assert!(defect_for(&r, n), "expected W630 for {n}: {:?}", r.defects());
            assert!(r.get(n).is_none(), "{n} must be ignored");
        }
        assert!(r.is_empty());
    }

    #[test]
    fn built_in_name_collisions_are_case_insensitive() {
        // `mitigatedby` (type) vs the built-in `mitigatedBy` edge kind, and
        // `Satisfies`-cased inverse vs the reserved `satisfies`.
        let r = reg("[linkTypes.mitigatedby]\n[linkTypes.fooLink]\ninverse = \"verifiedby\"\n[linkTypes.derivedfrom]\n");
        for n in ["mitigatedby", "fooLink", "derivedfrom"] {
            assert!(defect_for(&r, n), "expected W630 for {n}: {:?}", r.defects());
        }
        assert!(r.is_empty());
        // Declared names/inverses differing only in case collide too.
        let r = reg("[linkTypes.fooBar]\n[linkTypes.foobar]\n[linkTypes.aLink]\ninverse = \"fooBAR\"\n");
        assert!(defect_for(&r, "fooBar") && defect_for(&r, "foobar"), "{:?}", r.defects());
        assert!(defect_for(&r, "aLink"), "{:?}", r.defects());
        // A lowerCamel name unrelated to any built-in stays valid.
        assert!(reg("[linkTypes.mitigates]\ninverse = \"mitigatedBy\"\n").get("mitigates").is_some());
    }

    #[test]
    fn unknown_key_warns_but_keeps_the_entry() {
        let r = reg("[linkTypes.extraKey]\ncolour = \"blue\"\n");
        assert!(r.defects().iter().any(|d| d.contains("colour")));
        assert!(r.get("extraKey").is_some());
    }

    #[test]
    fn inverse_may_reuse_a_domain_edge_name_but_a_type_may_not() {
        let r = reg("[linkTypes.mitigates]\ninverse = \"mitigatedBy\"\n");
        assert!(r.get("mitigates").is_some(), "{:?}", r.defects());
        let r = reg("[linkTypes.mitigatedBy]\n");
        assert!(defect_for(&r, "mitigatedBy"));
    }

    #[test]
    fn cardinality_forms() {
        assert_eq!(Cardinality::parse("2"), Some(Cardinality { min: 2, max: Some(2) }));
        assert_eq!(Cardinality::parse("1..3"), Some(Cardinality { min: 1, max: Some(3) }));
        assert_eq!(Cardinality::parse("0..*"), Some(Cardinality { min: 0, max: None }));
        assert_eq!(Cardinality::parse("3..1"), None);
        assert_eq!(Cardinality::parse("x"), None);
        assert_eq!(Cardinality::parse("-1"), None);
        assert_eq!(Cardinality::parse("1..2").unwrap().to_string(), "1..2");
    }

    #[test]
    fn known_element_types() {
        assert!(is_known_element_type("PartDef"));
        assert!(is_known_element_type("Requirement"));
        assert!(!is_known_element_type("NoSuchType"));
        assert!(!is_known_element_type("Unknown"));
    }

    #[test]
    fn interner_is_stable() {
        let a = intern("someLinkName");
        assert_eq!(intern("someLinkName"), a);
        assert_eq!(interned(a), "someLinkName");
    }

    fn elem(qname: &str, yaml: &str) -> RawElement {
        RawElement {
            qualified_name: qname.to_string(),
            file_path: format!("{qname}.md"),
            frontmatter: serde_yaml::from_str(yaml).unwrap(),
            doc: String::new(),
            parse_issue: None,
            derived: Default::default(),
            derive_findings: vec![],
        }
    }

    #[test]
    fn effective_view_appends_and_tracks_provenance_without_touching_the_input() {
        let r = reg("[linkTypes.partSat]\nextends = \"satisfies\"\nrelax = [\"E313\"]\ncoverage = false\n");
        let elements = vec![
            elem("R", "type: Requirement\nid: REQ-001\nname: r\nstatus: draft\n"),
            elem("P", "type: PartDef\nsatisfies: [REQ-001]\nlinks:\n  partSat: [REQ-001, REQ-999]\n"),
        ];
        let view = effective_view(&elements, &r).expect("view");
        let p = &view.elements[1];
        // Only the resolved target is contributed.
        assert_eq!(p.frontmatter.satisfies, Some(vec!["REQ-001".to_string(), "REQ-001".to_string()]));
        assert!(!view.provenance.is_contributed("P", BaseLink::Satisfies, 0));
        assert!(view.provenance.relaxed("P", BaseLink::Satisfies, 1, "E313"));
        assert!(!view.provenance.relaxed("P", BaseLink::Satisfies, 0, "E313"));
        assert!(!view.provenance.in_reverse_index("P", BaseLink::Satisfies, 1));
        assert!(view.provenance.in_reverse_index("P", BaseLink::Satisfies, 0));
        // The caller's elements are never mutated.
        assert_eq!(elements[1].frontmatter.satisfies, Some(vec!["REQ-001".to_string()]));
    }

    #[test]
    fn coverage_view_skips_coverage_false_and_borrows_when_unused() {
        let r = reg("[linkTypes.strong]\nextends = \"satisfies\"\n[linkTypes.weak]\nextends = \"satisfies\"\ncoverage = false\n");
        let elements = vec![
            elem("R", "type: Requirement\nid: REQ-001\nname: r\nstatus: draft\n"),
            elem("A", "type: PartDef\nlinks:\n  strong: REQ-001\n"),
            elem("B", "type: PartDef\nlinks:\n  weak: REQ-001\n"),
        ];
        let (view, prov) = coverage_view_with_provenance(&elements, &r);
        assert!(matches!(view, std::borrow::Cow::Owned(_)));
        assert_eq!(view[1].frontmatter.satisfies, Some(vec!["REQ-001".to_string()]));
        assert_eq!(view[2].frontmatter.satisfies, None);
        assert_eq!(prov.via("A", BaseLink::Satisfies, 0), Some("strong"));
        // Idempotent: applying it again appends nothing.
        assert!(matches!(coverage_view(&view, &r), std::borrow::Cow::Borrowed(_)));
        let none = reg("[linkTypes.informs]\n");
        assert!(matches!(coverage_view(&elements, &none), std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn no_extensions_means_no_view() {
        let r = reg("[linkTypes.informs]\n");
        let elements = vec![elem("P", "type: PartDef\nlinks:\n  informs: P\n")];
        assert!(effective_view(&elements, &r).is_none());
    }

    #[test]
    fn follow_forward_reverse_transitive_and_cycles() {
        let r = reg("[linkTypes.next]\ninverse = \"prev\"\n");
        let elements = vec![
            elem("A", "type: PartDef\nlinks:\n  next: B\n"),
            elem("B", "type: PartDef\nlinks:\n  next: C\n"),
            elem("C", "type: PartDef\nlinks:\n  next: A\n"),
        ];
        let res = Resolver::new(&elements);
        let one = follow(&elements, &res, &r, &elements[0], "next", false, Some(1)).unwrap();
        assert_eq!(one.hits.iter().map(|h| h.qname.as_str()).collect::<Vec<_>>(), vec!["B"]);
        let all = follow(&elements, &res, &r, &elements[0], "next", false, None).unwrap();
        assert_eq!(all.hits.len(), 2);
        assert_eq!(all.hits[1].depth, 2);
        let inv = follow(&elements, &res, &r, &elements[0], "prev", false, Some(1)).unwrap();
        assert_eq!(inv.hits[0].qname, "C");
        assert_eq!(inv.direction, "reverse");
        let flipped = follow(&elements, &res, &r, &elements[0], "next", true, Some(1)).unwrap();
        assert_eq!(flipped.hits[0].qname, "C");
        let err = follow(&elements, &res, &r, &elements[0], "nope", false, None).unwrap_err();
        assert!(err.contains("next") && err.contains("satisfies"));
    }
}
