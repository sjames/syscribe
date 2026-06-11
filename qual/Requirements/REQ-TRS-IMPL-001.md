---
id: REQ-TRS-IMPL-001
type: Requirement
title: Tool shall support an implementedBy field on Part/PartDef and emit W023 when its path is missing
status: draft
reqDomain: software
verificationMethod: test
---

To close the V-model traceability leg `Requirement ─satisfies→ Architecture ─implementedBy→ Code ─verifies→ Test`, the tool **shall** support a new optional `implementedBy:` field on `Part` and `PartDef` elements that links an architecture element to the source code that realises it. Today the model can trace a requirement to the architecture element that satisfies it, and a test back to the code it verifies, but the architecture-to-code leg is unrepresented — leaving architecture and code free to drift apart silently (GH issue #13).

## `implementedBy` field

`implementedBy:` **shall** be accepted on `Part` and `PartDef` frontmatter as either a single string or a list of strings, each value being a path into the codebase. The field **shall** be optional; a `Part`/`PartDef` with no `implementedBy` carries no implementation linkage and is never flagged (the rule is strictly opt-in).

Each path **shall** be resolved with the **same source-location semantics already used for `TestCase.sourceFile`** (model/repo-relative, absolute, `file://`, and remote URI), reusing the existing classification so behaviour is identical across both fields:

| Form | Meaning |
|---|---|
| `scheme://…` (not `file`) | remote URI — accepted as external, not resolved locally |
| `file://…` | local path from the file URI |
| `repo:<path>` | relative to the repository root |
| `model:<path>` | relative to the model root |
| `/abs/path` | absolute path |
| `path` (bare) | relative to the model root (the default) |

## `W023` — path-exists rule

The tool **shall** define a new warning code `W023` (the direct architecture-to-code analog of `W004` for `TestCase.sourceFile`). `W023` **shall** be emitted when a `Part`/`PartDef` declares one or more `implementedBy` paths and a **local** path does not exist on disk. One finding **shall** be emitted per missing path. Remote URIs **shall** be accepted as external and not verified locally (matching `W004`'s treatment of remote `sourceFile` values).

`W023` enforcement **shall** be:

- **Opt-in / dormant** — a `Part`/`PartDef` with no `implementedBy` produces no finding.
- **Status-aware** — a `draft` architecture element is suppressed: draft/planned elements describe intended structure whose code may not exist yet, so no finding is emitted. Only non-draft elements are checked.
- **Gateable** — selectable via `--deny W023`, so a project may make architecture-to-code drift fail the build.

## Deferred (out of scope)

The stronger "named symbol must resolve in the module" form — the architecture-to-code analog of `W009` (function-level resolution of a named symbol inside the linked module) — is **deferred** and is **not** part of this requirement. It is noted here as a future extension layered on top of the file-level `W023` check.

**Source:** GH issue #13 (architecture↔code drift); closes the V-model traceability leg `Requirement ─satisfies→ Architecture ─implementedBy→ Code ─verifies→ Test`. Mirrors the `TestCase.sourceFile` / `W004` pattern ([[REQ-TRS-VAL-015]]).

**Acceptance criteria:** a non-`draft` `Part`/`PartDef` whose `implementedBy` path is missing on disk produces exactly one `W023`; the same element with an existing path produces none; an element with no `implementedBy` produces none (opt-in); a `draft` element produces none; `validate --deny W023` exits `2` in the presence of a `W023`.
