import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";
import { resolveBinaryPath } from "./binary";

interface McpConfig {
  servers?: Record<string, { command: string; args?: string[] }>;
}

function getMcpConfigPath(): string | null {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (!workspaceRoot) {
    return null;
  }
  return path.join(workspaceRoot, ".github", "copilot", "mcp.json");
}

function getCursorMcpPath(): string {
  return path.join(process.env.HOME ?? "", ".cursor", "mcp.json");
}

export function isMcpConfigured(): boolean {
  const paths = [getMcpConfigPath(), getCursorMcpPath()].filter(Boolean);

  for (const configPath of paths) {
    if (!configPath || !fs.existsSync(configPath)) {
      continue;
    }
    try {
      const content = fs.readFileSync(configPath, "utf-8");
      const config: McpConfig = JSON.parse(content);
      if (config.servers?.["better-ctx"]) {
        return true;
      }
    } catch {
      continue;
    }
  }
  return false;
}

export async function configureMcp(): Promise<void> {
  const binary = resolveBinaryPath();
  if (!binary) {
    vscode.window.showErrorMessage(
      "better-ctx binary not found. Install: cargo install better-ctx"
    );
    return;
  }

  const configPath = getMcpConfigPath();
  if (!configPath) {
    vscode.window.showErrorMessage("No workspace folder open.");
    return;
  }

  const dir = path.dirname(configPath);
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }

  let config: McpConfig = { servers: {} };
  if (fs.existsSync(configPath)) {
    try {
      config = JSON.parse(fs.readFileSync(configPath, "utf-8"));
    } catch {
      config = { servers: {} };
    }
  }

  if (!config.servers) {
    config.servers = {};
  }

  config.servers["better-ctx"] = {
    command: binary,
  };

  fs.writeFileSync(configPath, JSON.stringify(config, null, 2) + "\n");

  vscode.window.showInformationMessage(
    `better-ctx MCP configured in ${path.relative(
      vscode.workspace.workspaceFolders![0].uri.fsPath,
      configPath
    )}`
  );
}

export async function offerMcpSetup(): Promise<void> {
  if (isMcpConfigured()) {
    return;
  }

  const binary = resolveBinaryPath();
  if (!binary) {
    return;
  }

  const action = await vscode.window.showInformationMessage(
    "better-ctx detected but MCP not configured for Copilot. Configure now?",
    "Configure",
    "Later"
  );

  if (action === "Configure") {
    await configureMcp();
  }
}
