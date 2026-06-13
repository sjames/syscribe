---
id: REQ-TRS-SAFE-010
type: Requirement
name: "safety-case shall explain [unknown] verdicts when no results sidecar is loaded"
status: draft
reqDomain: software
verificationMethod: test
---

When the `safety-case` command renders a tree that contains at least one `TestCase`
leaf with an `[unknown]` verdict, the output **shall** append a one-line footnote:

```
(verdicts unknown — run `syscribe ingest-results` to populate)
```

This applies to both the text and JSON renders. In JSON the footnote is represented
as a top-level `"verdictsUnknown": true` boolean when any leaf has an unknown verdict.

The footnote **shall not** appear when a results sidecar has been ingested (even if
some verdicts are still unknown due to missing test IDs in the sidecar).

**Acceptance criteria:**

- `safety-case` on a model with TestCase leaves and no results sidecar prints the
  footnote after the last goal block.
- `safety-case --json` on the same model includes `"verdictsUnknown": true` at the
  top level.
- When a results sidecar is loaded (even with partial verdicts), the footnote is
  suppressed.
