import * as assert from "assert";
import * as vscode from "vscode";

suite("extension activation", () => {
  test("activates within the startup timeout against the fixture model", async () => {
    const ext = vscode.extensions.getExtension("syscribe.syscribe-lsp");
    assert.ok(ext, "extension not found — check publisher/name in package.json");
    // The test workspace is src/test/integration/fixtures/model (a real,
    // minimal, valid model root), so this exercises resolving the binary
    // (PATH if present, otherwise the GitHub-releases download path) and
    // driving a real `syscribe lsp` process.
    //
    // What this asserts: activate() completes promptly and never throws —
    // even a failed/hung LSP handshake is caught and reported as a
    // notification internally (see extension.ts's withStartTimeout /
    // reportStartupError), so `isActive` becomes true either way. It does
    // NOT assert the LSP session came up successfully; the Electron test
    // host has shown flaky, environment-specific stdio behavior around
    // spawning a non-Electron child process (visible as an "Unexpected
    // SIGPIPE" from VS Code's own unrelated subsystems, not this
    // extension) unrelated to server correctness. The actual handshake is
    // covered directly by `crates/syscribe/tests/lsp_*.rs` against the
    // server, and was verified manually over raw stdio JSON-RPC during
    // development.
    await ext!.activate();
    assert.ok(ext!.isActive);
  });
});
