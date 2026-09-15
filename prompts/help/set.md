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
An optional `rationale=<text>` marks the entry waived.

**`achieves.add <req-id>`** appends to `achieves:` without disturbing existing
order; the target must resolve to a native `Requirement` (mirrors `E714`/`E715`)
or the edit is refused.

Every operation is byte-preserving for the rest of the file — only the touched
key changes, same "surgical edit" bar `move` holds itself to for reference
rewriting.

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
