---
type: Requirement
id: REQ-TRS-PLUGIN-003
name: "Plugin execution is sandboxed: no network, no WASI filesystem preopens, scoped host-function file access"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLUGIN-000]
breakdownAdr: Decisions::WasmPluginsADR
tags:
  - plugins
  - security
---

A plugin shall run with no network access (`allowed_hosts` always empty) and no WASI filesystem
preopens. It shall read its declared foreign-format subtree only through three host-provided
functions — `fs_read`, `fs_list_dir`, `fs_exists` — each of which canonicalizes the requested path
and rejects anything that resolves outside that subtree before touching disk. Execution is bounded
by a configurable wall-clock `timeout_ms` and a fixed instruction-fuel ceiling.

## Rationale

`extism-js`'s QuickJS-ng runtime exposes no `fs`/`net`/syscall surface to JS/TS at all, so literal
WASI preopens are not achievable for a plugin authored in TypeScript on this toolchain — confirmed
against the toolchain's own documentation. Custom host functions give the identical sandboxing
property (read-only, escape-proof, supports lazy/conditional multi-file imports) via RPC instead
of syscalls, without changing the plugin-authoring language or toolchain.

## Scope

- `--wasi` is still enabled at the manifest level (required by the runtime's own clock), but no
  paths are preopened — the guest has no filesystem access except through the three scoped host
  functions.
- Path-escape fuzz-testing (symlinks, `..`, absolute paths, separator tricks) and deterministic
  fuel-based (rather than wall-clock-only) limiting are tracked as follow-on hardening, not
  required by this first cut.
