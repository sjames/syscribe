//! Native SysML v2/KerML submodel ingestion (`ADR-SYS-SYSMLV2-001`,
//! `REQ-TRS-SYSMLV2-*`).
//!
//! A package `_index.md` may declare `sysmlSubmodel: true`. Every
//! `.sysml`/`.kerml` file anywhere in that directory's subtree — however
//! nested — is parsed in-process via the `sysml-v2-parser` crate instead of
//! Markdown+YAML frontmatter; the package's own `_index.md` remains a normal
//! native element. This is a dedicated, always-on native subsystem — not a
//! `[plugins.<alias>]` engine variant (see the ADR's sub-decision 1): there is
//! no sandbox, no config, no alias, and it runs from its own call site in
//! [`crate::walker::walk_model`], not through `plugins::apply_foreign_plugins`.
//!
//! A model with no `sysmlSubmodel: true` package is completely unaffected
//! (`REQ-TRS-SYSMLV2-000`).

use std::path::Path;

use crate::element::RawElement;

/// Apply `sysmlSubmodel: true` subtree scoping to `elements` in place.
///
/// No-op today (stub) — filled in by `REQ-TRS-SYSMLV2-001`.
pub fn apply_sysmlv2_submodels(_elements: &mut Vec<RawElement>, _model_root: &Path) {}
