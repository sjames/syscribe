//! `REQ-TRS-FMED-007`: a feature model of 2,000 features and 300 constraints is
//! analysed, configured and drawn within an interactive time. The ceilings are
//! generous (a debug build is several times slower than release, where analysis
//! takes about 0.2 s and a configurator click about 0.25 s); the point is to catch
//! a regression back to one solver call per feature and direction, or to rebuilding
//! the solver for every constraint when extracting a core.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use syscribe_model::feature_model::{analysis_json, configure_selection, MAX_DEEP_FEATURES};
use syscribe_model::vis::derive::feature::feature_diagram;
use syscribe_model::walker::walk_model;

/// A deterministic pseudo-random generator, so the model is the same on every run.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// 2,000 features in a tree of branching 4 to 8, a fifth mandatory, a quarter of the
/// groups `or`, and 300 `requires` between random leaves (never an `excludes` or an
/// XOR group, so the model stays satisfiable).
fn model(features: usize, constraints: usize) -> PathBuf {
    let root = std::env::temp_dir().join(format!("syscribe-fscale-{}-{features}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("F")).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("F/_index.md"), "---\ntype: Package\nname: F\n---\n").unwrap();
    let mut rng = Lcg(11);
    let mut names = vec!["R".to_string()];
    let mut kids: Vec<Vec<usize>> = vec![Vec::new()];
    let mut queue = std::collections::VecDeque::from([(0usize, 0usize)]);
    while names.len() < features {
        let Some((cur, depth)) = queue.pop_front() else { break };
        if depth >= 5 {
            continue;
        }
        for k in 0..(4 + rng.below(5)) {
            if names.len() >= features {
                break;
            }
            let q = format!("{}::{}_{k}", names[cur], names[cur].rsplit("::").next().unwrap().chars().take(6).collect::<String>());
            names.push(q);
            kids.push(Vec::new());
            let idx = names.len() - 1;
            kids[cur].push(idx);
            queue.push_back((idx, depth + 1));
        }
    }
    let leaves: Vec<usize> = (0..names.len()).filter(|i| kids[*i].is_empty()).collect();
    let mut requires: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for _ in 0..constraints {
        let a = leaves[rng.below(leaves.len() as u64) as usize];
        let b = leaves[rng.below(leaves.len() as u64) as usize];
        if a != b {
            requires.entry(a).or_default().push(b);
        }
    }
    for (i, q) in names.iter().enumerate() {
        let mut fm = format!("---\ntype: FeatureDef\nid: FEAT-N{i:04}\nname: {}\n", q.rsplit("::").next().unwrap());
        if i == 0 || rng.below(5) == 0 {
            fm.push_str("mandatory: true\n");
        }
        if !kids[i].is_empty() && rng.below(4) == 0 {
            fm.push_str("groupKind: or\n");
        }
        if let Some(r) = requires.get(&i) {
            let ids: Vec<String> = r.iter().map(|b| format!("FEAT-N{b:04}")).collect();
            fm.push_str(&format!("requires: [{}]\n", ids.join(", ")));
        }
        fm.push_str("---\n");
        let dir: PathBuf = Path::new("F").join(q.replace("::", "/"));
        if kids[i].is_empty() {
            let file = root.join(format!("{}.md", dir.display()));
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, fm).unwrap();
        } else {
            std::fs::create_dir_all(root.join(&dir)).unwrap();
            std::fs::write(root.join(&dir).join("_index.md"), fm).unwrap();
        }
    }
    root
}

#[test]
fn two_thousand_features_are_analysed_configured_and_drawn_interactively() {
    assert!(MAX_DEEP_FEATURES >= 2000, "the analysis limit must admit a 2,000-feature model");
    let root = model(2000, 300);
    let els = walk_model(&root).unwrap();

    let t = Instant::now();
    let a = analysis_json(&els);
    let analysis = t.elapsed();
    assert!(a["skipped"].is_null(), "{}", a["skipped"]);
    assert_eq!(a["void"], false, "the generated model is satisfiable");
    assert_eq!(a["counts"]["features"], 2000);
    assert!(analysis < Duration::from_secs(30), "analysis took {analysis:?}");

    let t = Instant::now();
    let c = configure_selection(&els, &BTreeMap::new());
    let first = t.elapsed();
    assert_eq!(c["satisfiable"], true);
    assert_eq!(c["features"].as_object().unwrap().len(), 2000);
    assert!(first < Duration::from_secs(20), "propagation took {first:?}");

    // One choice: the same propagation, with the root's first child selected.
    let child = els.iter().find(|e| e.qualified_name.ends_with("::R_0") && e.qualified_name.matches("::").count() == 2).unwrap().qualified_name.clone();
    let t = Instant::now();
    let c = configure_selection(&els, &BTreeMap::from([(child.clone(), true)]));
    let one = t.elapsed();
    assert_eq!(c["satisfiable"], true, "{}", c["conflict"]);
    assert_eq!(c["features"][&child]["state"], "selected");
    assert!(one < Duration::from_secs(20), "a click took {one:?}");

    let t = Instant::now();
    let g = feature_diagram(&els, None);
    assert_eq!(g.nodes.len(), 2000);
    assert!(t.elapsed() < Duration::from_secs(10), "diagram took {:?}", t.elapsed());
}

#[test]
fn a_conflict_in_a_large_model_is_still_explained_quickly() {
    let root = model(2000, 300);
    let els = walk_model(&root).unwrap();
    // Choosing a feature and deselecting something it requires directly must be refused with just those two.
    let required = els.iter().find(|e| e.frontmatter.requires.is_some()).expect("some feature requires another");
    let rq = required.frontmatter.requires.as_ref().unwrap()[0].as_str().unwrap().to_string();
    let t = Instant::now();
    let c = configure_selection(&els, &BTreeMap::from([(required.qualified_name.clone(), true), (rq, false)]));
    assert_eq!(c["satisfiable"], false);
    assert_eq!(c["conflict"]["choices"].as_array().unwrap().len(), 2, "{}", c["conflict"]);
    assert!(c["conflict"]["constraints"].to_string().contains("requires"), "{}", c["conflict"]);
    assert!(t.elapsed() < Duration::from_secs(30), "conflict explanation took {:?}", t.elapsed());
}
