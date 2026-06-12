---
id: REQ-TRS-DISC-005
type: Requirement
name: Tool shall provide a why-active command explaining an element's activity in a projection
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a `why-active <qname|id> --config <CONF|features>` subcommand that explains whether the named element is active in the given projection, and **why**. The element argument **shall** be accepted as either a qualified name or a stable id; `--config` **shall** accept a `Configuration` element (by qualified name or id) or an inline feature-selection set.

For the resolved element the tool **shall** print:

- the element's `appliesWhen:` expression;
- the configuration's selections for each `FeatureDef` that the expression references;
- the boolean verdict — **active** or **inactive** — obtained by evaluating the expression against those selections (truth-table semantics per [[REQ-TRS-VAR-003]]).

The command **shall** observe these rules:

- an element with **no** `appliesWhen:` is reported as **"always active"**;
- `--config` is **required** — omitting it is an error (clear message, non-zero exit);
- an **unresolved** configuration (unknown `Configuration` / malformed selection set) is an error (non-zero exit);
- when the model has **no** feature model (no `FeatureDef`), every element is reported as **always active**.

The command **shall** be discoverable in `--help`.

## Rationale

When an element is unexpectedly present or absent in a projected variant ([[REQ-TRS-PROJ-001]]), an engineer needs to see the reasoning, not just the outcome. Showing the `appliesWhen:` expression alongside the configuration's selections of exactly the referenced features, and the resulting verdict, turns "why is this here / missing?" into a self-contained, auditable explanation.

**Source:** §9 (PLE); product-line feature discoverability; explains the projection lens from [[REQ-TRS-PROJ-001]].

**Acceptance criteria:** `why-active <element> --config <CONF>` prints the element's `appliesWhen:` expression, the referenced features' selections in that configuration, and an active/inactive verdict consistent with truth-table evaluation; an element with no `appliesWhen:` is reported "always active"; omitting `--config` is an error with non-zero exit; an unresolved configuration is an error with non-zero exit; on a model with no `FeatureDef` the element is reported "always active"; the command is listed in `--help`.
