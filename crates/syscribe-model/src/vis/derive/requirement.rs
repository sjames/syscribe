//! The `requirement` generator — see the requirement named in `derive/mod.rs`'s dispatch.
//! Stub: fills nothing yet; the planning item for this kind replaces it.

use crate::element::RawElement;
use crate::resolver::Resolver;

use super::super::ir::DiagramGraph;
use super::super::manifest::Issue;
use super::Filters;

pub fn generate(
    _graph: &mut DiagramGraph,
    _subject: &RawElement,
    _elements: &[RawElement],
    _resolver: &Resolver,
    _filters: &Filters,
    _issues: &mut Vec<Issue>,
) {
}
