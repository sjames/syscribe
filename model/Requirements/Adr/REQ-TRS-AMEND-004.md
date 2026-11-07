---
type: Requirement
id: REQ-TRS-AMEND-004
name: "Amendments are written by set and shown by show and export-html"
status: draft
reqDomain: software
reqClass: system
tags:
  - adr
  - amendments
---

The tool shall let an author record an amendment safely and shall show the amendment history (GH #454).

## Behavior

- `set <id> amendment.add kind=… reason=… [ref=…] [by=…] [date=…]` appends an entry; `date` defaults to today and `by` to `--by`. It refuses a non-ADR/Requirement target, a bad kind, an empty reason, a missing author, a bad date or an unresolved `ref`, writing nothing; a scope-change without an owner reference is written with a `W325` note.
- `show` prints an `## Amendments` table (date, kind, by, reason, ref); `export-html` renders the same table on the element page.
- Links to an amended element go suspect (`W090`) through the text edit only, never through the amendment entry.
- `template ADR`/`template Requirement`, `help ADR`/`help Requirement` and `help --search amend` teach the field.
