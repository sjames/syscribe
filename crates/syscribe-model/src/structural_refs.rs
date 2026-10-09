//! Unresolved structural cross-references (REQ-TRS-XREF-007, GH #125).
//!
//! `supertype:`, `typedBy:` (element-level and on inline `features:` entries),
//! `subsets:`, `redefines:` and `satisfies:` must resolve (§11.5 step 4, §11.7).
//! An unresolved one is reported as `E110`–`E114`:
//!
//! | Code | Field |
//! |---|---|
//! | `E110` | `supertype:` |
//! | `E111` | `typedBy:` (element-level, and inline `features:` entries) |
//! | `E112` | `subsets:` |
//! | `E113` | `redefines:` |
//! | `E114` | `satisfies:` |
//!
//! Resolution follows §11.5 and never flags a reference the rest of the tool
//! would resolve:
//!
//! 1. the ordinary id / qualified-name / display-name lookup
//!    ([`Resolver::resolve_ref`]), widened through the referencing element's
//!    enclosing-package scope chain ([`Resolver::resolve_scoped_ref`]) — the
//!    same lookup `graph.rs` and `W007` use for `typedBy:`, so SysML v2-ingested,
//!    plugin- and annotation-synthesized elements resolve exactly as elsewhere;
//! 2. a `./name` sibling reference (§5.2);
//! 3. `imports:` and `aliases:` declared by the element or any enclosing package
//!    (§5.3, §5.4, §11.5 step 3c);
//! 4. an inline, non-file feature of a resolvable owner (`Owner::feat`, or a bare
//!    `feat` of the element's owner), including one inherited through the owner's
//!    `supertype:`/`typedBy:` chain — `redefines:`/`subsets:` routinely name such
//!    features;
//! 5. the SysML v2 standard library: the auto-imported built-in packages
//!    (`ScalarValues`, `Base` — an unknown member of those is `W043`'s job), the
//!    curated ISQ/SI recognition, and any reference into a standard-library
//!    package (`ISQ::…`, `Parts::Part::…`, …) whose top-level name the model does
//!    not itself declare.
//!
//! A reference that resolves in a loaded `[repos]` peer is valid (§14.4). In a
//! `[repos]`-configured model an unresolved reference is reported as `E512`
//! (cross-repo reference resolves nowhere) instead of `E110`–`E114`, exactly as
//! `verifies:`/`derivedFrom:`/`satisfies:`/`allocatedTo:` already are — never both.

use std::collections::HashSet;

use crate::config::ValidateConfig;
use crate::element::{ElementType, RawElement};
use crate::members::parent_qname;
use crate::resolver::{builtin_type_kind, BuiltinType, Resolver};
use crate::validator::{Finding, Severity};

/// Top-level package names of the SysML v2 / KerML standard library (the
/// Kernel, Systems, Domain and Quantities-and-Units libraries). A reference
/// whose first segment is one of these, and which the model does not itself
/// declare, is an external library reference — never flagged.
pub const STDLIB_PACKAGES: &[&str] = &[
    // Kernel semantic / data-type / function libraries (KerML)
    "Base", "Links", "Occurrences", "Objects", "Performances", "Transfers", "Feedbacks",
    "Clocks", "Observation", "Triggers", "SpatialFrames", "ControlPerformances",
    "StatePerformances", "TransitionPerformances", "FeatureReferencingPerformances",
    "KerML", "Metaobjects", "ScalarValues", "VectorValues", "Collections",
    "BaseFunctions", "DataFunctions", "ScalarFunctions", "BooleanFunctions",
    "StringFunctions", "NumericalFunctions", "ComplexFunctions", "RealFunctions",
    "RationalFunctions", "IntegerFunctions", "NaturalFunctions", "TrigFunctions",
    "VectorFunctions", "SequenceFunctions", "CollectionFunctions", "ControlFunctions",
    "OccurrenceFunctions",
    // Systems library (SysML)
    "SysML", "Items", "Parts", "Ports", "Connections", "Interfaces", "Allocations",
    "Actions", "States", "Calculations", "Constraints", "Requirements", "Cases",
    "AnalysisCases", "VerificationCases", "UseCases", "Views", "Metadata",
    "Attributes", "Flows", "StandardViewDefinitions",
    // Domain libraries
    "AnalysisTooling", "SampledFunctions", "StateSpaceRepresentation",
    "TradeStudies", "ModelingMetadata", "RiskMetadata", "ParametersOfInterestMetadata",
    "ImageMetadata", "CauseAndEffect", "RequirementDerivation", "ShapeItems",
    "SpatialItems", "Quantities", "MeasurementReferences", "MeasurementRefCalculations",
    "QuantityCalculations", "TensorCalculations", "VectorCalculations", "Time",
    "ISQ", "ISQBase", "ISQSpaceTime", "ISQMechanics", "ISQThermodynamics",
    "ISQElectromagnetism", "ISQLight", "ISQAcoustics", "ISQChemistryMolecular",
    "ISQAtomicNuclear", "ISQCondensedMatter", "ISQCharacteristicNumbers",
    "ISQInformation", "SI", "SIPrefixes", "USCustomaryUnits",
];

/// Well-known root types of the standard library that SysML v2 sources
/// commonly reference by their bare name (the implicit supertypes of §11.4,
/// KerML's `Links::Link`/`BinaryLink`, and the `ScalarValues` primitives).
/// A bare reference to one of these, not shadowed by an in-model element, is a
/// library reference.
pub const STDLIB_BARE_TYPES: &[&str] = &[
    "Anything", "DataValue", "Link", "BinaryLink", "Occurrence", "EventOccurrence",
    "Object", "Performance", "Transfer", "Item", "Part", "Port", "Connection",
    "Interface", "Allocation", "Action", "StateAction", "Calculation", "Constraint",
    "Requirement", "Case", "AnalysisCase", "VerificationCase", "UseCase", "View",
    "Viewpoint", "Rendering", "SemanticMetadata", "Boolean", "String", "Integer",
    "Natural", "Real", "Rational", "Complex", "Number", "NumericalValue", "ScalarValue",
];

/// Frontmatter lists whose mapping entries declare named inline members.
fn inline_member_lists(e: &RawElement) -> impl Iterator<Item = &serde_yaml::Value> {
    let fm = &e.frontmatter;
    [
        fm.features.as_deref(),
        fm.parameters.as_deref(),
        fm.performs.as_deref(),
        fm.sub_actions.as_deref(),
        fm.sub_states.as_deref(),
        fm.ends.as_deref(),
        fm.operations.as_deref(),
        fm.connections.as_deref(),
        fm.flow_connections.as_deref(),
        fm.items.as_deref(),
    ]
    .into_iter()
    .flatten()
    .flatten()
}

fn yaml_strings(v: &serde_yaml::Value) -> Vec<&str> {
    match v {
        serde_yaml::Value::String(s) => vec![s.as_str()],
        serde_yaml::Value::Sequence(seq) => seq.iter().filter_map(|x| x.as_str()).collect(),
        _ => vec![],
    }
}

fn map_str<'a>(m: &'a serde_yaml::Mapping, k: &str) -> Option<&'a str> {
    m.get(serde_yaml::Value::String(k.to_string())).and_then(|v| v.as_str())
}

struct Ctx<'a> {
    elements: &'a [RawElement],
    resolver: &'a Resolver,
    config: &'a ValidateConfig,
    /// First `::` segment of every element's qualified name — the top-level
    /// names the model itself declares (shadowing a library package name).
    top_level: HashSet<&'a str>,
}

impl<'a> Ctx<'a> {
    fn scoped(&self, from: &str, r: &str) -> Option<&'a RawElement> {
        self.resolver.resolve_scoped_ref(self.elements, from, r)
    }

    /// A standard-library reference (item 5 of the module doc).
    fn is_library_ref(&self, r: &str) -> bool {
        if !matches!(builtin_type_kind(r), BuiltinType::NotBuiltin)
            || (r.contains("::") && crate::units::is_recognised_type_ref(r))
        {
            return true;
        }
        match r.split_once("::") {
            Some((head, _)) => STDLIB_PACKAGES.contains(&head) && !self.top_level.contains(head),
            None => STDLIB_BARE_TYPES.contains(&r),
        }
    }

    /// Whether `owner` declares — as a child element file or an inline entry —
    /// a member named `name`, directly or through its `supertype:`/`typedBy:`
    /// chain. Cycle-safe.
    fn has_member(&self, owner: &RawElement, name: &str, seen: &mut HashSet<String>) -> bool {
        if !seen.insert(owner.qualified_name.clone()) {
            return false;
        }
        let child = if owner.qualified_name.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", owner.qualified_name, name)
        };
        if self.resolver.get(self.elements, &child).is_some() {
            return true;
        }
        let declares_inline = inline_member_lists(owner).any(|v| match v {
            serde_yaml::Value::Mapping(m) => map_str(m, "name") == Some(name),
            _ => false,
        });
        if declares_inline {
            return true;
        }
        let fm = &owner.frontmatter;
        for field in [fm.supertype.as_ref(), fm.typed_by.as_ref()].into_iter().flatten() {
            for t in yaml_strings(field) {
                if self.is_library_ref(t) {
                    // An inherited library feature (e.g. `Parts::Part`'s members)
                    // cannot be enumerated — stay lenient.
                    return true;
                }
                if let Some(target) = self.scoped(&owner.qualified_name, t) {
                    if self.has_member(target, name, seen) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// The member `name` of `owner` — a child element, an inline entry, or one
    /// inherited through `supertype:`/`typedBy:`. `None`: no such member.
    /// `Some(None)`: it exists but its type cannot be followed (untyped, library
    /// or unresolved). `Some(Some(e))`: the element whose members the next
    /// chain segment is looked up in.
    fn member_type(
        &self,
        owner: &'a RawElement,
        name: &str,
        seen: &mut HashSet<String>,
    ) -> Option<Option<&'a RawElement>> {
        if !seen.insert(owner.qualified_name.clone()) {
            return None;
        }
        let child = if owner.qualified_name.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", owner.qualified_name, name)
        };
        if let Some(c) = self.resolver.get(self.elements, &child) {
            return Some(Some(c));
        }
        for v in inline_member_lists(owner) {
            let serde_yaml::Value::Mapping(m) = v else { continue };
            if map_str(m, "name") != Some(name) {
                continue;
            }
            return Some(map_str(m, "typedBy").and_then(|tb| self.scoped(&owner.qualified_name, tb)));
        }
        let fm = &owner.frontmatter;
        for field in [fm.supertype.as_ref(), fm.typed_by.as_ref()].into_iter().flatten() {
            for t in yaml_strings(field) {
                match self.scoped(&owner.qualified_name, t) {
                    Some(target) => {
                        if let Some(r) = self.member_type(target, name, seen) {
                            return Some(r);
                        }
                    }
                    // A library or unresolved supertype may supply any member.
                    None => return Some(None),
                }
            }
        }
        None
    }

    /// Walk a dotted feature chain from `start`; `Err(segment)` names the first
    /// segment that is not a member of the element reached so far.
    fn walk_chain(&self, start: &'a RawElement, chain: &str) -> Result<(), String> {
        let norm = chain.replace("::", ".");
        let mut owner = Some(start);
        for seg in norm.split('.').map(str::trim).filter(|s| !s.is_empty()) {
            let Some(o) = owner else { return Ok(()) };
            match self.member_type(o, seg, &mut HashSet::new()) {
                None => return Err(seg.to_string()),
                Some(next) => owner = next,
            }
        }
        Ok(())
    }

    /// `r` names a feature the element itself inherits through its `supertype:`/
    /// `typedBy:` chain or declares inline — what an inline feature may redefine
    /// or subset.
    fn inherited_by_element(&self, elem: &RawElement, r: &str) -> bool {
        let name = r.rsplit("::").next().unwrap_or(r);
        self.has_member(elem, name, &mut HashSet::new())
    }

    /// `r` resolves to an inline (or inherited) feature: `Owner::feat` with a
    /// resolvable `Owner`, or — for a bare name — a member of the referencing
    /// element's owner.
    fn resolves_as_feature(&self, elem: &RawElement, r: &str) -> bool {
        match r.rsplit_once("::") {
            Some((owner, feat)) => {
                if self.is_library_ref(owner) {
                    return true;
                }
                match self.scoped(&elem.qualified_name, owner) {
                    Some(o) => self.has_member(o, feat, &mut HashSet::new()),
                    None => false,
                }
            }
            None => {
                let Some(parent) = parent_qname(&elem.qualified_name) else { return false };
                let Some(owner) = self.resolver.get(self.elements, parent) else { return false };
                self.has_member(owner, r, &mut HashSet::new())
            }
        }
    }

    /// `r` resolves through an `imports:`/`aliases:` declaration of `elem` or
    /// any enclosing namespace (§11.5 step 3c, §5.4).
    fn resolves_via_imports(&self, elem: &RawElement, r: &str) -> bool {
        let (head, rest) = match r.split_once("::") {
            Some((h, t)) => (h, Some(t)),
            None => (r, None),
        };
        let mut scope = Some(elem.qualified_name.as_str());
        while let Some(q) = scope {
            if let Some(ns) = self.resolver.get(self.elements, q) {
                let fm = &ns.frontmatter;
                for a in fm.aliases.iter().flatten() {
                    let serde_yaml::Value::Mapping(m) = a else { continue };
                    if map_str(m, "name") == Some(head) {
                        if let Some(target) = map_str(m, "for") {
                            let full = match rest {
                                Some(t) => format!("{target}::{t}"),
                                None => target.to_string(),
                            };
                            if self.scoped(q, &full).is_some()
                                || self.is_library_ref(&full)
                                || self.is_library_ref(target)
                            {
                                return true;
                            }
                        }
                    }
                }
                for imp in fm.imports.iter().flatten() {
                    let target = match imp {
                        serde_yaml::Value::String(s) => s.as_str(),
                        serde_yaml::Value::Mapping(m) => match map_str(m, "target") {
                            Some(t) => t,
                            None => continue,
                        },
                        _ => continue,
                    };
                    if self.import_provides(q, target, head, r) {
                        return true;
                    }
                }
            }
            scope = parent_qname(q);
        }
        false
    }

    /// Whether `head` is the first segment of a qualified name the model itself
    /// declares. A wildcard import of a library package must not excuse a
    /// reference into the model's own namespaces (it cannot enumerate those).
    fn names_model_namespace(&self, head: &str) -> bool {
        // A model package that reuses a standard-library package name keeps the
        // library leniency (`Calculations::Calculation` under a model
        // `Calculations/` directory), as it had before.
        if STDLIB_PACKAGES.contains(&head) {
            return false;
        }
        let prefix = format!("{head}::");
        self.elements
            .iter()
            .any(|e| e.qualified_name == head || e.qualified_name.starts_with(&prefix))
    }

    /// Whether a wildcard import of the library package `pkg` makes `r` (first
    /// segment `head`) visible. Where the package's full membership is known
    /// (`ISQ`, `SI`, `ScalarValues`, … — [`crate::units::library_member_known`]) an
    /// unknown first segment is *not* provided (GH #210); for any other library
    /// package, whose members cannot be enumerated, the import stays lenient.
    fn library_wildcard_provides(&self, pkg: &str, head: &str, r: &str, recursive: bool) -> bool {
        if self.names_model_namespace(head) {
            return false;
        }
        if !self.top_level.contains(pkg) {
            match crate::units::library_member_known(pkg, head) {
                Some(true) => return true,
                // `pkg::**` also exposes the library's nested packages.
                Some(false) => return recursive && STDLIB_PACKAGES.contains(&head),
                None => {}
            }
        }
        self.is_library_ref(&format!("{pkg}::{r}")) || self.is_library_ref(pkg)
    }

    /// Whether one `imports:` entry `target`, declared in namespace `ns`, makes
    /// the reference `r` (whose first segment is `head`) visible.
    fn import_provides(&self, ns: &str, target: &str, head: &str, r: &str) -> bool {
        let target = target.trim();
        if let Some(pkg) = target.strip_suffix("::**") {
            if self.library_wildcard_provides(pkg, head, r, true) {
                return true;
            }
            let Some(p) = self.scoped(ns, pkg) else { return false };
            let prefix = format!("{}::", p.qualified_name);
            let suffix = format!("::{r}");
            return self
                .elements
                .iter()
                .any(|e| e.qualified_name.starts_with(&prefix) && e.qualified_name.ends_with(&suffix));
        }
        if let Some(pkg) = target.strip_suffix("::*") {
            let full = format!("{pkg}::{r}");
            if self.library_wildcard_provides(pkg, head, r, false) {
                return true;
            }
            return self.scoped(ns, &full).is_some();
        }
        // Membership import: `Pkg::Name` makes `Name` visible.
        let last = target.rsplit("::").next().unwrap_or(target);
        if last != head {
            return false;
        }
        let full = match r.split_once("::") {
            Some((_, t)) => format!("{target}::{t}"),
            None => target.to_string(),
        };
        // `import ISQ::Bogus;` — a member the complete library table lacks (GH #210).
        if let Some((pkg, member)) = target.rsplit_once("::") {
            if !self.top_level.contains(pkg.split("::").next().unwrap_or(pkg))
                && crate::units::library_member_known(pkg, member) == Some(false)
            {
                return self.scoped(ns, &full).is_some();
            }
        }
        self.is_library_ref(&full) || self.is_library_ref(target) || self.scoped(ns, &full).is_some()
    }

    /// E128 — a `via:`/`to:` port or receiver chain on a send/accept step. The step
    /// is written in an action that is performed by some part, so its chain may
    /// name a port or part the action itself does not own: it resolves when it
    /// resolves as a reference, walks as a feature chain from the element, or its
    /// first segment names any element or inline member in the model (`known`).
    fn port_chain_resolves(&self, elem: &'a RawElement, chain: &str, known: &HashSet<String>) -> bool {
        let c = chain.trim();
        if c.is_empty() || self.resolves(elem, c) || self.walk_chain(elem, c).is_ok() {
            return true;
        }
        let head = c.replace("::", ".");
        let head = head.split('.').next().unwrap_or("").trim().to_string();
        head.is_empty() || head == "self" || head == "this" || known.contains(&head)
    }

    fn resolves(&self, elem: &RawElement, r: &str) -> bool {
        let r = r.trim();
        if r.is_empty() {
            return true;
        }
        if let Some(sib) = r.strip_prefix("./") {
            let parent = parent_qname(&elem.qualified_name).unwrap_or("");
            let full = if parent.is_empty() { sib.to_string() } else { format!("{parent}::{sib}") };
            return self.resolver.get(self.elements, &full).is_some()
                || self
                    .resolver
                    .get(self.elements, parent)
                    .is_some_and(|o| self.has_member(o, sib, &mut HashSet::new()));
        }
        self.scoped(&elem.qualified_name, r).is_some()
            || self.is_library_ref(r)
            || self.config.peer_resolves(r)
            || self.resolves_as_feature(elem, r)
            || self.resolves_via_imports(elem, r)
    }
}

fn error(code: &'static str, file: &str, msg: String) -> Finding {
    Finding { code, file: file.to_string(), message: msg, severity: Severity::Error }
}

/// E110–E114: every unresolved `supertype:`/`typedBy:`/`subsets:`/
/// `redefines:`/`satisfies:` reference, one finding per reference. In a
/// `[repos]`-configured model an unresolved reference is `E512` instead (the
/// convention every other cross-reference field already follows, §14.4), so a
/// reference is never reported twice.
pub fn unresolved_structural_ref_findings(
    elements: &[RawElement],
    resolver: &Resolver,
    config: &ValidateConfig,
) -> Vec<Finding> {
    let ctx = Ctx {
        elements,
        resolver,
        config,
        top_level: elements
            .iter()
            .filter(|e| !e.qualified_name.is_empty())
            .map(|e| e.qualified_name.split("::").next().unwrap_or(""))
            .collect(),
    };
    let mut out = Vec::new();
    let mut report = |code: &'static str, field: &str, file: &str, r: &str, suffix: String| {
        if config.has_repos() {
            out.push(error(
                "E512",
                file,
                format!("cross-repo {field} reference '{r}' resolves neither locally nor in any loaded repo{suffix}"),
            ));
        } else {
            out.push(error(code, file, format!("unresolved {field} reference '{r}'{suffix}")));
        }
    };
    let mut warns: Vec<Finding> = Vec::new();
    for elem in elements {
        let fm = &elem.frontmatter;
        let file = elem.file_path.as_str();
        for s in fm.supertype.iter().flat_map(yaml_strings) {
            if !ctx.resolves(elem, s) {
                report("E110", "supertype", file, s, String::new());
            }
        }
        // An ingested `allocation` usage is checked like any other: SysML v2
        // ingestion maps `allocation def` to `AllocationDef` (REQ-TRS-SYSMLV2-029,
        // GH #142), so its typedBy can resolve in-model.
        for s in fm.typed_by.iter().flat_map(yaml_strings) {
            if !ctx.resolves(elem, s) {
                report("E111", "typedBy", file, s, String::new());
            }
        }
        for feat in fm.features.iter().flatten() {
            let serde_yaml::Value::Mapping(m) = feat else { continue };
            let Some(tb) = m.get(serde_yaml::Value::String("typedBy".into())) else { continue };
            let fname = map_str(m, "name").unwrap_or("?");
            for s in yaml_strings(tb) {
                if !ctx.resolves(elem, s) {
                    report("E111", "typedBy", file, s, format!(" on inline feature '{fname}'"));
                }
            }
        }
        for s in fm.subsets.iter().flatten() {
            if !ctx.resolves(elem, s) {
                report("E112", "subsets", file, s, String::new());
            }
        }
        for s in fm.redefines.iter().flat_map(yaml_strings) {
            if !ctx.resolves(elem, s) {
                report("E113", "redefines", file, s, String::new());
            }
        }
        // GH #201 — inline-feature `redefines:`/`subsets:`, `dependsOn:`, `imports:`
        // and `aliases:` targets.
        for feat in fm.features.iter().flatten() {
            let serde_yaml::Value::Mapping(m) = feat else { continue };
            let fname = map_str(m, "name").unwrap_or("?");
            for (key, code) in [("redefines", "E113"), ("subsets", "E112")] {
                let Some(v) = m.get(serde_yaml::Value::String(key.into())) else { continue };
                for s in yaml_strings(v) {
                    if !ctx.resolves(elem, s) && !ctx.inherited_by_element(elem, s) {
                        report(code, key, file, s, format!(" on inline feature '{fname}'"));
                    }
                }
            }
        }
        // `connections:` endpoints, plus the `from`/`to` of `flowConnections:` and the
        // `after`/`before` of `successionConnections:` on a structural element (a
        // behavioral element's own flow/succession endpoints are E116's, GH #207).
        let behavioral_owner = matches!(
            fm.element_type,
            Some(
                ElementType::ActionDef | ElementType::Action | ElementType::UseCaseDef | ElementType::UseCase
                    | ElementType::CaseDef | ElementType::Case | ElementType::AnalysisCaseDef
                    | ElementType::AnalysisCase | ElementType::VerificationCaseDef | ElementType::VerificationCase
            )
        );
        let mut chains: Vec<String> = Vec::new();
        {
            for entry in fm.connections.iter().flatten() {
                let Some(parsed) = crate::connections::parse_entry(entry) else { continue };
                chains.extend(parsed.endpoints.into_iter().map(|ep| ep.chain));
            }
            if !behavioral_owner {
                for (list, keys) in [
                    (fm.flow_connections.as_deref(), ["from", "to"]),
                    (fm.succession_connections.as_deref(), ["after", "before"]),
                ] {
                    for v in list.unwrap_or(&[]) {
                        let serde_yaml::Value::Mapping(m) = v else { continue };
                        chains.extend(keys.iter().filter_map(|k| map_str(m, k)).map(str::to_string));
                    }
                }
            }
        }
        for c in chains.iter().map(|c| c.trim()) {
            if c.is_empty() || ctx.resolves(elem, c) {
                continue;
            }
            // An ingested endpoint is owner-qualified (`Owner::a::b::c`, GH #206); the walk
            // starts at the owner, so drop that prefix.
            let owner_prefix = format!("{}::", elem.qualified_name);
            let rel = c.strip_prefix(owner_prefix.as_str()).unwrap_or(c);
            if let Err(seg) = ctx.walk_chain(elem, rel) {
                let head = rel.replace("::", ".");
                if head.split('.').next().map(str::trim) == Some(seg.as_str()) {
                    report(
                        "E127",
                        "connection endpoint",
                        file,
                        c,
                        format!(": '{seg}' is not a member of the element it is looked up in"),
                    );
                } else {
                    // The owning part exists but the port named after it is not found on
                    // it: models often wire a sibling port loosely, so advise only.
                    warns.push(Finding {
                        code: "W056",
                        file: file.to_string(),
                        message: format!("connection endpoint '{c}': '{seg}' is not a member of the element it is looked up in"),
                        severity: Severity::Warning,
                    });
                }
            }
        }
        // W059 — a `visibility: private` element referenced from outside the
        // namespace that owns it.
        {
            let mut private_refs: Vec<&str> = fm.supertype.iter().flat_map(yaml_strings).collect();
            private_refs.extend(fm.typed_by.iter().flat_map(yaml_strings));
            for feat in fm.features.iter().flatten() {
                if let serde_yaml::Value::Mapping(m) = feat {
                    if let Some(tb) = map_str(m, "typedBy") {
                        private_refs.push(tb);
                    }
                }
            }
            for r in private_refs {
                let Some(t) = ctx.scoped(&elem.qualified_name, r) else { continue };
                if t.frontmatter.visibility.as_deref() != Some("private") {
                    continue;
                }
                let owner = parent_qname(&t.qualified_name).unwrap_or("");
                let inside = owner.is_empty()
                    || elem.qualified_name == owner
                    || elem.qualified_name.starts_with(&format!("{owner}::"));
                if !inside {
                    warns.push(Finding {
                        code: "W059",
                        file: file.to_string(),
                        message: format!("'{r}' is `visibility: private` in '{owner}' and is referenced from outside it"),
                        severity: Severity::Warning,
                    });
                }
            }
        }
        for s in fm.depends_on.iter().flatten() {
            if !ctx.resolves(elem, s) {
                report("E126", "dependsOn", file, s, String::new());
            }
        }
        for imp in fm.imports.iter().flatten() {
            let target = match imp {
                serde_yaml::Value::String(s) => Some(s.as_str()),
                serde_yaml::Value::Mapping(m) => map_str(m, "target"),
                _ => None,
            };
            let Some(t) = target.map(str::trim) else { continue };
            let pkg = t.strip_suffix("::**").or_else(|| t.strip_suffix("::*")).unwrap_or(t);
            let library_pkg = {
                let head = pkg.split("::").next().unwrap_or(pkg);
                STDLIB_PACKAGES.contains(&head) && !ctx.top_level.contains(head)
            };
            if !pkg.is_empty() && !library_pkg && !ctx.resolves(elem, pkg) {
                report("E126", "imports", file, t, String::new());
            }
        }
        for a in fm.aliases.iter().flatten() {
            let serde_yaml::Value::Mapping(m) = a else { continue };
            if let Some(t) = map_str(m, "for") {
                if !ctx.resolves(elem, t) {
                    report("E126", "aliases", file, t, format!(" (alias '{}')", map_str(m, "name").unwrap_or("?")));
                }
            }
        }
        // satisfies: resolved exactly as the satisfiedBy index resolves it. In a
        // `[repos]` model the existing satisfies check already reports E512.
        if !config.has_repos() {
            for s in fm.satisfies.iter().flatten() {
                if resolver.resolve_ref(elements, s).is_none() {
                    report("E114", "satisfies", file, s, String::new());
                }
            }
        }
    }
    out.extend(warns);
    out
}

/// Closed vocabularies of the behavior schema (spec §8.7).
const SUB_ACTION_KINDS: &[&str] = &[
    "Action", "PerformAction", "SendAction", "AcceptAction", "AssignmentAction", "IfAction",
    "LoopAction", "TerminateAction", "DecisionNode", "ForkNode", "JoinNode", "MergeNode",
];
const CONTROL_NODE_KINDS: &[&str] = &["DecisionNode", "ForkNode", "JoinNode", "MergeNode"];
const LOOP_KINDS: &[&str] = &["while", "until", "for"];
const PARAM_DIRECTIONS: &[&str] = &["in", "out", "inout", "return"];

/// Every sub-action mapping reachable from `list` through `then:`/`else:`/
/// `body:`/`subActions:`.
fn walk_sub_actions<'v>(list: &'v [serde_yaml::Value], out: &mut Vec<&'v serde_yaml::Mapping>) {
    for v in list {
        let serde_yaml::Value::Mapping(m) = v else { continue };
        out.push(m);
        for key in ["then", "else", "body", "subActions"] {
            if let Some(serde_yaml::Value::Sequence(inner)) = m.get(serde_yaml::Value::String(key.into())) {
                walk_sub_actions(inner, out);
            }
        }
    }
}

/// Every state mapping reachable from `list` through nested `subStates:`.
fn walk_states<'v>(list: &'v [serde_yaml::Value], out: &mut Vec<&'v serde_yaml::Mapping>) {
    for v in list {
        let serde_yaml::Value::Mapping(m) = v else { continue };
        out.push(m);
        if let Some(serde_yaml::Value::Sequence(inner)) = m.get(serde_yaml::Value::String("subStates".into())) {
            walk_states(inner, out);
        }
    }
}

fn map_seq<'a>(m: &'a serde_yaml::Mapping, k: &str) -> &'a [serde_yaml::Value] {
    match m.get(serde_yaml::Value::String(k.to_string())) {
        Some(serde_yaml::Value::Sequence(s)) => s,
        _ => &[],
    }
}

/// `E115` — an unresolved reference in a behavior field (sub-action/parameter
/// `typedBy`, `payload`, `includes`, `extends[].target`, `subject`, `result`,
/// `returnType`); `E116` — a succession/binding/flow endpoint whose first
/// segment names no sub-action, control node or parameter of the element;
/// `E117` — a value outside a behavior field's closed vocabulary (sub-action
/// or control-node `kind`, `loopKind`, parameter `direction`). GH #199.
pub fn behavior_ref_findings(
    elements: &[RawElement],
    resolver: &Resolver,
    config: &ValidateConfig,
) -> Vec<Finding> {
    let ctx = Ctx {
        elements,
        resolver,
        config,
        top_level: elements
            .iter()
            .filter(|e| !e.qualified_name.is_empty())
            .map(|e| e.qualified_name.split("::").next().unwrap_or(""))
            .collect(),
    };
    let mut out = Vec::new();
    // Every name a send/accept `via:`/`to:` chain may start with: any element's own
    // name or any inline member name (ports, parts, features) anywhere in the model.
    let mut known_names: HashSet<String> = HashSet::new();
    for e in elements {
        if let Some(last) = e.qualified_name.rsplit("::").next().filter(|l| !l.is_empty()) {
            known_names.insert(last.to_string());
        }
        for v in inline_member_lists(e) {
            if let serde_yaml::Value::Mapping(m) = v {
                if let Some(n) = map_str(m, "name") {
                    known_names.insert(n.to_string());
                }
            }
        }
    }
    for elem in elements {
        let fm = &elem.frontmatter;
        let file = elem.file_path.as_str();
        // A Diagram's `subject:` is checked by the diagram rules (it may name a bare directory).
        if matches!(fm.element_type, Some(crate::element::ElementType::Diagram)) {
            continue;
        }
        // An FMEAEntry's `ref:` (stored as `subject`) is reported once, as `W904`
        // (GH #218) — not also as a behavior-field `E115`.
        if matches!(fm.element_type, Some(crate::element::ElementType::FMEAEntry)) {
            continue;
        }
        // GH #207 — `via:`/`to:` chains of send/accept steps and of state-transition
        // `accept:` triggers must name something that exists.
        {
            let mut chains: Vec<(String, String)> = Vec::new();
            let mut all_subs: Vec<&serde_yaml::Mapping> = Vec::new();
            walk_sub_actions(fm.sub_actions.as_deref().unwrap_or(&[]), &mut all_subs);
            for m in &all_subs {
                let n = map_str(m, "name").unwrap_or("?");
                let keys: &[&str] = match map_str(m, "kind") {
                    Some("SendAction") => &["via", "to"],
                    Some("AcceptAction") => &["via"],
                    _ => &[],
                };
                for k in keys {
                    if let Some(c) = map_str(m, k) {
                        chains.push((format!("sub-action '{n}' `{k}`"), c.to_string()));
                    }
                }
            }
            let mut states: Vec<&serde_yaml::Mapping> = Vec::new();
            walk_states(fm.sub_states.as_deref().unwrap_or(&[]), &mut states);
            let mut transitions: Vec<&serde_yaml::Value> = fm.transitions.iter().flatten().collect();
            for st in &states {
                transitions.extend(map_seq(st, "transitions"));
            }
            for t in transitions {
                let serde_yaml::Value::Mapping(tm) = t else { continue };
                if let Some(serde_yaml::Value::Mapping(acc)) = tm.get(serde_yaml::Value::String("accept".into())) {
                    if let Some(c) = map_str(acc, "via") {
                        chains.push(("transition accept `via`".to_string(), c.to_string()));
                    }
                }
            }
            for (what, c) in chains {
                if ctx.port_chain_resolves(elem, &c, &known_names) {
                    continue;
                }
                if config.has_repos() {
                    out.push(error(
                        "E512",
                        file,
                        format!("cross-repo {what} reference '{c}' resolves neither locally nor in any loaded repo"),
                    ));
                } else {
                    out.push(error(
                        "E128",
                        file,
                        format!("{what} chain '{c}' names no port, part or feature in the model"),
                    ));
                }
            }
        }
        let empty: Vec<serde_yaml::Value> = Vec::new();
        let subs_list = fm.sub_actions.as_ref().unwrap_or(&empty);
        let nodes = fm.control_nodes.as_ref().unwrap_or(&empty);
        let params = fm.parameters.as_ref().unwrap_or(&empty);
        let has_refs = fm.includes.is_some()
            || fm.extends.is_some()
            || fm.subject.is_some()
            || fm.result_type.is_some()
            || fm.return_type.is_some();
        if subs_list.is_empty() && nodes.is_empty() && params.is_empty() && !has_refs
            && fm.succession_connections.is_none()
            && fm.binding_connections.is_none()
        {
            continue;
        }
        let mut unresolved = |what: &str, r: &str| {
            let r = r.trim();
            if !r.is_empty() && !ctx.resolves(elem, r) {
                let mut msg = format!("unresolved {what} reference '{r}'");
                if config.has_repos() {
                    msg = format!("cross-repo {what} reference '{r}' resolves neither locally nor in any loaded repo");
                    out.push(error("E512", file, msg));
                } else {
                    out.push(error("E115", file, msg));
                }
            }
        };
        let mut subs: Vec<&serde_yaml::Mapping> = Vec::new();
        walk_sub_actions(subs_list, &mut subs);
        let mut names: HashSet<&str> = HashSet::new();
        let mut bad: Vec<String> = Vec::new();
        let check_param = |m: &serde_yaml::Mapping, owner: &str, unresolved: &mut dyn FnMut(&str, &str), bad: &mut Vec<String>| {
            if let Some(t) = map_str(m, "typedBy") {
                unresolved(&format!("{owner} parameter typedBy"), t);
            }
            if let Some(d) = map_str(m, "direction") {
                if !PARAM_DIRECTIONS.contains(&d) {
                    bad.push(format!("parameter direction '{d}' on {owner} (expected in, out, inout or return)"));
                }
            }
        };
        for v in params {
            if let serde_yaml::Value::Mapping(m) = v {
                if let Some(n) = map_str(m, "name") {
                    names.insert(n);
                }
                check_param(m, "element", &mut unresolved, &mut bad);
            }
        }
        for v in nodes {
            let serde_yaml::Value::Mapping(m) = v else { continue };
            let n = map_str(m, "name").unwrap_or("?");
            names.insert(n);
            match map_str(m, "kind") {
                Some(k) if CONTROL_NODE_KINDS.contains(&k) => {}
                Some(k) => bad.push(format!("control node '{n}' kind '{k}' (expected {})", CONTROL_NODE_KINDS.join(", "))),
                None => {}
            }
            for p in map_seq(m, "parameters") {
                if let serde_yaml::Value::Mapping(pm) = p {
                    check_param(pm, &format!("control node '{n}'"), &mut unresolved, &mut bad);
                }
            }
        }
        for m in &subs {
            let n = map_str(m, "name").unwrap_or("?");
            names.insert(n);
            if let Some(t) = map_str(m, "typedBy") {
                unresolved(&format!("sub-action '{n}' typedBy"), t);
            }
            if let Some(t) = map_str(m, "payload") {
                unresolved(&format!("sub-action '{n}' payload"), t);
            }
            if let Some(k) = map_str(m, "kind") {
                if !SUB_ACTION_KINDS.contains(&k) {
                    bad.push(format!("sub-action '{n}' kind '{k}' (expected {})", SUB_ACTION_KINDS.join(", ")));
                }
            }
            if let Some(k) = map_str(m, "loopKind") {
                if !LOOP_KINDS.contains(&k) {
                    bad.push(format!("sub-action '{n}' loopKind '{k}' (expected while, until or for)"));
                }
            }
            for p in map_seq(m, "parameters") {
                if let serde_yaml::Value::Mapping(pm) = p {
                    // Invocation bindings (`{name, value}`) carry no typedBy/direction.
                    check_param(pm, &format!("sub-action '{n}'"), &mut unresolved, &mut bad);
                }
            }
        }
        for s in fm.includes.iter().flatten() {
            unresolved("includes", s);
        }
        for e in fm.extends.iter().flatten() {
            if let serde_yaml::Value::Mapping(m) = e {
                if let Some(t) = map_str(m, "target") {
                    unresolved("extends target", t);
                }
            }
        }
        if let Some(s) = &fm.subject {
            unresolved("subject", s);
        }
        if let Some(s) = &fm.result_type {
            unresolved("result", s);
        }
        if let Some(s) = &fm.return_type {
            unresolved("returnType", s);
        }
        // Endpoint chains: the first segment must name something the element owns.
        // An element that declares no sub-actions, control nodes or parameters has no
        // names to check against (steps may live only in its prose).
        let has_names = !names.is_empty();
        let mut endpoint = |what: &str, chain: Option<&str>| {
            let Some(c) = chain else { return };
            if !has_names {
                return;
            }
            let head = c.trim().split(['.', ':']).next().unwrap_or("").trim();
            if head.is_empty() || names.contains(head) || head == "self" || head == "this" {
                return;
            }
            out.push(error(
                "E116",
                file,
                format!("{what} '{c}' names no sub-action, control node or parameter of this element"),
            ));
        };
        use crate::element::ElementType as T;
        let behavioral = matches!(
            fm.element_type,
            Some(T::ActionDef | T::Action | T::UseCaseDef | T::UseCase | T::CaseDef | T::Case
                | T::AnalysisCaseDef | T::AnalysisCase | T::VerificationCaseDef | T::VerificationCase)
        );
        for v in fm.succession_connections.iter().flatten().filter(|_| behavioral) {
            if let serde_yaml::Value::Mapping(m) = v {
                endpoint("succession `after`", map_str(m, "after"));
                endpoint("succession `before`", map_str(m, "before"));
            }
        }
        for v in fm.binding_connections.iter().flatten().filter(|_| behavioral) {
            if let serde_yaml::Value::Mapping(m) = v {
                endpoint("binding `left`", map_str(m, "left"));
                endpoint("binding `right`", map_str(m, "right"));
            }
        }
        if matches!(
            fm.element_type,
            Some(crate::element::ElementType::ActionDef) | Some(crate::element::ElementType::Action)
        ) {
            for v in fm.flow_connections.iter().flatten() {
                if let serde_yaml::Value::Mapping(m) = v {
                    endpoint("flow `from`", map_str(m, "from"));
                    endpoint("flow `to`", map_str(m, "to"));
                }
            }
        }
        for b in bad {
            out.push(error("E117", file, format!("invalid value: {b}")));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn elem(qname: &str, yaml: &str) -> RawElement {
        RawElement {
            qualified_name: qname.to_string(),
            file_path: format!("{qname}.md"),
            frontmatter: serde_yaml::from_str(yaml).unwrap(),
            doc: String::new(),
            parse_issue: None,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
            about_notes: Default::default(),
        }
    }

    fn codes(elements: &[RawElement]) -> Vec<&'static str> {
        let resolver = Resolver::new(elements);
        let mut c: Vec<_> = unresolved_structural_ref_findings(elements, &resolver, &ValidateConfig::default())
            .into_iter()
            .map(|f| f.code)
            .collect();
        c.sort();
        c
    }

    #[test]
    fn every_field_reports_its_own_code() {
        let els = vec![
            elem("P", "type: Package"),
            elem(
                "P::W",
                "type: PartDef\nsupertype: Nope::A\nfeatures:\n  - name: f\n    typedBy: Nope::F\n",
            ),
            elem(
                "P::w",
                "type: Part\ntypedBy: Nope::D\nsubsets: [Nope::S]\nredefines: Nope::R\nsatisfies: [REQ-NOPE-001]\n",
            ),
        ];
        assert_eq!(codes(&els), vec!["E110", "E111", "E111", "E112", "E113", "E114"]);
    }

    #[test]
    fn scoped_sibling_feature_import_alias_and_library_refs_resolve() {
        let els = vec![
            elem("", "type: Package\nimports:\n  - ISQ::*\n"),
            elem("P", "type: Package\naliases:\n  - name: EDef\n    for: P::Q::Engine\n"),
            elem("P::Q", "type: Package"),
            elem(
                "P::Q::Engine",
                "type: PartDef\nsupertype: Parts::Part\nfeatures:\n  - name: mass\n    typedBy: ISQ::MassValue\n  - name: n\n    typedBy: ScalarValues::Real\n",
            ),
            elem("P::Q::Turbo", "type: PartDef\nsupertype: Engine\nfeatures:\n  - name: t\n    typedBy: TorqueValue\n"),
            elem("P::e", "type: Part\ntypedBy: EDef\nsubsets: [./f]\nredefines: P::Q::Engine::mass\n"),
            elem("P::f", "type: Part\ntypedBy: P::Q::Engine\n"),
            elem("P::f::g", "type: Part\nredefines: mass\n"),
            elem("P::link", "type: Connection\ntypedBy: Link\n"),
        ];
        assert!(codes(&els).is_empty(), "{:?}", codes(&els));
    }

    #[test]
    fn model_package_shadows_a_library_package_name() {
        // A model that declares its own top-level `Requirements` package gets
        // no library leniency for `Requirements::…`.
        let els = vec![
            elem("Requirements", "type: Package"),
            elem("X", "type: PartDef\nsupertype: Requirements::Missing\n"),
        ];
        assert_eq!(codes(&els), vec!["E110"]);
    }

    #[test]
    fn inherited_feature_through_supertype_chain_resolves() {
        let els = vec![
            elem("A", "type: PartDef\nfeatures:\n  - name: speed\n"),
            elem("B", "type: PartDef\nsupertype: A\n"),
            elem("C", "type: PartDef\nsupertype: B\n"),
            elem("C::s", "type: Part\nredefines: speed\n"),
            elem("D", "type: PartDef\nsupertype: E\n"),
            elem("E", "type: PartDef\nsupertype: D\n"),
            elem("E::x", "type: Part\nredefines: nothing\n"),
        ];
        // The D/E supertype cycle terminates and still reports the missing feature.
        assert_eq!(codes(&els), vec!["E113"]);
    }

    #[test]
    fn library_wildcard_import_does_not_excuse_a_model_namespace_reference() {
        // GH #198: `imports: [ISQ::*]` must not silence unresolved references
        // into the model's own packages.
        let els = vec![
            elem("P", "type: Package\nimports:\n  - ISQ::*\n"),
            elem("P::A", "type: PartDef\nsupertype: P::Nope\n"),
            elem("P::B", "type: PartDef\nfeatures:\n  - name: m\n    typedBy: MassValue\n"),
        ];
        assert_eq!(codes(&els), vec!["E110"]);
    }

    fn typed_by_under_import(import: &str, name: &str) -> Vec<&'static str> {
        let els = vec![
            elem("P", &format!("type: Package\nimports:\n  - {import}\n")),
            elem("P::A", &format!("type: PartDef\nfeatures:\n  - name: m\n    typedBy: {name}\n")),
        ];
        codes(&els)
    }

    #[test]
    fn gh210_unknown_bare_name_under_complete_library_wildcard_is_reported() {
        // Known members stay clean (real ISQ/SI/ScalarValues/Base names).
        for (imp, n) in [
            ("ISQ::*", "MassValue"),
            ("ISQ::*", "TemperatureDifferenceValue"),
            ("ISQ::*", "ElectricPotentialDifferenceValue"),
            ("ISQ::**", "LengthValue"),
            ("SI::*", "kilogram"),
            ("SI::*", "MassValue"),
            ("SI::*", "kilo"),
            ("SIPrefixes::*", "milli"),
            ("ScalarValues::*", "Positive"),
            ("Base::*", "Anything"),
            ("ISQMechanics::*", "ForceValue"),
        ] {
            let c = typed_by_under_import(imp, n);
            assert!(c.is_empty(), "{imp} {n}: {c:?}");
        }
        // Unknown names are now E111.
        for (imp, n) in [
            ("ISQ::*", "NoSuchQuantityValue"),
            ("ISQ::**", "NoSuchQuantityValue"),
            ("SI::*", "furlongs"),
            ("SIPrefixes::*", "MassValue"),
            ("ScalarValues::*", "Floaty"),
            ("Base::*", "Nothing"),
        ] {
            assert_eq!(typed_by_under_import(imp, n), vec!["E111"], "{imp} {n}");
        }
    }

    #[test]
    fn gh210_membership_import_of_unknown_library_member_is_reported() {
        assert!(typed_by_under_import("ISQ::MassValue", "MassValue").is_empty());
        assert_eq!(typed_by_under_import("ISQ::Bogus", "Bogus"), vec!["E111"]);
    }

    #[test]
    fn gh210_unenumerated_library_wildcards_stay_lenient() {
        // Packages whose membership is not tabulated keep the old leniency.
        assert!(typed_by_under_import("Parts::*", "Part").is_empty());
        assert!(typed_by_under_import("Parts::*", "Anything").is_empty());
        assert!(typed_by_under_import("Actions::*", "Whatever").is_empty());
    }

    #[test]
    fn gh210_model_package_named_isq_keeps_old_behaviour() {
        let els = vec![
            elem("ISQ", "type: Package"),
            elem("ISQ::Mine", "type: PartDef"),
            elem("P", "type: Package\nimports:\n  - ISQ::*\n"),
            elem("P::A", "type: PartDef\nsupertype: Mine\n"),
        ];
        assert!(codes(&els).is_empty(), "{:?}", codes(&els));
    }

    #[test]
    fn gh210_library_member_tables_cover_the_standard_library() {
        use crate::units::library_member_known as k;
        assert_eq!(k("ISQ", "MassValue"), Some(true));
        assert_eq!(k("ISQ", "Zzz"), Some(false));
        assert_eq!(k("SI", "N"), Some(true));
        assert_eq!(k("SI", "Zzz"), Some(false));
        assert_eq!(k("ScalarValues", "Positive"), Some(true));
        assert_eq!(k("Parts", "Part"), None);
        assert!(crate::units::isq_name_known("ISQ::ElectricPotentialDifferenceValue") == Some(true));
    }

    fn behavior_codes(elements: &[RawElement]) -> Vec<&'static str> {
        let resolver = Resolver::new(elements);
        let mut c: Vec<_> = behavior_ref_findings(elements, &resolver, &ValidateConfig::default())
            .into_iter()
            .map(|f| f.code)
            .collect();
        c.sort();
        c
    }

    #[test]
    fn behavior_fields_resolve_and_valid_model_is_clean() {
        let els = vec![
            elem("B", "type: Package"),
            elem("B::Move", "type: ActionDef"),
            elem("T", "type: Package"),
            elem("T::Sig", "type: ItemDef"),
            elem("S", "type: PartDef\nfeatures:\n  - {name: port, type: Port}\n"),
            elem(
                "B::Serve",
                "type: ActionDef\nparameters:\n  - {name: p, typedBy: T::Sig, direction: in}\n\
                 subActions:\n  - {name: a, kind: PerformAction, typedBy: B::Move}\n  - {name: s, kind: SendAction, payload: T::Sig, via: port}\n  - name: l\n    kind: LoopAction\n    loopKind: while\n    body:\n      - {name: inner, kind: Action}\n\
                 controlNodes:\n  - {name: f, kind: ForkNode}\nsuccessionConnections:\n  - {after: a, before: f}\n  - {after: f, before: inner}\nbindingConnections:\n  - {left: a.x, right: p}\n",
            ),
            elem("B::UC", "type: UseCaseDef\nsubject: S\nincludes: [B::Move]\nextends:\n  - {target: B::Move}\n"),
        ];
        assert!(behavior_codes(&els).is_empty(), "{:?}", behavior_codes(&els));
    }

    #[test]
    fn dangling_behavior_references_are_e115() {
        let els = vec![
            elem(
                "A",
                "type: ActionDef\nparameters:\n  - {name: p, typedBy: Nope::P, direction: in}\nsubActions:\n  - {name: a, typedBy: Nope::X}\n  - {name: s, kind: SendAction, payload: Nope::Y}\n",
            ),
            elem("U", "type: UseCaseDef\nsubject: Nope::S\nincludes: [Nope::I]\nextends:\n  - {target: Nope::T}\nresult: Nope::R\n"),
        ];
        assert_eq!(behavior_codes(&els), vec!["E115"; 7]);
    }

    #[test]
    fn dangling_succession_and_binding_endpoints_are_e116() {
        let els = vec![elem(
            "A",
            "type: ActionDef\nsubActions:\n  - {name: a}\nsuccessionConnections:\n  - {after: a, before: ghost}\nbindingConnections:\n  - {left: nothing.x, right: a}\n",
        )];
        assert_eq!(behavior_codes(&els), vec!["E116", "E116"]);
    }

    #[test]
    fn closed_vocabularies_are_e117() {
        let els = vec![elem(
            "A",
            "type: ActionDef\nparameters:\n  - {name: p, direction: sideways}\nsubActions:\n  - {name: a, kind: Bogus}\n  - {name: l, kind: LoopAction, loopKind: whilst}\ncontrolNodes:\n  - {name: n, kind: Flibber}\n",
        )];
        assert_eq!(behavior_codes(&els), vec!["E117"; 4]);
    }

    #[test]
    fn part_binding_connections_are_not_behavior_endpoints() {
        let els = vec![elem("P", "type: PartDef\nbindingConnections:\n  - {left: a.x, right: y}\n")];
        assert!(behavior_codes(&els).is_empty());
    }

    #[test]
    fn imports_aliases_depends_on_and_inline_redefinitions_are_checked() {
        // GH #201.
        let els = vec![
            elem("P", "type: Package\nimports:\n  - Nowhere::*\n  - P::Q\naliases:\n  - {name: Z, for: P::Nothing}\n  - {name: Y, for: P::Q}\n"),
            elem("P::Q", "type: PartDef\nfeatures:\n  - {name: mass}\n"),
            elem(
                "P::R",
                "type: PartDef\nsupertype: P::Q\ndependsOn: [P::Missing, P::Q]\nfeatures:\n  - {name: mass, redefines: mass}\n  - {name: bad, redefines: nope}\n  - {name: s, subsets: ghost}\n",
            ),
        ];
        assert_eq!(codes(&els), vec!["E112", "E113", "E126", "E126", "E126"]);
    }

    #[test]
    fn connection_endpoint_chains_are_walked() {
        let els = vec![
            elem("PD", "type: PortDef\nfeatures:\n  - {name: tx}\n"),
            elem("Eng", "type: PartDef\nfeatures:\n  - {name: out, type: Port, typedBy: PD}\n"),
            elem(
                "Sys",
                "type: PartDef\nfeatures:\n  - {name: e, type: Part, typedBy: Eng}\nconnections:\n  - {from: e.out.tx, to: e.out}\n  - {from: e.nope, to: e.out}\n  - {from: ghost.p, to: e.out}\n  - {from: e.out.zz, to: e.out}\n",
            ),
        ];
        assert_eq!(codes(&els), vec!["E127", "W056", "W056"]);
    }

    fn behavior_codes_of(els: &[RawElement]) -> Vec<&'static str> {
        let resolver = Resolver::new(els);
        let mut c: Vec<_> = behavior_ref_findings(els, &resolver, &ValidateConfig::default())
            .into_iter()
            .map(|f| f.code)
            .collect();
        c.sort();
        c
    }

    #[test]
    fn flow_and_succession_endpoints_on_structural_elements_are_walked() {
        // GH #207.
        let els = vec![
            elem("PD", "type: PortDef\nfeatures:\n  - {name: tx}\n"),
            elem("Eng", "type: PartDef\nfeatures:\n  - {name: out, type: Port, typedBy: PD}\n"),
            elem(
                "Sys",
                "type: PartDef\nfeatures:\n  - {name: e, type: Part, typedBy: Eng}\nflowConnections:\n  - {from: e.out, to: e.out.tx}\n  - {from: ghost.p, to: e.out}\n  - {from: e.out, to: e.nope}\nsuccessionConnections:\n  - {after: e, before: e}\n  - {after: ghost, before: e}\n",
            ),
        ];
        assert_eq!(codes(&els), vec!["E127", "E127", "W056"]);
    }

    #[test]
    fn behavioral_flow_endpoints_are_left_to_e116() {
        let els = vec![elem(
            "A",
            "type: ActionDef\nsubActions:\n  - {name: a}\nflowConnections:\n  - {from: a, to: ghost}\nsuccessionConnections:\n  - {after: a, before: ghost}\n",
        )];
        assert!(codes(&els).is_empty());
    }

    #[test]
    fn send_accept_via_and_to_chains_are_e128() {
        let els = vec![
            elem("Ctl", "type: PartDef\nfeatures:\n  - {name: cmdPort, type: Port}\n"),
            elem(
                "Act",
                "type: ActionDef\nsubActions:\n  - {name: s1, kind: SendAction, via: cmdPort, to: Ctl}\n  - {name: s2, kind: SendAction, via: cmdPort.x, to: nobody}\n  - {name: a1, kind: AcceptAction, via: ghostPort}\n  - {name: a2, kind: AcceptAction, via: cmdPort}\n  - name: i\n    kind: IfAction\n    condition: c\n    then:\n      - {name: s3, kind: SendAction, via: missing}\n  - {name: p, kind: PerformAction, via: ignoredHere}\n",
            ),
        ];
        // s2.to=nobody, a1.via=ghostPort, s3.via=missing; `cmdPort.x` has a known head.
        assert_eq!(behavior_codes_of(&els), vec!["E128", "E128", "E128"]);
    }

    #[test]
    fn transition_accept_via_is_checked() {
        let els = vec![
            elem("P", "type: PartDef\nfeatures:\n  - {name: ctl, type: Port}\n"),
            elem(
                "SM",
                "type: StateDef\nsubStates:\n  - name: off\n    transitions:\n      - {target: on, accept: {payload: X, via: ctl}}\n      - {target: on, accept: {payload: X, via: nowhere}}\n  - name: on\ntransitions:\n  - {source: on, target: off, accept: {payload: X, via: alsoMissing}}\n",
            ),
        ];
        assert_eq!(behavior_codes_of(&els), vec!["E128", "E128"]);
    }

    #[test]
    fn via_resolves_to_a_port_element_by_qualified_name() {
        let els = vec![
            elem("Pk", "type: Package"),
            elem("Pk::Port1", "type: Port"),
            elem("Act", "type: ActionDef\nsubActions:\n  - {name: s, kind: SendAction, via: Pk::Port1}\n"),
        ];
        assert!(behavior_codes_of(&els).is_empty());
    }

    #[test]
    fn private_elements_are_not_referenced_from_outside_their_package() {
        let els = vec![
            elem("A", "type: Package"),
            elem("A::Hidden", "type: PartDef\nvisibility: private"),
            elem("A::Inside", "type: PartDef\nsupertype: A::Hidden"),
            elem("B", "type: Package"),
            elem("B::Outside", "type: PartDef\nsupertype: A::Hidden"),
        ];
        let resolver = Resolver::new(&els);
        let w: Vec<_> = unresolved_structural_ref_findings(&els, &resolver, &ValidateConfig::default())
            .into_iter()
            .filter(|f| f.code == "W059")
            .collect();
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].file.contains("Outside"));
    }
}
