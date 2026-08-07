---
type: Requirement
id: REQ-TRS-HPLE-004
name: "An unresolved required parameter anywhere in a consolidated subtree is an opt-in, --deny-gateable warning, never a hard error at an intermediate tier"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-HPLE-000]
breakdownAdr: Decisions::HierarchicalProductLineADR
tags:
  - variability
  - multi-repo
  - validation
---

Syscribe shall compute, for a `Configuration` with `subConfigurations:`, the transitive closure of
every `isRequired: true` parameter — of every `FeatureDef` actually selected anywhere in the
consolidated subtree, at any depth — that remains unbound after applying every `parameterBindings:`
entry from this `Configuration` down through every tier already resolved beneath it. A non-empty
closure shall be reported as a warning, silent by default and gateable via `--deny`, following the
same opt-in posture already established by `W510`/`W511`/`W512` (multi-repo reproducibility),
`W023` (missing implementation path), and `W090` (suspect links). It shall **never** be escalated to
a hard error purely because a single tier's own isolated validation run still finds it open.

## Rationale

An open required parameter at an intermediate tier is not a defect — it is the entire mechanism by
which staged, multi-party configuration across independently-developed product lines is possible. A
tier being validated on its own has no way to know whether it is the actual top of some larger
hierarchy (in which case "still open" now genuinely means incomplete) or will itself be consolidated
further by something it has never seen (in which case "still open" is exactly the intended,
deliberate deferral). Only whichever repo is actually positioned as the point of final assembly can
correctly decide which case applies — by choosing, in its own CI, whether to gate on this warning.

## Scope

- The check applies transitively across the whole consolidated subtree, not just this
  `Configuration`'s direct `subConfigurations:` entries — a parameter left open two or more tiers
  down, with nothing in between closing it, is still surfaced here.
- A parameter a tier author deliberately supplied a `default:` for (rather than declaring
  `isRequired: true` with no default) never appears in this closure at all — it is self-sufficient
  by construction, not something anyone up the chain needs to decide about. No new schema field is
  introduced on `FeatureDef` parameters for this distinction; the existing `isRequired`/`default`
  combination already expresses it fully.
- No new "this model is the root of the hierarchy" schema concept or self-declaration field is
  introduced — the existing opt-in/`--deny` idiom is reused exactly as-is.
