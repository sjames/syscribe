//! TC-TRS-ATTRREF-001..005 (GH #452): `{{Elem.attr}}` value references in prose and
//! scenarios, E331, W095 (copied value, opt-in), W096 (stale copy from git history) and
//! `refs Elem.attr`.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct Model {
    root: PathBuf,
}

const BOUNDS: &str = "---\ntype: PartDef\nname: PolicydBounds\ndomain: software\nram: 12 KiB\nmetadata:\n  - type: Meta::Budget\n    owner: platform\nfeatures:\n  - name: snapshotFixedBytes\n    typedBy: ScalarValues::Integer\n    value: VALUE\n    unit: B\n  - name: retries\n    value: 8\n  - name: since\n    value: 2026\n  - name: snapshot\n    features:\n      - name: header\n        value: 320\n        unit: B\n  - name: powerIn\n    type: Port\n    direction: in\n---\n\nBounds of the policy daemon.\n";

fn bounds(value: &str) -> String {
    BOUNDS.replace("VALUE", value)
}

impl Model {
    fn new(git: bool) -> Model {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-attrref-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        std::fs::create_dir_all(&root).unwrap();
        let m = Model { root };
        if git {
            m.git(&["init", "-q", "-b", "main"]);
            m.git(&["config", "user.email", "owner@example.com"]);
            m.git(&["config", "user.name", "Owner"]);
            m.git(&["config", "commit.gpgsign", "false"]);
        }
        m.write("_index.md", "---\ntype: Package\nname: Root\n---\n");
        m.write(
            "Meta/Budget.md",
            "---\ntype: MetadataDef\nname: Budget\nfeatures:\n  - name: owner\n    typedBy: ScalarValues::String\n  - name: limitKb\n    typedBy: ScalarValues::Integer\n    unit: KiB\n    default: 64\n---\n\nA budget stereotype.\n",
        );
        m.write("PolicydBounds.md", &bounds("47244"));
        m.write("PolicydBounds/maxClients.md", "---\ntype: Attribute\nname: maxClients\nvalue: 128\n---\n\nMaximum clients.\n");
        m.write("Design/_index.md", "---\ntype: Package\nname: Design\n---\n");
        m.write(
            "Design/Limits.md",
            "---\ntype: PartDef\nname: Limits\nfeatures:\n  - name: maxFrame\n    value: 1500\n    unit: B\n---\n\nFrame limits.\n",
        );
        m.write("Requirements/REQ-AR-001.md", &req("REQ-AR-001", "The daemon shall bound its snapshot."));
        m
    }
    fn git(&self, args: &[&str]) -> String {
        let o = Command::new("git").arg("-C").arg(&self.root).args(args).output().unwrap();
        assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    }
    fn commit(&self, msg: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", msg]);
    }
    fn write(&self, rel: &str, c: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.root.join(rel)).unwrap()
    }
    fn run(&self, args: &[&str]) -> (String, String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&self.root).args(args).output().unwrap();
        (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
    }
    fn validate(&self) -> String {
        let (o, e, _) = self.run(&["validate"]);
        format!("{o}{e}")
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn req(id: &str, body: &str) -> String {
    format!("---\nid: {id}\ntype: Requirement\nname: {id}\nstatus: approved\nreqDomain: software\nreqClass: system\n---\n\n{body}\n")
}

fn tc(id: &str, status: &str, body: &str) -> String {
    format!(
        "---\ntype: TestCase\nid: {id}\nname: {id}\nstatus: {status}\ntestLevel: L2\nverifies: [REQ-AR-001]\nverifiesElement: [PolicydBounds]\n---\n\n{body}\n"
    )
}

fn lines_with<'a>(out: &'a str, code: &str) -> Vec<&'a str> {
    out.lines().filter(|l| l.contains(code)).collect()
}

const SCENARIO: &str = "The snapshot is {{PolicydBounds.snapshotFixedBytes}} long; the header {{PolicydBounds.snapshot.header}}; at most {{PolicydBounds.maxClients}} clients; limit {{PolicydBounds.Budget.limitKb}}; ram {{PolicydBounds.ram}}; owner {{PolicydBounds.owner}}; frame {{Design::Limits.maxFrame}}.\n\n```gherkin\nFeature: f\n\n  Scenario: s\n    Given a snapshot of {{PolicydBounds.snapshotFixedBytes}}\n    Then it fits\n```\n\n```text\nraw {{PolicydBounds.snapshotFixedBytes}}\n```\n\nInline `{{PolicydBounds.snapshotFixedBytes}}` stays.";

// ── TC-TRS-ATTRREF-001: rendering ────────────────────────────────────────────

#[test]
fn show_renders_every_attribute_form_with_its_unit() {
    let m = Model::new(false);
    m.write("Verification/TC-AR-001.md", &tc("TC-AR-001", "active", SCENARIO));
    let (out, err, code) = m.run(&["show", "TC-AR-001"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("The snapshot is 47244 B long"), "{out}");
    assert!(out.contains("the header 320 B"), "{out}");
    assert!(out.contains("at most 128 clients"), "{out}");
    assert!(out.contains("limit 64 KiB"), "{out}");
    assert!(out.contains("ram 12 KiB"), "{out}");
    assert!(out.contains("owner platform"), "{out}");
    assert!(out.contains("frame 1500 B"), "{out}");
    assert!(out.contains("Given a snapshot of 47244 B"), "gherkin scenarios resolve: {out}");
    assert!(out.contains("raw {{PolicydBounds.snapshotFixedBytes}}"), "other fences stay literal: {out}");
    assert!(out.contains("Inline `{{PolicydBounds.snapshotFixedBytes}}` stays"), "inline code stays literal: {out}");
    assert!(m.read("Verification/TC-AR-001.md").contains("The snapshot is {{PolicydBounds.snapshotFixedBytes}} long"));

    let v = m.validate();
    assert!(lines_with(&v, "E331").is_empty(), "{v}");
    assert!(lines_with(&v, "E240").is_empty() && lines_with(&v, "E241").is_empty(), "qname value references are not feature placeholders: {v}");
}

#[test]
fn export_html_renders_the_value() {
    let m = Model::new(false);
    m.write("Verification/TC-AR-001.md", &tc("TC-AR-001", "active", SCENARIO));
    let dest = m.root.join("site");
    let (_o, err, code) = m.run(&["export-html", "--out", dest.to_str().unwrap()]);
    assert_eq!(code, 0, "{err}");
    let mut html = String::new();
    for e in walk(&dest) {
        let t = std::fs::read_to_string(&e).unwrap_or_default();
        if t.contains("The snapshot is") {
            html = t;
        }
    }
    assert!(html.contains("The snapshot is 47244 B long"), "{html}");
}

fn walk(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(walk(&p));
        } else {
            out.push(p);
        }
    }
    out
}

// ── TC-TRS-ATTRREF-002: E331 ─────────────────────────────────────────────────

#[test]
fn unresolvable_references_are_e331_with_the_available_attributes() {
    let m = Model::new(false);
    m.write(
        "Verification/TC-AR-001.md",
        &tc("TC-AR-001", "active", "Size {{NoSuchThing.size}}, {{PolicydBounds.snapshotBytes}} and {{PolicydBounds.powerIn}}."),
    );
    let v = m.validate();
    let e = lines_with(&v, "E331");
    assert_eq!(e.len(), 3, "{v}");
    assert!(e.iter().any(|l| l.contains("NoSuchThing") && l.contains("does not resolve")), "{v}");
    assert!(e.iter().any(|l| l.contains("snapshotBytes") && l.contains("snapshotFixedBytes") && l.contains("maxClients")), "{v}");
    assert!(e.iter().any(|l| l.contains("powerIn") && l.contains("no scalar value")), "{v}");
}

#[test]
fn template_text_and_code_are_not_references() {
    let m = Model::new(false);
    m.write(
        "Verification/TC-AR-001.md",
        &tc("TC-AR-001", "active", "Hello {{ user.name }}. `{{NoSuch.x}}`\n\n```text\n{{NoSuch.y}}\n```\n"),
    );
    let v = m.validate();
    assert!(lines_with(&v, "E331").is_empty(), "{v}");
}

// ── TC-TRS-ATTRREF-003: W095 ─────────────────────────────────────────────────

const COPY: &str = "```gherkin\nFeature: f\n\n  Scenario: s\n    Given a snapshot of 47 244 bytes\n    And 8 retries since 2026 per TC-47244\n    Then it fits\n```\n";

#[test]
fn w095_is_opt_in_and_names_the_attribute() {
    let m = Model::new(false);
    m.write("Verification/TC-AR-001.md", &tc("TC-AR-001", "active", COPY));
    assert!(lines_with(&m.validate(), "W095").is_empty(), "off by default");
    m.write(".syscribe.toml", "[references]\ncopied_values = true\n");
    let v = m.validate();
    let w = lines_with(&v, "W095");
    assert_eq!(w.len(), 1, "only the 47 244 copy (8, 2026 and TC-47244 are filtered): {v}");
    assert!(w[0].contains("PolicydBounds.snapshotFixedBytes") && w[0].contains("{{PolicydBounds.snapshotFixedBytes}}"), "{v}");
}

#[test]
fn malformed_copied_values_is_reported_on_the_config() {
    let m = Model::new(false);
    m.write(".syscribe.toml", "[references]\ncopied_values = \"yes\"\n");
    let v = m.validate();
    assert!(lines_with(&v, "W094").iter().any(|l| l.contains("copied_values") && l.contains("true or false")), "{v}");
}

#[test]
fn lint_docs_reports_w095_as_advisory() {
    let m = Model::new(false);
    m.write(".syscribe.toml", "[references]\ncopied_values = true\n");
    let doc = m.root.join("guide.md");
    std::fs::write(&doc, "The snapshot ({{PolicydBounds.maxClients}} clients) is 47244 bytes.\n").unwrap();
    let (out, err, code) = m.run(&["lint-docs", doc.to_str().unwrap()]);
    assert!(out.contains("W095") && out.contains("snapshotFixedBytes"), "{out}{err}");
    assert_eq!(code, 0, "W095 is advisory in lint-docs: {out}");
}

// ── TC-TRS-ATTRREF-004: W096 ─────────────────────────────────────────────────

#[test]
fn w096_flags_a_copy_of_a_previous_value() {
    let m = Model::new(true);
    m.write("PolicydBounds.md", &bounds("46852"));
    m.commit("first budget");
    m.write("Verification/TC-AR-001.md", &tc("TC-AR-001", "active", "```gherkin\nFeature: f\n\n  Scenario: s\n    Given a snapshot of 46 852 bytes\n    Then it fits\n```\n"));
    m.write("PolicydBounds.md", &bounds("47228"));
    m.commit("second budget");
    m.write("PolicydBounds.md", &bounds("47244"));
    m.commit("final budget");
    let v = m.validate();
    let w = lines_with(&v, "W096");
    assert_eq!(w.len(), 1, "{v}");
    assert!(w[0].contains("46852") && w[0].contains("47244") && w[0].contains("PolicydBounds.snapshotFixedBytes"), "{v}");

    // A draft holder is suppressed.
    m.write("Verification/TC-AR-001.md", &tc("TC-AR-001", "draft", "```gherkin\nFeature: f\n\n  Scenario: s\n    Given a snapshot of 46 852 bytes\n    Then it fits\n```\n"));
    assert!(lines_with(&m.validate(), "W096").is_empty());
}

#[test]
fn w096_needs_git() {
    let m = Model::new(false);
    m.write("Verification/TC-AR-001.md", &tc("TC-AR-001", "active", "Given 46 852 bytes."));
    assert!(lines_with(&m.validate(), "W096").is_empty());
}

// ── TC-TRS-ATTRREF-005: refs Elem.attr ───────────────────────────────────────

#[test]
fn refs_lists_uses_copies_and_stale_copies() {
    let m = Model::new(true);
    m.write("PolicydBounds.md", &bounds("46852"));
    m.commit("first");
    m.write("PolicydBounds.md", &bounds("47244"));
    m.commit("second");
    m.write("Verification/TC-AR-001.md", &tc("TC-AR-001", "active", "Uses {{PolicydBounds.snapshotFixedBytes}}."));
    m.write("Verification/TC-AR-002.md", &tc("TC-AR-002", "active", "Old figure 46 852 bytes."));
    m.write(
        "Requirements/REQ-AR-002.md",
        &req("REQ-AR-002", "The snapshot shall stay at 47244 bytes.").replace("reqClass: system\n", "reqClass: system\nrefines: [PolicydBounds]\n"),
    );
    let (out, err, code) = m.run(&["refs", "PolicydBounds.snapshotFixedBytes"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("47244 B"), "{out}");
    let uses = out.find("Uses").expect(&out);
    let copies = out.find("Literal copies").expect(&out);
    let stale = out.find("Stale copies").expect(&out);
    assert!(uses < copies && copies < stale, "{out}");
    assert!(out[uses..copies].contains("TC-AR-001"), "{out}");
    assert!(out[copies..stale].contains("REQ-AR-002"), "{out}");
    assert!(out[stale..].contains("TC-AR-002") && out[stale..].contains("46852"), "{out}");

    let (j, err, code) = m.run(&["refs", "PolicydBounds.snapshotFixedBytes", "--json"]);
    assert_eq!(code, 0, "{err}");
    let v: serde_json::Value = serde_json::from_str(&j).expect(&j);
    assert_eq!(v["value"], "47244");
    assert_eq!(v["unit"], "B");
    assert_eq!(v["element"], "PolicydBounds");
    assert_eq!(v["uses"][0]["element"], "TC-AR-001");
    assert_eq!(v["copies"][0]["element"], "REQ-AR-002");
    assert_eq!(v["stale"][0]["element"], "TC-AR-002");
    assert_eq!(v["stale"][0]["previous"], "46852");
}

// ── element part by basic name (short form), ambiguity, and the refs key ────

const ARCH_BOUNDS: &str = "---\ntype: PartDef\nname: Bounds\nfeatures:\n  - name: snapshotFixedBytes\n    value: 46852\n    unit: B\n---\n\nArchitecture bounds.\n";

#[test]
fn a_unique_basic_name_resolves_the_element() {
    let m = Model::new(false);
    m.write("Arch/_index.md", "---\ntype: Package\nname: Arch\n---\n");
    m.write("Arch/Bounds.md", ARCH_BOUNDS);
    m.write("Requirements/REQ-AR-003.md", &req("REQ-AR-003", "The snapshot shall be {{Bounds.snapshotFixedBytes}} and the frame {{Limits.maxFrame}}."));
    let (out, err, code) = m.run(&["show", "REQ-AR-003"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("The snapshot shall be 46852 B and the frame 1500 B."), "{out}");
    let v = m.validate();
    assert!(lines_with(&v, "E331").is_empty(), "{v}");
}

#[test]
fn an_ambiguous_basic_name_is_e331_listing_the_candidates() {
    let m = Model::new(false);
    m.write("Arch/_index.md", "---\ntype: Package\nname: Arch\n---\n");
    m.write("Arch/Bounds.md", ARCH_BOUNDS);
    m.write("Other/_index.md", "---\ntype: Package\nname: Other\n---\n");
    m.write("Other/Bounds.md", &ARCH_BOUNDS.replace("46852", "100"));
    m.write("Requirements/REQ-AR-003.md", &req("REQ-AR-003", "The snapshot shall be {{Bounds.snapshotFixedBytes}}."));
    let v = m.validate();
    let e = lines_with(&v, "E331");
    assert_eq!(e.len(), 1, "{v}");
    assert!(e[0].contains("ambiguous") && e[0].contains("Arch::Bounds") && e[0].contains("Other::Bounds"), "{v}");
    let (_o, err, code) = m.run(&["refs", "Bounds.snapshotFixedBytes"]);
    assert_eq!(code, 1);
    assert!(err.contains("Arch::Bounds") && err.contains("Other::Bounds"), "{err}");
}

#[test]
fn a_refs_value_key_never_falls_back_to_fuzzy_element_matching() {
    let m = Model::new(false);
    m.write("Arch/_index.md", "---\ntype: Package\nname: Arch\n---\n");
    m.write("Arch/Bounds.md", ARCH_BOUNDS);
    m.write("Verification/TC-AR-001.md", &tc("TC-AR-001", "active", "Uses {{Bounds.snapshotFixedBytes}}."));
    let (out, err, code) = m.run(&["refs", "Bounds.snapshotFixedBytes"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("Arch::Bounds.snapshotFixedBytes") && out.contains("46852 B") && out.contains("TC-AR-001"), "{out}");

    let (out, err, code) = m.run(&["refs", "Nowhere.snapshotFixedBytes"]);
    assert_eq!(code, 1, "{out}");
    assert!(err.contains("does not resolve"), "{err}");
    assert!(!out.contains("# References to"), "no fuzzy element listing: {out}");
}
