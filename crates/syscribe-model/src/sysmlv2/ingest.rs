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
    body: Vec<(sysml_v2_parser::PackageBodyElement, String)>,
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
        match sysml_v2_parser::parse(&content) {
            Ok(root) => {
                // `REQ-TRS-SYSMLV2-030`: surface what map-narrow ingestion drops.
                let mut counts = BTreeMap::new();
                count_unmapped_root(&root, &mut counts);
                file_counts.push((file_path.clone(), counts));
                merge_root(&mut merged, root, &file_path)
            }
            Err(e) => {
                owner.derive_findings.push(finding(
                    "W541",
                    &file_path,
                    &format!("SysML v2/KerML parse error in '{file_path}': {e}"),
                ));
            }
        }
    }

    let mut out = Vec::new();
    convert_merged(&merged, pkg_qname, &mut out);
    let unresolved = lift_package_satisfies(&merged, pkg_qname, &mut out);
    let unresolved_includes = resolve_includes(&mut out);
    let mut detailed = Vec::new();
    for (file_path, mut counts) in file_counts {
        if let Some(n) = unresolved.get(&file_path) {
            *counts.entry("satisfy").or_insert(0) += n;
        }
        if let Some(n) = unresolved_includes.get(&file_path) {
            *counts.entry("include").or_insert(0) += n;
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
fn satisfy_subject(s: &sysml_v2_parser::ast::Satisfy) -> Option<String> {
    if s.is_negated || s.inline_requirement.is_some() || s.source.value == s.target.value {
        return None;
    }
    match &s.target.value {
        sysml_v2_parser::Expression::FeatureRef(r) => Some(r.clone()),
        sysml_v2_parser::Expression::FeatureChainRef(c) => Some(c.segments.join("::")),
        _ => None,
    }
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
    for (elem, file) in &merged.body {
        if let sysml_v2_parser::PackageBodyElement::Satisfy(n) = elem {
            if let (Some(req), Some(subject)) = (satisfy_target(&n.value), satisfy_subject(&n.value)) {
                pending.push((qname.to_string(), req, subject, file.clone()));
            }
        }
    }
    for (name, child) in &merged.children {
        collect_package_satisfies(child, &format!("{qname}::{name}"), pending);
    }
}

/// Human-readable kind of a package-body member that ingestion parses but
/// does not map (`REQ-TRS-SYSMLV2-030`). `None` for mapped kinds and for pure
/// namespace plumbing (`import`, `comment`, parse-error nodes), which stay quiet.
fn unmapped_kind(e: &sysml_v2_parser::PackageBodyElement, in_named_pkg: bool) -> Option<&'static str> {
    use sysml_v2_parser::PackageBodyElement as E;
    Some(match e {
        E::TextualRep(_) => "textual representation",
        E::Filter(_) => "filter",
        // `REQ-TRS-SYSMLV2-043`: lifted onto the enclosing named package.
        E::AliasDef(a) => {
            if in_named_pkg && ident_name(&a.value.identification).is_some() {
                return None;
            }
            "alias"
        }
        // `REQ-TRS-SYSMLV2-046`: liftable ones are counted after resolution.
        E::Satisfy(s) => {
            if in_named_pkg && satisfy_target(&s.value).is_some() && satisfy_subject(&s.value).is_some() {
                return None;
            }
            "satisfy"
        }
        E::Actor(_) => "actor",
        E::IndividualDef(_) => "individual def",
        E::MetadataUsage(_) => "metadata",
        E::OccurrenceDef(_) => "occurrence def",
        E::OccurrenceUsage(_) => "occurrence",
        E::Dependency(_) => "dependency",
        E::FeatureDecl(_) | E::ClassifierDecl(_) | E::KermlSemanticDecl(_) | E::KermlFeatureDecl(_)
        | E::ExtendedLibraryDecl(_) => "KerML declaration",
        _ => return None,
    })
}

fn count_unmapped_body(
    elements: &[sysml_v2_parser::Node<sysml_v2_parser::PackageBodyElement>],
    counts: &mut BTreeMap<&'static str, usize>,
    in_named_pkg: bool,
) {
    use sysml_v2_parser::PackageBodyElement as E;
    for n in elements {
        match &n.value {
            E::Package(inner) => {
                if let sysml_v2_parser::PackageBody::Brace { elements } = &inner.value.body {
                    count_unmapped_body(elements, counts, ident_name(&inner.value.identification).is_some());
                }
            }
            E::LibraryPackage(inner) => {
                if let sysml_v2_parser::PackageBody::Brace { elements } = &inner.value.body {
                    count_unmapped_body(elements, counts, ident_name(&inner.value.identification).is_some());
                }
            }
            other => {
                if let Some(k) = unmapped_kind(other, in_named_pkg) {
                    *counts.entry(k).or_insert(0) += 1;
                }
            }
        }
    }
}

pub(crate) fn count_unmapped_root(root: &sysml_v2_parser::RootNamespace, counts: &mut BTreeMap<&'static str, usize>) {
    use sysml_v2_parser::RootElement as R;
    for n in &root.elements {
        match &n.value {
            R::Package(p) => {
                if let sysml_v2_parser::PackageBody::Brace { elements } = &p.value.body {
                    count_unmapped_body(elements, counts, ident_name(&p.value.identification).is_some());
                }
            }
            R::LibraryPackage(p) => {
                if let sysml_v2_parser::PackageBody::Brace { elements } = &p.value.body {
                    count_unmapped_body(elements, counts, ident_name(&p.value.identification).is_some());
                }
            }
            R::Namespace(p) => {
                if let sysml_v2_parser::PackageBody::Brace { elements } = &p.value.body {
                    count_unmapped_body(elements, counts, ident_name(&p.value.identification).is_some());
                }
            }
            R::Import(_) => {}
            R::Member(m) => {
                if let Some(k) = unmapped_kind(&m.value, false) {
                    *counts.entry(k).or_insert(0) += 1;
                }
            }
        }
    }
}

/// Merge one file's already-parsed root namespace into `target`. Only
/// `Package` is handled so far — `REQ-TRS-SYSMLV2-007`'s remaining fixed kinds
/// land in later commits; everything else (bare root members, `library
/// package`, `namespace`, imports) is silently invisible for now (parse-broad,
/// map-narrow — same posture the full mapping will keep for constructs outside
/// the fixed set).
fn merge_root(target: &mut MergedPackage, root: sysml_v2_parser::RootNamespace, file_path: &str) {
    use sysml_v2_parser::RootElement as R;
    for node in root.elements {
        match node.value {
            R::Package(p) => merge_named(target, &p.value.identification, p.value.body, file_path),
            // `REQ-TRS-SYSMLV2-044`: library packages and namespaces become Packages.
            R::LibraryPackage(p) => merge_named(target, &p.value.identification, p.value.body, file_path),
            R::Namespace(p) => merge_named(target, &p.value.identification, p.value.body, file_path),
            _ => {}
        }
    }
}

/// Merge one `package`/`library package`/`namespace` declaration into
/// `target.children`, combining with whatever a same-named package already
/// contributed (from this file or an earlier one).
fn merge_named(
    target: &mut MergedPackage,
    identification: &sysml_v2_parser::Identification,
    body: sysml_v2_parser::PackageBody,
    file_path: &str,
) {
    let Some(name) = ident_name(identification) else {
        return; // anonymous package: no identity to qname or merge against
    };
    let is_new = !target.children.contains_key(&name);
    let entry = target.children.entry(name).or_default();
    if is_new {
        entry.declared_in = Some(file_path.to_string());
    }
    if let sysml_v2_parser::PackageBody::Brace { elements } = body {
        merge_package_body(entry, elements, file_path);
    }
}

fn merge_package_body(
    target: &mut MergedPackage,
    elements: Vec<sysml_v2_parser::Node<sysml_v2_parser::PackageBodyElement>>,
    file_path: &str,
) {
    for node in elements {
        match node.value {
            sysml_v2_parser::PackageBodyElement::Package(inner) => {
                merge_named(target, &inner.value.identification, inner.value.body, file_path);
            }
            sysml_v2_parser::PackageBodyElement::LibraryPackage(inner) => {
                merge_named(target, &inner.value.identification, inner.value.body, file_path);
            }
            other => target.body.push((other, file_path.to_string())),
        }
    }
}

fn ident_name(id: &sysml_v2_parser::Identification) -> Option<String> {
    id.name.clone().or_else(|| id.short_name.clone())
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
        frontmatter: RawFrontmatter {
            element_type: Some(ty),
            name: Some(name.to_string()),
            supertype: spec.supertype.map(serde_yaml::Value::String),
            typed_by: spec.typed_by.map(serde_yaml::Value::String),
            is_variation: spec.is_variation,
            is_variant: spec.is_variant,
            variant_of: spec.variant_of,
            satisfies: spec.satisfies,
            verifies: spec.verifies,
            applies_when: spec.applies_when.map(serde_yaml::Value::String),
            domain: spec.domain,
            asil_level: spec.asil_level,
            sil_level: spec.sil_level,
            pl_level: spec.pl_level,
            short_name: spec.short_name,
            implemented_by: spec.implemented_by,
            connections: spec.connections,
            sub_states: spec.sub_states,
            transitions: spec.transitions,
            entry_action: spec.entry_action,
            do_action: spec.do_action,
            exit_action: spec.exit_action,
            sub_actions: spec.sub_actions,
            control_nodes: spec.control_nodes,
            succession_connections: spec.succession_connections,
            expose: spec.expose,
            viewpoint: spec.viewpoint,
            stakeholders: spec.stakeholders,
            concerns: spec.concerns,
            rendering: spec.rendering,
            subject: spec.subject,
            item_type: spec.item_type,
            flow_connections: spec.flow_connections,
            values: spec.values,
            actors: spec.actors,
            objectives: spec.objectives,
            result_type: spec.result_type,
            is_abstract: spec.is_abstract,
            allocated_from: spec.allocated_from,
            allocated_to: spec.allocated_to,
            parameters: spec.parameters,
            expression: spec.expression,
            body: spec.body,
            body_language: spec.body_language,
            return_type: spec.return_type,
            aliases: spec.aliases,
            multiplicity: spec.multiplicity,
            subsets: spec.subsets,
            redefines: spec.redefines,
            includes: spec.includes,
            value: spec.value,
            unit: spec.unit,
            ..Default::default()
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
        sysml_v2_parser::Expression::FeatureRef(s) => Some(s.clone()),
        _ => None,
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
fn satisfy_target(s: &sysml_v2_parser::ast::Satisfy) -> Option<String> {
    if s.is_negated || s.inline_requirement.is_some() {
        return None;
    }
    feature_ref_string(&s.source.value)
}

/// The `Requirement` reference a `verify` statement targets — the shorthand
/// `verify 'REQ-X';` form's `target` field is already a plain, quote-stripped
/// string. The fuller `verify requirement <inline> : Type ...` form declares
/// a fresh inline requirement usage rather than referencing an existing one
/// (`target` is `None` there), so it isn't mapped.
fn verify_target(v: &sysml_v2_parser::ast::VerifyRequirementMember) -> Option<String> {
    v.target.clone()
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
fn attribute_body_string(body: &sysml_v2_parser::AttributeBody, key: &str) -> Option<String> {
    let sysml_v2_parser::AttributeBody::Brace { elements } = body else {
        return None;
    };
    elements.iter().find_map(|n| match &n.value {
        sysml_v2_parser::ast::AttributeBodyElement::AttributeUsage(a) if a.value.name == key => a
            .value
            .value
            .as_ref()
            .and_then(|fv| match &fv.value.expression.value {
                sysml_v2_parser::Expression::LiteralString(s) => Some(s.clone()),
                other => feature_ref_string(other),
            }),
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
fn attribute_body_i64(body: &sysml_v2_parser::AttributeBody, key: &str) -> Option<i64> {
    let sysml_v2_parser::AttributeBody::Brace { elements } = body else {
        return None;
    };
    elements.iter().find_map(|n| match &n.value {
        sysml_v2_parser::ast::AttributeBodyElement::AttributeUsage(a) if a.value.name == key => {
            a.value.value.as_ref().and_then(|fv| match &fv.value.expression.value {
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
    if m.name != "SyscribeFeature" {
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
    match m.name.as_str() {
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
fn collect_doc<T>(elements: &[sysml_v2_parser::Node<T>], as_doc: impl Fn(&T) -> Option<&str>) -> String {
    elements
        .iter()
        .filter_map(|n| as_doc(&n.value))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// `doc /* ... */` lift over a `part def` body's already-sliced members.
fn part_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::PartDefBodyElement::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `part` usage body's already-sliced members.
fn part_usage_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartUsageBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::PartUsageBodyElement::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `port def` body's already-sliced members.
fn port_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::PortDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::PortDefBodyElement::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `port` usage body's already-sliced members.
fn port_usage_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::PortBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::PortBodyElement::Doc(d) => Some(d.value.text.as_str()),
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
        sysml_v2_parser::InterfaceUsageBodyElement::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `connection def` body's already-sliced members.
fn connection_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ConnectionDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ConnectionDefBodyElement::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    })
}

/// `doc /* ... */` lift over an `interface def` body's already-sliced members.
fn interface_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::InterfaceDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::InterfaceDefBodyElement::Doc(d) => Some(d.value.text.as_str()),
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
        sysml_v2_parser::ast::AttributeBodyElement::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `state def`/`state` usage body's already-sliced
/// members (`REQ-TRS-SYSMLV2-018`) — shared by both `StateDef` and
/// `StateUsage`, since they use the same `StateDefBody`/`StateDefBodyElement`
/// grammar (confirmed against the parser's AST: `StateUsage.body: StateDefBody`).
fn state_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::StateDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::StateDefBodyElement::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    })
}

/// `REQ-TRS-SYSMLV2-018` `@Syscribe*` fixed-field search over a `state
/// def`/`state` usage body's already-sliced members. See
/// [`part_def_syscribe_meta`].
fn state_def_syscribe_meta(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::StateDefBodyElement>]) -> SyscribeMeta {
    let mut meta = SyscribeMeta::default();
    for n in elements {
        if let sysml_v2_parser::ast::StateDefBodyElement::MetadataAnnotation(m) = &n.value {
            fold_syscribe_meta_annotation(&m.value, &mut meta);
        }
    }
    meta
}

/// `doc /* ... */` lift over an `action def` body's already-sliced members
/// (`REQ-TRS-SYSMLV2-019`).
fn action_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ActionDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ActionDefBodyElement::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    })
}

/// `REQ-TRS-SYSMLV2-019` `@Syscribe*` fixed-field search over an `action def`
/// body's already-sliced members.
fn action_def_syscribe_meta(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ActionDefBodyElement>]) -> SyscribeMeta {
    let mut meta = SyscribeMeta::default();
    for n in elements {
        if let sysml_v2_parser::ActionDefBodyElement::MetadataAnnotation(m) = &n.value {
            fold_syscribe_meta_annotation(&m.value, &mut meta);
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
        sysml_v2_parser::ActionUsageBodyElement::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    })
}

/// `REQ-TRS-SYSMLV2-019` `@Syscribe*` fixed-field search over an `action`
/// usage body's already-sliced members.
fn action_usage_syscribe_meta(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ActionUsageBodyElement>]) -> SyscribeMeta {
    let mut meta = SyscribeMeta::default();
    for n in elements {
        if let sysml_v2_parser::ActionUsageBodyElement::MetadataAnnotation(m) = &n.value {
            fold_syscribe_meta_annotation(&m.value, &mut meta);
        }
    }
    meta
}

/// `doc /* ... */` lift over a `view def` body's already-sliced members
/// (`REQ-TRS-SYSMLV2-020`).
fn view_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::ViewDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::ViewDefBodyElement::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `view` usage body's already-sliced members
/// (`REQ-TRS-SYSMLV2-020`) — `ViewBodyElement` is a distinct Rust type from
/// `ViewDefBodyElement` (structurally near-identical, but not shared), so
/// this needs its own wrapper, mirroring `part_def_doc`/`part_usage_doc`.
fn view_usage_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::ViewBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::ViewBodyElement::Doc(d) => Some(d.value.text.as_str()),
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
    let sysml_v2_parser::RequirementDefBody::Brace { elements } = body else {
        return String::new();
    };
    collect_doc(elements, |e| match e {
        sysml_v2_parser::RequirementDefBodyElement::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `rendering def` body's already-sliced members
/// (`REQ-TRS-SYSMLV2-022`).
fn rendering_def_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::RenderingDefBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::RenderingDefBodyElement::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    })
}

/// `doc /* ... */` lift over a `rendering` usage body's already-sliced
/// members (`REQ-TRS-SYSMLV2-022`).
fn rendering_usage_doc(elements: &[sysml_v2_parser::Node<sysml_v2_parser::ast::RenderingUsageBodyElement>]) -> String {
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::RenderingUsageBodyElement::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    })
}

/// The `rendering:` reference text a `render <name> [: <Type>]` clause
/// contributes — `REQ-TRS-SYSMLV2-020`/`-022`. Prefers the referenced
/// type's name (`type_name`); falls back to the render clause's own `name`
/// when untyped (an inline/self-defining render). `None` for a fully
/// anonymous, untyped render clause — nothing meaningful to reference.
fn view_rendering_target(u: &sysml_v2_parser::ast::ViewRenderingUsage) -> Option<String> {
    u.type_name.clone().or_else(|| (!u.name.is_empty()).then(|| u.name.clone()))
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
                Some(serde_yaml::Value::String(e.value.target.clone()))
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
        sysml_v2_parser::ast::ViewBodyElement::Satisfy(s) => Some(s.value.viewpoint_ref.clone()),
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
    let sysml_v2_parser::RequirementDefBody::Brace { elements } = body else {
        return (Vec::new(), Vec::new());
    };
    let mut stakeholders = Vec::new();
    let mut concerns = Vec::new();
    for n in elements {
        match &n.value {
            sysml_v2_parser::RequirementDefBodyElement::Stakeholder(s) => {
                stakeholders.push(s.value.name.clone());
            }
            sysml_v2_parser::RequirementDefBodyElement::Purpose(p) => {
                concerns.push(p.value.target.clone());
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
    let sysml_v2_parser::RequirementDefBody::Brace { elements } = body else {
        return None;
    };
    elements.iter().find_map(|n| match &n.value {
        sysml_v2_parser::RequirementDefBodyElement::SubjectDecl(s) => {
            nonempty(s.value.type_name.clone())
        }
        _ => None,
    })
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
        sysml_v2_parser::Expression::FeatureRef(s) => Some(s.clone()),
        sysml_v2_parser::Expression::FeatureChainRef(chain) => Some(chain.segments.join(".")),
        sysml_v2_parser::Expression::MemberAccess(base, member) => {
            connection_end_display(&base.value).map(|b| format!("{b}.{member}"))
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
        E::LiteralReal(s) => s.clone(),
        E::LiteralString(s) => format!("\"{s}\""),
        E::LiteralBoolean(b) => b.to_string(),
        E::FeatureRef(s) => s.clone(),
        E::MemberAccess(base, member) => format!("{}.{}", render_expression(&base.value), member),
        E::FeatureChainRef(chain) => chain.segments.join("."),
        E::Index { base, index } => format!("{}#({})", render_expression(&base.value), render_expression(&index.value)),
        E::Bracket(inner) => format!("[{}]", render_expression(&inner.value)),
        E::LiteralWithUnit { value, unit } => {
            format!("{} [{}]", render_expression(&value.value), render_expression(&unit.value))
        }
        E::BinaryOp { op, left, right } => {
            format!("{} {} {}", render_expression(&left.value), op.as_str(), render_expression(&right.value))
        }
        E::UnaryOp { op, operand } => format!("{}{}", op.as_str(), render_expression(&operand.value)),
        E::Invocation { callee, args } => {
            let rendered: Vec<String> = args.iter().map(render_argument).collect();
            format!("{}({})", render_expression(&callee.value), rendered.join(", "))
        }
        E::Tuple(items) => {
            let rendered: Vec<String> = items.iter().map(|n| render_expression(&n.value)).collect();
            format!("({})", rendered.join(", "))
        }
        E::Parenthesized(inner) => format!("({})", render_expression(&inner.value)),
        E::Constructor { type_name, args } => {
            let rendered: Vec<String> = args.iter().map(render_argument).collect();
            format!("new {type_name}({})", rendered.join(", "))
        }
        E::Extent { target } => format!("all {target}"),
        E::Null => "null".to_string(),
        E::Classification { .. } => "<classification expression>".to_string(),
        E::MetaCast { .. } => "<meta-cast expression>".to_string(),
        E::TypeCheck { .. } => "<type-check expression>".to_string(),
        E::Select { .. } => "<select expression>".to_string(),
        E::Collect { .. } => "<collect expression>".to_string(),
        E::CollectionOp { .. } => "<collection-operator expression>".to_string(),
        E::MetadataAccess(_) => "<metadata-access expression>".to_string(),
        E::Conditional { .. } => "<conditional expression>".to_string(),
    }
}

/// Render one call/constructor argument — `name = value` for a named
/// argument, bare `value` for a positional one.
fn render_argument(a: &sysml_v2_parser::Argument) -> String {
    match &a.name {
        Some(name) => format!("{name} = {}", render_expression(&a.value.value)),
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
                if su.value.name.is_empty() {
                    continue; // anonymous nested state: no identity to key isInitial/isFinal against
                }
                let type_name = su.value.type_name.as_deref();
                let body_elements = match &su.value.body {
                    sysml_v2_parser::ast::StateDefBody::Brace { elements } => elements.as_slice(),
                    sysml_v2_parser::ast::StateDefBody::Semicolon => &[],
                };
                let entry = state_usage_yaml_entry(&su.value.name, type_name, body_elements);
                children.push((su.value.name.clone(), entry));
            }
            E::Entry(a) => entry_action = a.value.action_name.clone().map(serde_yaml::Value::String),
            E::Do(a) => do_action = a.value.action_name.clone().map(serde_yaml::Value::String),
            E::Exit(a) => exit_action = a.value.action_name.clone().map(serde_yaml::Value::String),
            E::Then(t) => then_names.push(t.value.state_name.clone()),
            E::FinalState(f) => final_names.push(f.value.state_name.clone()),
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
            let text = payload.type_name.clone().unwrap_or_else(|| payload.name.clone());
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
        Eff::Perform { name, type_name } => effect_name_typed(name.as_deref(), type_name.as_deref()),
        Eff::Accept { payload, type_name, .. } => {
            let name = connection_end_display(&payload.value).unwrap_or_else(|| render_expression(&payload.value));
            effect_name_typed(Some(&name), type_name.as_deref())
        }
        Eff::Send { payload, type_name, .. } => {
            let name = connection_end_display(&payload.value).unwrap_or_else(|| render_expression(&payload.value));
            effect_name_typed(Some(&name), type_name.as_deref())
        }
        Eff::Assign { lhs, rhs } => {
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
        sysml_v2_parser::ast::StateDefBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::ast::StateDefBody::Semicolon => &[],
    };
    let body = build_state_body(elements, true);
    let spec = Spec {
        supertype: s.specializes.as_ref().map(|t| t.value.target_display()),
        entry_action: body.entry_action,
        do_action: body.do_action,
        exit_action: body.exit_action,
        ..Default::default()
    }
    .with_syscribe_meta(state_def_syscribe_meta(elements))
    .with_doc(state_def_doc(elements))
    .with_state_machine(body.sub_states, body.transitions);
    push_synth(out, &state_qname, file_path, ElementType::StateDef, &name, spec);
}

fn convert_state_usage(s: &sysml_v2_parser::ast::StateUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if s.name.is_empty() {
        return; // anonymous usage: no identity to qname against
    }
    let state_qname = format!("{qname}::{}", s.name);
    let elements = match &s.body {
        sysml_v2_parser::ast::StateDefBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::ast::StateDefBody::Semicolon => &[],
    };
    let body = build_state_body(elements, true);
    let spec = Spec {
        typed_by: s.type_name.clone(),
        entry_action: body.entry_action,
        do_action: body.do_action,
        exit_action: body.exit_action,
        ..Default::default()
    }
    .with_syscribe_meta(state_def_syscribe_meta(elements))
    .with_doc(state_def_doc(elements))
    .with_state_machine(body.sub_states, body.transitions);
    push_synth(out, &state_qname, file_path, ElementType::State, &s.name, spec);
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
        sysml_v2_parser::PartDefBodyElement::PartUsage(pu) if pu.value.name == head => Some(&pu.value),
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
        sysml_v2_parser::PartUsageBodyElement::PartUsage(pu) if pu.value.name == head => Some(&pu.value),
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
    let sysml_v2_parser::PartUsageBody::Brace { elements } = &pu.body else {
        return false;
    };
    elements.iter().any(|n| match &n.value {
        sysml_v2_parser::PartUsageBodyElement::PortUsage(p) => p.value.name == tail,
        sysml_v2_parser::PartUsageBodyElement::AttributeUsage(a) => a.value.name == tail,
        sysml_v2_parser::PartUsageBodyElement::PartUsage(p) => p.value.name == tail,
        sysml_v2_parser::PartUsageBodyElement::InterfaceUsage(iface) => matches!(
            &iface.value,
            sysml_v2_parser::InterfaceUsage::Declaration { name: Some(n), .. } if n == tail
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
    let typed_by = c.type_name.clone().and_then(nonempty);

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
    f.payload
        .as_ref()
        .and_then(|p| p.value.type_name.clone())
        .or_else(|| f.type_name.clone())
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
    let from = connection_end_display(&f.from.as_ref()?.value)?;
    let to = connection_end_display(&f.to.as_ref()?.value)?;
    let (from_q, from_trunc) = qualify_connection_end(owning_qname, &from, find_sibling);
    let (to_q, to_trunc) = qualify_connection_end(owning_qname, &to, find_sibling);
    truncations.extend(from_trunc);
    truncations.extend(to_trunc);

    let mut m = serde_yaml::Mapping::new();
    if let Some(n) = f.name.clone().filter(|n| !n.is_empty()) {
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
        sysml_v2_parser::PartDefBodyElement::MetadataAnnotation(m) => syscribe_feature_id(&m.value),
        _ => None,
    })
}

/// `@SyscribeFeature` search over a `part` usage body's already-sliced
/// members. See [`part_def_syscribe_feature_id`].
fn part_usage_syscribe_feature_id(
    elements: &[sysml_v2_parser::Node<sysml_v2_parser::PartUsageBodyElement>],
) -> Option<String> {
    elements.iter().find_map(|n| match &n.value {
        sysml_v2_parser::PartUsageBodyElement::MetadataAnnotation(m) => syscribe_feature_id(&m.value),
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
        if let sysml_v2_parser::PartDefBodyElement::MetadataAnnotation(m) = &n.value {
            fold_syscribe_meta_annotation(&m.value, &mut meta);
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
        if let sysml_v2_parser::PartUsageBodyElement::MetadataAnnotation(m) = &n.value {
            fold_syscribe_meta_annotation(&m.value, &mut meta);
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
    fn push_control_node(&mut self, name: String, kind: &str) {
        let mut m = serde_yaml::Mapping::new();
        m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
        m.insert(ykey("kind"), serde_yaml::Value::String(kind.to_string()));
        self.last_named = Some(name);
        self.control_nodes.push(serde_yaml::Value::Mapping(m));
    }

    fn push_succession(&mut self, after: String, before: String) {
        let mut m = serde_yaml::Mapping::new();
        m.insert(ykey("after"), serde_yaml::Value::String(after));
        m.insert(ykey("before"), serde_yaml::Value::String(before));
        self.succession_connections.push(serde_yaml::Value::Mapping(m));
    }
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

fn handle_perform_stmt(b: &mut ActionBodyBuilder, p: &sysml_v2_parser::ast::Perform) {
    push_perform_entry(b, &p.action_name, p.type_name.as_deref());
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
    if let Some(payload) = &au.accept {
        push_payload_entry(b, payload, "AcceptAction");
        apply_step_annotation(b, &au.body);
        return;
    }
    if let Some(payload) = &au.send {
        push_payload_entry(b, payload, "SendAction");
        apply_step_annotation(b, &au.body);
        return;
    }
    if handle_named_step(b, au) {
        return;
    }
    let type_name = (!au.type_name.is_empty()).then_some(au.type_name.as_str());
    push_perform_entry(b, &au.name, type_name);
}

/// `REQ-TRS-SYSMLV2-060`: `action <name> { <one control statement> }` with no typing, subsetting,
/// redefinition, multiplicity or accept/send clause is a *named* `if`/`while`/`loop`/`for`/`assign`/
/// `terminate` step -- the 0.54 grammar gives those statements no name of their own. The entry takes
/// the usage's name and leaves the synthesized-name counters alone. Returns `false` (nothing pushed)
/// for any other shape, which stays a `PerformAction`.
fn handle_named_step(b: &mut ActionBodyBuilder, au: &sysml_v2_parser::ActionUsage) -> bool {
    use sysml_v2_parser::ActionUsageBodyElement as E;
    let plain = !au.name.is_empty()
        && au.type_name.is_empty()
        && au.typing.is_none()
        && au.subsets.is_none()
        && au.redefines.is_none()
        && au.multiplicity.is_none()
        && !au.is_abstract
        && !au.is_variation
        && !au.is_reference;
    let sysml_v2_parser::ActionUsageBody::Brace { elements } = &au.body else { return false };
    // `REQ-TRS-SYSMLV2-069`: a `@SyscribeStep` annotation beside the statement is not a second statement.
    let stmts: Vec<_> = elements.iter().filter(|e| !matches!(&e.value, E::MetadataAnnotation(_))).collect();
    if !plain || stmts.len() != 1 {
        return false;
    }
    let before = b.sub_actions.len();
    b.forced_name = Some(au.name.clone());
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
        apply_step_annotation(b, &au.body);
    }
    pushed
}

/// `REQ-TRS-SYSMLV2-069`/`-070`: the fields the pinned 0.54 grammar has no syntax for travel in a
/// `@SyscribeStep { via = '…'; feature = '…' (the `referent`); valueKind = '…'; triggerKind = '…';
/// triggerCondition = '…'; loopKind = 'until'; condition = '…'; }` metadata annotation inside the
/// step's own body. This folds it into the entry just pushed (the last `subActions:` item).
/// `loopKind`/`condition` only ever turn an unconditioned `loop` into an `until` loop.
fn apply_step_annotation(b: &mut ActionBodyBuilder, body: &sysml_v2_parser::ActionUsageBody) {
    let sysml_v2_parser::ActionUsageBody::Brace { elements } = body else { return };
    let Some(ann) = elements.iter().rev().find_map(|e| match &e.value {
        sysml_v2_parser::ActionUsageBodyElement::MetadataAnnotation(m) if m.value.name == "SyscribeStep" => {
            Some(&m.value)
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
fn push_payload_entry(b: &mut ActionBodyBuilder, payload: &sysml_v2_parser::ast::PayloadClause, kind: &str) {
    let name = payload.name.clone();
    let payload_text = payload.type_name.clone().unwrap_or_else(|| payload.name.clone());
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String(kind.to_string()));
    m.insert(ykey("payload"), serde_yaml::Value::String(payload_text));
    b.push_sub_action(name, m);
}

fn handle_assign(b: &mut ActionBodyBuilder, a: &sysml_v2_parser::ast::AssignStmt) {
    let name = b.synth_name("assign");
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String("AssignmentAction".to_string()));
    m.insert(ykey("target"), serde_yaml::Value::String(render_expression(&a.lhs.value)));
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
    let inner = build_action_def_body(action_def_body_elements(&w.body));
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String("LoopAction".to_string()));
    m.insert(ykey("loopKind"), serde_yaml::Value::String("while".to_string()));
    m.insert(ykey("condition"), serde_yaml::Value::String(render_expression(&w.condition.value)));
    let sub_actions = absorb(b, inner);
    if !sub_actions.is_empty() {
        m.insert(ykey("body"), serde_yaml::Value::Sequence(sub_actions));
    }
    b.push_sub_action(name, m);
}

fn handle_loop(b: &mut ActionBodyBuilder, l: &sysml_v2_parser::ast::LoopStmt) {
    let name = b.synth_name("loop");
    let inner = build_action_def_body(action_def_body_elements(&l.body));
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String("LoopAction".to_string()));
    m.insert(ykey("loopKind"), serde_yaml::Value::String("loop".to_string()));
    let sub_actions = absorb(b, inner);
    if !sub_actions.is_empty() {
        m.insert(ykey("body"), serde_yaml::Value::Sequence(sub_actions));
    }
    b.push_sub_action(name, m);
}

fn handle_for_loop(b: &mut ActionBodyBuilder, f: &sysml_v2_parser::ast::ForLoop) {
    let name = b.synth_name("for");
    let inner = build_action_def_body(action_def_body_elements(&f.body));
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String("LoopAction".to_string()));
    m.insert(ykey("loopKind"), serde_yaml::Value::String("for".to_string()));
    m.insert(ykey("variable"), serde_yaml::Value::String(f.var.clone()));
    m.insert(ykey("sequence"), serde_yaml::Value::String(render_expression(&f.range.value)));
    let sub_actions = absorb(b, inner);
    if !sub_actions.is_empty() {
        m.insert(ykey("body"), serde_yaml::Value::Sequence(sub_actions));
    }
    b.push_sub_action(name, m);
}

fn handle_if(b: &mut ActionBodyBuilder, i: &sysml_v2_parser::ast::IfStmt) {
    let name = b.synth_name("if");
    let then_inner = build_action_def_body(action_def_body_elements(&i.then_body));
    let mut m = serde_yaml::Mapping::new();
    m.insert(ykey("name"), serde_yaml::Value::String(name.clone()));
    m.insert(ykey("kind"), serde_yaml::Value::String("IfAction".to_string()));
    m.insert(ykey("condition"), serde_yaml::Value::String(render_expression(&i.condition.value)));
    let then_actions = absorb(b, then_inner);
    if !then_actions.is_empty() {
        m.insert(ykey("then"), serde_yaml::Value::Sequence(then_actions));
    }
    if let Some(else_body) = &i.else_body {
        let else_inner = build_action_def_body(action_def_body_elements(else_body));
        let else_actions = absorb(b, else_inner);
        if !else_actions.is_empty() {
            m.insert(ykey("else"), serde_yaml::Value::Sequence(else_actions));
        }
    }
    b.push_sub_action(name, m);
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

fn handle_control_node(b: &mut ActionBodyBuilder, expr: &sysml_v2_parser::Expression, kind: &str) {
    let name = connection_end_display(expr).unwrap_or_else(|| render_expression(expr));
    b.push_control_node(name, kind);
}

/// `first X [then Y];` — the succession edge (`X` → `Y`). A bare `first X;`
/// with no `then` is an entry-point marker with no edge to emit; there is no
/// dedicated "entry point" field in this schema, so it's a documented no-op.
fn handle_first_stmt(b: &mut ActionBodyBuilder, f: &sysml_v2_parser::ast::FirstStmt) {
    let Some(then) = &f.then else { return };
    let first_name = connection_end_display(&f.first.value).unwrap_or_else(|| render_expression(&f.first.value));
    let then_name = connection_end_display(&then.value).unwrap_or_else(|| render_expression(&then.value));
    b.push_succession(first_name, then_name);
}

/// `then <target>;` succession shorthand — connects from whatever node was
/// most recently converted (`last_named`) to `target`. Silently dropped when
/// there's no preceding node to connect from (e.g. the very first body
/// element is a bare `then X;`, which isn't valid SysML v2 but stay
/// defensive rather than panic).
fn handle_then_action(b: &mut ActionBodyBuilder, t: &sysml_v2_parser::ast::ThenAction) {
    use sysml_v2_parser::ast::ThenTarget as T;
    let Some(after) = b.last_named.clone() else { return };
    let before = match &t.target {
        T::Action(au) => {
            let type_name = (!au.value.type_name.is_empty()).then_some(au.value.type_name.as_str());
            push_perform_entry(b, &au.value.name, type_name)
        }
        T::Perform(p) => push_perform_entry(b, &p.value.action_name, p.value.type_name.as_deref()),
        T::Merge(m) => {
            let name = connection_end_display(&m.value.merge.value).unwrap_or_else(|| render_expression(&m.value.merge.value));
            b.last_named = Some(name.clone());
            name
        }
        T::Feature(expr) => {
            let name = connection_end_display(&expr.value).unwrap_or_else(|| render_expression(&expr.value));
            b.last_named = Some(name.clone());
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
        sysml_v2_parser::ActionDefBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::ActionDefBody::Semicolon => &[],
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
            E::ForkStmt(f) => handle_control_node(&mut b, &f.value.fork.value, "ForkNode"),
            E::JoinStmt(j) => handle_control_node(&mut b, &j.value.join.value, "JoinNode"),
            E::DecisionStmt(d) => handle_control_node(&mut b, &d.value.decide.value, "DecisionNode"),
            E::MergeStmt(m) => handle_control_node(&mut b, &m.value.merge.value, "MergeNode"),
            E::FirstStmt(f) => handle_first_stmt(&mut b, &f.value),
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
            E::ForkStmt(f) => handle_control_node(&mut b, &f.value.fork.value, "ForkNode"),
            E::JoinStmt(j) => handle_control_node(&mut b, &j.value.join.value, "JoinNode"),
            E::DecisionStmt(d) => handle_control_node(&mut b, &d.value.decide.value, "DecisionNode"),
            E::MergeStmt(m) => handle_control_node(&mut b, &m.value.merge.value, "MergeNode"),
            E::FirstStmt(f) => handle_first_stmt(&mut b, &f.value),
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
        supertype: a.specializes.as_ref().map(|t| t.value.target_display()),
        ..Default::default()
    }
    .with_syscribe_meta(action_def_syscribe_meta(elements))
    .with_doc(action_def_doc(elements))
    .with_behavior(body.sub_actions, body.control_nodes, body.succession_connections);
    push_synth(out, &action_qname, file_path, ElementType::ActionDef, &name, spec);
    for node in elements {
        convert_action_def_body_element(&node.value, &action_qname, file_path, out);
    }
}

fn convert_action_usage(a: &sysml_v2_parser::ActionUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if a.name.is_empty() {
        return; // anonymous usage: no identity to qname against
    }
    let action_qname = format!("{qname}::{}", a.name);
    let elements = match &a.body {
        sysml_v2_parser::ActionUsageBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::ActionUsageBody::Semicolon => &[],
    };
    let body = build_action_usage_body(elements);
    let spec = Spec {
        typed_by: (!a.type_name.is_empty()).then(|| a.type_name.clone()),
        is_variation: a.is_variation.then_some(true),
        ..Default::default()
    }
    .with_syscribe_meta(action_usage_syscribe_meta(elements))
    .with_doc(action_usage_doc(elements))
    .with_behavior(body.sub_actions, body.control_nodes, body.succession_connections);
    push_synth(out, &action_qname, file_path, ElementType::Action, &a.name, spec);
    for node in elements {
        convert_action_usage_body_element(&node.value, &action_qname, file_path, out);
    }
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
        sysml_v2_parser::ast::ViewDefBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::ast::ViewDefBody::Semicolon => &[],
    };
    let spec = Spec {
        supertype: v.specializes.as_ref().map(|t| t.value.target_display()),
        ..Default::default()
    }
    .with_doc(view_def_doc(elements))
    .with_view(Vec::new(), None, view_def_rendering(elements));
    push_synth(out, &view_qname, file_path, ElementType::ViewDef, &name, spec);
}

/// `REQ-TRS-SYSMLV2-020` — a `view` usage synthesizes a real `View`, the
/// only place `expose:`/`viewpoint:` can actually be lifted from per this
/// grammar (see [`convert_view_def`]'s note). No recursion: none of
/// `ViewBodyElement`'s variants produce a further, separate `RawElement`.
fn convert_view_usage(v: &sysml_v2_parser::ast::ViewUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if v.name.is_empty() {
        return; // anonymous/redefinition-only usage: no identity to qname against
    }
    let view_qname = format!("{qname}::{}", v.name);
    let elements = match &v.body {
        sysml_v2_parser::ast::ViewBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::ast::ViewBody::Semicolon => &[],
    };
    let spec = Spec {
        typed_by: v.type_name.clone(),
        ..Default::default()
    }
    .with_doc(view_usage_doc(elements))
    .with_view(
        view_expose_entries(elements),
        view_satisfy_viewpoint(elements),
        view_usage_rendering(elements),
    );
    push_synth(out, &view_qname, file_path, ElementType::View, &v.name, spec);
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
        supertype: v.specializes.as_ref().map(|t| t.value.target_display()),
        ..Default::default()
    }
    .with_doc(requirement_def_body_doc(&v.body))
    .with_stakeholders_concerns(stakeholders, concerns);
    push_synth(out, &vp_qname, file_path, ElementType::ViewpointDef, &name, spec);
}

/// `REQ-TRS-SYSMLV2-021` — a `viewpoint` usage synthesizes a real `View`.
/// No dedicated `Viewpoint` usage `ElementType` exists in the native schema
/// — this maps onto `ElementType::View`, matching the doc's own framing of
/// `View` as "usage of a ViewDef or ViewpointDef".
fn convert_viewpoint_usage(v: &sysml_v2_parser::ast::ViewpointUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if v.name.is_empty() {
        return;
    }
    let vp_qname = format!("{qname}::{}", v.name);
    let (stakeholders, concerns) = collect_requirement_body_stakeholders_concerns(&v.body);
    let spec = Spec {
        // `ViewpointUsage.type_name` is a non-`Option<String>` (empty-string
        // sentinel for "untyped"), unlike `ViewUsage.type_name`'s
        // `Option<String>` — treat "" as absent.
        typed_by: (!v.type_name.is_empty()).then(|| v.type_name.clone()),
        ..Default::default()
    }
    .with_doc(requirement_def_body_doc(&v.body))
    .with_stakeholders_concerns(stakeholders, concerns);
    push_synth(out, &vp_qname, file_path, ElementType::View, &v.name, spec);
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
    if c.name.is_empty() {
        return; // anonymous concern/concern def: no identity to qname against
    }
    let concern_qname = format!("{qname}::{}", c.name);
    let (stakeholders, _) = collect_requirement_body_stakeholders_concerns(&c.body);
    let ty = if c.is_definition { ElementType::ConcernDef } else { ElementType::Concern };
    let spec = Spec {
        supertype: c.is_definition.then(|| c.type_name.clone()).flatten(),
        typed_by: (!c.is_definition).then(|| c.type_name.clone()).flatten(),
        subject: concern_body_subject(&c.body),
        ..Default::default()
    }
    .with_doc(requirement_def_body_doc(&c.body))
    // `ConcernDef` has no `concerns:` self-field (§8.11.5) -- only the
    // stakeholders half of the tuple is used; `requires:`/`assume:`/
    // `parameters:` are explicitly out of scope for this requirement (see
    // `REQ-TRS-SYSMLV2-023`'s Scope section).
    .with_stakeholders_concerns(stakeholders, Vec::new());
    push_synth(out, &concern_qname, file_path, ty, &c.name, spec);
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
        sysml_v2_parser::ast::RenderingDefBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::ast::RenderingDefBody::Semicolon => &[],
    };
    let spec = Spec {
        supertype: r.specializes.as_ref().map(|t| t.value.target_display()),
        ..Default::default()
    }
    .with_doc(rendering_def_doc(elements));
    push_synth(out, &rendering_qname, file_path, ElementType::RenderingDef, &name, spec);
}

/// `REQ-TRS-SYSMLV2-022` — a `rendering` usage synthesizes a real
/// `Rendering`. `RenderingUsageBodyElement::ViewUsage` (the narrow nested
/// `view :>> columnView[N] { render ...; }` redefinition shape, confirmed
/// against real SysML v2 standard-library fixtures) is deliberately not
/// recursed into — narrow, non-representative of ordinary modeling, and
/// there is no native "nested view" field to hold it.
fn convert_rendering_usage(r: &sysml_v2_parser::ast::RenderingUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if r.name.is_empty() {
        return;
    }
    let rendering_qname = format!("{qname}::{}", r.name);
    let elements = match &r.body {
        sysml_v2_parser::ast::RenderingUsageBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::ast::RenderingUsageBody::Semicolon => &[],
    };
    let spec = Spec {
        typed_by: r.type_name.clone(),
        ..Default::default()
    }
    .with_doc(rendering_usage_doc(elements));
    push_synth(out, &rendering_qname, file_path, ElementType::Rendering, &r.name, spec);
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
    r.target.iter().map(|t| t.value.to_display_string()).collect()
}

/// `REQ-TRS-SYSMLV2-043`: `alias <name> for <target>;` members of one merged package.
fn package_aliases(merged: &MergedPackage) -> Vec<serde_yaml::Value> {
    merged
        .body
        .iter()
        .filter_map(|(e, _)| match e {
            sysml_v2_parser::PackageBodyElement::AliasDef(a) => {
                let id = &a.value.identification;
                let (name, short) = match (&id.name, &id.short_name) {
                    (Some(n), s) => (n.clone(), s.clone()),
                    (None, Some(s)) => (s.clone(), None),
                    (None, None) => return None,
                };
                let mut m = serde_yaml::Mapping::new();
                m.insert(ykey("name"), ykey(&name));
                if let Some(s) = short {
                    m.insert(ykey("shortName"), ykey(&s));
                }
                m.insert(ykey("for"), ykey(&a.value.target.to_display_string()));
                Some(serde_yaml::Value::Mapping(m))
            }
            _ => None,
        })
        .collect()
}

fn convert_merged(merged: &MergedPackage, qname: &str, out: &mut Vec<RawElement>) {
    for (elem, file_path) in &merged.body {
        convert_package_body_element(elem, qname, file_path, out);
    }
    for (name, child) in &merged.children {
        let child_qname = format!("{qname}::{name}");
        let file_path = child.declared_in.as_deref().unwrap_or(qname);
        // `REQ-TRS-SYSMLV2-036`: package-level `doc` lifts onto the Package.
        let doc = child
            .body
            .iter()
            .filter_map(|(e, _)| match e {
                sysml_v2_parser::PackageBodyElement::Doc(d) => Some(d.value.text.trim()),
                _ => None,
            })
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");
        let spec = Spec { aliases: nonempty_vec(package_aliases(child)), ..Default::default() };
        push_synth(out, &child_qname, file_path, ElementType::Package, name, spec.with_doc(doc));
        convert_merged(child, &child_qname, out);
    }
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
        _ => {} // outside REQ-TRS-SYSMLV2-007's fixed set
    }
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
        sysml_v2_parser::PartDefBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::PartDefBody::Semicolon => &[],
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
        supertype: part.specializes.as_ref().map(|t| t.value.target_display()),
        is_variation: is_variation_prefix(&part.definition_prefix),
        satisfies,
        applies_when: part_def_syscribe_feature_id(elements),
        ..Default::default()
    }
    .with_syscribe_meta(part_def_syscribe_meta(elements))
    .with_doc(part_def_doc(elements))
    .with_connections(connections)
    .with_flow_connections(flow_connections);
    push_synth(out, &part_qname, file_path, ElementType::PartDef, &name, spec);
    push_connection_truncation_findings(out, file_path, truncations);
    push_connection_truncation_findings(out, file_path, flow_truncations);
    for node in elements {
        convert_part_def_body_element(&node.value, &part_qname, file_path, out);
    }
}

fn convert_part_usage(
    part: &sysml_v2_parser::PartUsage,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    if part.name.is_empty() {
        return; // anonymous usage: no identity to qname against
    }
    let part_qname = format!("{qname}::{}", part.name);
    let elements = match &part.body {
        sysml_v2_parser::PartUsageBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::PartUsageBody::Semicolon => &[],
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
        typed_by: (!part.type_name.is_empty()).then(|| part.type_name.clone()),
        is_variation: is_variation_prefix(&part.usage_prefix),
        satisfies,
        applies_when: part_usage_syscribe_feature_id(elements),
        ..Default::default()
    }
    .with_usage_relations(
        part.multiplicity.as_ref().map(|m| &m.value),
        part.subsets.as_ref().map(|(r, _)| &r.value),
        part.redefines.as_ref().map(|r| &r.value),
    )
    .with_syscribe_meta(part_usage_syscribe_meta(elements))
    .with_doc(part_usage_doc(elements))
    .with_connections(connections)
    .with_flow_connections(flow_connections);
    push_synth(out, &part_qname, file_path, ElementType::Part, &part.name, spec);
    push_connection_truncation_findings(out, file_path, truncations);
    push_connection_truncation_findings(out, file_path, flow_truncations);
    for node in elements {
        convert_part_usage_body_element(&node.value, &part_qname, file_path, out);
    }
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
        _ => {} // outside REQ-TRS-SYSMLV2-007's fixed set
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
        // No `ViewDef`/`ViewUsage`/`ViewpointDef`/`ViewpointUsage`/
        // `RenderingDef`/`RenderingUsage` variant exists in this enum at all
        // -- the whole view/viewpoint/rendering family cannot be declared
        // directly inside a `part` usage body per this grammar
        // (`REQ-TRS-SYSMLV2-020`/`-021`/`-022`), mirroring this same enum's
        // pre-existing absence of `PartDef`/`AllocationUsage`/`ActionDef`.
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
        _ => {} // outside REQ-TRS-SYSMLV2-007's fixed set
    }
}

fn is_variation_prefix(prefix: &Option<sysml_v2_parser::ast::DefinitionPrefix>) -> Option<bool> {
    matches!(prefix, Some(sysml_v2_parser::ast::DefinitionPrefix::Variation)).then_some(true)
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
            E::LiteralReal(s) => serde_yaml::Value::Number(s.parse::<f64>().ok().map(serde_yaml::Number::from)?),
            E::LiteralString(s) => serde_yaml::Value::String(s.clone()),
            E::LiteralBoolean(b) => serde_yaml::Value::Bool(*b),
            _ => return None,
        })
    };
    match e {
        E::LiteralWithUnit { value, unit } => {
            // The parser wraps the bracketed unit expression in `Bracket(..)`.
            let inner = match &unit.value {
                E::Bracket(b) => &b.value,
                other => other,
            };
            let u = match inner {
                E::FeatureRef(s) => s.clone(),
                E::FeatureChainRef(c) => c.segments.join("::"),
                // `REQ-TRS-SYSMLV2-065`: a spaced compound (`N * m`) lexes as an operator
                // expression; keep its whitespace-free text rather than dropping the unit.
                other => compound_unit_text(other)?,
            };
            Some((scalar(&value.value)?, Some(u)))
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
    if a.name.is_empty() {
        return;
    }
    let elem_qname = format!("{qname}::{}", a.name);
    let elements = match &a.body {
        sysml_v2_parser::AttributeBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::AttributeBody::Semicolon => &[],
    };
    // AttributeDef's `:>` specialization target is (inconsistently, upstream)
    // named `typing` rather than `specializes` like the other Def structs, but
    // it's the same semantic — a Def's supertype, not a Usage's typed-by.
    let spec = Spec {
        supertype: a.typing.as_ref().map(|t| t.value.target_display()),
        ..Default::default()
    }
    .with_doc(attribute_body_doc(elements));
    push_synth(out, &elem_qname, file_path, ElementType::AttributeDef, &a.name, spec);
}

fn convert_attribute_usage(
    a: &sysml_v2_parser::AttributeUsage,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    if a.name.is_empty() {
        return;
    }
    let elem_qname = format!("{qname}::{}", a.name);
    let elements = match &a.body {
        sysml_v2_parser::AttributeBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::AttributeBody::Semicolon => &[],
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
        typed_by: a.typing.as_ref().map(|t| t.value.target_display()),
        value,
        unit,
        ..Default::default()
    }
    .with_usage_relations(None, a.subsets.as_ref().map(|r| &r.value), a.redefines.as_ref().map(|r| &r.value))
    .with_doc(attribute_body_doc(elements));
    push_synth(out, &elem_qname, file_path, ElementType::Attribute, &a.name, spec);
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
        sysml_v2_parser::PortDefBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::PortDefBody::Semicolon => &[],
    };
    // REQ-TRS-SYSMLV2-014: PortDefBodyElement has no MetadataAnnotation
    // variant, so @Syscribe* fields reach a port def only via a doc-comment
    // directive, extracted from the already-lifted doc text.
    let (doc, meta) = extract_syscribe_doc_directives(&port_def_doc(elements));
    let spec = Spec {
        supertype: p.specializes.as_ref().map(|t| t.value.target_display()),
        ..Default::default()
    }
    .with_doc(doc)
    .with_syscribe_meta(meta);
    push_synth(out, &elem_qname, file_path, ElementType::PortDef, &name, spec);
    for node in elements {
        convert_port_def_body_element(&node.value, &elem_qname, file_path, out);
    }
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
        _ => {} // outside REQ-TRS-SYSMLV2-007's fixed set
    }
}

fn convert_port_usage(
    p: &sysml_v2_parser::PortUsage,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    if p.name.is_empty() {
        return;
    }
    let elem_qname = format!("{qname}::{}", p.name);
    // Unlike every other Usage's body in this module, `p.body` was never
    // read here at all before `REQ-TRS-SYSMLV2-009` — its nested
    // `AttributeUsage`/`ItemUsage` members stay unmapped exactly as before
    // (out of scope for this requirement, which is doc-lifting only); only
    // the `doc` extraction is new.
    let elements = match &p.body {
        sysml_v2_parser::PortBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::PortBody::Semicolon => &[],
    };
    let spec = Spec {
        typed_by: p.type_name.clone(),
        ..Default::default()
    }
    .with_usage_relations(
        p.multiplicity.as_ref().map(|m| &m.value),
        p.subsets.as_ref().map(|(r, _)| &r.value),
        p.redefines.as_ref().map(|r| &r.value),
    )
    .with_doc(port_usage_doc(elements));
    push_synth(out, &elem_qname, file_path, ElementType::Port, &p.name, spec);
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
        sysml_v2_parser::ConnectionDefBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::ConnectionDefBody::Semicolon => &[],
    };
    // REQ-TRS-SYSMLV2-014: ConnectionDefBodyElement has no MetadataAnnotation
    // variant, so @Syscribe* fields reach a connection def only via a
    // doc-comment directive, extracted from the already-lifted doc text.
    let (doc, meta) = extract_syscribe_doc_directives(&connection_def_doc(elements));
    let spec = Spec {
        supertype: c.specializes.as_ref().map(|t| t.value.target_display()),
        ..Default::default()
    }
    .with_doc(doc)
    .with_syscribe_meta(meta);
    push_synth(out, &elem_qname, file_path, ElementType::ConnectionDef, &name, spec);
    for node in elements {
        convert_connection_def_body_element(&node.value, &elem_qname, file_path, out);
    }
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
    let Some(name) = c.name.clone().filter(|n| !n.is_empty()) else {
        return; // anonymous connection usage: no identity to qname against
    };
    let elem_qname = format!("{qname}::{name}");
    // c.body is the same ConnectionDefBody/ConnectionDefBodyElement shape
    // convert_connection_def already reads its own body through — reused
    // unchanged here, REQ-TRS-SYSMLV2-012 (the sibling usage-body doc lift
    // REQ-TRS-SYSMLV2-009 didn't reach).
    let doc = match &c.body {
        sysml_v2_parser::ConnectionDefBody::Brace { elements } => connection_def_doc(elements),
        sysml_v2_parser::ConnectionDefBody::Semicolon => String::new(),
    };
    let spec = Spec {
        typed_by: c.type_name.clone(),
        ..Default::default()
    }
    .with_doc(doc);
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
    let sysml_v2_parser::ast::DefinitionBody::Brace { elements } = body else {
        return String::new();
    };
    collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::DefinitionBodyElement::Doc(d) => Some(d.value.text.as_str()),
        sysml_v2_parser::ast::DefinitionBodyElement::OccurrenceMember(m) => match &m.value {
            sysml_v2_parser::ast::OccurrenceBodyElement::Doc(d) => Some(d.value.text.as_str()),
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
        supertype: f.specializes.as_ref().map(|t| t.value.target_display()),
        ..Default::default()
    }
    .with_doc(flow_body_doc(&f.body));
    push_synth(out, &flow_qname, file_path, ElementType::FlowDef, &name, spec);
}

/// `REQ-TRS-SYSMLV2-024` — a *named* `flow`/`message`/`succession flow`
/// usage synthesizes a real `Flow`. An anonymous one (`f.name` empty/`None`)
/// has no identity to qname against and stays invisible as its own
/// element — it is still, separately, scanned by `flow_usage_entry` into
/// the owning part's `flowConnections:` (see `part_def_flow_entries`),
/// mirroring `convert_connection_usage`'s identical dual pattern exactly.
fn convert_flow_usage(f: &sysml_v2_parser::FlowUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = f.name.clone().filter(|n| !n.is_empty()) else {
        return;
    };
    let flow_qname = format!("{qname}::{name}");
    let spec = Spec {
        item_type: flow_item_type(f),
        ..Default::default()
    }
    .with_doc(flow_body_doc(&f.body));
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
    let values = match &e.body {
        sysml_v2_parser::ast::EnumerationBody::Brace { values } => values
            .iter()
            .map(|v| {
                let mut m = serde_yaml::Mapping::new();
                m.insert(serde_yaml::Value::from("name"), serde_yaml::Value::from(v.value.name.clone()));
                serde_yaml::Value::Mapping(m)
            })
            .collect(),
        sysml_v2_parser::ast::EnumerationBody::Semicolon => Vec::new(),
    };
    let spec = Spec {
        supertype: e.specializes.as_ref().map(|t| t.value.target_display()),
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
    if e.name.is_empty() {
        return; // anonymous enum usage: no identity to qname against
    }
    let enum_qname = format!("{qname}::{}", e.name);
    let elements = match &e.body {
        sysml_v2_parser::AttributeBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::AttributeBody::Semicolon => &[],
    };
    let spec = Spec {
        typed_by: e.type_name.clone(),
        ..Default::default()
    }
    .with_doc(attribute_body_doc(elements));
    push_synth(out, &enum_qname, file_path, ElementType::Enumeration, &e.name, spec);
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
}

fn case_body_fields(body: &sysml_v2_parser::ast::UseCaseDefBody) -> CaseBodyFields {
    let sysml_v2_parser::ast::UseCaseDefBody::Brace { elements } = body else {
        return CaseBodyFields {
            subject: None,
            actors: None,
            objectives: None,
            result_type: None,
            includes: Vec::new(),
            doc: String::new(),
        };
    };
    let mut subject = None;
    let mut actors = Vec::new();
    let mut objectives = Vec::new();
    let mut result_type = None;
    let mut includes: Vec<String> = Vec::new();
    for n in elements {
        match &n.value {
            sysml_v2_parser::ast::UseCaseDefBodyElement::IncludeUseCase(i) => includes.push(i.value.name.clone()),
            sysml_v2_parser::ast::UseCaseDefBodyElement::ThenIncludeUseCase(t) => {
                includes.push(t.value.include.value.name.clone())
            }
            sysml_v2_parser::ast::UseCaseDefBodyElement::SubjectDecl(s) => {
                subject = subject.or_else(|| nonempty(s.value.type_name.clone()));
            }
            sysml_v2_parser::ast::UseCaseDefBodyElement::ActorUsage(a) => {
                if let Some(t) = nonempty(a.value.type_name.clone()) {
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
                if let Some(label) = nonempty(r.name.clone()).or_else(|| r.type_name.clone()) {
                    objectives.push(serde_yaml::Value::from(label));
                }
            }
            // Multiple `return` declarations are legal (real fixtures show up
            // to three in one `verification def`) -- first one with a type
            // wins, matching the native `result:` field's single-string shape.
            sysml_v2_parser::ast::UseCaseDefBodyElement::CaseReturnDecl(r) => {
                result_type = result_type.or_else(|| r.value.type_name.clone());
            }
            _ => {}
        }
    }
    let doc = collect_doc(elements, |e| match e {
        sysml_v2_parser::ast::UseCaseDefBodyElement::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    });
    CaseBodyFields {
        subject,
        actors: nonempty_vec(actors),
        objectives: nonempty_vec(objectives),
        result_type,
        includes,
        doc,
    }
}

fn convert_case_def(c: &sysml_v2_parser::CaseDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&c.identification) else {
        return; // anonymous case def: no identity to qname against
    };
    let case_qname = format!("{qname}::{name}");
    let fields = case_body_fields(&c.body);
    let spec = Spec {
        supertype: c.specializes.as_ref().map(|t| t.value.target_display()),
        is_abstract: c.is_abstract.then_some(true),
        subject: fields.subject,
        actors: fields.actors,
        objectives: fields.objectives,
        result_type: fields.result_type,
        ..Default::default()
    }
    .with_doc(fields.doc);
    push_synth(out, &case_qname, file_path, ElementType::CaseDef, &name, spec);
}

fn convert_case_usage(c: &sysml_v2_parser::CaseUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if c.name.is_empty() {
        return; // anonymous case usage: no identity to qname against
    }
    let case_qname = format!("{qname}::{}", c.name);
    let fields = case_body_fields(&c.body);
    let spec = Spec {
        typed_by: c.type_name.clone(),
        is_abstract: c.is_abstract.then_some(true),
        subject: fields.subject,
        actors: fields.actors,
        objectives: fields.objectives,
        result_type: fields.result_type,
        ..Default::default()
    }
    .with_doc(fields.doc);
    push_synth(out, &case_qname, file_path, ElementType::Case, &c.name, spec);
}

fn convert_analysis_case_def(a: &sysml_v2_parser::AnalysisCaseDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&a.identification) else {
        return;
    };
    let case_qname = format!("{qname}::{name}");
    let fields = case_body_fields(&a.body);
    let spec = Spec {
        supertype: a.specializes.as_ref().map(|t| t.value.target_display()),
        is_abstract: a.is_abstract.then_some(true),
        subject: fields.subject,
        actors: fields.actors,
        objectives: fields.objectives,
        result_type: fields.result_type,
        ..Default::default()
    }
    .with_doc(fields.doc);
    push_synth(out, &case_qname, file_path, ElementType::AnalysisCaseDef, &name, spec);
}

fn convert_analysis_case_usage(a: &sysml_v2_parser::AnalysisCaseUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if a.name.is_empty() {
        return;
    }
    let case_qname = format!("{qname}::{}", a.name);
    let fields = case_body_fields(&a.body);
    let spec = Spec {
        typed_by: a.type_name.clone(),
        is_abstract: a.is_abstract.then_some(true),
        subject: fields.subject,
        actors: fields.actors,
        objectives: fields.objectives,
        result_type: fields.result_type,
        ..Default::default()
    }
    .with_doc(fields.doc);
    push_synth(out, &case_qname, file_path, ElementType::AnalysisCase, &a.name, spec);
}

fn convert_verification_case_def(v: &sysml_v2_parser::VerificationCaseDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&v.identification) else {
        return;
    };
    let case_qname = format!("{qname}::{name}");
    let fields = case_body_fields(&v.body);
    let spec = Spec {
        supertype: v.specializes.as_ref().map(|t| t.value.target_display()),
        is_abstract: v.is_abstract.then_some(true),
        subject: fields.subject,
        actors: fields.actors,
        objectives: fields.objectives,
        result_type: fields.result_type,
        ..Default::default()
    }
    .with_doc(fields.doc);
    push_synth(out, &case_qname, file_path, ElementType::VerificationCaseDef, &name, spec);
}

fn convert_verification_case_usage(v: &sysml_v2_parser::VerificationCaseUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if v.name.is_empty() {
        return;
    }
    let case_qname = format!("{qname}::{}", v.name);
    let fields = case_body_fields(&v.body);
    let spec = Spec {
        typed_by: v.type_name.clone(),
        is_abstract: v.is_abstract.then_some(true),
        subject: fields.subject,
        actors: fields.actors,
        objectives: fields.objectives,
        result_type: fields.result_type,
        ..Default::default()
    }
    .with_doc(fields.doc);
    push_synth(out, &case_qname, file_path, ElementType::VerificationCase, &v.name, spec);
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
        sysml_v2_parser::InterfaceDefBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::InterfaceDefBody::Semicolon => &[],
    };
    // REQ-TRS-SYSMLV2-014: InterfaceDefBodyElement has no MetadataAnnotation
    // variant, so @Syscribe* fields reach an interface def only via a
    // doc-comment directive, extracted from the already-lifted doc text.
    let (doc, meta) = extract_syscribe_doc_directives(&interface_def_doc(elements));
    let spec = Spec {
        supertype: i.specializes.as_ref().map(|t| t.value.target_display()),
        ..Default::default()
    }
    .with_doc(doc)
    .with_syscribe_meta(meta);
    push_synth(out, &elem_qname, file_path, ElementType::InterfaceDef, &name, spec);
    for node in elements {
        convert_interface_def_body_element(&node.value, &elem_qname, file_path, out);
    }
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
        _ => {} // outside REQ-TRS-SYSMLV2-007's fixed set
    }
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
    if let sysml_v2_parser::InterfaceUsage::Declaration {
        name: Some(name),
        interface_type,
        body_elements,
        ..
    } = i
    {
        if name.is_empty() {
            return;
        }
        let elem_qname = format!("{qname}::{name}");
        let spec = Spec {
            typed_by: interface_type.clone(),
            ..Default::default()
        }
        .with_doc(interface_usage_doc(body_elements));
        push_synth(out, &elem_qname, file_path, ElementType::Interface, name, spec);
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
        sysml_v2_parser::AttributeBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::AttributeBody::Semicolon => &[],
    };
    let spec = Spec {
        supertype: i.specializes.as_ref().map(|t| t.value.target_display()),
        ..Default::default()
    }
    .with_doc(attribute_body_doc(elements));
    push_synth(out, &elem_qname, file_path, ElementType::ItemDef, &name, spec);
    for node in elements {
        convert_attribute_body_element(&node.value, &elem_qname, file_path, out);
    }
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
    if i.name.is_empty() {
        return; // anonymous redefinition form (`item :>> shape ...`): skip
    }
    let elem_qname = format!("{qname}::{}", i.name);
    // ItemUsage.body IS an AttributeBody, the same shared shape
    // attribute_body_doc already handles for AttributeDef/AttributeUsage/
    // ItemDef — a review caught an earlier claim in this module that
    // ItemUsage "carries no body field," which was wrong (confirmed against
    // the parser's own struct definition and its own item-usage-with-body
    // test coverage); doc-lifting was silently missing here as a result.
    let elements = match &i.body {
        sysml_v2_parser::AttributeBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::AttributeBody::Semicolon => &[],
    };
    let spec = Spec {
        typed_by: i.type_name.clone(),
        ..Default::default()
    }
    .with_usage_relations(
        i.multiplicity.as_ref().map(|m| &m.value),
        None,
        i.redefines.as_ref().map(|r| &r.value),
    )
    .with_doc(attribute_body_doc(elements));
    push_synth(out, &elem_qname, file_path, ElementType::Item, &i.name, spec);
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
        supertype: r.specializes.as_ref().map(|t| t.value.target_display()),
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
    .with_doc(requirement_def_body_doc(&r.body));
    push_synth(out, &elem_qname, file_path, ElementType::RequirementDef, &name, spec);
}

fn convert_requirement_usage(
    r: &sysml_v2_parser::RequirementUsage,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    if r.name.is_empty() {
        return;
    }
    let elem_qname = format!("{qname}::{}", r.name);
    let spec = Spec {
        typed_by: r.type_name.clone(),
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
    .with_doc(requirement_def_body_doc(&r.body));
    push_synth(out, &elem_qname, file_path, ElementType::Requirement, &r.name, spec);
}

/// `verify` targets nested directly inside a `requirement def`/`requirement`
/// body (`REQ-TRS-SYSMLV2-003`) — the only body context this parser version
/// recognizes the `verify` keyword in at all (see this task's report for the
/// judgment call this reflects).
fn requirement_verify_targets(body: &sysml_v2_parser::RequirementDefBody) -> Vec<String> {
    let sysml_v2_parser::RequirementDefBody::Brace { elements } = body else {
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
    let sysml_v2_parser::RequirementDefBody::Brace { elements } = body else {
        return None;
    };
    elements.iter().find_map(|n| match &n.value {
        sysml_v2_parser::RequirementDefBodyElement::MetadataAnnotation(m) => {
            syscribe_feature_id(&m.value)
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
        supertype: a.specializes.as_ref().map(|t| t.value.target_display()),
        ..Default::default()
    }
    .with_doc(flow_body_doc(&a.body));
    push_synth(out, &def_qname, file_path, ElementType::AllocationDef, &name, spec);
}

fn convert_allocation_usage(
    a: &sysml_v2_parser::AllocationUsage,
    qname: &str,
    file_path: &str,
    out: &mut Vec<RawElement>,
) {
    if a.name.is_empty() {
        return;
    }
    let elem_qname = format!("{qname}::{}", a.name);
    // REQ-TRS-SYSMLV2-029 (GH #144): lift the `allocate <source> to
    // <target>` clause. The endpoints stay raw text here (this pass has no
    // view of the rest of the model); `super::resolve_allocation_endpoints`
    // resolves them against the fully merged element list afterwards.
    let endpoint = |e: &Option<sysml_v2_parser::Node<sysml_v2_parser::Expression>>| {
        e.as_ref().and_then(|n| connection_end_display(&n.value)).map(|s| vec![s])
    };
    let spec = Spec {
        typed_by: a.type_name.clone(),
        allocated_from: endpoint(&a.source),
        allocated_to: endpoint(&a.target),
        ..Default::default()
    };
    push_synth(out, &elem_qname, file_path, ElementType::Allocation, &a.name, spec);
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
    if v.name.is_empty() {
        return;
    }
    let elem_qname = format!("{part_qname}::{}", v.name);
    let base_spec = || Spec {
        is_variant: Some(true),
        variant_of: Some(part_qname.to_string()),
        ..Default::default()
    };
    match &v.typed {
        None => {
            // Bare reference to an already-declared sibling usage: nothing to
            // synthesize here (see doc comment above).
        }
        Some(sysml_v2_parser::ast::VariantTypedUsage::Part(pu)) => {
            let mut spec = base_spec();
            spec.typed_by = nonempty(pu.value.type_name.clone());
            let mut truncations = Vec::new();
            if let sysml_v2_parser::PartUsageBody::Brace { elements } = &pu.value.body {
                spec.applies_when = part_usage_syscribe_feature_id(elements);
                let (connections, t) = part_usage_connection_entries(&elem_qname, elements);
                truncations = t;
                spec = spec
                    .with_syscribe_meta(part_usage_syscribe_meta(elements))
                    .with_doc(part_usage_doc(elements))
                    .with_connections(connections);
            }
            push_synth(out, &elem_qname, file_path, ElementType::Part, &v.name, spec);
            push_connection_truncation_findings(out, file_path, truncations);
        }
        Some(sysml_v2_parser::ast::VariantTypedUsage::Attribute(au)) => {
            let mut spec = base_spec();
            spec.typed_by = au.value.typing.as_ref().map(|t| t.value.target_display());
            if let sysml_v2_parser::AttributeBody::Brace { elements } = &au.value.body {
                spec = spec.with_doc(attribute_body_doc(elements));
            }
            push_synth(out, &elem_qname, file_path, ElementType::Attribute, &v.name, spec);
        }
        Some(sysml_v2_parser::ast::VariantTypedUsage::Item(iu)) => {
            let mut spec = base_spec();
            spec.typed_by = iu.value.type_name.clone();
            if let sysml_v2_parser::AttributeBody::Brace { elements } = &iu.value.body {
                spec = spec.with_doc(attribute_body_doc(elements));
            }
            push_synth(out, &elem_qname, file_path, ElementType::Item, &v.name, spec);
        }
        Some(sysml_v2_parser::ast::VariantTypedUsage::Port(pu)) => {
            let mut spec = base_spec();
            spec.typed_by = pu.value.type_name.clone();
            if let sysml_v2_parser::PortBody::Brace { elements } = &pu.value.body {
                spec = spec.with_doc(port_usage_doc(elements));
            }
            push_synth(out, &elem_qname, file_path, ElementType::Port, &v.name, spec);
        }
        Some(sysml_v2_parser::ast::VariantTypedUsage::Perform(_)) => {
            // A `perform` variant is action/behavior-shaped — not in
            // REQ-TRS-SYSMLV2-007's fixed mapped set at all, regardless of
            // variation status, so it's never dispatched to a `push_synth`
            // call here (unlike Part/Attribute/Item/Port above, whose *kind*
            // is mapped even when the specific instance carries no
            // `@SyscribeFeature`).
        }
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
}

fn constraint_body_fields(body: &sysml_v2_parser::ast::ConstraintDefBody) -> ConstraintBodyFields {
    use sysml_v2_parser::ast::ConstraintDefBodyElement as B;
    let sysml_v2_parser::ast::ConstraintDefBody::Brace { elements } = body else {
        return ConstraintBodyFields { parameters: None, expression: None, doc: String::new() };
    };
    let mut params = Vec::new();
    let mut exprs = Vec::new();
    for n in elements {
        match &n.value {
            B::InOutDecl(d) => params.push(parameter_entry(&d.value.name, &d.value.type_name, direction_str(d.value.direction))),
            B::Expression(e) => exprs.push(render_expression(&e.value)),
            _ => {}
        }
    }
    let doc = collect_doc(elements, |e| match e {
        B::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    });
    ConstraintBodyFields { parameters: nonempty_vec(params), expression: join_expressions(exprs), doc }
}

/// `REQ-TRS-SYSMLV2-052`: whether `text`, written as the single body statement
/// block of a `constraint def`/`calc def`, is read back by ingestion as exactly
/// the same expression text. The parser recovers from nearly any input, so
/// "parses" proves nothing; only an exact read-back makes the text safe to
/// export as source.
pub(crate) fn expression_round_trips(is_calc: bool, text: &str) -> bool {
    let kw = if is_calc { "calc" } else { "constraint" };
    let probe = format!("package P {{ {kw} def X {{\n{text}\n}} }}");
    let Ok(root) = sysml_v2_parser::parse(&probe) else { return false };
    let norm = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ");
    for n in &root.elements {
        let sysml_v2_parser::RootElement::Package(p) = &n.value else { continue };
        let sysml_v2_parser::PackageBody::Brace { elements } = &p.value.body else { continue };
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
        supertype: c.specializes.as_ref().map(|t| t.value.target_display()),
        parameters: fields.parameters,
        expression: fields.expression,
        ..Default::default()
    }
    .with_doc(fields.doc);
    push_synth(out, &format!("{qname}::{name}"), file_path, ElementType::ConstraintDef, &name, spec);
}

fn convert_constraint_usage(c: &sysml_v2_parser::ast::ConstraintUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if c.name.is_empty() {
        return;
    }
    let fields = constraint_body_fields(&c.body);
    let spec = Spec {
        typed_by: c.type_name.clone(),
        parameters: fields.parameters,
        expression: fields.expression,
        ..Default::default()
    }
    .with_doc(fields.doc);
    push_synth(out, &format!("{qname}::{}", c.name), file_path, ElementType::Constraint, &c.name, spec);
}

struct CalcBodyFields {
    parameters: Option<Vec<serde_yaml::Value>>,
    return_type: Option<String>,
    body: Option<String>,
    doc: String,
}

fn calc_body_fields(body: &sysml_v2_parser::ast::CalcDefBody) -> CalcBodyFields {
    use sysml_v2_parser::ast::CalcDefBodyElement as B;
    let sysml_v2_parser::ast::CalcDefBody::Brace { elements } = body else {
        return CalcBodyFields { parameters: None, return_type: None, body: None, doc: String::new() };
    };
    let mut params = Vec::new();
    let mut exprs = Vec::new();
    let mut return_type = None;
    for n in elements {
        match &n.value {
            B::InOutDecl(d) => params.push(parameter_entry(&d.value.name, &d.value.type_name, direction_str(d.value.direction))),
            B::ReturnDecl(r) => {
                // First return type wins (the native `returnType:` is one
                // string); every return also stays visible as a parameter.
                return_type = return_type.or_else(|| nonempty(r.value.type_name.clone()));
                params.push(parameter_entry(&r.value.name, &r.value.type_name, "return"));
            }
            B::Expression(e) => exprs.push(render_expression(&e.value)),
            _ => {}
        }
    }
    let doc = collect_doc(elements, |e| match e {
        B::Doc(d) => Some(d.value.text.as_str()),
        _ => None,
    });
    CalcBodyFields { parameters: nonempty_vec(params), return_type, body: join_expressions(exprs), doc }
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
    .with_doc(f.doc);
    push_synth(out, &format!("{qname}::{name}"), file_path, ElementType::CalculationDef, &name, spec);
}

fn convert_calc_usage(c: &sysml_v2_parser::ast::CalcUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&c.identification) else {
        return;
    };
    let f = calc_body_fields(&c.body);
    let spec = Spec {
        typed_by: c.type_name.clone(),
        body_language: f.body.as_ref().map(|_| "kerml".to_string()),
        parameters: f.parameters,
        return_type: f.return_type,
        body: f.body,
        ..Default::default()
    }
    .with_doc(f.doc);
    push_synth(out, &format!("{qname}::{name}"), file_path, ElementType::Calculation, &name, spec);
}

/// `REQ-TRS-SYSMLV2-045`: a package-level `metadata def` -> native `MetadataDef`.
fn convert_metadata_def(m: &sysml_v2_parser::ast::MetadataDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&m.identification) else {
        return;
    };
    let elements = match &m.body {
        sysml_v2_parser::AttributeBody::Brace { elements } => elements.as_slice(),
        sysml_v2_parser::AttributeBody::Semicolon => &[],
    };
    let spec = Spec {
        supertype: m.specializes.as_ref().map(|t| t.value.target_display()),
        is_abstract: m.is_abstract.then_some(true),
        ..Default::default()
    }
    .with_doc(attribute_body_doc(elements));
    push_synth(out, &format!("{qname}::{name}"), file_path, ElementType::MetadataDef, &name, spec);
}

fn convert_use_case_def(c: &sysml_v2_parser::ast::UseCaseDef, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    let Some(name) = ident_name(&c.identification) else {
        return;
    };
    let f = case_body_fields(&c.body);
    let spec = Spec {
        supertype: c.specializes.as_ref().map(|t| t.value.target_display()),
        is_abstract: c.is_abstract.then_some(true),
        subject: f.subject,
        actors: f.actors,
        objectives: f.objectives,
        result_type: f.result_type,
        includes: nonempty_vec(f.includes),
        ..Default::default()
    }
    .with_doc(f.doc);
    push_synth(out, &format!("{qname}::{name}"), file_path, ElementType::UseCaseDef, &name, spec);
}

fn convert_use_case_usage(c: &sysml_v2_parser::ast::UseCaseUsage, qname: &str, file_path: &str, out: &mut Vec<RawElement>) {
    if c.name.is_empty() {
        return;
    }
    let f = case_body_fields(&c.body);
    let spec = Spec {
        typed_by: c.type_name.clone(),
        is_abstract: c.is_abstract.then_some(true),
        subject: f.subject,
        actors: f.actors,
        objectives: f.objectives,
        result_type: f.result_type,
        includes: nonempty_vec(f.includes),
        ..Default::default()
    }
    .with_doc(f.doc);
    push_synth(out, &format!("{qname}::{}", c.name), file_path, ElementType::UseCase, &c.name, spec);
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
    let root = sysml_v2_parser::parse(&probe).ok()?;
    for n in &root.elements {
        let sysml_v2_parser::RootElement::Package(p) = &n.value else { continue };
        let sysml_v2_parser::PackageBody::Brace { elements } = &p.value.body else { continue };
        for e in elements {
            let built = match &e.value {
                sysml_v2_parser::PackageBodyElement::ActionDef(a) if !is_usage => {
                    build_action_def_body(action_def_body_elements(&a.value.body))
                }
                sysml_v2_parser::PackageBodyElement::ActionUsage(a) if is_usage => match &a.value.body {
                    sysml_v2_parser::ActionUsageBody::Brace { elements } => build_action_usage_body(elements),
                    sysml_v2_parser::ActionUsageBody::Semicolon => build_action_usage_body(&[]),
                },
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
    let root = sysml_v2_parser::parse(&probe).ok()?;
    for n in &root.elements {
        let sysml_v2_parser::RootElement::Package(p) = &n.value else { continue };
        let sysml_v2_parser::PackageBody::Brace { elements } = &p.value.body else { continue };
        for e in elements {
            if let sysml_v2_parser::PackageBodyElement::StateDef(s) = &e.value {
                let els = match &s.value.body {
                    sysml_v2_parser::ast::StateDefBody::Brace { elements } => elements.as_slice(),
                    sysml_v2_parser::ast::StateDefBody::Semicolon => &[],
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
