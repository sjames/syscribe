//! TC-TRS-REQIFIMP-001 / GH #241.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn dir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-reqifimp-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(d.join("model/Requirements")).unwrap();
    std::fs::write(d.join("model/.syscribe.toml"), "[ids.prefixes]\nRequirement = [\"STK\"]\n").unwrap();
    std::fs::write(d.join("model/_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(d.join("model/Requirements/_index.md"), "---\ntype: Package\nname: Requirements\n---\n").unwrap();
    d
}

const OEM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<REQ-IF xmlns="http://www.omg.org/spec/ReqIF/20110401/reqif.xsd" xmlns:xhtml="http://www.w3.org/1999/xhtml">
<CORE-CONTENT><REQ-IF-CONTENT>
<SPEC-TYPES>
 <SPEC-OBJECT-TYPE IDENTIFIER="T-REQ" LONG-NAME="Requirement"><SPEC-ATTRIBUTES>
  <ATTRIBUTE-DEFINITION-STRING IDENTIFIER="A-ID" LONG-NAME="ReqIF.ForeignID"/>
  <ATTRIBUTE-DEFINITION-STRING IDENTIFIER="A-NAME" LONG-NAME="ReqIF.Name"/>
  <ATTRIBUTE-DEFINITION-XHTML IDENTIFIER="A-TEXT" LONG-NAME="ReqIF.Text"/>
 </SPEC-ATTRIBUTES></SPEC-OBJECT-TYPE>
 <SPEC-OBJECT-TYPE IDENTIFIER="T-FOLDER" LONG-NAME="Package"/>
</SPEC-TYPES>
<SPEC-OBJECTS>
 <SPEC-OBJECT IDENTIFIER="so-1" LONG-NAME="so-1"><TYPE><SPEC-OBJECT-TYPE-REF>T-REQ</SPEC-OBJECT-TYPE-REF></TYPE><VALUES>
  <ATTRIBUTE-VALUE-STRING THE-VALUE="OEM-0042"><DEFINITION><ATTRIBUTE-DEFINITION-STRING-REF>A-ID</ATTRIBUTE-DEFINITION-STRING-REF></DEFINITION></ATTRIBUTE-VALUE-STRING>
  <ATTRIBUTE-VALUE-STRING THE-VALUE="Boot time &amp; splash"><DEFINITION><ATTRIBUTE-DEFINITION-STRING-REF>A-NAME</ATTRIBUTE-DEFINITION-STRING-REF></DEFINITION></ATTRIBUTE-VALUE-STRING>
  <ATTRIBUTE-VALUE-XHTML><DEFINITION><ATTRIBUTE-DEFINITION-XHTML-REF>A-TEXT</ATTRIBUTE-DEFINITION-XHTML-REF></DEFINITION><THE-VALUE><xhtml:div><xhtml:p>The cluster shall show the splash within 2 s.</xhtml:p><xhtml:p>Applies &lt; 85 C.</xhtml:p></xhtml:div></THE-VALUE></ATTRIBUTE-VALUE-XHTML>
 </VALUES></SPEC-OBJECT>
 <SPEC-OBJECT IDENTIFIER="so-2" LONG-NAME="so-2"><TYPE><SPEC-OBJECT-TYPE-REF>T-REQ</SPEC-OBJECT-TYPE-REF></TYPE><VALUES>
  <ATTRIBUTE-VALUE-STRING THE-VALUE="OEM-0043"><DEFINITION><ATTRIBUTE-DEFINITION-STRING-REF>A-ID</ATTRIBUTE-DEFINITION-STRING-REF></DEFINITION></ATTRIBUTE-VALUE-STRING>
  <ATTRIBUTE-VALUE-STRING THE-VALUE="Telltales"><DEFINITION><ATTRIBUTE-DEFINITION-STRING-REF>A-NAME</ATTRIBUTE-DEFINITION-STRING-REF></DEFINITION></ATTRIBUTE-VALUE-STRING>
 </VALUES></SPEC-OBJECT>
 <SPEC-OBJECT IDENTIFIER="so-folder" LONG-NAME="Chapter 1"><TYPE><SPEC-OBJECT-TYPE-REF>T-FOLDER</SPEC-OBJECT-TYPE-REF></TYPE><VALUES/></SPEC-OBJECT>
</SPEC-OBJECTS>
</REQ-IF-CONTENT></CORE-CONTENT></REQ-IF>"#;

fn write_oem(d: &Path, text: &str) -> String {
    let p = d.join("oem.reqif");
    std::fs::write(&p, text).unwrap();
    p.to_string_lossy().into_owned()
}

fn run(d: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d.join("model")).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn read(d: &Path, rel: &str) -> String {
    std::fs::read_to_string(d.join("model").join(rel)).unwrap()
}

#[test]
fn objects_become_draft_requirements_with_the_oem_id_in_extref() {
    let d = dir();
    let f = write_oem(&d, OEM);
    let (o, c) = run(&d, &["import-reqif", &f, "--id-prefix", "STK-DC"]);
    assert_eq!(c, 0, "{o}");
    let r = read(&d, "Requirements/STK-DC-001.md");
    assert!(r.contains("id: STK-DC-001") && r.contains("type: Requirement") && r.contains("status: draft"), "{r}");
    assert!(r.contains("reqif:OEM-0042"), "{r}");
    assert!(r.contains("Boot time & splash"), "{r}");
    assert!(r.contains("The cluster shall show the splash within 2 s.\n\nApplies < 85 C."), "{r}");
    assert!(r.contains("reqClass: stakeholder") && r.contains("reqDomain: system"), "{r}");
    assert!(read(&d, "Requirements/STK-DC-002.md").contains("Telltales"));
    assert!(!d.join("model/Requirements/STK-DC-003.md").exists(), "folder object must not import");
    // the imported model validates without errors
    let (v, _) = run(&d, &["validate"]);
    assert!(!v.contains("| E"), "{v}");
}

#[test]
fn reimport_is_idempotent_and_update_rewrites_only_name_and_body() {
    let d = dir();
    let f = write_oem(&d, OEM);
    run(&d, &["import-reqif", &f, "--id-prefix", "STK-DC"]);
    let before = read(&d, "Requirements/STK-DC-001.md");
    let (o, c) = run(&d, &["import-reqif", &f, "--id-prefix", "STK-DC"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.contains("exists") && !d.join("model/Requirements/STK-DC-003.md").exists(), "{o}");
    assert_eq!(before, read(&d, "Requirements/STK-DC-001.md"));
    // hand-edit a field, then re-import a changed text with --update
    let edited = before.replace("status: draft", "status: review");
    std::fs::write(d.join("model/Requirements/STK-DC-001.md"), &edited).unwrap();
    let f2 = write_oem(&d, &OEM.replace("within 2 s", "within 1.5 s"));
    let (o, c) = run(&d, &["import-reqif", &f2, "--id-prefix", "STK-DC", "--update"]);
    assert_eq!(c, 0, "{o}");
    let after = read(&d, "Requirements/STK-DC-001.md");
    assert!(after.contains("within 1.5 s") && after.contains("status: review"), "{after}");
    assert!(o.contains("updated"), "{o}");
}

#[test]
fn dry_run_writes_nothing_and_bad_input_exits_one() {
    let d = dir();
    let f = write_oem(&d, OEM);
    let (o, c) = run(&d, &["import-reqif", &f, "--dry-run"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.contains("would create") && o.contains("OEM-0042"), "{o}");
    assert!(!d.join("model/Requirements/REQ-IMP-001.md").exists());
    let bad = write_oem(&d, "<REQ-IF><oops></REQ-IF>");
    assert_eq!(run(&d, &["import-reqif", &bad]).1, 1);
    let empty = write_oem(&d, "<REQ-IF xmlns=\"x\"><CORE-CONTENT/></REQ-IF>");
    assert_eq!(run(&d, &["import-reqif", &empty]).1, 1);
    assert_eq!(run(&d, &["import-reqif"]).1, 1);
    assert_eq!(run(&d, &["import-reqif", "/no/such/file.reqif"]).1, 1);
}

#[test]
fn an_export_reqif_file_imports_back() {
    let d = dir();
    // source model with two requirements
    let src = d.join("src");
    std::fs::create_dir_all(src.join("Requirements")).unwrap();
    std::fs::write(src.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(src.join("Requirements/_index.md"), "---\ntype: Package\nname: Requirements\n---\n").unwrap();
    for (id, name) in [("REQ-RT-001", "First"), ("REQ-RT-002", "Second")] {
        std::fs::write(src.join(format!("Requirements/{id}.md")), format!("---\nid: {id}\ntype: Requirement\nname: \"{name}\"\nstatus: approved\nreqDomain: software\nreqClass: system\n---\n\nThe system shall do {name}.\n")).unwrap();
    }
    let out = d.join("out.reqif");
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&src).args(["export-reqif", "--output"]).arg(d.join("out")).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let produced = if out.exists() { out } else { d.join("out.reqif") };
    let (o, c) = run(&d, &["import-reqif", produced.to_str().unwrap(), "--id-prefix", "STK-RT"]);
    assert_eq!(c, 0, "{o}");
    assert!(read(&d, "Requirements/STK-RT-001.md").contains("The system shall do First."));
    assert!(read(&d, "Requirements/STK-RT-001.md").contains("reqif:REQ-RT-001"));
    assert!(d.join("model/Requirements/STK-RT-002.md").exists());
}
