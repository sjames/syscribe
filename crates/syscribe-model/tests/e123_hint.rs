//! TC-TRS-E123HINT-001 / GH #254.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn e123(files: &[(&str, &str)]) -> Vec<String> {
    static N: AtomicU64 = AtomicU64::new(0);
    let r: PathBuf = std::env::temp_dir().join(format!("syscribe-e123-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    std::fs::write(r.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (rel, c) in files {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    let els = walk_model(&r).unwrap();
    validate(&els).findings.into_iter().filter(|f| f.code == "E123").map(|f| f.message).collect()
}

const ITEM_DEF: (&str, &str) = ("Data/Rec.md", "---\ntype: ItemDef\nname: Rec\n---\n");
const ATTR_DEF: (&str, &str) = ("Data/Val.md", "---\ntype: AttributeDef\nname: Val\n---\n");

#[test]
fn inline_attribute_typed_by_item_def_suggests_attribute_def() {
    let m = e123(&[
        ITEM_DEF,
        ("Unit.md", "---\ntype: PartDef\nname: Unit\nfeatures:\n  - {name: record, type: Attribute, typedBy: Data::Rec}\n---\n"),
    ]);
    assert_eq!(m.len(), 1, "{m:?}");
    assert!(m[0].contains("AttributeDef") && m[0].contains("type: Item") && m[0].contains("flow"), "{}", m[0]);
}

#[test]
fn standalone_attribute_usage_gets_the_remedy() {
    let m = e123(&[ITEM_DEF, ("U.md", "---\ntype: Attribute\nname: U\ntypedBy: Data::Rec\n---\n")]);
    assert_eq!(m.len(), 1, "{m:?}");
    assert!(m[0].contains("AttributeDef") && m[0].contains("type: Item"), "{}", m[0]);
}

#[test]
fn item_typed_by_attribute_def_gets_the_mirror_remedy() {
    let m = e123(&[
        ATTR_DEF,
        ("Unit.md", "---\ntype: PartDef\nname: Unit\nfeatures:\n  - {name: msg, type: Item, typedBy: Data::Val}\n---\n"),
    ]);
    assert_eq!(m.len(), 1, "{m:?}");
    assert!(m[0].contains("ItemDef") && m[0].contains("type: Attribute"), "{}", m[0]);
}
