import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { Type } from "@sinclair/typebox";
import type { McpBridgeStatus } from "./types.js";

const CLI_OVERRIDE_TOOLS = new Set([
  "ctx_read",
  "ctx_multi_read",
  "ctx_shell",
  "ctx_search",
  "ctx_tree",
]);

const MAX_RECONNECT_ATTEMPTS = 3;
const RECONNECT_DELAY_MS = 2000;

type McpTool = {
  name: string;
  description?: string;
  inputSchema?: Record<string, unknown>;
};

export class McpBridge {
  private client: Client | null = null;
  private transport: StdioClientTransport | null = null;
  private registeredTools: string[] = [];
  private connected = false;
  private binary: string;
  private reconnectAttempts = 0;

  constructor(binary: string) {
    this.binary = binary;
  }

  async start(pi: ExtensionAPI): Promise<void> {
    try {
      await this.connect();
      await this.discoverAndRegisterTools(pi);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      console.error(`[better-ctx MCP bridge] Failed to start: ${msg}`);
    }
  }

  private async connect(): Promise<void> {
    this.transport = new StdioClientTransport({
      command: this.binary,
      args: [],
      stderr: "pipe",
    });

    this.client = new Client({
      name: "pi-better-ctx",
      version: "2.0.0",
    });

    this.transport.onclose = () => {
      this.connected = false;
      this.scheduleReconnect();
    };

    this.transport.onerror = (err) => {
      console.error(`[better-ctx MCP bridge] Transport error: ${err.message}`);
    };

    await this.client.connect(this.transport);
    this.connected = true;
    this.reconnectAttempts = 0;
  }

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
      console.error(
        `[better-ctx MCP bridge] Max reconnect attempts (${MAX_RECONNECT_ATTEMPTS}) reached. MCP tools unavailable.`,
      );
      return;
    }

    this.reconnectAttempts++;
    const delay = RECONNECT_DELAY_MS * this.reconnectAttempts;

    setTimeout(async () => {
      try {
        await this.connect();
        console.error("[better-ctx MCP bridge] Reconnected successfully");
      } catch {
        this.scheduleReconnect();
      }
    }, delay);
  }

  private async discoverAndRegisterTools(pi: ExtensionAPI): Promise<void> {
    if (!this.client) return;

    const result = await this.client.listTools();
    const tools = (result.tools ?? []) as McpTool[];

    for (const tool of tools) {
      if (CLI_OVERRIDE_TOOLS.has(tool.name)) continue;
      this.registerMcpTool(pi, tool);
    }
  }

  private registerMcpTool(pi: ExtensionAPI, tool: McpTool): void {
    const bridge = this;
    const schema = this.jsonSchemaToTypebox(tool.inputSchema);

    pi.registerTool({
      name: tool.name,
      label: tool.name,
      description: tool.description ?? `better-ctx MCP tool: ${tool.name}`,
      promptSnippet: tool.description ?? tool.name,
      parameters: schema,
      async execute(_toolCallId, params, _signal) {
        return bridge.callTool(tool.name, params as Record<string, unknown>);
      },
    });

    this.registeredTools.push(tool.name);
  }

  async callTool(
    name: string,
    args: Record<string, unknown>,
  ): Promise<{ content: Array<{ type: string; text: string }> }> {
    if (!this.client || !this.connected) {
      throw new Error(
        `better-ctx MCP bridge not connected. Tool "${name}" unavailable.`,
      );
    }

    const result = await this.client.callTool({ name, arguments: args });

    const content = (
      result.content as Array<{ type: string; text?: string }>
    ).map((block) => ({
      type: "text" as const,
      text: block.text ?? "",
    }));

    return { content };
  }

  private jsonSchemaToTypebox(
    schema?: Record<string, unknown>,
  ): ReturnType<typeof Type.Object> {
    if (!schema || !schema.properties) {
      return Type.Object({});
    }

    const properties = schema.properties as Record<
      string,
      Record<string, unknown>
    >;
    const required = new Set(
      (schema.required as string[] | undefined) ?? [],
    );
    const fields: Record<string, ReturnType<typeof Type.String>> = {};

    for (const [key, prop] of Object.entries(properties)) {
      const desc = (prop.description as string) ?? undefined;
      const jsonType = prop.type as string | undefined;

      let field;
      switch (jsonType) {
        case "number":
        case "integer":
          field = Type.Number({ description: desc });
          break;
        case "boolean":
          field = Type.Boolean({ description: desc });
          break;
        case "array":
          field = Type.Array(Type.Unknown(), { description: desc });
          break;
        case "object":
          field = Type.Record(Type.String(), Type.Unknown(), {
            description: desc,
          });
          break;
        default:
          field = Type.String({ description: desc });
          break;
      }

      fields[key] = required.has(key)
        ? field
        : Type.Optional(field);
    }

    return Type.Object(fields);
  }

  getStatus(): McpBridgeStatus {
    return {
      mode: "embedded",
      connected: this.connected,
      toolCount: this.registeredTools.length,
      toolNames: [...this.registeredTools],
    };
  }

  async shutdown(): Promise<void> {
    this.reconnectAttempts = MAX_RECONNECT_ATTEMPTS;
    try {
      await this.client?.close();
    } catch {
      // best-effort cleanup
    }
    this.client = null;
    this.transport = null;
    this.connected = false;
  }
}
