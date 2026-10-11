//! REQ-TRS-SYSCOMP-001 (GH #299): in a build without an FMI backend a composite with a plant part is refused with the reason.

use syscribe_model::system_run::SystemModel;
use syscribe_model::walker::walk_model;

#[test]
fn without_a_backend_a_plant_part_is_refused_with_the_reason() {
    let root = std::env::temp_dir().join(format!("syscribe-sysgate-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("CtlM.md"), "---\ntype: StateDef\nname: CtlM\nvariables:\n  - {name: v, value: \"0\"}\nsubStates:\n  - {name: A, isInitial: true}\n---\n").unwrap();
    std::fs::write(root.join("CtlP.md"), "---\ntype: PartDef\nname: CtlP\nexhibitsStates: [CtlM]\n---\nC.\n").unwrap();
    std::fs::write(root.join("BallP.md"), "---\ntype: PartDef\nname: BallP\nfmu:\n  file: ball.fmu\n  kind: cosimulation\n---\nB.\n").unwrap();
    std::fs::write(root.join("Sys.md"), "---\ntype: PartDef\nname: Sys\nplantStep: \"10 ms\"\nfeatures:\n  - {name: ctl, typedBy: CtlP}\n  - {name: ball, typedBy: BallP}\n---\nS.\n").unwrap();
    let e = SystemModel::build_in(&walk_model(&root).unwrap(), "Sys", &root).err().expect("refused");
    assert!(e.iter().any(|x| x.contains("no FMI support")), "{e:?}");
}
