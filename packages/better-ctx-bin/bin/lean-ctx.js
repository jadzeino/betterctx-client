#!/usr/bin/env node
"use strict";

const { spawnSync } = require("child_process");
const path = require("path");

const IS_WIN = process.platform === "win32";
const BINARY = path.join(__dirname, IS_WIN ? "better-ctx.exe" : "better-ctx");

const result = spawnSync(BINARY, process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
});

if (result.error) {
  if (result.error.code === "ENOENT") {
    console.error("better-ctx binary not found. Run: npm rebuild better-ctx-bin");
  } else {
    console.error(`better-ctx: ${result.error.message}`);
  }
  process.exit(127);
}

process.exit(result.status ?? 1);
