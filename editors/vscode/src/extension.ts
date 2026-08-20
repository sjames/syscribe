import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";
import { resolveServerCommand } from "./serverBinary";

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const log = vscode.window.createOutputChannel("Syscribe");
  context.subscriptions.push(log);

  const config = vscode.workspace.getConfiguration("syscribe");
  const modelRoot = config.get<string>("modelRoot", "");

  let serverCommand: string;
  try {
    serverCommand = await resolveServerCommand(context, log);
  } catch (err) {
    const message = `Syscribe: could not start the language server. ${String(err)}`;
    log.appendLine(message);
    const choice = await vscode.window.showErrorMessage(message, "Open Settings", "Show Output");
    if (choice === "Open Settings") {
      void vscode.commands.executeCommand("workbench.action.openSettings", "syscribe.serverPath");
    } else if (choice === "Show Output") {
      log.show();
    }
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
    await client.start();
  } catch (err) {
    // Most commonly: the resolved binary ran but exited immediately (e.g. no
    // discoverable model root — `syscribe lsp` requires one) before completing
    // the LSP handshake. Fail the same clean way as an unresolvable binary
    // rather than letting VS Code surface a raw activation-rejection stack.
    const message =
      `Syscribe: the language server process did not start correctly ` +
      `(command: "${serverCommand}"). ${String(err)}`;
    log.appendLine(message);
    client = undefined;
    const choice = await vscode.window.showErrorMessage(message, "Open Settings", "Show Output");
    if (choice === "Open Settings") {
      void vscode.commands.executeCommand("workbench.action.openSettings", "syscribe.modelRoot");
    } else if (choice === "Show Output") {
      log.show();
    }
    return;
  }
  context.subscriptions.push({ dispose: () => void client?.stop() });
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
