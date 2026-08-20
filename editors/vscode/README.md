# Syscribe for VS Code

A thin, pure-LSP client: it spawns `syscribe lsp` over stdio and does nothing else
(`ADR-SYS-LSP-001`). Every capability — diagnostics, navigation, completion, rename — is
standard LSP served by the `syscribe` binary itself, so this extension is intentionally
small.

Open any `.md` file in a workspace containing a [Syscribe] model
(a `.syscribe.toml` and/or a `model/` directory) to get:

- **Diagnostics** — validation findings (`E***`/`W***`) on open/save.
- **Go to definition** / **find references** on a qualified name or stable id.
- **Hover** — resolved element summary (type, id, qname, status).
- **Workspace symbol search** (`Ctrl+T` / `Cmd+T`) across the whole model.
- **Completion** inside cross-reference fields (`derivedFrom:`, `verifies:`, `supertype:`, ...)
  and enum fields (`status:`, `testLevel:`, `reqDomain:`, ...).
- **Rename** (`F2`) on a stable id, rewritten safely across every referencing file.
- **CodeLens / CodeAction** (lightbulb) on findings like `E310`/`W090`.

[Syscribe]: https://github.com/sjames/syscribe

## The `syscribe` binary

This extension needs the `syscribe` CLI, built with the `lsp` subcommand, to actually run
anything. It resolves one automatically, in order:

1. **`syscribe.serverPath`**, if set — used as-is (absolute path or a name on `PATH`).
2. **`syscribe` on `PATH`**, if found.
3. Otherwise, a **managed copy** is downloaded from the [GitHub releases] of this project
   and cached in the extension's global storage, keyed by release tag and platform — so this
   only happens once per version, not on every window reload. Use `syscribe.version` to pin a
   specific release instead of always tracking `"latest"`.

[GitHub releases]: https://github.com/sjames/syscribe/releases

## Settings

| Setting | Default | Description |
|---|---|---|
| `syscribe.serverPath` | `""` (auto) | Explicit path to the `syscribe` binary. Leave empty to auto-resolve. |
| `syscribe.version` | `"latest"` | Release tag to download when auto-resolving and no `syscribe` is on `PATH`. |
| `syscribe.modelRoot` | `""` (auto) | Model root passed as `-m <path>`. Leave empty to let the server auto-discover it (`.syscribe.toml` walk-up). |

## Developing this extension

1. `npm install`
2. Open this folder (`editors/vscode/`) in VS Code.
3. Press **F5** (or Run → Start Debugging). This compiles the extension and opens an
   **Extension Development Host** window with it loaded.
4. In that new window, open a folder containing a Syscribe model (e.g. the repo root, or
   `model/` itself) and open any `.md` file under it.

Set `syscribe.serverPath` to `target/debug/syscribe` or `target/release/syscribe` to use a
locally built binary (`cargo build -p syscribe` from the repo root) instead of the
auto-resolved one while iterating on the server.

### Tests

- `npm test` — fast unit tests (platform/target-mapping logic), plain Node/Mocha, no VS Code
  host required.
- `npm run test:integration` — downloads a throwaway VS Code build and runs an
  Extension-Development-Host smoke test (extension activates cleanly). Full LSP behavior
  (diagnostics, navigation, completion, rename, ...) is covered server-side by
  `crates/syscribe/tests/lsp_*.rs`, not duplicated here.

### Packaging

`npm run build:vsix` produces a `.vsix` (via `vsce package`, which esbuild-bundles
`dist/extension.js` first — `vscode-languageclient` and its dependencies are inlined, so the
shipped package carries no `node_modules`) for local sideloading or manual Marketplace upload.
