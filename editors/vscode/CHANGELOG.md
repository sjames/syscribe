# Changelog

All notable changes to the Syscribe VS Code extension are documented here.

## 0.2.0

- Auto-resolve the `syscribe` server binary: explicit `syscribe.serverPath` > `syscribe` on
  `PATH` > a managed copy downloaded from the [sjames/syscribe] GitHub releases and cached
  per-version in the extension's global storage. New `syscribe.version` setting pins a
  release (`"latest"` by default).
- `syscribe.serverPath` now defaults to empty (auto-resolve) instead of assuming `syscribe`
  is already on `PATH`.
- Marketplace packaging: icon, license, repository metadata, `.vscodeignore`.
- Bundled with esbuild (`dist/extension.js`) instead of shipping raw `tsc` output plus
  `node_modules` — cuts the packaged `.vsix` from 334 files / ~493 KB down to 8 files / ~94 KB.

[sjames/syscribe]: https://github.com/sjames/syscribe

## 0.1.0

- Initial pure-LSP client: spawns `syscribe lsp` over stdio
  (`ADR-SYS-LSP-001`). Diagnostics, go-to-definition, find-references, hover,
  workspace symbol search, completion, rename, codeLens/codeAction.
