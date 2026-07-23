# Syscribe VSCode extension (dev)

A thin, pure-LSP client: it spawns `syscribe lsp` over stdio and does nothing else
(`ADR-SYS-LSP-001`). Not published — for local testing while the LSP server is under
active development.

## Run it

1. `npm install`
2. Open this folder (`editors/vscode/`) in VSCode.
3. Press **F5** (or Run → Start Debugging). This compiles the extension and opens an
   **Extension Development Host** window with it loaded.
4. In that new window, open a folder containing a Syscribe model (e.g. the repo root,
   or `model/` itself) and open any `.md` file under it.
5. You should get diagnostics on open/save, hover, go-to-definition, find-references,
   workspace symbol search (`Ctrl+T`/`Cmd+T`), completion inside cross-reference
   fields, rename (`F2`) on a stable id, and codeLens/codeAction (lightbulb) on
   `E310`/`W090` findings.

## Settings

- `syscribe.serverPath` — path to the `syscribe` binary. Defaults to `syscribe` (must
  be on `PATH`); point it at `target/debug/syscribe` or `target/release/syscribe` to
  use a locally built binary without installing it.
- `syscribe.modelRoot` — passed as `-m <path>`. Leave empty to let the server
  auto-discover it (walks up to the nearest `.syscribe.toml`), which is normally what
  you want when the workspace folder is (or is inside) the model's repo.

## Requirements

`syscribe` must be built with the `lsp` subcommand (`cargo build -p syscribe` from the
repo root — it's there as of `ADR-SYS-LSP-001`).
