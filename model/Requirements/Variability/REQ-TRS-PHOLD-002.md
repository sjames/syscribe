---
type: Requirement
id: REQ-TRS-PHOLD-002
name: "A parameter binding change drifts only the baselines of the configuration it changes"
status: draft
reqDomain: software
reqClass: system
tags:
  - variability
  - baseline
---

Per-configuration freshness of placeholder text shall be tracked without making unrelated configurations or the base model stale (GH #266).

## Behavior

- A suspect-link or full-model baseline hash is taken on the symbolic text, so editing a configuration's `parameterBindings:` never raises `W090` on the base model and never changes a placeholder requirement's own hash in a full-model baseline. (The `Configuration` file is itself content, so a full-model baseline that includes it reports that file as changed, as for any edit.)
- A baseline frozen with `frozenScope.config` hashes the substituted text of that configuration. Changing a binding of configuration X changes the requirement's hash in a baseline frozen for X and not in a baseline frozen for another configuration Y (`baseline diff <BL> --current` lists it; `baseline verify --detail` lists it on drift).
