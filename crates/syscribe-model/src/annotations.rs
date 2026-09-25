//! In-process comment-marker scanner for annotated-source ingestion
//! (`ADR-SYS-ANNOTATE-001`).
//!
//! A package `_index.md` declaring `annotationFormat: <label>` hands its
//! subtree to this module instead of the native Markdown+YAML parser. Unlike
//! [`crate::plugins`] (`ADR-SYS-PLUGIN-002`), there is no external process and
//! no `.syscribe.toml` indirection: `marker`/`include`/`exclude` live inline
//! on the same `_index.md`, and every file they select is scanned in-process
//! for comment blocks that themselves contain literal Syscribe frontmatter
//! YAML — the exact same grammar every native `.md` file's frontmatter uses,
//! so no second grammar exists anywhere in this module.
//!
//! This is the *push* counterpart of `§12.8`'s `implementedBy:` (a *pull*,
//! where the model reaches down into its source). A marker's element gets its
//! `implementedBy:` auto-filled with its own source location when the marker
//! doesn't set one explicitly (`W563`) — the marker's location already answers
//! "where is this implemented."
//!
//! Runs inside [`crate::walker::walk_model`] itself, immediately after
//! [`crate::plugins::apply_foreign_plugins`], so every consumer (CLI/MCP/LSP/
//! web server) picks up annotated elements for free with no per-call-site
//! wiring. A model with no package declaring `annotationFormat:` is
//! completely unaffected.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::derive::finding;
use crate::element::{ElementType, RawElement, RawFrontmatter};

/// A resolved `annotationFormat:`-marked package, ready to be scanned.
pub struct AnnotationPackage {
    /// The human-readable `annotationFormat:` tag — documentation only, not a
    /// lookup key (contrast `foreignFormat:`'s plugin alias).
    pub label: String,
    pub dir: PathBuf,
    pub qname: String,
    /// The owning `_index.md`'s path — every synthesized element's `file_path`
    /// starts relative to this package's directory.
    pub index_path: String,
    pub marker: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

/// `(code, location, message)` — same shape as [`crate::derive::Finding`],
/// spelled out locally since that type alias is `pub(crate)` in `derive`.
type ScanFinding = (&'static str, String, String);

/// Run every `annotationFormat:`-marked package's scan over `elements`,
/// replacing each package's native placeholder content with the elements
/// extracted from its source comments.
///
/// No-op when no package declares `annotationFormat:` (inert by default).
pub fn apply_annotation_scans(elements: &mut Vec<RawElement>, model_root: &Path) {
    // Only an `_index.md` (a package) can declare `annotationFormat:` — a
    // marker anywhere else is silently ignored here (same posture
    // `foreignFormat:`/`sysmlSubmodel:` already take: still visible in that
    // element's own frontmatter, just no special handling).
    //
    // Anchors are confirmed shallowest-first, mirroring
    // `plugins::apply_foreign_plugins`: an `annotationFormat:`-marked package
    // nested inside an already-claimed subtree is absorbed into the outer
    // package rather than escaping exclusion by claiming itself first.
    let mut candidates: Vec<(usize, PathBuf, String)> = elements
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            if !e.file_path.ends_with("_index.md") {
                return None;
            }
            e.frontmatter.annotation_format.as_ref()?;
            let dir = Path::new(&e.file_path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
            Some((i, dir, e.file_path.clone()))
        })
        .collect();
    candidates.sort_by_key(|(_, dir, _)| dir.components().count());

    let mut confirmed: Vec<(usize, AnnotationPackage)> = Vec::new();
    for (idx, dir, file_path) in candidates {
        if confirmed.iter().any(|(_, p)| under_dir(&file_path, &p.dir)) {
            continue; // inside an already-claimed subtree: absorbed, not a new anchor
        }
        let fm = &elements[idx].frontmatter;

        // W562: annotationFormat: is mutually exclusive with foreignFormat:/
        // sysmlSubmodel: on the same package — each mechanism claims the
        // whole subtree, and letting two claim it at once is ambiguous rather
        // than additive. The other mechanism still runs on this package
        // normally; only annotation scanning is skipped here.
        if fm.foreign_format.is_some() || fm.sysml_submodel == Some(true) {
            elements[idx].derive_findings.push(finding(
                "W562",
                &file_path,
                "annotationFormat: is set alongside foreignFormat:/sysmlSubmodel: on the \
                 same package; annotation scanning skipped for this package",
            ));
            continue;
        }

        confirmed.push((
            idx,
            AnnotationPackage {
                label: fm.annotation_format.clone().unwrap_or_default(),
                dir,
                qname: elements[idx].qualified_name.clone(),
                index_path: file_path,
                marker: fm.marker.clone().unwrap_or_default(),
                include: fm.include.clone().unwrap_or_default(),
                exclude: fm.exclude.clone().unwrap_or_default(),
            },
        ));
    }

    if confirmed.is_empty() {
        return;
    }

    let mut synthetic: Vec<RawElement> = Vec::new();
    let mut owned_dirs: Vec<(PathBuf, String)> = Vec::new();

    for (owner_idx, pkg) in &confirmed {
        let (elems, findings) = scan_package(pkg, model_root);
        let fatal = findings.iter().any(|(code, _, _)| *code == "E560");
        for (code, location, msg) in &findings {
            elements[*owner_idx]
                .derive_findings
                .push(finding(code, location, msg));
        }
        if fatal {
            // Config error: the subtree is not claimed at all, so whatever
            // native content is there (if any) is left alone rather than
            // silently dropped for a scan that never ran.
            continue;
        }

        owned_dirs.push((pkg.dir.clone(), pkg.index_path.clone()));
        synthetic.extend(elems);
    }

    // Strip native elements under each annotated package's directory — the
    // whole subtree is scan-owned — except the package's own `_index.md` anchor.
    if !owned_dirs.is_empty() {
        elements.retain(|e| {
            if owned_dirs.iter().any(|(_, anchor)| anchor == &e.file_path) {
                return true;
            }
            !owned_dirs.iter().any(|(dir, _)| under_dir(&e.file_path, dir))
        });
    }

    elements.extend(synthetic);
}

/// Scan one package's subtree: validate its `marker`/`include`, walk matching
/// files, extract marker blocks, and convert each into a [`RawElement`].
///
/// A malformed `marker`/empty `include` is the one fatal condition, reported
/// as a single `E560` with no elements. Every other failure is per-marker and
/// degrades gracefully: that one marker is dropped (`E561`/`W560`/`W561`)
/// while its siblings are still processed.
pub fn scan_package(pkg: &AnnotationPackage, model_root: &Path) -> (Vec<RawElement>, Vec<ScanFinding>) {
    if pkg.marker.trim().is_empty() || pkg.include.is_empty() {
        return (
            Vec::new(),
            vec![(
                "E560",
                pkg.index_path.clone(),
                "annotationFormat: requires a non-empty marker: regex and a non-empty \
                 include: glob list"
                    .to_string(),
            )],
        );
    }
    let marker_re = match Regex::new(&pkg.marker) {
        Ok(r) => r,
        Err(e) => {
            return (
                Vec::new(),
                vec![(
                    "E560",
                    pkg.index_path.clone(),
                    format!("marker: is not a valid regex: {e}"),
                )],
            );
        }
    };

    // `pkg.dir` is already a real, walkable filesystem path (derived from a
    // `RawElement`'s `file_path`, which `walker::walk_model` builds by
    // joining `model_root` in — same as `plugins::PluginPackage::dir`), not
    // model-root-relative — used directly, no extra `model_root.join`.
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in walkdir::WalkDir::new(&pkg.dir).follow_links(false).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix(&pkg.dir).unwrap_or(entry.path());
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let included = pkg.include.iter().any(|g| glob_match(g, &rel_str));
        let excluded = pkg.exclude.iter().any(|g| glob_match(g, &rel_str));
        if included && !excluded {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();

    let mut elems: Vec<RawElement> = Vec::new();
    let mut findings: Vec<ScanFinding> = Vec::new();

    for file in &files {
        let Ok(content) = std::fs::read_to_string(file) else { continue };
        let rel_model = file
            .strip_prefix(model_root)
            .unwrap_or(file)
            .to_string_lossy()
            .replace('\\', "/");

        for block in extract_marker_blocks(&content, &marker_re) {
            let location = format!("{rel_model}:{}", block.line);
            match convert_block(&pkg.qname, &rel_model, &block.text) {
                Ok(mut elem) => {
                    if elem.frontmatter.implemented_by.is_none() {
                        elem.frontmatter.implemented_by = Some(vec![rel_model.clone()]);
                        findings.push((
                            "W563",
                            location,
                            format!(
                                "implementedBy: auto-filled with source location for '{}'",
                                elem.qualified_name
                            ),
                        ));
                    }
                    elems.push(elem);
                }
                Err((code, msg)) => findings.push((code, location, msg)),
            }
        }
    }

    (elems, findings)
}

struct MarkerBlock {
    /// 1-indexed line number of the marker's start line, for diagnostics.
    line: usize,
    text: String,
}

/// Find every marker in `content`. `marker_re` locates the start line; the
/// text preceding the match, plus any leading non-alphanumeric run *within*
/// the match itself (typically a comment token, e.g. `"//"`), becomes the
/// required prefix for contiguous continuation lines — covering both common
/// ways to write `marker` (`//\s*@syscribe`, leader inside the pattern, or
/// bare `@syscribe`, leader outside it). Continuation lines fold into the
/// same block until the first line that doesn't start with that exact
/// prefix. A marker with no real comment-prefix to key on (e.g. matching at
/// column 0 with no leader characters at all) accumulates no continuation
/// lines, to avoid an empty prefix silently swallowing the rest of the file.
fn extract_marker_blocks(content: &str, marker_re: &Regex) -> Vec<MarkerBlock> {
    let lines: Vec<&str> = content.lines().collect();
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let Some(m) = marker_re.find(line) else {
            i += 1;
            continue;
        };
        // The continuation prefix is the text before the match plus any
        // leading non-alphanumeric run *within* the match itself (typically
        // the comment token, e.g. "//"/"# "/"--"). This covers both common
        // ways to write `marker`: one that includes the comment leader in the
        // pattern (`//\s*@syscribe`, so the leader is part of the match), and
        // one that names only the distinctive token (`@syscribe`, so the
        // leader is the literal text preceding the match on the line).
        let match_text = &line[m.start()..m.end()];
        let leader_len = match_text
            .char_indices()
            .take_while(|(_, c)| !c.is_alphanumeric() && *c != '@')
            .map(|(i, c)| i + c.len_utf8())
            .last()
            .unwrap_or(0);
        let prefix = format!("{}{}", &line[..m.start()], &match_text[..leader_len]);

        let mut content_lines: Vec<String> = Vec::new();
        let first_rest = line[m.end()..].trim();
        if !first_rest.is_empty() {
            content_lines.push(first_rest.to_string());
        }
        let start_line_no = i + 1;
        let mut j = i + 1;
        if !prefix.trim().is_empty() {
            while j < lines.len() {
                // A line that itself starts a new marker ends the current
                // block even if it happens to also match `prefix` textually
                // (e.g. two adjacent markers sharing the same comment style)
                // — a new marker is never folded into its predecessor's block.
                if marker_re.is_match(lines[j]) {
                    break;
                }
                match lines[j].strip_prefix(prefix.as_str()) {
                    Some(stripped) => {
                        // Not `.trim_start()`-ed: relative indentation beyond
                        // `prefix` is significant YAML (multi-line block
                        // scalars for `doc:`, nested mappings/sequences).
                        content_lines.push(stripped.to_string());
                        j += 1;
                    }
                    None => break,
                }
            }
        }
        blocks.push(MarkerBlock {
            line: start_line_no,
            text: content_lines.join("\n"),
        });
        i = j.max(i + 1);
    }
    blocks
}

/// Parse one marker's accumulated text as literal frontmatter YAML and build
/// the [`RawElement`] it describes. Identity (and qname suffix) comes from
/// `id:` when present, else `name:` — exactly the identity rule each declared
/// `type:` already enforces everywhere else (an id-identified type requires
/// `id:`; a name-identified type's `name:` is both label and qname segment,
/// subject to the same basic-name grammar `W042` checks anywhere else).
///
/// `doc:` is a documented exception to "literal frontmatter, no second
/// grammar": `RawFrontmatter` has no `doc` field (a native file's doc body is
/// the Markdown *below* its frontmatter, not part of it) but a marker has no
/// equivalent separate region, so a top-level `doc:` key is read out of the
/// same block and becomes [`RawElement::doc`] — mirrors
/// [`crate::plugins::envelope::EnvelopeElement`]'s own separate `doc` field
/// for the identical reason. Removed before `RawFrontmatter` deserialization
/// (rather than left for it to ignore): `RawFrontmatter`'s `#[serde(flatten)]
/// extra` catch-all would otherwise capture it and `W047` ("unrecognized
/// frontmatter field") would fire on every marker that sets `doc:`.
fn convert_block(pkg_qname: &str, file_path: &str, text: &str) -> Result<RawElement, (&'static str, String)> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(("W561", "marker has no content".to_string()));
    }

    let mut yaml_value: serde_yaml::Value =
        serde_yaml::from_str(trimmed).map_err(|e| ("E561", format!("marker block is not valid YAML: {e}")))?;

    let doc = yaml_value
        .as_mapping_mut()
        .and_then(|m| m.remove(serde_yaml::Value::String("doc".to_string())))
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let fm: RawFrontmatter = serde_yaml::from_value(yaml_value)
        .map_err(|e| ("W560", format!("marker block is not a legal element: {e}")))?;

    match fm.element_type {
        None | Some(ElementType::Unknown) => {
            return Err(("W561", "marker has no type: (or an unrecognised one)".to_string()));
        }
        _ => {}
    }

    let suffix = fm.id.clone().or_else(|| fm.name.clone());
    let Some(suffix) = suffix else {
        return Err(("W561", "marker has no id:/name: to derive identity from".to_string()));
    };

    Ok(RawElement {
        qualified_name: format!("{pkg_qname}::{suffix}"),
        file_path: file_path.to_string(),
        frontmatter: fm,
        doc,
        parse_issue: None,
        derived: Default::default(),
        derive_findings: Vec::new(),
        locale_docs: Default::default(),
    })
}

/// True if `file_path` lies inside `dir` (component-wise, not a string prefix).
fn under_dir(file_path: &str, dir: &Path) -> bool {
    if dir.as_os_str().is_empty() {
        return false;
    }
    Path::new(file_path).starts_with(dir)
}

/// Minimal glob matcher for `include:`/`exclude:` (`*`, `**`, `?`; `/`-separated,
/// matched against a POSIX-style relative path). `**` matches across path
/// separators, including zero directories, so `**/*.c` matches both
/// `engine.c` and `drivers/engine.c` — the common "globstar" convention.
fn glob_match(pattern: &str, path: &str) -> bool {
    match glob_to_regex(pattern) {
        Some(re) => re.is_match(path),
        None => false,
    }
}

fn glob_to_regex(pattern: &str) -> Option<Regex> {
    let mut out = String::from("^");
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' if chars.get(i + 1) == Some(&'*') => {
                out.push_str(".*");
                i += 2;
                if chars.get(i) == Some(&'/') {
                    i += 1; // "**/​" also matches zero directories
                }
                continue;
            }
            '*' => out.push_str("[^/]*"),
            '?' => out.push_str("[^/]"),
            c @ ('.' | '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' | '[' | ']' | '\\') => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
        i += 1;
    }
    out.push('$');
    Regex::new(&out).ok()
}

/// The qualified names of every `RawElement` synthesized by annotation
/// scanning (`ADR-SYS-ANNOTATE-001`), for validator policy decisions gated on
/// actual scan origin rather than element kind alone
/// (`Resolver::is_verify_target`'s `E104` widening) — mirrors
/// [`crate::plugins::synthesized_qnames`]'s role and derivation exactly:
/// after [`apply_annotation_scans`] runs, every element whose qname is nested
/// under an `annotationFormat:`-declaring package's own qname (other than
/// that package itself) is guaranteed scan-synthesized, since the whole
/// subtree's native content was already stripped.
pub fn synthesized_qnames(elements: &[RawElement]) -> HashSet<String> {
    let pkg_qnames: Vec<&str> = elements
        .iter()
        .filter(|e| e.frontmatter.annotation_format.is_some())
        .map(|e| e.qualified_name.as_str())
        .collect();
    if pkg_qnames.is_empty() {
        return HashSet::new();
    }
    elements
        .iter()
        .filter(|e| {
            e.frontmatter.annotation_format.is_none()
                && pkg_qnames
                    .iter()
                    .any(|pkg| e.qualified_name.starts_with(pkg) && e.qualified_name[pkg.len()..].starts_with("::"))
        })
        .map(|e| e.qualified_name.clone())
        .collect()
}

/// Find the package whose `_index.md` declares `annotationFormat:`, matching
/// `selector` against the package's qualified name first, then (first match)
/// against its `annotationFormat:` label — scanning `_index.md` files
/// directly rather than running the full [`crate::walker::walk_model`]
/// pipeline (which would invoke [`apply_annotation_scans`] itself). Used
/// standalone by `syscribe annotations scan <selector> --dry-run`.
pub fn find_package(model_root: &Path, selector: &str) -> Option<AnnotationPackage> {
    let mut by_label: Option<AnnotationPackage> = None;
    for entry in walkdir::WalkDir::new(model_root).follow_links(false).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() || entry.file_name() != "_index.md" {
            continue;
        }
        let path = entry.path();
        let Ok(content) = std::fs::read_to_string(path) else { continue };
        let (fm_str, _body) = crate::frontmatter::split_frontmatter(&content);
        let Some(yaml) = fm_str else { continue };
        let Ok(fm) = crate::frontmatter::parse_frontmatter(yaml) else { continue };
        let Some(label) = fm.annotation_format.clone() else { continue };

        let rel = path.strip_prefix(model_root).unwrap_or(path);
        let qname = crate::walker::derive_qname(rel);
        let dir = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let pkg = AnnotationPackage {
            label: label.clone(),
            dir,
            qname: qname.clone(),
            index_path: path.display().to_string(),
            marker: fm.marker.clone().unwrap_or_default(),
            include: fm.include.clone().unwrap_or_default(),
            exclude: fm.exclude.clone().unwrap_or_default(),
        };
        if qname == selector {
            return Some(pkg);
        }
        if by_label.is_none() && label == selector {
            by_label = Some(pkg);
        }
    }
    by_label
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{ElementType as ET, RawFrontmatter};
    use std::sync::atomic::{AtomicU64, Ordering};

    fn tempdir() -> PathBuf {
        static N: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "syscribe-annotations-test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn index_elem(qname: &str, file_path: &str, fm: RawFrontmatter) -> RawElement {
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None,
            derived: Default::default(),
            derive_findings: Vec::new(),
            locale_docs: Default::default(),
        }
    }

    fn pkg_fm(label: &str, marker: &str, include: &[&str]) -> RawFrontmatter {
        RawFrontmatter {
            element_type: Some(ET::Package),
            annotation_format: Some(label.to_string()),
            marker: Some(marker.to_string()),
            include: Some(include.iter().map(|s| s.to_string()).collect()),
            ..Default::default()
        }
    }

    #[test]
    fn no_annotation_format_marker_is_a_no_op() {
        let root = tempdir();
        let mut elements = vec![index_elem(
            "Pkg",
            &root.join("Pkg/_index.md").display().to_string(),
            RawFrontmatter { element_type: Some(ET::Package), ..Default::default() },
        )];
        apply_annotation_scans(&mut elements, &root);
        assert_eq!(elements.len(), 1);
        assert!(elements[0].derive_findings.is_empty());
    }

    #[test]
    fn combined_with_foreign_format_is_w562_and_skipped() {
        let root = tempdir();
        let mut elements = vec![index_elem(
            "Pkg",
            &root.join("Pkg/_index.md").display().to_string(),
            RawFrontmatter {
                element_type: Some(ET::Package),
                annotation_format: Some("c".to_string()),
                marker: Some(r"//\s*@syscribe".to_string()),
                include: Some(vec!["**/*.c".to_string()]),
                foreign_format: Some("toydsl".to_string()),
                ..Default::default()
            },
        )];
        apply_annotation_scans(&mut elements, &root);
        let codes: Vec<&str> = elements[0].derive_findings.iter().map(|(c, _, _)| c.as_str()).collect();
        assert_eq!(codes, vec!["W562"]);
    }

    #[test]
    fn missing_marker_or_include_is_e560() {
        let root = tempdir();
        std::fs::create_dir_all(root.join("Pkg")).unwrap();
        let mut elements = vec![index_elem(
            "Pkg",
            &root.join("Pkg/_index.md").display().to_string(),
            RawFrontmatter {
                element_type: Some(ET::Package),
                annotation_format: Some("c".to_string()),
                ..Default::default()
            },
        )];
        apply_annotation_scans(&mut elements, &root);
        assert_eq!(elements.len(), 1, "no native content stripped when the config itself is fatal");
        let codes: Vec<&str> = elements[0].derive_findings.iter().map(|(c, _, _)| c.as_str()).collect();
        assert_eq!(codes, vec!["E560"]);
    }

    #[test]
    fn invalid_marker_regex_is_e560() {
        let root = tempdir();
        std::fs::create_dir_all(root.join("Pkg")).unwrap();
        let mut elements = vec![index_elem("Pkg", &root.join("Pkg/_index.md").display().to_string(), pkg_fm("c", "(unclosed", &["**/*.c"]))];
        apply_annotation_scans(&mut elements, &root);
        let codes: Vec<&str> = elements[0].derive_findings.iter().map(|(c, _, _)| c.as_str()).collect();
        assert_eq!(codes, vec!["E560"]);
    }

    #[test]
    fn well_formed_marker_synthesizes_element_and_autofills_implemented_by() {
        let root = tempdir();
        std::fs::create_dir_all(root.join("Firmware")).unwrap();
        std::fs::write(
            root.join("Firmware/engine.c"),
            "int x;\n// @syscribe\n// type: Part\n// name: EngineController\n// satisfies: [REQ-100]\nvoid f() {}\n",
        )
        .unwrap();
        let mut elements = vec![
            index_elem("Firmware", &root.join("Firmware/_index.md").display().to_string(), pkg_fm("c", r"//\s*@syscribe\b", &["**/*.c"])),
            // A stray native file under the annotated subtree — must be stripped.
            RawElement {
                qualified_name: "Firmware::Stray".to_string(),
                file_path: root.join("Firmware/Stray.md").display().to_string(),
                frontmatter: RawFrontmatter::default(),
                doc: String::new(),
                parse_issue: None,
                derived: Default::default(),
                derive_findings: Vec::new(),
                locale_docs: Default::default(),
            },
        ];
        apply_annotation_scans(&mut elements, &root);

        let qnames: Vec<&str> = elements.iter().map(|e| e.qualified_name.as_str()).collect();
        assert!(qnames.contains(&"Firmware"), "anchor _index.md survives");
        assert!(!qnames.contains(&"Firmware::Stray"), "native stray under the subtree is stripped");

        let elem = elements
            .iter()
            .find(|e| e.qualified_name == "Firmware::EngineController")
            .expect("marker-synthesized element merged in");
        assert_eq!(elem.frontmatter.element_type, Some(ET::Part));
        assert_eq!(elem.frontmatter.satisfies, Some(vec!["REQ-100".to_string()]));
        assert_eq!(
            elem.frontmatter.implemented_by,
            Some(vec!["Firmware/engine.c".to_string()]),
            "implementedBy: auto-filled from the marker's own source location"
        );

        let anchor = elements.iter().find(|e| e.qualified_name == "Firmware").unwrap();
        let codes: Vec<&str> = anchor.derive_findings.iter().map(|(c, _, _)| c.as_str()).collect();
        assert_eq!(codes, vec!["W563"]);
    }

    #[test]
    fn explicit_implemented_by_overrides_autofill_with_no_w563() {
        let root = tempdir();
        std::fs::create_dir_all(root.join("Firmware")).unwrap();
        std::fs::write(
            root.join("Firmware/engine.c"),
            "// @syscribe\n// type: Part\n// name: EngineController\n// implementedBy: [custom/path.c]\n",
        )
        .unwrap();
        let mut elements = vec![index_elem(
            "Firmware",
            &root.join("Firmware/_index.md").display().to_string(),
            pkg_fm("c", r"//\s*@syscribe\b", &["**/*.c"]),
        )];
        apply_annotation_scans(&mut elements, &root);

        let elem = elements.iter().find(|e| e.qualified_name == "Firmware::EngineController").unwrap();
        assert_eq!(elem.frontmatter.implemented_by, Some(vec!["custom/path.c".to_string()]));

        let anchor = elements.iter().find(|e| e.qualified_name == "Firmware").unwrap();
        assert!(anchor.derive_findings.is_empty(), "no auto-fill happened, so no W563");
    }

    #[test]
    fn malformed_yaml_is_e561_siblings_still_processed() {
        let root = tempdir();
        std::fs::create_dir_all(root.join("Firmware")).unwrap();
        std::fs::write(
            root.join("Firmware/engine.c"),
            "// @syscribe\n// type: Part\n// name: [unterminated\nint x;\n// @syscribe\n// type: Part\n// name: Good\n",
        )
        .unwrap();
        let mut elements = vec![index_elem(
            "Firmware",
            &root.join("Firmware/_index.md").display().to_string(),
            pkg_fm("c", r"//\s*@syscribe\b", &["**/*.c"]),
        )];
        apply_annotation_scans(&mut elements, &root);

        let qnames: Vec<&str> = elements.iter().map(|e| e.qualified_name.as_str()).collect();
        assert!(qnames.contains(&"Firmware::Good"), "well-formed sibling marker still processed");
        assert!(!qnames.contains(&"Firmware::[unterminated"));

        let anchor = elements.iter().find(|e| e.qualified_name == "Firmware").unwrap();
        let codes: Vec<&str> = anchor.derive_findings.iter().map(|(c, _, _)| c.as_str()).collect();
        assert!(codes.contains(&"E561"), "{codes:?}");
    }

    #[test]
    fn no_type_or_no_identity_is_w561() {
        let root = tempdir();
        std::fs::create_dir_all(root.join("Firmware")).unwrap();
        std::fs::write(
            root.join("Firmware/engine.c"),
            "// @syscribe\n// name: NoType\n// @syscribe\n// type: Part\n",
        )
        .unwrap();
        let mut elements = vec![index_elem(
            "Firmware",
            &root.join("Firmware/_index.md").display().to_string(),
            pkg_fm("c", r"//\s*@syscribe\b", &["**/*.c"]),
        )];
        apply_annotation_scans(&mut elements, &root);
        let anchor = elements.iter().find(|e| e.qualified_name == "Firmware").unwrap();
        let codes: Vec<&str> = anchor.derive_findings.iter().map(|(c, _, _)| c.as_str()).collect();
        assert_eq!(codes.iter().filter(|c| **c == "W561").count(), 2, "{codes:?}");
    }

    #[test]
    fn exclude_glob_wins_over_include() {
        let root = tempdir();
        std::fs::create_dir_all(root.join("Firmware/vendor")).unwrap();
        std::fs::write(root.join("Firmware/engine.c"), "// @syscribe\n// type: Part\n// name: Real\n").unwrap();
        std::fs::write(
            root.join("Firmware/vendor/lib.c"),
            "// @syscribe\n// type: Part\n// name: Vendored\n",
        )
        .unwrap();
        let mut elements = vec![index_elem(
            "Firmware",
            &root.join("Firmware/_index.md").display().to_string(),
            RawFrontmatter {
                exclude: Some(vec!["vendor/**".to_string()]),
                ..pkg_fm("c", r"//\s*@syscribe\b", &["**/*.c"])
            },
        )];
        apply_annotation_scans(&mut elements, &root);
        let qnames: Vec<&str> = elements.iter().map(|e| e.qualified_name.as_str()).collect();
        assert!(qnames.contains(&"Firmware::Real"));
        assert!(!qnames.contains(&"Firmware::Vendored"));
    }

    #[test]
    fn nested_annotation_format_marker_inside_a_claimed_subtree_is_absorbed_not_a_new_anchor() {
        let root = tempdir();
        std::fs::create_dir_all(root.join("Outer/Inner")).unwrap();
        let mut elements = vec![
            index_elem("Outer", &root.join("Outer/_index.md").display().to_string(), pkg_fm("c", "(unclosed", &["**/*.c"])),
            index_elem("Outer::Inner", &root.join("Outer/Inner/_index.md").display().to_string(), pkg_fm("c", "(unclosed", &["**/*.c"])),
        ];
        apply_annotation_scans(&mut elements, &root);
        let outer = elements.iter().find(|e| e.qualified_name == "Outer").unwrap();
        assert_eq!(outer.derive_findings.len(), 1);
        assert_eq!(outer.derive_findings[0].0, "E560");
        let inner = elements.iter().find(|e| e.qualified_name == "Outer::Inner").unwrap();
        assert!(inner.derive_findings.is_empty(), "absorbed, not scanned as its own anchor");
    }

    #[test]
    fn synthesized_qnames_tracks_scan_origin() {
        let elements = vec![
            index_elem("Firmware", "Firmware/_index.md", pkg_fm("c", r"//\s*@syscribe\b", &["**/*.c"])),
            index_elem(
                "Firmware::EngineController",
                "Firmware/engine.c",
                RawFrontmatter { element_type: Some(ET::Part), ..Default::default() },
            ),
            index_elem(
                "Other",
                "Other/Part.md",
                RawFrontmatter { element_type: Some(ET::Part), ..Default::default() },
            ),
        ];
        let set = synthesized_qnames(&elements);
        assert!(set.contains("Firmware::EngineController"));
        assert!(!set.contains("Firmware"));
        assert!(!set.contains("Other"));
    }

    #[test]
    fn doc_key_becomes_element_doc_and_is_ignored_by_frontmatter() {
        let root = tempdir();
        std::fs::create_dir_all(root.join("Firmware")).unwrap();
        std::fs::write(
            root.join("Firmware/engine.c"),
            "// @syscribe\n// type: Part\n// name: EngineController\n// doc: Holds the commanded target RPM.\n",
        )
        .unwrap();
        let mut elements = vec![index_elem(
            "Firmware",
            &root.join("Firmware/_index.md").display().to_string(),
            pkg_fm("c", r"//\s*@syscribe\b", &["**/*.c"]),
        )];
        apply_annotation_scans(&mut elements, &root);
        let elem = elements.iter().find(|e| e.qualified_name == "Firmware::EngineController").unwrap();
        assert_eq!(elem.doc, "Holds the commanded target RPM.");
        assert!(
            !elem.frontmatter.extra.contains_key("doc"),
            "doc: must be removed before RawFrontmatter deserialization, or W047 fires: {:?}",
            elem.frontmatter.extra
        );
    }

    #[test]
    fn glob_match_handles_globstar_and_extension() {
        assert!(glob_match("**/*.c", "engine.c"));
        assert!(glob_match("**/*.c", "drivers/engine.c"));
        assert!(!glob_match("**/*.c", "engine.h"));
        assert!(glob_match("vendor/**", "vendor/lib.c"));
        assert!(!glob_match("vendor/**", "src/lib.c"));
    }
}
