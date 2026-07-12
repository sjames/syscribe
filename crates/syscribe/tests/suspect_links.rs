//! Integration harness for suspect-link detection (REQ-TRS-SUS-LINKS-000..007,
//! verified by TC-TRS-SUS-LINKS-001..007 / 100 / 101).
//!
//! Black-box: each test builds a minimal, self-contained model in a fresh temp
//! directory and drives the `syscribe` binary against it. These tests are written
//! BEFORE the implementation exists — until `traceBaselines`, warning `W090`, and
//! the `suspect` subcommand land, the assertions that expect suspect detection
//! fail (the intended RED state). Pure "stays silent" assertions are already green
//! and must remain so (the opt-in / additive guarantee, REQ-TRS-SUS-LINKS-004/101).

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use common::dir_hash;

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A hash that cannot match any real BLAKE3 projection → guarantees a stale baseline.
const STALE: &str = "blake3:0000000000000000000000000000000000000000000000000000000000000000";

// ---- model construction -----------------------------------------------------

/// Fresh temp model root with a root `_index.md` package.
fn new_model() -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir()
        .join(format!("syscribe-suspect-{}-{}-{}", std::process::id(), nanos, n))
        .join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: SL\n---\n\nSuspect-link test model.\n");
    root
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

/// A minimal leaf Requirement (no `derivedFrom`, so no `breakdownAdr` needed).
fn write_req(root: &Path, id: &str, body: &str) {
    write(
        root,
        &format!("Requirements/{id}.md"),
        &format!(
            "---\ntype: Requirement\nid: {id}\nname: \"Req {id}\"\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\n{body}\n"
        ),
    );
}

/// An accepted ADR (needed as a `breakdownAdr` target for `derivedFrom` reqs).
fn write_adr(root: &Path, id: &str) {
    write(
        root,
        &format!("Decisions/{id}.md"),
        &format!("---\ntype: ADR\nid: {id}\nname: \"ADR {id}\"\nstatus: accepted\n---\n\nDecision.\n"),
    );
}

/// A child Requirement deriving from `parent`, with an optional baseline on that link.
fn write_child_req(root: &Path, id: &str, parent: &str, adr: &str, baseline: Option<&str>) {
    let bl = match baseline {
        Some(h) => format!("traceBaselines:\n  {parent}: \"{h}\"\n"),
        None => String::new(),
    };
    write(
        root,
        &format!("Requirements/{id}.md"),
        &format!(
            "---\ntype: Requirement\nid: {id}\nname: \"Req {id}\"\nstatus: draft\nreqDomain: software\nreqClass: system\nderivedFrom: [{parent}]\nbreakdownAdr: {adr}\n{bl}---\n\nChild requirement body.\n"
        ),
    );
}

/// A TestCase verifying `verifies`, optionally carrying a baseline for that target and/or
/// a `displayOrder` editorial field.
fn write_tc(root: &Path, id: &str, verifies: &str, baseline: Option<&str>, display_order: Option<i32>) {
    let bl = match baseline {
        Some(h) => format!("traceBaselines:\n  {verifies}: \"{h}\"\n"),
        None => String::new(),
    };
    let disp = match display_order {
        Some(d) => format!("displayOrder: {d}\n"),
        None => String::new(),
    };
    write(
        root,
        &format!("Verification/{id}.md"),
        &format!(
            "---\ntype: TestCase\nid: {id}\nname: \"TC {id}\"\nstatus: draft\ntestLevel: L2\n{disp}verifies:\n  - {verifies}\n{bl}---\n\n```gherkin\nFeature: {id}\n  Scenario: baseline\n    Given the model\n    Then the link holds\n```\n"
        ),
    );
}

/// Base model: one Requirement + one TestCase verifying it (optional baseline on the link).
fn base_pair(baseline: Option<&str>) -> PathBuf {
    let root = new_model();
    write_req(&root, "REQ-SL-001", "Original requirement body.");
    write_tc(&root, "TC-SL-001", "REQ-SL-001", baseline, None);
    root
}

// ---- binary drivers ---------------------------------------------------------

fn run(root: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn syscribe");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn validate(root: &Path, extra: &[&str]) -> (String, String, i32) {
    let mut a = vec!["validate"];
    a.extend_from_slice(extra);
    run(root, &a)
}

/// Combined stdout+stderr of a plain `validate`.
fn validate_out(root: &Path) -> String {
    let (o, e, _) = validate(root, &[]);
    format!("{o}\n{e}")
}

fn has_w090(root: &Path) -> bool {
    validate_out(root).contains("W090")
}

/// Lines of `validate` output that carry a W090 finding.
fn w090_lines(root: &Path) -> Vec<String> {
    validate_out(root)
        .lines()
        .filter(|l| l.contains("W090"))
        .map(|s| s.to_string())
        .collect()
}

// =============================================================================
// TC-TRS-SUS-LINKS-001 — traceBaselines is a recognized, optional schema field
// =============================================================================

#[test]
fn tracebaselines_is_a_recognized_field() {
    // A source carrying `traceBaselines` must NOT be flagged as an unrecognized
    // frontmatter field (W047). RED until the field is added to the schema.
    let root = base_pair(Some(STALE));
    let out = validate_out(&root);
    assert!(
        !out.contains("W047"),
        "traceBaselines must be a recognized field (no W047):\n{out}"
    );
}

#[test]
fn tracebaselines_is_optional() {
    // Sanity / additivity: a model without the field validates without errors.
    let root = base_pair(None);
    let (_o, _e, code) = validate(&root, &[]);
    assert_eq!(code, 0, "a model with no baselines validates clean");
}

// =============================================================================
// TC-TRS-SUS-LINKS-002 — projection excludes editorial fields; body/normative flips
// =============================================================================

#[test]
fn editing_excluded_field_keeps_link_current() {
    // Baseline the link, then change ONLY editorial/presentation fields on the target
    // (displayOrder, extRef) while leaving the body and normative fields intact. The
    // projection must exclude those fields → the link stays current (no W090).
    let root = base_pair(None);
    let _ = run(&root, &["suspect", "accept", "TC-SL-001", "REQ-SL-001"]);
    // Re-emit the target with excluded fields added but identical projected content.
    write(
        &root,
        "Requirements/REQ-SL-001.md",
        "---\ntype: Requirement\nid: REQ-SL-001\nname: \"Req REQ-SL-001\"\nstatus: draft\nreqDomain: software\nreqClass: system\ndisplayOrder: 99\nextRef: \"DNG:1\"\n---\n\nOriginal requirement body.\n",
    );
    assert!(
        !has_w090(&root),
        "changing only excluded editorial fields must not make the link suspect"
    );
}

#[test]
fn editing_target_body_makes_link_suspect() {
    // Baseline, then change the target's normative body → W090. RED until implemented.
    let root = base_pair(None);
    let _ = run(&root, &["suspect", "accept", "TC-SL-001", "REQ-SL-001"]);
    write_req(&root, "REQ-SL-001", "CHANGED requirement body — materially different.");
    assert!(
        has_w090(&root),
        "changing the target body must make the baselined link suspect (W090)"
    );
}

// =============================================================================
// TC-TRS-SUS-LINKS-003 — coverage across trace-link kinds (verifies + derivedFrom)
// =============================================================================

#[test]
fn verifies_link_can_be_suspect() {
    let root = base_pair(Some(STALE));
    assert!(has_w090(&root), "a stale baseline on a `verifies` link emits W090");
}

#[test]
fn derivedfrom_link_can_be_suspect() {
    let root = new_model();
    write_adr(&root, "ADR-SL-001");
    write_req(&root, "REQ-SL-PARENT-001", "Parent requirement.");
    write_child_req(&root, "REQ-SL-CHILD-001", "REQ-SL-PARENT-001", "ADR-SL-001", Some(STALE));
    assert!(has_w090(&root), "a stale baseline on a `derivedFrom` link emits W090");
}

// =============================================================================
// TC-TRS-SUS-LINKS-004 — W090 on stale; silent when unbaselined; gateable; unresolved≠W090
// =============================================================================

#[test]
fn stale_baseline_emits_w090_naming_source_and_target() {
    let root = base_pair(Some(STALE));
    let lines = w090_lines(&root);
    assert!(!lines.is_empty(), "stale baseline emits W090");
    let joined = lines.join("\n");
    assert!(joined.contains("TC-SL-001"), "W090 names the source:\n{joined}");
    assert!(joined.contains("REQ-SL-001"), "W090 names the target:\n{joined}");
}

#[test]
fn unbaselined_link_is_silent() {
    // Opt-in guarantee: no baseline → never W090. Green now, must stay green.
    let root = base_pair(None);
    assert!(!has_w090(&root), "an unbaselined link is never suspect");
}

#[test]
fn w090_is_gateable_with_deny() {
    let root = base_pair(Some(STALE));
    let (_o, _e, code) = validate(&root, &["--deny", "W090"]);
    assert_eq!(code, 2, "validate --deny W090 exits 2 when a suspect link exists");
}

#[test]
fn unresolved_baselined_target_is_not_w090() {
    // A baseline whose target does not resolve must not masquerade as a content mismatch.
    let root = new_model();
    write_tc(&root, "TC-SL-001", "REQ-DOES-NOT-EXIST", Some(STALE), None);
    assert!(!has_w090(&root), "an unresolvable target is not reported as W090");
}

// =============================================================================
// TC-TRS-SUS-LINKS-005 — suspect accept captures/refreshes baselines, clears the flag
// =============================================================================

#[test]
fn accept_writes_a_blake3_baseline() {
    let root = base_pair(None);
    let (_o, _e, code) = run(&root, &["suspect", "accept", "TC-SL-001", "REQ-SL-001"]);
    assert_eq!(code, 0, "suspect accept exits 0");
    let src = read(&root, "Verification/TC-SL-001.md");
    assert!(src.contains("traceBaselines"), "accept writes traceBaselines:\n{src}");
    assert!(src.contains("blake3:"), "the stored baseline is BLAKE3-prefixed:\n{src}");
}

#[test]
fn accept_clears_a_stale_flag() {
    let root = base_pair(Some(STALE));
    assert!(has_w090(&root), "precondition: stale baseline is suspect");
    let _ = run(&root, &["suspect", "accept", "TC-SL-001", "REQ-SL-001"]);
    assert!(!has_w090(&root), "after accept, the link is no longer suspect");
}

#[test]
fn accept_all_rebaselines_every_suspect_link() {
    let root = new_model();
    write_req(&root, "REQ-SL-001", "Body one.");
    write_req(&root, "REQ-SL-002", "Body two.");
    write_tc(&root, "TC-SL-001", "REQ-SL-001", Some(STALE), None);
    write_tc(&root, "TC-SL-002", "REQ-SL-002", Some(STALE), None);
    let (_o, _e, code) = run(&root, &["suspect", "accept", "--all"]);
    assert_eq!(code, 0, "suspect accept --all exits 0");
    assert!(!has_w090(&root), "accept --all clears every suspect link");
}

#[test]
fn accept_all_unbaselined_onboards_every_unbaselined_link() {
    // Onboarding mode: baseline every link that has no baseline yet, then clean.
    let root = new_model();
    write_req(&root, "REQ-SL-001", "Body one.");
    write_req(&root, "REQ-SL-002", "Body two.");
    write_tc(&root, "TC-SL-001", "REQ-SL-001", None, None);
    write_tc(&root, "TC-SL-002", "REQ-SL-002", None, None);
    let (_o, _e, code) = run(&root, &["suspect", "accept", "--all-unbaselined"]);
    assert_eq!(code, 0, "suspect accept --all-unbaselined exits 0");
    assert!(read(&root, "Verification/TC-SL-001.md").contains("traceBaselines"), "TC-SL-001 baselined");
    assert!(read(&root, "Verification/TC-SL-002.md").contains("traceBaselines"), "TC-SL-002 baselined");
    assert!(!has_w090(&root), "freshly onboarded links are current");
}

#[test]
fn accept_all_unbaselined_does_not_clear_existing_suspect() {
    // A stale (suspect) link must be left untouched: onboarding never overwrites an
    // existing baseline, so it cannot mask an outstanding suspect flag.
    let root = new_model();
    write_req(&root, "REQ-SL-001", "Body one.");
    write_req(&root, "REQ-SL-002", "Body two.");
    write_tc(&root, "TC-SL-001", "REQ-SL-001", Some(STALE), None); // already suspect
    write_tc(&root, "TC-SL-002", "REQ-SL-002", None, None); // unbaselined
    let (_o, _e, code) = run(&root, &["suspect", "accept", "--all-unbaselined"]);
    assert_eq!(code, 0, "onboarding exits 0");
    assert!(read(&root, "Verification/TC-SL-002.md").contains("traceBaselines"), "the unbaselined link is onboarded");
    // The stale link is still stale → W090 persists (not silently cleared).
    let lines = w090_lines(&root).join("\n");
    assert!(lines.contains("TC-SL-001"), "the pre-existing suspect link is left suspect:\n{lines}");
    assert!(!lines.contains("TC-SL-002"), "the onboarded link is not suspect:\n{lines}");
}

#[test]
fn accept_all_unbaselined_is_idempotent() {
    let root = base_pair(None);
    let _ = run(&root, &["suspect", "accept", "--all-unbaselined"]);
    let after_first = read(&root, "Verification/TC-SL-001.md");
    let (_o, _e, code) = run(&root, &["suspect", "accept", "--all-unbaselined"]);
    assert_eq!(code, 0, "a second onboarding run exits 0");
    assert_eq!(after_first, read(&root, "Verification/TC-SL-001.md"), "a second run baselines nothing further");
}

#[test]
fn accept_all_and_all_unbaselined_are_mutually_exclusive() {
    let root = base_pair(Some(STALE));
    let before = dir_hash(&root);
    let (_o, _e, code) = run(&root, &["suspect", "accept", "--all", "--all-unbaselined"]);
    assert_ne!(code, 0, "combining --all and --all-unbaselined is a usage error");
    assert_eq!(before, dir_hash(&root), "a rejected accept writes nothing");
}

#[test]
fn accept_preserves_frontmatter_verbatim() {
    // Surgical write: only a `traceBaselines` block is appended; every pre-existing
    // byte of the frontmatter (quoting, list indentation, body) is untouched.
    let root = new_model();
    write_req(&root, "REQ-SL-001", "Original requirement body.");
    let original = "---\ntype: TestCase\nid: TC-SL-001\nname: \"Quoted label — keep me\"\nstatus: draft\ntestLevel: L2\nverifies:\n  - REQ-SL-001\ntags:\n  - alpha\n  - beta\n---\n\n```gherkin\nFeature: x\n  Scenario: s\n    Given a\n    Then b\n```\n";
    write(&root, "Verification/TC-SL-001.md", original);
    let (_o, _e, code) = run(&root, &["suspect", "accept", "TC-SL-001", "REQ-SL-001"]);
    assert_eq!(code, 0, "accept exits 0");
    let after = read(&root, "Verification/TC-SL-001.md");
    // The block is appended just before the closing `---`, so the whole original
    // frontmatter (opener through the last field line) is preserved as a prefix.
    let head = &original[..original.find("\n---\n").unwrap()];
    assert!(after.starts_with(head), "original frontmatter preserved verbatim as a prefix:\n{after}");
    assert!(after.contains("name: \"Quoted label — keep me\""), "quoting preserved:\n{after}");
    assert!(after.contains("verifies:\n  - REQ-SL-001"), "list indentation preserved:\n{after}");
    assert!(after.contains("traceBaselines:\n  REQ-SL-001: \"blake3:"), "baseline block appended:\n{after}");
    assert!(after.ends_with("```\n"), "body preserved verbatim:\n{after}");
}

#[test]
fn accept_preserves_other_frontmatter() {
    let root = base_pair(None);
    let _ = run(&root, &["suspect", "accept", "TC-SL-001", "REQ-SL-001"]);
    let src = read(&root, "Verification/TC-SL-001.md");
    for field in ["type: TestCase", "id: TC-SL-001", "testLevel: L2", "verifies:"] {
        assert!(src.contains(field), "accept preserves `{field}`:\n{src}");
    }
}

#[test]
fn accept_nonreferenced_target_is_an_error() {
    let root = base_pair(None);
    let before = dir_hash(&root);
    let (_o, _e, code) = run(&root, &["suspect", "accept", "TC-SL-001", "REQ-NOT-LINKED"]);
    assert_ne!(code, 0, "accepting a target that isn't referenced is an error");
    assert_eq!(before, dir_hash(&root), "a failed accept writes nothing");
}

// =============================================================================
// TC-TRS-SUS-LINKS-006 — suspect list reports suspect + unbaselined, deterministic, read-only
// =============================================================================

#[test]
fn list_reports_suspect_links() {
    let root = base_pair(Some(STALE));
    let (o, _e, _c) = run(&root, &["suspect", "list"]);
    assert!(o.contains("TC-SL-001") && o.contains("REQ-SL-001"), "suspect list names source+target:\n{o}");
}

#[test]
fn list_is_deterministic() {
    let root = base_pair(Some(STALE));
    let (a, _, _) = run(&root, &["suspect", "list"]);
    let (b, _, _) = run(&root, &["suspect", "list"]);
    assert_eq!(a, b, "suspect list output is stable across runs");
}

#[test]
fn list_is_read_only() {
    let root = base_pair(Some(STALE));
    let before = dir_hash(&root);
    let _ = run(&root, &["suspect", "list"]);
    assert_eq!(before, dir_hash(&root), "suspect list mutates no model file");
}

// =============================================================================
// TC-TRS-SUS-LINKS-007 — implicit one-hop propagation, no eager flooding
// =============================================================================

#[test]
fn only_the_direct_link_is_flagged() {
    // Chain C -> A -> B via derivedFrom, every link baselined and current.
    let root = new_model();
    write_adr(&root, "ADR-SL-001");
    write_req(&root, "REQ-SL-B-001", "Leaf B body.");
    write_child_req(&root, "REQ-SL-A-001", "REQ-SL-B-001", "ADR-SL-001", None);
    write_child_req(&root, "REQ-SL-C-001", "REQ-SL-A-001", "ADR-SL-001", None);
    // Baseline both links to their current targets.
    let _ = run(&root, &["suspect", "accept", "REQ-SL-A-001", "REQ-SL-B-001"]);
    let _ = run(&root, &["suspect", "accept", "REQ-SL-C-001", "REQ-SL-A-001"]);
    assert!(!has_w090(&root), "precondition: all baselines current");

    // Change only leaf B. Only the A->B link may be suspect; C->A must NOT be.
    write_req(&root, "REQ-SL-B-001", "Leaf B body — CHANGED.");
    let lines = w090_lines(&root).join("\n");
    assert!(lines.contains("REQ-SL-A-001") && lines.contains("REQ-SL-B-001"), "A->B is suspect:\n{lines}");
    assert!(
        !lines.contains("REQ-SL-C-001"),
        "suspicion must not flood to C->A before A itself is edited:\n{lines}"
    );
}

// =============================================================================
// TC-TRS-SUS-LINKS-100 — full lifecycle (integration)
// =============================================================================

#[test]
fn lifecycle_baseline_change_detect_clear() {
    let root = base_pair(None);

    // 1. Baseline a reviewed link → clean.
    let _ = run(&root, &["suspect", "accept", "TC-SL-001", "REQ-SL-001"]);
    assert!(!has_w090(&root), "freshly baselined link is not suspect");

    // 2. Change the target → suspect, and `suspect list` shows it.
    write_req(&root, "REQ-SL-001", "Edited requirement text.");
    assert!(has_w090(&root), "changed target makes the link suspect");
    let (o, _e, _c) = run(&root, &["suspect", "list"]);
    assert!(o.contains("TC-SL-001"), "suspect list surfaces the suspect link:\n{o}");

    // 3. Re-accept → clean again.
    let _ = run(&root, &["suspect", "accept", "TC-SL-001", "REQ-SL-001"]);
    assert!(!has_w090(&root), "re-accepting clears the suspect flag");
}

// =============================================================================
// TC-TRS-SUS-LINKS-101 — feature is purely additive
// =============================================================================

#[test]
fn additive_unbaselined_model_has_no_w090() {
    // A model that has never baselined anything emits no W090, even after the target changes.
    let root = base_pair(None);
    assert!(!has_w090(&root), "no baselines → no W090");
    write_req(&root, "REQ-SL-001", "Changed body, but nothing was ever baselined.");
    assert!(!has_w090(&root), "detection only activates after `suspect accept`");
}

// =============================================================================
// Additional coverage — under-tested requirement clauses.
// =============================================================================

/// Write a leaf Requirement with an explicit `status` and `name`, but otherwise
/// identical to `write_req`. Lets a test vary exactly one projection input.
fn write_req_full(root: &Path, id: &str, name: &str, status: &str, body: &str) {
    write(
        root,
        &format!("Requirements/{id}.md"),
        &format!(
            "---\ntype: Requirement\nid: {id}\nname: \"{name}\"\nstatus: {status}\nreqDomain: software\nreqClass: system\n---\n\n{body}\n"
        ),
    );
}

// ── REQ-TRS-SUS-LINKS-002 — the projection includes normative frontmatter and
//    excludes editorial fields; canonicalization ignores cosmetic reformatting ──

#[test]
fn editing_normative_frontmatter_field_makes_link_suspect() {
    // The projection must include normative frontmatter (status/reqDomain/safety),
    // not just the body. Baseline while `status: draft`, then flip ONLY `status`
    // (body and name held identical) → the link must go suspect.
    let root = base_pair(None);
    let _ = run(&root, &["suspect", "accept", "TC-SL-001", "REQ-SL-001"]);
    assert!(!has_w090(&root), "precondition: freshly baselined link is current");
    write_req_full(
        &root,
        "REQ-SL-001",
        "Req REQ-SL-001",
        "approved", // was `draft`
        "Original requirement body.",
    );
    assert!(
        has_w090(&root),
        "changing a normative frontmatter field (status) must make the link suspect"
    );
}

#[test]
fn editing_only_name_keeps_link_current() {
    // `name` is an excluded (editorial) field: changing only the label — body,
    // status and every normative field held identical — must NOT flip suspect.
    let root = base_pair(None);
    let _ = run(&root, &["suspect", "accept", "TC-SL-001", "REQ-SL-001"]);
    write_req_full(
        &root,
        "REQ-SL-001",
        "A completely different human label", // only the name changed
        "draft",
        "Original requirement body.",
    );
    assert!(
        !has_w090(&root),
        "changing only the excluded `name` field must not make the link suspect"
    );
}

#[test]
fn crlf_reformatting_does_not_change_the_hash() {
    // Canonicalization normalizes line endings, so re-saving the target with CRLF
    // line endings but byte-identical logical content must not flip suspect.
    let root = base_pair(None);
    let _ = run(&root, &["suspect", "accept", "TC-SL-001", "REQ-SL-001"]);
    // Same content as `write_req` produces, but every `\n` becomes `\r\n`.
    let crlf = "---\r\ntype: Requirement\r\nid: REQ-SL-001\r\nname: \"Req REQ-SL-001\"\r\n\
                status: draft\r\nreqDomain: software\r\nreqClass: system\r\n---\r\n\r\n\
                Original requirement body.\r\n";
    write(&root, "Requirements/REQ-SL-001.md", crlf);
    assert!(
        !has_w090(&root),
        "CRLF-only reformatting of the target must not make the link suspect"
    );
}

// ── REQ-TRS-SUS-LINKS-003 — coverage across further trace-link kinds ──

#[test]
fn satisfies_link_can_be_suspect() {
    // A `satisfies` link from an architecture element carries a stale baseline → W090.
    let root = new_model();
    write_req(&root, "REQ-SL-001", "Requirement body.");
    write(
        &root,
        "Architecture/SatPart.md",
        &format!(
            "---\ntype: PartDef\nname: SatPart\ndomain: software\nsatisfies:\n  - REQ-SL-001\n\
             traceBaselines:\n  REQ-SL-001: \"{STALE}\"\n---\n\nA satisfying part.\n"
        ),
    );
    let lines = w090_lines(&root).join("\n");
    assert!(!lines.is_empty(), "a stale baseline on a `satisfies` link emits W090:\n{lines}");
    assert!(lines.contains("REQ-SL-001"), "W090 names the satisfied target:\n{lines}");
}

#[test]
fn supertype_link_can_be_suspect() {
    // A structural `supertype` link (a YAML scalar, not a list) carries a stale
    // baseline keyed by the supertype reference → W090.
    let root = new_model();
    write(&root, "Base.md", "---\ntype: PartDef\nname: Base\n---\n\nBase part.\n");
    write(
        &root,
        "Derived.md",
        &format!(
            "---\ntype: PartDef\nname: Derived\nsupertype: Base\n\
             traceBaselines:\n  Base: \"{STALE}\"\n---\n\nDerived part.\n"
        ),
    );
    let lines = w090_lines(&root).join("\n");
    assert!(!lines.is_empty(), "a stale baseline on a `supertype` link emits W090:\n{lines}");
    assert!(lines.contains("Base"), "W090 names the supertype target:\n{lines}");
}

#[test]
fn breakdown_adr_link_can_be_suspect() {
    // `breakdownAdr` is a scalar single-target trace link; a stale baseline on it → W090,
    // while the co-located unbaselined `derivedFrom` link stays silent.
    let root = new_model();
    write_adr(&root, "ADR-SL-001");
    write_req(&root, "REQ-SL-PARENT-001", "Parent requirement.");
    write(
        &root,
        "Requirements/REQ-SL-CHILD-001.md",
        &format!(
            "---\ntype: Requirement\nid: REQ-SL-CHILD-001\nname: \"Req child\"\nstatus: draft\n\
             reqDomain: software\nreqClass: system\nderivedFrom: [REQ-SL-PARENT-001]\n\
             breakdownAdr: ADR-SL-001\ntraceBaselines:\n  ADR-SL-001: \"{STALE}\"\n---\n\nChild.\n"
        ),
    );
    let lines = w090_lines(&root).join("\n");
    assert!(
        lines.contains("ADR-SL-001"),
        "a stale baseline on a `breakdownAdr` link emits W090 naming the ADR:\n{lines}"
    );
    assert!(
        !lines.contains("REQ-SL-PARENT-001"),
        "the co-located unbaselined derivedFrom link stays silent:\n{lines}"
    );
}

// ── REQ-TRS-SUS-LINKS-001 — a multi-valued link keeps a per-target baseline ──

#[test]
fn multivalued_link_baselines_each_target_independently() {
    // A TestCase verifying [A, B] baselines each target separately; changing only B
    // makes B's link suspect while A's stays current.
    let root = new_model();
    write_req(&root, "REQ-SL-001", "Body A.");
    write_req(&root, "REQ-SL-002", "Body B.");
    write(
        &root,
        "Verification/TC-SL-001.md",
        "---\ntype: TestCase\nid: TC-SL-001\nname: \"TC\"\nstatus: draft\ntestLevel: L2\n\
         verifies:\n  - REQ-SL-001\n  - REQ-SL-002\n---\n\n\
         ```gherkin\nFeature: multi\n  Scenario: s\n    Given x\n    Then y\n```\n",
    );
    // Baseline both links to their current targets.
    let _ = run(&root, &["suspect", "accept", "TC-SL-001", "REQ-SL-001"]);
    let _ = run(&root, &["suspect", "accept", "TC-SL-001", "REQ-SL-002"]);
    assert!(!has_w090(&root), "precondition: both per-target baselines current");

    // Change only B.
    write_req(&root, "REQ-SL-002", "Body B — CHANGED.");
    let lines = w090_lines(&root).join("\n");
    assert!(lines.contains("REQ-SL-002"), "B's link is suspect:\n{lines}");
    assert!(
        !lines.contains("REQ-SL-001"),
        "A's independent baseline must stay current:\n{lines}"
    );
}

// ── REQ-TRS-SUS-LINKS-006 — `suspect list` surfaces unbaselined links ──

#[test]
fn list_reports_unbaselined_links() {
    // With no baseline stored, `validate` is silent but `suspect list` must still
    // surface the link as an un-baselined candidate for `suspect accept`.
    let root = base_pair(None);
    assert!(!has_w090(&root), "precondition: unbaselined link is validation-silent");
    let (o, _e, _c) = run(&root, &["suspect", "list"]);
    assert!(
        o.contains("TC-SL-001") && o.contains("REQ-SL-001"),
        "suspect list surfaces the unbaselined link (source+target):\n{o}"
    );
}

// ── REQ-TRS-SUS-LINKS-005 — accept accepts qualified-name arguments ──

#[test]
fn accept_resolves_qualified_name_arguments() {
    // `suspect accept` must accept a source/target given as a qualified name, not
    // only a stable id, and baseline the same link (keyed by the authored ref).
    let root = base_pair(None);
    let (_o, _e, code) = run(
        &root,
        &["suspect", "accept", "Verification::TC-SL-001", "Requirements::REQ-SL-001"],
    );
    assert_eq!(code, 0, "accept by qualified name exits 0");
    let src = read(&root, "Verification/TC-SL-001.md");
    assert!(src.contains("traceBaselines"), "accept by qname writes traceBaselines:\n{src}");
    // The stored key is the authored ref (the id form on the link), and the link clears.
    assert!(src.contains("REQ-SL-001"), "the baseline is keyed by the authored target ref:\n{src}");
    assert!(!has_w090(&root), "the link is baselined and current after accept-by-qname");
}
