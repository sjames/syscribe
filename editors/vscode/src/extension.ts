import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext): void {
  const config = vscode.workspace.getConfiguration("syscribe");
  const serverPath = config.get<string>("serverPath", "syscribe");
  const modelRoot = config.get<string>("modelRoot", "");

  const args = ["lsp"];
  if (modelRoot.trim().length > 0) {
    args.push("-m", modelRoot);
  }

  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  const serverOptions: ServerOptions = {
    command: serverPath,
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
  };

  client = new LanguageClient(
    "syscribe",
    "Syscribe Language Server",
    serverOptions,
    clientOptions,
  );

  client.start();
  context.subscriptions.push({ dispose: () => void client?.stop() });
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
