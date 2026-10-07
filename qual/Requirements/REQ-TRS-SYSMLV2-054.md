---
id: REQ-TRS-SYSMLV2-054
type: Requirement
name: "A use-case include of a name that resolves to a use case in the submodel maps to the includes field; an unresolved include is counted as unmapped"
status: verified
reqDomain: software
verificationMethod: test
---

`include X;` in a `use case def`/`use case` body shall append the resolved qualified name of `X` to the element's native `includes:` list, resolving `X` innermost-scope-first against the ingested use-case elements. An include that does not resolve to an ingested `UseCaseDef`/`UseCase` is not stored and is counted in the file's `W543` unmapped total as `include` (so `syscribe sysml` and `sysml_submodels` show it). The pinned parser exposes only the simple name of the included use case, so a qualified or declaring form (`include use case x : T`) is not part of this requirement.

**Source:** `REQ-TRS-SYSMLV2-054` (product model), `ADR-SYS-SYSMLV2-001`.
