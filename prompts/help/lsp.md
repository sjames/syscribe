# lsp — run a Language Server Protocol server over stdio for editors

`syscribe -m <root> lsp` starts a Language Server (LSP) that speaks
`Content-Length`-framed JSON-RPC 2.0 over **stdio**. It lets an LSP-capable editor
(VSCode and others) navigate and validate the Syscribe model bound at `-m`.

## SYNOPSIS
    syscribe -m <root> lsp

## USAGE

The server runs until it receives `shutdown` followed by `exit` and the client
closes stdin. It is intended to be spawned by an editor's LSP client, not
invoked interactively. It speaks only standard LSP methods — no custom
(non-LSP) protocol extensions (ADR-SYS-LSP-001) — so any LSP-capable client
works, not just a purpose-built extension.

## Capabilities

Advertised in `initialize`:


- **Diagnostics** (`textDocument/publishDiagnostics`) — the model validator's
  findings (`E***`/`W***`), republished on `textDocument/didOpen` and after a
  full model reload.
- **Go to definition** (`textDocument/definition`) — resolves the qualified
  name or stable id under the cursor to its defining file.
- **Find references** (`textDocument/references`) — every element whose
  frontmatter cross-references the element under the cursor.
- **Hover** (`textDocument/hover`) — a resolved summary (type, id, qname,
  status) for the qualified name or stable id under the cursor.
- **Workspace symbol search** (`workspace/symbol`) — find elements by name,
  id, or qualified name across the whole model.
- **Completion** (`textDocument/completion`, ADR-SYS-LSP-002) — field-aware
  id/qname candidates inside cross-reference fields (`supertype`/`subsets`/
  `redefines` → same type, `typedBy` → the matching `*Def`, `satisfies`/
  `verifies`/`derivedFrom` → `Requirement`, `breakdownAdr` → `ADR`), plus enum
  candidates for `type:` and `status:`.
- **Rename** (`textDocument/prepareRename` + `textDocument/rename`) — rename a
  stable id (id-identified elements only) and every reference to it. The server
  never writes: it returns a `WorkspaceEdit` for the client to apply, after
  validating the result in memory (a malformed or colliding id is refused).
  Renaming a qualified name is a file move — use `move` or the MCP
  `move_element` tool.
- **Code lens** (`textDocument/codeLens`, ADR-SYS-LSP-003) — a display-only lens
  on the frontmatter: `N verifiedBy · N derivedChildren · N suspect links`.
- **Code actions** (`textDocument/codeAction`, quick fixes) — for `E310` (a
  derived Requirement with no `breakdownAdr:`), one "Set breakdownAdr: …" edit
  per accepted ADR; for `W090` (suspect link), an "Accept as reviewed" command.
- **Execute command** (`workspace/executeCommand`) — `syscribe.suspectAccept
  <source> <target>` re-baselines one suspect link (the only LSP path that
  writes to disk), then reloads and republishes diagnostics.
- **Capabilities handshake** (`experimental`) — the `initialize` result carries the document of
  `syscribe capabilities --json` under `capabilities.experimental.syscribe`: tool version, commands and
  options, the published JSON contracts, optional capabilities (`stagedConfiguration`, …), MCP tools and the
  namespaced `syscribe/*` custom requests (ADR-SYS-LSP-004). A front end tests the capability it needs.
- **Change notification** (`syscribe/modelChanged`, GH #415) — sent to a client that set
  `capabilities.experimental.syscribe.modelChanged = true` in `initialize`. After a reload it lists the
  qualified names `added`, `changed` and `removed` and whether the feature model, any Configuration
  (`configurationsChanged`) or the set of validation findings (`validationChanged`) changed, so a panel
  refreshes only what moved. A burst of reloads is reported once after a quiet period (150 ms; override with
  `SYSCRIBE_LSP_NOTIFY_DEBOUNCE_MS`), a burst that ends where it began is not reported, and `sequence`
  increases by one per notification. The schema is `syscribe schema model-changed`.
- **Propagation request** (`syscribe/propagate`, GH #416) — read-only; params `{from?, select?, stage?}`, result
  the `propagate` payload (`syscribe schema propagate`): features forced on, forced off or free for a partial
  selection, without writing a file.
- **Explanation request** (`syscribe/explainFeature`, GH #417) — read-only; params `{feature, from?, select?}`,
  result the `explain-feature` payload (`syscribe schema explain-feature`): a feature's state for a partial
  selection, the choices and constraints (with kind, declaring feature and file) that cause it, and what would
  have to change to select it.
- **Tree request** (`syscribe/children`, GH #425) — read-only; params `{of?, view?, offset?, limit?}`, result the
  `children` payload (`syscribe schema children`): the direct children of a node with counters, one page per call,
  for the containment tree or the Configuration tree.
- **Impact request** (`syscribe/changeImpact`, GH #423) — params `{element, set?, parameter?}`; the edit is evaluated
  on a candidate copy and nothing is written. Result the `change-impact` payload (`syscribe schema change-impact`):
  per Configuration, validity before and after and projection changes.
- **Timeline request** (`syscribe/timeline`, GH #418) — read-only; params `{configuration}`, result the `timeline`
  payload (`syscribe schema timeline`): the `derivedFrom:` chain with what each stage decided, the effective values
  with provenance and what is still open.
- **Product-line completion** (GH #419) — `textDocument/completion` also knows the variability fields: feature
  qualified names and `FEAT-*` ids in `appliesWhen:`, a Configuration's `features:` keys, `requires:` and
  `excludes:`; parameter paths in `parameterBindings:` keys and `parameterConstraints:` expressions; the
  `enumValues:` of the parameter being bound; and the stage names of the binding-time vocabulary for `bindingTime:`,
  `latestBindingTime:` and `bindingStage:`. Items are documented (type, unit, range, default, window) and replace
  the whole token under the cursor.
- **Variability hover and lenses** (GH #420) — hovering `appliesWhen:` lists the Configurations the element is
  active in; hovering a `parameterBindings:` or `parameterNarrowing:` key of a Configuration shows the parameter's
  type, unit, range, window and default, the stage it is bound at and the parameter constraints it takes part in
  with their state. A code lens above a gated element reads `active in N of M configurations`; one on each
  parameter of a `FeatureDef` reads `bound at <stage> by <Configuration>` or `unbound — default …`.
- **Product-line quick fixes** (GH #421) — `textDocument/codeAction` also offers a fix for `E222` (the closest
  declared parameter), `E754` (remove the rebinding), `E751` (a valid `bindingStage`), `E752` (remove the binding or
  selection the stage cannot make), `E756` (a value inside what was narrowed), `W016` (`selections:` → `features:`)
  and `E201` on a `FeatureDef` (a derived `FEAT-*` id). Each is validated in memory first and offered only if the
  diagnostic goes away and no error appears; the server never writes the file.
- **Semantic tokens** (`textDocument/semanticTokens/full`, GH #422) — feature, parameter and stage references in the
  variability fields (`appliesWhen:`, a Configuration's `features:` and `parameterBindings:` keys, `requires:`,
  `excludes:`, a constraint's `expression:`, `bindingTime:`/`bindingStage:`). Legend, token types by index:
  0 `class` (a feature by qualified name), 1 `enumMember` (a feature by `FEAT-*` id), 2 `property` (a parameter
  path `<feature>.<parameter>`), 3 `keyword` (a binding-time stage). Modifier bit 0 `unresolved` marks a reference
  that names nothing in the model, so a typo shows before validation runs.
- **Health request** (`syscribe/health`, GH #428) — read-only; result the `health` payload
  (`syscribe schema health`) for a status-bar item, kept until the next reload.

The model state is disk-backed and reloads in full on `textDocument/didSave`
and `workspace/didChangeWatchedFiles` — `textDocument/didChange` alone never
triggers a reload or diagnostics (the server validates saved content, not
unsaved buffers).
