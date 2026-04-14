# better-ctx Chrome Extension

**Token compression for web-based AI chat tools.** Automatically compresses code when pasting into ChatGPT, Claude.ai, Gemini, or GitHub Copilot Chat.

## Features

- **Auto-compress pastes** — Code pasted into AI chat inputs is automatically compressed
- **Token counter badge** — Shows savings in real-time after each compression
- **Native messaging** — Connects to local better-ctx binary for full compression (90+ patterns)
- **Fallback compression** — Basic comment/whitespace removal when native host unavailable
- **Popup dashboard** — Token savings stats + toggle controls

## Supported Sites

- [ChatGPT](https://chatgpt.com) / [chat.openai.com](https://chat.openai.com)
- [Claude.ai](https://claude.ai)
- [Gemini](https://gemini.google.com)
- [GitHub Copilot Chat](https://github.com/copilot)
- [Lovable](https://lovable.dev)
- [Bolt.new](https://bolt.new)
- [v0.dev](https://v0.dev)
- [Poe](https://poe.com)
- [Google AI Studio](https://aistudio.google.com)
- [Perplexity Labs](https://labs.perplexity.ai)

## Installation

### 1. Load Extension (Developer Mode)

1. Open `chrome://extensions`
2. Enable "Developer mode"
3. Click "Load unpacked" and select this directory
4. Note the Extension ID

### 2. Install Native Messaging Host (optional, for full compression)

```bash
cd native-host
chmod +x install.sh bridge.sh
./install.sh
```

Then edit the native messaging manifest to include your Extension ID.

### 3. Requirements

- [better-ctx](https://betterctx.com) binary for native messaging (`cargo install better-ctx`)
- Python 3 for the native messaging bridge

## How It Works

1. When you paste text (>200 chars) into a supported AI chat input
2. The extension intercepts the paste event
3. Text is sent to the background service worker
4. If native messaging is available, better-ctx compresses it
5. Otherwise, fallback compression removes comments and whitespace
6. Compressed text replaces the paste, badge shows savings

## Settings

Toggle via the popup (click extension icon):

- **Auto-compress pastes** — Enable/disable automatic compression
- **Native messaging** — Use better-ctx binary for advanced compression
