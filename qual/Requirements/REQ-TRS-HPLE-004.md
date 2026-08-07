---
id: REQ-TRS-HPLE-004
type: Requirement
name: An unresolved required parameter anywhere in a consolidated subtree shall be an opt-in, deny-gateable warning, never a hard error at an intermediate tier
status: draft
reqDomain: software
verificationMethod: test
---

For a `Configuration` with `subConfigurations:`, the tool **shall** compute the transitive closure
of every `isRequired: true`, no-`default:` parameter — of every `FeatureDef` actually selected
anywhere in the consolidated subtree, at any depth — that remains unbound after applying every
`parameterBindings:` entry from that `Configuration` down through every tier already resolved
beneath it. A non-empty closure **shall** be reported as a warning, silent to the exit code by
default and gateable via `--deny`, following the same opt-in posture as other opt-in warnings in
this tool. It **shall not** be escalated to a hard error purely because one tier's own isolated
validation run still finds it open.

**Source:** `REQ-TRS-HPLE-004` (product model), `ADR-SYS-HPLE-001`.

**Acceptance criteria:** a `Configuration` whose consolidated subtree leaves a selected, required,
no-default parameter unbound reports exactly one warning naming that parameter, at Warning
severity (never Error) and with a non-zero exit code only under `--deny` for that warning's code; a
`Configuration` whose subtree is fully closed (every such parameter bound by some tier on the path,
or self-sufficient via `default:`) reports none.
