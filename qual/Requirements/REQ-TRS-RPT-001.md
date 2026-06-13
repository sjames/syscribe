---
id: REQ-TRS-RPT-001
type: Requirement
name: "fmea report command shall emit a human-readable FMEA sheet with RPN sorted rows"
status: draft
reqDomain: software
verificationMethod: test
---

The `fmea report` sub-command **shall** render a human-readable FMEA report for a
given model directory.

## Output format

The report **shall** be a Markdown table with columns:

`ID | Name | Failure Mode | Effect | Severity | Occurrence | Detection | RPN | Controls | Status`

- Rows **shall** be sorted by RPN descending (highest risk first).
- RPN **shall** be the computed value (see REQ-TRS-FMEA-002 auto-compute rule).
- **`--json`** flag **shall** emit a JSON array of FMEA entry objects, each carrying the
  same fields.

## Scope filter

`--fmea-sheet <id>` **shall** restrict output to entries within the named `FMEASheet`.
Without this flag, all `FMEASheet` elements are included.

## Fault-tree render command

The `fault-tree render` sub-command **shall** emit a Mermaid flowchart string for a
named `FaultTree` element.

- `fault-tree render <id>` produces a Mermaid `flowchart TD` diagram of the fault-tree
  structure rooted at the given `FaultTree`.
- Nodes show event type, event id, and description label.
- Gate type (`AND`, `OR`) is reflected in connecting edges (label or node shape).

**Acceptance criteria:**

- `fmea report` prints a Markdown table with correct columns.
- Rows are sorted by RPN descending.
- `fmea report --json` emits a JSON array.
- `fmea report --fmea-sheet FM-KERN` restricts output to that sheet's entries.
- `fault-tree render FT-KERN-001` prints a valid Mermaid `flowchart TD` string.
