//! `timing` — where the time goes when a model is loaded and validated (GH #429, `REQ-TRS-TIMING-001`).
//!
//! Incremental revalidation (re-checking only the subgraph an edit touches) is a large change to a validator whose
//! rules are mostly global, so `#429` asks for a measurement and a target before anything is built. This command is
//! the measurement: it loads the model and validates it, and reports each phase, so the question "is this model slow
//! enough to need it?" has a number for an answer on the reader's own model, not only on the repository's.

use std::path::Path;
use std::time::Instant;

use serde_json::{json, Value};
use syscribe_model::config::ValidateConfig;
use syscribe_model::element::ElementType;

/// Peak resident memory of this process in KiB (Linux), else `None`.
fn peak_rss_kib() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status").ok()?.lines().find_map(|l| l.strip_prefix("VmHWM:")?.trim().trim_end_matches("kB").trim().parse().ok())
}

pub fn measure(model_root: &Path) -> Result<Value, String> {
    let t0 = Instant::now();
    let elements = syscribe_model::walker::walk_model(model_root).map_err(|e| e.to_string())?;
    let load_ms = t0.elapsed().as_millis() as u64;
    let config = ValidateConfig::with_model_root(model_root);

    let t1 = Instant::now();
    let result = syscribe_model::validator::validate_with_config(&elements, &config);
    let validate_ms = t1.elapsed().as_millis() as u64;

    // Each Configuration's projected variant is validated again; that is the cost `--all-configs` and `health` pay.
    let confs: Vec<_> = elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Configuration)).collect();
    let t2 = Instant::now();
    for c in &confs {
        let sel = syscribe_model::projection::canonical_selection(&elements, c);
        let _ = syscribe_model::projection::validate_projected(&elements, &config, &sel);
    }
    let variants_ms = t2.elapsed().as_millis() as u64;

    let files = walkdir_count(model_root);
    Ok(json!({
        "schemaVersion": "1.0",
        "files": files,
        "elements": elements.len(),
        "configurations": confs.len(),
        "findings": result.findings.len(),
        "loadMs": load_ms,
        "validateMs": validate_ms,
        "variantsMs": variants_ms,
        "totalMs": load_ms + validate_ms,
        "peakRssKiB": peak_rss_kib(),
        "build": if cfg!(debug_assertions) { "debug" } else { "release" },
    }))
}

fn walkdir_count(root: &Path) -> usize {
    fn go(p: &Path, n: &mut usize) {
        let Ok(rd) = std::fs::read_dir(p) else { return };
        for e in rd.flatten() {
            let path = e.path();
            if e.file_type().is_ok_and(|t| t.is_dir()) {
                if !e.file_name().to_string_lossy().starts_with('.') {
                    go(&path, n);
                }
            } else if path.extension().is_some_and(|x| x == "md") {
                *n += 1;
            }
        }
    }
    let mut n = 0;
    go(root, &mut n);
    n
}

pub fn cmd_timing(model_root: &Path, args: &[String]) -> i32 {
    if let Some(bad) = args.iter().find(|a| a.starts_with('-') && *a != "--json") {
        eprintln!("timing: unknown option '{bad}'");
        return 2;
    }
    let v = match measure(model_root) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };
    if args.iter().any(|a| a == "--json") {
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        return 0;
    }
    println!("# Timing ({} build)\n", v["build"].as_str().unwrap_or(""));
    println!("- {} file(s), {} element(s), {} configuration(s), {} finding(s)", v["files"], v["elements"], v["configurations"], v["findings"]);
    println!("- load (walk, parse, synthesis passes): {} ms", v["loadMs"]);
    println!("- validate: {} ms", v["validateMs"]);
    println!("- variants (each Configuration's projected validation): {} ms", v["variantsMs"]);
    println!("- load + validate: {} ms", v["totalMs"]);
    if let Some(r) = v["peakRssKiB"].as_u64() {
        println!("- peak memory: {} MiB", r / 1024);
    }
    0
}
