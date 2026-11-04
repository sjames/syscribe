# Front-end CI — testing a client against the real `syscribe`

A front end (an editor extension, a web panel, a CI bot) talks to `syscribe` over its CLI `--json` forms, the language
server (`syscribe lsp`) or the MCP server (`syscribe mcp`). This guide is what such a client's CI needs: the binary,
a fixture model, and the contract to assert (GH #430). It prescribes no language; everything below is a command to run
and a document to check.

## 1. The binary

Use a release binary, or build the one under test (`cargo build -p syscribe`). Everything needs only the executable.

## 2. Gate on the handshake first

```bash
syscribe capabilities --json
```

returns the tool `version`, the commands with their checked options, the published JSON contracts (`jsonSchemas`,
each with its `version`, command and `transport`), which optional capabilities exist (`features`, all booleans), the MCP
tools, the custom LSP requests and notifications. The same document is `capabilities.experimental.syscribe` in the LSP
`initialize` result.

A client decides what to show from `features[...] === true`, never from comparing version numbers: a feature that does
not exist is absent, so asking about a newer one is safe against an older binary. **Version-skew test:** run the client
against the oldest binary it claims to support and assert that each feature it needs (`ple-fixture/EXPECTED.json`
lists the ones this repository's front end needs under `frontendRequires`) is either present or that the client hides
the view that depends on it.

## 3. The fixture

`examples/ple-fixture/` is a small product line built for this: a mandatory alternative group and an or group, cross-tree
`requires`/`excludes`, typed parameters (range, enumeration, unit, default, required, binding window), parameter
constraints, three binding stages in a `derivedFrom:` chain with a narrowing plus an unstaged product, requirements
gated by features, tests on two rigs and a planning tree. It validates with **no errors**; `EXPECTED.json` states exactly
what it contains and the answers of a few documented calls, so a client can assert exact data:

| call | answer |
|---|---|
| `propagate --select Features::Camera=true` | forced on `Platform`, `CortexM`, `Link`, `Propulsion`; forced off `Cellular`, `RiscV` |
| `explain-feature Features::Link::Cellular --select Features::Camera=true` | `forcedOff`, because `Camera` excludes it |
| `bind CONF-PLF-DEPLOY-001 Features::Propulsion.motorKV 5000 --dry-run` | refused with `E205` |

Copy the fixture before a test that writes (the LSP and MCP write paths edit files).

## 4. The contracts

Every `--json` form a client depends on has a JSON Schema and a `schemaVersion`:

```bash
syscribe schema                 # the list
syscribe schema residual        # one schema (JSON Schema 2020-12)
```

Run each command against the fixture and validate its output with any JSON Schema validator; check that
`schemaVersion` equals the advertised `jsonSchemas[name].version`. Schemas allow extra properties, so a client ignores
fields it does not know. A breaking change bumps the version and is listed in
[the JSON contract changelog](../releases/json-contract.md).

| transport | what to run |
|---|---|
| CLI | the command of each `jsonSchemas` entry with `transport: "cli"` |
| LSP | `initialize` (opt in to `syscribe/modelChanged` with `capabilities.experimental.syscribe.modelChanged`), then each name in `lspRequests`: `syscribe/propagate`, `explainFeature`, `children`, `changeImpact`, `timeline`, `health`, `preview`; `workspace/executeCommand` `syscribe.apply`; `textDocument/semanticTokens/full` |
| MCP | `tools/list` offers every `mcpTools` entry (write tools are hidden under `--read-only`); call the read tools |

## 5. This repository's reference suite

`crates/syscribe/tests/frontend_contract.rs` does all of the above in Rust, against the real binary and a copy of the
fixture: the fixture matches `EXPECTED.json`; every published CLI contract validates on it; every advertised LSP
request answers with a schema-valid payload and a saved change produces `syscribe/modelChanged`; the MCP server offers
every advertised read tool and hides the write tools when read-only; the required capabilities are present. A contract
or request added without a case in that suite fails it, so the fixture keeps covering everything published. The shared
harness it uses is `crates/syscribe/tests/common/mod.rs` (an `Lsp` and an `Mcp` client over stdio).

```bash
cargo test -p syscribe --test frontend_contract
```
