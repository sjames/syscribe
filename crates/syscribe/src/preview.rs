//! The preview step shared by every checked write (`bind`, `new`, and the language server's `syscribe/preview` and
//! `syscribe.apply`): apply the planned files to a **candidate copy** of the model through the guarded-write engine in
//! dry-run mode, and report what validation would say. Nothing on disk changes.

use std::path::{Path, PathBuf};

use syscribe_model::config::ValidateConfig;
use syscribe_model::element::RawElement;
use syscribe_model::mutate::guard::{guarded_write_inspect, GuardedWriteOutcome};

/// Preview writing `files` (absolute path, full content; a missing file is created) into the model at `model_root`.
pub fn preview_files(model_root: &Path, elements: &[RawElement], config: &ValidateConfig, files: &[(PathBuf, String)]) -> Result<GuardedWriteOutcome, String> {
    let model_abs = std::fs::canonicalize(model_root).unwrap_or_else(|_| model_root.to_path_buf());
    let mut rels: Vec<(PathBuf, String)> = Vec::new();
    for (path, content) in files {
        // The parent exists even when the file does not, so canonicalise that to compare with the canonical root.
        let abs = std::fs::canonicalize(path.parent().unwrap_or(Path::new("."))).map(|p| p.join(path.file_name().unwrap_or_default())).unwrap_or_else(|_| path.clone());
        let rel = abs.strip_prefix(&model_abs).or_else(|_| path.strip_prefix(model_root)).map_err(|_| format!("{} is not under the model root", path.display()))?;
        rels.push((rel.to_path_buf(), content.clone()));
    }
    let (outcome, _) = guarded_write_inspect(
        model_root,
        elements,
        config,
        true, // dry run: the candidate copy only
        false,
        true,
        move |root: &Path| {
            for (rel, content) in &rels {
                let p = root.join(rel);
                if let Some(d) = p.parent() {
                    std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
                }
                std::fs::write(&p, content).map_err(|e| e.to_string())?;
            }
            Ok(())
        },
        None::<&dyn Fn(&[RawElement]) -> ()>,
        |_| false,
    );
    match &outcome.reason {
        Some(r) => Err(r.clone()),
        None => Ok(outcome),
    }
}

/// How many distinct errors the preview added (the same finding can be raised by two checks).
pub fn new_error_count(outcome: &GuardedWriteOutcome) -> usize {
    outcome.new_errors.iter().map(|e| (e.0.clone(), e.2.clone())).collect::<std::collections::HashSet<_>>().len()
}
