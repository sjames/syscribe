---
type: Requirement
id: REQ-TRS-SAFEMECH-001
name: "SafetyMechanism records which failure modes a mechanism covers, with DC and reaction time"
status: draft
reqDomain: software
reqClass: system
tags:
  - safety
---

The format shall have a `SafetyMechanism` element (`SM-*`) so "which mechanism covers which failure mode, with what diagnostic coverage and reaction time" is answerable from the model (GH #236, v1: the element, its checks and a `mechanisms` listing; the hardware FMEDA table feeding SPFM/LFM follows).

## Behavior

- Frontmatter: `id: SM-<…>-NNN`, `name`, `status` (`draft` · `review` · `approved` · `retired`), `covers:` (string or list of `FMEAEntry`, `FaultTreeEvent`, `Requirement`, `SafetyGoal` or `HazardousEvent`), `diagnosticCoverage` and `latentDiagnosticCoverage` (numbers in 0..1), `reactionTime` (a duration such as `10 ms`), `safeState` (text) and `allocatedTo:`.
- `E896` — `id`, `name` or `status` missing, an id not `SM-*`, a status outside the enum, a coverage outside 0..1. `E897` — a `covers` entry that resolves to nothing or to an element of another type.
- `W894` — a non-draft mechanism's `reactionTime` exceeds the `ftti` of a `SafetyGoal` it covers, directly or through a covered requirement's `derivedFromSafetyGoal`.
- `mechanisms [--json] [--uncovered]` lists each mechanism with what it covers, its coverages, reaction time, safe state and allocation; `--uncovered` lists the `FMEAEntry` rows no mechanism covers.
