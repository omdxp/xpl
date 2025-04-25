import * as path from "path";
import * as vscode from "vscode";
import { workspace, ExtensionContext } from "vscode";
import { exec } from "child_process";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
  RevealOutputChannelOn,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  // Create an output channel for the language server
  const outputChannel = vscode.window.createOutputChannel("xplLanguageServer");

  // Determine server executable path (assumes binary built in workspace target) server/xpl_ls
  const serverModule = path.join(context.extensionPath, "server", "xpl_ls");

  // Server run options
  const serverOptions: ServerOptions = {
    run: { command: serverModule, transport: TransportKind.stdio },
    debug: { command: serverModule, transport: TransportKind.stdio },
  };

  // Client options
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "xpl" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.xpl"),
    },
    outputChannel, // show server logs in our named channel
    revealOutputChannelOn: RevealOutputChannelOn.Error, // Reveal on error
  };

  // Create and start the language client
  client = new LanguageClient(
    "xplLanguageServer", // ID
    "xplLanguageServer", // Display name, matches output channel
    serverOptions,
    clientOptions
  );

  // Start the language client and register disposables
  client.start();
  context.subscriptions.push(client, outputChannel);

  // Command for running the active XPL file (invoked by CodeLens)
  const runCmd = vscode.commands.registerCommand("xpl.runFile", async () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== "xpl") {
      return;
    }
    await editor.document.save();
    const file = editor.document.uri.fsPath;
    exec(`xpl ${file}`, (error, stdout, stderr) => {
      if (error) {
        outputChannel.appendLine(stderr);
        vscode.window.showErrorMessage(`xpl run error: ${error.message}`);
      } else {
        outputChannel.appendLine(stdout);
        vscode.window.showInformationMessage("xpl execution complete");
      }
    });
  });
  context.subscriptions.push(runCmd);
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
