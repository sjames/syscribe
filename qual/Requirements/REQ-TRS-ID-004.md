---
id: REQ-TRS-ID-004
type: Requirement
name: "Tool shall emit E101 when two elements share the same id: value"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** detect duplicate `id:` values across all loaded elements and emit error `E101` for each collision. The `id:` field must be globally unique within a model.

**Source:** §11.12 `E101`

**Acceptance criteria:** Two `Requirement` files with the same `id:` value produce exactly one `E101` finding (attributed to the second occurrence or both, as implemented).
