---
type: ADR
id: ADR-SYS-PLUGIN-001
name: "Foreign-format ingestion via sandboxed WASM plugins: read-only, Extism/TS, host-function filesystem access"
status: accepted
tags:
  - plugins
  - wasm
  - interop
---

## Context

Syscribe models are Markdown+YAML by design, but other teams and tools express systems models in
other text-based methodologies — plain SysMLv2 textual notation, or a project's own
domain-specific language. There was no way for a directory inside the model tree to be authored
in a different notation while still participating in Syscribe's traceability graph: a
`Requirement`'s `derivedFrom:`, an `Allocation`, a `satisfies:`/`verifies:` link could not target
an element that lived outside the native parser's understanding.

Two existing subsystems were evaluated as precedent:

1. **Multi-repository composition** (`[repos]`, §14) already brings external content into a
   model, but only as an existence check: `LoadedRepo` indexes a peer repo's qnames/stable-ids
   into two flat `HashSet<String>`s so cross-repo references don't false-flag as dangling. It
   never builds real graph nodes for peer content — there is no cross-repo traversal, and nothing
   from a peer repo shows up in `/api/elements`, the web UI, or an `Allocation` target list.
2. **The embedded Rhai scripting engine** (`crates/syscribe/src/scripting.rs`) is a real, sandboxed
   precedent — no filesystem/network access, operation-count limits, used for custom validation
   checks. But its `Model`/`Element` API is strictly read-only observation; it has no path to
   create new elements.

Neither precedent alone gets a foreign-format element to full first-class status: visible in
every read surface, targetable by every cross-reference kind, and covered by validation.

The enabling fact that made this feasible is that the core element/graph types are already
**origin-agnostic**. `RawElement` (`crates/syscribe-model/src/element.rs`) is a plain
qname+frontmatter+doc struct with no requirement that it came from parsing a real `.md` file;
`Resolver::resolve_ref` and `validate_with_config` operate over a `&[RawElement]` slice with no
special-casing by origin. The FMEA/TARA "row explosion" passes in `walker.rs` already prove this
out: they synthesize sibling `RawElement`s (borrowed `file_path`, synthetic qname) from table rows
in one sheet's frontmatter, and those synthetic elements flow through the resolver and validator
identically to hand-authored ones.

## Decision

A package `_index.md` may declare `foreignFormat: <alias>`, handing its entire subtree to a WASM
plugin named by a `[plugins.<alias>]` entry in `.syscribe.toml`. The plugin's `parse` export
returns a JSON envelope of elements, which are merged into the graph the same way FMEA/TARA
synthesize their rows — as real `RawElement`s, qname-prefixed under the owning package, sharing
the package's `_index.md` as their `file_path`.

Four sub-decisions, each with a rejected alternative:

1. **Read-only ingestion.** The plugin only parses; the foreign folder stays authoritative and is
   edited by its own native tooling, never by Syscribe's web UI or mutate commands. *Rejected:*
   bidirectional read/write (plugin also serializes edits back), which would roughly double the
   plugin contract and require every foreign format to also be a full serializer — deferred
   indefinitely unless a concrete need for diagram-editor-style editing across the boundary shows up.
2. **Extism-style TypeScript/WASM plugin runtime.** JS/TS compiled via `extism-js` (`@extism/js-pdk`,
   QuickJS-ng) to a `.wasm` module, invoked through the `extism` Rust crate (wraps `wasmtime`) with a
   single JSON-in/JSON-out `parse()` call. *Rejected:* the WASM Component Model + WIT + `jco`
   toolchain — more standards-track and strongly typed, but heavier tooling and less mature for TS
   today, for no capability this project actually needs.
3. **Scoped filesystem access via custom Extism host functions, not WASI preopens.** `js-pdk`'s
   QuickJS build exposes zero `fs`/syscall surface to JS at all ("No Node.js APIs. No fs, path,
   net..." — confirmed against the toolchain's own documentation), so literal WASI preopens are not
   achievable for a TS plugin on this toolchain regardless of `Manifest.allowed_paths`. Three host
   functions (`fs_read`, `fs_list_dir`, `fs_exists`) give the plugin the same practical capability —
   each canonicalizes the requested path and rejects anything that resolves outside the plugin's
   declared subtree before touching disk. `--wasi` is still enabled at the manifest level (the
   `extism-js` runtime requires it for its own clock), but no paths are preopened, so the guest has
   no filesystem access except through these three RPC calls.
4. **`wasm-plugins` is a Cargo feature on `syscribe-model`, on by default in `syscribe` and
   `syscribe-server`.** Plugin execution lives inside `walker::walk_model` itself — not bolted onto
   each of its ~20 call sites across the CLI, MCP, LSP, and web server — so every consumer picks up
   foreign elements automatically with no per-call-site wiring risk. *Rejected:* a separate
   `syscribe-plugins` crate explicitly invoked by each caller, which keeps `syscribe-model` free of
   the `wasmtime` dependency tree but is fragile — a 21st `walk_model` call site added later could
   easily forget to also wire in plugin execution and silently drop foreign elements on just that
   surface. The feature-gate keeps a bare `cargo check -p syscribe-model` dependency-light while
   still guaranteeing uniform behavior everywhere `walk_model` is called.

## Rationale

- **Why not extend the multi-repo existence-check pattern instead of building real graph nodes?**
  Because the whole point is for a `Requirement`/`Allocation` to be able to target a foreign
  element the same way it targets a native one — `satisfies:`, `verifies:`, `derivedFrom:` all need
  the target to actually resolve via `Resolver::resolve_ref`, not just avoid a "dangling reference"
  warning. The FMEA/TARA precedent already proved synthetic `RawElement` injection is
  low-risk and requires zero special-casing in the resolver or validator.
- **Why accept the RPC-style host functions instead of chasing real WASI for JS/TS?** The
  sandboxing property that actually matters — read-only, scoped, escape-proof, supports
  lazy/conditional multi-file imports — is identical either way. Switching toolchains to get literal
  WASI semantics would buy standards-purity, not new capability, at real toolchain-maturity cost.
- **Why feature-gate `syscribe-model` rather than accept the dependency unconditionally?** `extism`
  pulls in `wasmtime`/`cranelift`, a genuinely large addition to compile time and binary size. Gating
  it behind a feature (on by default in the two shipped binaries, off for a bare library build) is a
  low-cost way to keep that cost opt-in at the crate level while still shipping it "just works" to
  every actual user of `syscribe`/`syscribe-server`.

## Consequences

- A model with no `[plugins]` configured is completely unaffected — `apply_foreign_plugins`
  no-ops immediately (REQ-TRS-PLUGIN-000).
- Plugin execution failure (missing wasm, trap, timeout, malformed JSON) degrades to "zero elements
  from that package plus one warning finding," never aborting the rest of `validate` — the same
  graceful-degradation posture multi-repo's `RefState::Unknown` already established.
- The `RawElement.file_path`/qname-collision assumptions this design leans on surfaced one latent,
  origin-agnostic gap: `Resolver::new` silently kept only the last-inserted element on a duplicate
  qname with no diagnostic. `E108` (duplicate qualified name) closes that gap for every element,
  not just plugin-originated ones.
- **Phasing** (this ADR covers Phase 1 — core plumbing, one working example plugin, read-only
  merge, soft-fail diagnostics; see `docs/model-guide/wasm-plugins.md`). Since first landing, Phase
  2's two substantive items have both shipped: path-escape unit tests directly against the
  sandboxing boundary (`..` traversal, absolute paths, a real symlink-escape attempt, all
  confirmed rejected — REQ-TRS-PLUGIN-003), and an on-disk, content-hash-keyed cache at
  `.syscribe/cache/plugins.json` mirroring `syscribe summarize`'s existing cache convention, so an
  unchanged (wasm, subtree) pair skips re-invocation entirely — fixing both `syscribe-server`
  live-reload's full-model re-walk on any file change and the guarded-write candidate-copy walk's
  fresh-mtimes-defeat-caching problem (REQ-TRS-PLUGIN-007). Remaining: true fuzzing (randomized
  adversarial input generation, vs. the hand-written cases already covered) and write-protection
  for plugin-owned elements in the mutate routes — a plugin-owned element is not currently rejected
  by `PUT`/mutate routes beyond what "the file doesn't correspond to a real writable path" already
  implies incidentally. Treat that gap as open until a follow-on requirement closes it.
