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
use crate::element::RawElement;
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
            || crate::units::is_recognised_type_ref(r)
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

    /// Whether one `imports:` entry `target`, declared in namespace `ns`, makes
    /// the reference `r` (whose first segment is `head`) visible.
    fn import_provides(&self, ns: &str, target: &str, head: &str, r: &str) -> bool {
        let target = target.trim();
        if let Some(pkg) = target.strip_suffix("::**") {
            if self.is_library_ref(&format!("{pkg}::{r}")) || self.is_library_ref(pkg) {
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
            if self.is_library_ref(&full) || self.is_library_ref(pkg) {
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
        self.is_library_ref(&full) || self.is_library_ref(target) || self.scoped(ns, &full).is_some()
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
}
