---
id: REQ-TRS-CFLD-003
type: Requirement
name: Tool shall render custom fields in CLI show and the web detail panel, read-only
status: draft
reqDomain: software
verificationMethod: test
---

Custom fields ([[REQ-TRS-CFLD-001]]) **shall** be surfaced to the reader in both the CLI
and the web UI, **read-only**.

### CLI `show`

- `syscribe -m <root> show <qname>` **shall** render an element's `custom_fields:` (when
  present) as a labelled section listing each key and its value (scalars inline, lists
  comma-joined). When absent, no section is shown.

### Web UI

- The element **detail panel** **shall** render `custom_fields:` (when present) as a
  read-only key/value section.
- Custom fields **shall not** be editable via the `PUT /api/elements/<qname>` editor in
  this version — display only.

**Source:** GH #39 (custom fields rendering; read-only decided).

**Acceptance criteria:**

- `show` on an element with `custom_fields:` prints each key and value; `show` on an
  element without them prints no custom-fields section.
- The web detail panel renders custom fields read-only; the element editor does not
  expose them for editing.
