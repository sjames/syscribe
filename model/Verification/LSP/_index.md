---
type: Package
name: LSP
---

Integration test cases for the `syscribe lsp` subcommand — the stdio Language Server, its
diagnostics/navigation/symbol capabilities, and reload behavior. Each case is realised as a
black-box Rust integration test that spawns `syscribe lsp` against a fixture model and drives
it over the LSP stdio protocol.
