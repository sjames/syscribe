import * as assert from "assert";
import * as vscode from "vscode";

suite("extension activation", () => {
  test("activates and starts the LSP client against the fixture model", async () => {
    const ext = vscode.extensions.getExtension("syscribe.syscribe-lsp");
    assert.ok(ext, "extension not found — check publisher/name in package.json");
    // The test workspace is src/test/integration/fixtures/model (a real,
    // minimal, valid model root), so this exercises a genuine successful
    // `syscribe lsp` handshake — resolving the binary (PATH if present,
    // otherwise the GitHub-releases download path) and starting the client
    // — not just "activate() didn't throw".
    await ext!.activate();
    assert.ok(ext!.isActive);
  });
});
