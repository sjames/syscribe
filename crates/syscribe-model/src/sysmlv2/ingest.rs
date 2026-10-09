//! Native parsing of `.sysml`/`.kerml` files inside a `sysmlSubmodel: true` subtree
//! into `RawElement`s (`REQ-TRS-SYSMLV2-002`, `REQ-TRS-SYSMLV2-007`).
//!
//! `W541` (parse/read failure) is a **placeholder** code — `REQ-TRS-SYSMLV2-006`
//! formalizes the dedicated error/warning code range for this subsystem later;
//! don't read anything permanent into the exact number yet.
//!
//! **Known residual gap (`REQ-TRS-SYSMLV2-008`'s `@Syscribe*` fixed-field lift):**
//! an annotation member whose value doesn't match any recognized expression
//! shape for that field (e.g. `sil = 2.5;`, a non-integer numeric form) is
//! indistinguishable, downstream, from the annotation not being written at
//! all — no field is lifted and no diagnostic is raised. This mirrors the
//! module's existing parse-broad/map-narrow posture (an unmapped *construct*
//! is silently invisible too, `ADR-SYS-SYSMLV2-001` sub-decision 3) rather
//! than extending it with new validation machinery, but it's a real,
//! user-reachable authoring trap worth knowing about if this set grows.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::derive::finding;
use crate::element::{ElementType, RawElement, RawFrontmatter};

// ---------------------------------------------------------------------------
// Parsed-document access (`REQ-TRS-SYSMLV2-073`).
//
// Since sysml-v2-parser 0.55 every declaration name, short name, literal and qualified reference
// in the AST is a source-span handle that can only be read through the owning
// `ParsedDocument`. The converters below are pure functions of the AST, so rather than add a
// document parameter to ~170 signatures the *current* document is held in a thread-local that
// `with_doc` installs for exactly the extent of one conversion; `dn`/`qr`/`lit_*` read through it.
// Documents are held as `Rc<ParsedDocument>` shells (source + reference arena, empty root) so the
// AST can be moved out while the handles stay resolvable.
// ---------------------------------------------------------------------------

pub(crate) type DocRef = std::rc::Rc<sysml_v2_parser::ParsedDocument>;

thread_local! {
    static CUR_DOC: std::cell::RefCell<Option<DocRef>> = const { std::cell::RefCell::new(None) };
}

/// Split a parsed document into its (movable) root and a shell that still resolves handles.
pub(crate) fn split_document(doc: sysml_v2_parser::ParsedDocument) -> (DocRef, sysml_v2_parser::RootNamespace) {
    let sysml_v2_parser::ParsedDocument { source, qualified_references, root } = doc;
    let shell = sysml_v2_parser::ParsedDocument {
        source,
        qualified_references,
        root: sysml_v2_parser::RootNamespace { elements: Vec::new() },
    };
    (std::rc::Rc::new(shell), root)
}

/// Run `f` with `doc` as the document every handle reader resolves against.
pub(crate) fn with_doc<R>(doc: &DocRef, f: impl FnOnce() -> R) -> R {
    let prev = CUR_DOC.with(|c| c.borrow_mut().replace(doc.clone()));
    let out = f();
    CUR_DOC.with(|c| *c.borrow_mut() = prev);
    out
}

fn with_cur<R>(f: impl FnOnce(&sysml_v2_parser::ParsedDocument) -> R) -> Option<R> {
    CUR_DOC.with(|c| c.borrow().as_ref().map(|d| f(d)))
}

/// Decoded text of a declaration name (quotes removed, `\'` decoded).
pub(crate) fn dn(n: sysml_v2_parser::DeclarationName) -> String {
    with_cur(|d| d.decoded_declaration_name(n).map(|c| c.into_owned())).flatten().unwrap_or_default()
}

pub(crate) fn odn(n: Option<sysml_v2_parser::DeclarationName>) -> Option<String> {
    n.map(dn)
}

/// A qualified reference as the 0.54 parser spelled it: decoded segments joined by `::`
/// (a `.` separator is kept as `.`), prefixed `$::` when absolute.
pub(crate) fn qr(id: sysml_v2_parser::QualifiedReferenceId) -> String {
    use sysml_v2_parser::ReferenceSeparator as S;
    with_cur(|d| {
        let v = d.qualified_reference(id)?;
        let mut out = String::new();
        if v.metadata.is_absolute {
            out.push_str("$::");
        }
        for (i, seg) in v.segments.iter().enumerate() {
            if i > 0 {
                out.push_str(match seg.separator_before {
                    Some(S::Dot) => ".",
                    _ => "::",
                });
            }
            out.push_str(&v.segment_decoded_text(i)?);
        }
        Some(out)
    })
    .flatten()
    .unwrap_or_default()
}

pub(crate) fn oqr(id: Option<sysml_v2_parser::QualifiedReferenceId>) -> Option<String> {
    id.map(qr)
}

/// The `::`-separated segments of a qualified reference.
pub(crate) fn qr_segments(id: sysml_v2_parser::QualifiedReferenceId) -> Vec<String> {
    with_cur(|d| {
        let v = d.qualified_reference(id)?;
        (0..v.segments.len()).map(|i| v.segment_decoded_text(i).map(|c| c.into_owned())).collect::<Option<Vec<_>>>()
    })
    .flatten()
    .unwrap_or_default()
}

/// A SysML v2 `package`, merged across every `.sysml`/`.kerml` file in the
/// subtree that contributes to it by name (`REQ-TRS-SYSMLV2-002`'s multi-file
/// merge: two files each declaring `package Foo { ... }` combine into one
/// `Foo` namespace instead of colliding on qname).
#[derive(Default)]
struct MergedPackage {
    /// The file that first introduced this package name at this nesting
    /// position — used as the synthesized `Package` element's own `file_path`.
    /// Not meaningful beyond "some real contributing file"; a package merged
    /// from several files doesn't have one canonical owner.
    declared_in: Option<String>,
    /// Non-`Package` body elements contributed by any file, paired with the
    /// source file each came from (so a synthesized element's own `file_path`
    /// reflects where it was actually declared, not just the owning package).
    body: Vec<(sysml_v2_parser::PackageBodyElement, String, DocRef)>,
    /// Nested packages, keyed by name and merged the same way as this level.
    children: BTreeMap<String, MergedPackage>,
}

/// Every `.sysml`/`.kerml` file under `dir`, recursively — however nested, since a
/// `sysmlSubmodel` subtree's directory layout below the marked root carries no
/// namespace meaning of its own (`REQ-TRS-SYSMLV2-001`). Sorted for determinism.
pub(crate) fn find_sysml_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "sysml" || ext == "kerml")
        })
        .map(|e| e.into_path())
        .collect();
    files.sort();
    files
}

/// Parse every `.sysml`/`.kerml` file under `dir`, merge same-named packages
/// across files, and convert the mapped element kinds into `RawElement`s owned
/// by `pkg_qname`. A read or parse failure pushes a `W541` finding onto `owner`
/// (the package's own `_index.md` element) and contributes zero elements from
/// that file — never aborts the rest of the subtree.
///
/// The top-level [`MergedPackage`] *is* the anchor package (`REQ-TRS-SYSMLV2-098`): a file's
/// root-level members are its body, exactly like a package body, so a bare `part def X;` becomes
/// `<pkg_qname>::X` and a root-level `alias`/`doc`/metadata application lifts onto `owner`.
pub fn ingest_subtree(owner: &mut RawElement, pkg_qname: &str, dir: &Path) -> Vec<RawElement> {
    ingest_subtree_detailed(owner, pkg_qname, dir).elements
}

/// What one ingestion pass produced: the elements plus, per *parsed* file, the final unmapped
/// counts that feed `W543` (`REQ-TRS-SYSMLV2-059` -- the report reads these instead of
/// re-deriving them).
pub(crate) struct IngestDetail {
    pub elements: Vec<RawElement>,
    pub file_counts: Vec<(String, BTreeMap<&'static str, usize>)>,
}

/// [`ingest_subtree`] plus the per-file unmapped counts it computed.
pub(crate) fn ingest_subtree_detailed(owner: &mut RawElement, pkg_qname: &str, dir: &Path) -> IngestDetail {
    let mut merged = MergedPackage::default();
    // Per-file unmapped counts; W543 is emitted after conversion so unresolved
    // package-level `satisfy` statements (`REQ-TRS-SYSMLV2-046`) can be added.
    let mut file_counts: Vec<(String, BTreeMap<&'static str, usize>)> = Vec::new();
    for path in find_sysml_files(dir) {
        let file_path = path.display().to_string();
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                owner.derive_findings.push(finding(
                    "W541",
                    &file_path,
                    &format!("could not read '{file_path}': {e}"),
                ));
                continue;
            }
        };
        let parsed = match sysml_v2_parser::parse(&content) {
            Ok(parsed) => Some(parsed),
            Err(e) => {
                // GH #203: one unparsable member must not discard the whole file. The
                // recovering parse yields a partial tree plus a diagnostic (with its line)
                // per bad member; ingest what survived and raise `W541` for each bad one.
                let recovered = sysml_v2_parser::parse_for_editor(&content);
                let real: Vec<&sysml_v2_parser::ParseError> =
                    recovered.errors.iter().filter(|d| d.is_cascade != Some(true)).collect();
                // Recovery is only worth ingesting when something other than a lone error
                // node survived (an unbalanced brace swallows the whole declaration).
                let salvageable = recovered.document.root.elements.iter().any(|n| {
                    !matches!(
                        &n.value,
                        sysml_v2_parser::RootElement::Member(m)
                            if matches!(m.value, sysml_v2_parser::PackageBodyElement::Error(_))
                    )
                });
                if real.is_empty() || !salvageable {
                    owner.derive_findings.push(finding(
                        "W541",
                        &file_path,
                        &format!("SysML v2/KerML parse error in '{file_path}': {e}"),
                    ));
                    None
                } else {
                    for d in &real {
                        let at = match (d.line, d.column) {
                            (Some(l), Some(c)) => format!(" at line {l}, column {c}"),
                            (Some(l), None) => format!(" at line {l}"),
                            _ => String::new(),
                        };
                        owner.derive_findings.push(finding(
                            "W541",
                            &file_path,
                            &format!(
                                "SysML v2/KerML parse error in '{file_path}'{at}: {} (the unparsable member was skipped; the rest of the file was ingested)",
                                d.message
                            ),
                        ));
                    }
                    Some(recovered.document)
                }
            }
        };
        if let Some(parsed) = parsed {
            let (doc, root) = split_document(parsed);
            // `REQ-TRS-SYSMLV2-030`: surface what map-narrow ingestion drops.
            let mut counts = BTreeMap::new();
            with_doc(&doc, || count_unmapped_root(&root, &mut counts));
            file_counts.push((file_path.clone(), counts));
            with_doc(&doc, || merge_root(&mut merged, root, &file_path, &doc))
        }
    }

    let mut out = Vec::new();
    let _ = take_body_unmapped();
    convert_merged(&merged, pkg_qname, &mut out);
    // GH #203: members dropped inside definition/usage bodies, by file.
    for (file, kinds) in take_body_unmapped() {
        if let Some((_, counts)) = file_counts.iter_mut().find(|(f, _)| *f == file) {
            for (k, n) in kinds {
                *counts.entry(k).or_insert(0) += n;
            }
        }
    }
    resolve_exhibit_states(&mut out);
    let unresolved = lift_package_satisfies(&merged, pkg_qname, &mut out);
    let unresolved_includes = resolve_includes(&mut out);
    resolve_dependency_ends(&mut out);
    let mut unresolved_about = resolve_metadata(&mut out);
    // `REQ-TRS-SYSMLV2-095`/`-098`: what a package body lifts onto its `Package` element, the
    // file root lifts onto the anchor package's own element.
    let root_aliases = package_aliases(&merged);
    if !root_aliases.is_empty() {
        owner.frontmatter.aliases.get_or_insert_with(Vec::new).extend(root_aliases);
    }
    let root_imports = package_imports(&merged);
    if !root_imports.is_empty() {
        owner.frontmatter.imports.get_or_insert_with(Vec::new).extend(root_imports);
    }
    let root_doc = package_doc(&merged);
    if !root_doc.is_empty() {
        if !owner.doc.trim().is_empty() {
            owner.doc = format!("{}\n\n{root_doc}", owner.doc.trim_end());
        } else {
            owner.doc = root_doc;
        }
    }
    for (file, n) in lift_root_metadata(&merged, pkg_qname, owner, &mut out) {
        *unresolved_about.entry(file).or_insert(0) += n;
    }
    let mut detailed = Vec::new();
    for (file_path, mut counts) in file_counts {
        if let Some(n) = unresolved.get(&file_path) {
            *counts.entry("satisfy").or_insert(0) += n;
        }
        if let Some(n) = unresolved_includes.get(&file_path) {
            *counts.entry("include").or_insert(0) += n;
        }
        if let Some(n) = unresolved_about.get(&file_path) {
            *counts.entry("metadata").or_insert(0) += n;
        }
        detailed.push((file_path.clone(), counts.clone()));
        if counts.is_empty() {
            continue;
        }
        let total: usize = counts.values().sum();
        let list = counts
            .iter()
            .map(|(k, n)| format!("{k} x{n}"))
            .collect::<Vec<_>>()
            .join(", ");
        owner.derive_findings.push(finding(
            "W543",
            &file_path,
            &format!(
                "'{file_path}': {total} parsed construct(s) have no Syscribe mapping and were not ingested: {list}"
            ),
        ));
    }
    IngestDetail { elements: out, file_counts: detailed }
}

/// GH #203: an `exhibit` of a state named relative to its owner (`exhibit state s : S;`,
/// `exhibit s;`) -- qualify it against the ingested subtree so `exhibitsStates:` resolves.
/// Names that resolve nowhere in the subtree stay as written (`W501` then reports them).
fn resolve_exhibit_states(out: &mut [RawElement]) {
    if !out.iter().any(|e| e.frontmatter.exhibits_states.is_some()) {
        return;
    }
    let index: super::EndpointIndex = out.iter().map(|e| (e.qualified_name.clone(), (None, None))).collect();
    for e in out.iter_mut() {
        let scope = e.qualified_name.clone();
        if let Some(list) = e.frontmatter.exhibits_states.as_mut() {
            for s in list.iter_mut() {
                if let Some(q) = super::lookup_scoped(&index, &scope, s) {
                    *s = q;
                }
            }
        }
    }
}

/// `REQ-TRS-SYSMLV2-054`: replace each use case's raw `include X;` names by the qualified name
/// of the `UseCaseDef`/`UseCase` they resolve to (innermost scope first, from the including
/// element's own qname outward). Unresolved names are dropped; returns the per-file count.
fn resolve_includes(out: &mut [RawElement]) -> BTreeMap<String, usize> {
    let mut unresolved = BTreeMap::new();
    let index: super::EndpointIndex = out
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::UseCaseDef) | Some(ElementType::UseCase)))
        .map(|e| (e.qualified_name.clone(), (None, None)))
        .collect();
    for e in out.iter_mut() {
        let Some(raw) = e.frontmatter.includes.take() else { continue };
        let mut resolved: Vec<String> = Vec::new();
        for name in raw {
            match super::lookup_scoped(&index, &e.qualified_name, &name) {
                Some(q) if q != e.qualified_name => {
                    if !resolved.contains(&q) {
                        resolved.push(q);
                    }
                }
                _ => *unresolved.entry(e.file_path.clone()).or_insert(0) += 1,
            }
        }
        e.frontmatter.includes = nonempty_vec(resolved);
    }
    unresolved
}

/// `REQ-TRS-SYSMLV2-046`: the subject of a package-level `satisfy <req> by
/// <subject>;`, as a `::`-joined reference, or `None` when the statement is
/// negated, inline-declared, the bare shorthand (no distinct subject), or the
/// subject is not a plain feature reference.
fn satisfy_subject(s: &sysml_v2_parser::SatisfyRequirementUsage) -> Option<String> {
    if s.not_span.is_some() || matches!(s.requirement, sysml_v2_parser::SatisfiedRequirement::Declaration(_)) {
        return None;
    }
    // `satisfy R;` (no `by`) carries no distinct subject; a chain subject joins with `::`.
    let subject = s.subject.as_ref()?;
    Some(qr_segments(subject.value.reference).join("::"))
}

/// Resolve every liftable package-level `satisfy` against the converted
/// elements and append the requirement to the subject's `satisfies:`. Returns
/// per-file counts of the statements that could not be lifted.
fn lift_package_satisfies(
    merged: &MergedPackage,
    qname: &str,
    out: &mut Vec<RawElement>,
) -> BTreeMap<String, usize> {
    let mut pending: Vec<(String, String, String, String)> = Vec::new(); // scope, req, subject, file
    collect_package_satisfies(merged, qname, &mut pending);
    let mut unresolved = BTreeMap::new();
    if pending.is_empty() {
        return unresolved;
    }
    let index: super::EndpointIndex = out
        .iter()
        .map(|e| (e.qualified_name.clone(), (None, None)))
        .collect();
    for (scope, req, subject, file) in pending {
        let target = super::lookup_scoped(&index, &scope, &subject);
        let elem = target.and_then(|q| out.iter_mut().find(|e| e.qualified_name == q));
        match elem {
            Some(e) => {
                let list = e.frontmatter.satisfies.get_or_insert_with(Vec::new);
                if !list.contains(&req) {
                    list.push(req);
                }
            }
            None => *unresolved.entry(file).or_insert(0) += 1,
        }
    }
    unresolved
}

fn collect_package_satisfies(
    merged: &MergedPackage,
    qname: &str,
    pending: &mut Vec<(String, String, String, String)>,
) {
    for (elem, file, doc) in &merged.body {
        if let sysml_v2_parser::PackageBodyElement::Satisfy(n) = elem {
            with_doc(doc, || {
                if let (Some(req), Some(subject)) = (satisfy_target(&n.value), satisfy_subject(&n.value)) {
                    pending.push((qname.to_string(), req, subject, file.clone()));
                }
            });
        }
    }
    for (name, child) in &merged.children {
        collect_package_satisfies(child, &format!("{qname}::{name}"), pending);
    }
}

/// Human-readable kind of a package-body member that ingestion parses but
/// does not map (`REQ-TRS-SYSMLV2-030`). `None` for mapped kinds and for pure
/// namespace plumbing (`import`, `comment`, parse-error nodes), which stay quiet.
/// Only ever asked about a member of a *named* package (the anchor included): a package declared
/// with a qualified name is counted whole by [`count_unmapped_package`] and never walked.
fn unmapped_kind(e: &sysml_v2_parser::PackageBodyElement) -> Option<&'static str> {
    use sysml_v2_parser::PackageBodyElement as E;
    Some(match e {
        E::Annotating(sysml_v2_parser::ast::AnnotatingMember::TextualRep(_)) => "textual representation",
        E::Filter(_) => "filter",
        // `REQ-TRS-SYSMLV2-043`: lifted onto the enclosing package. The 0.57 parser rejects an
        // anonymous alias, so this arm is defensive only.
        E::AliasDef(a) => {
            if ident_name(&a.value.identification).is_some() {
                return None;
            }
            "alias"
        }
        // `REQ-TRS-SYSMLV2-046`: liftable ones are counted after resolution.
        E::Satisfy(s) => {
            if satisfy_target(&s.value).is_some() && satisfy_subject(&s.value).is_some() {
                return None;
            }
            "satisfy"
        }
        E::Actor(_) => "actor",
        // GH #203: an `interface ... connect` has no owning part at package level to hold the
        // wiring; the interface usage itself is still ingested when it is named.
        E::InterfaceUsage(i) => {
            if matches!(i.value, sysml_v2_parser::ast::InterfaceUsage::Declaration { .. }) {
                return None;
            }
            "interface connect"
        }
        // `REQ-TRS-SYSMLV2-083`/`-084`/`-093`/`-094`: mapped (every dependency, every named
        // occurrence usage). `REQ-TRS-SYSMLV2-086`/`-087`: a metadata usage is lifted; only an
        // unresolved `about` target is counted, after resolution.
        E::IndividualDef(_) | E::MetadataUsage(_) | E::OccurrenceDef(_) | E::Dependency(_) => return None,
        E::OccurrenceUsage(o) => {
            if occurrence_usage_mappable(&o.value) {
                return None;
            }
            "occurrence"
        }
        E::FeatureDecl(_) | E::ClassifierDecl(_) | E::KermlSemanticDecl(_) | E::KermlFeatureDecl(_)
        | E::ExtendedLibraryDecl(_) | E::KermlClassifier(_) | E::KermlInvariant(_) | E::KermlConnector(_)
        | E::KermlRelationship(_) | E::KermlFeature(_) | E::KermlBareDeclaration(_) => "KerML declaration",
        // `REQ-TRS-SYSMLV2-097`: package-level usage/relationship members with no package-level
        // native target (they are definition-body members in the native format), user-defined
        // keyword declarations, and grammar the parser itself marks unsupported.
        E::Unsupported(_) | E::Expose(_) | E::Ref(_) | E::Connect(_) | E::DefaultReferenceUsage(_)
        | E::AssertConstraint(_) | E::PerformUsage(_) | E::BindingConnectorUsage(_) | E::Succession(_)
        | E::ExhibitState(_) | E::IncludeUseCase(_) | E::ExtendedDefinition(_) | E::ExtendedUsage(_) => {
            "other package member"
        }
        _ => return None,
    })
}

fn count_unmapped_body(
    elements: &[sysml_v2_parser::Node<sysml_v2_parser::PackageBodyElement>],
    counts: &mut BTreeMap<&'static str, usize>,
) {
    use sysml_v2_parser::PackageBodyElement as E;
    for n in elements {
        match &n.value {
            E::Package(inner) => count_unmapped_package(&inner.value.identification, &inner.value.body, counts),
            E::LibraryPackage(inner) => count_unmapped_package(&inner.value.identification, &inner.value.body, counts),
            other => {
                if let Some(k) = unmapped_kind(other) {
                    *counts.entry(k).or_insert(0) += 1;
                }
            }
        }
    }
}

/// A `package`/`library package`/`namespace` declaration: its body is walked when it has a simple
/// name. `REQ-TRS-SYSMLV2-100`: one declared with a *qualified* name (`package A::B { }`, which
/// the parser accepts and the language gives no meaning to) has no identity to merge under
/// (`merge_named` skips it), so it is counted once as `qualified package` and its members —
/// which are not ingested — are not walked.
fn count_unmapped_package(
    id: &sysml_v2_parser::QualifiedIdentification,
    body: &sysml_v2_parser::PackageBody,
    counts: &mut BTreeMap<&'static str, usize>,
) {
    if qident_name(id).is_none() {
        *counts.entry("qualified package").or_insert(0) += 1;
        return;
    }
    if let sysml_v2_parser::PackageBody::Brace { elements, .. } = body {
        count_unmapped_body(elements, counts);
    }
}

pub(crate) fn count_unmapped_root(root: &sysml_v2_parser::RootNamespace, counts: &mut BTreeMap<&'static str, usize>) {
    use sysml_v2_parser::RootElement as R;
    for n in &root.elements {
        match &n.value {
            R::Package(p) => count_unmapped_package(&p.value.identification, &p.value.body, counts),
            R::LibraryPackage(p) => count_unmapped_package(&p.value.identification, &p.value.body, counts),
            R::Namespace(p) => count_unmapped_package(&p.value.identification, &p.value.body, counts),
            R::Import(_) => {}
            // `REQ-TRS-SYSMLV2-098`: the file root is the anchor package's own (named) body, so a
            // bare root member counts under exactly the kind it would count under in a package.
            R::Member(m) => count_unmapped_body(std::slice::from_ref(&**m), counts),
        }
    }
}

/// Merge one file's already-parsed root namespace into `target`, the anchor package. A
/// `package`/`library package`/`namespace` merges by name into `target.children`; every other
/// root member is the anchor's own body member (`REQ-TRS-SYSMLV2-098`), exactly as the same
/// member inside a package body would be — a definition or usage becomes `<anchor>::X`, a named
/// `alias` lifts onto the anchor (`REQ-TRS-SYSMLV2-095`). Imports are namespace plumbing and
/// carry nothing to merge.
fn merge_root(target: &mut MergedPackage, root: sysml_v2_parser::RootNamespace, file_path: &str, doc: &DocRef) {
    use sysml_v2_parser::RootElement as R;
    for node in root.elements {
        match node.value {
            R::Package(p) => merge_named(target, &p.value.identification, p.value.body, file_path, doc),
            // `REQ-TRS-SYSMLV2-044`: library packages and namespaces become Packages.
            R::LibraryPackage(p) => merge_named(target, &p.value.identification, p.value.body, file_path, doc),
            R::Namespace(p) => merge_named(target, &p.value.identification, p.value.body, file_path, doc),
            R::Member(m) => merge_package_body(target, vec![*m], file_path, doc),
            // An `import` is a namespace relationship of the anchor: keep it as `imports:`.
            R::Import(i) => {
                let n = *i;
                target.body.push((sysml_v2_parser::PackageBodyElement::Import(n), file_path.to_string(), doc.clone()))
            }
        }
    }
}

/// Merge one `package`/`library package`/`namespace` declaration into
/// `target.children`, combining with whatever a same-named package already
/// contributed (from this file or an earlier one).
fn merge_named(
    target: &mut MergedPackage,
    identification: &sysml_v2_parser::QualifiedIdentification,
    body: sysml_v2_parser::PackageBody,
    file_path: &str,
    doc: &DocRef,
) {
    let Some(name) = qident_name(identification) else {
        return; // anonymous package: no identity to qname or merge against
    };
    let is_new = !target.children.contains_key(&name);
    let entry = target.children.entry(name).or_default();
    if is_new {
        entry.declared_in = Some(file_path.to_string());
    }
    if let sysml_v2_parser::PackageBody::Brace { elements, .. } = body {
        merge_package_body(entry, elements, file_path, doc);
    }
}

fn merge_package_body(
    target: &mut MergedPackage,
    elements: Vec<sysml_v2_parser::Node<sysml_v2_parser::PackageBodyElement>>,
    file_path: &str,
    doc: &DocRef,
) {
    for node in elements {
        match node.value {
            sysml_v2_parser::PackageBodyElement::Package(inner) => {
                merge_named(target, &inner.value.identification, inner.value.body, file_path, doc);
            }
            sysml_v2_parser::PackageBodyElement::LibraryPackage(inner) => {
                merge_named(target, &inner.value.identification, inner.value.body, file_path, doc);
            }
            other => target.body.push((other, file_path.to_string(), doc.clone())),
        }
    }
}

fn ident_name(id: &sysml_v2_parser::Identification) -> Option<String> {
    odn(id.name).or_else(|| odn(id.short_name))
}

/// Name of a package-like declaration. A *qualified* declared name (`package A::B`, new in the
/// 0.55 grammar) is not a simple identity, so the declaration is not merged (`None`); its members
/// are not ingested and `REQ-TRS-SYSMLV2-100` counts the declaration once in `W543`.
fn qident_name(id: &sysml_v2_parser::QualifiedIdentification) -> Option<String> {
    id.simple_name().map(dn).or_else(|| odn(id.short_name))
}

/// Fields common to every synthesized SysMLv2-originated `RawElement`.
/// `supertype` carries a Def's `:>` specialization target; `typed_by` carries a
/// Usage's `:` typing target — kept distinct exactly like hand-authored
/// frontmatter does. `is_variant`/`variant_of` are `REQ-TRS-SYSMLV2-007`'s
/// "variation/variant membership" recognition. `satisfies`/`verifies` are
/// `REQ-TRS-SYSMLV2-003`'s native `satisfy`/`verify` relationship targets,
/// carried verbatim (quoted or unquoted, already quote-stripped by the
/// parser's own lexer) — resolution is the existing id-or-qname resolver,
/// unchanged. `applies_when` is `REQ-TRS-SYSMLV2-005`'s `@SyscribeFeature {
/// featureId = '...'; }` lift — written into the exact same field a native
/// element's `appliesWhen:` uses, so the feature-model/SAT engine needs no
/// changes at all to reason about it. `domain`/`asil_level`/`sil_level`/
/// `pl_level`/`short_name`/`implemented_by` are `REQ-TRS-SYSMLV2-008`'s fixed
/// `@Syscribe*` annotation lift — same posture: written into the exact fields
/// a hand-authored element uses, no validator changes.
#[derive(Default)]
struct Spec {
    supertype: Option<String>,
    typed_by: Option<String>,
    is_variation: Option<bool>,
    is_variant: Option<bool>,
    variant_of: Option<String>,
    satisfies: Option<Vec<String>>,
    verifies: Option<Vec<String>>,
    applies_when: Option<String>,
    domain: Option<String>,
    asil_level: Option<String>,
    sil_level: Option<u8>,
    pl_level: Option<String>,
    short_name: Option<String>,
    implemented_by: Option<Vec<String>>,
    /// `REQ-TRS-SYSMLV2-009`'s `doc /* ... */` lift — written into
    /// `RawElement.doc` (not `RawFrontmatter`, unlike every other field
    /// here; see `push_synth`) the same way a hand-authored `.md` file's
    /// body below its `---` closer populates it. Empty string, not
    /// `Option`, since that's `RawElement.doc`'s own type — "no doc member"
    /// and "" are the same thing.
    doc: String,
    /// `REQ-TRS-SYSMLV2-010`'s connection-endpoint lift -- the *owning*
    /// `part def`/`part`'s own `connections:` YAML entries (not the nested
    /// `Connection` element's own `Spec`; see `connection_usage_entry` for
    /// why entries are qualified-qname, not literal chain text).
    connections: Option<Vec<serde_yaml::Value>>,
    /// `REQ-TRS-SYSMLV2-018` -- a `StateDef`/`StateUsage`'s own nested
    /// substates, each carrying its own `transitions:`/`entryAction`/etc.
    /// inline (never separate `RawElement`s -- see `convert_state_def`).
    sub_states: Option<Vec<serde_yaml::Value>>,
    /// `REQ-TRS-SYSMLV2-018` -- transitions declared as siblings at this
    /// state's own body level (as opposed to nested inside one of
    /// `sub_states`' own entries) -- always carries an explicit `source:`.
    transitions: Option<Vec<serde_yaml::Value>>,
    entry_action: Option<serde_yaml::Value>,
    do_action: Option<serde_yaml::Value>,
    exit_action: Option<serde_yaml::Value>,
    /// `REQ-TRS-SYSMLV2-019` -- an `ActionDef`/`ActionUsage`'s own nested,
    /// `kind:`-tagged action tree (`PerformAction`/`IfAction`/`LoopAction`/
    /// `AssignmentAction`/`TerminateAction`).
    sub_actions: Option<Vec<serde_yaml::Value>>,
    /// `REQ-TRS-SYSMLV2-019` -- flat `{name, kind}` control-flow markers
    /// (`ForkNode`/`JoinNode`/`DecisionNode`/`MergeNode`) with no recoverable
    /// internal content -- the pinned parser discards `fork`/`join`/`decide`/
    /// `merge` block bodies itself (`FirstMergeBody::Brace` carries no data).
    control_nodes: Option<Vec<serde_yaml::Value>>,
    /// `REQ-TRS-SYSMLV2-019` -- flat `{after, before}` control-flow edges
    /// lifted from `first`/`then` successions.
    succession_connections: Option<Vec<serde_yaml::Value>>,
    /// `REQ-TRS-SYSMLV2-020` -- a `view` usage's `expose <target>;` members,
    /// always plain-string entries (never the richer `{ref, isRecursive,
    /// filter}` map form -- see `view_expose_entries`). Never set for a
    /// `view def` -- the grammar structurally cannot carry `expose` there.
    expose: Option<Vec<serde_yaml::Value>>,
    /// `REQ-TRS-SYSMLV2-020` -- a `view` usage's `satisfy <viewpoint>;`
    /// target. Never set for a `view def`, same reason as `expose` above.
    viewpoint: Option<String>,
    /// `REQ-TRS-SYSMLV2-021` -- a `viewpoint def`/`viewpoint` usage's
    /// `stakeholder <name>;` members (name only).
    stakeholders: Option<Vec<String>>,
    /// `REQ-TRS-SYSMLV2-021` -- a `viewpoint def`/`viewpoint` usage's
    /// `purpose <target>;` members.
    concerns: Option<Vec<String>>,
    /// `REQ-TRS-SYSMLV2-020` -- a `view def`/`view` usage's own `render
    /// <name> [: <Type>];` clause, first one wins (single-string field).
    rendering: Option<String>,
    /// `REQ-TRS-SYSMLV2-023` -- a `concern def`/`concern` usage's `subject
    /// <name> : <Type>;` declaration (`SubjectDecl.type_name`). The bare
    /// `subject;` shorthand (`SubjectRef`, an empty AST node) carries
    /// nothing to extract and never sets this. Plain field, no dedicated
    /// builder -- set directly in the `Spec { ... }` literal like
    /// `supertype`/`typed_by`.
    subject: Option<String>,
    /// `REQ-TRS-SYSMLV2-024` -- a `flow def`/`flow` usage's item type,
    /// sourced from `FlowUsage.payload.type_name` (the `of` clause) or
    /// `FlowUsage.type_name` (the bare `:` typing) -- both item-shaped, not
    /// a `typedBy`-style supertype reference (see `flow_item_type`). Plain
    /// field, no dedicated builder, like `subject`.
    item_type: Option<String>,
    /// `REQ-TRS-SYSMLV2-024` -- the *owning* `part def`/`part`'s own
    /// `flowConnections:` YAML entries, lifted from every `FlowUsage` found
    /// directly in its body (named or anonymous) -- mirrors `connections`
    /// above exactly, one field per relationship kind.
    flow_connections: Option<Vec<serde_yaml::Value>>,
    /// `REQ-TRS-SYSMLV2-025` -- an `enum def`'s literal values, each a
    /// `{name: ...}` map (`EnumeratedValue` carries no other data -- see
    /// `convert_enum_def`). Plain field, no dedicated builder, like
    /// `item_type`.
    values: Option<Vec<serde_yaml::Value>>,
    /// `REQ-TRS-SYSMLV2-026`/`-027`/`-028` -- a case/analysis-case/
    /// verification-case's `actor <name> : <Type>;` members (`type_name`
    /// only -- see `case_body_fields`).
    actors: Option<Vec<String>>,
    /// `REQ-TRS-SYSMLV2-026`/`-027`/`-028` -- a case family element's
    /// `objective <name>? : <Type> { ... }` members, one plain-string entry
    /// per objective (name, falling back to type when anonymous).
    objectives: Option<Vec<serde_yaml::Value>>,
    /// `REQ-TRS-SYSMLV2-026`/`-027`/`-028` -- a case family element's first
    /// `return [attribute|part] name : <Type>;` declaration's type -- first
    /// one wins, the native `result:` field is a single string.
    result_type: Option<String>,
    /// `REQ-TRS-SYSMLV2-026`/`-027`/`-028` -- the AST's own `is_abstract`
    /// bool, present on all six case-family Def/Usage structs. Plain field,
    /// no dedicated builder, like `subject`.
    is_abstract: Option<bool>,
    /// `REQ-TRS-SYSMLV2-029` (GH #144) -- a named `allocation` usage's
    /// `allocate <source> to <target>` endpoints, carried as raw endpoint
    /// text (a possibly `::`-qualified name, optionally followed by a
    /// `.`-separated feature chain). Rewritten to resolved qualified names by
    /// [`super::resolve_allocation_endpoints`] once the whole model is merged.
    allocated_from: Option<Vec<String>>,
    allocated_to: Option<Vec<String>>,
    /// `REQ-TRS-SYSMLV2-033`/`-034` -- a `constraint def`/`calc def`'s `in`/
    /// `out`/`inout` parameter declarations as `{name, typedBy, direction}`
    /// maps (`direction: return` for a `calc`'s `return` declaration).
    parameters: Option<Vec<serde_yaml::Value>>,
    /// `REQ-TRS-SYSMLV2-033` -- a constraint's expression text (opaque).
    expression: Option<String>,
    /// `REQ-TRS-SYSMLV2-034` -- a calc's expression text (opaque) + language.
    body: Option<String>,
    body_language: Option<String>,
    /// `REQ-TRS-SYSMLV2-034` -- a calc's first `return` declaration's type.
    return_type: Option<String>,
    /// `REQ-TRS-SYSMLV2-043` -- a package's `alias` members as `{name, for}` maps.
    aliases: Option<Vec<serde_yaml::Value>>,
    /// A package's `import` members as `imports:` strings (`::*`/`::**` suffixes kept).
    imports: Option<Vec<serde_yaml::Value>>,
    /// `REQ-TRS-SYSMLV2-048` -- a usage's multiplicity text, `subsets`, `redefines`.
    multiplicity: Option<String>,
    subsets: Option<Vec<String>>,
    redefines: Option<serde_yaml::Value>,
    /// `REQ-TRS-SYSMLV2-054` -- raw `include X;` names of a use case; resolved to qnames (or dropped
    /// and counted) once the whole subtree is merged, see `resolve_includes`.
    includes: Option<Vec<String>>,
    /// `REQ-TRS-SYSMLV2-055` -- an attribute usage's literal value and its unit.
    value: Option<serde_yaml::Value>,
    unit: Option<String>,
    /// `REQ-TRS-SYSMLV2-083` -- `individual` on an occurrence usage.
    is_individual: Option<bool>,
    /// `REQ-TRS-SYSMLV2-084` -- a `dependency`'s endpoints (raw text until resolved after the merge).
    clients: Option<Vec<String>>,
    suppliers: Option<Vec<String>>,
    /// `REQ-TRS-SYSMLV2-086`/`-087` -- the element's `metadata:` applications; `type:` and a
    /// pending `about:` are raw text until [`resolve_metadata`] runs over the merged subtree.
    metadata: Option<Vec<serde_yaml::Value>>,
    /// `REQ-TRS-SYSMLV2-094` -- `snapshot`/`timeslice` on an occurrence usage.
    is_portion: Option<bool>,
    portion_kind: Option<String>,
    /// `REQ-TRS-SYSMLV2-086` -- a `metadata def`'s attribute members as inline `features:`.
    features: Option<Vec<serde_yaml::Value>>,
    /// GH #203 -- `ref` on a usage, `~Type` conjugated port typing, a feature's `in`/`out`/`inout`.
    is_reference: Option<bool>,
    is_conjugated: Option<bool>,
    direction: Option<String>,
    /// GH #203 -- an `interface def`/`connection def`'s `end` features.
    ends: Option<Vec<serde_yaml::Value>>,
    /// GH #203 -- `bind a = b;` on the owning part, `perform action ...;`, `exhibit state ...;`.
    binding_connections: Option<Vec<serde_yaml::Value>>,
    performs: Option<Vec<serde_yaml::Value>>,
    exhibits_states: Option<Vec<String>>,
}

impl Spec {
    /// Copy `REQ-TRS-SYSMLV2-008`'s six lifted fields from `meta` onto `self`.
    /// A small builder rather than six repeated assignments at each of the
    /// three call sites (`part def`, `part` usage, `variant part` usage).
    fn with_syscribe_meta(mut self, meta: SyscribeMeta) -> Self {
        self.domain = meta.domain;
        self.asil_level = meta.asil_level;
        self.sil_level = meta.sil_level;
        self.pl_level = meta.pl_level;
        self.short_name = meta.short_name;
        self.implemented_by = meta.implemented_by;
        self
    }

    /// Set `REQ-TRS-SYSMLV2-048`'s lifted multiplicity/subsets/redefines.
    fn with_usage_relations(
        mut self,
        multiplicity: Option<&sysml_v2_parser::ast::Multiplicity>,
        subsets: Option<&sysml_v2_parser::ast::SubsettingRelationship>,
        redefines: Option<&sysml_v2_parser::ast::SubsettingRelationship>,
    ) -> Self {
        self.multiplicity = multiplicity.map(multiplicity_text);
        self.subsets = subsets.map(subsetting_targets).filter(|v| !v.is_empty());
        self.redefines = redefines.map(subsetting_targets).and_then(|mut v| match v.len() {
            0 => None,
            1 => Some(serde_yaml::Value::String(v.remove(0))),
            _ => Some(serde_yaml::Value::Sequence(v.into_iter().map(serde_yaml::Value::String).collect())),
        });
        self
    }

    /// Set `REQ-TRS-SYSMLV2-009`'s lifted `doc` text.
    fn with_doc(mut self, doc: String) -> Self {
        self.doc = doc;
        self
    }

    /// Set `REQ-TRS-SYSMLV2-086`'s lifted `metadata:` applications.
    fn with_metadata(mut self, metadata: Vec<serde_yaml::Value>) -> Self {
        self.metadata = nonempty_vec(metadata);
        self
    }

    /// Set `REQ-TRS-SYSMLV2-010`'s lifted `connections:` entries.
    fn with_connections(mut self, connections: Vec<serde_yaml::Value>) -> Self {
        self.connections = nonempty_vec(connections);
        self
    }

    /// Set `REQ-TRS-SYSMLV2-024`'s lifted `flowConnections:` entries.
    fn with_flow_connections(mut self, flow_connections: Vec<serde_yaml::Value>) -> Self {
        self.flow_connections = nonempty_vec(flow_connections);
        self
    }

    /// Set `REQ-TRS-SYSMLV2-018`'s lifted `subStates:`/`transitions:`.
    fn with_state_machine(mut self, sub_states: Vec<serde_yaml::Value>, transitions: Vec<serde_yaml::Value>) -> Self {
        self.sub_states = nonempty_vec(sub_states);
        self.transitions = nonempty_vec(transitions);
        self
    }

    /// Set `REQ-TRS-SYSMLV2-019`'s lifted `subActions:`/`controlNodes:`/
    /// `successionConnections:`.
    fn with_behavior(
        mut self,
        sub_actions: Vec<serde_yaml::Value>,
        control_nodes: Vec<serde_yaml::Value>,
        succession_connections: Vec<serde_yaml::Value>,
    ) -> Self {
        self.sub_actions = nonempty_vec(sub_actions);
        self.control_nodes = nonempty_vec(control_nodes);
        self.succession_connections = nonempty_vec(succession_connections);
        self
    }

    /// Set `REQ-TRS-SYSMLV2-020`'s lifted `expose:`/`viewpoint:`/`rendering:`.
    fn with_view(mut self, expose: Vec<serde_yaml::Value>, viewpoint: Option<String>, rendering: Option<String>) -> Self {
        self.expose = nonempty_vec(expose);
        self.viewpoint = viewpoint;
        self.rendering = rendering;
        self
    }

    /// Set the lifted `stakeholders:`/`concerns:` — originally
    /// `REQ-TRS-SYSMLV2-021` (Viewpoint, both fields), generalized in name
    /// only for `REQ-TRS-SYSMLV2-023` (Concern, `concerns` always empty).
    fn with_stakeholders_concerns(mut self, stakeholders: Vec<String>, concerns: Vec<String>) -> Self {
        self.stakeholders = nonempty_vec(stakeholders);
        self.concerns = nonempty_vec(concerns);
        self
    }
}

fn push_synth(
    out: &mut Vec<RawElement>,
    qname: &str,
    file_path: &str,
    ty: ElementType,
    name: &str,
    spec: Spec,
) {
    let derive_findings = spec
        .multiplicity
        .as_deref()
        .and_then(multiplicity_problem)
        .map(|why| vec![finding("W544", file_path, &format!("'{qname}': multiplicity [{}] {why}", spec.multiplicity.as_deref().unwrap_or("")))])
        .unwrap_or_default();
    out.push(RawElement {
        qualified_name: qname.to_string(),
        file_path: file_path.to_string(),
        frontmatter: {
            let mut fm = RawFrontmatter {
            element_type: Some(ty),
            name: Some(name.to_string()),
            supertype: spec.supertype.map(serde_yaml::Value::String),
            typed_by: spec.typed_by.map(serde_yaml::Value::String),
            satisfies: spec.satisfies,
            verifies: spec.verifies,
            applies_when: spec.applies_when.map(serde_yaml::Value::String),
            domain: spec.domain,
            short_name: spec.short_name,
            implemented_by: spec.implemented_by,
            connections: spec.connections,
            subject: spec.subject,
            item_type: spec.item_type,
            is_abstract: spec.is_abstract,
            allocated_to: spec.allocated_to,
            parameters: spec.parameters,
            multiplicity: spec.multiplicity,
            subsets: spec.subsets,
            redefines: spec.redefines,
            unit: spec.unit,
            features: spec.features,
            ..Default::default()
            };
            fm.is_variation = spec.is_variation;
            fm.is_variant = spec.is_variant;
            fm.variant_of = spec.variant_of;
            fm.asil_level = spec.asil_level;
            fm.sil_level = spec.sil_level;
            fm.pl_level = spec.pl_level;
            fm.sub_states = spec.sub_states;
            fm.transitions = spec.transitions;
            fm.entry_action = spec.entry_action;
            fm.do_action = spec.do_action;
            fm.exit_action = spec.exit_action;
            fm.sub_actions = spec.sub_actions;
            fm.control_nodes = spec.control_nodes;
            fm.succession_connections = spec.succession_connections;
            fm.expose = spec.expose;
            fm.viewpoint = spec.viewpoint;
            fm.stakeholders = spec.stakeholders;
            fm.concerns = spec.concerns;
            fm.rendering = spec.rendering;
            fm.flow_connections = spec.flow_connections;
            fm.values = spec.values;
            fm.actors = spec.actors;
            fm.objectives = spec.objectives;
            fm.result_type = spec.result_type;
            fm.allocated_from = spec.allocated_from;
            fm.expression = spec.expression;
            fm.body = spec.body;
            fm.body_language = spec.body_language;
            fm.return_type = spec.return_type;
            fm.aliases = spec.aliases;
            fm.imports = spec.imports;
            fm.includes = spec.includes;
            fm.value = spec.value;
            fm.is_individual = spec.is_individual;
            fm.clients = spec.clients;
            fm.suppliers = spec.suppliers;
            fm.metadata = spec.metadata;
            fm.is_portion = spec.is_portion;
            fm.portion_kind = spec.portion_kind;
            fm.is_reference = spec.is_reference;
            fm.is_conjugated = spec.is_conjugated;
            fm.direction = spec.direction;
            fm.ends = spec.ends;
            fm.binding_connections = spec.binding_connections;
            fm.performs = spec.performs;
            fm.exhibits_states = spec.exhibits_states;
            fm
        },
        doc: spec.doc,
        parse_issue: None,
        derived: Default::default(),
        derive_findings,
        locale_docs: Default::default(),
        about_notes: Default::default(),
    });
}

/// `REQ-TRS-SYSMLV2-066` (`W544`): what is wrong with multiplicity text `3..1`/`-1`/`1.5..2`, if
/// anything. Only literal bounds are judged; a name or other expression is never evaluated.
fn multiplicity_problem(text: &str) -> Option<&'static str> {
    enum B {
        Star,
        Nat(u64),
        Other,
        Bad(&'static str),
    }
    let bound = |t: &str| -> B {
        let t = t.trim();
        if t == "*" {
            B::Star
        } else if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) {
            t.parse().map_or(B::Other, B::Nat)
        } else if t.starts_with('-') && t[1..].chars().next().is_some_and(|c| c.is_ascii_digit()) {
            B::Bad("has a negative bound")
        } else if t.contains('.') && t.replace('.', "").chars().all(|c| c.is_ascii_digit()) {
            B::Bad("has a non-integer bound")
        } else {
            B::Other
        }
    };
    let (lo, hi) = match text.split_once("..") {
        Some((l, u)) if !u.contains("..") => (bound(l), Some(bound(u))),
        Some(_) => return None,
        None => (bound(text), None),
    };
    for b in std::iter::once(&lo).chain(hi.iter()) {
        if let B::Bad(why) = b {
            return Some(why);
        }
    }
    match (lo, hi) {
        (B::Nat(l), Some(B::Nat(u))) if l > u => Some("has a lower bound greater than its upper bound"),
        (B::Star, Some(B::Nat(_))) => Some("has a lower bound greater than its upper bound"),
        _ => None,
    }
}

/// Push each of `REQ-TRS-SYSMLV2-015`'s connect-endpoint truncation messages
/// as a `W542` finding onto the element `push_synth` just pushed. Called
/// immediately after `push_synth` for a `part def`/`part`/`variant part`
/// usage whose `connections:` lift produced one: `part_def_connection_entries`/
/// `part_usage_connection_entries` run *before* the owning element exists as
/// a `RawElement` to attach findings to directly (they only compute the
/// `connections:` YAML value that later goes into that element's `Spec`), so
/// the findings are attached here, one statement after `push_synth`, via
/// `out.last_mut()` rather than threaded back through `Spec` itself.
fn push_connection_truncation_findings(out: &mut [RawElement], file_path: &str, truncations: Vec<String>) {
    if truncations.is_empty() {
        return;
    }
    if let Some(last) = out.last_mut() {
        for msg in truncations {
            last.derive_findings.push(finding("W542", file_path, &msg));
        }
    }
}

/// A `satisfy`/`verify` relationship target from a plain `Expression` — only
/// the common `FeatureRef` shape (a single quoted/unquoted name, or a
/// `::`-qualified name, already quote-stripped by the parser's lexer) is
/// recognized; other expression shapes aren't meaningful reference targets
/// here and are left unmapped.
fn feature_ref_string(e: &sysml_v2_parser::Expression) -> Option<String> {
    match e {
        sysml_v2_parser::Expression::FeatureRef(id) => Some(qr(*id)),
        _ => None,
    }
}

/// A qualified reference spelled with `::` only (no `.` feature-chain step) as a string: the
/// shape 0.54 produced as `Expression::FeatureRef`; a dotted spelling was a `FeatureChainRef`.
fn plain_ref_string(id: sysml_v2_parser::QualifiedReferenceId) -> Option<String> {
    let dotted = with_cur(|d| {
        d.qualified_reference(id)
            .map(|v| v.segments.iter().any(|s| matches!(s.separator_before, Some(sysml_v2_parser::ReferenceSeparator::Dot))))
    })
    .flatten()
    .unwrap_or(true);
    if dotted {
        None
    } else {
        Some(qr(id))
    }
}

/// The `Requirement` reference a `satisfy` statement targets, or `None` if
/// this one isn't a simple reference we map.
///
/// Whichever form is used — `satisfy 'REQ-X';` (shorthand, no `by` clause) or
/// `satisfy 'REQ-X' by subject;` (fuller form) — the parser's `source` field
/// always holds the requirement-being-satisfied expression (`target` holds
/// the post-`by` subject in the fuller form, or mirrors `source` for the
/// shorthand). A negated (`not satisfy ...`) or inline-declared
/// (`satisfy requirement myReq : Type ...`) statement isn't a reference to an
/// existing target, so neither maps here.
fn satisfy_target(s: &sysml_v2_parser::SatisfyRequirementUsage) -> Option<String> {
    if s.not_span.is_some() {
        return None;
    }
    match &s.requirement {
        sysml_v2_parser::SatisfiedRequirement::Reference { reference } => plain_ref_string(*reference),
        sysml_v2_parser::SatisfiedRequirement::Declaration(_) => None,
    }
}

/// The `Requirement` reference a `verify` statement targets — the shorthand
/// `verify 'REQ-X';` form's `target` field is already a plain, quote-stripped
/// string. The fuller `verify requirement <inline> : Type ...` form declares
/// a fresh inline requirement usage rather than referencing an existing one
/// (`target` is `None` there), so it isn't mapped.
fn verify_target(v: &sysml_v2_parser::ast::VerifyRequirementMember) -> Option<String> {
    v.target.map(qr)
}

/// The string value of the member named `key` inside an `AttributeBody` —
/// e.g. `value = 'software';` or `value = "software";` or `featureId =
/// 'FEAT-ROTOR';`. Shared by every `@Syscribe*` annotation reader below.
/// Accepts both forms a real `.sysml` author might reach for: a
/// single-quoted "restricted name" token (`Expression::FeatureRef`, already
/// quote-stripped by the parser's own lexer — the same shape a
/// `satisfy`/`verify` shorthand target uses) and an ordinary double-quoted
/// string literal (`Expression::LiteralString`). `featureId` has only ever
/// been written single-quoted in this codebase's own examples/tests, but
/// nothing in the SysML v2 grammar forbids the double-quoted form for any of
/// these annotations, and silently dropping a syntactically valid value
/// (confirmed empirically: `value = "software";` produced no `domain:` and
/// no diagnostic at all before this fixed) would be a usability trap.
fn attribute_body_string(body: &sysml_v2_parser::ast::MetadataBody, key: &str) -> Option<String> {
    metadata_body_value(body, key).and_then(|e| match &e.value {
        sysml_v2_parser::Expression::LiteralString(s) => with_cur(|d| d.decoded_string_literal(*s).map(|c| c.into_owned())).flatten(),
        other => feature_ref_string(other),
    })
}

/// The value expression of the member named `key` in a metadata body. 0.57 models
/// `featureId = '...';` as a `MetadataBodyUsage` (a redefinition of the metadata type's feature);
/// an attribute-usage spelling (`attribute featureId = ...;`) is read the same way.
fn metadata_body_value<'a>(
    body: &'a sysml_v2_parser::ast::MetadataBody,
    key: &str,
) -> Option<&'a sysml_v2_parser::Node<sysml_v2_parser::Expression>> {
    use sysml_v2_parser::ast::MetadataBodyElement as M;
    body.members().find_map(|n| match &n.value {
        M::Usage(u) if qr(u.value.target) == key => u.value.value.as_ref().map(|fv| &fv.value.expression),
        M::Definition(d) => match &d.value {
            sysml_v2_parser::ast::AttributeBodyElement::AttributeUsage(a) if odn(a.value.name).as_deref() == Some(key) => {
                a.value.value.as_ref().map(|fv| &fv.value.expression)
            }
            _ => None,
        },
        _ => None,
    })
}

/// The integer value of the member named `key` inside an `AttributeBody` —
/// e.g. `sil = 2;` or `sil = -1;`. Unlike [`attribute_body_string`]'s
/// quoted/restricted-name values, a bare integer parses as
/// `Expression::LiteralInteger`; a negative one parses one level deeper, as
/// `Expression::UnaryOp { op: Minus, operand: LiteralInteger }` — the parser
/// has no negative-integer-literal token of its own, only unary minus
/// applied to a positive one. A non-integer numeric form (`sil = 2.5;`,
/// `Expression::LiteralReal`) isn't handled: `silLevel` is inherently an
/// integer scale (1-4), so there's no sensible truncate-or-round value to
/// recover, and it's left to silently produce no `silLevel:` — same as any
/// other malformed/unrecognized annotation value (see module-level note on
/// this class of gap).
fn attribute_body_i64(body: &sysml_v2_parser::ast::MetadataBody, key: &str) -> Option<i64> {
    metadata_body_value(body, key).and_then(|e| match &e.value {
        sysml_v2_parser::Expression::LiteralInteger(i) => Some(*i),
        sysml_v2_parser::Expression::UnaryOp {
            op: sysml_v2_parser::ast::UnaryOperator::Minus,
            operand,
        } => match &operand.value {
            sysml_v2_parser::Expression::LiteralInteger(i) => Some(-*i),
            _ => None,
        },
        _ => None,
    })
}

/// The `FeatureDef` reference from a `@SyscribeFeature { featureId = '...';
/// }` metadata annotation (`REQ-TRS-SYSMLV2-005`), or `None` if `m` isn't one
/// (wrong name) or carries no `featureId` member.
///
/// Confirmed against the parser's actual AST: `@Name { ... }` is a real,
/// structurally parseable `MetadataAnnotation` (`name`, `body: AttributeBody`)
/// — not a comment convention — and `featureId = '<FEAT-id>'` inside it is an
/// ordinary `AttributeUsage` whose value expression is a quote-stripped
/// `FeatureRef`, exactly like a `satisfy`/`verify` shorthand target.
fn syscribe_feature_id(m: &sysml_v2_parser::ast::MetadataAnnotation) -> Option<String> {
    if qr(m.type_reference) != "SyscribeFeature" {
        return None;
    }
    attribute_body_string(&m.body, "featureId")
}

/// Fields lifted from a fixed set of `@Syscribe*` metadata annotations on a
/// `part def`/`part` body (`REQ-TRS-SYSMLV2-008`) — domain classification,
/// integrity level, a display shortName, and an implementedBy source path.
/// Mirrors `@SyscribeFeature`'s precedent (same `MetadataAnnotation` AST
/// node, matched by name) but as a fixed, named set rather than a single
/// field, per `ADR-SYS-SYSMLV2-001`'s addendum.
#[derive(Default)]
struct SyscribeMeta {
    domain: Option<String>,
    asil_level: Option<String>,
    sil_level: Option<u8>,
    pl_level: Option<String>,
    short_name: Option<String>,
    implemented_by: Option<Vec<String>>,
}

/// Fold one metadata annotation into `meta` if its name matches one of the
/// four `REQ-TRS-SYSMLV2-008` annotations; any other name (including
/// `@SyscribeFeature`, handled separately by [`syscribe_feature_id`])
/// contributes nothing. `@SyscribeIntegrity` reads all three of
/// `asil`/`sil`/`pl` independently — more than one present on the same
/// annotation is not rejected here; it's caught downstream by the exact same
/// `W006` `asilLevel`/`silLevel` mutual-exclusion check a hand-authored
/// element gets today, since both fields land on the synthesized element's
/// frontmatter exactly like a native one would.
fn fold_syscribe_meta_annotation(m: &sysml_v2_parser::ast::MetadataAnnotation, meta: &mut SyscribeMeta) {
    match qr(m.type_reference).as_str() {
        "SyscribeDomain" => {
            if let Some(v) = attribute_body_string(&m.body, "value") {
                meta.domain = Some(v);
            }
        }
        "SyscribeIntegrity" => {
            if let Some(v) = attribute_body_string(&m.body, "asil") {
                meta.asil_level = Some(v);
            }
            if let Some(v) = attribute_body_i64(&m.body, "sil") {
                // Saturate rather than `u8::try_from(v).ok()`, which would
                // silently drop the whole field for any out-of-`u8`-range
                // value (confirmed empirically: `sil = 999;` produced no
                // `silLevel:` and no diagnostic at all before this fix) — a
                // hand-authored `silLevel: 999` at least reaches the
                // existing `E009` "out of range 1-4" check downstream, and
                // saturating here lets a too-large SysMLv2-authored value
                // reach that same check instead of vanishing.
                meta.sil_level = Some(v.clamp(0, u8::MAX as i64) as u8);
            }
            if let Some(v) = attribute_body_string(&m.body, "pl") {
                meta.pl_level = Some(v);
            }
        }
        "SyscribeShortName" => {
            if let Some(v) = attribute_body_string(&m.body, "value") {
                meta.short_name = Some(v);
            }
        }
        "SyscribeImplementedBy" => {
            if let Some(v) = attribute_body_string(&m.body, "path") {
                meta.implemented_by = Some(vec![v]);
            }
        }
        _ => {}
    }
}

/// A recognized `REQ-TRS-SYSMLV2-014` doc-comment directive prefix (with its
/// trailing colon) for `interface def`/`port def`/`connection def` — the
/// three element kinds whose body grammars carry no `MetadataAnnotation`
/// slot for the real `@Name { field = value; }` form `REQ-TRS-SYSMLV2-008`
/// establishes for `part def`/`part` (confirmed by direct inspection of the
/// vendored `sysml-v2-parser` source; see the ADR addendum). A directive line
/// spells the same four field names as a colon-suffixed comment line instead
/// of a structural annotation.
const DOC_DIRECTIVE_PREFIXES: &[&str] =
    &["@SyscribeDomain:", "@SyscribeIntegrity:", "@SyscribeShortName:", "@SyscribeImplementedBy:"];

/// Fold one recognized `REQ-TRS-SYSMLV2-014` doc-comment directive's value
/// into `meta`. `prefix` is one of [`DOC_DIRECTIVE_PREFIXES`]; `value` is the
/// raw text after the colon (not yet trimmed). Mirrors
/// `fold_syscribe_meta_annotation`'s per-name field semantics, reading a
/// plain string value instead of an `AttributeBody`. `@SyscribeIntegrity`
/// accepts a comma-separated `key=value` list (`asil`/`sil`/`pl`), the
/// doc-comment analogue of that annotation's three independent keys; an
/// unparseable `sil=...` (non-integer) is silently skipped, same posture as
/// `attribute_body_i64` returning `None` for the real-annotation form.
fn fold_syscribe_doc_directive(prefix: &str, value: &str, meta: &mut SyscribeMeta) {
    let value = value.trim();
    if value.is_empty() {
        return;
    }
    match prefix {
        "@SyscribeDomain:" => meta.domain = Some(value.to_string()),
        "@SyscribeShortName:" => meta.short_name = Some(value.to_string()),
        "@SyscribeImplementedBy:" => meta.implemented_by = Some(vec![value.to_string()]),
        "@SyscribeIntegrity:" => {
            for kv in value.split(',') {
                let Some((k, v)) = kv.split_once('=') else { continue };
                let (k, v) = (k.trim(), v.trim());
                if v.is_empty() {
                    continue;
                }
                match k {
                    "asil" => meta.asil_level = Some(v.to_string()),
                    // Saturate rather than drop, mirroring `attribute_body_i64`'s
                    // own `sil = 999;` handling: let an out-of-range value reach
                    // the existing E009 check downstream instead of vanishing.
                    "sil" => {
                        if let Ok(n) = v.parse::<i64>() {
                            meta.sil_level = Some(n.clamp(0, u8::MAX as i64) as u8);
                        }
                    }
                    "pl" => meta.pl_level = Some(v.to_string()),
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

/// Collapse runs of 2+ consecutive blank lines down to exactly one — tidies
/// up the gap a removed directive line can leave behind in
/// [`extract_syscribe_doc_directives`]'s output (a directive on its own line
/// between two prose paragraphs would otherwise leave a double blank line
/// once stripped).
fn collapse_blank_lines(s: &str) -> String {
    let mut out = String::new();
    let mut blank_run = false;
    for line in s.lines() {
        let is_blank = line.trim().is_empty();
        if is_blank {
            if blank_run {
                continue;
            }
            blank_run = true;
        } else {
            blank_run = false;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// `REQ-TRS-SYSMLV2-014`: scan already-lifted `doc` text (the output of
/// `REQ-TRS-SYSMLV2-009`'s `collect_doc`) for directive lines, stripping each
/// recognized one out of the returned doc text and folding its value into a
/// `SyscribeMeta`. A line that doesn't start with one of
/// [`DOC_DIRECTIVE_PREFIXES`] (after trimming) is left in the doc text
/// untouched — it's prose, not a directive. A later directive line for the
/// same field overrides an earlier one, matching
/// `fold_syscribe_meta_annotation`'s last-annotation-wins behavior for the
/// real annotation form.
fn extract_syscribe_doc_directives(doc: &str) -> (String, SyscribeMeta) {
    let mut meta = SyscribeMeta::default();
    let mut kept_lines: Vec<&str> = Vec::new();
    for line in doc.lines() {
        let trimmed = line.trim();
        if let Some(prefix) = DOC_DIRECTIVE_PREFIXES.iter().find(|p| trimmed.starts_with(**p)) {
            fold_syscribe_doc_directive(prefix, &trimmed[prefix.len()..], &mut meta);
            continue;
        }
        kept_lines.push(line);
    }
    (collapse_blank_lines(&kept_lines.join("\n")).trim().to_string(), meta)
}

/// Concatenate every `doc /* ... */` member's text found in `elements`, in
/// source order, joined by a blank line — `REQ-TRS-SYSMLV2-009`'s
/// requirement that a second/third `doc` block accumulates rather than only
/// the first (or last) winning. `as_doc` extracts the doc text from one
/// body-element node's value; each of the several relevant body-element
/// enums below is a structurally distinct Rust type sharing this one
/// `Doc(Node<DocComment>)` variant shape, not a common trait, so each gets
/// its own thin one-line wrapper around this shared fold.
///
/// Each block's text is `.trim()`ed individually before joining —
/// `sysml-v2-parser` includes the incidental whitespace directly adjacent to
/// `/*`/`*/` verbatim (e.g. `doc /* Explanation. */` parses to `"
/// Explanation. "`, not `"Explanation."`), which is delimiter padding, not
/// meaningful content; internal formatting/newlines within a single block
/// are left untouched. A block that trims to nothing (`doc /* */`, or
/// whitespace-only) is dropped entirely, not kept as an empty entry — a
/// review caught that the naive version left a stray leading/embedded blank
/// line (`"\n\nReal text."`) when an earlier block trimmed empty, which
/// would have been a real, if minor, defect in the lifted `doc` field.
fn collect_doc<T>(elements: &[sysml_v2_parser::Node<T>], as_doc: impl Fn(&T) -> Option<String>) -> String {
    elements
        .iter()
        .filter_map(|n| as_doc(&n.value))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// The text of a `doc /* ... */` annotating member (`REQ-TRS-SYSMLV2-009`): the authored comment
/// body without its delimiters, exactly what 0.54 exposed as `DocComment::text`. Other annotating
/// members (`comment`, `rep`, `@Metadata`) are not doc text.
fn annotating_doc(a: &sysml_v2_parser::ast::AnnotatingMember) -> Option<String> {
    match a {
        sysml_v2_parser::ast::AnnotatingMember::Doc(d) => {
            with_cur(|doc| doc.comment_body(d.value.body).map(str::to_string)).flatten()
        }
        _ => None,
    }
}

/// The `@Name { ... }` metadata annotation of an annotating member, if it is one.
fn annotating_meta(a: &sysml_v2_parser::ast::AnnotatingMember) -> Option<&sysml_v2_parser::ast::MetadataAnnotation> {
    match a {
        sysml_v2_parser::ast::AnnotatingMember::MetadataAnnotation(m) => Some(&m.value),
        _ => None,
    }
}

// ── Metadata applications (REQ-TRS-SYSMLV2-086/-087) ───────────────────────

/// One metadata application member of a body: the `@T { ... }` / `@n : T` / `@T about Y` form,
/// or the standalone `#T;` / `#T { ... }` keyword form.
enum MetaMember<'a> {
    Annotation(&'a sysml_v2_parser::ast::MetadataAnnotation),
    Keyword(&'a sysml_v2_parser::ast::MetadataKeywordUsage),
}

/// A body-element enum that can carry a metadata application member.
trait MetaBody {
    fn meta_member(&self) -> Option<MetaMember<'_>>;
}

macro_rules! impl_meta_body {
    ($t:ty, keyword) => {
        impl MetaBody for $t {
            fn meta_member(&self) -> Option<MetaMember<'_>> {
                match self {
                    Self::Annotating(a) => annotating_meta(a).map(MetaMember::Annotation),
                    Self::MetadataKeywordUsage(k) => Some(MetaMember::Keyword(&k.value)),
                    _ => None,
                }
            }
        }
    };
    ($t:ty) => {
        impl MetaBody for $t {
            fn meta_member(&self) -> Option<MetaMember<'_>> {
                match self {
                    Self::Annotating(a) => annotating_meta(a).map(MetaMember::Annotation),
                    _ => None,
                }
            }
        }
    };
}

impl_meta_body!(sysml_v2_parser::PartDefBodyElement, keyword);
impl_meta_body!(sysml_v2_parser::PartUsageBodyElement, keyword);
impl_meta_body!(sysml_v2_parser::PortDefBodyElement, keyword);
impl_meta_body!(sysml_v2_parser::PortBodyElement);
impl_meta_body!(sysml_v2_parser::ConnectionDefBodyElement, keyword);
impl_meta_body!(sysml_v2_parser::InterfaceDefBodyElement, keyword);
impl_meta_body!(sysml_v2_parser::InterfaceUsageBodyElement);
impl_meta_body!(sysml_v2_parser::ast::AttributeBodyElement, keyword);
impl_meta_body!(sysml_v2_parser::ast::StateDefBodyElement, keyword);
impl_meta_body!(sysml_v2_parser::ActionDefBodyElement, keyword);
impl_meta_body!(sysml_v2_parser::ActionUsageBodyElement, keyword);
impl_meta_body!(sysml_v2_parser::RequirementDefBodyElement, keyword);
impl_meta_body!(sysml_v2_parser::ast::UseCaseDefBodyElement, keyword);
impl_meta_body!(sysml_v2_parser::ast::ViewDefBodyElement, keyword);
impl_meta_body!(sysml_v2_parser::ast::ViewBodyElement);
impl_meta_body!(sysml_v2_parser::ast::RenderingDefBodyElement);
impl_meta_body!(sysml_v2_parser::ast::RenderingUsageBodyElement);
impl_meta_body!(sysml_v2_parser::ast::OccurrenceBodyElement, keyword);
impl_meta_body!(sysml_v2_parser::ast::RelationshipBodyElement);
impl_meta_body!(sysml_v2_parser::ast::ConstraintDefBodyElement, keyword);
impl_meta_body!(sysml_v2_parser::ast::CalcDefBodyElement, keyword);

impl MetaBody for sysml_v2_parser::ast::DefinitionBodyElement {
    fn meta_member(&self) -> Option<MetaMember<'_>> {
        match self {
            Self::OccurrenceMember(m) => m.value.meta_member(),
            _ => None,
        }
    }
}

/// Whether a metadata type reference is one of the reserved `@Syscribe*` annotations, which
/// keep their own fixed field lift and never appear in `metadata:`.
fn is_syscribe_annotation(type_ref: &str) -> bool {
    type_ref.rsplit("::").next().is_some_and(|s| s.starts_with("Syscribe"))
}

/// The YAML value of one tagged value: a literal keeps its type (`3`, `2.5`, `true`, `"text"`,
/// a quoted restricted name `'text'`); any other expression is carried as its rendered text.
fn metadata_literal(e: &sysml_v2_parser::Expression) -> serde_yaml::Value {
    use sysml_v2_parser::Expression as E;
    match e {
        E::LiteralInteger(i) => serde_yaml::Value::Number((*i).into()),
        E::LiteralReal(_) => {
            let text = render_expression(e);
            text.parse::<f64>().map_or(serde_yaml::Value::String(text), |f| serde_yaml::Value::Number(f.into()))
        }
        E::LiteralBoolean(b) => serde_yaml::Value::Bool(*b),
        E::LiteralString(_) | E::FeatureRef(_) => serde_yaml::Value::String(attribute_body_text(e).unwrap_or_else(|| render_expression(e))),
        E::UnaryOp { op: sysml_v2_parser::ast::UnaryOperator::Minus, operand } => match &operand.value {
            E::LiteralInteger(i) => serde_yaml::Value::Number((-*i).into()),
            _ => serde_yaml::Value::String(render_expression(e)),
        },
        other => serde_yaml::Value::String(render_expression(other)),
    }
}

/// The decoded text of a string literal or quoted/bare name expression.
fn attribute_body_text(e: &sysml_v2_parser::Expression) -> Option<String> {
    match e {
        sysml_v2_parser::Expression::LiteralString(s) => with_cur(|d| d.decoded_string_literal(*s).map(|c| c.into_owned())).flatten(),
        other => feature_ref_string(other),
    }
}

/// Every `k = v;` member of a `MetadataBody`, in source order.
fn metadata_body_values(body: &sysml_v2_parser::ast::MetadataBody) -> Vec<(String, serde_yaml::Value)> {
    use sysml_v2_parser::ast::MetadataBodyElement as M;
    body.members()
        .filter_map(|n| match &n.value {
            M::Usage(u) => u.value.value.as_ref().map(|fv| (qr(u.value.target), metadata_literal(&fv.value.expression.value))),
            M::Definition(d) => match &d.value {
                sysml_v2_parser::ast::AttributeBodyElement::AttributeUsage(a) => {
                    a.value.value.as_ref().map(|fv| (a.value.name.s(), metadata_literal(&fv.value.expression.value)))
                }
                _ => None,
            },
            _ => None,
        })
        .filter(|(k, _)| !k.is_empty())
        .collect()
}

/// Every `k = v;` member of a `#T { ... }` keyword body (an `AttributeBody`).
fn attribute_body_values(body: &sysml_v2_parser::AttributeBody) -> Vec<(String, serde_yaml::Value)> {
    let sysml_v2_parser::AttributeBody::Brace { elements, .. } = body else { return Vec::new() };
    elements
        .iter()
        .filter_map(|n| match &n.value {
            sysml_v2_parser::ast::AttributeBodyElement::AttributeUsage(a) => {
                a.value.value.as_ref().map(|fv| (a.value.name.s(), metadata_literal(&fv.value.expression.value)))
            }
            _ => None,
        })
        .filter(|(k, _)| !k.is_empty())
        .collect()
}

/// Build the `metadata:` entries of one application: `{type, name?, <values>}`, plus `about:`
/// (one entry per written target) when the application names targets -- resolved, moved or
/// counted by [`resolve_metadata`] once the whole subtree is merged.
fn metadata_entries(type_ref: String, name: Option<String>, values: Vec<(String, serde_yaml::Value)>, about: Vec<String>) -> Vec<serde_yaml::Value> {
    if type_ref.is_empty() || is_syscribe_annotation(&type_ref) {
        return Vec::new();
    }
    let base = |about: Option<&str>| {
        let mut m = serde_yaml::Mapping::new();
        m.insert(ykey("type"), ykey(&type_ref));
        if let Some(n) = name.as_deref().filter(|n| !n.is_empty()) {
            m.insert(ykey("name"), ykey(n));
        }
        if let Some(a) = about {
            m.insert(ykey("about"), ykey(a));
        }
        for (k, v) in &values {
            if k != "type" && k != "name" && k != "about" {
                m.insert(ykey(k), v.clone());
            }
        }
        serde_yaml::Value::Mapping(m)
    };
    if about.is_empty() {
        vec![base(None)]
    } else {
        about.iter().map(|a| base(Some(a))).collect()
    }
}

/// The `metadata:` entries of one `@`/`#` body member (`REQ-TRS-SYSMLV2-086`).
fn meta_member_entries(m: MetaMember<'_>) -> Vec<serde_yaml::Value> {
    match m {
        MetaMember::Annotation(a) => metadata_entries(
            qr(a.type_reference),
            a.declared_name.as_ref().and_then(|d| ident_name(&d.value.identification)),
            metadata_body_values(&a.body),
            a.about_targets.iter().map(|t| qr(*t)).collect(),
        ),
        // A `#T` with no body of its own is a prefix on the member that follows, not an application
        // on the holder: `convert_merged` applies it at package level, `convert_body_with_prefixes`
        // inside a body (`REQ-TRS-SYSMLV2-099`).
        MetaMember::Keyword(k) => match &k.body {
            Some(body) => metadata_entries(qr(k.reference), None, attribute_body_values(body), Vec::new()),
            None => Vec::new(),
        },
    }
}

/// `REQ-TRS-SYSMLV2-099`: the `#T` prefixes the parser keeps on a usage's own occurrence prefix
/// (`#Safety part a : A;` for `part`/`item`/`port`/`connection`/`occurrence`/`constraint`/`view`/
/// analysis-case usages), as `{type: T}` entries ahead of the body's own applications.
fn prefix_keyword_entries(prefix: &sysml_v2_parser::ast::OccurrenceUsagePrefix) -> Vec<serde_yaml::Value> {
    prefix
        .extension_keywords
        .iter()
        .flat_map(|k| metadata_entries(qr(k.value.annotation), None, Vec::new(), Vec::new()))
        .collect()
}

/// [`prefix_keyword_entries`] followed by `body`.
fn with_prefix_keywords(prefix: &sysml_v2_parser::ast::OccurrenceUsagePrefix, body: Vec<serde_yaml::Value>) -> Vec<serde_yaml::Value> {
    let mut v = prefix_keyword_entries(prefix);
    v.extend(body);
    v
}

/// `REQ-TRS-SYSMLV2-099`: convert a body's members in order, applying a bodiless `#T` member
/// (the shape the parser keeps for a prefix on an `attribute`/`action`/`state`/`interface`/
/// `requirement`/`flow`/`allocation`/`ref` usage) as `{type: T}` on the next member that
/// synthesizes an element — the same rule `convert_merged` applies at package level. A prefix
/// whose member produces no element is dropped.
fn convert_body_with_prefixes<T: MetaBody>(
    elements: &[sysml_v2_parser::Node<T>],
    out: &mut Vec<RawElement>,
    mut convert: impl FnMut(&T, &mut Vec<RawElement>),
) {
    let mut prefix: Option<String> = None;
    for node in elements {
        if let Some(MetaMember::Keyword(k)) = node.value.meta_member() {
            if k.body.is_none() {
                prefix = Some(qr(k.reference));
                continue;
            }
        }
        let before = out.len();
        convert(&node.value, out);
        if let Some(type_ref) = prefix.take() {
            if out.len() > before {
                let entries = metadata_entries(type_ref, None, Vec::new(), Vec::new());
                out[before].frontmatter.metadata.get_or_insert_with(Vec::new).extend(entries);
            }
        }
    }
}

/// `metadata:` entries of a `RequirementDefBody`-shaped body (requirement, viewpoint, concern).
fn requirement_def_body_metadata(body: &sysml_v2_parser::RequirementDefBody) -> Vec<serde_yaml::Value> {
    match body {
        sysml_v2_parser::RequirementDefBody::Brace { elements, .. } => body_metadata(elements),
        _ => Vec::new(),
    }
}

/// `metadata:` entries of a thin `DefinitionBody` (flow, allocation def, occurrence def).
fn definition_body_metadata(body: &sysml_v2_parser::ast::DefinitionBody) -> Vec<serde_yaml::Value> {
    match body {
        sysml_v2_parser::ast::DefinitionBody::Brace { elements, .. } => body_metadata(elements),
        _ => Vec::new(),
    }
}

/// Lift every metadata application member of a body into `metadata:` entries, in source order.
fn body_metadata<T: MetaBody>(elements: &[sysml_v2_parser::Node<T>]) -> Vec<serde_yaml::Value> {
    elements.iter().filter_map(|n| n.value.meta_member()).flat_map(meta_member_entries).collect()
}

/// `REQ-TRS-SYSMLV2-087`: a package-level `metadata m : T about Y { ... }` / `metadata T about Y;`
/// usage. A usage with no `:` typing has only one qualified name, and the grammar's required
/// `OwnedFeatureTyping` makes it the type, so the parser's `name` is read as the type then.
fn metadata_usage_entries(u: &sysml_v2_parser::ast::MetadataUsage) -> Vec<serde_yaml::Value> {
    let (type_ref, name) = match u.type_reference {
        Some(t) => (qr(t), Some(dn(u.name))),
        None => (dn(u.name), None),
    };
    metadata_entries(type_ref, name, metadata_body_values(&u.body), u.about_targets.iter().map(|t| qr(*t)).collect())
}

/// The `metadata:` entries declared directly in one merged package's body (`@T;`, `#T { }`,
/// `metadata m : T;`, with or without `about`).
fn package_metadata(merged: &MergedPackage) -> Vec<serde_yaml::Value> {
    package_metadata_by_file(merged).into_iter().map(|(entry, _)| entry).collect()
}

/// [`package_metadata`] paired with the file each entry was declared in.
fn package_metadata_by_file(merged: &MergedPackage) -> Vec<(serde_yaml::Value, String)> {
    use sysml_v2_parser::PackageBodyElement as E;
    merged
        .body
        .iter()
        .flat_map(|(e, file, doc)| {
            let entries = with_doc(doc, || match e {
                E::Annotating(a) => annotating_meta(a).map(MetaMember::Annotation).map(meta_member_entries).unwrap_or_default(),
                E::MetadataKeywordUsage(k) => meta_member_entries(MetaMember::Keyword(&k.value)),
                E::MetadataUsage(u) => metadata_usage_entries(&u.value),
                _ => Vec::new(),
            });
            entries.into_iter().map(move |entry| (entry, file.clone()))
        })
        .collect()
}

/// `REQ-TRS-SYSMLV2-098`: the file root's metadata applications are the anchor package's own,
/// resolved exactly like [`resolve_metadata`] does for a synthesized element — `type:` rewritten
/// to the ingested `MetadataDef`, a resolvable `about` target receiving the entry itself, an
/// unresolved one kept on `owner` and counted per file for `W543`. Runs after
/// [`resolve_metadata`], so the index it resolves against is complete.
fn lift_root_metadata(
    merged: &MergedPackage,
    pkg_qname: &str,
    owner: &mut RawElement,
    out: &mut [RawElement],
) -> BTreeMap<String, usize> {
    let mut unresolved = BTreeMap::new();
    let entries = package_metadata_by_file(merged);
    if entries.is_empty() {
        return unresolved;
    }
    let (index, defs) = metadata_indices(out);
    for (entry, file) in entries {
        match resolve_metadata_entry(entry, pkg_qname, &index, &defs) {
            ResolvedMetadata::Moved(target, entry) => {
                if let Some(t) = out.iter_mut().find(|e| e.qualified_name == target) {
                    t.frontmatter.metadata.get_or_insert_with(Vec::new).push(entry);
                }
            }
            ResolvedMetadata::Kept(entry, dangling) => {
                if dangling {
                    *unresolved.entry(file).or_insert(0) += 1;
                }
                owner.frontmatter.metadata.get_or_insert_with(Vec::new).push(entry);
            }
        }
    }
    unresolved
}

/// The two lookup indices [`resolve_metadata`] needs: every synthesized qname (for `about`
/// targets) and the `MetadataDef` qnames (for `type:`).
fn metadata_indices(out: &[RawElement]) -> (super::EndpointIndex, super::EndpointIndex) {
    let index: super::EndpointIndex = out.iter().map(|e| (e.qualified_name.clone(), (None, None))).collect();
    let defs: super::EndpointIndex = out
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::MetadataDef)))
        .map(|e| (e.qualified_name.clone(), (None, None)))
        .collect();
    (index, defs)
}

/// What [`resolve_metadata_entry`] decided for one `metadata:` entry.
enum ResolvedMetadata {
    /// The entry's `about` target resolved: it moves onto that element (`about:` removed).
    Moved(String, serde_yaml::Value),
    /// The entry stays on its holder; `true` when it carries an `about:` that did not resolve.
    Kept(serde_yaml::Value, bool),
}

/// Resolve one entry from `holder`'s scope: `type:` rewritten to the `MetadataDef` it resolves to
/// (kept as written otherwise), then the `about` target looked up the same way.
fn resolve_metadata_entry(
    entry: serde_yaml::Value,
    holder: &str,
    index: &super::EndpointIndex,
    defs: &super::EndpointIndex,
) -> ResolvedMetadata {
    let serde_yaml::Value::Mapping(mut m) = entry else {
        return ResolvedMetadata::Kept(entry, false);
    };
    if let Some(t) = m.get(ykey("type")).and_then(|v| v.as_str()).map(str::to_string) {
        if let Some(q) = super::lookup_scoped(defs, holder, &t) {
            m.insert(ykey("type"), ykey(&q));
        }
    }
    let about = m.get(ykey("about")).and_then(|v| v.as_str()).map(str::to_string);
    match about.as_deref().and_then(|a| super::lookup_scoped(index, holder, a)) {
        Some(target) => {
            m.remove(ykey("about"));
            ResolvedMetadata::Moved(target, serde_yaml::Value::Mapping(m))
        }
        None => ResolvedMetadata::Kept(serde_yaml::Value::Mapping(m), about.is_some()),
    }
}

/// `REQ-TRS-SYSMLV2-086`/`-087`: after the whole subtree is merged, rewrite each `metadata:`
/// entry's `type:` to the qualified name of the ingested `MetadataDef` it resolves to
/// (innermost-scope-first from the holder; kept as written otherwise), and move each entry
/// carrying an `about:` onto the element that target resolves to. An unresolved target keeps
/// the entry on the holder, `about:` intact, and is counted per file for `W543`.
fn resolve_metadata(out: &mut [RawElement]) -> BTreeMap<String, usize> {
    let (index, defs) = metadata_indices(out);
    let mut unresolved = BTreeMap::new();
    let mut moved: Vec<(String, serde_yaml::Value)> = Vec::new();
    for e in out.iter_mut() {
        let Some(entries) = e.frontmatter.metadata.take() else { continue };
        let holder = e.qualified_name.clone();
        let mut kept = Vec::new();
        for entry in entries {
            match resolve_metadata_entry(entry, &holder, &index, &defs) {
                ResolvedMetadata::Moved(target, entry) => moved.push((target, entry)),
                ResolvedMetadata::Kept(entry, dangling) => {
                    if dangling {
                        *unresolved.entry(e.file_path.clone()).or_insert(0) += 1;
                    }
                    kept.push(entry);
                }
            }
        }
        e.frontmatter.metadata = nonempty_vec(kept);
    }
    for (target, entry) in moved {
        if let Some(t) = out.iter_mut().find(|e| e.qualified_name == target) {
            t.frontmatter.metadata.get_or_insert_with(Vec::new).push(entry);
        }
    }
    unresolved
}

/// `doc /* ... */` lift over a `part def` body's already-sliced members.
fn part_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::PartDefBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `part` usage body's already-sliced members.
fn part_usage_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartUsageBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::PartUsageBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `port def` body's already-sliced members.
fn port_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::PortDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::PortDefBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `port` usage body's already-sliced members.
fn port_usage_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::PortBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::PortBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `doc /* ... */` lift over an `interface` usage's already-sliced
/// `body_elements` — a review caught this one missing entirely from the
/// first version of this module: `InterfaceUsageBodyElement` carries its own
/// `Doc` variant (distinct from `InterfaceDefBodyElement`, which
/// [`interface_def_doc`] handles), and `InterfaceUsage`'s three variants
/// (`TypedConnect`/`Connection`/`Declaration`) all carry `body_elements` of
/// this type — but only `Declaration` is a synthesized element at all
/// (`TypedConnect`/`Connection` are anonymous binary connectors, out of
/// scope per [`convert_interface_usage`]'s own doc comment).
fn interface_usage_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::InterfaceUsageBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::InterfaceUsageBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `connection def` body's already-sliced members.
fn connection_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ConnectionDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ConnectionDefBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `doc /* ... */` lift over an `interface def` body's already-sliced members.
fn interface_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::InterfaceDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::InterfaceDefBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `doc /* ... */` lift over an `AttributeBody`'s already-sliced members —
/// shared by `attribute def`, `attribute` usage, and `item def`, which all
/// three use this exact enum for their own body (`ItemDef.body`'s doc
/// comment confirmed: `AttributeBody`/`AttributeBodyElement` shared with
/// `AttributeDef`/`AttributeUsage`, not a distinct item-specific shape).
fn attribute_body_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::AttributeBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::AttributeBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `state def`/`state` usage body's already-sliced
/// members (`REQ-TRS-SYSMLV2-018`) — shared by both `StateDef` and
/// `StateUsage`, since they use the same `StateDefBody`/`StateDefBodyElement`
/// grammar (confirmed against the parser's AST: `StateUsage.body: StateDefBody`).
fn state_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::StateDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::StateDefBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `REQ-TRS-SYSMLV2-018` `@Syscribe*` fixed-field search over a `state
/// def`/`state` usage body's already-sliced members. See
/// [`part_def_syscribe_meta`].
fn state_def_syscribe_meta(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::StateDefBodyElement>]) -> SyscribeMeta {
    let mut meta = SyscribeMeta::default();
    for n in elements {
        if let sysml_v2_parser::ast::StateDefBodyElement::Annotating(a) = &n.value {
            if let Some(m) = annotating_meta(a) {
                fold_syscribe_meta_annotation(m, &mut meta);
            }
        }
    }
    meta
}

/// `doc /* ... */` lift over an `action def` body's already-sliced members
/// (`REQ-TRS-SYSMLV2-019`).
fn action_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ActionDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ActionDefBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `REQ-TRS-SYSMLV2-019` `@Syscribe*` fixed-field search over an `action def`
/// body's already-sliced members.
fn action_def_syscribe_meta(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ActionDefBodyElement>]) -> SyscribeMeta {
    let mut meta = SyscribeMeta::default();
    for n in elements {
        if let sysml_v2_parser::ActionDefBodyElement::Annotating(a) = &n.value {
            if let Some(m) = annotating_meta(a) {
                fold_syscribe_meta_annotation(m, &mut meta);
            }
        }
    }
    meta
}

/// `doc /* ... */` lift over an `action` usage body's already-sliced members
/// (`REQ-TRS-SYSMLV2-019`) — `ActionUsageBodyElement` is a distinct Rust type
/// from `ActionDefBodyElement` (structurally near-identical, but not shared),
/// so this needs its own wrapper, mirroring `part_def_doc`/`part_usage_doc`.
fn action_usage_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ActionUsageBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ActionUsageBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `REQ-TRS-SYSMLV2-019` `@Syscribe*` fixed-field search over an `action`
/// usage body's already-sliced members.
fn action_usage_syscribe_meta(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ActionUsageBodyElement>]) -> SyscribeMeta {
    let mut meta = SyscribeMeta::default();
    for n in elements {
        if let sysml_v2_parser::ActionUsageBodyElement::Annotating(a) = &n.value {
            if let Some(m) = annotating_meta(a) {
                fold_syscribe_meta_annotation(m, &mut meta);
            }
        }
    }
    meta
}

/// `doc /* ... */` lift over a `view def` body's already-sliced members
/// (`REQ-TRS-SYSMLV2-020`).
fn view_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::ViewDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::ViewDefBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `view` usage body's already-sliced members
/// (`REQ-TRS-SYSMLV2-020`) — `ViewBodyElement` is a distinct Rust type from
/// `ViewDefBodyElement` (structurally near-identical, but not shared), so
/// this needs its own wrapper, mirroring `part_def_doc`/`part_usage_doc`.
fn view_usage_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::ViewBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::ViewBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `doc /* ... */` lift over any `RequirementDefBody`-shaped body —
/// `viewpoint def`/`viewpoint` usage (`REQ-TRS-SYSMLV2-021`) and
/// `concern def`/`concern` usage (`REQ-TRS-SYSMLV2-023`) both share this
/// exact type (confirmed against the parser's own AST: `ViewpointDef.body`/
/// `ViewpointUsage.body`/`ConcernUsage.body` are literally `RequirementDefBody`),
/// the same shape a plain `requirement def` uses — but `REQ-TRS-SYSMLV2-009`
/// deliberately did not extend doc-lifting to `Requirement`/`RequirementDef`/
/// `RequirementUsage`, so there is no existing collector here to reuse; this
/// one is new, originally scoped to Viewpoint and generalized in name only
/// when Concern needed the exact same thing.
fn requirement_def_body_doc(body: &sysml_v2_parser::RequirementDefBody) -> String {
    let sysml_v2_parser::RequirementDefBody::Brace { elements, .. } = body else {
        return String::new();
    };
    collect_doc(elements, |e| match e {
        sysml_v2_parser::RequirementDefBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `rendering def` body's already-sliced members
/// (`REQ-TRS-SYSMLV2-022`).
fn rendering_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::RenderingDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::RenderingDefBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `rendering` usage body's already-sliced
/// members (`REQ-TRS-SYSMLV2-022`).
fn rendering_usage_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::RenderingUsageBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::RenderingUsageBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    })
}

/// The `rendering:` reference text a `render <name> [: <Type>]` clause
/// contributes — `REQ-TRS-SYSMLV2-020`/`-022`. Prefers the referenced
/// type's name (`type_name`); falls back to the render clause's own `name`
/// when untyped (an inline/self-defining render). `None` for a fully
/// anonymous, untyped render clause — nothing meaningful to reference.
fn view_rendering_target(u: &sysml_v2_parser::ast::ViewRenderingUsage) -> Option<String> {
    oqr(u.type_name).or_else(|| match u.form {
        sysml_v2_parser::ast::ViewRenderingForm::Reference(id) => Some(qr(id)),
        sysml_v2_parser::ast::ViewRenderingForm::Inline(n) => Some(dn(n)).filter(|s| !s.is_empty()),
    })
}

/// First `render` clause's target text found in a `view def` body's
/// already-sliced members — `rendering:` is a single-string native field,
/// so only the first `render` clause (in source order) can be represented;
/// a second one is silently not represented, the same "single-string
/// field, first wins" posture `view_satisfy_viewpoint` uses for multiple
/// `satisfy` clauses.
fn view_def_rendering(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::ViewDefBodyElement>]) -> Option<String> {
    elements.iter().find_map(|n| match &n.value {
        sysml_v2_parser::ast::ViewDefBodyElement::ViewRendering(r) => view_rendering_target(&r.value),
        _ => None,
    })
}

/// Same as [`view_def_rendering`], for a `view` usage body.
fn view_usage_rendering(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::ViewBodyElement>]) -> Option<String> {
    elements.iter().find_map(|n| match &n.value {
        sysml_v2_parser::ast::ViewBodyElement::ViewRendering(r) => view_rendering_target(&r.value),
        _ => None,
    })
}

/// `expose:` entries lifted from a `view` usage body's `Expose` members —
/// `REQ-TRS-SYSMLV2-020`. Always a flat plain-string entry using
/// `ExposeMember.target` verbatim (which already includes any `::*`/`::**`
/// suffix textually), never the richer `{ref, isRecursive, filter}` map
/// form — matches both real hand-authored `expose:` lists in `model/`
/// (`model/Views/SystemArchitectureView.md`) and sidesteps a pre-existing,
/// unrelated `W502` inconsistency (its map-form branch reads a `ref` key,
/// while `spec/markdown-sysml-format.md` §8.14.3 documents `target` — see
/// this feature's ADR addendum). `ExposeMember.body`'s own brace content (if
/// any) and the BNF's optional `[ expr ]` filter suffix are both parsed and
/// discarded by the vendored parser itself before this crate ever sees them
/// — nothing to recover.
fn view_expose_entries(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::ViewBodyElement>]) -> Vec<serde_yaml::Value> {
    elements
        .iter()
        .filter_map(|n| match &n.value {
            sysml_v2_parser::ast::ViewBodyElement::Expose(e) => {
                Some(serde_yaml::Value::String(import_target_text(&e.value.target)))
            }
            _ => None,
        })
        .collect()
}

/// `viewpoint:` lifted from a `view` usage body's first `satisfy` clause —
/// `REQ-TRS-SYSMLV2-020`. Multiple `satisfy` clauses: first one wins,
/// matching the native field's own single-string shape.
fn view_satisfy_viewpoint(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::ViewBodyElement>]) -> Option<String> {
    elements.iter().find_map(|n| match &n.value {
        sysml_v2_parser::ast::ViewBodyElement::Satisfy(s) => match &s.value.requirement {
            sysml_v2_parser::SatisfiedRequirement::Reference { reference } => Some(qr(*reference)),
            sysml_v2_parser::SatisfiedRequirement::Declaration(_) => None,
        },
        _ => None,
    })
}

/// `stakeholders:`/`concerns:` lifted from any `RequirementDefBody`-shaped
/// body's `Stakeholder`/`Purpose` members — originally `REQ-TRS-SYSMLV2-021`
/// (`viewpoint def`/`viewpoint` usage, which uses both halves of the
/// returned tuple), generalized in name only for `REQ-TRS-SYSMLV2-023`
/// (`concern def`/`concern` usage, which uses only the `stakeholders` half —
/// `ConcernDef` has no `concerns:` self-field per §8.11.5, so its caller
/// discards the second element). `StakeholderMember.name` only (`type_name`/
/// `is_redefinition` have no native slot); `PurposeMember.target`, the
/// closest AST equivalent to §8.14.1's "concerns (qnames of ConcernDefs)".
/// `Frame` and every other `RequirementDefBodyElement` variant are unmapped
/// here — no native "framed concern" field exists.
fn collect_requirement_body_stakeholders_concerns(
    body: &sysml_v2_parser::RequirementDefBody,
) -> (Vec<String>, Vec<String>) {
    let sysml_v2_parser::RequirementDefBody::Brace { elements, .. } = body else {
        return (Vec::new(), Vec::new());
    };
    let mut stakeholders = Vec::new();
    let mut concerns = Vec::new();
    for n in elements {
        match &n.value {
            sysml_v2_parser::RequirementDefBodyElement::Stakeholder(s) => {
                stakeholders.push(odn(s.value.declaration_name).or_else(|| oqr(s.value.target)).unwrap_or_default());
            }
            sysml_v2_parser::RequirementDefBodyElement::Purpose(p) => {
                concerns.push(qr(p.value.target));
            }
            _ => {}
        }
    }
    (stakeholders, concerns)
}

/// `subject:` lifted from a `concern def`/`concern` usage body's
/// `SubjectDecl` member — `REQ-TRS-SYSMLV2-023`. Only the typed-declaration
/// form (`subject <name> : <Type>;`) carries anything to extract
/// (`SubjectDecl.type_name`); the bare `subject;` shorthand parses as an
/// empty `SubjectRef` node with no data at all, and is left unmapped.
fn concern_body_subject(body: &sysml_v2_parser::RequirementDefBody) -> Option<String> {
    let sysml_v2_parser::RequirementDefBody::Brace { elements, .. } = body else {
        return None;
    };
    elements.iter().find_map(|n| match &n.value {
        sysml_v2_parser::RequirementDefBodyElement::SubjectDecl(s) => {
            nonempty(typing_display(s.value.typing.as_ref()).unwrap_or_default())
        }
        _ => None,
    })
}

/// `target_display` of an optional typing relationship: the comma-joined target references.
fn typing_display(t: Option<&sysml_v2_parser::Node<sysml_v2_parser::ast::TypingRelationship>>) -> Option<String> {
    t.map(|t| refs_display(&t.value.target))
}

fn refs_display(ids: &[sysml_v2_parser::QualifiedReferenceId]) -> String {
    ids.iter().map(|i| qr(*i)).collect::<Vec<_>>().join(", ")
}

/// An import/expose target's text including its `::*`/`::**` suffix (`REQ-TRS-SYSMLV2-020`).
fn import_target_text(t: &sysml_v2_parser::ImportTarget) -> String {
    use sysml_v2_parser::ImportShape as S;
    let mut out = qr(t.reference);
    match &t.shape {
        S::Membership { recursive_suffix } => {
            if recursive_suffix.is_some() {
                out.push_str("::**");
            }
        }
        S::Namespace { recursive_suffix, .. } => {
            out.push_str("::*");
            if recursive_suffix.is_some() {
                out.push_str("::**");
            }
        }
        S::Filter { recursive_suffix, .. } => {
            if recursive_suffix.is_some() {
                out.push_str("::**");
            }
        }
    }
    out
}

/// A `connect`-clause endpoint's dotted display text, e.g. `a` or `a.p1` —
/// `REQ-TRS-SYSMLV2-010`. A single, unchained name parses as
/// `Expression::FeatureRef`; a genuine multi-segment dotted chain parses as
/// `Expression::FeatureChainRef` (confirmed against the parser's own AST and
/// its own doc comments: `path_expression`, used for `connect` endpoints
/// specifically, produces `FeatureChainRef` rather than folding into nested
/// `MemberAccess` the way the general value-expression grammar's postfix `.`
/// chaining does). `MemberAccess(base, member)` — confirmed empirically
/// (`REQ-TRS-SYSMLV2-024`) to be the shape `FlowUsage.from`/`.to` actually
/// use: unlike `connect`, a flow's endpoints are typed as a general
/// `Expression` (the value-expression grammar's postfix `.` chaining),
/// never the dedicated `path_expression` production, so a dotted flow
/// endpoint (`a.x`) parses as nested `MemberAccess`, not
/// `FeatureChainRef` — recursed here, since the base itself can be an
/// arbitrarily long chain. Other expression shapes aren't meaningful
/// endpoints and aren't mapped here, matching `feature_ref_string`'s
/// existing posture for `satisfy`/`verify` targets.
fn connection_end_display(expr: &sysml_v2_parser::Expression) -> Option<String> {
    match expr {
        sysml_v2_parser::Expression::FeatureRef(id) => Some(qr(*id)),
        sysml_v2_parser::Expression::FeatureChainRef(chain) => Some(qr_segments(*chain).join(".")),
        sysml_v2_parser::Expression::MemberAccess { base, member, .. } => {
            connection_end_display(&base.value).map(|b| format!("{b}.{}", qr(*member)))
        }
        _ => None,
    }
}

/// Best-effort rendering of a general `Expression` to display text
/// (`REQ-TRS-SYSMLV2-018`/`-019` guard/condition/assign-operand text).
/// Unlike `feature_ref_string`/`connection_end_display` (which only
/// recognize a reference shape and return `None` for anything else, since
/// their callers treat "not a reference" as "not mapped here"), this always
/// produces *some* non-empty text: a guard/condition must never silently
/// vanish, since the existing `W072` non-determinism check and
/// `docs/model-guide/state-machines.md`'s own contract depend only on the
/// field being present and non-empty, not on it being a faithful
/// re-rendering of the source. Recognized shapes render exactly (including
/// operators via the parser's own `BinaryOperator`/`UnaryOperator::as_str()`,
/// so `>=`/`and`/etc. spelling always matches the grammar exactly); the long
/// tail (`Classification`/`MetaCast`/`TypeCheck`/`Select`/`Collect`/
/// `CollectionOp`/`MetadataAccess`/`Conditional`/`Extent`) falls back to a
/// fixed, kind-naming placeholder — a Syscribe-owned, revisitable-later
/// limitation (see `ADR-SYS-SYSMLV2-001`'s addendum), explicitly distinct
/// from the fork/join/decide/merge upstream parser ceiling.
fn render_expression(e: &sysml_v2_parser::Expression) -> String {
    use sysml_v2_parser::Expression as E;
    match e {
        E::LiteralInteger(i) => i.to_string(),
        E::LiteralReal(r) => with_cur(|d| d.real_literal(*r).map(str::to_string)).flatten().unwrap_or_default(),
        E::LiteralString(s) => {
            format!("\"{}\"", with_cur(|d| d.decoded_string_literal(*s).map(|c| c.into_owned())).flatten().unwrap_or_default())
        }
        E::LiteralBoolean(b) => b.to_string(),
        E::FeatureRef(id) => qr(*id),
        E::MemberAccess { base, member, separator } => {
            let sep = match separator {
                sysml_v2_parser::ReferenceSeparator::Dot => ".",
                sysml_v2_parser::ReferenceSeparator::ColonColon => "::",
            };
            format!("{}{sep}{}", render_expression(&base.value), qr(*member))
        }
        E::FeatureChainRef(chain) => qr_segments(*chain).join("."),
        E::Index { base, operands, .. } => {
            format!("{}#({})", render_expression(&base.value), render_sequence(&operands.value))
        }
        // `12.5 [kg]` (0.54's `LiteralWithUnit`) is a `Bracket` over the literal in 0.55+.
        E::Bracket { base, operands, .. } => {
            format!("{} [{}]", render_expression(&base.value), render_sequence(&operands.value))
        }
        // A range (`1..3`) is written without spaces, the way 0.54 spelled it.
        E::BinaryOp { op: sysml_v2_parser::ast::BinaryOperator::Range, left, right } => {
            format!("{}..{}", render_expression(&left.value), render_expression(&right.value))
        }
        E::BinaryOp { op, left, right } => {
            format!("{} {} {}", render_expression(&left.value), op.as_str(), render_expression(&right.value))
        }
        E::UnaryOp { op, operand } => format!("{}{}", op.as_str(), render_expression(&operand.value)),
        E::Invocation { callee, args } => {
            let rendered: Vec<String> = args.iter().map(render_argument).collect();
            format!("{}({})", render_expression(&callee.value), rendered.join(", "))
        }
        // `(a, b)` tuples and `(a)` parenthesised expressions are both a parenthesised sequence.
        E::Sequence { operands, .. } => format!("({})", render_sequence(&operands.value)),
        E::Constructor { type_name, args } => {
            let rendered: Vec<String> = args.iter().map(render_argument).collect();
            format!("new {}({})", qr(*type_name), rendered.join(", "))
        }
        E::Extent { target } => format!("all {}", qr(*target)),
        E::Null => "null".to_string(),
        E::Classification { .. } => "<classification expression>".to_string(),
        E::MetaCast { .. } => "<meta-cast expression>".to_string(),
        E::TypeCheck { .. } => "<type-check expression>".to_string(),
        E::Select { .. } => "<select expression>".to_string(),
        E::Collect { .. } => "<collect expression>".to_string(),
        E::CollectionOp { .. } => "<collection-operator expression>".to_string(),
        E::MetadataAccess(_) => "<metadata-access expression>".to_string(),
        E::Conditional { .. } => "<conditional expression>".to_string(),
        E::BodyExpr(_) => "<body expression>".to_string(),
    }
}

/// Comma-joined rendering of a parenthesised/bracketed expression list.
fn render_sequence(list: &sysml_v2_parser::ast::SequenceExpressionList) -> String {
    list.elements.iter().map(|el| render_expression(&el.expression.value)).collect::<Vec<_>>().join(", ")
}

/// Render one call/constructor argument — `name = value` for a named
/// argument, bare `value` for a positional one.
fn render_argument(a: &sysml_v2_parser::Argument) -> String {
    match a.parameter {
        Some(name) => format!("{} = {}", qr(name), render_expression(&a.value.value)),
        None => render_expression(&a.value.value),
    }
}

/// Shorthand for a YAML mapping key.
fn ykey(s: &str) -> serde_yaml::Value {
    serde_yaml::Value::String(s.to_string())
}

// ── State machine mapping (REQ-TRS-SYSMLV2-018) ────────────────────────────

/// One recursion level's worth of state-machine content: `StateDef`'s and
/// `StateUsage`'s shared `StateDefBody`/`StateDefBodyElement` grammar
/// produces exactly these five things at any nesting depth.
struct StateBody {
    sub_states: Vec<serde_yaml::Value>,
    transitions: Vec<serde_yaml::Value>,
    entry_action: Option<serde_yaml::Value>,
    do_action: Option<serde_yaml::Value>,
    exit_action: Option<serde_yaml::Value>,
}

/// Walk one state's body-element slice, producing its `StateBody`.
///
/// Two passes, since `ThenStmt`/`FinalState` name a substate rather than
/// flagging it directly (`REQ-TRS-SYSMLV2-018`): first collect nested
/// `StateUsage` children (recursing into each's own body the same way —
/// composite states nest arbitrarily deep) and the `Then`/`FinalState`
/// siblings naming one of them; then apply `isInitial`/`isFinal` onto the
/// matching child by name. A name matching no collected child is dropped
/// silently, the module's established no-meaningful-mapping posture.
///
/// `require_explicit_source` distinguishes the two places `docs/model-guide/
/// state-machines.md`'s canonical transition schema allows a `Transition` to
/// live: `true` for the outermost `StateDef`/`StateUsage`'s own top-level
/// body (a `source`-less transition there means nothing — dropped); `false`
/// when recursing into a specific child `StateUsage`'s own body (that
/// child's own `name:` already supplies the implicit source, exactly
/// matching `validator.rs::transitions_from`'s `implicit_source` parameter).
fn build_state_body(
    elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::StateDefBodyElement>],
    require_explicit_source: bool,
) -> StateBody {
    use sysml_v2_parser::ast::StateDefBodyElement as E;

    let mut children: Vec<(String, serde_yaml::Mapping)> = Vec::new();
    let mut own_transitions = Vec::new();
    let mut entry_action = None;
    let mut do_action = None;
    let mut exit_action = None;
    let mut then_names: Vec<String> = Vec::new();
    let mut final_names: Vec<String> = Vec::new();

    for n in elements {
        match &n.value {
            E::StateUsage(su) => {
                let Some(su_name) = odn(su.value.name).filter(|n| !n.is_empty()) else {
                    continue; // anonymous nested state: no identity to key isInitial/isFinal against
                };
                let type_name = oqr(su.value.type_name).or_else(|| typing_first(su.value.typing.as_ref()));
                let type_name = type_name.as_deref();
                let body_elements = match &su.value.body {
                    sysml_v2_parser::ast::StateDefBody::Brace { elements, .. } => elements.as_slice(),
                    sysml_v2_parser::ast::StateDefBody::Semicolon { .. } => &[],
                };
                let entry = state_usage_yaml_entry(&su_name, type_name, body_elements);
                children.push((su_name, entry));
            }
            E::Entry(a) => entry_action = state_action_name(a.value.declared_name, a.value.action_reference).map(serde_yaml::Value::String),
            E::Do(a) => do_action = state_action_name(a.value.declared_name, a.value.action_reference).map(serde_yaml::Value::String),
            E::Exit(a) => exit_action = state_action_name(a.value.declared_name, a.value.action_reference).map(serde_yaml::Value::String),
            E::Then(t) => then_names.push(qr(t.value.state_reference)),
            E::FinalState(f) => final_names.push(dn(f.value.state_name)),
            E::Transition(t) => {
                if let Some(m) = render_transition(&t.value, require_explicit_source) {
                    own_transitions.push(serde_yaml::Value::Mapping(m));
                }
            }
            // InOutDecl, Ref, RequirementUsage, Annotation, MetadataKeywordUsage,
            // Other, Error, MetadataAnnotation (handled separately by
            // `state_def_syscribe_meta`) — outside REQ-TRS-SYSMLV2-018's fixed set.
            _ => {}
        }
    }

    for (name, mapping) in &mut children {
        if then_names.iter().any(|n| n == name) {
            mapping.insert(ykey("isInitial"), serde_yaml::Value::Bool(true));
        }
        if final_names.iter().any(|n| n == name) {
            mapping.insert(ykey("isFinal"), serde_yaml::Value::Bool(true));
        }
    }

    StateBody {
        sub_states: children.into_iter().map(|(_, m)| serde_yaml::Value::Mapping(m)).collect(),
        transitions: own_transitions,
        entry_action,
        do_action,
        exit_action,
    }
}

/// Name of an `entry`/`do`/`exit` action: the declared name (`entry action n;`) or, for the
/// reference form (`entry n;`), the referenced action.
fn state_action_name(
    declared: Option<sysml_v2_parser::DeclarationName>,
    reference: Option<sysml_v2_parser::QualifiedReferenceId>,
) -> Option<String> {
    odn(declared).or_else(|| oqr(reference))
}

/// Decoded text of a declaration name where the AST holds a handle (`""` for an absent name), so
/// converters can keep treating "no name" and "empty name" alike.
pub(crate) trait NameStr {
    fn s(&self) -> String;
}

impl NameStr for sysml_v2_parser::DeclarationName {
    fn s(&self) -> String {
        dn(*self)
    }
}

impl NameStr for Option<sysml_v2_parser::DeclarationName> {
    fn s(&self) -> String {
        self.map(dn).unwrap_or_default()
    }
}

/// Whether an optional declared name equals `s`.
fn name_is(n: Option<sysml_v2_parser::DeclarationName>, s: &str) -> bool {
    n.is_some_and(|n| dn(n) == s)
}

/// First target of an optional typing relationship, as text.
fn typing_first(t: Option<&sysml_v2_parser::Node<sysml_v2_parser::ast::TypingRelationship>>) -> Option<String> {
    t.and_then(|t| t.value.first_target()).map(qr)
}

/// Build one nested `subStates:` entry (`REQ-TRS-SYSMLV2-018`) — does *not*
/// set `isInitial`/`isFinal`; the caller ([`build_state_body`]) applies those
/// from the enclosing `Then`/`FinalState` siblings, since a `StateUsage`
/// carries no such information about itself.
fn state_usage_yaml_entry(
    name: &str,
    type_name: Option<&str>,
    elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::StateDefBodyElement>],
) -> serde_yaml::Mapping {
    let body = build_state_body(elements, false);
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.to_string()));
    if let Some(tb) = type_name {
        m.insert(ykey("typedBy"), serde_yaml::Value::String(tb.to_string()));
    }
    if let Some(v) = body.entry_action {
        m.insert(ykey("entryAction"), v);
    }
    if let Some(v) = body.do_action {
        m.insert(ykey("doAction"), v);
    }
    if let Some(v) = body.exit_action {
        m.insert(ykey("exitAction"), v);
    }
    if !body.sub_states.is_empty() {
        m.insert(ykey("subStates"), serde_yaml::Value::Sequence(body.sub_states));
    }
    if !body.transitions.is_empty() {
        m.insert(ykey("transitions"), serde_yaml::Value::Sequence(body.transitions));
    }
    m
}

/// Render one `Transition` into a `transitions:` entry — `None` when
/// `require_explicit_source` is set and the AST carries no `source` (a
/// top-level, source-less transition means nothing per the canonical
/// schema). Field mapping matches `docs/model-guide/state-machines.md`'s
/// canonical vocabulary exactly (`source`/`target`/`accept`/`guard`/`effect`)
/// — never the deprecated `from`/`to`/`trigger` aliases, so `W075` never
/// fires on SysMLv2-synthesized output.
fn render_transition(t: &sysml_v2_parser::ast::Transition, require_explicit_source: bool) -> Option<serde_yaml::Mapping> {
    let source_display = t
        .source
        .as_ref()
        .map(|s| connection_end_display(&s.value).unwrap_or_else(|| render_expression(&s.value)));
    if require_explicit_source && source_display.is_none() {
        return None;
    }

    let mut m = serde_yaml::Mapping::new();
    if let Some(s) = source_display {
        m.insert(ykey("source"), serde_yaml::Value::String(s));
    }
    let target = connection_end_display(&t.target.value).unwrap_or_else(|| render_expression(&t.target.value));
    m.insert(ykey("target"), serde_yaml::Value::String(target));
    if let Some(accept) = &t.accept {
        m.insert(ykey("accept"), render_transition_accept(accept));
    }
    if let Some(guard) = &t.guard {
        let g = render_expression(&guard.value);
        if !g.is_empty() {
            m.insert(ykey("guard"), serde_yaml::Value::String(g));
        }
    }
    if let Some(effect) = &t.effect {
        if let Some(e) = render_transition_effect(effect) {
            m.insert(ykey("effect"), e);
        }
    }
    Some(m)
}

/// `accept:` value — plain string for the common shorthand-without-`via`
/// case (matching `docs/model-guide/state-machines.md`'s own worked
/// examples verbatim), `{payload, via}` map when a `via <port>` clause is
/// present, or `{payload: "<at|when|after> <expr>"}` for a time trigger.
fn render_transition_accept(a: &sysml_v2_parser::ast::TransitionAccept) -> serde_yaml::Value {
    use sysml_v2_parser::ast::TransitionAccept as A;
    match a {
        A::Shorthand(expr, via) => {
            let text = connection_end_display(&expr.value).unwrap_or_else(|| render_expression(&expr.value));
            payload_with_via(text, via.as_ref())
        }
        A::Payload(payload, via) => {
            let text = oqr(payload.type_name).unwrap_or_else(|| dn(payload.name));
            payload_with_via(text, via.as_ref())
        }
        A::TimeTrigger(kind, expr) => {
            let kind_s = match kind {
                sysml_v2_parser::ast::TriggerKind::At => "at",
                sysml_v2_parser::ast::TriggerKind::When => "when",
                sysml_v2_parser::ast::TriggerKind::After => "after",
            };
            let text = format!("{kind_s} {}", render_expression(&expr.value));
            let mut m = serde_yaml::Mapping::new();
            m.insert(ykey("payload"), serde_yaml::Value::String(text));
            serde_yaml::Value::Mapping(m)
        }
    }
}

fn payload_with_via(payload_text: String, via: Option<&sysml_v2_parser::Node<sysml_v2_parser::Expression>>) -> serde_yaml::Value {
    match via {
        None => serde_yaml::Value::String(payload_text),
        Some(via_expr) => {
            let mut m = serde_yaml::Mapping::new();
            m.insert(ykey("payload"), serde_yaml::Value::String(payload_text));
            let v = connection_end_display(&via_expr.value).unwrap_or_else(|| render_expression(&via_expr.value));
            m.insert(ykey("via"), serde_yaml::Value::String(v));
            serde_yaml::Value::Mapping(m)
        }
    }
}

/// `effect:` value, matching `validator.rs::collect_state_refs`'s exact
/// `W079` contract: a bare `String` is always resolved-checked; a `Mapping`
/// is checked only via its `typedBy` key, never `name`. So a `Perform`/
/// `Accept`/`Send` with a real `type_name` becomes `{name, typedBy}`
/// (W079-checked, matching the documented worked example verbatim); with no
/// `type_name` it becomes `{name}` only (deliberately *not* checked — a
/// local label isn't necessarily a global qname). `Assign` is display-only,
/// never checked. `Expression` becomes a plain, checked `String` when it's a
/// genuine reference, else a `{name}` display fallback — avoiding spurious
/// `W079` false positives either way. `None` when there is truly nothing to
/// display (omits the `effect:` key entirely).
fn render_transition_effect(e: &sysml_v2_parser::ast::TransitionEffect) -> Option<serde_yaml::Value> {
    use sysml_v2_parser::ast::TransitionEffect as Eff;
    match e {
        Eff::Perform { name, type_name, .. } => {
            effect_name_typed(odn(*name).as_deref(), oqr(*type_name).as_deref())
        }
        Eff::Accept { payload, type_name, .. } => {
            let name = connection_end_display(&payload.value).unwrap_or_else(|| render_expression(&payload.value));
            effect_name_typed(Some(&name), oqr(*type_name).as_deref())
        }
        Eff::Send { payload, type_name, .. } => {
            let name = connection_end_display(&payload.value).unwrap_or_else(|| render_expression(&payload.value));
            effect_name_typed(Some(&name), oqr(*type_name).as_deref())
        }
        Eff::Assign { lhs, rhs, .. } => {
            let text = format!("{} := {}", render_expression(&lhs.value), render_expression(&rhs.value));
            let mut m = serde_yaml::Mapping::new();
            m.insert(ykey("name"), serde_yaml::Value::String(text));
            Some(serde_yaml::Value::Mapping(m))
        }
        Eff::Expression(expr) => match connection_end_display(&expr.value) {
            Some(s) => Some(serde_yaml::Value::String(s)),
            None => {
                let mut m = serde_yaml::Mapping::new();
                m.insert(ykey("name"), serde_yaml::Value::String(render_expression(&expr.value)));
                Some(serde_yaml::Value::Mapping(m))
            }
        },
    }
}

fn effect_name_typed(name: Option<&str>, type_name: Option<&str>) -> Option<serde_yaml::Value> {
    let name = name?;
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.to_string()));
    if let Some(tb) = type_name {
        m.insert(ykey("typedBy"), serde_yaml::Value::String(tb.to_string()));
    }
    Some(serde_yaml::Value::Mapping(m))
}

fn convert_state_def(s: &sysml_v2_parser::ast::StateDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&s.identification) else {
        return; // anonymous state def: no identity to qname against
    };
    let state_qname = format!("{qname}::{name}");
    let elements = match &s.body {
        sysml_v2_parser::ast::StateDefBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::ast::StateDefBody::Semicolon { .. } => &[],
    };
    let body = build_state_body(elements, true);
    let spec = Spec {
        supertype: s.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        entry_action: body.entry_action,
        do_action: body.do_action,
        exit_action: body.exit_action,
        ..Default::default()
    }
    .with_syscribe_meta(state_def_syscribe_meta(elements))
    .with_doc(state_def_doc(elements))
    .with_metadata(body_metadata(elements))
    .with_state_machine(body.sub_states, body.transitions);
    push_synth(out, &state_qname, file_path, ElementType::StateDef, &name, spec);
}

fn convert_state_usage(s: &sysml_v2_parser::ast::StateUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(s_name) = odn(s.name).filter(|n| !n.is_empty()) else {
        return; // anonymous usage: no identity to qname against
    };
    let state_qname = format!("{qname}::{s_name}");
    let elements = match &s.body {
        sysml_v2_parser::ast::StateDefBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::ast::StateDefBody::Semicolon { .. } => &[],
    };
    let body = build_state_body(elements, true);
    let spec = Spec {
        typed_by: oqr(s.type_name).or_else(|| typing_first(s.typing.as_ref())),
        entry_action: body.entry_action,
        do_action: body.do_action,
        exit_action: body.exit_action,
        ..Default::default()
    }
    .with_syscribe_meta(state_def_syscribe_meta(elements))
    .with_doc(state_def_doc(elements))
    .with_metadata(body_metadata(elements))
    .with_state_machine(body.sub_states, body.transitions);
    push_synth(out, &state_qname, file_path, ElementType::State, &s_name, spec);
}

/// The `part` usage struct (name + own body) a `sysml_v2_parser::PartUsage`
/// carries, regardless of which body-element enum wrapped it —
/// `PartDefBodyElement::PartUsage`/`PartUsageBodyElement::PartUsage` both
/// wrap `Box<Node<PartUsage>>`, the exact same type, so once found the rest
/// of the lookahead logic doesn't care which enclosing body it came from.
type PartUsageSibling<'a> = &'a sysml_v2_parser::PartUsage;

/// Find a `part` usage named `head` directly in `elements` (a `part def`
/// body's already-sliced members) — `REQ-TRS-SYSMLV2-013`'s local
/// lookahead, step 1: is the connect endpoint's head itself a `part` usage
/// declared in the same enclosing body as the `connection` usage?
fn find_part_usage_in_part_def_body<'a>(
    elements: &'a [sysml_v2_parser::Node<sysml_v2_parser::PartDefBodyElement>],
    head: &str,
) -> Option<PartUsageSibling<'a>> {
    elements.iter().find_map(|n| match &n.value {
        sysml_v2_parser::PartDefBodyElement::PartUsage(pu) if name_is(pu.value.name, head) => Some(&pu.value),
        _ => None,
    })
}

/// Find a `part` usage named `head` directly in `elements` (a `part` usage
/// body's already-sliced members). See [`find_part_usage_in_part_def_body`].
fn find_part_usage_in_part_usage_body<'a>(
    elements: &'a [sysml_v2_parser::Node<sysml_v2_parser::PartUsageBodyElement>],
    head: &str,
) -> Option<PartUsageSibling<'a>> {
    elements.iter().find_map(|n| match &n.value {
        sysml_v2_parser::PartUsageBodyElement::PartUsage(pu) if name_is(pu.value.name, head) => Some(&pu.value),
        _ => None,
    })
}

/// Whether `pu`'s own body declares a direct child named `tail` — a
/// `port`/`attribute`/`interface` usage, or a nested `part` usage —
/// `REQ-TRS-SYSMLV2-013`'s local lookahead, step 2. No `item` usage arm:
/// `PartUsageBodyElement` (a `part` *usage*'s own body-element enum, unlike
/// `part def`'s) carries no `ItemUsage` variant in this grammar at all, so a
/// `part` usage cannot declare a nested `item` usage to begin with —
/// confirmed against the parser's own enum definition, not an oversight.
fn part_usage_has_named_child(pu: PartUsageSibling<'_>, tail: &str) -> bool {
    let sysml_v2_parser::PartUsageBody::Brace { elements, .. } = &pu.body else {
        return false;
    };
    elements.iter().any(|n| match &n.value {
        sysml_v2_parser::PartUsageBodyElement::PortUsage(p) => name_is(p.value.name, tail),
        sysml_v2_parser::PartUsageBodyElement::AttributeUsage(a) => name_is(a.value.name, tail),
        sysml_v2_parser::PartUsageBodyElement::PartUsage(p) => name_is(p.value.name, tail),
        sysml_v2_parser::PartUsageBodyElement::InterfaceUsage(iface) => matches!(
            &iface.value,
            sysml_v2_parser::InterfaceUsage::Declaration { name: Some(n), .. } if dn(*n) == tail
        ),
        _ => false,
    })
}

/// Rewrite a `connect`-clause endpoint's dotted chain text into the
/// qualified qname `REQ-TRS-SYSMLV2-010`'s `connections:` lift actually
/// needs — see the `ADR-SYS-SYSMLV2-001` addendum for the full
/// investigation (two rounds of it: a literal, unqualified `"a.p1"` never
/// resolves at all; a full `"a.p1"` → `"Holder::a::p1"` conversion mostly
/// doesn't either, since `p1` is overwhelmingly a port *inherited* from
/// `a`'s type rather than redeclared on the usage itself, and this module
/// does no inheritance resolution — so `Holder::a::p1` isn't a real
/// synthesized child in the common case).
///
/// Default (and fallback) behavior: only the **head** segment (before the
/// first `.`) is kept — `a.p1` under the owning part `Holder` becomes
/// `Holder::a`, matching this module's connection graph's own existing
/// precedent for `features:`-declared endpoints (`graph.rs::resolve_endpoint`
/// — "NOTE (deferred, issue #26 MVP): edges carry `kind` only", resolving
/// only the head, discarding the rest of the chain).
///
/// `REQ-TRS-SYSMLV2-013` widens this one step: for a genuinely two-segment
/// chain (`head.tail`, no further `.`), `find_sibling(head)` — a pure,
/// local AST lookahead into the *same enclosing body*, no resolver, no
/// global element list — is tried; if it finds a `part` usage sibling whose
/// own body declares a direct child named by `tail`
/// ([`part_usage_has_named_child`]), the *full* chain qualifies instead:
/// `Holder::a::p1`. Any other outcome (three-plus segments, no matching
/// sibling, sibling has no matching child) falls back to head-only —
/// a strict widening, never a new failure mode.
///
/// Returns `(qualified qname, truncation message)` — `REQ-TRS-SYSMLV2-015`.
/// The message is `Some` exactly when a genuinely two-segment chain existed
/// but wasn't resolved to a redeclared nested feature (the common case: the
/// tail is a feature *inherited* from the head's type, never redeclared on
/// the usage, which this module still can't verify without a full-model
/// resolver — see the `ADR-SYS-SYSMLV2-001` addendum). `None` for a bare,
/// undotted endpoint and for a three-plus-segment chain — the latter is
/// `REQ-TRS-SYSMLV2-013`'s own, separately-documented, deliberately
/// unwarned fallback, not this requirement's concern.
fn qualify_connection_end<'a>(
    owning_qname: &str,
    chain: &str,
    find_sibling: &impl Fn(&str) -> Option<PartUsageSibling<'a>>,
) -> (String, Option<String>) {
    let mut segments = chain.splitn(2, '.');
    let head = segments.next().unwrap_or(chain);
    if let Some(tail) = segments.next() {
        if !tail.contains('.') {
            if let Some(pu) = find_sibling(head) {
                if part_usage_has_named_child(pu, tail) {
                    return (format!("{owning_qname}::{head}::{tail}"), None);
                }
            }
            let qname = format!("{owning_qname}::{head}");
            let message = format!(
                "connect endpoint '{chain}' has no locally-redeclared '{tail}' feature on \
                 '{head}' -- truncated to the head-only edge '{qname}' (a feature inherited from \
                 '{head}'s type, rather than redeclared on the usage, cannot be verified without \
                 a full-model resolver; see REQ-TRS-SYSMLV2-013/-015)"
            );
            return (qname, Some(message));
        }
    }
    (format!("{owning_qname}::{head}"), None)
}

/// One `connections:`-shaped YAML entry for a single named `connection name
/// : Type connect a to b (, c)*;` usage — `REQ-TRS-SYSMLV2-010`. `None` for
/// a connection usage with no `connect` clause at all (`connect_from` is
/// `None`) or whose `connect_from`/`connect_to` expression isn't a mapped
/// shape (see [`connection_end_display`]) — either way, no regression: the
/// same "nothing to contribute" outcome a plain `connection c : SomeConnDef;`
/// declaration already has. Binary form (no `connect_extra_ends`) reuses
/// `crate::connections::add_connection` so the emitted shape can never drift
/// from the hand-authored binary convention; the n-ary form
/// (`connect (a, b, c)`) is built directly in the `ends:` shape
/// `crate::connections::parse_entry` already reads back, since
/// `add_connection` only ever writes the binary form.
fn connection_usage_entry<'a>(
    owning_qname: &str,
    c: &sysml_v2_parser::ast::ConnectionUsageMember,
    find_sibling: &impl Fn(&str) -> Option<PartUsageSibling<'a>>,
    truncations: &mut Vec<String>,
) -> Option<serde_yaml::Value> {
    let from = connection_end_display(&c.connect_from.as_ref()?.value.expression.value)?;
    let to = connection_end_display(&c.connect_to.as_ref()?.value.expression.value)?;
    let extras: Vec<String> = c
        .connect_extra_ends
        .iter()
        .filter_map(|n| connection_end_display(&n.value.expression.value))
        .collect();
    let typed_by = typing_first(c.typing.as_ref()).and_then(nonempty);
    build_connection_value(owning_qname, from, to, extras, typed_by, find_sibling, truncations)
}

/// The `connections:` entry for a resolved `from`/`to`(/extras) endpoint list -- shared by the
/// named `connection` usage, the anonymous `connect` member and the `interface ... connect`
/// usage (GH #203).
fn build_connection_value<'a>(
    owning_qname: &str,
    from: String,
    to: String,
    extras: Vec<String>,
    typed_by: Option<String>,
    find_sibling: &impl Fn(&str) -> Option<PartUsageSibling<'a>>,
    truncations: &mut Vec<String>,
) -> Option<serde_yaml::Value> {
    let (from_q, from_trunc) = qualify_connection_end(owning_qname, &from, find_sibling);
    let (to_q, to_trunc) = qualify_connection_end(owning_qname, &to, find_sibling);
    truncations.extend(from_trunc);
    truncations.extend(to_trunc);

    if extras.is_empty() {
        let mut tmp = Vec::new();
        crate::connections::add_connection(&mut tmp, &from_q, &to_q, typed_by.as_deref());
        tmp.pop()
    } else {
        let extras_q: Vec<String> = extras
            .iter()
            .map(|e| {
                let (q, trunc) = qualify_connection_end(owning_qname, e, find_sibling);
                truncations.extend(trunc);
                q
            })
            .collect();
        let mut ends: Vec<serde_yaml::Value> = [from_q, to_q]
            .into_iter()
            .chain(extras_q)
            .map(|chain| {
                let mut em = serde_yaml::Mapping::new();
                em.insert(serde_yaml::Value::from("binds"), serde_yaml::Value::from(chain));
                serde_yaml::Value::Mapping(em)
            })
            .collect();
        let mut m = serde_yaml::Mapping::new();
        if let Some(tb) = &typed_by {
            m.insert(serde_yaml::Value::from("typedBy"), serde_yaml::Value::from(tb.as_str()));
        }
        m.insert(serde_yaml::Value::from("ends"), serde_yaml::Value::Sequence(std::mem::take(&mut ends)));
        Some(serde_yaml::Value::Mapping(m))
    }
}

/// `connections:` entries over a `part def` body's already-sliced members —
/// scans every `PartDefBodyElement::Connection` (the named `connection` form
/// — a distinct AST variant from the anonymous `PartDefBodyElement::Connect`,
/// which stays unmapped: no identity to synthesize an owning-part-relative
/// entry against, `REQ-TRS-SYSMLV2-010`'s Scope). Second element of the
/// return tuple is `REQ-TRS-SYSMLV2-015`'s truncation messages, one per
/// connect endpoint whose genuinely two-segment chain fell back to
/// head-only.
fn part_def_connection_entries(
    owning_qname: &str,
    elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartDefBodyElement>],
) -> (Vec<serde_yaml::Value>, Vec<String>) {
    let find_sibling = |head: &str| find_part_usage_in_part_def_body(elements, head);
    let mut truncations = Vec::new();
    let entries = elements
        .iter()
        .filter_map(|n| match &n.value {
            sysml_v2_parser::PartDefBodyElement::Connection(node) => {
                connection_usage_entry(owning_qname, &node.value, &find_sibling, &mut truncations)
            }
            _ => None,
        })
        .collect();
    (entries, truncations)
}

/// `connections:` entries over a `part` usage body's already-sliced members.
/// See [`part_def_connection_entries`].
fn part_usage_connection_entries(
    owning_qname: &str,
    elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartUsageBodyElement>],
) -> (Vec<serde_yaml::Value>, Vec<String>) {
    let find_sibling = |head: &str| find_part_usage_in_part_usage_body(elements, head);
    let mut truncations = Vec::new();
    let entries = elements
        .iter()
        .filter_map(|n| match &n.value {
            sysml_v2_parser::PartUsageBodyElement::Connection(node) => {
                connection_usage_entry(owning_qname, &node.value, &find_sibling, &mut truncations)
            }
            _ => None,
        })
        .collect();
    (entries, truncations)
}

/// The item type a `flow`/`message`/`succession flow` usage carries —
/// `REQ-TRS-SYSMLV2-024`. `FlowUsage.payload.type_name` (the explicit `of
/// <name>? : <Type>` clause) and `FlowUsage.type_name` (the bare `:
/// <Type>` shorthand with no `of` clause) are both item-shaped per the
/// vendored parser's own real fixtures (`flow t of Payload from a to b;`
/// and `flow t : Fuel from a to b;` are parallel, interchangeable forms
/// for identifying *what flows* — matching Syscribe's own spec, which
/// frames `itemType` as "shorthand: qualified name of the ItemDef carried
/// by this flow", §8.6.1) — neither is a `typedBy`-style supertype
/// reference. The `of` clause wins when both are somehow present (real
/// grammar likely never has both at once).
fn flow_item_type(f: &sysml_v2_parser::FlowUsage) -> Option<String> {
    let (_, typing, payload, _) = flow_parts(f);
    payload.and_then(oqr).or(typing)
}

/// The pieces of a 0.57 `FlowUsage` declaration in the shape 0.54 exposed them:
/// `(name, ':' typing, 'of' payload type, (from, to))`. A repeated `of` clause is read as the
/// first one, like the single payload 0.54 kept.
fn flow_parts(
    f: &sysml_v2_parser::FlowUsage,
) -> (
    Option<String>,
    Option<String>,
    Option<Option<sysml_v2_parser::QualifiedReferenceId>>,
    Option<(String, String)>,
) {
    use sysml_v2_parser::ast::FlowDeclaration as D;
    let ends = |e: &sysml_v2_parser::ast::FlowEndpoints| {
        (qr_segments(e.from.value.target).join("."), qr_segments(e.to.value.target).join("."))
    };
    match &f.declaration {
        D::Declared { declaration, payloads, endpoints, .. } => {
            let decl = &declaration.value;
            (
                odn(decl.identification.name),
                typing_first(decl.typing.as_ref()),
                payloads.first().map(|p| p.value.feature.value.type_name),
                (**endpoints).as_ref().map(ends),
            )
        }
        D::EndpointOnly { endpoints } => (None, None, None, Some(ends(endpoints))),
    }
}

/// `flowConnections:`'s `kind:` vocabulary — `REQ-TRS-SYSMLV2-024`, matching
/// `spec/markdown-sysml-format.md` §8.6.2's kind-semantics table exactly.
fn flow_kind_str(kind: sysml_v2_parser::FlowUsageKind) -> &'static str {
    match kind {
        sysml_v2_parser::FlowUsageKind::Flow => "streaming",
        sysml_v2_parser::FlowUsageKind::Message => "message",
        sysml_v2_parser::FlowUsageKind::SuccessionFlow => "succession",
    }
}

/// One `flowConnections:`-shaped YAML entry for a single `flow`/`message`/
/// `succession flow` usage — `REQ-TRS-SYSMLV2-024`, the exact `from`/`to`/
/// `kind`/`item`/`name` sub-schema `spec/markdown-sysml-format.md` §8.6.2
/// documents. Built regardless of whether `f.name` is set (mirrors
/// `connection_usage_entry`'s identical "regardless of name" posture) — an
/// anonymous `flow a.x to b.y;` statement contributes an entry here even
/// though it never becomes its own `RawElement` (see `convert_flow_usage`).
/// `None` when `from`/`to` isn't present or isn't a mapped `Expression`
/// shape (see [`connection_end_display`]) — the same "nothing to
/// contribute" outcome a bare `flow : SomeFlowDef;` (no endpoints at all)
/// already has.
fn flow_usage_entry<'a>(
    owning_qname: &str,
    f: &sysml_v2_parser::FlowUsage,
    find_sibling: &impl Fn(&str) -> Option<PartUsageSibling<'a>>,
    truncations: &mut Vec<String>,
) -> Option<serde_yaml::Value> {
    let (f_name, _, _, ends) = flow_parts(f);
    let (from, to) = ends?;
    let (from_q, from_trunc) = qualify_connection_end(owning_qname, &from, find_sibling);
    let (to_q, to_trunc) = qualify_connection_end(owning_qname, &to, find_sibling);
    truncations.extend(from_trunc);
    truncations.extend(to_trunc);

    let mut m = serde_yaml::Mapping::new();
    if let Some(n) = f_name.filter(|n| !n.is_empty()) {
        m.insert(serde_yaml::Value::from("name"), serde_yaml::Value::from(n));
    }
    m.insert(serde_yaml::Value::from("from"), serde_yaml::Value::from(from_q));
    m.insert(serde_yaml::Value::from("to"), serde_yaml::Value::from(to_q));
    m.insert(serde_yaml::Value::from("kind"), serde_yaml::Value::from(flow_kind_str(f.kind)));
    if let Some(item) = flow_item_type(f) {
        m.insert(serde_yaml::Value::from("item"), serde_yaml::Value::from(item));
    }
    Some(serde_yaml::Value::Mapping(m))
}

/// `flowConnections:` entries over a `part def` body's already-sliced
/// members — scans every `PartDefBodyElement::FlowUsage`, named or
/// anonymous alike (mirrors [`part_def_connection_entries`] exactly, one
/// field per relationship kind, `REQ-TRS-SYSMLV2-024`). Second element of
/// the return tuple is the same `REQ-TRS-SYSMLV2-015`-style truncation
/// messages, reusing `qualify_connection_end` unchanged.
fn part_def_flow_entries(
    owning_qname: &str,
    elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartDefBodyElement>],
) -> (Vec<serde_yaml::Value>, Vec<String>) {
    let find_sibling = |head: &str| find_part_usage_in_part_def_body(elements, head);
    let mut truncations = Vec::new();
    let entries = elements
        .iter()
        .filter_map(|n| match &n.value {
            sysml_v2_parser::PartDefBodyElement::FlowUsage(node) => {
                flow_usage_entry(owning_qname, &node.value, &find_sibling, &mut truncations)
            }
            _ => None,
        })
        .collect();
    (entries, truncations)
}

/// `flowConnections:` entries over a `part` usage body's already-sliced
/// members. See [`part_def_flow_entries`].
fn part_usage_flow_entries(
    owning_qname: &str,
    elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartUsageBodyElement>],
) -> (Vec<serde_yaml::Value>, Vec<String>) {
    let find_sibling = |head: &str| find_part_usage_in_part_usage_body(elements, head);
    let mut truncations = Vec::new();
    let entries = elements
        .iter()
        .filter_map(|n| match &n.value {
            sysml_v2_parser::PartUsageBodyElement::FlowUsage(node) => {
                flow_usage_entry(owning_qname, &node.value, &find_sibling, &mut truncations)
            }
            _ => None,
        })
        .collect();
    (entries, truncations)
}

/// `@SyscribeFeature` search over a `part def` body's already-sliced members.
///
/// Variation is **not** Part-exclusive in this grammar — that was this
/// function's original assumption and it was wrong: `RequirementUsage` also
/// carries an independent `is_variation: bool` (see
/// [`requirement_body_syscribe_feature_id`], which mirrors this function for
/// the requirement-body case; a review caught the gap where it was missing).
/// What *is* still true, checked per body-element enum rather than assumed:
/// `PartDefBodyElement`/`PartUsageBodyElement`/`RequirementDefBodyElement`
/// each genuinely carry a `MetadataAnnotation` variant, while
/// `AttributeBodyElement`/`PortBodyElement` do not (only the unrelated
/// `#keyword`-style `MetadataKeywordUsage`) — so `@SyscribeFeature` on a
/// `variant attribute`/`variant port` typed-usage form has nowhere to attach
/// per this grammar version, not because those forms can't vary, but because
/// their body shape doesn't carry this AST node at all.
fn part_def_syscribe_feature_id(
    elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartDefBodyElement>],
) -> Option<String> {
    elements.iter().find_map(|n| match &n.value {
        sysml_v2_parser::PartDefBodyElement::Annotating(a) => annotating_meta(a).and_then(syscribe_feature_id),
        _ => None,
    })
}

/// `@SyscribeFeature` search over a `part` usage body's already-sliced
/// members. See [`part_def_syscribe_feature_id`].
fn part_usage_syscribe_feature_id(
    elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartUsageBodyElement>],
) -> Option<String> {
    elements.iter().find_map(|n| match &n.value {
        sysml_v2_parser::PartUsageBodyElement::Annotating(a) => annotating_meta(a).and_then(syscribe_feature_id),
        _ => None,
    })
}

/// `REQ-TRS-SYSMLV2-008` `@Syscribe*` fixed-field search over a `part def`
/// body's already-sliced members — scans every `MetadataAnnotation` rather
/// than stopping at the first match, since a real `part def` may carry
/// several of these (plus `@SyscribeFeature`) side by side.
fn part_def_syscribe_meta(
    elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartDefBodyElement>],
) -> SyscribeMeta {
    let mut meta = SyscribeMeta::default();
    for n in elements {
        if let sysml_v2_parser::PartDefBodyElement::Annotating(a) = &n.value {
            if let Some(m) = annotating_meta(a) {
                fold_syscribe_meta_annotation(m, &mut meta);
            }
        }
    }
    meta
}

/// `REQ-TRS-SYSMLV2-008` `@Syscribe*` fixed-field search over a `part` usage
/// body's already-sliced members. See [`part_def_syscribe_meta`].
fn part_usage_syscribe_meta(
    elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartUsageBodyElement>],
) -> SyscribeMeta {
    let mut meta = SyscribeMeta::default();
    for n in elements {
        if let sysml_v2_parser::PartUsageBodyElement::Annotating(a) = &n.value {
            if let Some(m) = annotating_meta(a) {
                fold_syscribe_meta_annotation(m, &mut meta);
            }
        }
    }
    meta
}

// ── Action mapping (REQ-TRS-SYSMLV2-019) ───────────────────────────────────

/// Accumulated result of walking one action body.
struct ActionBody {
    sub_actions: Vec<serde_yaml::Value>,
    control_nodes: Vec<serde_yaml::Value>,
    succession_connections: Vec<serde_yaml::Value>,
}

/// Mutable accumulator while walking one action body. `last_named` tracks
/// the most recently converted node's own name, for `ThenAction`'s implicit
/// "after" side; `counters` synthesizes deterministic, stable names
/// (`if_1`, `while_1`, ...) for constructs the grammar itself gives no name
/// to (`IfStmt`/`WhileStmt`/`LoopStmt`/`ForLoop` carry no name field at all)
/// — a Syscribe-owned naming convention, not a parser fact.
#[derive(Default)]
struct ActionBodyBuilder {
    sub_actions: Vec<serde_yaml::Value>,
    control_nodes: Vec<serde_yaml::Value>,
    succession_connections: Vec<serde_yaml::Value>,
    last_named: Option<String>,
    counters: std::collections::HashMap<&'static str, u32>,
    /// `REQ-TRS-SYSMLV2-060`: a name declared by an enclosing single-statement `action <name> {…}`
    /// usage; consumed by the next `synth_name` call instead of a synthesized one.
    forced_name: Option<String>,
}

impl ActionBodyBuilder {
    fn synth_name(&mut self, kind: &'static str) -> String {
        if let Some(name) = self.forced_name.take() {
            return name;
        }
        let n = self.counters.entry(kind).or_insert(0);
        *n += 1;
        format!("{kind}_{n}")
    }

    fn push_sub_action(&mut self, name: String, entry: serde_yaml::Mapping) {
        self.last_named = Some(name);
        self.sub_actions.push(serde_yaml::Value::Mapping(entry));
    }

    /// `ForkNode`/`JoinNode`/`DecisionNode`/`MergeNode` — flat, `{name,
    /// kind}` only, no recoverable internal content: the pinned parser
    /// itself discards `fork`/`join`/`decide`/`merge` block bodies
    /// (`FirstMergeBody::Brace` carries no data), so there is nothing to
    /// recurse into even in principle. A non-negotiable upstream ceiling,
    /// not a Syscribe scope choice (see `ADR-SYS-SYSMLV2-001`'s addendum).
    fn push_control_node(&mut self, name: String, kind: &str, parameters: Vec<serde_yaml::Value>) {
        let mut m = serde_yaml::Mapping::new();
        m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
        m.insert(ykey("kind"), serde_yaml::Value::String(kind.to_string()));
        // `REQ-TRS-SYSMLV2-090`: the node's `in`/`out` pin declarations.
        if !parameters.is_empty() {
            m.insert(ykey("parameters"), serde_yaml::Value::Sequence(parameters));
        }
        self.last_named = Some(name);
        self.control_nodes.push(serde_yaml::Value::Mapping(m));
    }

    fn push_succession(&mut self, after: String, before: String) {
        self.push_guarded_succession(after, before, None, SuccessionExtras::default());
    }

    /// A succession edge, with the native `guard:` text when the statement carried one
    /// (`REQ-TRS-SYSMLV2-074`) and its own name/multiplicities (`REQ-TRS-SYSMLV2-081`).
    fn push_guarded_succession(&mut self, after: String, before: String, guard: Option<String>, extras: SuccessionExtras) {
        self.succession_connections.push(succession_entry(after, before, guard, extras));
    }
}

/// One `successionConnections:` entry (`REQ-TRS-SYSMLV2-019`/`-074`/`-081`/`-091`/`-092`).
fn succession_entry(after: String, before: String, guard: Option<String>, extras: SuccessionExtras) -> serde_yaml::Value {
    let mut m = serde_yaml::Mapping::new();
    if let Some(n) = extras.name.filter(|n| !n.is_empty()) {
        m.insert(ykey("name"), serde_yaml::Value::String(n));
    }
    // `REQ-TRS-SYSMLV2-091`: the succession's own type.
    if let Some(t) = extras.typed_by.filter(|t| !t.is_empty()) {
        m.insert(ykey("typedBy"), serde_yaml::Value::String(t));
    }
    m.insert(ykey("after"), serde_yaml::Value::String(after));
    m.insert(ykey("before"), serde_yaml::Value::String(before));
    if let Some(g) = guard.filter(|g| !g.is_empty()) {
        m.insert(ykey("guard"), serde_yaml::Value::String(g));
    }
    for (k, v) in [("multiplicity", extras.multiplicity), ("afterMultiplicity", extras.after_multiplicity), ("beforeMultiplicity", extras.before_multiplicity)] {
        if let Some(v) = v {
            m.insert(ykey(k), serde_yaml::Value::String(v));
        }
    }
    serde_yaml::Value::Mapping(m)
}

/// `REQ-TRS-SYSMLV2-090`: the `in`/`out`/`inout` parameter declarations of a control node's body
/// (`fork f { in a; out b : T; }`), as `parameters:` entries.
fn control_node_parameters(body: &sysml_v2_parser::ast::FirstMergeBody) -> Vec<serde_yaml::Value> {
    body.braced_elements()
        .unwrap_or(&[])
        .iter()
        .filter_map(|n| match &n.value {
            sysml_v2_parser::ast::FirstMergeBodyElement::Member(m) => match &m.value {
                sysml_v2_parser::ActionDefBodyElement::InOutDecl(d) => {
                    Some(parameter_entry(&d.value.name.s(), &oqr(d.value.type_name).unwrap_or_default(), direction_str(d.value.direction)))
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// The `(after, before, extras)` of a `first X then Y;` statement, or `None` for the bare
/// entry-point marker `first X;` (no edge to emit).
fn first_stmt_parts(f: &sysml_v2_parser::ast::FirstStmt) -> Option<(String, String, SuccessionExtras)> {
    let then = f.then.as_ref()?;
    let first_name = connection_end_display(&f.first.value).unwrap_or_else(|| render_expression(&f.first.value));
    let then_name = connection_end_display(&then.value).unwrap_or_else(|| render_expression(&then.value));
    let mult = |m: &Option<sysml_v2_parser::Node<sysml_v2_parser::ast::Multiplicity>>| m.as_ref().map(|m| multiplicity_text(&m.value));
    let extras = SuccessionExtras {
        name: odn(f.succession_name),
        typed_by: oqr(f.succession_type),
        multiplicity: mult(&f.succession_multiplicity),
        after_multiplicity: mult(&f.first_multiplicity),
        before_multiplicity: mult(&f.then_multiplicity),
    };
    Some((first_name, then_name, extras))
}

/// The `(after, before, extras)` of a keyword-form `succession s : T [m] first [x] a then [y] b;`.
fn succession_usage_parts(s: &sysml_v2_parser::ast::SuccessionUsage) -> (String, String, SuccessionExtras) {
    let mult = |m: &Option<sysml_v2_parser::Node<sysml_v2_parser::ast::Multiplicity>>| m.as_ref().map(|m| multiplicity_text(&m.value));
    let extras = SuccessionExtras {
        name: odn(s.name),
        typed_by: oqr(s.type_name),
        multiplicity: mult(&s.multiplicity),
        after_multiplicity: mult(&s.source_multiplicity),
        before_multiplicity: mult(&s.target_multiplicity),
    };
    (expr_text(&s.source), expr_text(&s.target), extras)
}

/// `REQ-TRS-SYSMLV2-092`: the structural successions declared directly in a `part def` body.
fn part_def_successions(elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartDefBodyElement>]) -> Vec<serde_yaml::Value> {
    use sysml_v2_parser::PartDefBodyElement as E;
    elements
        .iter()
        .filter_map(|n| match &n.value {
            // A `part def` body parses both spellings as `FirstStmt`.
            E::FirstStmt(f) => first_stmt_parts(&f.value),
            _ => None,
        })
        .map(|(a, b, x)| succession_entry(a, b, None, x))
        .collect()
}

/// `REQ-TRS-SYSMLV2-092`: the structural successions declared directly in a `part` usage body.
fn part_usage_successions(elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartUsageBodyElement>]) -> Vec<serde_yaml::Value> {
    use sysml_v2_parser::PartUsageBodyElement as E;
    elements
        .iter()
        .filter_map(|n| match &n.value {
            // A `part` usage body parses both spellings as `SuccessionUsage`.
            E::SuccessionUsage(s) => Some(succession_usage_parts(&s.value)),
            _ => None,
        })
        .map(|(a, b, x)| succession_entry(a, b, None, x))
        .collect()
}

/// The optional own-name and multiplicities of a succession statement (`REQ-TRS-SYSMLV2-081`).
#[derive(Default)]
struct SuccessionExtras {
    name: Option<String>,
    /// `REQ-TRS-SYSMLV2-091`: the succession's own type.
    typed_by: Option<String>,
    multiplicity: Option<String>,
    after_multiplicity: Option<String>,
    before_multiplicity: Option<String>,
}

/// Build and push one `PerformAction` `subActions:` entry, synthesizing a
/// name when `action_name` is empty (an anonymous `perform action { ... }`).
/// Returns the name actually used, so a caller building a
/// `successionConnections:` edge to/from this node uses the same identifier.
fn push_perform_entry(b: &mut ActionBodyBuilder, action_name: &str, type_name: Option<&str>) -> String {
    let name = if action_name.is_empty() { b.synth_name("perform") } else { action_name.to_string() };
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String("PerformAction".to_string()));
    if let Some(tb) = type_name {
        m.insert(ykey("typedBy"), serde_yaml::Value::String(tb.to_string()));
    }
    b.push_sub_action(name.clone(), m);
    name
}

/// `(action name, typing)` of a `perform`: a declared action (`perform action n : T;`) or a
/// reference to an existing one (`perform n;`, the referenced name).
fn perform_parts(p: &sysml_v2_parser::ast::Perform) -> (String, Option<String>) {
    use sysml_v2_parser::ast::PerformActionTarget as T;
    match &p.target {
        T::Action(decl) => (
            odn(decl.value.identification.name).unwrap_or_default(),
            typing_first(decl.value.typing.as_ref()),
        ),
        T::Reference { action, .. } => (qr(*action), None),
    }
}

fn handle_perform_stmt(b: &mut ActionBodyBuilder, p: &sysml_v2_parser::ast::Perform) {
    let (name, ty) = perform_parts(p);
    push_perform_entry(b, &name, ty.as_deref());
}

/// The parts of a standalone `accept` node (`REQ-TRS-SYSMLV2-019`/`-077`/`-078`): the typed form
/// `accept n : T via p;` names the payload `n` with type `T`; the shorthand `accept X via p;` names and
/// types it by the expression text; a time trigger (`accept after e;`, `accept when c;`,
/// `accept at e;`) has neither a name nor a payload and becomes a native `trigger:` map.
struct AcceptParts {
    name: Option<String>,
    payload: Option<String>,
    via: Option<String>,
    trigger: Option<serde_yaml::Mapping>,
}

fn expr_text(e: &sysml_v2_parser::Node<sysml_v2_parser::Expression>) -> String {
    connection_end_display(&e.value).unwrap_or_else(|| render_expression(&e.value))
}

fn accept_parts(a: &sysml_v2_parser::ast::TransitionAccept) -> AcceptParts {
    use sysml_v2_parser::ast::TransitionAccept as A;
    use sysml_v2_parser::ast::TriggerKind as K;
    match a {
        A::Payload(pc, via) => {
            let name = dn(pc.name);
            AcceptParts { name: Some(name.clone()), payload: Some(oqr(pc.type_name).unwrap_or(name)), via: via.as_ref().map(expr_text), trigger: None }
        }
        A::Shorthand(expr, via) => {
            let text = expr_text(expr);
            AcceptParts { name: Some(text.clone()), payload: Some(text), via: via.as_ref().map(expr_text), trigger: None }
        }
        A::TimeTrigger(kind, expr) => {
            let (k, field) = match kind {
                K::After => ("timeOut", "when"),
                K::When => ("change", "condition"),
                K::At => ("at", "when"),
            };
            let mut t = serde_yaml::Mapping::new();
            t.insert(ykey("kind"), serde_yaml::Value::String(k.to_string()));
            t.insert(ykey(field), serde_yaml::Value::String(render_expression(&expr.value)));
            AcceptParts { name: None, payload: None, via: None, trigger: Some(t) }
        }
    }
}

/// `(name, payload text)` of a standalone `send` node. See [`accept_parts`].
fn send_payload(p: &sysml_v2_parser::ast::SendPayload) -> (String, String) {
    match p {
        sysml_v2_parser::ast::SendPayload::Typed(pc) => {
            let name = dn(pc.name);
            (name.clone(), oqr(pc.type_name).unwrap_or(name))
        }
        sysml_v2_parser::ast::SendPayload::Expression(expr) => {
            let text = expr_text(expr);
            (text.clone(), text)
        }
    }
}

/// An action usage's name (empty when anonymous).
fn au_name(au: &sysml_v2_parser::ActionUsage) -> String {
    odn(au.name).unwrap_or_default()
}

/// An action usage's type reference text (empty when untyped).
fn au_type(au: &sysml_v2_parser::ActionUsage) -> String {
    oqr(au.type_name).or_else(|| typing_first(au.typing.as_ref())).unwrap_or_default()
}

fn au_body_elements(au: &sysml_v2_parser::ActionUsage) -> &[sysml_v2_parser::Node<sysml_v2_parser::ActionUsageBodyElement>] {
    match &au.body {
        Some(sysml_v2_parser::ActionUsageBody::Brace { elements, .. }) => elements.as_slice(),
        _ => &[],
    }
}

/// A nested `ActionUsage` found inside an action body becomes a
/// `PerformAction` `subActions:` entry referencing it by `typedBy:` — its
/// own body content is intentionally not recursed into, matching the
/// hand-authored convention already in this repo (`MissionExecution.md`'s
/// `subActions:` reference sibling `ActionDef`s only via `typedBy:`, never
/// inline their bodies). A documented, Syscribe-owned scope cut.
fn handle_nested_action_usage(b: &mut ActionBodyBuilder, au: &sysml_v2_parser::ActionUsage) {
    // `accept`/`send` aren't distinct body-element variants in this grammar
    // — they're `ActionUsage.accept`/`.send: Option<PayloadClause>` fields on
    // an ordinary action usage node. Check those first so `accept X;`/`send
    // Y;` become `AcceptAction`/`SendAction` entries (matching
    // `TakeoffAction.md`/`LandingAction.md`'s hand-authored convention),
    // falling back to the default `PerformAction` otherwise.
    if push_accept_or_send(b, au).is_some() {
        apply_step_annotation(b, au_body_elements(au));
        return;
    }
    if handle_named_step(b, au) {
        return;
    }
    let type_name = au_type(au);
    push_perform_entry(b, &au_name(au), (!type_name.is_empty()).then_some(type_name.as_str()));
}

/// `REQ-TRS-SYSMLV2-060`: `action <name> { <one control statement> }` with no typing, subsetting,
/// redefinition, multiplicity or accept/send clause is a *named* `if`/`while`/`loop`/`for`/`assign`/
/// `terminate` step -- the 0.54 grammar gives those statements no name of their own. The entry takes
/// the usage's name and leaves the synthesized-name counters alone. Returns `false` (nothing pushed)
/// for any other shape, which stays a `PerformAction`.
fn handle_named_step(b: &mut ActionBodyBuilder, au: &sysml_v2_parser::ActionUsage) -> bool {
    use sysml_v2_parser::ActionUsageBodyElement as E;
    let plain = !au_name(au).is_empty()
        && au_type(au).is_empty()
        && au.typing.is_none()
        && au.subsets.is_none()
        && au.redefines.is_none()
        && au.multiplicity.is_none()
        && !au.is_abstract
        && !au.is_variation
        && !au.is_reference;
    let Some(sysml_v2_parser::ActionUsageBody::Brace { elements, .. }) = &au.body else { return false };
    // `REQ-TRS-SYSMLV2-069`: a `@SyscribeStep` annotation beside the statement is not a second statement.
    let stmts: Vec<_> = elements
        .iter()
        .filter(|e| !matches!(&e.value, E::Annotating(a) if annotating_meta(a).is_some()))
        .collect();
    if !plain || stmts.len() != 1 {
        return false;
    }
    let before = b.sub_actions.len();
    b.forced_name = Some(au_name(au));
    match &stmts[0].value {
        E::Assign(a) => handle_assign(b, &a.value),
        E::WhileStmt(w) => handle_while(b, &w.value),
        E::LoopStmt(l) => handle_loop(b, &l.value),
        E::ForLoop(f) => handle_for_loop(b, &f.value),
        E::IfStmt(i) => handle_if(b, &i.value),
        E::TerminateStmt(t) => handle_terminate(b, &t.value),
        _ => {}
    }
    b.forced_name = None;
    let pushed = b.sub_actions.len() > before;
    if pushed {
        apply_step_annotation(b, au_body_elements(au));
    }
    pushed
}

/// `REQ-TRS-SYSMLV2-069`/`-070`: the fields the pinned 0.54 grammar has no syntax for travel in a
/// `@SyscribeStep { via = '…'; feature = '…' (the `referent`); valueKind = '…'; triggerKind = '…';
/// triggerCondition = '…'; loopKind = 'until'; condition = '…'; }` metadata annotation inside the
/// step's own body. This folds it into the entry just pushed (the last `subActions:` item).
/// `loopKind`/`condition` only ever turn an unconditioned `loop` into an `until` loop.
fn apply_step_annotation(b: &mut ActionBodyBuilder, elements: &[sysml_v2_parser::Node<sysml_v2_parser::ActionUsageBodyElement>]) {
    let Some(ann) = elements.iter().rev().find_map(|e| match &e.value {
        sysml_v2_parser::ActionUsageBodyElement::Annotating(a) => {
            annotating_meta(a).filter(|m| qr(m.type_reference) == "SyscribeStep")
        }
        _ => None,
    }) else {
        return;
    };
    let Some(serde_yaml::Value::Mapping(entry)) = b.sub_actions.last_mut() else { return };
    let kind = entry.get(ykey("kind")).and_then(|v| v.as_str()).unwrap_or("").to_string();
    let get = |k: &str| attribute_body_string(&ann.body, k);
    let set = |entry: &mut serde_yaml::Mapping, k: &str, v: String| {
        entry.insert(ykey(k), serde_yaml::Value::String(v));
    };
    if matches!(kind.as_str(), "AcceptAction" | "SendAction") {
        if let Some(v) = get("via") {
            set(entry, "via", v);
        }
    }
    if kind == "AcceptAction" {
        if let (Some(k), Some(c)) = (get("triggerKind"), get("triggerCondition")) {
            let mut t = serde_yaml::Mapping::new();
            t.insert(ykey("kind"), serde_yaml::Value::String(k));
            t.insert(ykey("condition"), serde_yaml::Value::String(c));
            entry.insert(ykey("trigger"), serde_yaml::Value::Mapping(t));
        }
    }
    if kind == "AssignmentAction" {
        // The annotation spells `referent` as `feature`: the 0.54 lexer reads an attribute named
        // `referent` as the `ref` keyword plus a stray identifier and drops the value.
        for (ann_key, field) in [("feature", "referent"), ("valueKind", "valueKind")] {
            if let Some(v) = get(ann_key) {
                set(entry, field, v);
            }
        }
    }
    if kind == "LoopAction"
        && entry.get(ykey("loopKind")).and_then(|v| v.as_str()) == Some("loop")
        && get("loopKind").as_deref() == Some("until")
    {
        if let Some(c) = get("condition") {
            set(entry, "loopKind", "until".to_string());
            set(entry, "condition", c);
        }
    }
}

/// Build and push one `AcceptAction`/`SendAction` `subActions:` entry —
/// `{name, kind, payload}`. Identified by the `PayloadClause`'s own name,
/// not the enclosing `ActionUsage.name` — confirmed against the parser's
/// actual output that a bare `accept cmd : StartCmd;`/`send ack : AckCmd;`
/// (no separate action name given) sets `ActionUsage.name` to the literal
/// keyword itself (`"accept"`/`"send"`), which would collide across
/// multiple such statements in one body; the payload's own name is the
/// semantically meaningful, and actually distinct, identity here.
/// `payload:` is the `PayloadClause`'s own type (falling back to its bare
/// name when untyped, same rule `render_transition_accept`'s `Payload` case
/// already uses).
fn push_payload_entry(b: &mut ActionBodyBuilder, name: String, payload_text: String, kind: &str) {
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String(kind.to_string()));
    m.insert(ykey("payload"), serde_yaml::Value::String(payload_text));
    b.push_sub_action(name, m);
}

/// Set one string field on the entry just pushed (the last `subActions:` item).
fn set_last_field(b: &mut ActionBodyBuilder, k: &str, v: serde_yaml::Value) {
    if let Some(serde_yaml::Value::Mapping(m)) = b.sub_actions.last_mut() {
        m.insert(ykey(k), v);
    }
}

/// One `AcceptAction` entry from a parsed accept (`REQ-TRS-SYSMLV2-077`/`-078`); `explicit` is the
/// name of an enclosing `action <n> accept ...;` usage, used for a payload-less time trigger.
fn push_accept_parts(b: &mut ActionBodyBuilder, acc: &sysml_v2_parser::ast::TransitionAccept, explicit: Option<String>) -> Option<String> {
    let parts = accept_parts(acc);
    let name = match (&parts.payload, explicit) {
        (Some(_), _) => parts.name.clone().unwrap_or_default(),
        (None, Some(n)) => n,
        (None, None) => b.synth_name("accept"),
    };
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String("AcceptAction".to_string()));
    if let Some(p) = parts.payload {
        m.insert(ykey("payload"), serde_yaml::Value::String(p));
    }
    if let Some(v) = parts.via {
        m.insert(ykey("via"), serde_yaml::Value::String(v));
    }
    if let Some(t) = parts.trigger {
        m.insert(ykey("trigger"), serde_yaml::Value::Mapping(t));
    }
    b.push_sub_action(name.clone(), m);
    Some(name)
}

/// A standalone/`then` `accept` or `send` usage as an `AcceptAction`/`SendAction` entry with its native
/// `via`/`to`/`trigger` (`REQ-TRS-SYSMLV2-077`/`-078`). Returns the entry name, or `None` when `au`
/// is neither.
fn push_accept_or_send(b: &mut ActionBodyBuilder, au: &sysml_v2_parser::ActionUsage) -> Option<String> {
    if let Some(acc) = &au.accept {
        let explicit = (au.keyword == sysml_v2_parser::ast::ActionUsageKeyword::Action).then(|| au_name(au)).filter(|n| !n.is_empty());
        return push_accept_parts(b, acc, explicit);
    }
    if let Some(sp) = &au.send {
        let (name, payload) = send_payload(sp);
        push_payload_entry(b, name.clone(), payload, "SendAction");
        if let Some(v) = &au.via {
            set_last_field(b, "via", serde_yaml::Value::String(expr_text(v)));
        }
        if let Some(t) = &au.to {
            set_last_field(b, "to", serde_yaml::Value::String(expr_text(t)));
        }
        return Some(name);
    }
    None
}

/// `a.b.c` -> `("a.b", "c")` for a plain dotted feature chain; `None` for a single segment or
/// anything that is not a simple chain (calls, indexing, spaces).
pub(crate) fn split_feature_chain(chain: &str) -> Option<(String, String)> {
    let plain = chain.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.');
    let (target, referent) = chain.rsplit_once('.')?;
    (plain && !target.is_empty() && !referent.is_empty() && !target.ends_with('.')).then(|| (target.to_string(), referent.to_string()))
}

fn handle_assign(b: &mut ActionBodyBuilder, a: &sysml_v2_parser::ast::AssignStmt) {
    let name = b.synth_name("assign");
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String("AssignmentAction".to_string()));
    // `REQ-TRS-SYSMLV2-079`: `assign a.b := v;` assigns feature `b` of `a`, the native `target` + `referent`.
    let lhs = render_expression(&a.lhs.value);
    match split_feature_chain(&lhs) {
        Some((target, referent)) => {
            m.insert(ykey("target"), serde_yaml::Value::String(target));
            m.insert(ykey("referent"), serde_yaml::Value::String(referent));
        }
        None => {
            m.insert(ykey("target"), serde_yaml::Value::String(lhs));
        }
    }
    m.insert(ykey("value"), serde_yaml::Value::String(render_expression(&a.rhs.value)));
    b.push_sub_action(name, m);
}

/// Fold a nested action-body's own control nodes/successions up onto `b` —
/// `controlNodes:`/`successionConnections:` are flat, owning-`ActionDef`-wide
/// lists (matching the hand-authored convention), never nested per branch,
/// regardless of how deeply the `fork`/`join`/etc. is nested inside
/// `if`/`while`/`loop`/`for` bodies.
fn absorb(b: &mut ActionBodyBuilder, inner: ActionBody) -> Vec<serde_yaml::Value> {
    b.control_nodes.extend(inner.control_nodes);
    b.succession_connections.extend(inner.succession_connections);
    inner.sub_actions
}

fn handle_while(b: &mut ActionBodyBuilder, w: &sysml_v2_parser::ast::WhileStmt) {
    let name = b.synth_name("while");
    let inner = build_action_def_body(action_def_body_elements(&w.body.body));
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String("LoopAction".to_string()));
    m.insert(ykey("loopKind"), serde_yaml::Value::String("while".to_string()));
    m.insert(ykey("condition"), serde_yaml::Value::String(render_expression(&w.condition.value)));
    // `REQ-TRS-SYSMLV2-089`: `while c { } until d;` keeps both conditions.
    if let Some(u) = &w.until {
        m.insert(ykey("untilCondition"), serde_yaml::Value::String(render_expression(&u.expression.value)));
    }
    let sub_actions = absorb(b, inner);
    if !sub_actions.is_empty() {
        m.insert(ykey("body"), serde_yaml::Value::Sequence(sub_actions));
    }
    b.push_sub_action(name, m);
}

fn handle_loop(b: &mut ActionBodyBuilder, l: &sysml_v2_parser::ast::LoopStmt) {
    let name = b.synth_name("loop");
    let inner = build_action_def_body(action_def_body_elements(&l.body.body));
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String("LoopAction".to_string()));
    // `REQ-TRS-SYSMLV2-079`: `loop { } until c;` is the native `loopKind: until`.
    match &l.until {
        Some(u) => {
            m.insert(ykey("loopKind"), serde_yaml::Value::String("until".to_string()));
            m.insert(ykey("condition"), serde_yaml::Value::String(render_expression(&u.expression.value)));
        }
        None => {
            m.insert(ykey("loopKind"), serde_yaml::Value::String("loop".to_string()));
        }
    }
    let sub_actions = absorb(b, inner);
    if !sub_actions.is_empty() {
        m.insert(ykey("body"), serde_yaml::Value::Sequence(sub_actions));
    }
    b.push_sub_action(name, m);
}

fn handle_for_loop(b: &mut ActionBodyBuilder, f: &sysml_v2_parser::ast::ForLoop) {
    let name = b.synth_name("for");
    let inner = build_action_def_body(action_def_body_elements(&f.body.body));
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String("LoopAction".to_string()));
    m.insert(ykey("loopKind"), serde_yaml::Value::String("for".to_string()));
    m.insert(ykey("variable"), serde_yaml::Value::String(odn(f.variable.value.identification.name).unwrap_or_default()));
    m.insert(ykey("sequence"), serde_yaml::Value::String(render_expression(&f.in_parameter.expression.value)));
    let sub_actions = absorb(b, inner);
    if !sub_actions.is_empty() {
        m.insert(ykey("body"), serde_yaml::Value::Sequence(sub_actions));
    }
    b.push_sub_action(name, m);
}

fn handle_if(b: &mut ActionBodyBuilder, i: &sysml_v2_parser::ast::IfStmt) {
    let name = b.synth_name("if");
    let then_inner = build_action_def_body(branch_body_elements(&i.then_body));
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String("IfAction".to_string()));
    m.insert(ykey("condition"), serde_yaml::Value::String(render_expression(&i.condition.value)));
    let then_actions = absorb(b, then_inner);
    if !then_actions.is_empty() {
        m.insert(ykey("then"), serde_yaml::Value::Sequence(then_actions));
    }
    if let Some(else_body) = &i.else_body {
        let else_inner = build_action_def_body(branch_body_elements(else_body));
        let else_actions = absorb(b, else_inner);
        if !else_actions.is_empty() {
            m.insert(ykey("else"), serde_yaml::Value::Sequence(else_actions));
        }
    }
    b.push_sub_action(name, m);
}

/// The statements of an `if` branch: a braced body, or the single-statement shorthand.
fn branch_body_elements(body: &sysml_v2_parser::ast::ActionBranchBody) -> &[sysml_v2_parser::Node<sysml_v2_parser::ActionDefBodyElement>] {
    match body {
        sysml_v2_parser::ast::ActionBranchBody::Braced(b) => action_def_body_elements(b),
        sysml_v2_parser::ast::ActionBranchBody::Shorthand(e) => std::slice::from_ref(&**e),
    }
}

/// The node name of a fork/join/decide/merge statement (`None` for the anonymous form).
fn control_node_expr(d: &sysml_v2_parser::ast::ControlNodeDeclaration) -> Option<&sysml_v2_parser::Node<sysml_v2_parser::Expression>> {
    match d {
        sysml_v2_parser::ast::ControlNodeDeclaration::Named(e) => Some(e),
        sysml_v2_parser::ast::ControlNodeDeclaration::Anonymous => None,
    }
}

fn handle_terminate(b: &mut ActionBodyBuilder, t: &sysml_v2_parser::ast::TerminateStmt) {
    let name = b.synth_name("terminate");
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String("TerminateAction".to_string()));
    if let Some(target) = &t.target {
        let disp = connection_end_display(&target.value).unwrap_or_else(|| render_expression(&target.value));
        m.insert(ykey("target"), serde_yaml::Value::String(disp));
    }
    b.push_sub_action(name, m);
}

fn handle_control_node(b: &mut ActionBodyBuilder, decl: &sysml_v2_parser::ast::ControlNodeDeclaration, kind: &str, body: &sysml_v2_parser::ast::FirstMergeBody) {
    // An anonymous node (`fork;`) has no name to key a succession against; 0.54 could not parse one.
    let Some(expr) = control_node_expr(decl) else { return };
    let name = connection_end_display(&expr.value).unwrap_or_else(|| render_expression(&expr.value));
    b.push_control_node(name, kind, control_node_parameters(body));
}

/// `first X [then Y];` — the succession edge (`X` → `Y`). A bare `first X;`
/// with no `then` is an entry-point marker with no edge to emit; there is no
/// dedicated "entry point" field in this schema, so it's a documented no-op.
fn handle_first_stmt(b: &mut ActionBodyBuilder, f: &sysml_v2_parser::ast::FirstStmt) {
    let Some((first_name, then_name, extras)) = first_stmt_parts(f) else { return };
    b.push_guarded_succession(first_name, then_name, None, extras);
}

/// `first a if <guard> then b;` (`REQ-TRS-SYSMLV2-074`): the succession edge `a` -> `b` with its
/// guard as the native `guard:` text. 0.54 exposed no slot for the guard.
fn handle_guarded_succession(b: &mut ActionBodyBuilder, g: &sysml_v2_parser::ast::GuardedSuccession) {
    let first = qr_segments(g.first).join(".");
    let then = qr_segments(g.target.value.target).join(".");
    let name = g.succession.as_ref().and_then(|d| odn(d.declaration.value.identification.name));
    // `REQ-TRS-SYSMLV2-091`: the succession's own type.
    let typed_by = g.succession.as_ref().and_then(|d| typing_first(d.declaration.value.typing.as_ref()));
    b.push_guarded_succession(first, then, Some(render_expression(&g.guard.value)), SuccessionExtras { name, typed_by, ..Default::default() });
}

/// `then <target>;` succession shorthand — connects from whatever node was
/// most recently converted (`last_named`) to `target`. Silently dropped when
/// there's no preceding node to connect from (e.g. the very first body
/// element is a bare `then X;`, which isn't valid SysML v2 but stay
/// defensive rather than panic).
fn handle_then_action(b: &mut ActionBodyBuilder, t: &sysml_v2_parser::ast::ThenAction) {
    use sysml_v2_parser::ast::ThenTarget as T;
    let Some(after) = b.last_named.clone() else { return };
    let node_name = |d: &sysml_v2_parser::ast::ControlNodeDeclaration| {
        control_node_expr(d).map(|e| connection_end_display(&e.value).unwrap_or_else(|| render_expression(&e.value)))
    };
    let before = match &t.target {
        T::Action(au) => {
            let type_name = au_type(&au.value);
            push_perform_entry(b, &au_name(&au.value), (!type_name.is_empty()).then_some(type_name.as_str()))
        }
        T::Perform(p) => {
            let (name, ty) = perform_parts(&p.value);
            push_perform_entry(b, &name, ty.as_deref())
        }
        T::Merge(m) => {
            let Some(name) = node_name(&m.value.declaration) else { return };
            b.last_named = Some(name.clone());
            name
        }
        T::Feature(expr) => {
            let name = connection_end_display(&expr.value).unwrap_or_else(|| render_expression(&expr.value));
            b.last_named = Some(name.clone());
            name
        }
        // `REQ-TRS-SYSMLV2-082`: the node the succession lands on is created here too, so the edge
        // has a real endpoint. An anonymous `then fork;` has no name to key an edge against.
        T::Fork(f) => {
            let Some(name) = node_name(&f.value.declaration) else { return };
            b.push_control_node(name.clone(), "ForkNode", control_node_parameters(&f.value.body));
            name
        }
        T::Decide(d) => {
            let Some(name) = node_name(&d.value.declaration) else { return };
            b.push_control_node(name.clone(), "DecisionNode", control_node_parameters(&d.value.body));
            name
        }
        T::Join(j) => {
            let Some(name) = node_name(&j.value.declaration) else { return };
            b.push_control_node(name.clone(), "JoinNode", control_node_parameters(&j.value.body));
            name
        }
        T::Accept(acc) => {
            let Some(name) = push_accept_parts(b, &acc.value, None) else { return };
            name
        }
        T::Send(au) => {
            let Some(name) = push_accept_or_send(b, &au.value) else { return };
            name
        }
        T::If(i) => {
            handle_if(b, &i.value);
            let Some(name) = b.last_named.clone() else { return };
            name
        }
    };
    b.push_succession(after, before);
}

/// Extract a `Node<T>` body enum's already-sliced members, or `&[]` for the
/// `;`-only form — `ActionDefBody` is the recursion target for *every*
/// nested control-flow body (`IfStmt.then_body`/`.else_body`,
/// `WhileStmt.body`, `LoopStmt.body`, `ForLoop.body` are all typed
/// `ActionDefBody`, confirmed against the parser's AST, regardless of
/// whether the enclosing construct itself was found inside an `ActionDef` or
/// an `ActionUsage` body) — so this one helper covers every recursive case.
fn action_def_body_elements(body: &sysml_v2_parser::ActionDefBody) -> &[sysml_v2_parser::Node<sysml_v2_parser::ActionDefBodyElement>] {
    match body {
        sysml_v2_parser::ActionDefBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::ActionDefBody::Semicolon { .. } => &[],
    }
}

/// Walk one `action def` (or nested control-flow) body-element slice,
/// producing its `ActionBody`. The single recursion point for every nested
/// case (see [`action_def_body_elements`]'s doc comment for why one walker
/// suffices for both enclosing-context kinds).
fn build_action_def_body(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ActionDefBodyElement>]) -> ActionBody {
    use sysml_v2_parser::ActionDefBodyElement as E;
    let mut b = ActionBodyBuilder::default();
    for n in elements {
        match &n.value {
            E::Perform(p) => handle_perform_stmt(&mut b, &p.value),
            E::ActionUsage(au) => handle_nested_action_usage(&mut b, &au.value),
            E::Assign(a) => handle_assign(&mut b, &a.value),
            E::WhileStmt(w) => handle_while(&mut b, &w.value),
            E::LoopStmt(l) => handle_loop(&mut b, &l.value),
            E::ForLoop(f) => handle_for_loop(&mut b, &f.value),
            E::IfStmt(i) => handle_if(&mut b, &i.value),
            E::TerminateStmt(t) => handle_terminate(&mut b, &t.value),
            E::ForkStmt(f) => handle_control_node(&mut b, &f.value.declaration, "ForkNode", &f.value.body),
            E::JoinStmt(j) => handle_control_node(&mut b, &j.value.declaration, "JoinNode", &j.value.body),
            E::DecisionStmt(d) => handle_control_node(&mut b, &d.value.declaration, "DecisionNode", &d.value.body),
            E::MergeStmt(m) => handle_control_node(&mut b, &m.value.declaration, "MergeNode", &m.value.body),
            E::FirstStmt(f) => handle_first_stmt(&mut b, &f.value),
            E::GuardedSuccession(g) => handle_guarded_succession(&mut b, &g.value),
            E::ThenAction(t) => handle_then_action(&mut b, &t.value),
            // PartUsage/ItemUsage nested in an action body are structural,
            // not behavioral — handled separately by
            // `convert_action_def_body_element` (real, separate
            // `RawElement`s), not here. Bind/FlowUsage/AssertConstraint/
            // OccurrenceUsage/Decl/DefaultReferenceUsage/InOutDecl/RefDecl/
            // StateUsage nested in an action body/Doc/Annotation/
            // MetadataAnnotation (handled separately)/MetadataKeywordUsage/
            // Error — outside REQ-TRS-SYSMLV2-019's fixed set.
            _ => {}
        }
    }
    ActionBody {
        sub_actions: b.sub_actions,
        control_nodes: b.control_nodes,
        succession_connections: b.succession_connections,
    }
}

/// Walk one `action` *usage*'s own top-level body-element slice.
/// `ActionUsageBodyElement` is a distinct Rust type from
/// `ActionDefBodyElement` (structurally near-identical, but no `Perform`
/// variant — a nested `ActionUsage` covers that case here), so this needs
/// its own top-level dispatch; every per-construct handler it calls is
/// shared with [`build_action_def_body`] since the inner structs
/// (`WhileStmt`/`IfStmt`/...) are the same types regardless of which body
/// enum wraps them.
fn build_action_usage_body(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ActionUsageBodyElement>]) -> ActionBody {
    use sysml_v2_parser::ActionUsageBodyElement as E;
    let mut b = ActionBodyBuilder::default();
    for n in elements {
        match &n.value {
            E::ActionUsage(au) => handle_nested_action_usage(&mut b, &au.value),
            E::Assign(a) => handle_assign(&mut b, &a.value),
            E::WhileStmt(w) => handle_while(&mut b, &w.value),
            E::LoopStmt(l) => handle_loop(&mut b, &l.value),
            E::ForLoop(f) => handle_for_loop(&mut b, &f.value),
            E::IfStmt(i) => handle_if(&mut b, &i.value),
            E::TerminateStmt(t) => handle_terminate(&mut b, &t.value),
            E::ForkStmt(f) => handle_control_node(&mut b, &f.value.declaration, "ForkNode", &f.value.body),
            E::JoinStmt(j) => handle_control_node(&mut b, &j.value.declaration, "JoinNode", &j.value.body),
            E::DecisionStmt(d) => handle_control_node(&mut b, &d.value.declaration, "DecisionNode", &d.value.body),
            E::MergeStmt(m) => handle_control_node(&mut b, &m.value.declaration, "MergeNode", &m.value.body),
            E::FirstStmt(f) => handle_first_stmt(&mut b, &f.value),
            E::GuardedSuccession(g) => handle_guarded_succession(&mut b, &g.value),
            E::ThenAction(t) => handle_then_action(&mut b, &t.value),
            _ => {}
        }
    }
    ActionBody {
        sub_actions: b.sub_actions,
        control_nodes: b.control_nodes,
        succession_connections: b.succession_connections,
    }
}

/// Recurse into a nested `part`/`item` usage inside an `action def` body —
/// the only `ActionDefBodyElement` variants that produce a real, separate
/// `RawElement` (structural, not behavioral). Everything else is either
/// `subActions:`/`controlNodes:` data (see [`build_action_def_body`]) or
/// outside REQ-TRS-SYSMLV2-019's fixed set.
fn convert_action_def_body_element(
    elem: &sysml_v2_parser::ActionDefBodyElement,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    use sysml_v2_parser::ActionDefBodyElement as E;
    match elem {
        E::PartUsage(node) => convert_part_usage(&node.value, qname, file_path, out),
        E::ItemUsage(node) => convert_item_usage(&node.value, qname, file_path, out),
        _ => {}
    }
}

/// See [`convert_action_def_body_element`].
fn convert_action_usage_body_element(
    elem: &sysml_v2_parser::ActionUsageBodyElement,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    use sysml_v2_parser::ActionUsageBodyElement as E;
    match elem {
        E::PartUsage(node) => convert_part_usage(&node.value, qname, file_path, out),
        E::ItemUsage(node) => convert_item_usage(&node.value, qname, file_path, out),
        _ => {}
    }
}

fn convert_action_def(a: &sysml_v2_parser::ActionDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&a.identification) else {
        return; // anonymous action def: no identity to qname against
    };
    let action_qname = format!("{qname}::{name}");
    let elements = action_def_body_elements(&a.body);
    let body = build_action_def_body(elements);
    let spec = Spec {
        supertype: a.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        ..Default::default()
    }
    .with_syscribe_meta(action_def_syscribe_meta(elements))
    .with_doc(action_def_doc(elements))
    .with_metadata(body_metadata(elements))
    .with_behavior(body.sub_actions, body.control_nodes, body.succession_connections);
    push_synth(out, &action_qname, file_path, ElementType::ActionDef, &name, spec);
    convert_body_with_prefixes(elements, out, |e, out| convert_action_def_body_element(e, &action_qname, file_path, out));
}

fn convert_action_usage(a: &sysml_v2_parser::ActionUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let a_name = au_name(a);
    if a_name.is_empty() {
        return; // anonymous usage: no identity to qname against
    }
    let action_qname = format!("{qname}::{a_name}");
    let elements = au_body_elements(a);
    let body = build_action_usage_body(elements);
    let a_type = au_type(a);
    let spec = Spec {
        typed_by: (!a_type.is_empty()).then(|| a_type.clone()),
        is_variation: a.is_variation.then_some(true),
        ..Default::default()
    }
    .with_syscribe_meta(action_usage_syscribe_meta(elements))
    .with_doc(action_usage_doc(elements))
    .with_metadata(body_metadata(elements))
    .with_behavior(body.sub_actions, body.control_nodes, body.succession_connections);
    push_synth(out, &action_qname, file_path, ElementType::Action, &a_name, spec);
    convert_body_with_prefixes(elements, out, |e, out| convert_action_usage_body_element(e, &action_qname, file_path, out));
}

/// `REQ-TRS-SYSMLV2-020` — a `view def` synthesizes a real `ViewDef`.
/// Unlike a `view` usage (see [`convert_view_usage`]), `ViewDefBodyElement`
/// carries no `Expose`/`Satisfy` variant at all — the grammar structurally
/// cannot carry `expose:`/`viewpoint:` here — so only `rendering:`/`doc` are
/// ever populated. No recursion into nested elements: none of
/// `ViewDefBodyElement`'s variants (`Doc`, `MetadataAnnotation`, `Filter`,
/// `ViewRendering`) produce a further, separate `RawElement`.
fn convert_view_def(v: &sysml_v2_parser::ast::ViewDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&v.identification) else {
        return; // anonymous view def: no identity to qname against
    };
    let view_qname = format!("{qname}::{name}");
    let elements = match &v.body {
        sysml_v2_parser::ast::ViewDefBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::ast::ViewDefBody::Semicolon { .. } => &[],
    };
    let spec = Spec {
        supertype: v.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        ..Default::default()
    }
    .with_doc(view_def_doc(elements))
    .with_metadata(body_metadata(elements))
    .with_view(Vec::new(), None, view_def_rendering(elements));
    push_synth(out, &view_qname, file_path, ElementType::ViewDef, &name, spec);
}

/// `REQ-TRS-SYSMLV2-020` — a `view` usage synthesizes a real `View`, the
/// only place `expose:`/`viewpoint:` can actually be lifted from per this
/// grammar (see [`convert_view_def`]'s note). No recursion: none of
/// `ViewBodyElement`'s variants produce a further, separate `RawElement`.
fn convert_view_usage(v: &sysml_v2_parser::ast::ViewUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(v_name) = odn(v.name).filter(|n| !n.is_empty()) else {
        return; // anonymous/redefinition-only usage: no identity to qname against
    };
    let view_qname = format!("{qname}::{v_name}");
    let elements = match &v.body {
        sysml_v2_parser::ast::ViewBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::ast::ViewBody::Semicolon { .. } => &[],
    };
    let spec = Spec {
        typed_by: typing_first(v.typing.as_ref()),
        ..Default::default()
    }
    .with_doc(view_usage_doc(elements))
    .with_metadata(with_prefix_keywords(&v.prefix, body_metadata(elements)))
    .with_view(
        view_expose_entries(elements),
        view_satisfy_viewpoint(elements),
        view_usage_rendering(elements),
    );
    push_synth(out, &view_qname, file_path, ElementType::View, &v_name, spec);
}

/// `REQ-TRS-SYSMLV2-021` — a `viewpoint def` synthesizes a real
/// `ViewpointDef`. `methods:`/`satisfiedBy:` are deliberately never
/// populated — no AST source exists (the relationship only exists in the
/// other direction, as a `view`'s own `satisfy <viewpoint>;` clause), and
/// computing it here would point the link the wrong way per §12.1's OSLC
/// upstream-link-direction rule. No recursion: `RequirementDefBody`'s own
/// nested-element variants (`RequirementUsage`, `AttributeDef`, ...) are not
/// walked here, matching `convert_requirement_def`'s own existing posture.
fn convert_viewpoint_def(v: &sysml_v2_parser::ast::ViewpointDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&v.identification) else {
        return;
    };
    let vp_qname = format!("{qname}::{name}");
    let (stakeholders, concerns) = collect_requirement_body_stakeholders_concerns(&v.body);
    let spec = Spec {
        supertype: v.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        ..Default::default()
    }
    .with_doc(requirement_def_body_doc(&v.body))
    .with_metadata(requirement_def_body_metadata(&v.body))
    .with_stakeholders_concerns(stakeholders, concerns);
    push_synth(out, &vp_qname, file_path, ElementType::ViewpointDef, &name, spec);
}

/// `REQ-TRS-SYSMLV2-021` — a `viewpoint` usage synthesizes a real `View`.
/// No dedicated `Viewpoint` usage `ElementType` exists in the native schema
/// — this maps onto `ElementType::View`, matching the doc's own framing of
/// `View` as "usage of a ViewDef or ViewpointDef".
fn convert_viewpoint_usage(v: &sysml_v2_parser::ast::ViewpointUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let v_name = dn(v.name);
    if v_name.is_empty() {
        return;
    }
    let vp_qname = format!("{qname}::{v_name}");
    let (stakeholders, concerns) = collect_requirement_body_stakeholders_concerns(&v.body);
    let spec = Spec {
        // `ViewpointUsage.type_name` is a non-`Option<String>` (empty-string
        // sentinel for "untyped"), unlike `ViewUsage.type_name`'s
        // `Option<String>` — treat "" as absent.
        typed_by: oqr(v.type_name).filter(|t| !t.is_empty()),
        ..Default::default()
    }
    .with_doc(requirement_def_body_doc(&v.body))
    .with_metadata(requirement_def_body_metadata(&v.body))
    .with_stakeholders_concerns(stakeholders, concerns);
    push_synth(out, &vp_qname, file_path, ElementType::View, &v_name, spec);
}

/// `REQ-TRS-SYSMLV2-023` — a `concern def`/`concern` usage synthesizes a
/// real `ConcernDef`/`Concern`. Unlike View/Viewpoint/Rendering, the
/// vendored parser has no separate `ConcernDef` struct at all: one
/// `ConcernUsage` AST node parses both textual forms, `is_definition`
/// the sole discriminator — this one function branches on it instead of
/// having a `_def`/`_usage` pair.
///
/// `ConcernUsage.type_name` carries a double meaning the AST itself doesn't
/// disambiguate: it comes from the *same* shared `feature_usage_header` the
/// parser calls regardless of `is_definition` (confirmed against
/// `concern_usage`'s own parser function). For `concern def X : Y` this is
/// semantically a supertype ("X specializes Y"); for a bare `concern x : Y`
/// usage it's semantically a typedBy. Exactly one of `supertype`/`typed_by`
/// is ever set below, never both.
fn convert_concern_usage(c: &sysml_v2_parser::ast::ConcernUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let c_name = dn(c.name);
    if c_name.is_empty() {
        return; // anonymous concern/concern def: no identity to qname against
    }
    let concern_qname = format!("{qname}::{c_name}");
    let (stakeholders, _) = collect_requirement_body_stakeholders_concerns(&c.body);
    let ty = if c.is_definition { ElementType::ConcernDef } else { ElementType::Concern };
    let spec = Spec {
        supertype: c.is_definition.then(|| oqr(c.type_name)).flatten(),
        typed_by: (!c.is_definition).then(|| oqr(c.type_name)).flatten(),
        subject: concern_body_subject(&c.body),
        ..Default::default()
    }
    .with_doc(requirement_def_body_doc(&c.body))
    .with_metadata(requirement_def_body_metadata(&c.body))
    // `ConcernDef` has no `concerns:` self-field (§8.11.5) -- only the
    // stakeholders half of the tuple is used; `requires:`/`assume:`/
    // `parameters:` are explicitly out of scope for this requirement (see
    // `REQ-TRS-SYSMLV2-023`'s Scope section).
    .with_stakeholders_concerns(stakeholders, Vec::new());
    push_synth(out, &concern_qname, file_path, ty, &c_name, spec);
}

/// `REQ-TRS-SYSMLV2-022` — a `rendering def` synthesizes a real
/// `RenderingDef`. Thinnest of the six: `RenderingDefBodyElement` carries no
/// field the native schema (§8.14.4: `supertype`, `features`) has room for
/// beyond `doc`/`supertype` — `Filter`/nested `ViewRendering` stay unmapped,
/// same "no native field" posture as `ViewDefBodyElement::Filter`.
fn convert_rendering_def(r: &sysml_v2_parser::ast::RenderingDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&r.identification) else {
        return;
    };
    let rendering_qname = format!("{qname}::{name}");
    let elements = match &r.body {
        sysml_v2_parser::ast::RenderingDefBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::ast::RenderingDefBody::Semicolon { .. } => &[],
    };
    let spec = Spec {
        supertype: r.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        ..Default::default()
    }
    .with_doc(rendering_def_doc(elements))
    .with_metadata(body_metadata(elements));
    push_synth(out, &rendering_qname, file_path, ElementType::RenderingDef, &name, spec);
}

/// `REQ-TRS-SYSMLV2-022` — a `rendering` usage synthesizes a real
/// `Rendering`. `RenderingUsageBodyElement::ViewUsage` (the narrow nested
/// `view :>> columnView[N] { render ...; }` redefinition shape, confirmed
/// against real SysML v2 standard-library fixtures) is deliberately not
/// recursed into — narrow, non-representative of ordinary modeling, and
/// there is no native "nested view" field to hold it.
fn convert_rendering_usage(r: &sysml_v2_parser::ast::RenderingUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(r_name) = odn(r.name).filter(|n| !n.is_empty()) else {
        return;
    };
    let rendering_qname = format!("{qname}::{r_name}");
    let elements = match &r.body {
        sysml_v2_parser::ast::RenderingUsageBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::ast::RenderingUsageBody::Semicolon { .. } => &[],
    };
    let spec = Spec {
        typed_by: oqr(r.type_name),
        ..Default::default()
    }
    .with_doc(rendering_usage_doc(elements))
    .with_metadata(body_metadata(elements));
    push_synth(out, &rendering_qname, file_path, ElementType::Rendering, &r_name, spec);
}

/// Walk the merged package tree, emitting `RawElement`s under `qname`.
/// `REQ-TRS-SYSMLV2-048`: native multiplicity text -- `2`, `0..1`, `1..*`, `*`.
fn multiplicity_text(m: &sysml_v2_parser::ast::Multiplicity) -> String {
    let bound = |b: &Option<Box<sysml_v2_parser::Node<sysml_v2_parser::Expression>>>| {
        b.as_ref().map_or_else(|| "*".to_string(), |e| render_expression(&e.value))
    };
    match (&m.lower, &m.upper) {
        (None, None) => "*".to_string(),
        (l, u) if l == u => bound(l),
        (l, u) => format!("{}..{}", bound(l), bound(u)),
    }
}

fn subsetting_targets(r: &sysml_v2_parser::ast::SubsettingRelationship) -> Vec<String> {
    r.target.iter().map(|t| qr(*t)).collect()
}

/// One `alias <name> for <target>;` as an `aliases:` entry (`REQ-TRS-SYSMLV2-043`/`-095`);
/// `None` for an anonymous alias. Must run under the declaring file's document.
fn alias_entry(a: &sysml_v2_parser::ast::AliasDef) -> Option<serde_yaml::Value> {
    let id = &a.identification;
    let (name, short) = match (odn(id.name), odn(id.short_name)) {
        (Some(n), s) => (n, s),
        (None, Some(s)) => (s, None),
        (None, None) => return None,
    };
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), ykey(&name));
    if let Some(s) = short {
        m.insert(ykey("shortName"), ykey(&s));
    }
    m.insert(ykey("for"), ykey(&qr(a.target)));
    Some(serde_yaml::Value::Mapping(m))
}

/// `REQ-TRS-SYSMLV2-043`: `alias <name> for <target>;` members of one merged package.
fn package_aliases(merged: &MergedPackage) -> Vec<serde_yaml::Value> {
    merged
        .body
        .iter()
        .filter_map(|(e, _, doc)| match e {
            sysml_v2_parser::PackageBodyElement::AliasDef(a) => with_doc(doc, || alias_entry(&a.value)),
            _ => None,
        })
        .collect()
}

/// A package's `import` members as `imports:` entries, so a bare reference to an
/// imported library name resolves exactly as it does for a native model.
fn package_imports(merged: &MergedPackage) -> Vec<serde_yaml::Value> {
    merged
        .body
        .iter()
        .filter_map(|(e, _, doc)| match e {
            sysml_v2_parser::PackageBodyElement::Import(i) => {
                with_doc(doc, || Some(ykey(&import_target_text(&i.value.target))))
            }
            _ => None,
        })
        .collect()
}

fn convert_merged(merged: &MergedPackage, qname: &str, out: &mut Vec<RawElement>) {
    // `REQ-TRS-SYSMLV2-086`: a package-level `#T` with no body of its own prefixes the member
    // that follows it in the same file (`#T part def B;`), which receives `{type: T}`.
    let mut prefix: Option<(String, String)> = None;
    for (elem, file_path, doc) in &merged.body {
        if let sysml_v2_parser::PackageBodyElement::MetadataKeywordUsage(k) = elem {
            if k.value.body.is_none() {
                prefix = Some((with_doc(doc, || qr(k.value.reference)), file_path.clone()));
                continue;
            }
        }
        let before = out.len();
        with_doc(doc, || convert_package_body_element(elem, qname, file_path, out));
        if let Some((type_ref, file)) = prefix.take() {
            if file == *file_path && out.len() > before {
                let entries = metadata_entries(type_ref, None, Vec::new(), Vec::new());
                out[before].frontmatter.metadata.get_or_insert_with(Vec::new).extend(entries);
            }
        }
    }
    for (name, child) in &merged.children {
        let child_qname = format!("{qname}::{name}");
        let file_path = child.declared_in.as_deref().unwrap_or(qname);
        let spec = Spec {
            aliases: nonempty_vec(package_aliases(child)),
            imports: nonempty_vec(package_imports(child)),
            ..Default::default()
        }
            .with_metadata(package_metadata(child));
        push_synth(out, &child_qname, file_path, ElementType::Package, name, spec.with_doc(package_doc(child)));
        convert_merged(child, &child_qname, out);
    }
}

/// `REQ-TRS-SYSMLV2-036`: a package's own `doc` members, joined in source order — lifted onto the
/// `Package` (or, for the file root, onto the anchor package's element; `REQ-TRS-SYSMLV2-098`).
fn package_doc(merged: &MergedPackage) -> String {
    merged
        .body
        .iter()
        .filter_map(|(e, _, d)| match e {
            sysml_v2_parser::PackageBodyElement::Annotating(a) => with_doc(d, || annotating_doc(a)),
            _ => None,
        })
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Dispatch one top-level (package-body) member. Only the kinds in
/// `REQ-TRS-SYSMLV2-007`'s fixed set are mapped; everything else is silently
/// invisible (parse-broad, map-narrow).
fn convert_package_body_element(
    elem: &sysml_v2_parser::PackageBodyElement,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    use sysml_v2_parser::PackageBodyElement as E;
    match elem {
        E::PartDef(node) => convert_part_def(&node.value, qname, file_path, out),
        E::PartUsage(node) => convert_part_usage(&node.value, qname, file_path, out),
        E::AttributeDef(node) => convert_attribute_def(&node.value, qname, file_path, out),
        E::AttributeUsage(node) => convert_attribute_usage(&node.value, qname, file_path, out),
        E::PortDef(node) => convert_port_def(&node.value, qname, file_path, out),
        E::PortUsage(node) => convert_port_usage(&node.value, qname, file_path, out),
        E::ConnectionDef(node) => convert_connection_def(&node.value, qname, file_path, out),
        E::ConnectionUsage(node) => convert_connection_usage(&node.value, qname, file_path, out),
        E::InterfaceDef(node) => convert_interface_def(&node.value, qname, file_path, out),
        E::InterfaceUsage(node) => convert_interface_usage(&node.value, qname, file_path, out),
        E::ItemDef(node) => convert_item_def(&node.value, qname, file_path, out),
        E::ItemUsage(node) => convert_item_usage(&node.value, qname, file_path, out),
        E::RequirementDef(node) => convert_requirement_def(&node.value, qname, file_path, out),
        E::RequirementUsage(node) => convert_requirement_usage(&node.value, qname, file_path, out),
        E::AllocationUsage(node) => convert_allocation_usage(&node.value, qname, file_path, out),
        // `REQ-TRS-SYSMLV2-029` (GH #142).
        E::AllocationDef(node) => convert_allocation_def(&node.value, qname, file_path, out),
        E::StateDef(node) => convert_state_def(&node.value, qname, file_path, out),
        E::StateUsage(node) => convert_state_usage(&node.value, qname, file_path, out),
        E::ActionDef(node) => convert_action_def(&node.value, qname, file_path, out),
        E::ActionUsage(node) => convert_action_usage(&node.value, qname, file_path, out),
        E::ViewDef(node) => convert_view_def(&node.value, qname, file_path, out),
        E::ViewUsage(node) => convert_view_usage(&node.value, qname, file_path, out),
        E::ViewpointDef(node) => convert_viewpoint_def(&node.value, qname, file_path, out),
        E::ViewpointUsage(node) => convert_viewpoint_usage(&node.value, qname, file_path, out),
        E::RenderingDef(node) => convert_rendering_def(&node.value, qname, file_path, out),
        E::RenderingUsage(node) => convert_rendering_usage(&node.value, qname, file_path, out),
        // `REQ-TRS-SYSMLV2-023`. `ConcernUsage` is reachable *only* from
        // `PackageBodyElement` in this parser version -- confirmed absent
        // from both `PartDefBodyElement` and `PartUsageBodyElement`, so
        // there is no matching arm to add in either of those two dispatch
        // functions below; a `concern`/`concern def` nested inside any
        // `part`/`part def` body is a genuine parse failure (`W541`), not a
        // silent per-kind skip.
        E::ConcernUsage(node) => convert_concern_usage(&node.value, qname, file_path, out),
        // `REQ-TRS-SYSMLV2-024`. A *named* flow becomes its own element
        // here; every `FlowUsage` (named or anonymous) found nested inside
        // a `part def`/`part` body is *also*, separately, lifted onto the
        // owning part's `flowConnections:` (`part_def_flow_entries`/
        // `part_usage_flow_entries`, called from `convert_part_def`/
        // `convert_part_usage`, not from this dispatch) — the same dual
        // pattern `Connection`/`REQ-TRS-SYSMLV2-010` already established.
        E::FlowDef(node) => convert_flow_def(&node.value, qname, file_path, out),
        E::FlowUsage(node) => convert_flow_usage(&node.value, qname, file_path, out),
        // `REQ-TRS-SYSMLV2-025`. Reachable from all three dispatch enums
        // this module cares about, same posture as Flow.
        E::EnumDef(node) => convert_enum_def(&node.value, qname, file_path, out),
        E::EnumerationUsage(node) => convert_enum_usage(&node.value, qname, file_path, out),
        // `REQ-TRS-SYSMLV2-026`/`-027`/`-028`. All six reachable here (and
        // from `convert_part_def_body_element`); `use case def`/`use case`
        // deliberately stay out of scope for this increment.
        E::CaseDef(node) => convert_case_def(&node.value, qname, file_path, out),
        E::CaseUsage(node) => convert_case_usage(&node.value, qname, file_path, out),
        E::AnalysisCaseDef(node) => convert_analysis_case_def(&node.value, qname, file_path, out),
        E::AnalysisCaseUsage(node) => convert_analysis_case_usage(&node.value, qname, file_path, out),
        E::VerificationCaseDef(node) => convert_verification_case_def(&node.value, qname, file_path, out),
        E::VerificationCaseUsage(node) => convert_verification_case_usage(&node.value, qname, file_path, out),
        // `REQ-TRS-SYSMLV2-033`/`-034`/`-035`.
        E::ConstraintDef(node) => convert_constraint_def(&node.value, qname, file_path, out),
        E::ConstraintUsage(node) => convert_constraint_usage(&node.value, qname, file_path, out),
        E::CalcDef(node) => convert_calc_def(&node.value, qname, file_path, out),
        E::MetadataDef(node) => convert_metadata_def(&node.value, qname, file_path, out),
        E::UseCaseDef(node) => convert_use_case_def(&node.value, qname, file_path, out),
        E::UseCaseUsage(node) => convert_use_case_usage(&node.value, qname, file_path, out),
        // `REQ-TRS-SYSMLV2-100`: a usage shape the parser cannot read as a `calc def`
        // (`calc estimate [1];`), already converted this way inside part bodies.
        E::CalcUsage(node) => convert_calc_usage(&node.value, qname, file_path, out),
        // `REQ-TRS-SYSMLV2-083`/`-084`.
        E::OccurrenceDef(node) => convert_occurrence_def(&node.value, qname, file_path, out),
        E::IndividualDef(node) => convert_individual_def(&node.value, qname, file_path, out),
        E::OccurrenceUsage(node) => convert_occurrence_usage(&node.value, qname, file_path, out),
        E::Dependency(node) => convert_dependency(&node.value, qname, file_path, out),
        _ => {} // outside REQ-TRS-SYSMLV2-007's fixed set
    }
}

// ---------------------------------------------------------------------------
// GH #203 -- wiring/behaviour members of a part body (`connect`, `bind`, `perform`, `exhibit`,
// `interface ... connect`), feature directions, `ref`, port conjugation, interface/connection
// `end` features, and counting of every body member that still has no mapping.
// ---------------------------------------------------------------------------

thread_local! {
    /// Body-level members dropped during one conversion, per source file, by kind. Drained into
    /// the per-file `W543` counts by [`ingest_subtree_detailed`].
    static BODY_UNMAPPED: std::cell::RefCell<BTreeMap<String, BTreeMap<&'static str, usize>>> =
        const { std::cell::RefCell::new(BTreeMap::new()) };
}

fn note_unmapped(file: &str, kind: &'static str) {
    BODY_UNMAPPED.with(|m| *m.borrow_mut().entry(file.to_string()).or_default().entry(kind).or_insert(0) += 1);
}

fn take_body_unmapped() -> BTreeMap<String, BTreeMap<&'static str, usize>> {
    BODY_UNMAPPED.with(|m| std::mem::take(&mut *m.borrow_mut()))
}

/// `in`/`out`/`inout` written on a usage (`in item x : Real;`).
fn usage_direction(prefix: &sysml_v2_parser::ast::OccurrenceUsagePrefix) -> Option<String> {
    prefix
        .basic()
        .and_then(|b| b.ref_prefix.direction.as_ref())
        .map(|d| direction_str(d.value).to_string())
}

/// `ref` written on a usage (`ref part helper : Eng;`).
fn usage_is_reference(prefix: &sysml_v2_parser::ast::OccurrenceUsagePrefix) -> Option<bool> {
    prefix.basic().and_then(|b| b.reference_span.as_ref()).map(|_| true)
}

/// `~Type` conjugated typing (`port inp : ~PP;`).
fn typing_is_conjugated(t: Option<&sysml_v2_parser::Node<sysml_v2_parser::ast::TypingRelationship>>) -> Option<bool> {
    t.is_some_and(|t| t.value.is_conjugated).then_some(true)
}

/// One wiring/behaviour member of a part body, borrowed from either body enum.
enum Wire<'a> {
    Connect(&'a sysml_v2_parser::ast::Connect),
    Bind(&'a sysml_v2_parser::ast::Bind),
    Perform(&'a sysml_v2_parser::ast::Perform),
    Exhibit(&'a sysml_v2_parser::ast::ExhibitState),
    Interface(&'a sysml_v2_parser::ast::InterfaceUsage),
}

fn part_def_wires(elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartDefBodyElement>]) -> Vec<Wire<'_>> {
    use sysml_v2_parser::PartDefBodyElement as E;
    elements
        .iter()
        .filter_map(|n| match &n.value {
            E::Connect(x) => Some(Wire::Connect(&x.value)),
            E::Bind(x) => Some(Wire::Bind(&x.value)),
            E::Perform(x) => Some(Wire::Perform(&x.value)),
            E::ExhibitState(x) => Some(Wire::Exhibit(&x.value)),
            E::InterfaceUsage(x) => Some(Wire::Interface(&x.value)),
            _ => None,
        })
        .collect()
}

fn part_usage_wires(elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartUsageBodyElement>]) -> Vec<Wire<'_>> {
    use sysml_v2_parser::PartUsageBodyElement as E;
    elements
        .iter()
        .filter_map(|n| match &n.value {
            E::Connect(x) => Some(Wire::Connect(&x.value)),
            E::Bind(x) => Some(Wire::Bind(&x.value)),
            E::Perform(x) => Some(Wire::Perform(&x.value)),
            E::InterfaceUsage(x) => Some(Wire::Interface(&x.value)),
            _ => None,
        })
        .collect()
}

#[derive(Default)]
struct Wiring {
    connections: Vec<serde_yaml::Value>,
    bindings: Vec<serde_yaml::Value>,
    performs: Vec<serde_yaml::Value>,
    exhibits: Vec<String>,
    /// `exhibit state name [: T]` usages that declare a state of their own: `(name, typedBy)`.
    exhibit_states: Vec<(String, Option<String>)>,
    truncations: Vec<String>,
}

impl Wiring {
    /// Fold into `spec` (appending to any `connections:` it already carries); returns the
    /// `W542` truncation messages and the states an `exhibit state name;` declares.
    fn apply(self, mut spec: Spec) -> (Spec, Vec<String>, Vec<(String, Option<String>)>) {
        let mut conns = spec.connections.take().unwrap_or_default();
        conns.extend(self.connections);
        spec.connections = nonempty_vec(conns);
        spec.binding_connections = nonempty_vec(self.bindings);
        spec.performs = nonempty_vec(self.performs);
        spec.exhibits_states = nonempty_vec(self.exhibits);
        (spec, self.truncations, self.exhibit_states)
    }
}

fn collect_wiring<'a, 'b>(
    owner: &str,
    file: &str,
    wires: Vec<Wire<'b>>,
    find_sibling: &impl Fn(&str) -> Option<PartUsageSibling<'a>>,
) -> Wiring {
    let mut w = Wiring::default();
    for wire in wires {
        match wire {
            Wire::Connect(c) => {
                let (Some(from), Some(to)) = (
                    connection_end_display(&c.from.value.expression.value),
                    connection_end_display(&c.to.value.expression.value),
                ) else {
                    note_unmapped(file, "connect");
                    continue;
                };
                match build_connection_value(owner, from, to, Vec::new(), None, find_sibling, &mut w.truncations) {
                    Some(v) => w.connections.push(v),
                    None => note_unmapped(file, "connect"),
                }
            }
            Wire::Bind(b) => {
                let (Some(l), Some(r)) = (connection_end_display(&b.left.value), connection_end_display(&b.right.value)) else {
                    note_unmapped(file, "bind");
                    continue;
                };
                let (lq, lt) = qualify_connection_end(owner, &l, find_sibling);
                let (rq, rt) = qualify_connection_end(owner, &r, find_sibling);
                w.truncations.extend(lt);
                w.truncations.extend(rt);
                let mut m = serde_yaml::Mapping::new();
                if let Some(n) = odn(b.binding_name).filter(|n| !n.is_empty()) {
                    m.insert(ykey("name"), ykey(&n));
                }
                if let Some(t) = oqr(b.binding_type) {
                    m.insert(ykey("typedBy"), ykey(&t));
                }
                m.insert(ykey("left"), ykey(&lq));
                m.insert(ykey("right"), ykey(&rq));
                w.bindings.push(serde_yaml::Value::Mapping(m));
            }
            Wire::Perform(p) => match perform_entry(p) {
                Some(v) => w.performs.push(v),
                None => note_unmapped(file, "perform"),
            },
            Wire::Exhibit(e) => {
                let typing = typing_first(e.typing.as_ref()).and_then(nonempty);
                match (odn(e.name).filter(|n| !n.is_empty()), e.state_reference) {
                    (Some(n), _) => {
                        w.exhibits.push(format!("{owner}::{n}"));
                        w.exhibit_states.push((n, typing));
                    }
                    (None, Some(r)) => w.exhibits.push(qr(r)),
                    (None, None) => match typing {
                        Some(t) => w.exhibits.push(t),
                        None => note_unmapped(file, "exhibit state"),
                    },
                }
                if e.body.braced_elements().is_some_and(|b| !b.is_empty()) {
                    note_unmapped(file, "exhibit state body");
                }
            }
            Wire::Interface(i) => {
                use sysml_v2_parser::ast::InterfaceUsage as I;
                let (name, ty, part) = match i {
                    I::TypedConnect { name, interface_type, part, .. } => (odn(*name), oqr(*interface_type), part),
                    I::Connection { part, .. } => (None, None, part),
                    I::Declaration { .. } => continue,
                };
                match interface_connection_entry(owner, name.filter(|n| !n.is_empty()), ty, &part.value, find_sibling, &mut w.truncations) {
                    Some(v) => w.connections.push(v),
                    None => note_unmapped(file, "interface connect"),
                }
            }
        }
    }
    w
}

/// `perform action drive [: T];` / `perform Ref;` as a `performs:` entry -- the string shorthand
/// when only a type is written, the map form otherwise.
fn perform_entry(p: &sysml_v2_parser::ast::Perform) -> Option<serde_yaml::Value> {
    use sysml_v2_parser::ast::PerformActionTarget as T;
    let (name, typed_by, multiplicity, redefines) = match &p.target {
        T::Action(decl) => {
            let d = &decl.value;
            (
                ident_name(&d.identification),
                typing_first(d.typing.as_ref()).and_then(nonempty),
                d.multiplicity.as_ref().map(|m| multiplicity_text(&m.value)),
                d.redefines.as_ref().map(|r| subsetting_targets(&r.value)).unwrap_or_default(),
            )
        }
        T::Reference { action, redefines } => (
            None,
            Some(qr(*action)),
            None,
            redefines.as_ref().map(|r| subsetting_targets(&r.value)).unwrap_or_default(),
        ),
    };
    if name.is_none() && typed_by.is_none() && redefines.is_empty() {
        return None;
    }
    if name.is_none() && multiplicity.is_none() && redefines.is_empty() {
        return typed_by.map(serde_yaml::Value::String);
    }
    let mut m = serde_yaml::Mapping::new();
    if let Some(n) = name {
        m.insert(ykey("name"), ykey(&n));
    }
    if let Some(t) = typed_by {
        m.insert(ykey("typedBy"), ykey(&t));
    }
    if let Some(x) = multiplicity {
        m.insert(ykey("multiplicity"), ykey(&x));
    }
    if !redefines.is_empty() {
        m.insert(ykey("redefines"), serde_yaml::Value::Sequence(redefines.iter().map(|r| ykey(r)).collect()));
    }
    Some(serde_yaml::Value::Mapping(m))
}

/// One `InterfaceEnd` of an `interface ... connect` clause: `(end name, endpoint chain)`.
fn interface_end(e: &sysml_v2_parser::ast::InterfaceEnd) -> (Option<String>, String) {
    use sysml_v2_parser::ast::InterfaceEndTarget as T;
    match &e.target {
        T::Direct(r) => (None, qr(*r)),
        T::Named { name, target, .. } => (Some(dn(*name)).filter(|n| !n.is_empty()), qr(*target)),
    }
}

/// `interface i : PI connect a ::> e1.p to b ::> e2.q;` as a `connections:` entry on the owner:
/// `typedBy` the interface def, the def's end names kept as `ends: [{end, binds}]` when written.
fn interface_connection_entry<'a>(
    owner: &str,
    name: Option<String>,
    typed_by: Option<String>,
    part: &sysml_v2_parser::ast::InterfacePart,
    find_sibling: &impl Fn(&str) -> Option<PartUsageSibling<'a>>,
    truncations: &mut Vec<String>,
) -> Option<serde_yaml::Value> {
    use sysml_v2_parser::ast::InterfacePart as P;
    let ends: Vec<(Option<String>, String)> = match part {
        P::Binary { from, to, .. } => vec![interface_end(&from.value), interface_end(&to.value)],
        P::Nary { ends, .. } => ends.iter().map(|m| interface_end(&m.end.value)).collect(),
    };
    if ends.len() < 2 {
        return None;
    }
    let mut value = if ends.len() == 2 && ends.iter().all(|(n, _)| n.is_none()) {
        build_connection_value(owner, ends[0].1.clone(), ends[1].1.clone(), Vec::new(), typed_by.clone(), find_sibling, truncations)?
    } else {
        let entries: Vec<serde_yaml::Value> = ends
            .iter()
            .enumerate()
            .map(|(i, (n, chain))| {
                let (q, t) = qualify_connection_end(owner, chain, find_sibling);
                truncations.extend(t);
                let mut em = serde_yaml::Mapping::new();
                em.insert(ykey("end"), ykey(&n.clone().unwrap_or_else(|| format!("end{}", i + 1))));
                em.insert(ykey("binds"), ykey(&q));
                serde_yaml::Value::Mapping(em)
            })
            .collect();
        let mut m = serde_yaml::Mapping::new();
        if let Some(tb) = &typed_by {
            m.insert(ykey("typedBy"), ykey(tb));
        }
        m.insert(ykey("ends"), serde_yaml::Value::Sequence(entries));
        serde_yaml::Value::Mapping(m)
    };
    if let (Some(n), serde_yaml::Value::Mapping(m)) = (name, &mut value) {
        m.insert(ykey("name"), ykey(&n));
    }
    Some(value)
}

/// `end a : PP;` of an `interface def`/`connection def` as an inline `ends:` entry.
fn end_decl_entry(e: &sysml_v2_parser::ast::EndDecl) -> serde_yaml::Value {
    let mut m = serde_yaml::Mapping::new();
    if let sysml_v2_parser::ast::EndIdentity::Declaration(n) = &e.identity {
        let n = dn(*n);
        if !n.is_empty() {
            m.insert(ykey("name"), ykey(&n));
        }
    }
    if let Some(t) = typing_first(e.typing.as_ref()).and_then(nonempty) {
        m.insert(ykey("typedBy"), ykey(&t));
    }
    if typing_is_conjugated(e.typing.as_ref()).is_some() {
        m.insert(ykey("isConjugated"), serde_yaml::Value::Bool(true));
    }
    if let Some(x) = &e.multiplicity {
        m.insert(ykey("multiplicity"), ykey(&multiplicity_text(&x.value)));
    }
    m.insert(ykey("isEnd"), serde_yaml::Value::Bool(true));
    serde_yaml::Value::Mapping(m)
}

/// Kind label of a `part def` body member that has no mapping (`None` for mapped or
/// pure-plumbing members: comments, metadata, imports, parse-error nodes already reported as `W541`).
fn part_def_unmapped_kind(e: &sysml_v2_parser::PartDefBodyElement) -> Option<&'static str> {
    use sysml_v2_parser::PartDefBodyElement as E;
    Some(match e {
        E::Package(_) | E::LibraryPackage(_) => "nested package",
        E::DefaultReferenceUsage(_) | E::Ref(_) => "reference usage",
        E::Allocate(_) => "allocate",
        E::UnsupportedMember(_) => "unsupported member",
        E::ConstraintDef(_) => "constraint def",
        E::AssertConstraint(_) | E::RequireConstraint(_) => "constraint member",
        E::MetadataDef(_) => "metadata def",
        E::CalcDef(_) => "calc def",
        E::UseCaseDef(_) => "use case def",
        E::ViewRendering(_) | E::VerifyRequirement(_) => "view/verify member",
        E::KermlClassifier(_) => "KerML declaration",
        E::AliasDef(_) => "alias",
        _ => return None,
    })
}

fn part_usage_unmapped_kind(e: &sysml_v2_parser::PartUsageBodyElement) -> Option<&'static str> {
    use sysml_v2_parser::PartUsageBodyElement as E;
    Some(match e {
        E::EndDecl(_) => "end feature",
        E::InOutDecl(_) => "directed parameter",
        E::DefaultReferenceUsage(_) | E::Ref(_) => "reference usage",
        E::Allocate(_) => "allocate",
        E::ExtendedUsage(_) => "extended usage",
        E::AssertConstraint(_) => "constraint member",
        E::IncludeUseCase(_) => "include",
        E::AliasDef(_) => "alias",
        E::KermlClassifier(_) => "KerML declaration",
        _ => return None,
    })
}

fn convert_part_def(
    part: &sysml_v2_parser::PartDef,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    let Some(name) = ident_name(&part.identification) else {
        return; // anonymous part def: no identity to qname against
    };
    let part_qname = format!("{qname}::{name}");
    let elements = match &part.body {
        sysml_v2_parser::PartDefBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::PartDefBody::Semicolon { .. } => &[],
    };
    let satisfies = nonempty_vec(
        elements
            .iter()
            .filter_map(|n| match &n.value {
                sysml_v2_parser::PartDefBodyElement::Satisfy(s) => satisfy_target(&s.value),
                _ => None,
            })
            .collect(),
    );
    let (connections, truncations) = part_def_connection_entries(&part_qname, elements);
    let (flow_connections, flow_truncations) = part_def_flow_entries(&part_qname, elements);
    let spec = Spec {
        supertype: part.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        is_variation: is_variation_prefix(&part.definition_prefix),
        satisfies,
        applies_when: part_def_syscribe_feature_id(elements),
        // `REQ-TRS-SYSMLV2-092`: structural successions between owned usages.
        succession_connections: nonempty_vec(part_def_successions(elements)),
        ..Default::default()
    }
    .with_syscribe_meta(part_def_syscribe_meta(elements))
    .with_doc(part_def_doc(elements))
    .with_metadata(body_metadata(elements))
    .with_connections(connections)
    .with_flow_connections(flow_connections);
    let find_sibling = |head: &str| find_part_usage_in_part_def_body(elements, head);
    let (spec, wire_truncations, exhibit_states) =
        collect_wiring(&part_qname, file_path, part_def_wires(elements), &find_sibling).apply(spec);
    push_synth(out, &part_qname, file_path, ElementType::PartDef, &name, spec);
    push_connection_truncation_findings(out, file_path, truncations);
    push_connection_truncation_findings(out, file_path, flow_truncations);
    push_connection_truncation_findings(out, file_path, wire_truncations);
    convert_body_with_prefixes(elements, out, |e, out| convert_part_def_body_element(e, &part_qname, file_path, out));
    push_exhibit_states(out, &part_qname, file_path, exhibit_states);
}

fn convert_part_usage(
    part: &sysml_v2_parser::PartUsage,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    if part.name.s().is_empty() {
        return; // anonymous usage: no identity to qname against
    }
    let part_qname = format!("{qname}::{}", part.name.s());
    let elements = match &part.body {
        sysml_v2_parser::PartUsageBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::PartUsageBody::Semicolon { .. } => &[],
    };
    let satisfies = nonempty_vec(
        elements
            .iter()
            .filter_map(|n| match &n.value {
                sysml_v2_parser::PartUsageBodyElement::Satisfy(s) => satisfy_target(&s.value),
                _ => None,
            })
            .collect(),
    );
    let (connections, truncations) = part_usage_connection_entries(&part_qname, elements);
    let (flow_connections, flow_truncations) = part_usage_flow_entries(&part_qname, elements);
    let spec = Spec {
        typed_by: typing_first(part.typing.as_ref()).filter(|t| !t.is_empty()),
        is_variation: usage_variation(&part.prefix).then_some(true),
        is_reference: usage_is_reference(&part.prefix),
        direction: usage_direction(&part.prefix),
        satisfies,
        applies_when: part_usage_syscribe_feature_id(elements),
        // `REQ-TRS-SYSMLV2-092`: structural successions between owned usages.
        succession_connections: nonempty_vec(part_usage_successions(elements)),
        ..Default::default()
    }
    .with_usage_relations(
        part.multiplicity.as_ref().map(|m| &m.value),
        part.subsets.as_ref().map(|(r, _)| &r.value),
        part.redefines.as_ref().map(|r| &r.value),
    )
    .with_syscribe_meta(part_usage_syscribe_meta(elements))
    .with_doc(part_usage_doc(elements))
    .with_metadata(with_prefix_keywords(&part.prefix, body_metadata(elements)))
    .with_connections(connections)
    .with_flow_connections(flow_connections);
    let find_sibling = |head: &str| find_part_usage_in_part_usage_body(elements, head);
    let (spec, wire_truncations, _) =
        collect_wiring(&part_qname, file_path, part_usage_wires(elements), &find_sibling).apply(spec);
    push_synth(out, &part_qname, file_path, ElementType::Part, &part.name.s(), spec);
    push_connection_truncation_findings(out, file_path, truncations);
    push_connection_truncation_findings(out, file_path, flow_truncations);
    push_connection_truncation_findings(out, file_path, wire_truncations);
    convert_body_with_prefixes(elements, out, |e, out| convert_part_usage_body_element(e, &part_qname, file_path, out));
}

/// `None` for an empty `Vec` — several `Spec` fields are `Option<Vec<T>>`
/// and an absent relationship/entry list should serialize as `None`, not
/// `Some(vec![])`. Generic since `REQ-TRS-SYSMLV2-010` reuses this for
/// `Vec<serde_yaml::Value>` `connections:` entries alongside the existing
/// `Vec<String>` uses (`satisfies`/`verifies`).
fn nonempty_vec<T>(v: Vec<T>) -> Option<Vec<T>> {
    (!v.is_empty()).then_some(v)
}

/// Dispatch one member of a `part def` body. Recurses into nested
/// `PartDef`/`PartUsage` so a realistic containment tree (a part containing
/// attributes/ports/nested parts) is fully walked, not just one level deep.
/// Note: the parser names this enum's plain-connection-usage variant
/// `Connection`, not `ConnectionUsage` (that name is `PackageBodyElement`'s).
fn convert_part_def_body_element(
    elem: &sysml_v2_parser::PartDefBodyElement,
    part_qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    use sysml_v2_parser::PartDefBodyElement as E;
    match elem {
        E::PartDef(node) => convert_part_def(&node.value, part_qname, file_path, out),
        E::PartUsage(node) => convert_part_usage(&node.value, part_qname, file_path, out),
        E::AttributeDef(node) => convert_attribute_def(&node.value, part_qname, file_path, out),
        E::AttributeUsage(node) => convert_attribute_usage(&node.value, part_qname, file_path, out),
        E::PortDef(node) => convert_port_def(&node.value, part_qname, file_path, out),
        E::PortUsage(node) => convert_port_usage(&node.value, part_qname, file_path, out),
        E::ConnectionDef(node) => convert_connection_def(&node.value, part_qname, file_path, out),
        E::Connection(node) => convert_connection_usage(&node.value, part_qname, file_path, out),
        E::InterfaceDef(node) => convert_interface_def(&node.value, part_qname, file_path, out),
        E::InterfaceUsage(node) => convert_interface_usage(&node.value, part_qname, file_path, out),
        E::ItemDef(node) => convert_item_def(&node.value, part_qname, file_path, out),
        E::ItemUsage(node) => convert_item_usage(&node.value, part_qname, file_path, out),
        E::RequirementDef(node) => convert_requirement_def(&node.value, part_qname, file_path, out),
        E::RequirementUsage(node) => convert_requirement_usage(&node.value, part_qname, file_path, out),
        E::AllocationUsage(node) => convert_allocation_usage(&node.value, part_qname, file_path, out),
        // `REQ-TRS-SYSMLV2-029` (GH #142).
        E::AllocationDef(node) => convert_allocation_def(&node.value, part_qname, file_path, out),
        E::VariantUsage(node) => convert_variant_usage(&node.value, part_qname, file_path, out),
        E::StateDef(node) => convert_state_def(&node.value, part_qname, file_path, out),
        E::StateUsage(node) => convert_state_usage(&node.value, part_qname, file_path, out),
        E::ActionDef(node) => convert_action_def(&node.value, part_qname, file_path, out),
        E::ActionUsage(node) => convert_action_usage(&node.value, part_qname, file_path, out),
        E::ViewDef(node) => convert_view_def(&node.value, part_qname, file_path, out),
        E::ViewUsage(node) => convert_view_usage(&node.value, part_qname, file_path, out),
        E::ViewpointDef(node) => convert_viewpoint_def(&node.value, part_qname, file_path, out),
        E::ViewpointUsage(node) => convert_viewpoint_usage(&node.value, part_qname, file_path, out),
        E::RenderingDef(node) => convert_rendering_def(&node.value, part_qname, file_path, out),
        E::RenderingUsage(node) => convert_rendering_usage(&node.value, part_qname, file_path, out),
        // `REQ-TRS-SYSMLV2-024`. See the identical arm/comment in
        // `convert_package_body_element`.
        E::FlowDef(node) => convert_flow_def(&node.value, part_qname, file_path, out),
        E::FlowUsage(node) => convert_flow_usage(&node.value, part_qname, file_path, out),
        // `REQ-TRS-SYSMLV2-025`. See the identical arm/comment in
        // `convert_package_body_element`.
        E::EnumDef(node) => convert_enum_def(&node.value, part_qname, file_path, out),
        E::EnumerationUsage(node) => convert_enum_usage(&node.value, part_qname, file_path, out),
        // `REQ-TRS-SYSMLV2-026`/`-027`/`-028`. See the identical arm/comment
        // in `convert_package_body_element`.
        E::CaseDef(node) => convert_case_def(&node.value, part_qname, file_path, out),
        E::CaseUsage(node) => convert_case_usage(&node.value, part_qname, file_path, out),
        E::AnalysisCaseDef(node) => convert_analysis_case_def(&node.value, part_qname, file_path, out),
        E::AnalysisCaseUsage(node) => convert_analysis_case_usage(&node.value, part_qname, file_path, out),
        E::VerificationCaseDef(node) => convert_verification_case_def(&node.value, part_qname, file_path, out),
        E::VerificationCaseUsage(node) => convert_verification_case_usage(&node.value, part_qname, file_path, out),
        // `REQ-TRS-SYSMLV2-033`/`-034`/`-035`: usage variants reachable here.
        E::ConstraintUsage(node) => convert_constraint_usage(&node.value, part_qname, file_path, out),
        E::CalcUsage(node) => convert_calc_usage(&node.value, part_qname, file_path, out),
        E::UseCaseUsage(node) => convert_use_case_usage(&node.value, part_qname, file_path, out),
        // `REQ-TRS-SYSMLV2-083`/`-084`.
        E::OccurrenceDef(node) => convert_occurrence_def(&node.value, part_qname, file_path, out),
        E::OccurrenceUsage(node) => convert_occurrence_usage(&node.value, part_qname, file_path, out),
        E::Dependency(node) => convert_dependency(&node.value, part_qname, file_path, out),
        _ => {
            // GH #203: never silent -- counted into the file's `W543`.
            if let Some(k) = part_def_unmapped_kind(elem) {
                note_unmapped(file_path, k);
            }
        }
    }
}

/// Dispatch one member of a `part` usage body. See
/// [`convert_part_def_body_element`]. Note: unlike `PartDefBodyElement`, this
/// enum has no nested-`PartDef` or `AllocationUsage` variant at all — a `part
/// def`/named `allocation` cannot be declared directly inside a `part` usage
/// body per this grammar.
fn convert_part_usage_body_element(
    elem: &sysml_v2_parser::PartUsageBodyElement,
    part_qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    use sysml_v2_parser::PartUsageBodyElement as E;
    match elem {
        E::PartUsage(node) => convert_part_usage(&node.value, part_qname, file_path, out),
        E::AttributeUsage(node) => convert_attribute_usage(&node.value, part_qname, file_path, out),
        E::PortDef(node) => convert_port_def(&node.value, part_qname, file_path, out),
        E::PortUsage(node) => convert_port_usage(&node.value, part_qname, file_path, out),
        E::ConnectionDef(node) => convert_connection_def(&node.value, part_qname, file_path, out),
        E::Connection(node) => convert_connection_usage(&node.value, part_qname, file_path, out),
        E::InterfaceUsage(node) => convert_interface_usage(&node.value, part_qname, file_path, out),
        E::ItemDef(node) => convert_item_def(&node.value, part_qname, file_path, out),
        E::ItemUsage(node) => convert_item_usage(&node.value, part_qname, file_path, out),
        E::RequirementDef(node) => convert_requirement_def(&node.value, part_qname, file_path, out),
        E::RequirementUsage(node) => convert_requirement_usage(&node.value, part_qname, file_path, out),
        E::VariantUsage(node) => convert_variant_usage(&node.value, part_qname, file_path, out),
        E::StateDef(node) => convert_state_def(&node.value, part_qname, file_path, out),
        E::StateUsage(node) => convert_state_usage(&node.value, part_qname, file_path, out),
        // No `ActionDef` variant exists in this enum at all -- an `action
        // def` cannot be declared directly inside a `part` usage body per
        // this grammar (mirrors this same enum's pre-existing absence of
        // `PartDef`/`AllocationUsage`, noted in this function's own doc
        // comment above). Unchanged from before this feature: stays invisible.
        E::ActionUsage(node) => convert_action_usage(&node.value, part_qname, file_path, out),
        // `REQ-TRS-SYSMLV2-080`: the 0.57 grammar reaches the view/viewpoint/rendering family (and the
        // other already-mapped kinds below) inside a `part` usage body; 0.54 failed the whole file.
        E::ViewDef(node) => convert_view_def(&node.value, part_qname, file_path, out),
        E::ViewUsage(node) => convert_view_usage(&node.value, part_qname, file_path, out),
        E::ViewpointDef(node) => convert_viewpoint_def(&node.value, part_qname, file_path, out),
        E::ViewpointUsage(node) => convert_viewpoint_usage(&node.value, part_qname, file_path, out),
        E::RenderingDef(node) => convert_rendering_def(&node.value, part_qname, file_path, out),
        E::RenderingUsage(node) => convert_rendering_usage(&node.value, part_qname, file_path, out),
        E::ConstraintDef(node) => convert_constraint_def(&node.value, part_qname, file_path, out),
        E::CalcDef(node) => convert_calc_def(&node.value, part_qname, file_path, out),
        E::CalcUsage(node) => convert_calc_usage(&node.value, part_qname, file_path, out),
        E::MetadataDef(node) => convert_metadata_def(&node.value, part_qname, file_path, out),
        E::UseCaseUsage(node) => convert_use_case_usage(&node.value, part_qname, file_path, out),
        E::VerificationCaseUsage(node) => convert_verification_case_usage(&node.value, part_qname, file_path, out),
        // `REQ-TRS-SYSMLV2-083`.
        E::OccurrenceDef(node) => convert_occurrence_def(&node.value, part_qname, file_path, out),
        E::OccurrenceUsage(node) => convert_occurrence_usage(&node.value, part_qname, file_path, out),
        // `REQ-TRS-SYSMLV2-024`. Unlike the view/viewpoint/rendering family
        // above, both `FlowDef`/`FlowUsage` variants *do* exist in this enum
        // — see the identical arm/comment in `convert_package_body_element`.
        E::FlowDef(node) => convert_flow_def(&node.value, part_qname, file_path, out),
        E::FlowUsage(node) => convert_flow_usage(&node.value, part_qname, file_path, out),
        // `REQ-TRS-SYSMLV2-025`. Both `EnumDef`/`EnumerationUsage` variants
        // exist in this enum too — see the identical arm/comment in
        // `convert_package_body_element`.
        E::EnumDef(node) => convert_enum_def(&node.value, part_qname, file_path, out),
        E::EnumerationUsage(node) => convert_enum_usage(&node.value, part_qname, file_path, out),
        // `REQ-TRS-SYSMLV2-026`/`-027`/`-028`. Unlike `PackageBodyElement`/
        // `PartDefBodyElement`, this enum carries only `AnalysisCaseDef`/
        // `AnalysisCaseUsage` -- `CaseDef`/`CaseUsage`/`VerificationCaseDef`/
        // `VerificationCaseUsage` have no variant here at all (confirmed
        // against the AST, not a choice): a `case`/`verification` declared
        // directly inside a `part` usage body fails to parse outright,
        // gracefully degrading to `W541`, the same posture Concern's
        // single-enum gap used.
        E::AnalysisCaseDef(node) => convert_analysis_case_def(&node.value, part_qname, file_path, out),
        E::AnalysisCaseUsage(node) => convert_analysis_case_usage(&node.value, part_qname, file_path, out),
        // `REQ-TRS-SYSMLV2-033`.
        E::ConstraintUsage(node) => convert_constraint_usage(&node.value, part_qname, file_path, out),
        _ => {
            if let Some(k) = part_usage_unmapped_kind(elem) {
                note_unmapped(file_path, k);
            }
        }
    }
}

/// `exhibit state name [: T];` declares a state of its own: synthesize it (unless the body
/// already declares a sibling of that name) so the owner's `exhibitsStates:` entry resolves.
fn push_exhibit_states(out: &mut Vec<RawElement>, owner: &str, file_path: &str, states: Vec<(String, Option<String>)>) {
    for (name, typed_by) in states {
        let qname = format!("{owner}::{name}");
        if out.iter().any(|e| e.qualified_name == qname) {
            continue;
        }
        let spec = Spec { typed_by, ..Default::default() };
        push_synth(out, &qname, file_path, ElementType::State, &name, spec);
    }
}

fn is_variation_prefix(prefix: &Option<sysml_v2_parser::Node<sysml_v2_parser::ast::DefinitionPrefix>>) -> Option<bool> {
    matches!(prefix.as_ref().map(|p| &p.value), Some(sysml_v2_parser::ast::DefinitionPrefix::Variation)).then_some(true)
}

/// `variation` on a usage's `OccurrenceUsagePrefix` (the `RefPrefix` variance slot).
fn usage_variation(prefix: &sysml_v2_parser::ast::OccurrenceUsagePrefix) -> bool {
    prefix.basic().and_then(|b| b.ref_prefix.variance.as_ref()).is_some_and(|v| {
        matches!(v.value, sysml_v2_parser::ast::DefinitionPrefix::Variation)
    })
}

/// `abstract` on a usage's prefix.
fn usage_abstract(prefix: &sysml_v2_parser::ast::OccurrenceUsagePrefix) -> bool {
    prefix.basic().and_then(|b| b.ref_prefix.variance.as_ref()).is_some_and(|v| {
        matches!(v.value, sysml_v2_parser::ast::DefinitionPrefix::Abstract)
    })
}

/// `abstract` on a definition prefix.
fn def_abstract(prefix: &Option<sysml_v2_parser::Node<sysml_v2_parser::ast::DefinitionPrefix>>) -> bool {
    matches!(prefix.as_ref().map(|p| &p.value), Some(sysml_v2_parser::ast::DefinitionPrefix::Abstract))
}

/// `None` for an empty string — several usage structs carry `type_name: String`
/// (not `Option<String>`) that's simply empty when no `:`/`typed by` clause was
/// written.
fn nonempty(s: String) -> Option<String> {
    (!s.is_empty()).then_some(s)
}

/// `REQ-TRS-SYSMLV2-055`: a *literal* attribute value as a YAML scalar, with the unit of a
/// literal-with-unit (`12.5 [kg]`). Anything that is not a plain literal (a reference,
/// operator expression, ...) is not mapped, so `value:` always means a literal.
fn literal_value(e: &sysml_v2_parser::Expression) -> Option<(serde_yaml::Value, Option<String>)> {
    use sysml_v2_parser::Expression as E;
    let scalar = |e: &E| -> Option<serde_yaml::Value> {
        Some(match e {
            E::LiteralInteger(i) => serde_yaml::Value::Number((*i).into()),
            E::LiteralReal(r) => serde_yaml::Value::Number(
                with_cur(|d| d.real_literal(*r).and_then(|t| t.parse::<f64>().ok()))
                    .flatten()
                    .map(serde_yaml::Number::from)?,
            ),
            E::LiteralString(s) => serde_yaml::Value::String(
                with_cur(|d| d.decoded_string_literal(*s).map(|c| c.into_owned())).flatten().unwrap_or_default(),
            ),
            E::LiteralBoolean(b) => serde_yaml::Value::Bool(*b),
            _ => return None,
        })
    };
    match e {
        // 0.54's `LiteralWithUnit` is a `Bracket` over the literal in 0.55+: `12.5 [kg]`.
        E::Bracket { base, operands, .. } => {
            let [only] = operands.value.elements.as_slice() else { return None };
            let inner = &only.expression.value;
            let u = match inner {
                E::FeatureRef(s) => qr(*s),
                E::FeatureChainRef(c) => qr_segments(*c).join("::"),
                // `REQ-TRS-SYSMLV2-065`: a spaced compound (`N * m`) lexes as an operator
                // expression; keep its whitespace-free text rather than dropping the unit.
                other => compound_unit_text(other)?,
            };
            Some((scalar(&base.value)?, Some(u)))
        }
        other => Some((scalar(other)?, None)),
    }
}

/// `REQ-TRS-SYSMLV2-065`: the whitespace-free text of a compound unit expression (names joined by
/// `*`, `/`, `^` and integer exponents), or `None` for any other shape.
fn compound_unit_text(e: &sysml_v2_parser::Expression) -> Option<String> {
    let text = render_expression(e).replace(' ', "");
    let ok = text.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && text.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '*' | '/' | '^' | '-' | '.'));
    ok.then_some(text)
}

fn convert_attribute_def(
    a: &sysml_v2_parser::AttributeDef,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    if a.name.s().is_empty() {
        return;
    }
    let elem_qname = format!("{qname}::{}", a.name.s());
    let elements = match &a.body {
        sysml_v2_parser::AttributeBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::AttributeBody::Semicolon { .. } => &[],
    };
    // AttributeDef's `:>` specialization target is (inconsistently, upstream)
    // named `typing` rather than `specializes` like the other Def structs, but
    // it's the same semantic — a Def's supertype, not a Usage's typed-by.
    let spec = Spec {
        supertype: a.typing.as_ref().map(|t| refs_display(&t.value.target)),
        ..Default::default()
    }
    .with_doc(attribute_body_doc(elements))
    .with_metadata(body_metadata(elements));
    push_synth(out, &elem_qname, file_path, ElementType::AttributeDef, &a.name.s(), spec);
}

fn convert_attribute_usage(
    a: &sysml_v2_parser::AttributeUsage,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    if a.name.s().is_empty() {
        return;
    }
    let elem_qname = format!("{qname}::{}", a.name.s());
    let elements = match &a.body {
        sysml_v2_parser::AttributeBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::AttributeBody::Semicolon { .. } => &[],
    };
    let (value, mut unit) = a
        .value
        .as_ref()
        .and_then(|v| literal_value(&v.value.expression.value))
        .unzip();
    let mut unit = unit.take().flatten();
    // `REQ-TRS-SYSMLV2-065`: in `= 4 [N * m]` the 0.54 parser reads a *spaced* compound as a
    // trailing multiplicity (`[N*m]`, unspaced, is one unit token). A bracket after a literal
    // value is a unit, never a multiplicity (that precedes `=`), so recover it from the bounds.
    if let (Some(v), Some(m), None) = (a.value.as_ref(), a.multiplicity.as_ref(), unit.as_ref()) {
        if value.is_some() && m.span.offset >= v.span.offset {
            unit = m.value.lower.as_ref().and_then(|l| compound_unit_text(&l.value));
        }
    }
    let spec = Spec {
        typed_by: a.typing.as_ref().map(|t| refs_display(&t.value.target)),
        value,
        unit,
        ..Default::default()
    }
    .with_usage_relations(None, a.subsets.as_ref().map(|r| &r.value), a.redefines.as_ref().map(|r| &r.value))
    .with_doc(attribute_body_doc(elements))
    .with_metadata(body_metadata(elements));
    push_synth(out, &elem_qname, file_path, ElementType::Attribute, &a.name.s(), spec);
}

fn convert_port_def(
    p: &sysml_v2_parser::PortDef,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    let Some(name) = ident_name(&p.identification) else {
        return;
    };
    let elem_qname = format!("{qname}::{name}");
    let elements = match &p.body {
        sysml_v2_parser::PortDefBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::PortDefBody::Semicolon { .. } => &[],
    };
    // REQ-TRS-SYSMLV2-014: PortDefBodyElement has no MetadataAnnotation
    // variant, so @Syscribe* fields reach a port def only via a doc-comment
    // directive, extracted from the already-lifted doc text.
    let (doc, meta) = extract_syscribe_doc_directives(&port_def_doc(elements));
    let spec = Spec {
        supertype: p.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        ..Default::default()
    }
    .with_doc(doc)
    .with_syscribe_meta(meta)
    .with_metadata(body_metadata(elements));
    push_synth(out, &elem_qname, file_path, ElementType::PortDef, &name, spec);
    convert_body_with_prefixes(elements, out, |e, out| convert_port_def_body_element(e, &elem_qname, file_path, out));
}

/// Dispatch a member of a `port def` body — nested attributes/items only
/// (this enum has no nested port/interface variant).
fn convert_port_def_body_element(
    elem: &sysml_v2_parser::PortDefBodyElement,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    use sysml_v2_parser::PortDefBodyElement as E;
    match elem {
        E::AttributeDef(node) => convert_attribute_def(&node.value, qname, file_path, out),
        E::AttributeUsage(node) => convert_attribute_usage(&node.value, qname, file_path, out),
        E::ItemDef(node) => convert_item_def(&node.value, qname, file_path, out),
        E::ItemUsage(node) => convert_item_usage(&node.value, qname, file_path, out),
        E::PortUsage(node) => convert_port_usage(&node.value, qname, file_path, out),
        E::PartUsage(node) => convert_part_usage(&node.value, qname, file_path, out),
        E::EnumerationUsage(node) => convert_enum_usage(&node.value, qname, file_path, out),
        E::InOutDecl(_) => note_unmapped(file_path, "directed parameter"),
        E::RefDecl(_) => note_unmapped(file_path, "reference usage"),
        E::VariantUsage(_) => note_unmapped(file_path, "variant"),
        E::AliasDef(_) => note_unmapped(file_path, "alias"),
        E::Unsupported(_) => note_unmapped(file_path, "unsupported member"),
        _ => {}
    }
}

/// A port usage's `:` type text.
fn p_type(p: &sysml_v2_parser::PortUsage) -> Option<String> {
    typing_first(p.typing.as_ref())
}

fn convert_port_usage(
    p: &sysml_v2_parser::PortUsage,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    if p.name.s().is_empty() {
        return;
    }
    let elem_qname = format!("{qname}::{}", p.name.s());
    // Unlike every other Usage's body in this module, `p.body` was never
    // read here at all before `REQ-TRS-SYSMLV2-009` — its nested
    // `AttributeUsage`/`ItemUsage` members stay unmapped exactly as before
    // (out of scope for this requirement, which is doc-lifting only); only
    // the `doc` extraction is new.
    let elements = match &p.body {
        sysml_v2_parser::PortBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::PortBody::Semicolon { .. } => &[],
    };
    let spec = Spec {
        typed_by: p_type(p),
        is_conjugated: typing_is_conjugated(p.typing.as_ref()),
        is_reference: usage_is_reference(&p.prefix),
        direction: usage_direction(&p.prefix),
        ..Default::default()
    }
    .with_usage_relations(
        p.multiplicity.as_ref().map(|m| &m.value),
        p.subsets.as_ref().map(|(r, _)| &r.value),
        p.redefines.as_ref().map(|r| &r.value),
    )
    .with_doc(port_usage_doc(elements))
    .with_metadata(with_prefix_keywords(&p.prefix, body_metadata(elements)));
    push_synth(out, &elem_qname, file_path, ElementType::Port, &p.name.s(), spec);
    // GH #203: a port usage's nested features were never walked.
    for n in elements {
        use sysml_v2_parser::PortBodyElement as E;
        match &n.value {
            E::PortUsage(x) => convert_port_usage(&x.value, &elem_qname, file_path, out),
            E::AttributeUsage(x) => convert_attribute_usage(&x.value, &elem_qname, file_path, out),
            E::ItemUsage(x) => convert_item_usage(&x.value, &elem_qname, file_path, out),
            E::PartUsage(x) => convert_part_usage(&x.value, &elem_qname, file_path, out),
            E::InOutDecl(_) => note_unmapped(file_path, "directed parameter"),
            E::OccurrenceUsage(_) => note_unmapped(file_path, "occurrence"),
            E::RefDecl(_) => note_unmapped(file_path, "reference usage"),
            E::VariantUsage(_) => note_unmapped(file_path, "variant"),
            _ => {}
        }
    }
}

fn convert_connection_def(
    c: &sysml_v2_parser::ConnectionDef,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    let Some(name) = ident_name(&c.identification) else {
        return;
    };
    let elem_qname = format!("{qname}::{name}");
    let elements = match &c.body {
        sysml_v2_parser::ConnectionDefBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::ConnectionDefBody::Semicolon { .. } => &[],
    };
    // REQ-TRS-SYSMLV2-014: ConnectionDefBodyElement has no MetadataAnnotation
    // variant, so @Syscribe* fields reach a connection def only via a
    // doc-comment directive, extracted from the already-lifted doc text.
    let (doc, meta) = extract_syscribe_doc_directives(&connection_def_doc(elements));
    let spec = Spec {
        supertype: c.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        ..Default::default()
    }
    .with_doc(doc)
    .with_syscribe_meta(meta)
    .with_metadata(body_metadata(elements));
    push_synth(out, &elem_qname, file_path, ElementType::ConnectionDef, &name, spec);
    convert_body_with_prefixes(elements, out, |e, out| convert_connection_def_body_element(e, &elem_qname, file_path, out));
}

/// Dispatch a member of a `connection def` body — real SysML v2 source uses
/// this to give a connection named ports/attributes/items
/// (`REQ-TRS-SYSMLV2-007`'s "reasonable structural browsing" goal).
fn convert_connection_def_body_element(
    elem: &sysml_v2_parser::ConnectionDefBodyElement,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    use sysml_v2_parser::ConnectionDefBodyElement as E;
    match elem {
        E::AttributeDef(node) => convert_attribute_def(&node.value, qname, file_path, out),
        E::AttributeUsage(node) => convert_attribute_usage(&node.value, qname, file_path, out),
        E::ItemDef(node) => convert_item_def(&node.value, qname, file_path, out),
        E::ItemUsage(node) => convert_item_usage(&node.value, qname, file_path, out),
        E::PortDef(node) => convert_port_def(&node.value, qname, file_path, out),
        E::PortUsage(node) => convert_port_usage(&node.value, qname, file_path, out),
        _ => {} // outside REQ-TRS-SYSMLV2-007's fixed set
    }
}

fn convert_connection_usage(
    c: &sysml_v2_parser::ast::ConnectionUsageMember,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    let Some(name) = odn(c.name).filter(|n| !n.is_empty()) else {
        return; // anonymous connection usage: no identity to qname against
    };
    let elem_qname = format!("{qname}::{name}");
    // c.body is the same ConnectionDefBody/ConnectionDefBodyElement shape
    // convert_connection_def already reads its own body through — reused
    // unchanged here, REQ-TRS-SYSMLV2-012 (the sibling usage-body doc lift
    // REQ-TRS-SYSMLV2-009 didn't reach).
    let elements = match &c.body {
        sysml_v2_parser::ConnectionDefBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::ConnectionDefBody::Semicolon { .. } => &[],
    };
    let spec = Spec {
        typed_by: typing_first(c.typing.as_ref()),
        ..Default::default()
    }
    .with_doc(connection_def_doc(elements))
    .with_metadata(with_prefix_keywords(&c.prefix, body_metadata(elements)));
    push_synth(out, &elem_qname, file_path, ElementType::Connection, &name, spec);
}

/// `doc /* ... */` lift over a `flow def`/`flow` usage body's already-sliced
/// members — `REQ-TRS-SYSMLV2-024`. Both `FlowDef.body` and `FlowUsage.body`
/// share this exact `DefinitionBody`/`DefinitionBodyElement` type (confirmed
/// against the parser's own AST) — a deliberately thin, generic body shape
/// also shared by `AllocationDef`/`AllocationUsage`/`OccurrenceDef`. A `doc`
/// member here is *not* a direct `DefinitionBodyElement::Doc` the way it is
/// for every other body type in this file — confirmed empirically (parsing
/// real source and inspecting the AST directly, not assumed from the enum
/// shape): it lands wrapped as `OccurrenceMember(OccurrenceBodyElement::Doc)`
/// instead, so both shapes are checked here. Every *other*
/// `OccurrenceBodyElement` variant (`FlowUsage`, `PartUsage`, `EndDecl`, ...)
/// is deliberately left unwalked — no unambiguous "this is an end port"
/// signal exists to derive `ends:`/`itemType:` from (see
/// `convert_flow_def`'s doc comment).
fn flow_body_doc(body: &sysml_v2_parser::ast::DefinitionBody) -> String {
    let sysml_v2_parser::ast::DefinitionBody::Brace { elements, .. } = body else {
        return String::new();
    };
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::DefinitionBodyElement::OccurrenceMember(m) => match &m.value {
            sysml_v2_parser::ast::OccurrenceBodyElement::Annotating(a) => annotating_doc(a),
            _ => None,
        },
        _ => None,
    })
}

/// `REQ-TRS-SYSMLV2-024` — a `flow def` synthesizes a real `FlowDef`.
/// `ends:`/`itemType:` (§8.6.1, the shape `model/Flows/PowerFlowDef.md`
/// uses) are deliberately **not** derived from the body: `DefinitionBody`'s
/// only structured content beyond `Doc` reaches through the generic
/// `OccurrenceMember(OccurrenceBodyElement)` variant, which gives no
/// unambiguous "this nested member is an end port" signal the way a nested
/// `StateUsage`/`ActionUsage` did for State/Action — an explicit descope,
/// not an oversight (see the ADR addendum).
fn convert_flow_def(f: &sysml_v2_parser::FlowDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&f.identification) else {
        return; // anonymous flow def: no identity to qname against
    };
    let flow_qname = format!("{qname}::{name}");
    let spec = Spec {
        supertype: f.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        ..Default::default()
    }
    .with_doc(flow_body_doc(&f.body))
    .with_metadata(definition_body_metadata(&f.body));
    push_synth(out, &flow_qname, file_path, ElementType::FlowDef, &name, spec);
}

/// `REQ-TRS-SYSMLV2-024` — a *named* `flow`/`message`/`succession flow`
/// usage synthesizes a real `Flow`. An anonymous one (`f.name` empty/`None`)
/// has no identity to qname against and stays invisible as its own
/// element — it is still, separately, scanned by `flow_usage_entry` into
/// the owning part's `flowConnections:` (see `part_def_flow_entries`),
/// mirroring `convert_connection_usage`'s identical dual pattern exactly.
fn convert_flow_usage(f: &sysml_v2_parser::FlowUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = flow_parts(f).0.filter(|n| !n.is_empty()) else {
        return;
    };
    let flow_qname = format!("{qname}::{name}");
    let spec = Spec {
        item_type: flow_item_type(f),
        ..Default::default()
    }
    .with_doc(flow_body_doc(&f.body))
    .with_metadata(definition_body_metadata(&f.body));
    push_synth(out, &flow_qname, file_path, ElementType::Flow, &name, spec);
}

/// `REQ-TRS-SYSMLV2-025` — an `enum def` synthesizes a real `EnumerationDef`.
/// No `.with_doc(...)` call here at all, deliberately: `EnumDef.body` is
/// `EnumerationBody::{Semicolon, Brace { values: Vec<EnumeratedValue> }}`
/// (confirmed against the parser's own AST) — a flat list of literals with
/// no `Doc` variant anywhere in that shape, unlike every other body type
/// mapped elsewhere in this file. A `doc /* ... */` written inside an
/// `enum def` genuinely has nowhere to land in this parser version; `doc`
/// stays `""`, the same as any element with no doc member at all.
/// `EnumeratedValue` itself carries only a `name` — any inline body or `=
/// expr` initializer on a literal is parsed and discarded by the vendored
/// crate before this crate ever sees it, so `values:` entries can only ever
/// be `{name: ...}`, never the spec's optional `value:`/`valueKind:`/
/// `unit:`/`metadata:` sub-fields (§8.5.2) — a real upstream ceiling, not a
/// Syscribe choice.
fn convert_enum_def(e: &sysml_v2_parser::ast::EnumDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&e.identification) else {
        return; // anonymous enum def: no identity to qname against
    };
    let enum_qname = format!("{qname}::{name}");
    let values = e
        .body
        .members()
        .filter_map(|n| match &n.value {
            sysml_v2_parser::ast::EnumerationBodyElement::Value(v) => {
                let mut m = serde_yaml::Mapping::new();
                m.insert(
                    serde_yaml::Value::from("name"),
                    serde_yaml::Value::from(odn(v.value.identification.name).unwrap_or_default()),
                );
                Some(serde_yaml::Value::Mapping(m))
            }
            _ => None,
        })
        .collect();
    let spec = Spec {
        supertype: e.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        values: nonempty_vec(values),
        ..Default::default()
    };
    push_synth(out, &enum_qname, file_path, ElementType::EnumerationDef, &name, spec);
}

/// `REQ-TRS-SYSMLV2-025` — an `enum` usage synthesizes a real `Enumeration`.
/// `EnumerationUsage.body` is exactly `AttributeBody` — the same shared
/// type `AttributeDef`/`AttributeUsage`/`ItemDef` already use — so
/// `attribute_body_doc` is reused unchanged, no new doc helper needed.
/// `Enumeration` has no documented frontmatter schema of its own at all
/// (the spec only lists it as "usage of an EnumerationDef" in the usage
/// summary table) — `multiplicity`/`is_end` have no obvious native field to
/// land in and stay unmapped, the same class of descope as Flow's
/// `payload.multiplicity`.
fn convert_enum_usage(e: &sysml_v2_parser::ast::EnumerationUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if e.name.s().is_empty() {
        return; // anonymous enum usage: no identity to qname against
    }
    let enum_qname = format!("{qname}::{}", e.name.s());
    let elements = match &e.body {
        sysml_v2_parser::AttributeBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::AttributeBody::Semicolon { .. } => &[],
    };
    let spec = Spec {
        typed_by: oqr(e.type_name),
        ..Default::default()
    }
    .with_doc(attribute_body_doc(elements))
    .with_metadata(body_metadata(elements));
    push_synth(out, &enum_qname, file_path, ElementType::Enumeration, &e.name.s(), spec);
}

/// The common fields shared by every `case`/`analysis`/`verification`
/// Def/Usage body — `REQ-TRS-SYSMLV2-026`/`-027`/`-028`. All six AST
/// structs (`CaseDef`/`CaseUsage`/`AnalysisCaseDef`/`AnalysisCaseUsage`/
/// `VerificationCaseDef`/`VerificationCaseUsage`) share exactly one body
/// type, `UseCaseDefBody` (confirmed directly against the parser's own AST
/// — not `RequirementDefBody`, a genuinely distinct shape reflecting
/// SysMLv2's own specialization hierarchy). `verifies:`/`verdictExpression:`/
/// `verdictType:` (§8.12.3's `VerificationCaseDef`-specific fields) have no
/// AST source here at all -- `UseCaseDefBodyElement` carries no
/// verify-statement or verdict-semantics variant -- and are never
/// populated by this helper or its callers.
struct CaseBodyFields {
    subject: Option<String>,
    actors: Option<Vec<String>>,
    objectives: Option<Vec<serde_yaml::Value>>,
    result_type: Option<String>,
    /// `REQ-TRS-SYSMLV2-054` -- raw names of `include X;` / `then include X;` members.
    includes: Vec<String>,
    doc: String,
    /// `REQ-TRS-SYSMLV2-086` -- metadata applications in the body.
    metadata: Vec<serde_yaml::Value>,
}

/// The use case an `include` member references (`REQ-TRS-SYSMLV2-054`/`-075`): the target of
/// `include X;`/`include a::X;`, or the typing of the declaring form `include use case v : V;`
/// (the included use case is `V`).
fn include_reference(i: &sysml_v2_parser::ast::IncludeUseCase) -> Option<String> {
    i.target.map(qr).or_else(|| typing_first(i.typing.as_ref()))
}

fn case_body_fields(body: &sysml_v2_parser::ast::UseCaseDefBody) -> CaseBodyFields {
    let sysml_v2_parser::ast::UseCaseDefBody::Brace { elements, .. } = body else {
        return CaseBodyFields {
            subject: None,
            actors: None,
            objectives: None,
            result_type: None,
            includes: Vec::new(),
            doc: String::new(),
            metadata: Vec::new(),
        };
    };
    let mut subject = None;
    let mut actors = Vec::new();
    let mut objectives = Vec::new();
    let mut result_type = None;
    let mut includes: Vec<String> = Vec::new();
    for n in elements {
        match &n.value {
            sysml_v2_parser::ast::UseCaseDefBodyElement::IncludeUseCase(i) => {
                includes.extend(include_reference(&i.value))
            }
            sysml_v2_parser::ast::UseCaseDefBodyElement::ThenIncludeUseCase(t) => {
                includes.extend(include_reference(&t.value.include.value))
            }
            sysml_v2_parser::ast::UseCaseDefBodyElement::SubjectDecl(s) => {
                subject = subject
                    .or_else(|| nonempty(typing_display(s.value.typing.as_ref()).unwrap_or_default()));
            }
            sysml_v2_parser::ast::UseCaseDefBodyElement::ActorUsage(a) => {
                if let Some(t) = oqr(a.value.type_name).and_then(nonempty) {
                    actors.push(t);
                }
            }
            // `Objective.requirement` is itself a full nested `RequirementUsage`
            // (name/type_name/... `body: RequirementDefBody`) -- only its own
            // identity is lifted here, as a plain string, matching the native
            // `objectives:` field's simpler documented form (§8.12.1); the
            // objective's own inner body content is not recursed into.
            sysml_v2_parser::ast::UseCaseDefBodyElement::Objective(o) => {
                let r = &o.value.requirement.value;
                if let Some(label) = nonempty(r.name.s()).or_else(|| oqr(r.type_name)) {
                    objectives.push(serde_yaml::Value::from(label));
                }
            }
            // Multiple `return` declarations are legal (real fixtures show up
            // to three in one `verification def`) -- first one with a type
            // wins, matching the native `result:` field's single-string shape.
            sysml_v2_parser::ast::UseCaseDefBodyElement::CaseReturnDecl(r) => {
                result_type = result_type.or_else(|| oqr(r.value.type_name));
            }
            _ => {}
        }
    }
    let doc = collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::UseCaseDefBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    });
    CaseBodyFields {
        subject,
        actors: nonempty_vec(actors),
        objectives: nonempty_vec(objectives),
        result_type,
        includes,
        doc,
        metadata: body_metadata(elements),
    }
}

fn convert_case_def(c: &sysml_v2_parser::CaseDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&c.identification) else {
        return; // anonymous case def: no identity to qname against
    };
    let case_qname = format!("{qname}::{name}");
    let fields = case_body_fields(&c.body);
    let spec = Spec {
        supertype: c.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        is_abstract: def_abstract(&c.definition_prefix).then_some(true),
        subject: fields.subject,
        actors: fields.actors,
        objectives: fields.objectives,
        result_type: fields.result_type,
        ..Default::default()
    }
    .with_doc(fields.doc)
    .with_metadata(fields.metadata);
    push_synth(out, &case_qname, file_path, ElementType::CaseDef, &name, spec);
}

fn convert_case_usage(c: &sysml_v2_parser::CaseUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if c.name.s().is_empty() {
        return; // anonymous case usage: no identity to qname against
    }
    let case_qname = format!("{qname}::{}", c.name.s());
    let fields = case_body_fields(&c.body);
    let spec = Spec {
        typed_by: oqr(c.type_name),
        is_abstract: c.is_abstract.then_some(true),
        subject: fields.subject,
        actors: fields.actors,
        objectives: fields.objectives,
        result_type: fields.result_type,
        ..Default::default()
    }
    .with_doc(fields.doc)
    .with_metadata(fields.metadata);
    push_synth(out, &case_qname, file_path, ElementType::Case, &c.name.s(), spec);
}

fn convert_analysis_case_def(a: &sysml_v2_parser::AnalysisCaseDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&a.identification) else {
        return;
    };
    let case_qname = format!("{qname}::{name}");
    let fields = case_body_fields(&a.body);
    let spec = Spec {
        supertype: a.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        is_abstract: def_abstract(&a.definition_prefix).then_some(true),
        subject: fields.subject,
        actors: fields.actors,
        objectives: fields.objectives,
        result_type: fields.result_type,
        ..Default::default()
    }
    .with_doc(fields.doc)
    .with_metadata(fields.metadata);
    push_synth(out, &case_qname, file_path, ElementType::AnalysisCaseDef, &name, spec);
}

fn convert_analysis_case_usage(a: &sysml_v2_parser::AnalysisCaseUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if a.name.s().is_empty() {
        return;
    }
    let case_qname = format!("{qname}::{}", a.name.s());
    let fields = case_body_fields(&a.body);
    let spec = Spec {
        typed_by: oqr(a.type_name),
        is_abstract: usage_abstract(&a.prefix).then_some(true),
        subject: fields.subject,
        actors: fields.actors,
        objectives: fields.objectives,
        result_type: fields.result_type,
        ..Default::default()
    }
    .with_doc(fields.doc)
    .with_metadata(with_prefix_keywords(&a.prefix, fields.metadata));
    push_synth(out, &case_qname, file_path, ElementType::AnalysisCase, &a.name.s(), spec);
}

fn convert_verification_case_def(v: &sysml_v2_parser::VerificationCaseDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&v.identification) else {
        return;
    };
    let case_qname = format!("{qname}::{name}");
    let fields = case_body_fields(&v.body);
    let spec = Spec {
        supertype: v.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        is_abstract: def_abstract(&v.definition_prefix).then_some(true),
        subject: fields.subject,
        actors: fields.actors,
        objectives: fields.objectives,
        result_type: fields.result_type,
        ..Default::default()
    }
    .with_doc(fields.doc)
    .with_metadata(fields.metadata);
    push_synth(out, &case_qname, file_path, ElementType::VerificationCaseDef, &name, spec);
}

fn convert_verification_case_usage(v: &sysml_v2_parser::VerificationCaseUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if v.name.s().is_empty() {
        return;
    }
    let case_qname = format!("{qname}::{}", v.name.s());
    let fields = case_body_fields(&v.body);
    let spec = Spec {
        typed_by: oqr(v.type_name),
        is_abstract: v.is_abstract.then_some(true),
        subject: fields.subject,
        actors: fields.actors,
        objectives: fields.objectives,
        result_type: fields.result_type,
        ..Default::default()
    }
    .with_doc(fields.doc)
    .with_metadata(fields.metadata);
    push_synth(out, &case_qname, file_path, ElementType::VerificationCase, &v.name.s(), spec);
}

fn convert_interface_def(
    i: &sysml_v2_parser::InterfaceDef,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    let Some(name) = ident_name(&i.identification) else {
        return;
    };
    let elem_qname = format!("{qname}::{name}");
    let elements = match &i.body {
        sysml_v2_parser::InterfaceDefBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::InterfaceDefBody::Semicolon { .. } => &[],
    };
    // REQ-TRS-SYSMLV2-014: InterfaceDefBodyElement has no MetadataAnnotation
    // variant, so @Syscribe* fields reach an interface def only via a
    // doc-comment directive, extracted from the already-lifted doc text.
    let (doc, meta) = extract_syscribe_doc_directives(&interface_def_doc(elements));
    let ends = end_decl_entries(elements, file_path);
    let spec = Spec {
        supertype: i.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        ends,
        ..Default::default()
    }
    .with_doc(doc)
    .with_syscribe_meta(meta)
    .with_metadata(body_metadata(elements));
    push_synth(out, &elem_qname, file_path, ElementType::InterfaceDef, &name, spec);
    convert_body_with_prefixes(elements, out, |e, out| convert_interface_def_body_element(e, &elem_qname, file_path, out));
}

/// Dispatch a member of an `interface def` body — e.g. a named port on the
/// interface (`interface def PowerInterface { port supplyPort : ...; }`).
fn convert_interface_def_body_element(
    elem: &sysml_v2_parser::InterfaceDefBodyElement,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    use sysml_v2_parser::InterfaceDefBodyElement as E;
    match elem {
        E::AttributeDef(node) => convert_attribute_def(&node.value, qname, file_path, out),
        E::AttributeUsage(node) => convert_attribute_usage(&node.value, qname, file_path, out),
        E::ItemDef(node) => convert_item_def(&node.value, qname, file_path, out),
        E::ItemUsage(node) => convert_item_usage(&node.value, qname, file_path, out),
        E::PortDef(node) => convert_port_def(&node.value, qname, file_path, out),
        E::PortUsage(node) => convert_port_usage(&node.value, qname, file_path, out),
        E::EndDecl(_) => {} // lifted onto the interface def's `ends:` by `end_decl_entries`
        E::RefDecl(_) => note_unmapped(file_path, "reference usage"),
        E::ConnectStmt(_) => note_unmapped(file_path, "connect"),
        E::FlowUsage(_) => note_unmapped(file_path, "flow"),
        E::ConstraintUsage(_) => note_unmapped(file_path, "constraint member"),
        _ => {}
    }
}

/// `end` features of an `interface def` as `ends:` entries. A single declared end cannot stand
/// alone (`E125`: a connection needs at least two, and the other may be inherited), so it is
/// counted for `W543` instead of being emitted.
fn end_decl_entries(
    elements: &[sysml_v2_parser::Node<sysml_v2_parser::InterfaceDefBodyElement>],
    file_path: &str,
) -> Option<Vec<serde_yaml::Value>> {
    let ends: Vec<serde_yaml::Value> = elements
        .iter()
        .filter_map(|n| match &n.value {
            sysml_v2_parser::InterfaceDefBodyElement::EndDecl(e) => Some(end_decl_entry(&e.value)),
            _ => None,
        })
        .collect();
    if ends.len() == 1 {
        note_unmapped(file_path, "interface end (fewer than two)");
        return None;
    }
    nonempty_vec(ends)
}

/// Only the `Declaration` variant carries a name — `TypedConnect`/`Connection`
/// are anonymous binary connectors between two endpoints (no identity to
/// qname against), so they contribute nothing here.
fn convert_interface_usage(
    i: &sysml_v2_parser::InterfaceUsage,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    use sysml_v2_parser::InterfaceUsage as I;
    if let I::Declaration { name: Some(name), interface_type, body, .. }
    | I::TypedConnect { name: Some(name), interface_type, body, .. } = i
    {
        let name = dn(*name);
        if name.is_empty() {
            return;
        }
        let elem_qname = format!("{qname}::{name}");
        let spec = Spec {
            typed_by: oqr(*interface_type),
            ..Default::default()
        }
        .with_doc(interface_usage_doc(body.braced_elements().unwrap_or(&[])))
        .with_metadata(body_metadata(body.braced_elements().unwrap_or(&[])));
        push_synth(out, &elem_qname, file_path, ElementType::Interface, &name, spec);
        for n in body.braced_elements().unwrap_or(&[]) {
            use sysml_v2_parser::InterfaceUsageBodyElement as B;
            match &n.value {
                B::RefRedef { .. } => note_unmapped(file_path, "reference usage"),
                B::EndDecl(_) => note_unmapped(file_path, "end feature"),
                B::PortUsage(_) => note_unmapped(file_path, "port"),
                B::FlowUsage(_) => note_unmapped(file_path, "flow"),
                B::Perform(_) => note_unmapped(file_path, "perform"),
                _ => {}
            }
        }
    }
}

fn convert_item_def(
    i: &sysml_v2_parser::ast::ItemDef,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    let Some(name) = ident_name(&i.identification) else {
        return;
    };
    let elem_qname = format!("{qname}::{name}");
    // ItemDef's body is a plain AttributeBody (shared with attribute def/usage
    // bodies) — only nested attributes are legal there, no ports/items.
    let elements = match &i.body {
        sysml_v2_parser::AttributeBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::AttributeBody::Semicolon { .. } => &[],
    };
    let spec = Spec {
        supertype: i.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        ..Default::default()
    }
    .with_doc(attribute_body_doc(elements))
    .with_metadata(body_metadata(elements));
    push_synth(out, &elem_qname, file_path, ElementType::ItemDef, &name, spec);
    convert_body_with_prefixes(elements, out, |e, out| convert_attribute_body_element(e, &elem_qname, file_path, out));
}

/// Dispatch a member of an `item def` body — e.g. a named attribute on the item.
fn convert_attribute_body_element(
    elem: &sysml_v2_parser::ast::AttributeBodyElement,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    use sysml_v2_parser::ast::AttributeBodyElement as E;
    match elem {
        E::AttributeDef(node) => convert_attribute_def(&node.value, qname, file_path, out),
        E::AttributeUsage(node) => convert_attribute_usage(&node.value, qname, file_path, out),
        _ => {} // outside REQ-TRS-SYSMLV2-007's fixed set
    }
}

fn convert_item_usage(
    i: &sysml_v2_parser::ItemUsage,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    if i.name.s().is_empty() {
        return; // anonymous redefinition form (`item :>> shape ...`): skip
    }
    let elem_qname = format!("{qname}::{}", i.name.s());
    // ItemUsage.body IS an AttributeBody, the same shared shape
    // attribute_body_doc already handles for AttributeDef/AttributeUsage/
    // ItemDef — a review caught an earlier claim in this module that
    // ItemUsage "carries no body field," which was wrong (confirmed against
    // the parser's own struct definition and its own item-usage-with-body
    // test coverage); doc-lifting was silently missing here as a result.
    let elements = match &i.body {
        sysml_v2_parser::AttributeBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::AttributeBody::Semicolon { .. } => &[],
    };
    let spec = Spec {
        typed_by: oqr(i.type_name).or_else(|| None),
        is_reference: usage_is_reference(&i.prefix),
        direction: usage_direction(&i.prefix),
        ..Default::default()
    }
    .with_usage_relations(
        i.multiplicity.as_ref().map(|m| &m.value),
        None,
        i.redefines.as_ref().map(|r| &r.value),
    )
    .with_doc(attribute_body_doc(elements))
    .with_metadata(with_prefix_keywords(&i.prefix, body_metadata(elements)));
    push_synth(out, &elem_qname, file_path, ElementType::Item, &i.name.s(), spec);
}

/// `REQ-TRS-SYSMLV2-083` -- `occurrence def X :> S;` is a native `OccurrenceDef` (an `individual`
/// one, or `individual def`, is an `IndividualDef`).
fn convert_occurrence_def(o: &sysml_v2_parser::ast::OccurrenceDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&o.identification) else { return };
    let ty = if o.is_individual { ElementType::IndividualDef } else { ElementType::OccurrenceDef };
    let spec = Spec {
        supertype: o.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        is_abstract: matches!(o.definition_prefix.as_ref().map(|p| &p.value), Some(sysml_v2_parser::ast::DefinitionPrefix::Abstract)).then_some(true),
        is_variation: is_variation_prefix(&o.definition_prefix),
        ..Default::default()
    }
    .with_doc(flow_body_doc(&o.body))
    .with_metadata(definition_body_metadata(&o.body));
    push_synth(out, &format!("{qname}::{name}"), file_path, ty, &name, spec);
}

/// `REQ-TRS-SYSMLV2-083` -- `individual def X :> S;`.
fn convert_individual_def(i: &sysml_v2_parser::ast::IndividualDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&i.identification) else { return };
    let elements = match &i.body {
        sysml_v2_parser::AttributeBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::AttributeBody::Semicolon { .. } => &[],
    };
    let spec = Spec {
        supertype: i.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        is_abstract: matches!(i.definition_prefix.as_ref().map(|p| &p.value), Some(sysml_v2_parser::ast::DefinitionPrefix::Abstract)).then_some(true),
        is_variation: is_variation_prefix(&i.definition_prefix),
        ..Default::default()
    }
    .with_doc(attribute_body_doc(elements))
    .with_metadata(body_metadata(elements));
    push_synth(out, &format!("{qname}::{name}"), file_path, ElementType::IndividualDef, &name, spec);
}

/// Whether an `occurrence` usage maps natively: named (`REQ-TRS-SYSMLV2-083`; a portion kind is
/// native since `REQ-TRS-SYSMLV2-094`).
fn occurrence_usage_mappable(o: &sysml_v2_parser::ast::OccurrenceUsage) -> bool {
    odn(o.name).is_some_and(|n| !n.is_empty())
}

/// `REQ-TRS-SYSMLV2-083` -- `occurrence o : T [m];` / `individual occurrence` / `event occurrence`;
/// `REQ-TRS-SYSMLV2-094` -- `snapshot`/`timeslice` is the native `isPortion`/`portionKind`.
fn convert_occurrence_usage(o: &sysml_v2_parser::ast::OccurrenceUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if !occurrence_usage_mappable(o) {
        return;
    }
    let name = odn(o.name).unwrap_or_default();
    let individual = matches!(&o.prefix.head, sysml_v2_parser::ast::OccurrenceUsagePrefixHead::Basic { individual_span: Some(_), .. });
    let portion = o.prefix.portion().map(|p| p.value.keyword().to_string());
    let elements = o.body.braced_elements().unwrap_or(&[]);
    let doc = collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::OccurrenceBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    });
    let spec = Spec {
        typed_by: oqr(o.type_name).filter(|t| !t.is_empty()),
        is_individual: individual.then_some(true),
        is_portion: portion.is_some().then_some(true),
        portion_kind: portion,
        ..Default::default()
    }
    .with_usage_relations(o.multiplicity.as_ref().map(|m| &m.value), o.subsets.as_ref().map(|r| &r.value), o.redefines.as_ref().map(|r| &r.value))
    .with_doc(doc)
    .with_metadata(with_prefix_keywords(&o.prefix, body_metadata(elements)));
    let ty = if o.is_event { ElementType::EventOccurrence } else { ElementType::Occurrence };
    push_synth(out, &format!("{qname}::{name}"), file_path, ty, &name, spec);
}

/// `REQ-TRS-SYSMLV2-084` -- `dependency n from a, b to c;`; `REQ-TRS-SYSMLV2-093` -- an anonymous
/// one is named `dependency_N`, N counting from 1 in source order within the owning scope and
/// skipping a name already taken there (the same convention as `accept_N`/`if_N`).
fn convert_dependency(d: &sysml_v2_parser::ast::Dependency, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let name = match d.identification.as_ref().and_then(ident_name).filter(|n| !n.is_empty()) {
        Some(n) => n,
        None => (1..)
            .map(|n| format!("dependency_{n}"))
            .find(|n| !out.iter().any(|e| e.qualified_name == format!("{qname}::{n}")))
            .unwrap_or_default(),
    };
    let elements = d.body.braced_elements().unwrap_or(&[]);
    let doc = collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::RelationshipBodyElement::Annotating(a) => annotating_doc(a),
        _ => None,
    });
    let spec = Spec {
        clients: nonempty_vec(d.clients.iter().map(|c| qr_segments(*c).join("::")).collect()),
        suppliers: nonempty_vec(d.suppliers.iter().map(|c| qr_segments(*c).join("::")).collect()),
        ..Default::default()
    }
    .with_doc(doc)
    .with_metadata(body_metadata(elements));
    push_synth(out, &format!("{qname}::{name}"), file_path, ElementType::Dependency, &name, spec);
}

/// `REQ-TRS-SYSMLV2-084`: rewrite each dependency's raw `clients`/`suppliers` names to the
/// qualified name of the ingested element they resolve to (innermost scope first). A name that
/// resolves to nothing here (a library type, another file's tool) is kept as written.
fn resolve_dependency_ends(out: &mut [RawElement]) {
    let index: super::EndpointIndex = out.iter().map(|e| (e.qualified_name.clone(), (None, None))).collect();
    for e in out.iter_mut().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Dependency))) {
        let scope = e.qualified_name.clone();
        for which in 0..2 {
            // The two fields may sit in different tiers, so borrow one at a time.
            let list = if which == 0 { &mut e.frontmatter.clients } else { &mut e.frontmatter.suppliers };
            if let Some(v) = list {
                for name in v.iter_mut() {
                    if let Some(q) = super::lookup_scoped(&index, &scope, name) {
                        *name = q;
                    }
                }
            }
        }
    }
}

fn convert_requirement_def(
    r: &sysml_v2_parser::RequirementDef,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    let Some(name) = ident_name(&r.identification) else {
        return;
    };
    let elem_qname = format!("{qname}::{name}");
    let spec = Spec {
        supertype: r.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        verifies: nonempty_vec(requirement_verify_targets(&r.body)),
        // RequirementDef itself has no variation-prefix field at all in this
        // parser version (confirmed: no `DefinitionPrefix`/`is_variation`-like
        // member on the `RequirementDef` struct) — a `variation requirement
        // def ...` isn't parseable as a variation point per this grammar, so
        // unlike `RequirementUsage` below there's no `is_variation` to set
        // here. The `@SyscribeFeature` search still applies unconditionally,
        // though, matching the same policy `convert_part_def` uses: any
        // element carrying the annotation gets applies_when regardless of
        // whether it's specifically a variation/variant.
        applies_when: requirement_body_syscribe_feature_id(&r.body),
        ..Default::default()
    }
    .with_doc(requirement_def_body_doc(&r.body))
    .with_metadata(requirement_def_body_metadata(&r.body));
    push_synth(out, &elem_qname, file_path, ElementType::RequirementDef, &name, spec);
}

fn convert_requirement_usage(
    r: &sysml_v2_parser::RequirementUsage,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    if r.name.s().is_empty() {
        return;
    }
    let elem_qname = format!("{qname}::{}", r.name.s());
    let spec = Spec {
        typed_by: oqr(r.type_name),
        is_variation: (r.is_variation).then_some(true),
        verifies: nonempty_vec(requirement_verify_targets(&r.body)),
        // REQ-TRS-SYSMLV2-005: `RequirementUsage` carries its own independent
        // `is_variation: bool` ("variation requirement ..." — a variation
        // point whose body holds `variant` members), unrelated to
        // `PartDef`/`PartUsage`'s `DefinitionPrefix`. Its shared
        // `RequirementDefBody` genuinely carries a `MetadataAnnotation`
        // variant, so `@SyscribeFeature{ featureId = '...'; }` is reachable
        // here exactly like it is on a Part.
        applies_when: requirement_body_syscribe_feature_id(&r.body),
        ..Default::default()
    }
    .with_doc(requirement_def_body_doc(&r.body))
    .with_metadata(requirement_def_body_metadata(&r.body));
    push_synth(out, &elem_qname, file_path, ElementType::Requirement, &r.name.s(), spec);
}

/// `verify` targets nested directly inside a `requirement def`/`requirement`
/// body (`REQ-TRS-SYSMLV2-003`) — the only body context this parser version
/// recognizes the `verify` keyword in at all (see this task's report for the
/// judgment call this reflects).
fn requirement_verify_targets(body: &sysml_v2_parser::RequirementDefBody) -> Vec<String> {
    let sysml_v2_parser::RequirementDefBody::Brace { elements, .. } = body else {
        return Vec::new();
    };
    elements
        .iter()
        .filter_map(|n| match &n.value {
            sysml_v2_parser::RequirementDefBodyElement::VerifyRequirement(v) => {
                verify_target(&v.value)
            }
            _ => None,
        })
        .collect()
}

/// `@SyscribeFeature` search over a `requirement def`/`requirement` usage
/// body (`REQ-TRS-SYSMLV2-005`) — both `RequirementDef` and `RequirementUsage`
/// share this `RequirementDefBody`/`RequirementDefBodyElement` shape, which
/// carries a real `MetadataAnnotation` variant. See [`syscribe_feature_id`].
fn requirement_body_syscribe_feature_id(
    body: &sysml_v2_parser::RequirementDefBody,
) -> Option<String> {
    let sysml_v2_parser::RequirementDefBody::Brace { elements, .. } = body else {
        return None;
    };
    elements.iter().find_map(|n| match &n.value {
        sysml_v2_parser::RequirementDefBodyElement::Annotating(a) => {
            annotating_meta(a).and_then(syscribe_feature_id)
        }
        _ => None,
    })
}

/// `REQ-TRS-SYSMLV2-029` (GH #142) — an `allocation def` synthesizes a native
/// `AllocationDef`, so an ingested `allocation` usage's `typedBy:` can resolve
/// in-model (and an unresolvable one is `E111` like any other `typedBy:`).
/// `AllocationDef.body` is the same thin `DefinitionBody` a `flow def` has, so
/// the doc comment is lifted by the shared `flow_body_doc`.
fn convert_allocation_def(
    a: &sysml_v2_parser::ast::AllocationDef,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    let Some(name) = ident_name(&a.identification) else {
        return; // anonymous allocation def: no identity to qname against
    };
    let def_qname = format!("{qname}::{name}");
    let spec = Spec {
        supertype: a.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        ..Default::default()
    }
    .with_doc(flow_body_doc(&a.body))
    .with_metadata(definition_body_metadata(&a.body));
    push_synth(out, &def_qname, file_path, ElementType::AllocationDef, &name, spec);
}

fn convert_allocation_usage(
    a: &sysml_v2_parser::AllocationUsage,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    if a.name.s().is_empty() {
        return;
    }
    let elem_qname = format!("{qname}::{}", a.name.s());
    // REQ-TRS-SYSMLV2-029 (GH #144): lift the `allocate <source> to
    // <target>` clause. The endpoints stay raw text here (this pass has no
    // view of the rest of the model); `super::resolve_allocation_endpoints`
    // resolves them against the fully merged element list afterwards.
    let endpoint = |e: &Option<sysml_v2_parser::Node<sysml_v2_parser::ast::KermlConnectorEnd>>| {
        e.as_ref().map(|n| vec![qr_segments(n.value.target).join(".")])
    };
    let spec = Spec {
        typed_by: oqr(a.type_name),
        allocated_from: endpoint(&a.source),
        allocated_to: endpoint(&a.target),
        ..Default::default()
    };
    push_synth(out, &elem_qname, file_path, ElementType::Allocation, &a.name.s(), spec);
}

/// `variant name;` / `variant part name : Type { ... }` member of a
/// `variation` def/usage body (`REQ-TRS-SYSMLV2-007`'s "variation/variant
/// membership"). The element kind follows the typed form when present
/// (`Part`/`Attribute`/`Item`/`Port`); the untyped bare-reference form
/// (`variant name;`) and the `Perform`-typed form (behavior-related, outside
/// the fixed set) synthesize nothing.
///
/// The untyped form doesn't declare anything new — per SysML v2 semantics it
/// just marks an *already-declared* sibling usage (elsewhere in the same
/// body) as a variant. Synthesizing a fresh placeholder for it would create a
/// second `RawElement` at the exact qname the real usage already occupies,
/// silently shadowing it in any qname-keyed index (no `E108`-style duplicate
/// diagnostic exists to catch this on this branch). Full variant-membership
/// linkage back to the real sibling usage is `REQ-TRS-SYSMLV2-005`'s job
/// (`@SyscribeFeature`-adjacent follow-on), not this one — so for now the
/// untyped form is simply invisible, exactly like a dangling reference to a
/// name that doesn't exist at all would be.
fn convert_variant_usage(
    v: &sysml_v2_parser::ast::VariantUsage,
    part_qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    use sysml_v2_parser::ast::VariantUsageForm as F;
    use sysml_v2_parser::ast::VariantTypedUsage as T;
    // The bare-reference form (`variant name;`, new body-carrying shape in 0.55+) marks an
    // already-declared sibling and declares nothing (see above); only the typed forms synthesize.
    let F::Typed(typed) = &v.form else { return };
    let (v_name, typed_kind) = match typed {
        T::Part(pu) => (pu.value.name.s(), 0),
        T::Attribute(au) => (au.value.name.s(), 1),
        T::Item(iu) => (iu.value.name.s(), 2),
        T::Port(pu) => (pu.value.name.s(), 3),
        // `variant action`/`variant requirement` and `perform` variants are outside the fixed set.
        T::Action(_) | T::Perform(_) | T::Requirement(_) => return,
    };
    let _ = typed_kind;
    if v_name.is_empty() {
        return;
    }
    let elem_qname = format!("{part_qname}::{v_name}");
    let base_spec = || Spec {
        is_variant: Some(true),
        variant_of: Some(part_qname.to_string()),
        ..Default::default()
    };
    match typed {
        T::Part(pu) => {
            let mut spec = base_spec();
            spec.typed_by = typing_first(pu.value.typing.as_ref()).and_then(nonempty);
            let mut truncations = Vec::new();
            if let sysml_v2_parser::PartUsageBody::Brace { elements, .. } = &pu.value.body {
                spec.applies_when = part_usage_syscribe_feature_id(elements);
                let (connections, t) = part_usage_connection_entries(&elem_qname, elements);
                truncations = t;
                spec = spec
                    .with_syscribe_meta(part_usage_syscribe_meta(elements))
                    .with_doc(part_usage_doc(elements))
                    .with_connections(connections);
            }
            push_synth(out, &elem_qname, file_path, ElementType::Part, &v_name, spec);
            push_connection_truncation_findings(out, file_path, truncations);
        }
        T::Attribute(au) => {
            let mut spec = base_spec();
            spec.typed_by = au.value.typing.as_ref().map(|t| refs_display(&t.value.target));
            if let sysml_v2_parser::AttributeBody::Brace { elements, .. } = &au.value.body {
                spec = spec.with_doc(attribute_body_doc(elements))
    .with_metadata(body_metadata(elements));
            }
            push_synth(out, &elem_qname, file_path, ElementType::Attribute, &v_name, spec);
        }
        T::Item(iu) => {
            let mut spec = base_spec();
            spec.typed_by = oqr(iu.value.type_name);
            if let sysml_v2_parser::AttributeBody::Brace { elements, .. } = &iu.value.body {
                spec = spec.with_doc(attribute_body_doc(elements))
    .with_metadata(body_metadata(elements));
            }
            push_synth(out, &elem_qname, file_path, ElementType::Item, &v_name, spec);
        }
        T::Port(pu) => {
            let mut spec = base_spec();
            spec.typed_by = p_type(&pu.value);
            if let sysml_v2_parser::PortBody::Brace { elements, .. } = &pu.value.body {
                spec = spec.with_doc(port_usage_doc(elements));
            }
            push_synth(out, &elem_qname, file_path, ElementType::Port, &v_name, spec);
        }
        T::Action(_) | T::Perform(_) | T::Requirement(_) => {}
    }
}

// ---------------------------------------------------------------------------
// `REQ-TRS-SYSMLV2-033`/`-034`/`-035`: constraint, calc and use case families.
// ---------------------------------------------------------------------------

fn direction_str(d: sysml_v2_parser::ast::InOut) -> &'static str {
    match d {
        sysml_v2_parser::ast::InOut::In => "in",
        sysml_v2_parser::ast::InOut::Out => "out",
        sysml_v2_parser::ast::InOut::InOut => "inout",
    }
}

fn parameter_entry(name: &str, type_name: &str, direction: &str) -> serde_yaml::Value {
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), ykey(name));
    if !type_name.is_empty() {
        m.insert(ykey("typedBy"), ykey(type_name));
    }
    m.insert(ykey("direction"), ykey(direction));
    serde_yaml::Value::Mapping(m)
}

/// Joined rendering of every body expression -- one per line, in source
/// order; `None` when the body has none.
fn join_expressions(exprs: Vec<String>) -> Option<String> {
    nonempty(exprs.join("\n"))
}

struct ConstraintBodyFields {
    parameters: Option<Vec<serde_yaml::Value>>,
    expression: Option<String>,
    doc: String,
    metadata: Vec<serde_yaml::Value>,
}

fn constraint_body_fields(body: &sysml_v2_parser::ast::ConstraintDefBody) -> ConstraintBodyFields {
    use sysml_v2_parser::ast::ConstraintDefBodyElement as B;
    let sysml_v2_parser::ast::ConstraintDefBody::Brace { elements, .. } = body else {
        return ConstraintBodyFields { parameters: None, expression: None, doc: String::new(), metadata: Vec::new() };
    };
    let mut params = Vec::new();
    let mut exprs = Vec::new();
    for n in elements {
        match &n.value {
            B::InOutDecl(d) => params.push(parameter_entry(
                &d.value.name.s(),
                &oqr(d.value.type_name).unwrap_or_default(),
                direction_str(d.value.direction),
            )),
            B::Expression(e) => exprs.push(render_expression(&e.value)),
            _ => {}
        }
    }
    let doc = collect_doc(elements, |e| match e {
        B::Annotating(a) => annotating_doc(a),
        _ => None,
    });
    ConstraintBodyFields { parameters: nonempty_vec(params), expression: join_expressions(exprs), doc, metadata: body_metadata(elements) }
}

/// `REQ-TRS-SYSMLV2-052`: whether `text`, written as the single body statement
/// block of a `constraint def`/`calc def`, is read back by ingestion as exactly
/// the same expression text. The parser recovers from nearly any input, so
/// "parses" proves nothing; only an exact read-back makes the text safe to
/// export as source.
pub(crate) fn expression_round_trips(is_calc: bool, text: &str) -> bool {
    let kw = if is_calc { "calc" } else { "constraint" };
    let probe = format!("package P {{ {kw} def X {{\n{text}\n}} }}");
    let Ok(parsed) = sysml_v2_parser::parse(&probe) else { return false };
    let (doc, root) = split_document(parsed);
    with_doc(&doc, || expression_probe_matches(is_calc, text, &root))
}

fn expression_probe_matches(is_calc: bool, text: &str, root: &sysml_v2_parser::RootNamespace) -> bool {
    let norm = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ");
    for n in &root.elements {
        let sysml_v2_parser::RootElement::Package(p) = &n.value else { continue };
        let sysml_v2_parser::PackageBody::Brace { elements, .. } = &p.value.body else { continue };
        for e in elements {
            match &e.value {
                sysml_v2_parser::PackageBodyElement::ConstraintDef(c) if !is_calc => {
                    return constraint_body_fields(&c.value.body).expression.is_some_and(|x| norm(&x) == norm(text));
                }
                sysml_v2_parser::PackageBodyElement::CalcDef(c) if is_calc => {
                    return calc_body_fields(&c.value.body).body.is_some_and(|x| norm(&x) == norm(text));
                }
                _ => {}
            }
        }
    }
    false
}

fn convert_constraint_def(c: &sysml_v2_parser::ast::ConstraintDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&c.identification) else {
        return; // anonymous: no identity to qname against
    };
    let fields = constraint_body_fields(&c.body);
    let spec = Spec {
        supertype: c.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        parameters: fields.parameters,
        expression: fields.expression,
        ..Default::default()
    }
    .with_doc(fields.doc)
    .with_metadata(fields.metadata);
    push_synth(out, &format!("{qname}::{name}"), file_path, ElementType::ConstraintDef, &name, spec);
}

fn convert_constraint_usage(c: &sysml_v2_parser::ast::ConstraintUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if c.name.s().is_empty() {
        return;
    }
    let fields = constraint_body_fields(&c.body);
    let spec = Spec {
        typed_by: oqr(c.type_name),
        parameters: fields.parameters,
        expression: fields.expression,
        ..Default::default()
    }
    .with_doc(fields.doc)
    .with_metadata(with_prefix_keywords(&c.prefix, fields.metadata));
    push_synth(out, &format!("{qname}::{}", c.name.s()), file_path, ElementType::Constraint, &c.name.s(), spec);
}

struct CalcBodyFields {
    parameters: Option<Vec<serde_yaml::Value>>,
    return_type: Option<String>,
    body: Option<String>,
    doc: String,
    metadata: Vec<serde_yaml::Value>,
}

fn calc_body_fields(body: &sysml_v2_parser::ast::CalcDefBody) -> CalcBodyFields {
    use sysml_v2_parser::ast::CalcDefBodyElement as B;
    let sysml_v2_parser::ast::CalcDefBody::Brace { elements, .. } = body else {
        return CalcBodyFields { parameters: None, return_type: None, body: None, doc: String::new(), metadata: Vec::new() };
    };
    let mut params = Vec::new();
    let mut exprs = Vec::new();
    let mut return_type = None;
    for n in elements {
        match &n.value {
            B::ActionMember(m) => {
                if let sysml_v2_parser::ActionDefBodyElement::InOutDecl(d) = &m.value {
                    params.push(parameter_entry(
                        &d.value.name.s(),
                        &oqr(d.value.type_name).unwrap_or_default(),
                        direction_str(d.value.direction),
                    ));
                }
            }
            B::ReturnDecl(r) => {
                // First return type wins (the native `returnType:` is one
                // string); every return also stays visible as a parameter.
                let ty = oqr(r.value.type_name).unwrap_or_default();
                return_type = return_type.or_else(|| nonempty(ty.clone()));
                params.push(parameter_entry(&r.value.name.s(), &ty, "return"));
            }
            B::Expression(e) => exprs.push(render_expression(&e.value)),
            _ => {}
        }
    }
    let doc = collect_doc(elements, |e| match e {
        B::Annotating(a) => annotating_doc(a),
        _ => None,
    });
    CalcBodyFields { parameters: nonempty_vec(params), return_type, body: join_expressions(exprs), doc, metadata: body_metadata(elements) }
}

fn convert_calc_def(c: &sysml_v2_parser::ast::CalcDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&c.identification) else {
        return;
    };
    let f = calc_body_fields(&c.body);
    let spec = Spec {
        body_language: f.body.as_ref().map(|_| "kerml".to_string()),
        parameters: f.parameters,
        return_type: f.return_type,
        body: f.body,
        ..Default::default()
    }
    .with_doc(f.doc)
    .with_metadata(f.metadata);
    push_synth(out, &format!("{qname}::{name}"), file_path, ElementType::CalculationDef, &name, spec);
}

fn convert_calc_usage(c: &sysml_v2_parser::ast::CalcUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&c.identification) else {
        return;
    };
    let f = calc_body_fields(&c.body);
    let spec = Spec {
        typed_by: oqr(c.type_name),
        body_language: f.body.as_ref().map(|_| "kerml".to_string()),
        parameters: f.parameters,
        return_type: f.return_type,
        body: f.body,
        ..Default::default()
    }
    .with_doc(f.doc)
    .with_metadata(f.metadata);
    push_synth(out, &format!("{qname}::{name}"), file_path, ElementType::Calculation, &name, spec);
}

/// `REQ-TRS-SYSMLV2-045`: a package-level `metadata def` -> native `MetadataDef`.
fn convert_metadata_def(m: &sysml_v2_parser::ast::MetadataDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&m.identification) else {
        return;
    };
    let elements = match &m.body {
        sysml_v2_parser::AttributeBody::Brace { elements, .. } => elements.as_slice(),
        sysml_v2_parser::AttributeBody::Semicolon { .. } => &[],
    };
    // `REQ-TRS-SYSMLV2-086`: the def's `attribute` members are its `features:` (the declared
    // tagged-value keys `W045` checks an application against).
    let features = elements
        .iter()
        .filter_map(|n| match &n.value {
            sysml_v2_parser::ast::AttributeBodyElement::AttributeUsage(a) if !a.value.name.s().is_empty() => {
                let mut f = serde_yaml::Mapping::new();
                f.insert(ykey("name"), ykey(&a.value.name.s()));
                if let Some(t) = a.value.typing.as_ref().map(|t| refs_display(&t.value.target)).filter(|t| !t.is_empty()) {
                    f.insert(ykey("typedBy"), ykey(&t));
                }
                Some(serde_yaml::Value::Mapping(f))
            }
            _ => None,
        })
        .collect();
    let spec = Spec {
        supertype: m.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        is_abstract: m.is_abstract.then_some(true),
        features: nonempty_vec(features),
        ..Default::default()
    }
    .with_doc(attribute_body_doc(elements))
    .with_metadata(body_metadata(elements));
    push_synth(out, &format!("{qname}::{name}"), file_path, ElementType::MetadataDef, &name, spec);
}

fn convert_use_case_def(c: &sysml_v2_parser::ast::UseCaseDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&c.identification) else {
        return;
    };
    let f = case_body_fields(&c.body);
    let spec = Spec {
        supertype: c.specializes.as_ref().map(|t| refs_display(&t.value.target)),
        is_abstract: def_abstract(&c.definition_prefix).then_some(true),
        subject: f.subject,
        actors: f.actors,
        objectives: f.objectives,
        result_type: f.result_type,
        includes: nonempty_vec(f.includes),
        ..Default::default()
    }
    .with_doc(f.doc)
    .with_metadata(f.metadata);
    push_synth(out, &format!("{qname}::{name}"), file_path, ElementType::UseCaseDef, &name, spec);
}

fn convert_use_case_usage(c: &sysml_v2_parser::ast::UseCaseUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if c.name.s().is_empty() {
        return;
    }
    let f = case_body_fields(&c.body);
    let spec = Spec {
        typed_by: oqr(c.type_name),
        is_abstract: c.is_abstract.then_some(true),
        subject: f.subject,
        actors: f.actors,
        objectives: f.objectives,
        result_type: f.result_type,
        includes: nonempty_vec(f.includes),
        ..Default::default()
    }
    .with_doc(f.doc)
    .with_metadata(f.metadata);
    push_synth(out, &format!("{qname}::{}", c.name.s()), file_path, ElementType::UseCase, &c.name.s(), spec);
}

// ── Behaviour round-trip probes (REQ-TRS-SYSMLV2-056..058) ─────────────────

/// What ingestion reads out of one action body (`REQ-TRS-SYSMLV2-019`).
pub(crate) struct ProbedAction {
    pub sub_actions: Vec<serde_yaml::Value>,
    pub control_nodes: Vec<serde_yaml::Value>,
    pub successions: Vec<serde_yaml::Value>,
}

/// What ingestion reads out of one state body (`REQ-TRS-SYSMLV2-018`).
pub(crate) struct ProbedState {
    pub sub_states: Vec<serde_yaml::Value>,
    pub transitions: Vec<serde_yaml::Value>,
    pub entry: Option<serde_yaml::Value>,
    pub do_action: Option<serde_yaml::Value>,
    pub exit: Option<serde_yaml::Value>,
}

/// The exporter's faithfulness check: parse `body` as the body of an `action def` (or `action`
/// usage) and return exactly what the real ingestion converters make of it, or `None` when the
/// text does not parse. The exporter only emits a behaviour statement when this equals the
/// native value it came from.
pub(crate) fn probe_action_body(is_usage: bool, body: &str) -> Option<ProbedAction> {
    let kw = if is_usage { "action x" } else { "action def X" };
    let probe = format!("package P {{ {kw} {{\n{body}\n}} }}");
    let (doc, root) = split_document(sysml_v2_parser::parse(&probe).ok()?);
    with_doc(&doc, || probe_action_root(is_usage, &root))
}

fn probe_action_root(is_usage: bool, root: &sysml_v2_parser::RootNamespace) -> Option<ProbedAction> {
    for n in &root.elements {
        let sysml_v2_parser::RootElement::Package(p) = &n.value else { continue };
        let sysml_v2_parser::PackageBody::Brace { elements, .. } = &p.value.body else { continue };
        for e in elements {
            let built = match &e.value {
                sysml_v2_parser::PackageBodyElement::ActionDef(a) if !is_usage => {
                    build_action_def_body(action_def_body_elements(&a.value.body))
                }
                sysml_v2_parser::PackageBodyElement::ActionUsage(a) if is_usage => {
                    build_action_usage_body(au_body_elements(&a.value))
                }
                _ => continue,
            };
            return Some(ProbedAction {
                sub_actions: built.sub_actions,
                control_nodes: built.control_nodes,
                successions: built.succession_connections,
            });
        }
    }
    None
}

/// `REQ-TRS-SYSMLV2-092`: the exporter's faithfulness check for a structural succession -- parse
/// `body` as the body of a `part def` (or `part` usage) and return the `successionConnections:`
/// entries ingestion reads out of it, or `None` when the text does not parse.
pub(crate) fn probe_part_body(is_usage: bool, body: &str) -> Option<Vec<serde_yaml::Value>> {
    let kw = if is_usage { "part x" } else { "part def X" };
    let probe = format!("package P {{ {kw} {{\n{body}\n}} }}");
    let (doc, root) = split_document(sysml_v2_parser::parse(&probe).ok()?);
    with_doc(&doc, || {
        for n in &root.elements {
            let sysml_v2_parser::RootElement::Package(p) = &n.value else { continue };
            let sysml_v2_parser::PackageBody::Brace { elements, .. } = &p.value.body else { continue };
            for e in elements {
                match &e.value {
                    sysml_v2_parser::PackageBodyElement::PartDef(d) if !is_usage => {
                        let els = match &d.value.body {
                            sysml_v2_parser::PartDefBody::Brace { elements, .. } => elements.as_slice(),
                            sysml_v2_parser::PartDefBody::Semicolon { .. } => &[],
                        };
                        return Some(part_def_successions(els));
                    }
                    sysml_v2_parser::PackageBodyElement::PartUsage(u) if is_usage => {
                        let els = match &u.value.body {
                            sysml_v2_parser::PartUsageBody::Brace { elements, .. } => elements.as_slice(),
                            sysml_v2_parser::PartUsageBody::Semicolon { .. } => &[],
                        };
                        return Some(part_usage_successions(els));
                    }
                    _ => {}
                }
            }
        }
        None
    })
}

/// `REQ-TRS-SYSMLV2-068`: the text of `expr` after ingestion's own expression renderer (so `a and b`
/// becomes `a && b`), or `None` when it does not parse as an expression.
pub(crate) fn canonical_expression(expr: &str) -> Option<String> {
    let p = probe_action_body(false, &format!("assign t := {expr};"))?;
    match p.sub_actions.as_slice() {
        [serde_yaml::Value::Mapping(m)] if p.control_nodes.is_empty() => {
            m.get(ykey("value")).and_then(|v| v.as_str()).map(str::to_string)
        }
        _ => None,
    }
}

/// Same as [`probe_action_body`] for a `state def` body.
pub(crate) fn probe_state_body(body: &str) -> Option<ProbedState> {
    let probe = format!("package P {{ state def X {{\n{body}\n}} }}");
    let (doc, root) = split_document(sysml_v2_parser::parse(&probe).ok()?);
    with_doc(&doc, || probe_state_root(&root))
}

fn probe_state_root(root: &sysml_v2_parser::RootNamespace) -> Option<ProbedState> {
    for n in &root.elements {
        let sysml_v2_parser::RootElement::Package(p) = &n.value else { continue };
        let sysml_v2_parser::PackageBody::Brace { elements, .. } = &p.value.body else { continue };
        for e in elements {
            if let sysml_v2_parser::PackageBodyElement::StateDef(s) = &e.value {
                let els = match &s.value.body {
                    sysml_v2_parser::ast::StateDefBody::Brace { elements, .. } => elements.as_slice(),
                    sysml_v2_parser::ast::StateDefBody::Semicolon { .. } => &[],
                };
                let b = build_state_body(els, true);
                return Some(ProbedState {
                    sub_states: b.sub_states,
                    transitions: b.transitions,
                    entry: b.entry_action,
                    do_action: b.do_action,
                    exit: b.exit_action,
                });
            }
        }
    }
    None
}
