import * as vscode from "vscode";
import { resolveBinaryPath } from "./binary";
import { createStatusBar } from "./statusbar";
import { configureMcp, offerMcpSetup } from "./mcp";
import {
  cmdSetup,
  cmdDoctor,
  cmdGain,
  cmdDashboard,
  cmdHeatmap,
  showWelcome,
  disposeOutputChannel,
} from "./commands";

export function activate(context: vscode.ExtensionContext): void {
  const binary = resolveBinaryPath();
  if (!binary) {
    vscode.window.showWarningMessage(
      "better-ctx binary not found. Install: cargo install better-ctx"
    );
    return;
  }

  context.subscriptions.push(
    vscode.commands.registerCommand("better-ctx.setup", cmdSetup),
    vscode.commands.registerCommand("better-ctx.doctor", cmdDoctor),
    vscode.commands.registerCommand("better-ctx.gain", cmdGain),
    vscode.commands.registerCommand("better-ctx.dashboard", cmdDashboard),
    vscode.commands.registerCommand("better-ctx.heatmap", cmdHeatmap),
    vscode.commands.registerCommand("better-ctx.configureMcp", configureMcp)
  );

  createStatusBar(context);

  const config = vscode.workspace.getConfiguration("better-ctx");
  if (config.get<boolean>("autoSetup", true)) {
    offerMcpSetup();
  }

  showWelcome();
}

export function deactivate(): void {
  disposeOutputChannel();
}
