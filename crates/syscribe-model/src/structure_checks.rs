//! Structural-semantics checks the format spec (§11.7) requires of a conformant
//! validator (GH #200):
//!
//! * `E118` — malformed or reversed `multiplicity:`;
//! * `E119` — `visibility:` other than `public`/`protected`/`private`;
//! * `E120` — `supertype:` on a usage (a usage is typed with `typedBy:`);
//! * `E121` — `isVariant: true` on an element not owned by a variation;
//! * `E122` — `EnumerationDef` without `values:`, or specializing another
//!   `EnumerationDef`;
//! * `E123` — a usage typed by a definition of the wrong kind, or
//!   `conjugates:` naming something that is not a `PortDef`;
//! * `E124` — an enumeration literal `Enum::lit` that the enumeration lacks;
//! * `E125` — an `InterfaceDef`/`ConnectionDef` that declares fewer than two `ends:`.
//!
//! Only authored Markdown elements are checked; ingested `.sysml`/`.kerml`
//! members have their own diagnostics (`W544`, …).

use crate::element::{ElementType as T, RawElement};
use crate::resolver::Resolver;
use crate::validator::{Finding, Severity};

fn warning(code: &'static str, file: &str, msg: String) -> Finding {
    Finding { code, file: file.to_string(), message: msg, severity: Severity::Warning }
}

fn error(code: &'static str, file: &str, msg: String) -> Finding {
    Finding { code, file: file.to_string(), message: msg, severity: Severity::Error }
}

/// Why a multiplicity text is malformed, if it is. A bound is a natural
/// number, `*`, or an identifier / feature chain (SysML allows expressions).
fn multiplicity_problem(text: &str) -> Option<String> {
    let t = text.trim();
    let t = t.strip_prefix('[').and_then(|x| x.strip_suffix(']')).unwrap_or(t).trim();
    if t.is_empty() {
        return Some("is empty".into());
    }
    let bound_ok = |b: &str| {
        let b = b.trim();
        b == "*"
            || (!b.is_empty() && b.chars().all(|c| c.is_ascii_digit()))
            || (b.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                && b.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '.')))
    };
    let parts: Vec<&str> = t.split("..").collect();
    match parts.as_slice() {
        [b] if bound_ok(b) => None,
        [lo, hi] if bound_ok(lo) && bound_ok(hi) => {
            if let (Ok(l), Ok(h)) = (lo.trim().parse::<u64>(), hi.trim().parse::<u64>()) {
                if l > h {
                    return Some(format!("has lower bound {l} above upper bound {h}"));
                }
            }
            if lo.trim() == "*" {
                return Some("has `*` as its lower bound".into());
            }
            None
        }
        _ => Some("is not `N`, `N..M`, `N..*` or `*` (bounds are natural numbers or names)".into()),
    }
}

fn usage_expected_defs(t: &T) -> Option<&'static [T]> {
    Some(match t {
        T::Part | T::Item | T::Individual | T::Occurrence => {
            &[T::PartDef, T::ItemDef, T::OccurrenceDef, T::IndividualDef, T::EventOccurrenceDef]
        }
        T::Port => &[T::PortDef],
        T::Attribute => &[T::AttributeDef, T::EnumerationDef],
        T::Enumeration => &[T::EnumerationDef],
        T::Connection => &[T::ConnectionDef, T::InterfaceDef],
        T::Interface => &[T::InterfaceDef],
        T::Action => &[T::ActionDef],
        T::State | T::ExhibitState => &[T::StateDef],
        // A calculation/constraint usage may be typed by the part it is evaluated
        // in (the demo model's context-part idiom), besides its own definition.
        T::Constraint => &[T::ConstraintDef, T::PartDef, T::ItemDef],
        T::Calculation => &[T::CalculationDef, T::PartDef, T::ItemDef],
        T::UseCase => &[T::UseCaseDef],
        T::View => &[T::ViewDef],
        T::Metadata => &[T::MetadataDef],
        T::Allocation => &[T::AllocationDef],
        _ => return None,
    })
}

fn is_usage(t: &T) -> bool {
    usage_expected_defs(t).is_some()
}

fn type_from_name(s: &str) -> Option<T> {
    serde_yaml::from_str::<T>(s).ok().filter(|t| *t != T::Unknown)
}

fn yaml_str<'a>(m: &'a serde_yaml::Mapping, k: &str) -> Option<&'a str> {
    m.get(serde_yaml::Value::String(k.to_string())).and_then(|v| v.as_str())
}

fn yaml_strings(v: &serde_yaml::Value) -> Vec<&str> {
    match v {
        serde_yaml::Value::String(s) => vec![s.as_str()],
        serde_yaml::Value::Sequence(seq) => seq.iter().filter_map(|x| x.as_str()).collect(),
        _ => vec![],
    }
}

fn parent_of(q: &str) -> Option<&str> {
    q.rsplit_once("::").map(|(p, _)| p)
}

pub fn structure_findings(elements: &[RawElement], resolver: &Resolver) -> Vec<Finding> {
    let mut out = Vec::new();
    for elem in elements {
        if !elem.file_path.ends_with(".md") {
            continue;
        }
        let fm = &elem.frontmatter;
        let file = elem.file_path.as_str();
        let q = elem.qualified_name.as_str();
        let Some(ty) = fm.element_type.as_ref() else { continue };

        // W058 — a name-identified element's qualified name is its path, so a
        // `name:` that differs from the file stem (or directory) is never what
        // references must use; flag the silent mismatch (GH #201).
        if !ty.is_id_identified() && !matches!(ty, T::FeatureModel | T::FMEASheet) {
            if let (Some(n), Some(seg)) = (fm.name.as_deref(), q.rsplit("::").next()) {
                if !seg.is_empty() && n != seg && !n.contains(' ') && !n.starts_with('<') {
                    out.push(warning(
                        "W058",
                        file,
                        format!("`name: {n}` differs from the file name '{seg}' that forms the qualified name '{q}' — references must use '{seg}'; rename the file or the `name:`"),
                    ));
                }
            }
        }
        if let Some(m) = &fm.multiplicity {
            if let Some(why) = multiplicity_problem(m) {
                out.push(error("E118", file, format!("multiplicity '{m}' {why}")));
            }
        }
        for f in fm.features.iter().flatten() {
            let serde_yaml::Value::Mapping(fm_) = f else { continue };
            if let Some(m) = yaml_str(fm_, "multiplicity") {
                if let Some(why) = multiplicity_problem(m) {
                    let n = yaml_str(fm_, "name").unwrap_or("?");
                    out.push(error("E118", file, format!("multiplicity '{m}' on inline feature '{n}' {why}")));
                }
            }
        }
        if let Some(v) = &fm.visibility {
            if !matches!(v.as_str(), "public" | "protected" | "private") {
                out.push(error("E119", file, format!("visibility '{v}' is not public, protected or private")));
            }
        }
        if is_usage(ty) && fm.supertype.is_some() {
            out.push(error(
                "E120",
                file,
                format!("a {ty:?} is a usage — type it with `typedBy:` (and specialize with `subsets:`), not `supertype:`"),
            ));
        }
        if fm.is_variant == Some(true) {
            let owned = parent_of(q)
                .and_then(|p| resolver.get(elements, p))
                .is_some_and(|p| p.frontmatter.is_variation == Some(true));
            let via_variant_of = fm
                .variant_of
                .as_deref()
                .and_then(|v| resolver.resolve_scoped_ref(elements, q, v))
                .is_some_and(|p| p.frontmatter.is_variation == Some(true));
            if !owned && !via_variant_of {
                out.push(error(
                    "E121",
                    file,
                    "`isVariant: true` on an element that is neither a member of a variation (`isVariation: true`) nor names one in `variantOf:`".into(),
                ));
            }
        }
        if *ty == T::EnumerationDef {
            if fm.values.as_ref().is_none_or(|v| v.is_empty()) {
                out.push(error("E122", file, "an EnumerationDef must list its literals in `values:`".into()));
            }
            for s in fm.supertype.iter().flat_map(yaml_strings) {
                if resolver
                    .resolve_scoped_ref(elements, q, s)
                    .is_some_and(|t| t.frontmatter.element_type == Some(T::EnumerationDef))
                {
                    out.push(error(
                        "E122",
                        file,
                        format!("an EnumerationDef cannot specialize another EnumerationDef ('{s}')"),
                    ));
                }
            }
        }
        if let Some(allowed) = usage_expected_defs(ty) {
            let mut check = |r: &str, ctx: &str| {
                let Some(t) = resolver.resolve_scoped_ref(elements, q, r) else { return };
                let Some(tt) = t.frontmatter.element_type.as_ref() else { return };
                let is_def = format!("{tt:?}").ends_with("Def");
                if is_def && !allowed.contains(tt) {
                    out.push(error(
                        "E123",
                        file,
                        format!("{ctx} is typed by '{r}', a {tt:?}, which cannot type a {ty:?}"),
                    ));
                }
            };
            for r in fm.typed_by.iter().flat_map(yaml_strings) {
                check(r, "usage");
            }
        }
        for f in fm.features.iter().flatten() {
            let serde_yaml::Value::Mapping(m) = f else { continue };
            let (Some(kind), Some(tb)) = (yaml_str(m, "type").and_then(type_from_name), yaml_str(m, "typedBy")) else {
                continue;
            };
            let Some(allowed) = usage_expected_defs(&kind) else { continue };
            let Some(t) = resolver.resolve_scoped_ref(elements, q, tb) else { continue };
            let Some(tt) = t.frontmatter.element_type.as_ref() else { continue };
            if format!("{tt:?}").ends_with("Def") && !allowed.contains(tt) {
                let n = yaml_str(m, "name").unwrap_or("?");
                out.push(error(
                    "E123",
                    file,
                    format!("inline feature '{n}' ({kind:?}) is typed by '{tb}', a {tt:?}, which cannot type it"),
                ));
            }
        }
        if let Some(c) = &fm.conjugates {
            let ok_owner = *ty == T::PortDef;
            let target_ok = resolver
                .resolve_scoped_ref(elements, q, c)
                .is_none_or(|t| t.frontmatter.element_type == Some(T::PortDef));
            if !ok_owner || !target_ok {
                out.push(error("E123", file, format!("`conjugates: {c}` must be declared on a PortDef and name a PortDef")));
            }
        }
        if matches!(ty, T::InterfaceDef | T::ConnectionDef) {
            if let Some(ends) = &fm.ends {
                if ends.len() < 2 {
                    out.push(error("E125", file, format!("{ty:?} declares {} end(s); a connection needs at least two", ends.len())));
                }
            }
        }
        // E124 — `value: Enum::literal` against the enumeration's literals.
        let value_literal = |v: &serde_yaml::Value| -> Option<String> { v.as_str().map(str::to_string) };
        let mut check_literal = |val: &str| {
            let Some((en, lit)) = val.trim().rsplit_once("::") else { return };
            let Some(e) = resolver.resolve_scoped_ref(elements, q, en) else { return };
            if e.frontmatter.element_type != Some(T::EnumerationDef) {
                return;
            }
            let has = e.frontmatter.values.iter().flatten().any(|v| match v {
                serde_yaml::Value::Mapping(m) => yaml_str(m, "name") == Some(lit),
                serde_yaml::Value::String(s) => s == lit,
                _ => false,
            });
            if !has {
                out.push(error("E124", file, format!("'{lit}' is not a literal of enumeration '{en}'")));
            }
        };
        if let Some(s) = fm.value.as_ref().and_then(value_literal) {
            check_literal(&s);
        }
        for f in fm.features.iter().flatten() {
            if let serde_yaml::Value::Mapping(m) = f {
                if let Some(s) = yaml_str(m, "value") {
                    check_literal(s);
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
            about_notes: Default::default(),
        }
    }

    fn codes(elements: &[RawElement]) -> Vec<&'static str> {
        let resolver = Resolver::new(elements);
        let mut c: Vec<_> = structure_findings(elements, &resolver).into_iter().map(|f| f.code).collect();
        c.sort();
        c
    }

    #[test]
    fn valid_structure_is_clean() {
        let els = vec![
            elem("E", "type: EnumerationDef\nvalues: [{name: a}, {name: b}]"),
            elem("PD", "type: PortDef"),
            elem("Eng", "type: PartDef\nisVariation: true\nmultiplicity: \"0..maxN\""),
            elem("Eng::V6", "type: Part\nisVariant: true\nmultiplicity: \"1..*\"\ntypedBy: Eng"),
            elem("p", "type: Port\ntypedBy: PD\nvisibility: private"),
            elem("x", "type: Part\ntypedBy: Eng\nsubsets: [Eng]\nfeatures:\n  - {name: a, type: Attribute, typedBy: E, value: \"E::a\", multiplicity: \"[2]\"}"),
            elem("I", "type: InterfaceDef\nends: [{name: a}, {name: b}]"),
        ];
        assert!(codes(&els).is_empty(), "{:?}", codes(&els));
    }

    #[test]
    fn each_violation_has_its_code() {
        let els = vec![
            elem("E", "type: EnumerationDef\nvalues: [{name: a}]"),
            elem("PD", "type: PortDef"),
            elem("PartD", "type: PartDef\nmultiplicity: \"5..2\"\nvisibility: secret"),
            elem("U", "type: Part\nsupertype: PartD"),
            elem("V", "type: Part\nisVariant: true"),
            elem("NoVals", "type: EnumerationDef"),
            elem("Sub", "type: EnumerationDef\nvalues: [{name: z}]\nsupertype: E"),
            elem("K", "type: Part\ntypedBy: PD"),
            elem("Conj", "type: PortDef\nconjugates: PartD"),
            elem("Lit", "type: Attribute\ntypedBy: E\nvalue: \"E::nope\""),
            elem("I", "type: InterfaceDef\nends: [{name: a}]"),
            elem("Inl", "type: PartDef\nfeatures:\n  - {name: f, type: Port, typedBy: PartD, multiplicity: \"abc def\"}"),
        ];
        assert_eq!(
            codes(&els),
            vec!["E118", "E118", "E119", "E120", "E121", "E122", "E122", "E123", "E123", "E123", "E124", "E125"]
        );
    }

    #[test]
    fn ingested_sysml_members_are_not_checked() {
        let mut e = elem("S", "type: PartDef\nmultiplicity: \"3..1\"");
        e.file_path = "S.sysml".into();
        assert!(codes(&[e]).is_empty());
    }

    #[test]
    fn name_differing_from_the_file_stem_is_w058() {
        let els = vec![
            elem("Cases::ServeUC", "type: UseCaseDef\nname: RideUC"),
            elem("Cases::Ok", "type: UseCaseDef\nname: Ok"),
            elem("Cases::Prose", "type: UseCaseDef\nname: A free label"),
            elem("Reqs::REQ-001", "type: Requirement\nid: REQ-001\nname: Anything goes"),
        ];
        let resolver = Resolver::new(&els);
        let w: Vec<_> = structure_findings(&els, &resolver).into_iter().filter(|f| f.code == "W058").collect();
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].file.contains("ServeUC"));
    }
}
