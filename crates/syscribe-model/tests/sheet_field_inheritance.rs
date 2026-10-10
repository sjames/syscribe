//! TC-TRS-SHEET-001 / GH #229: exploded TARA and FMEA rows inherit sheet-level fields.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn model(files: &[(&str, &str)]) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-sheet-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (rel, c) in files {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    r
}

const TARA: &str = "---
type: TARASheet
id: TARA-X-001
name: Sheet
status: approved
responsibility: OEM Cybersecurity
appliesWhen: FEAT-A
tags: [cyber]
goalTable:
  - id: CSG-X-001
    name: g1
  - id: CSG-X-002
    name: g2
    responsibility: Supplier
    tags: [own]
---
";

const FMEA: &str = "---
type: FMEASheet
id: FMEA-X-001
name: FM
status: approved
responsibility: Safety Team
appliesWhen: FEAT-A
tags: [fm]
entries:
  - id: FM-X-001
    failureMode: stuck
---
";

fn find<'a>(els: &'a [syscribe_model::element::RawElement], id: &str) -> &'a syscribe_model::element::RawElement {
    els.iter().find(|e| e.frontmatter.id.as_deref() == Some(id)).unwrap_or_else(|| panic!("{id} not found"))
}

#[test]
fn tara_rows_inherit_and_override() {
    let r = model(&[("T/TARA-X-001.md", TARA)]);
    let els = walk_model(&r).unwrap();
    let a = &find(&els, "CSG-X-001").frontmatter;
    assert_eq!(a.responsibility.as_deref(), Some("OEM Cybersecurity"));
    assert!(a.applies_when.is_some());
    assert_eq!(a.tags.as_deref(), Some(&["cyber".to_string()][..]));
    let b = &find(&els, "CSG-X-002").frontmatter;
    assert_eq!(b.responsibility.as_deref(), Some("Supplier"));
    assert_eq!(b.tags.as_deref(), Some(&["own".to_string()][..]));
    assert!(b.applies_when.is_some());
}

#[test]
fn fmea_entries_inherit() {
    let r = model(&[("T/FMEA-X-001.md", FMEA)]);
    let els = walk_model(&r).unwrap();
    let e = &find(&els, "FM-X-001").frontmatter;
    assert_eq!(e.responsibility.as_deref(), Some("Safety Team"));
    assert!(e.applies_when.is_some());
    assert_eq!(e.tags.as_deref(), Some(&["fm".to_string()][..]));
}

#[test]
fn w038_does_not_fire_per_row_when_the_sheet_declares_responsibility() {
    let r = model(&[("T/TARA-X-001.md", &TARA.replace("    responsibility: Supplier\n", ""))]);
    let els = walk_model(&r).unwrap();
    let res = validate(&els);
    let n = res.findings.iter().filter(|f| f.code == "W038" && f.message.contains("CSG-X")).count();
    assert_eq!(n, 0, "{:?}", res.findings.iter().filter(|f| f.code == "W038").collect::<Vec<_>>());
}

#[test]
fn a_bad_sheet_applies_when_is_reported_once_not_once_per_row() {
    // Review of GH #229: the copy on each row must not repeat the sheet's own finding.
    let bad = TARA.replace("appliesWhen: FEAT-A", "appliesWhen: Nope::Missing");
    let r = model(&[("T/TARA-X-001.md", &bad)]);
    let els = walk_model(&r).unwrap();
    let res = validate(&els);
    let n = res.findings.iter().filter(|f| f.code == "E209").count();
    assert_eq!(n, 1, "{:?}", res.findings.iter().filter(|f| f.code == "E209").collect::<Vec<_>>());
}

#[test]
fn an_inherited_applies_when_does_not_repeat_the_nesting_finding() {
    // Sheet and its package both declare appliesWhen: the sheet is the one nested
    // declaration (E228 once); the rows' inherited copy is not reported again.
    let pkg = "---\ntype: Package\nname: T\nappliesWhen: FEAT-A\n---\n";
    let feat = "---\ntype: FeatureDef\nid: FEAT-A\nname: A\n---\n";
    let r = model(&[("T/_index.md", pkg), ("Features/A.md", feat), ("T/TARA-X-001.md", TARA)]);
    let els = walk_model(&r).unwrap();
    let res = validate(&els);
    let n = res.findings.iter().filter(|f| f.code == "E228").count();
    assert_eq!(n, 1, "{:?}", res.findings.iter().filter(|f| f.code == "E228").collect::<Vec<_>>());
}
