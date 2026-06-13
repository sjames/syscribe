---
id: REQ-TRS-OUT-015
type: Requirement
name: "list AssumptionOfUse shall emit SRAC-oriented columns and include appliesTo + body in JSON"
status: draft
reqDomain: software
verificationMethod: test
---

When the `list` command's type filter is `AssumptionOfUse`, the tool **shall** produce
output specialized for safety-manual SRAC tables.

## Human-readable table

The table **shall** contain the columns: `ID | Name | Applies To | Status`.

- **ID** — the stable `AOU-*` identifier.
- **Name** — the `name:` label.
- **Applies To** — comma-joined `appliesTo:` list, or `—` if empty.
- **Status** — the `status:` value.

## JSON output (`--json`)

In addition to the base fields (`qualifiedName`, `type`, `name`, `id`, `status`), an
AssumptionOfUse element **shall** include:

- `appliesTo` — array of target reference strings (empty array if absent).
- `body` — the Markdown body text of the element (trimmed), or `null` if empty.

These two fields give a documentation pipeline everything needed to generate a
safety-manual SRAC section programmatically, eliminating hand-maintained copy.

**Acceptance criteria:**

- `list AssumptionOfUse` produces a table with columns `ID | Name | Applies To | Status`.
- `list AssumptionOfUse --json` includes `appliesTo` (array) and `body` (string or null).
- An AOU with `appliesTo: [SG-X, SG-Y]` shows both in the table's Applies To column.
- `list Requirement` and other types still use the generic table (no regression).
