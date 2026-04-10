#!/usr/bin/env python3
"""better-ctx native messaging bridge for Chrome.

One-shot mode: reads a single JSON message from Chrome via stdin
(length-prefixed), processes it, returns the result, then exits.
"""
import json
import struct
import subprocess
import sys
import os
import traceback

LOG_PATH = os.path.expanduser("~/.better-ctx/bridge-debug.log")


def log(msg):
    try:
        os.makedirs(os.path.dirname(LOG_PATH), exist_ok=True)
        with open(LOG_PATH, "a") as f:
            f.write(msg + "\n")
    except Exception:
        pass


def find_better_ctx():
    for candidate in [
        os.path.expanduser("~/.cargo/bin/better-ctx"),
        "/usr/local/bin/better-ctx",
        "/opt/homebrew/bin/better-ctx",
    ]:
        if os.path.isfile(candidate) and os.access(candidate, os.X_OK):
            return candidate
    import shutil
    return shutil.which("better-ctx")


def read_message():
    raw = sys.stdin.buffer.read(4)
    log(f"header bytes: {len(raw)}")
    if len(raw) < 4:
        return None
    length = struct.unpack("I", raw)[0]
    log(f"message length: {length}")
    if length > 1024 * 1024:
        return None
    data = sys.stdin.buffer.read(length)
    log(f"body bytes: {len(data)}")
    if len(data) < length:
        return None
    return json.loads(data.decode("utf-8"))


def send_message(obj):
    encoded = json.dumps(obj, ensure_ascii=False).encode("utf-8")
    sys.stdout.buffer.write(struct.pack("I", len(encoded)))
    sys.stdout.buffer.write(encoded)
    sys.stdout.buffer.flush()
    log(f"sent: {len(encoded)} bytes")


def compress(text, binary_path):
    try:
        env = os.environ.copy()
        env.pop("BETTER_CTX_ACTIVE", None)
        env.pop("BETTER_CTX_DISABLED", None)
        env["NO_COLOR"] = "1"
        result = subprocess.run(
            [binary_path, "-c", "cat"],
            input=text,
            capture_output=True,
            text=True,
            timeout=10,
            env=env,
        )
        compressed = result.stdout.strip() if result.returncode == 0 else text
    except (subprocess.TimeoutExpired, FileNotFoundError) as e:
        log(f"compress error: {e}")
        compressed = text

    input_tokens = len(text) // 4
    output_tokens = len(compressed) // 4
    savings = ((input_tokens - output_tokens) / max(input_tokens, 1)) * 100

    return {
        "compressed": compressed,
        "inputTokens": input_tokens,
        "outputTokens": output_tokens,
        "savings": round(savings, 1),
    }


def main():
    log("--- bridge started ---")
    try:
        msg = read_message()
        if msg is None:
            log("no input received")
            send_message({"error": "no input"})
            return

        binary = find_better_ctx()
        action = msg.get("action", "")
        log(f"action: {action}, binary: {binary}")

        if action == "ping":
            send_message({"status": "ok", "binary": binary or "not found"})
        elif action == "compress":
            text = msg.get("text", "")
            if binary and text:
                send_message(compress(text, binary))
            else:
                send_message({"error": "better-ctx not found or empty text"})
        else:
            send_message({"error": f"unknown action: {action}"})
    except Exception as e:
        log(f"EXCEPTION: {traceback.format_exc()}")
        try:
            send_message({"error": str(e)})
        except Exception:
            pass


if __name__ == "__main__":
    main()
