---
type: Requirement
id: REQ-TRS-SCBACK-001
name: "safety-case text prints a back-reference for a subtree already expanded"
status: draft
reqDomain: software
reqClass: system
tags:
  - safety
---

The text output of `safety-case` shall not repeat a subtree it has already printed (GH #247).

## Behavior

- Within one goal's tree, a Requirement or Argument node that has children and was already expanded earlier is printed once more as its own line followed by ` (see above)`, without its children.
- Leaf nodes (for example a TestCase) are printed wherever they occur.
- The JSON output (which keeps the full tree), the DOT and Mermaid outputs (which already draw each node once) and the completeness counts are unchanged.
