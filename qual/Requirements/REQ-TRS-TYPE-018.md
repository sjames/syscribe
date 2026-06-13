---
id: REQ-TRS-TYPE-018
type: Requirement
name: Tool shall recognise and validate the native ReviewRecord element (RR-*) — thin, pointer-based
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise a native **`ReviewRecord`** element (§19, GH #71) that captures a
formal review event (design review, requirements review, hazard review, TRR, inspection,
walkthrough) and links it to the model elements it covers — a **baselined, version-controlled
traceability anchor**, deliberately **thin**: a `recordedAt:` URI points to the external
review (e.g. a GitHub PR/review), so the discussion stays in the review tool while the model
keeps the auditable anchor and the coverage gate.

**ID pattern:** `^RR(-[A-Z0-9]{2,12})+-[0-9]{3,8}$` (id-identified; files are `<RR-id>.md`).

**Frontmatter:** `id` (RR-*), `name` (free-prose label), `status` (`open|closed|waived`),
`reviewType` (`design_review|requirements_review|hazard_review|test_readiness_review|inspection|walk_through`),
`reviews:` (≥1 element qnames/IDs covered), optional `reviewDate`, `reviewedBy`, `recordedAt`
(external URI), and `items:` (action items with `disposition: open|closed|not_applicable` and
optional `closedBy`).

### Validation rules

| Code | Condition |
|---|---|
| `E700` | Missing `id`, `name`, `status`, `reviewType`, or `reviews` (≥1). |
| `E701` | `id` does not match the `RR-*` pattern. |
| `E702` | `status` not in `open \| closed \| waived`. |
| `E703` | `reviewType` not in the allowed enum. |
| `E704` | A `reviews:` entry does not resolve to a known element. |
| `E705` | An `items[].disposition` is not `open \| closed \| not_applicable`. |
| `W700` | A `status: closed` review has an `items[]` entry with `disposition: open`. |
| `W704` | A non-`draft` native `Requirement` appears in no `ReviewRecord.reviews:` list — dormant unless the model uses ReviewRecords; opt-in, gateable with `--deny W704`. (Drafted as `W701`, which is already in use.) |

### CLI

The tool **shall** provide `reviews [<qname>] [--open-only] [--json]` (list, optionally
filtered to reviews with open items or covering a given element), `review <RR-id> [--json]`
(detail), and `reviews --coverage [<qname>] [--json]` (element → review cross-table). The
`template ReviewRecord` skeleton **shall** be available.

> **Code note:** the spec drafted the coverage warning as `W701`, already in use (Requirement
> asilLevel needs verificationMethod). It is reassigned to **`W704`** and the spec §19
> corrected. `E700`–`E705`/`W700` were free.

**Source:** §19 (Review Records), GH #71. Thin/pointer design confirmed: the model holds the
baselined anchor + coverage gate; review content lives in the external tool via `recordedAt`.

**Acceptance criteria:**

- A well-formed `ReviewRecord` validates clean; `E700`–`E705` fire on the matching defects.
- `W700` fires when a closed review has an open action item.
- `W704` fires for a non-draft requirement covered by no review (only when ReviewRecords
  exist); a covered requirement does not.
- `template ReviewRecord` produces a valid skeleton; `reviews`, `review <id>`, and
  `reviews --coverage` produce correct output.
- Shipped models (no ReviewRecords) are clean.
