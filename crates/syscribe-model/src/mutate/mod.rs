//! Guarded-write / element-mutation engine (ADR-SYS-DE-001, REQ-TRS-DE-001):
//! path-confined writes, referential-integrity dry-run/commit staging, stable-id
//! allocation for new elements, and qualified-name move/rename with reference
//! rewriting.
//!
//! This is the shared machinery behind `syscribe mv` and the MCP guarded-write
//! tools (`create_element` / `update_element` / `move_element` / `delete_element`
//! / `apply_changes`); any crate depending on `syscribe-model` (`syscribe` today,
//! `syscribe-server` later) can build on it rather than re-implementing it.

pub mod create;
pub mod diff;
pub mod guard;
pub mod mv;
pub mod update;

pub use create::{plan_create, CreateError, CreatePlan};
pub use diff::{file_unified_diff, tree_unified_diff};
pub use guard::{
    element_ref_strings, guarded_write, ref_errors, referrers, validator_warnings,
    write_confined, Entry, GuardedWriteOutcome, WriteConfinedError,
};
pub use mv::{move_element, valid_qname, MoveError, MoveReport};
pub use update::apply_update_fields;
