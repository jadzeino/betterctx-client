import * as vscode from "vscode";
import { execFile } from "child_process";
import { promisify } from "util";

const execFileAsync = promisify(execFile);

let cachedBinaryPath: string | null = null;

export function resolveBinaryPath(): string | null {
  const config = vscode.workspace.getConfiguration("better-ctx");
  const configPath = config.get<string>("binaryPath");
  if (configPath) {
    return configPath;
  }
  if (cachedBinaryPath) {
    return cachedBinaryPath;
  }

  const candidates = [
    "better-ctx",
    `${process.env.HOME}/.cargo/bin/better-ctx`,
    "/usr/local/bin/better-ctx",
    "/opt/homebrew/bin/better-ctx",
  ];

  for (const candidate of candidates) {
    try {
      require("child_process").execFileSync(candidate, ["--version"], {
        timeout: 5000,
        stdio: "pipe",
      });
      cachedBinaryPath = candidate;
      return candidate;
    } catch {
      continue;
    }
  }

  return null;
}

export async function runCommand(
  args: string[]
): Promise<{ stdout: string; stderr: string }> {
  const binary = resolveBinaryPath();
  if (!binary) {
    throw new Error(
      "better-ctx binary not found. Install: cargo install better-ctx"
    );
  }

  const workspaceRoot =
    vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? process.cwd();

  return execFileAsync(binary, args, {
    cwd: workspaceRoot,
    timeout: 30000,
    env: { ...process.env, BETTER_CTX_ACTIVE: "0", NO_COLOR: "1" },
  });
}

export async function getVersion(): Promise<string | null> {
  try {
    const { stdout } = await runCommand(["--version"]);
    const match = stdout.match(/(\d+\.\d+\.\d+)/);
    return match ? match[1] : null;
  } catch {
    return null;
  }
}
