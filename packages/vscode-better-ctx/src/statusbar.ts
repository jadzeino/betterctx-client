import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";

let statusBarItem: vscode.StatusBarItem | null = null;
let refreshTimer: NodeJS.Timeout | null = null;

interface StatsData {
  total_input_tokens: number;
  total_output_tokens: number;
  total_commands: number;
}

function getStatsPath(): string {
  return path.join(
    process.env.HOME ?? "",
    ".better-ctx",
    "stats.json"
  );
}

function readStats(): StatsData | null {
  try {
    const content = fs.readFileSync(getStatsPath(), "utf-8");
    return JSON.parse(content);
  } catch {
    return null;
  }
}

function formatTokens(tokens: number): string {
  if (tokens >= 1_000_000) {
    return `${(tokens / 1_000_000).toFixed(1)}M`;
  }
  if (tokens >= 1_000) {
    return `${(tokens / 1_000).toFixed(1)}K`;
  }
  return `${tokens}`;
}

function updateStatusBar(): void {
  if (!statusBarItem) {
    return;
  }

  const stats = readStats();
  if (!stats) {
    statusBarItem.text = "$(zap) better-ctx";
    statusBarItem.tooltip = "better-ctx: No stats yet. Run some commands!";
    return;
  }

  const saved = stats.total_input_tokens;
  const commands = stats.total_commands;
  const savingsText = formatTokens(saved);

  statusBarItem.text = `$(zap) ${savingsText} saved`;
  statusBarItem.tooltip = [
    `better-ctx Token Savings`,
    `────────────────────────`,
    `Tokens saved: ${saved.toLocaleString()}`,
    `Commands: ${commands.toLocaleString()}`,
    ``,
    `Click to open dashboard`,
  ].join("\n");
}

export function createStatusBar(context: vscode.ExtensionContext): void {
  const config = vscode.workspace.getConfiguration("better-ctx");
  if (!config.get<boolean>("statusBar", true)) {
    return;
  }

  statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    100
  );
  statusBarItem.command = "better-ctx.gain";
  context.subscriptions.push(statusBarItem);

  updateStatusBar();
  statusBarItem.show();

  const interval = config.get<number>("refreshInterval", 30) * 1000;
  refreshTimer = setInterval(updateStatusBar, interval);
  context.subscriptions.push({
    dispose: () => {
      if (refreshTimer) {
        clearInterval(refreshTimer);
      }
    },
  });

  const statsPath = getStatsPath();
  if (fs.existsSync(path.dirname(statsPath))) {
    const watcher = fs.watch(path.dirname(statsPath), (_, filename) => {
      if (filename === "stats.json") {
        updateStatusBar();
      }
    });
    context.subscriptions.push({ dispose: () => watcher.close() });
  }
}
