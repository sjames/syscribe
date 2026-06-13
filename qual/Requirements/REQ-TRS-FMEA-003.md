---
id: REQ-TRS-FMEA-003
type: Requirement
name: "FaultTreeEvent shall support fmeaRef: cross-reference to a reconciling FMEAEntry"
status: draft
reqDomain: software
verificationMethod: test
---

A `FaultTreeEvent` element **shall** accept an optional frontmatter field
**`fmeaRef:`** (string, `RawFrontmatter.fmea_ref`). The value is a cross-reference to
a `FMEAEntry` element (FM-* id or qualified name) that corresponds to the same failure
mode in the inductive FMEA analysis.

An `FMEAEntry` row **shall** accept an optional key **`ftaRef:`** whose value is a
cross-reference to a `FaultTreeEvent` (FTE-* id or qualified name).

Both fields **shall** be surfaced by `links`/`refs`. The validator **shall** raise
**W926** if `fmeaRef:` does not resolve to a known model element; likewise **W927**
for `ftaRef:`. Both are warnings (the referenced analysis may be in a separate model
run) rather than errors, and are opt-in (dormant when the field is absent).

**Acceptance criteria:**

- A `FaultTreeEvent` with `fmeaRef: FM-KERN-001` in a model that contains that
  `FMEAEntry` resolves without W926.
- A `FaultTreeEvent` with `fmeaRef: FM-NONEXIST-001` raises **W926**.
- An `FMEASheet` entry with `ftaRef: FTE-KERN-001` in a model that contains that
  event resolves without W927.
- An `FMEASheet` entry with `ftaRef: FTE-NONEXIST-001` raises **W927**.
- `refs FM-KERN-001` lists the `FaultTreeEvent` that references it via `fmeaRef`.
