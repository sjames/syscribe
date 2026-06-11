---
id: REQ-TRS-TRACE-003
type: Requirement
title: "Tool shall emit W303 when breakdownAdr: references a proposed ADR"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** emit warning `W303` when a `Requirement` with `status: approved` or higher references a `breakdownAdr:` whose `ADR.status` is `proposed`.

**Source:** §12.2; §11.12 `W303`

**Acceptance criteria:** A `Requirement` at `status: approved` referencing an `ADR` at `status: proposed` produces exactly one `W303` finding. The same `Requirement` at `status: draft` does not produce `W303`.
