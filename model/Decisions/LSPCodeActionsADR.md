---
type: ADR
id: ADR-SYS-LSP-003
name: "LSP v3 — display-only codeLens counts, plus guarded codeAction quick-fixes for E310 and W090"
status: accepted
tags:
  - lsp
  - editor
  - tooling
---

## Context

`syscribe lsp` v1 (`ADR-SYS-LSP-001`) shipped navigation; v2 (`ADR-SYS-LSP-002`) proposes
completion and rename. The remaining v1-scoped phase is inline affordances: `codeLens` (glance
counts above an element without opening `references`) and `codeAction` (one-click fixes for
specific diagnostics). Both need scoping decisions the earlier ADRs left open.

Axes evaluated:

- **CodeLens click behavior** — attach a `command` (jump to references) vs. display-only counts
  with no action.
- **CodeAction scope** — which diagnostics get a quick-fix, and how ambiguity is resolved when
  a fix isn't uniquely determined (e.g. *which* ADR to set as `breakdownAdr:` when several
  accepted ADRs exist).
- **CodeAction write mechanism** — a direct `WorkspaceEdit` (client applies, like v2's rename)
  vs. a server-executed `command` via `workspace/executeCommand` (server writes to disk itself).
- **Confirmation before a write-side-effect codeAction runs** — an explicit dry-run round trip
  (mirroring `mcp`'s `dry_run`-by-default writes) vs. none.

## Decision

**CodeLens** (`textDocument/codeLens`, computed eagerly from the in-memory store, no
`resolveProvider`): **display-only.** Each `Requirement`/`TestCase`/element with incoming
references gets a lens line reporting counts (e.g. `2 verifiedBy · 1 derivedChildren`) and any
`W090` suspect-link count; no lens carries a `command`.

**CodeAction** (`textDocument/codeAction`, `codeActionKinds: ["quickfix"]`), scoped to exactly
two diagnostics:

- **`E310`** (`Requirement` with `derivedFrom:` missing `breakdownAdr:`) — one `CodeAction` per
  `accepted` ADR currently in the model, each a direct `WorkspaceEdit` inserting
  `breakdownAdr: <that ADR's qname>` into the requirement's frontmatter. If the model has zero
  accepted ADRs, no action is offered (nothing sensible to insert). This is the one **write via
  `WorkspaceEdit`** case in v3: it's a pure text insertion, not a disk-mutating side effect.
- **`W090`** (stale suspect link) — one `CodeAction`, "Accept as reviewed," delivered as a
  `command` (not a `WorkspaceEdit`) dispatched through `workspace/executeCommand`
  (`executeCommandProvider`, command `syscribe.suspectAccept`). Executing it calls
  `crate::suspect::plan_accept` + `crate::suspect::write_baseline` — the same functions `mcp`'s
  guarded `suspect_accept` tool already calls — then reloads the store and republishes
  diagnostics. No dry-run step: unlike `mcp` (an LLM may invoke tools with no human review in
  the loop, which is exactly what `dry_run`-by-default guards against), a `codeAction` requires
  the user to open the lightbulb menu and click the specific action — that click **is** the
  human-in-the-loop confirmation.

No other diagnostics get a v3 quick-fix.

## Rationale

**Display-only codeLens over a click action.** A codeLens jump-to-references command would
just duplicate `textDocument/references` (already shipped in v1) behind an extra click, only on
clients that support it — zero net new capability for real added complexity (either a
client-native command like VSCode's `editor.action.showReferences`, which a generic LSP client
can't be assumed to support, or a `window/showDocument` round trip to reposition the cursor,
which is 3.16+-only). The counts alone are the useful part; drilling in is already one
"Find References" away.

**Two diagnostics, not a general quick-fix framework.** `E310`/`W090` were picked because each
has an unambiguous, mechanically-derivable fix (or a small, enumerable set of fixes, in E310's
case). Other errors either have no single correct auto-fix (most validator errors require
human judgment about *what the author meant*) or belong to a future phase; scoping codeAction
to a hand-picked pair keeps this phase small and each fix trustworthy, rather than shipping a
framework whose long tail is half-finished. (An earlier, looser sketch of this phase also
mentioned a "stub a Baseline" quick-fix; dropped here — creating a `Baseline` isn't a fix *for*
any diagnostic, so it has no natural codeAction trigger and doesn't belong in this phase.)

**`WorkspaceEdit` for E310, `executeCommand` for W090 — because the two fixes are different
shapes of change.** E310's fix is a pure, client-applicable text insertion with no side effects
beyond the edit itself, so it fits the same `WorkspaceEdit` contract v2's rename uses. W090's
fix (`suspect accept`) computes and writes a content hash — a real side effect the client isn't
positioned to compute — so it has to run server-side, and `workspace/executeCommand` is the
standard LSP mechanism for that, not a custom protocol extension.

**No dry-run for codeAction writes.** `mcp`'s `dry_run`-by-default exists because tool calls can
happen with no human reviewing each one. A codeAction only ever runs after a person opens the
lightbulb and picks it — adding a second confirmation round-trip over LSP would slow down the
one workflow this phase exists to speed up, for no safety benefit `suspect accept`'s own
idempotence doesn't already provide (re-running it just re-baselines to the current hash).

## Consequences

- New capabilities advertised: `codeLensProvider` (no `resolveProvider`), `codeActionProvider`
  (`codeActionKinds: ["quickfix"]`), `executeCommandProvider` (`commands: ["syscribe.suspectAccept"]`).
- `syscribe.suspectAccept`'s implementation reuses `crate::suspect::plan_accept` and
  `crate::suspect::write_baseline` directly (already `pub(crate)`-visible and already called
  from `mcp/mod.rs`'s `suspect_accept` tool) — no new suspect-link write logic.
- E310's insertion reuses `frontmatter_range`-style line location (from v1's diagnostics) to
  find where to insert, not the full `mcp` frontmatter writer (a single-line insertion doesn't
  need it).
- After `syscribe.suspectAccept` runs, the store must reload and diagnostics must republish for
  all open documents — the same path `REQ-TRS-LSP-007`'s `didSave`/`didChangeWatchedFiles`
  reload already uses, invoked directly rather than waiting for a filesystem-watch round trip.
- Follow-on requirements: `REQ-TRS-LSP-013`..`REQ-TRS-LSP-016`.
