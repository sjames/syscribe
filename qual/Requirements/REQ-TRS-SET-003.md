---
id: REQ-TRS-SET-003
type: Requirement
name: Tool shall make set evidence.add idempotent and edit list fields without disturbing other frontmatter lines
status: draft
reqDomain: software
verificationMethod: test
---

Refines `REQ-TRS-SET-002`.

**Idempotence.** `set <qname|id> evidence.add` **shall** be a reported no-op
when the target's `evidence:` list already has an entry naming the same target
— the same `ref:` for `ref=<id>`, the same `path:` for `path=<path>`. The
command exits 0, reports that the entry is already present, and leaves the
file byte-identical, including the existing entry's `rationale:`. This matches
`achieves.add` on a requirement that is already listed.

**Surgical edit.** `evidence.add` and `achieves.add` **shall** change only the
lines of the list they append to. The new item's lines are inserted after the
list's last item, at that list's own indentation, or a new block list is
appended at the end of the frontmatter when the field is absent. Every other
frontmatter line **shall** be preserved byte-for-byte, including full-line and
trailing YAML comments, the quoting of unrelated fields and the existing list
items. A list written inline on one line (`[]`, a flow list, a scalar) is
rewritten as a block list on that line only.

**Source:** GH issue #152.

**Acceptance criteria:** `evidence.add ref=<id>` / `path=<path>` for an
already-present target exits 0, reports it, and writes nothing; adding a new
evidence entry or `achieves:` item to a frontmatter carrying YAML comments
leaves every original line unchanged and adds exactly the new item's lines.
