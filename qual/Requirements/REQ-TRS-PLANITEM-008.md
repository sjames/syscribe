---
id: REQ-TRS-PLANITEM-008
type: Requirement
name: A PlanningItem may be assigned to a single Unix-style username, checked against an optional project-declared roster mapping username to display name
status: draft
reqDomain: software
verificationMethod: test
---

A `PlanningItem` **shall** accept an optional `assignedTo: <username>` field. The value **shall**
be checked, unconditionally, against a Unix-style username pattern
(`^[a-z_][a-z0-9_-]{0,31}$`) — lowercase, starting with a letter or underscore, then lowercase
letters/digits/underscore/hyphen, 1-32 characters. A project **shall** be able to declare a
`[users]` table in `.syscribe.toml` mapping each valid username to its display name; when that
table is non-empty, an `assignedTo:` value not present as a key in it **shall** be reported as a
validation error, unless the value already failed the format check (reported once, not twice). A
key in `[users]` that is not itself a valid username **shall** be reported as a warning and
excluded from the effective roster.

**Source:** `REQ-TRS-PLANITEM-008` (product model), `ADR-SYS-PLANITEM-001` addendum.

**Acceptance criteria:** a well-formed username assigned and present in a configured roster
validates cleanly; a well-formed username not present in a non-empty roster is rejected; a
malformed value (uppercase, spaces) is rejected regardless of whether any roster is configured, and
is not also reported as "not declared"; a roster configured with one malformed key alongside
well-formed ones still correctly validates the well-formed entries and flags only the malformed
key.
