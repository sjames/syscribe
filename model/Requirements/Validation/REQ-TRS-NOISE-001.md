---
type: Requirement
id: REQ-TRS-NOISE-001
name: "W015 is one finding per requirement; validate --summary counts findings per code"
status: draft
reqDomain: software
reqClass: system
tags:
  - validation
---

Per-configuration coverage noise shall be reduced (GH #245).

## Behavior

- `W015` shall be reported once per requirement, naming every configuration in which the requirement is active but has no covering TestCase, instead of once per (requirement, configuration). The finding is filed against the requirement.
- A requirement covered in every configuration where it is active raises nothing; gating with `--deny W015` still works.
- `validate --summary` shall print a table of finding counts per code and severity instead of the per-finding tables; the exit code, gating and `--json` shape rules are unchanged (`--json --summary` emits `[{code, severity, count}]`).
- `W022` (`feature-check --deep`) is already one finding per requirement and is unchanged.
