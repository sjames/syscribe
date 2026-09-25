# set — safe, schema-aware mutation of a small allowlist of fields

## SYNOPSIS
    syscribe -m <root> set <qname|id> status=<value> [--dry-run]
    syscribe -m <root> set <qname|id> evidence.add ref=<id> [rationale=<text>] [--dry-run]
    syscribe -m <root> set <qname|id> evidence.add path=<path> [rationale=<text>] [--dry-run]
    syscribe -m <root> set <qname|id> achieves.add <req-id> [--dry-run]

## DESCRIPTION
Mutates one field on an existing element without hand-editing YAML frontmatter.
Deliberately narrow — only `status=`, `evidence.add`, and `achieves.add` — rather
than the MCP `update_element` tool's arbitrary-field merge: every operation here
validates its own value **before** anything is written, so a typo is caught at
the point of the edit instead of only at the next full-model `validate` (which
might not catch it at all, if the typo doesn't happen to trip an existing rule).

**`status=<value>`** is checked against the target type's own status enum when
it has one (`Requirement`, `TestCase`, `TestPlan`, `ADR`, `PlanningItem`,
`ReviewRecord`) — an out-of-enum value is refused with the allowed list, no file
written. Setting a `PlanningItem` to `status: done` additionally runs the same
`W310` check `validate` would (an `achieves:` Requirement not yet meeting its own
verification bar) and prints it as a warning — non-blocking, matching `W310`'s
own severity; the write still proceeds.

**`evidence.add`** appends one entry to `evidence:` without disturbing existing
order. `ref=<id>` must resolve to some model element (permissive by kind, like
`blockedBy:`); `path=<path>` must exist on disk under the model root, or be an
`http(s)://` URI (accepted as external, same as `sourceFile`/`implementedBy`).
An optional `rationale=<text>` marks the entry waived. If an entry naming the
same target already exists (the same `ref:`, or the same `path:`), the command
reports it and changes nothing — the existing entry, rationale included, is
kept as authored.

**`achieves.add <req-id>`** appends to `achieves:` without disturbing existing
order; the target must resolve to a native `Requirement` (mirrors `E714`/`E715`)
or the edit is refused. Adding an id that is already listed reports it and
changes nothing.

Every operation is a line-level edit, byte-preserving for the rest of the file
— the same "surgical edit" bar `move` holds itself to for reference rewriting.
`status=` replaces only the `status:` line. `achieves.add` and `evidence.add`
insert only the new item's lines after the list's last item, at the list's own
indentation (or append a new block list at the end of the frontmatter when the
field is absent); YAML comments, the quoting of other fields and the existing
entries are left exactly as they were. A list written inline on one line
(`achieves: [REQ-A]`) is rewritten as a block list on that line only; a flow
list spread over several lines is the one layout that falls back to
re-serializing the frontmatter (comments and quoting may then be normalized).
The Markdown body is never touched.

## OPTIONS
    --dry-run        Preview the unified diff without writing.

## EXAMPLES
    syscribe -m model/ set REQ-UAV-NAV-001 status=approved
    syscribe -m model/ set TC-UAV-NAV-001 status=active --dry-run
    syscribe -m model/ set PI-HPLE-001 status=done
    syscribe -m model/ set PI-HPLE-001 evidence.add ref=TC-UAV-NAV-001
    syscribe -m model/ set PI-HPLE-001 evidence.add path=src/nav/controller.rs
    syscribe -m model/ set PI-HPLE-001 achieves.add REQ-UAV-NAV-002

## SEE ALSO
    applies-when, move, show
