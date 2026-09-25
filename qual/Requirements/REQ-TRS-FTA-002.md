---
id: REQ-TRS-FTA-002
type: Requirement
name: Tool shall accept, resolve and surface the FaultTreeEvent ref link to the modelled architecture element
status: draft
reqDomain: software
verificationMethod: test
---

A `FaultTreeEvent` **may** declare `ref:` — a single qualified name or stable id naming the model element (typically a `Part`/`PartDef`) whose failure the event represents. The tool **shall**:

1. Recognise `ref:` as a schema field on `FaultTreeEvent`, so declaring it raises no `W047` (unrecognized frontmatter field).
2. Resolve `ref:` with the standard resolver (qualified name or `id`). A `ref:` that does not resolve to any model element **shall** raise error **`E927`** on the event's file.
3. Surface a resolved link wherever FTA elements are shown: `show <event>` lists `ref`; `links <target>` lists the event as an inbound `ref` source; the element graph carries a `faultTreeEventRef` edge (event → target); `fault-tree render` includes the referenced element in the event's node label.

**Source:** `docs/model-guide/safety-analysis.md` (FaultTreeEvent); GitHub issue #148 — the field was documented and used by the bundled demo models but rejected as unknown.

**Acceptance criteria:** A fixture whose events reference an existing element by qualified name and by id validates with no `W047` and no `E927`, and shows the link in `show`, `links` and `fault-tree render`; a fixture whose event references a non-existent element raises `E927`.
