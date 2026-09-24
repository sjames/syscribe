---
type: ADR
id: ADR-SYS-PKG-001
name: "Package membership is generated from the directory, never hand-maintained in _index.md prose"
status: accepted
tags:
  - packages
  - documentation
---

## Context

A package's containment is defined by the directory tree (§ directory/namespace convention), but
authors routinely enumerate a package's members in the prose body of its `_index.md`. Nothing ties
that prose to the files, so it drifts silently: `Requirements/PlanningItem/_index.md` listed
`REQ-TRS-PLANITEM-001`…`006` while the directory held `000`…`009`, and described capabilities that
had since changed (GH #120). Readers and LLM agents open `_index.md` first to orient themselves, so a
stale list actively misleads.

## Decision

- **Membership is derived, never authored.** `show` on a package, the web UI detail panel and
  `export-html` package pages render the package's direct members (id or qualified name, type,
  name, status) from the directory at load time. `ls` already does.
- **`_index.md` prose states purpose and scope** — the *why*, not the *what*.
- **A lint hint discourages hand-maintained lists.** `lint-docs` reports `W103` when an
  `_index.md` body enumerates three or more distinct stable ids that resolve to that package's own
  direct members. Advisory: it does not change `lint-docs`'s exit status and `validate` is unaffected.
- The bundled model's enumerating `_index.md` files are rewritten to the new style.

## Alternatives considered

- **Validate prose ids against the model** (warn on dangling ids / unmentioned children): catches
  drift but keeps two sources of truth and makes every new file a docs chore.
- **A structured `members:` frontmatter list**: checkable, but still a hand-maintained duplicate
  of the directory.

## Consequences

- No new validation finding; bundled model `validate` output is unchanged.
- Package pages in every surface always reflect the files on disk.
