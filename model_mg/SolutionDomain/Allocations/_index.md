---
type: Package
name: Allocations
---

Logical-to-physical allocations linking W3 logical subsystems to S3 physical components.

These edges now use the OSLC-default lightweight form: `allocatedTo:` is declared on the
**source** (each W3 logical subsystem under `ProblemDomain::WhiteBox::LogicalSubsystems`), and the
reverse `allocatedFrom` is derived (shown as `## Allocated from` on each physical component). No
standalone `Allocation` elements remain here because none of these edges carry per-edge
documentation. Use a standalone `Allocation` element only when an individual edge needs its own
narrative.
