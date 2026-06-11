---
id: REQ-TRS-EXTREF-001
type: Requirement
title: Tool shall support an optional extRef field on all elements and warn on duplicates
status: draft
reqDomain: software
verificationMethod: test
---

A Syscribe element frequently **stands in for an artifact that lives in another tool** — a requirement in IBM DOORS Next (DNG), an architectural element in a SysML modelling tool (Cameo/Magic, Rhapsody), a ticket, or any externally-managed object. To make that correspondence explicit and navigable, the tool **shall** support a new optional common frontmatter field **`extRef`** (external reference) on **every** element type.

## `extRef` field

`extRef:` **shall** be accepted on the frontmatter of **any** element — every `type:` (`Requirement`, `TestCase`, `ADR`, `PartDef`, `Part`, `PortDef`, `Connection`, `FeatureDef`, `Configuration`, `Package`/`_index.md`, …) — as a **common** field alongside `id`, `name`, and `tags` (§3).

- The value **shall** be either a **single string** or a **list of strings**. Each string is one opaque external reference: a URI (`https://dng.example/resources/4521`), a tool-qualified token (`DNG:4521`, `cameo://model/Engine#id-99`), or any other stable handle understood by the external system.
- The field **shall** be **optional**. An element with no `extRef` represents nothing external and is never flagged — the field is strictly opt-in.
- The tool **shall not** constrain the syntax of an external reference (external systems use widely varying identifier schemes); any non-empty string is accepted. A blank/empty entry carries no reference and is ignored.
- `extRef` is an **external** pointer and **shall not** participate in model cross-reference resolution: it is never a valid target for `supertype:`, `subsets:`, `verifies:`, `derivedFrom:`, connections, or any other model-internal reference, and resolving those fields is unchanged.

```yaml
# any element
extRef: "https://dng.example/resources/4521"
# or
extRef:
  - "DNG:4521"
  - "cameo://model/Engine#id-99"
```

## `W028` — duplicate external reference

Because an external artifact normally maps to a single model element, a reference appearing on more than one element usually signals a modelling error (a stray copy, a bad merge). The tool **shall** define a new warning code **`W028`**, emitted when the **same** `extRef` string is declared by **two or more** elements. Enforcement **shall** be:

- **Opt-in / dormant** — emitted only when at least one element declares `extRef`; a model using no external references produces no finding.
- **Per duplicated value** — one finding per colliding `extRef` value, naming the elements that share it.
- **Non-fatal but gateable** — duplicates are permitted (lookup, [[REQ-TRS-EXTREF-002]], returns every match), but `W028` is selectable via `--deny W028` so a project may make external-reference collisions fail the build.

**Source:** external-tool interchange (OSLC-style linkage to DNG / SysML tools); a common-field companion modelled on the optional-field pattern of [[REQ-TRS-IMPL-001]].

**Acceptance criteria:** an element with a single-string `extRef` and an element with a list-valued `extRef` both parse without error; two elements declaring the same `extRef` value produce exactly one `W028` naming both; a unique `extRef` across the model produces none; a model with no `extRef` anywhere produces none (opt-in); `validate --deny W028` exits `2` in the presence of a `W028`.
