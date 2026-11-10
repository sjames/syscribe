//! TC-TRS-SIDECAR-001 (GH #472, #473, #474): the mutation and campaign sidecars are bounded, are never overwritten when
//! they cannot be read, and are written beside and renamed, so an interruption cannot truncate them.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use syscribe_model::exec_evidence::{load_mutation, mutation_sidecar, record_mutation, MutationRecord, MUTATION_KEEP};
use syscribe_model::residual_risk::{self, CampaignRecord};

fn root() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-sidecar-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn rec(machine: &str, subject: &str, hash: &str, at: u64) -> MutationRecord {
    MutationRecord { subject: subject.into(), machine: machine.into(), machine_hash: hash.into(), score: 90.0, killed: 9, survived: 1, recorded_at_unix: at }
}

#[test]
fn mutation_records_are_bounded_per_machine_and_subject_and_the_newest_survive() {
    let d = root();
    for i in 0..(MUTATION_KEEP + 5) {
        record_mutation(&d, vec![rec("M", "S", &format!("h{i}"), i as u64)]).unwrap();
    }
    for i in 0..3 {
        record_mutation(&d, vec![rec("M", "T", &format!("t{i}"), i as u64)]).unwrap();
    }
    let all = load_mutation(&d).unwrap();
    let list = &all["M"];
    let of = |s: &str| list.iter().filter(|r| r.subject == s).collect::<Vec<_>>();
    assert_eq!((of("S").len(), of("T").len()), (MUTATION_KEEP, 3), "the bound is per subject, so another subject keeps its own");
    let hashes: Vec<&str> = of("S").iter().map(|r| r.machine_hash.as_str()).collect();
    assert_eq!(hashes.first(), Some(&format!("h{}", MUTATION_KEEP + 4).as_str()), "newest first");
    assert!(!hashes.contains(&"h0") && !hashes.contains(&"h4") && hashes.contains(&"h5"), "the five oldest were evicted: {hashes:?}");
    // Recording the same hash again replaces its record: no growth.
    record_mutation(&d, vec![rec("M", "S", "h50", 999)]).unwrap();
    assert_eq!(load_mutation(&d).unwrap()["M"].iter().filter(|r| r.subject == "S").count(), MUTATION_KEEP);
}

#[test]
fn a_missing_mutation_file_is_no_records_and_an_unreadable_one_is_an_error_never_overwritten() {
    let d = root();
    assert!(load_mutation(&d).unwrap().is_empty(), "no file: nothing recorded");
    std::fs::create_dir_all(d.join(".syscribe")).unwrap();
    std::fs::write(mutation_sidecar(&d), "{ not json").unwrap();
    let e = load_mutation(&d).unwrap_err();
    assert!(e.contains("mutation.json"), "the file is named: {e}");
    let e = record_mutation(&d, vec![rec("M", "S", "h", 1)]).unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
    assert!(e.to_string().contains("cannot be read, so nothing was recorded") && e.to_string().contains("fix or remove"), "{e}");
    assert_eq!(std::fs::read_to_string(mutation_sidecar(&d)).unwrap(), "{ not json", "the damaged file is left for a person to fix");
}

fn campaign() -> CampaignRecord {
    serde_json::from_value(serde_json::json!({"subject": "TC-X-001", "composite": "Comp", "compositeHash": "h", "sets": []})).unwrap()
}

#[test]
fn both_sidecars_are_written_beside_and_renamed_leaving_no_temporary_file() {
    let d = root();
    let m = record_mutation(&d, vec![rec("M", "S", "h", 1)]).unwrap();
    let c = residual_risk::record(&d, campaign()).unwrap();
    for p in [&m, &c] {
        let dir = p.parent().unwrap();
        let strays: Vec<_> = std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).filter(|n| n.ends_with(".tmp")).collect();
        assert!(strays.is_empty(), "{}: {strays:?}", p.display());
        serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(p).unwrap()).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    }
    // A second record of the campaign replaces the file whole and keeps it valid JSON.
    residual_risk::record(&d, campaign()).unwrap();
    assert!(residual_risk::load(&d).is_ok());
}
