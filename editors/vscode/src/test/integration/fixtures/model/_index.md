---
type: Namespace
name: IntegrationTestFixture
---

Minimal model used only by the extension's `test:integration` smoke test
(`src/test/integration/`) to give `syscribe lsp` a real, valid model root to
auto-discover via this directory's `.syscribe.toml` — so activation exercises
a genuine successful LSP handshake, not just "didn't throw".
