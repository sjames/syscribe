import * as path from "path";
import { runTests } from "@vscode/test-electron";

async function main(): Promise<void> {
  try {
    const extensionDevelopmentPath = path.resolve(__dirname, "../../../");
    const extensionTestsPath = path.resolve(__dirname, "./suite/index");
    // Opened as the workspace folder so `syscribe lsp` (with no explicit
    // `-m`) auto-discovers this fixture's `.syscribe.toml` and gets a real,
    // valid model root — the smoke test exercises an actual successful LSP
    // handshake, not just "activation didn't throw". Full LSP capability
    // coverage (diagnostics, navigation, completion, rename, ...) lives in
    // `crates/syscribe/tests/lsp_*.rs` against the server directly, not here.
    const fixtureWorkspace = path.resolve(__dirname, "../fixtures/model");
    await runTests({
      extensionDevelopmentPath,
      extensionTestsPath,
      launchArgs: [fixtureWorkspace, "--disable-extensions"],
    });
  } catch (err) {
    console.error("Failed to run integration tests:", err);
    process.exit(1);
  }
}

void main();
