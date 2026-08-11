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
fn find_sysml_files(dir: &Path) -> Vec<PathBuf> {
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
    let mut merged = MergedPackage::default();
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
            Ok(root) => merge_root(&mut merged, root, &file_path),
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
    out
}

/// Merge one file's already-parsed root namespace into `target`. Only
/// `Package` is handled so far — `REQ-TRS-SYSMLV2-007`'s remaining fixed kinds
/// land in later commits; everything else (bare root members, `library
/// package`, `namespace`, imports) is silently invisible for now (parse-broad,
/// map-narrow — same posture the full mapping will keep for constructs outside
/// the fixed set).
fn merge_root(target: &mut MergedPackage, root: sysml_v2_parser::RootNamespace, file_path: &str) {
    for node in root.elements {
        if let sysml_v2_parser::RootElement::Package(pkg_node) = node.value {
            merge_package(target, pkg_node.value, file_path);
        }
    }
}

/// Merge one `package` declaration into `target.children`, combining with
/// whatever a same-named package already contributed (from this file or an
/// earlier one).
fn merge_package(target: &mut MergedPackage, pkg: sysml_v2_parser::Package, file_path: &str) {
    let Some(name) = ident_name(&pkg.identification) else {
        return; // anonymous package: no identity to qname or merge against
    };
    let is_new = !target.children.contains_key(&name);
    let entry = target.children.entry(name).or_default();
    if is_new {
        entry.declared_in = Some(file_path.to_string());
    }
    if let sysml_v2_parser::PackageBody::Brace { elements } = pkg.body {
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
                merge_package(target, inner.value, file_path);
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
}

fn push_synth(
    out: &mut Vec<RawElement>,
    qname: &str,
    file_path: &str,
    ty: ElementType,
    name: &str,
    spec: Spec,
) {
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
            ..Default::default()
        },
        doc: spec.doc,
        parse_issue: None,
        derived: Default::default(),
        derive_findings: Vec::new(),
    });
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

/// A `connect`-clause endpoint's dotted display text, e.g. `a` or `a.p1` —
/// `REQ-TRS-SYSMLV2-010`. A single, unchained name parses as
/// `Expression::FeatureRef`; a genuine multi-segment dotted chain parses as
/// `Expression::FeatureChainRef` (confirmed against the parser's own AST and
/// its own doc comments: `path_expression`, used for `connect` endpoints
/// specifically, produces `FeatureChainRef` rather than folding into nested
/// `MemberAccess` the way the general value-expression grammar's postfix `.`
/// chaining does). Other expression shapes aren't meaningful connect
/// endpoints and aren't mapped here, matching `feature_ref_string`'s
/// existing posture for `satisfy`/`verify` targets.
fn connection_end_display(expr: &sysml_v2_parser::Expression) -> Option<String> {
    match expr {
        sysml_v2_parser::Expression::FeatureRef(s) => Some(s.clone()),
        sysml_v2_parser::Expression::FeatureChainRef(chain) => Some(chain.segments.join(".")),
        _ => None,
    }
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

/// Walk the merged package tree, emitting `RawElement`s under `qname`.
fn convert_merged(merged: &MergedPackage, qname: &str, out: &mut Vec<RawElement>) {
    for (elem, file_path) in &merged.body {
        convert_package_body_element(elem, qname, file_path, out);
    }
    for (name, child) in &merged.children {
        let child_qname = format!("{qname}::{name}");
        let file_path = child.declared_in.as_deref().unwrap_or(qname);
        push_synth(out, &child_qname, file_path, ElementType::Package, name, Spec::default());
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
    let spec = Spec {
        supertype: part.specializes.as_ref().map(|t| t.value.target_display()),
        is_variation: is_variation_prefix(&part.definition_prefix),
        satisfies,
        applies_when: part_def_syscribe_feature_id(elements),
        ..Default::default()
    }
    .with_syscribe_meta(part_def_syscribe_meta(elements))
    .with_doc(part_def_doc(elements))
    .with_connections(connections);
    push_synth(out, &part_qname, file_path, ElementType::PartDef, &name, spec);
    push_connection_truncation_findings(out, file_path, truncations);
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
    let spec = Spec {
        typed_by: (!part.type_name.is_empty()).then(|| part.type_name.clone()),
        is_variation: is_variation_prefix(&part.usage_prefix),
        satisfies,
        applies_when: part_usage_syscribe_feature_id(elements),
        ..Default::default()
    }
    .with_syscribe_meta(part_usage_syscribe_meta(elements))
    .with_doc(part_usage_doc(elements))
    .with_connections(connections);
    push_synth(out, &part_qname, file_path, ElementType::Part, &part.name, spec);
    push_connection_truncation_findings(out, file_path, truncations);
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
        E::VariantUsage(node) => convert_variant_usage(&node.value, part_qname, file_path, out),
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
    let spec = Spec {
        typed_by: a.typing.as_ref().map(|t| t.value.target_display()),
        ..Default::default()
    }
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
    };
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
    };
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
    let spec = Spec {
        typed_by: a.type_name.clone(),
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
