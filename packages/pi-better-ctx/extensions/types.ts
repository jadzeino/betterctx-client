export type CompressionStats = {
  originalTokens: number;
  compressedTokens: number;
  percentSaved: number;
};

export type McpBridgeStatus = {
  mode: "embedded" | "adapter" | "disabled";
  connected: boolean;
  toolCount: number;
  toolNames: string[];
  error?: string;
};
