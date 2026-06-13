---
id: REQ-TRS-OUT-018
type: Requirement
name: Tool shall report behavioral coverage of TestCases over behavioral elements
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a read-only **`behavioral-coverage`** command (§20, GH #72) that
reports how completely the **active** `TestCase` elements exercise the behavioral elements
(`ActionDef`, `Action`, `StateDef`, `State`) in scope.

**`syscribe behavioral-coverage [<qname>] [--depth <N>] [--format text|json] [--uncovered-only] [--include-planned]`**

A behavioral element `B` is **covered** by an active `TestCase` `TC` when any of four paths
holds:

1. **Source overlap** — `TC.sourceFile` is under a path in `B.implementedBy`.
2. **Requirement chain** — `TC.verifies` → a requirement satisfied by an element that is
   `typedBy:` or `supertype:` `B`.
3. **Test function** — a `TC.testFunctions[].file` is under `B.implementedBy`.
4. **Allocation** — the satisfying element is `allocatedTo:` `B`.

- Only `active` TestCases contribute by default; `--include-planned` adds
  `draft`/`review`/`approved` TestCases in a separate **planned** column/field.
- `--uncovered-only` shows only uncovered elements; `--depth` limits the namespace depth
  under `<qname>`.
- Output: `text` (element / type / covered ✓✗ / test ids + a coverage % summary) and `json`
  (`{ scope, covered, total, coverage_pct, elements: [{qname, type, covered, coveredBy}] }`).
- The coverage percentage **shall** be `covered / total × 100`.

**Source:** §20 (Behavioral Coverage), GH #72. Read-only; no new element types or rules.

**Acceptance criteria:**

- An active TestCase whose `sourceFile` is under a behavioral element's `implementedBy`
  marks it covered (path 1).
- A behavioral element `allocatedTo:` by an element that satisfies a verified requirement is
  covered (path 4).
- Only active TestCases count by default; `--include-planned` surfaces planned coverage.
- `--uncovered-only` filters to uncovered elements; the coverage percentage is correct.
- `--format json` matches the schema; cycles do not cause non-termination.
- The demo model (`model/`) achieves **> 50%** behavioral coverage out of the box.
