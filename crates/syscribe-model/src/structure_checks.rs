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

/// "a" or "an" for a type name (`an Attribute`, `a Port`).
fn article(name: &str) -> &'static str {
    if name.starts_with(['A', 'E', 'I', 'O', 'U']) { "an" } else { "a" }
}

/// How to fix an `E123`: the common data-record mistake first (an `Attribute` typed by an
/// `ItemDef`, or the mirror), else the definition kinds that can type the usage (GH #254).
fn e123_remedy(usage: &T, def: &T, def_name: &str, allowed: &[T]) -> String {
    match (usage, def) {
        (T::Attribute, T::ItemDef) => format!(
            "Define '{def_name}' as an `AttributeDef` (a data record or value type); change the usage to `type: Item` only if it is a flowing item (sent/received or carried by a flow)."
        ),
        (T::Item, T::AttributeDef) => format!(
            "Define '{def_name}' as an `ItemDef` if it is something that flows, or change the usage to `type: Attribute` if it is a data record or value."
        ),
        _ => {
            let names: Vec<String> = allowed.iter().map(|t| format!("{t:?}")).collect();
            format!(
                "{} {usage:?} must be typed by one of: {}.",
                if article(&format!("{usage:?}")) == "an" { "An" } else { "A" },
                names.join(", ")
            )
        }
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

/// A parsed multiplicity: `(lower, upper)`, `upper == None` meaning unbounded.
/// `None` for malformed text or a non-numeric (named / expression) bound, which
/// cannot be compared statically.
fn parse_multiplicity(text: &str) -> Option<(u64, Option<u64>)> {
    let t = text.trim();
    let t = t.strip_prefix('[').and_then(|x| x.strip_suffix(']')).unwrap_or(t).trim();
    let num = |b: &str| b.trim().parse::<u64>().ok();
    let parts: Vec<&str> = t.split("..").collect();
    match parts.as_slice() {
        ["*"] => Some((0, None)),
        [n] => num(n).map(|n| (n, Some(n))),
        [lo, hi] => {
            let lo = num(lo)?;
            if hi.trim() == "*" {
                Some((lo, None))
            } else {
                Some((lo, Some(num(hi)?)))
            }
        }
        _ => None,
    }
}

/// Whether `inner` is contained in `outer` (a redefinition may only narrow).
fn mult_contained(inner: (u64, Option<u64>), outer: (u64, Option<u64>)) -> bool {
    inner.0 >= outer.0
        && match (inner.1, outer.1) {
            (_, None) => true,
            (None, Some(_)) => false,
            (Some(i), Some(o)) => i <= o,
        }
}

/// The declared `multiplicity:` of the member `name` of `owner`, found as a
/// child element, an inline `features:` entry (both only when `include_self`),
/// or through the owner's `supertype:`/`typedBy:` chain. `Some(None)` = member
/// found but declares no multiplicity. Cycle-safe.
fn member_multiplicity(
    elements: &[RawElement],
    resolver: &Resolver,
    owner: &RawElement,
    name: &str,
    include_self: bool,
    seen: &mut Vec<String>,
) -> Option<Option<String>> {
    if include_self {
        if seen.contains(&owner.qualified_name) {
            return None;
        }
        seen.push(owner.qualified_name.clone());
        let child = if owner.qualified_name.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", owner.qualified_name, name)
        };
        if let Some(c) = resolver.get(elements, &child) {
            return Some(c.frontmatter.multiplicity.clone());
        }
        for f in owner.frontmatter.features.iter().flatten() {
            if let serde_yaml::Value::Mapping(m) = f {
                if yaml_str(m, "name") == Some(name) {
                    return Some(yaml_str(m, "multiplicity").map(str::to_string));
                }
            }
        }
    }
    let fm = &owner.frontmatter;
    for r in fm.supertype.iter().chain(fm.typed_by.iter()).flat_map(yaml_strings) {
        if let Some(base) = resolver.resolve_scoped_ref(elements, &owner.qualified_name, r) {
            if let Some(found) = member_multiplicity(elements, resolver, base, name, true, seen) {
                return Some(found);
            }
        }
    }
    None
}

/// The multiplicity of the feature a `redefines:` reference names, if it can be
/// determined: `Owner::feat`, or a bare name inherited by `scope` (the owner
/// whose supertype chain is searched).
fn redefined_multiplicity(
    elements: &[RawElement],
    resolver: &Resolver,
    scope: &RawElement,
    include_scope_self: bool,
    r: &str,
) -> Option<String> {
    let r = r.trim();
    if let Some((owner, feat)) = r.rsplit_once("::") {
        let o = resolver.resolve_scoped_ref(elements, &scope.qualified_name, owner)?;
        return member_multiplicity(elements, resolver, o, feat, true, &mut Vec::new())?;
    }
    member_multiplicity(elements, resolver, scope, r, include_scope_self, &mut Vec::new())?
}

/// W068 — see the validation catalogue.
fn redefinition_multiplicity_finding(
    file: &str,
    what: &str,
    r: &str,
    own: &str,
    base: &str,
) -> Option<Finding> {
    let (o, b) = (parse_multiplicity(own)?, parse_multiplicity(base)?);
    if mult_contained(o, b) {
        return None;
    }
    Some(warning(
        "W068",
        file,
        format!("{what} has multiplicity '{own}' which is not contained in the multiplicity '{base}' of the feature it redefines ('{r}'); a redefinition may only narrow the redefined multiplicity"),
    ))
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
        // W068 — a redefining feature's multiplicity must lie within the redefined one's.
        if let Some(own) = fm.multiplicity.as_deref() {
            if let Some(owner) = parent_of(q).and_then(|p| resolver.get(elements, p)) {
                for r in fm.redefines.iter().flat_map(yaml_strings) {
                    // Search the owner's supertype chain first; the owner's own declaration of
                    // the name is only a fallback (the element may be that very declaration).
                    let base = redefined_multiplicity(elements, resolver, owner, false, r)
                        .or_else(|| redefined_multiplicity(elements, resolver, owner, true, r));
                    if let Some(base) = base {
                        out.extend(redefinition_multiplicity_finding(file, "element", r, own, &base));
                    }
                }
            }
        }
        for f in fm.features.iter().flatten() {
            let serde_yaml::Value::Mapping(m) = f else { continue };
            let (Some(own), Some(rv)) = (
                yaml_str(m, "multiplicity"),
                m.get(serde_yaml::Value::String("redefines".into())),
            ) else {
                continue;
            };
            let n = yaml_str(m, "name").unwrap_or("?");
            for r in yaml_strings(rv) {
                // Inline feature: the redefined feature is inherited, never the entry itself.
                if let Some(base) = redefined_multiplicity(elements, resolver, elem, false, r) {
                    out.extend(redefinition_multiplicity_finding(
                        file,
                        &format!("inline feature '{n}'"),
                        r,
                        own,
                        &base,
                    ));
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
                        format!(
                            "{ctx} is typed by '{r}', {} {tt:?}, which cannot type {} {ty:?}. {}",
                            article(&format!("{tt:?}")),
                            article(&format!("{ty:?}")),
                            e123_remedy(ty, tt, r, allowed)
                        ),
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
                    format!(
                        "inline feature '{n}' ({kind:?}) is typed by '{tb}', {} {tt:?}, which cannot type it. {}",
                        article(&format!("{tt:?}")),
                        e123_remedy(&kind, tt, tb, allowed)
                    ),
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

    fn w068(elements: &[RawElement]) -> usize {
        let resolver = Resolver::new(elements);
        structure_findings(elements, &resolver).iter().filter(|f| f.code == "W068").count()
    }

    #[test]
    fn multiplicity_parsing_forms() {
        assert_eq!(parse_multiplicity("1"), Some((1, Some(1))));
        assert_eq!(parse_multiplicity("[3]"), Some((3, Some(3))));
        assert_eq!(parse_multiplicity("0..1"), Some((0, Some(1))));
        assert_eq!(parse_multiplicity("[1..*]"), Some((1, None)));
        assert_eq!(parse_multiplicity("*"), Some((0, None)));
        assert_eq!(parse_multiplicity(" 2 .. 4 "), Some((2, Some(4))));
        assert_eq!(parse_multiplicity("0..maxN"), None);
        assert_eq!(parse_multiplicity("abc def"), None);
    }

    #[test]
    fn multiplicity_containment_table() {
        let c = |a: &str, b: &str| mult_contained(parse_multiplicity(a).unwrap(), parse_multiplicity(b).unwrap());
        assert!(c("1", "1"));
        assert!(c("1", "0..1"));
        assert!(c("1", "*"));
        assert!(c("1..*", "*"));
        assert!(c("2..4", "1..*"));
        assert!(c("[2]", "1..3"));
        assert!(!c("0..1", "1"));
        assert!(!c("*", "1..*"));
        assert!(!c("1..*", "1"));
        assert!(!c("0..2", "1..3"));
        assert!(!c("2", "1"));
        assert!(!c("0", "1..*"));
    }

    #[test]
    fn inline_feature_redefinition_widening_is_w068() {
        let els = vec![
            elem("A", "type: PartDef\nfeatures:\n  - {name: wheel, multiplicity: \"1..2\"}\n  - {name: free}\n"),
            elem("B", "type: PartDef\nsupertype: A\nfeatures:\n  - {name: w, redefines: wheel, multiplicity: \"0..2\"}\n"),
            elem("C", "type: PartDef\nsupertype: B\nfeatures:\n  - {name: w2, redefines: wheel, multiplicity: \"*\"}\n"),
        ];
        // B widens the lower bound; C inherits `wheel` through B's chain and widens the upper.
        assert_eq!(w068(&els), 2);
    }

    #[test]
    fn inline_feature_redefinition_narrowing_is_clean() {
        let els = vec![
            elem("A", "type: PartDef\nfeatures:\n  - {name: wheel, multiplicity: \"*\"}\n  - {name: seat, multiplicity: \"[1..4]\"}\n"),
            elem(
                "B",
                "type: PartDef\nsupertype: A\nfeatures:\n  - {name: w, redefines: wheel, multiplicity: \"4\"}\n  - {name: s, redefines: seat, multiplicity: \"2..3\"}\n  - {name: same, redefines: seat, multiplicity: \"[1..4]\"}\n",
            ),
        ];
        assert_eq!(w068(&els), 0);
    }

    #[test]
    fn redefinition_without_a_declared_multiplicity_on_either_side_is_silent() {
        let els = vec![
            elem("A", "type: PartDef\nfeatures:\n  - {name: wheel}\n  - {name: seat, multiplicity: \"1\"}\n"),
            elem(
                "B",
                "type: PartDef\nsupertype: A\nfeatures:\n  - {name: w, redefines: wheel, multiplicity: \"5\"}\n  - {name: s, redefines: seat}\n  - {name: u, redefines: unknownthing, multiplicity: \"9\"}\n",
            ),
        ];
        assert_eq!(w068(&els), 0);
    }

    #[test]
    fn named_bounds_are_not_compared() {
        let els = vec![
            elem("A", "type: PartDef\nfeatures:\n  - {name: wheel, multiplicity: \"0..maxN\"}\n"),
            elem("B", "type: PartDef\nsupertype: A\nfeatures:\n  - {name: w, redefines: wheel, multiplicity: \"7\"}\n"),
        ];
        assert_eq!(w068(&els), 0);
    }

    #[test]
    fn element_level_redefinition_is_checked() {
        let els = vec![
            elem("A", "type: PartDef\nfeatures:\n  - {name: speed, multiplicity: \"1\"}\n"),
            elem("B", "type: PartDef\nsupertype: A\n"),
            elem("B::s", "type: Part\nredefines: speed\nmultiplicity: \"0..1\"\n"),
            elem("B::ok", "type: Part\nredefines: speed\nmultiplicity: \"1\"\n"),
            elem("B::none", "type: Part\nredefines: speed\n"),
        ];
        assert_eq!(w068(&els), 1);
    }

    #[test]
    fn element_level_redefinition_of_a_qualified_feature() {
        let els = vec![
            elem("A", "type: PartDef\nfeatures:\n  - {name: speed, multiplicity: \"1..2\"}\n"),
            elem("A::file", "type: Part\nmultiplicity: \"0..1\"\n"),
            elem("P", "type: Package"),
            elem("P::x", "type: Part\nredefines: A::speed\nmultiplicity: \"3\"\n"),
            elem("P::y", "type: Part\nredefines: A::file\nmultiplicity: \"1..*\"\n"),
            elem("P::z", "type: Part\nredefines: A::file\nmultiplicity: \"[0]\"\n"),
        ];
        // x widens 1..2 to 3; y widens 0..1 to 1..*; z (0) is inside 0..1.
        assert_eq!(w068(&els), 2);
    }

    #[test]
    fn redefinition_cycles_terminate() {
        let els = vec![
            elem("D", "type: PartDef\nsupertype: E\n"),
            elem("E", "type: PartDef\nsupertype: D\nfeatures:\n  - {name: x, redefines: nothing, multiplicity: \"2\"}\n"),
        ];
        assert_eq!(w068(&els), 0);
    }
}
