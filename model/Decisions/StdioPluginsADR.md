---
type: ADR
id: ADR-SYS-PLUGIN-002
name: "Foreign-format ingestion via stdio-subprocess plugins: read-only, JSON-in/JSON-out, no sandbox"
status: accepted
tags:
  - plugins
  - stdio
  - interop
---

## Context

Syscribe models are Markdown+YAML by design, but a team may already have a subsystem authored in
some other, genuinely arbitrary custom notation — not SysMLv2, not anything this project could
reasonably ship a native parser for. There was no way for a directory inside the model tree to be
authored in such a notation while still participating in Syscribe's traceability graph: a
`Requirement`'s `derivedFrom:`, an `Allocation`, a `satisfies:`/`verifies:` link could not target an
element that lived outside the native parser's understanding.

A near-identical problem was already designed and even implemented — using a *sandboxed WASM
(Extism/wasmtime)* plugin runtime instead of a subprocess — on an unmerged branch
(`feat/wasm-plugins`, commit `ac1519b`, `ADR-SYS-PLUGIN-001`, only present on that branch as
`model/Decisions/WasmPluginsADR.md`). It never landed on `main`, but several already-committed
decisions on `main` forward-reference it by name as "the sandboxed WASM plugin mechanism"
(`ADR-SYS-SYSMLV2-001`'s own Context section, `HierarchicalProductLineADR.md`,
`REQ-TRS-SYSMLV2-000/002/006`) — its public contract was real and referenced even though its code
never shipped.

That contract is directly reusable: a package `_index.md` declares `foreignFormat: <alias>`, handing
its subtree to a named entry in `.syscribe.toml`'s `[plugins.<alias>]` table; the plugin's output is
a JSON envelope of elements merged into the graph the same way the FMEA/TARA row-explosion passes in
`walker.rs` already synthesize sibling `RawElement`s from table rows — real, origin-agnostic
elements resolvable by every cross-reference kind, proven low-risk by that existing precedent and
reused unchanged by `ADR-SYS-SYSMLV2-001`'s own native SysMLv2 ingestion. This ADR delivers that
contract, superseding and finalizing `ADR-SYS-PLUGIN-001` with a different transport.

## Decision

A package `_index.md` may declare `foreignFormat: <alias>`, handing its entire subtree to an
external **process** named by `[plugins.<alias>]` in `.syscribe.toml`. Syscribe spawns it, writes
one JSON request object to its stdin (naming the package's directory and qualified name), closes
stdin, and reads one JSON envelope object from its stdout. The envelope is merged into the graph
exactly as `ADR-SYS-PLUGIN-001` specified — real `RawElement`s, qname-prefixed under the owning
package, sharing the package's `_index.md` as their `file_path`.

Four sub-decisions, each with a rejected alternative:

1. **stdio JSON transport, not WASM.** *Rejected:* WASM/Extism (`ADR-SYS-PLUGIN-001`'s actual
   choice) — real sandboxing, but a compile-to-WASM toolchain requirement on every plugin author
   (in whatever source language) plus a large `wasmtime`/`cranelift` compile-time dependency on
   Syscribe itself. A stdio subprocess needs nothing beyond "read JSON on stdin, write JSON on
   stdout" — genuinely any language, no build step, no new Cargo dependency at all (`std::process::
   Command` is already in the standard library) — matching what "an arbitrary third-party or
   user-authored parser" actually calls for better than a WASM toolchain requirement does.
2. **No sandbox — an explicit, accepted trust trade-off.** A stdio subprocess has full OS access
   (filesystem, network, everything the Syscribe process itself can do) — sharply different from
   `ADR-SYS-PLUGIN-001`'s WASM host-function boundary (`fs_read`/`fs_list_dir`/`fs_exists`, no
   network, no raw filesystem). This is intentional: the operator opts in explicitly by naming a
   `command` in `.syscribe.toml`, at the same trust level this codebase already accepts for
   `[plantuml] jar`/`plantuml` on `PATH` (`crates/syscribe/src/plantuml.rs`) and the `[remote]`
   `sh -c` download hook (`crates/syscribe-model/src/remote.rs`) — configuring a command to run has
   always meant trusting that command. *Rejected:* retrofitting OS-level sandboxing (seccomp/
   Landlock/pledge) onto a subprocess — large, platform-specific effort for a property the WASM
   design already delivers better, for the cases that genuinely need it; if a project needs that
   harder guarantee, `ADR-SYS-PLUGIN-001`'s design remains available to resurrect rather than
   bolted onto this one.
3. **No `memory_max_bytes`.** *Rejected:* enforcing a memory ceiling via `setrlimit`/cgroups/Windows
   Job Objects — real, but platform-specific extra work with no single clean cross-platform std API,
   out of scope for this pass. A wall-clock `timeout_ms` kill is the only enforced safety net; a
   runaway plugin still gets killed, just not memory-bounded.
4. **The hook lives inside `walker::walk_model` itself**, the same call site
   `ADR-SYS-PLUGIN-001`/`ADR-SYS-SYSMLV2-001` both used — not bolted onto each of its many callers
   across the CLI/MCP/LSP/web server. *Rejected:* per-call-site wiring, which risks a future caller
   silently dropping foreign elements on just that surface (the exact risk `ADR-SYS-PLUGIN-001`'s
   sub-decision 4 already named). One call site means CLI, server startup, and server live-reload
   all pick up plugin-sourced elements automatically with no extra wiring.

## Rationale

- **Why does simplicity win here even though it drops the sandbox?** The target audience shifted
  from "a plugin author who will tolerate a WASM toolchain for a real security boundary" to
  "genuinely arbitrary third-party/user-authored parser in any language" — for that audience, the
  WASM toolchain requirement was the real adoption blocker, not the missing sandbox. A team with an
  existing Python/Node/Go/shell parser for their own DSL can point `[plugins.<alias>].command` at it
  today with zero porting.
- **Why is "operator explicitly configures a command" an acceptable trust boundary here, when it
  wasn't for `ADR-SYS-PLUGIN-001`?** It always was, for different tools in this codebase — `[plantuml]
  jar`/`remote.rs`'s `sh -c` hook are the same shape of risk, already accepted. What changed is which
  mechanism a *foreign-format package* uses; the general "configuring a command means trusting it"
  boundary is not new.
- **Why a separate error/warning code range from both the WASM-plugin family and the SysMLv2
  placeholder range?** `E530`–`E532`/`W530`–`W534` are documented (in already-committed `main` docs)
  as the WASM-plugin family's reserved range even though unimplemented; `W540`–`W542` are native
  SysMLv2 ingestion's own (temporary-placeholder) range. Reusing either would misattribute a
  stdio-plugin finding to the wrong mechanism to anyone grepping a validation report. `E550`/`E551`/
  `W550`–`W553` (confirmed unused on `main`) are this mechanism's own dedicated range.

## Consequences

- A model with no `[plugins]` table configured, and no package declaring `foreignFormat:`, is
  completely unaffected.
- Plugin execution failure (command not found, spawn error, non-zero exit, malformed JSON, killed
  on timeout) degrades to "zero elements from that package plus one warning finding," never
  aborting the rest of `validate` — the same graceful-degradation posture multi-repo composition's
  `RefState::Unknown` and native SysMLv2 ingestion's `W541` already established.
- `E108` (duplicate qualified name, any origin) ships alongside this feature: `Resolver::by_qname`
  previously kept only the last-inserted element on a qname collision with no diagnostic — a latent,
  origin-agnostic gap (any two elements, not just plugin-originated ones, could collide silently).
  The diagnostic is new; `Resolver`'s resolution behavior on a collision (last-inserted wins) is
  unchanged, a deliberately separate, smaller-blast-radius concern from this ADR's actual subject.
- **Phasing.** This ADR covers Phase 1: core mechanism, `plugins run <alias> --dry-run`, the
  validation codes above, one worked example (`examples/stdio-plugins/toy-python/`), and this
  document. **Deferred, not built here:** content-hash caching of plugin output (so
  `syscribe-server`'s reload-on-any-change doesn't re-spawn a plugin process on every unrelated file
  edit — would mirror `syscribe summarize`'s `.syscribe/cache/summaries.json` convention);
  write-protection for plugin-owned elements in the web UI/MCP mutate routes; a `plugins list` CLI
  verb (mirrors `repos list`).
- **Housekeeping, non-blocking.** A handful of `main`-branch docs/tests describe the general
  foreign-format-plugin mechanism using literal "WASM"/"sandboxed WASM" wording
  (`crates/syscribe-model/src/sysmlv2/mod.rs`, `crates/syscribe-model/tests/
  sysmlv2_graceful_degradation.rs`, `model/Decisions/HierarchicalProductLineADR.md`,
  `model/Requirements/HierarchicalProductLines/REQ-TRS-HPLE-001.md`, `model/Requirements/
  SysmlV2Submodel/REQ-TRS-SYSMLV2-000.md`, `REQ-TRS-SYSMLV2-002.md`, `REQ-TRS-SYSMLV2-006.md`,
  `docs/model-guide/sysmlv2-submodel.md`) — worth a light reword to say "subprocess"/point at this
  ADR where they describe the *general* mechanism, left as a follow-on rather than a blocker since
  none of them are incorrect about the history they actually cite (`ADR-SYS-PLUGIN-001` was a real,
  designed, forward-cited decision — just never shipped).

## Addendum: `verifies:` target legality — widening `E104` to plugin-synthesized elements

Phase 1 shipped generic cross-references (`satisfies:`, `derivedFrom:`, `allocatedTo:`, `typedBy:`,
`supertype:`) working bidirectionally between a plugin-synthesized element and the native model with
zero extra work — they were already origin-agnostic, resolved by plain qname/id lookup. `verifies:`
was the one exception: `E104` (`Resolver::is_verify_target`) hard-gates its target's legality to a
native `Requirement`, or an element of a fixed requirement/architecture-shaped kind list that was
*actually synthesized* by native SysMLv2 ingestion — checked via a side-channel provenance set
(`sysmlv2_qnames`, `REQ-TRS-SYSMLV2-004`), since `RawElement` itself carries no origin marker. A
plugin-synthesized element of one of those same kinds had no equivalent widening: a native
`TestCase.verifies:` pointing at a plugin-emitted `PartDef` was rejected with `E104`, discovered by
testing the actual "link from the foreign model to the Syscribe model" scenario end to end.

The fix mirrors `REQ-TRS-SYSMLV2-004` exactly: `crate::plugins::synthesized_qnames` derives its own
side-channel provenance set (the same "fresh from `elements`, no second return value threaded
through `walk_model`'s many callers" tradeoff, but by a different mechanism — SysMLv2-synthesized
elements are identifiable by `file_path` alone, since it's a real `.sysml`/`.kerml` source path;
plugin-synthesized elements share their `file_path` with their owning package's `_index.md` anchor,
so the set is instead derived from qname nesting: after `apply_foreign_plugins` runs, every element
whose qname is nested under a `foreignFormat:`-declaring package's own qname, other than that
package itself, is guaranteed plugin-synthesized — the whole subtree's native content was already
stripped). `Resolver::is_verify_target` now takes both provenance sets and widens for either,
independently — deliberately not merged into one set, so each origin's widening stays its own
reasoned decision, matching the ADR's own stated rationale for why SysMLv2 didn't reuse the plugin
abstraction in the first place. The fixed kind list itself is untouched and shared verbatim: this
is about *which qnames* qualify, not loosening *which kinds* do. A hand-authored element of a
matching kind sitting outside any `foreignFormat:` package is unaffected — still rejected by `E104`,
confirmed by a regression test alongside the new widening.
