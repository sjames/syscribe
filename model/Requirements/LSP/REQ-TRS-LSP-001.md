---
type: Requirement
id: REQ-TRS-LSP-001
name: "lsp subcommand starts a Language Server over stdio bound to the model root"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPServerADR
tags:
  - lsp
---

`syscribe -m <root> lsp` shall start a Language Server that communicates over stdio
(stdin/stdout), bound to the resolved model root, and shall complete the LSP `initialize`
handshake with a client, advertising exactly the server capabilities it implements in the
current release.

## Transport and lifecycle

- The server reads JSON-RPC messages from stdin and writes responses to stdin's paired stdout;
  diagnostic logging goes to stderr only, so it never corrupts the protocol stream.
- The subcommand resolves the model root the same way every other model-bound command does
  (`--model`/`-m` flag, `$SYSCRIBE_MODEL`, walk-up to `.syscribe.toml`, default `model`).
- The process is long-lived: it serves requests until the client sends `shutdown` followed by
  `exit`, then terminates with code `0`.

## Protocol scope

- The server shall expose functionality **only** through standard LSP requests and
  notifications. It shall not define or require any custom (non-LSP) JSON-RPC methods, so any
  LSP-capable client can use it without a bespoke extension.
- `initialize` shall advertise only the capabilities actually implemented in the running
  version (see `REQ-TRS-LSP-002`..`REQ-TRS-LSP-006` for the v1 set); capabilities not yet
  implemented shall not be advertised.
