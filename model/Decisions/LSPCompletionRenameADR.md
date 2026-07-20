---
type: ADR
id: ADR-SYS-LSP-002
name: "LSP v2 — field-aware id/qname and enum completion, plus safe cross-file rename via WorkspaceEdit"
status: accepted
tags:
  - lsp
  - editor
  - tooling
---

## Context

`syscribe lsp` v1 (`ADR-SYS-LSP-001`) shipped read-only navigation: diagnostics, definition,
references, hover, workspace/symbol. Authors still hand-type cross-reference values
(`verifies: REQ-...`, `supertype: ...`) with no completion, and renaming a stable id today
means grepping every referencing file and editing each by hand — exactly the error-prone,
manual cross-reference maintenance the format's resolver already automates for reading, but
not yet for writing.

`ADR-SYS-LSP-001`'s Consequences section sketched v2 rename as reusing "the `mcp` write path's
frontmatter round-trip" without working through what that means for an LSP server. On closer
look that framing doesn't fit: `mcp`'s guarded writes commit directly to disk (via a temp-copy
candidate, a referential-integrity gate, then an atomic swap) because an MCP client has no
notion of open buffers to keep in sync. An LSP client does — VSCode and friends expect the
**server to propose an edit** (a `WorkspaceEdit`) and the **client to apply** it, so it lands
in open buffers, participates in undo, and never races an unsaved edit the user is mid-typing
in some other open file. This ADR corrects that and settles the real design.

Axes evaluated:

- **Completion candidate scope** — every id/qname in the model, unfiltered, vs. field-aware
  filtering (e.g. `verifies:` should only suggest `Requirement` ids, `breakdownAdr:` only
  accepted-or-any `ADR` ids).
- **Completion candidate source for enums** (`type:`, `status:`) — a new hand-maintained table
  in the `lsp` module vs. reusing whichever domain source already backs `mcp`'s
  `describe_type`/`template` tools.
- **Rename delivery mechanism** — the server writes to disk directly (mirroring `mcp`) vs. the
  server computes a `WorkspaceEdit` and returns it for the **client** to apply.
- **Rename scope** — stable ids (`REQ-*`, `TC-*`, `ADR-*`, `FEAT-*`, `BL-*`, + configured
  `[ids.prefixes]`) only, vs. also qname/name-identified elements (whose "rename" is really a
  file move — `mcp`'s `move_element` territory).
- **Rename safety gate** — return the edit unconditionally vs. validate the candidate first and
  refuse if it would introduce a new unresolved reference, mirroring `mcp`'s guarded-write
  philosophy.
- **`textDocument/prepareRename`** — implement it (seeds the client's rename input with the
  current identifier, rejects non-renameable positions before the user types anything) vs.
  skip it and let `rename` fail blind on a bad position.

## Decision

**Completion** (`textDocument/completion`, no trigger characters, no `resolveProvider` — every
candidate is fully populated upfront from the already-in-memory store):

- **Field-aware, type-filtered candidates** for a bounded, explicit table of known
  cross-reference fields (`supertype`, `subsets`, `redefines`, `typedBy`, `derivedFrom`,
  `verifies`, `satisfies`, `breakdownAdr`, `implementedBy`). Each maps to the element type(s)
  a valid value must resolve to; completion only offers ids/qnames of that type. This table is
  explicit rather than generic (unlike v1's field-agnostic reference search in
  `REQ-TRS-LSP-004`) because completion needs to know the *expected* type to filter by, which a
  generic value-walk cannot infer.
- **Enum-value candidates** for `type:` (`ElementType` variants) and `status:` (the valid
  statuses for the enclosing element's type), sourced from whatever domain table already backs
  `mcp`'s `describe_type`/`template` tools — not a second hand-maintained copy.

**Rename** (`textDocument/prepareRename` + `textDocument/rename`):

- **Scoped to stable ids only.** Renaming a qname/name-identified element is a file move, not a
  text edit, and stays `mcp move_element`'s job; `lsp` rename is refused (a clear error) on a
  name-identified position.
- **`prepareRename`** resolves the token under the cursor exactly as v1's hover/definition do;
  it succeeds (returning the current id as the default rename range) only on a stable-id
  position, giving the client an early, position-level accept/reject before any typing.
- **`rename` returns a `WorkspaceEdit`; the server never writes to disk.** The edit rewrites the
  target element's own `id:` line plus every referencing file's matching occurrence(s) (the
  same needle-matching v1's `references` uses, generalized from a boolean check into a text
  edit). The **client** applies it — to open buffers and unopened files alike, per the
  `WorkspaceEdit` contract — and is responsible for saving.
- **Validated before it's offered.** Before returning the edit, the server clones the in-memory
  element list, applies the rename transform to the clones, and re-runs `validate_with_config`
  against that candidate. If the rename would introduce a *new* unresolved-reference finding,
  `rename` returns an LSP error (not an edit) naming the reason — the same referential-integrity
  gate `mcp`'s guarded writes use, but done entirely in memory (the store already holds parsed
  elements; no disk temp-copy is needed, since nothing is committed here).

## Rationale

**Field-aware over unfiltered completion.** An unfiltered id/qname list is cheap to build but
noisy — the completion list quickly becomes hundreds of irrelevant entries in a model this
model's size. A bounded, explicit field table costs little to write and is trivially extensible
as new cross-reference field kinds are added; the alternative (inferring expected type from
context) has no reliable signal to infer from.

**WorkspaceEdit over server-side disk write.** This is the actual course-correction this ADR
makes. A direct disk write (à la `mcp`) is right for an LLM client with no open-buffer notion,
but for an editor it bypasses undo/redo, risks clobbering an unsaved edit sitting in another
open buffer, and isn't how any mainstream LSP client expects `rename` to behave — VSCode's
rename UI is built entirely around receiving and previewing a `WorkspaceEdit`. Matching the
spec's actual contract here isn't optional if the goal is "works in real editors."

**Stable-ids-only over also renaming qnames.** A qname rename is inseparable from a file
move (the qname *is* the path). Folding that into `textDocument/rename` would mean emitting a
`ResourceOp::Rename` file-move operation inside the `WorkspaceEdit` alongside text edits — LSP
supports this, but it duplicates `mcp move_element`'s validated-move logic in a second place
for no clear benefit yet. Deferred rather than half-built now.

**Validate-then-offer over return-unconditionally.** Returning an edit that would silently
break a cross-reference elsewhere defeats the entire point of a resolver-aware rename — the
author might as well have used a plain text editor's find/replace. In-memory revalidation is
cheap (the store is already parsed) and requires no new disk I/O machinery.

## Consequences

- **Supersedes** `ADR-SYS-LSP-001`'s Consequences note that "v2's rename reuses the `mcp` write
  path's frontmatter round-trip" — that undersold the design. Rename does not write to disk,
  does not use `mcp`'s temp-copy candidate machinery, and does not reuse `mcp`'s writer; it
  computes an in-memory candidate, validates it, and returns a `WorkspaceEdit`.
- New capabilities advertised: `completionProvider` (no trigger characters, no
  `resolveProvider`), `renameProvider` (`prepareProvider: true`).
- A small field→target-type table is added inside the `lsp` module. It is not shared with
  `mcp` or `syscribe-model` yet; if the model format grows a first-class "cross-reference field
  schema," this table should be replaced by it rather than hand-maintained twice in parallel.
- Follow-on requirements: `REQ-TRS-LSP-008`..`REQ-TRS-LSP-012`.
