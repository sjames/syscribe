---
id: REQ-TRS-OUT-017
type: Requirement
name: Tool shall provide change impact analysis (impact command)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a read-only **`impact`** command (§17, GH #65) that traverses the
traceability graph from a named element and reports every reachable node, its hop distance,
and the edge kind that connects it.

**`syscribe impact <qname|id> [--direction downstream|upstream|both] [--depth <N>] [--format text|json|dot] [--kinds <csv>]`**

- **Downstream** (default) **shall** follow reverse traceability links — `specializedBy`,
  `derivedChildren`, `verifiedBy`, `satisfiedBy`, `refinedBy`, `conditionalOn`,
  `allocatedFrom`, `safetyGoalChildren`. **Upstream** **shall** follow forward links —
  `supertype`, `derivedFrom`, `verifies`, `satisfies`, `refines`, `allocatedTo`,
  `derivedFromSafetyGoal`. `both` does both.
- `--depth N` limits the hop distance; `--kinds csv` restricts traversal to the named base
  kinds (`verifies`, `derivedFrom`, `satisfies`, `supertype`, `appliesWhen`, `allocatedTo`,
  `refines`, `derivedFromSafetyGoal`).
- Output formats: `text` (indented tree with type / status / `via`), `json`
  (`{ root, nodes: [{qname, type, id, depth, via}] }`), and `dot` (valid Graphviz).
- The traversal **shall** be cycle-safe (each element visited once) and **shall** accept the
  root as a qualified name or a stable id.

**Source:** §17 (Change Impact Analysis), GH #65. Read-only; no new element types or
validation rules.

**Acceptance criteria:**

- Downstream from an element reaches its derived children, satisfying elements, and verifying
  tests, with correct `via` labels.
- Upstream from a derived requirement traces back through `derivedFrom` to its
  `derivedFromSafetyGoal` safety goal.
- `--depth` limits the reported hops; `--kinds` restricts the followed edge kinds.
- `--format json` matches the `{ root, nodes }` schema; `--format dot` is valid Graphviz.
- Cycles do not cause non-termination; both qualified names and stable ids work as the root.
