import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";
import { resolveServerCommand } from "./serverBinary";

let client: LanguageClient | undefined;

/**
 * Show a startup-failure notification without blocking on it. `activate()`
 * must not `await` this: `showErrorMessage` only resolves once a human
 * clicks a button or dismisses it, and in a headless/automated host (or a
 * real user who just ignores the toast) nothing ever does — awaiting it
 * would hang activation indefinitely instead of just failing cleanly.
 */
function reportStartupError(log: vscode.OutputChannel, message: string, settingKey: string): void {
  log.appendLine(message);
  void vscode.window.showErrorMessage(message, "Open Settings", "Show Output").then((choice) => {
    if (choice === "Open Settings") {
      void vscode.commands.executeCommand("workbench.action.openSettings", settingKey);
    } else if (choice === "Show Output") {
      log.show();
    }
  });
}

const START_TIMEOUT_MS = 15000;

/**
 * `client.start()` awaits the LSP `initialize` handshake, which should be
 * near-instant for a local stdio process. It's not supposed to be able to
 * hang, but internal error-recovery paths in the client library can, in
 * practice, leave its own promise unsettled instead of rejecting (observed:
 * a close-handler-triggered `stop()` throwing "client not running" as a
 * *separate* unhandled rejection instead of propagating into `start()`'s
 * promise). Activation must never hang indefinitely on that, so bound it here.
 */
function withStartTimeout<T>(promise: Promise<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(
      () => reject(new Error(`timed out after ${START_TIMEOUT_MS}ms waiting for the LSP handshake`)),
      START_TIMEOUT_MS,
    );
    promise.then(
      (v) => {
        clearTimeout(timer);
        resolve(v);
      },
      (e) => {
        clearTimeout(timer);
        reject(e);
      },
    );
  });
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const log = vscode.window.createOutputChannel("Syscribe");
  context.subscriptions.push(log);

  const config = vscode.workspace.getConfiguration("syscribe");
  const modelRoot = config.get<string>("modelRoot", "");

  let serverCommand: string;
  try {
    serverCommand = await resolveServerCommand(context, log);
  } catch (err) {
    reportStartupError(
      log,
      `Syscribe: could not start the language server. ${String(err)}`,
      "syscribe.serverPath",
    );
    return;
  }

  const args = ["lsp"];
  if (modelRoot.trim().length > 0) {
    args.push("-m", modelRoot);
  }

  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  const serverOptions: ServerOptions = {
    command: serverCommand,
    args,
    transport: TransportKind.stdio,
    options: { cwd: workspaceFolder?.uri.fsPath },
  };

  // Broad markdown selector: `syscribe lsp` harmlessly returns empty
  // results/diagnostics for markdown files outside the loaded model root
  // (no model root match, no crash), so this stays safe in a workspace that
  // mixes model files with ordinary docs (README.md, etc.). Scope this to a
  // narrower glob later if that proves noisy in practice.
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "markdown" }],
    synchronize: {
      // Forwarded to the server as workspace/didChangeWatchedFiles — the
      // trigger for its full-model reload (REQ-TRS-LSP-007).
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.md"),
    },
    outputChannel: log,
  };

  client = new LanguageClient(
    "syscribe",
    "Syscribe Language Server",
    serverOptions,
    clientOptions,
  );

  try {
    await withStartTimeout(client.start());
  } catch (err) {
    // Most commonly: the resolved binary ran but exited immediately (e.g. no
    // discoverable model root — `syscribe lsp` requires one) before completing
    // the LSP handshake. Fail the same clean way as an unresolvable binary
    // rather than letting VS Code surface a raw activation-rejection stack.
    const failedClient = client;
    client = undefined;
    // Best-effort cleanup only on the timeout path — start() may still be
    // retrying internally; swallow any error since we've already given up on it.
    void failedClient.stop().then(undefined, () => undefined);
    reportStartupError(
      log,
      `Syscribe: the language server process did not start correctly (command: "${serverCommand}"). ${String(err)}`,
      "syscribe.modelRoot",
    );
    return;
  }
  context.subscriptions.push({ dispose: () => void client?.stop() });
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
