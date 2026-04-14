document.addEventListener("DOMContentLoaded", () => {
  const tokensSaved = document.getElementById("tokens-saved");
  const commands = document.getElementById("commands");
  const toggleEnabled = document.getElementById("toggle-enabled");
  const togglePaste = document.getElementById("toggle-paste");

  chrome.runtime.sendMessage({ action: "getStats" }, (stats) => {
    if (stats) {
      tokensSaved.textContent = formatNumber(stats.totalSaved || 0);
      commands.textContent = String(stats.totalCommands || 0);
    }
  });

  chrome.runtime.sendMessage({ action: "getSettings" }, (settings) => {
    if (settings) {
      toggleEnabled.checked = settings.enabled !== false;
      togglePaste.checked = settings.autoCompressPaste !== false;
    }
  });

  toggleEnabled.addEventListener("change", () => {
    updateSetting("enabled", toggleEnabled.checked);
  });

  togglePaste.addEventListener("change", () => {
    updateSetting("autoCompressPaste", togglePaste.checked);
  });

  checkNativeStatus();
});

function updateSetting(key, value) {
  chrome.storage.local.get(["settings"], (result) => {
    const settings = result.settings || {};
    settings[key] = value;
    chrome.storage.local.set({ settings });
  });
}

function formatNumber(n) {
  if (n >= 1000000) return (n / 1000000).toFixed(1) + "M";
  if (n >= 1000) return (n / 1000).toFixed(1) + "k";
  return String(n);
}

function checkNativeStatus() {
  const footer = document.querySelector(".footer");

  chrome.runtime.sendMessage({ action: "pingNative" }, (response) => {
    const lastErr = chrome.runtime.lastError;
    if (lastErr) {
      showSetupHint(footer, "Message error: " + lastErr.message);
      return;
    }
    if (response && response.nativeOk) {
      footer.innerHTML = `
        <span style="color:#00d4aa">Native messaging active</span><br>
        <span style="font-size:10px;color:#666">${response.binary || ""}</span><br>
        <a href="https://betterctx.com" target="_blank">betterctx.com</a> ·
        <a href="https://github.com/jadzeino/betterctx-client" target="_blank">GitHub</a>
      `;
    } else {
      const detail = response ? (response.error || "not connected") : "no response";
      showSetupHint(footer, detail);
    }
  });
}

function showSetupHint(footer, errorDetail) {
  const extId = chrome.runtime.id;

  footer.innerHTML = `
    <div style="text-align:left;font-size:11px;margin-bottom:4px;color:#ff9800">
      Native messaging not connected
    </div>
    <div style="text-align:left;font-size:9px;color:#666;margin-bottom:8px;font-family:monospace">
      ${errorDetail || ""}
    </div>
    <div style="text-align:left;font-size:10px;color:#888;margin-bottom:6px">
      1. Clone better-ctx &amp; run this in Terminal:
    </div>
    <div style="background:#1a1a2e;border-radius:6px;padding:8px;font-family:monospace;font-size:9px;
                word-break:break-all;cursor:pointer;border:1px solid #333;position:relative"
         id="copy-cmd" title="Click to copy">
      cd better-ctx/packages/chrome-better-ctx/native-host &amp;&amp; chmod +x install.sh bridge.sh &amp;&amp; ./install.sh ${extId}
    </div>
    <div id="copy-feedback" style="font-size:10px;color:#00d4aa;margin-top:4px;display:none">
      Copied!
    </div>
    <div style="text-align:left;font-size:10px;color:#888;margin-top:8px">
      2. Quit Chrome completely (Cmd+Q) and reopen
    </div>
    <div style="margin-top:8px">
      <a href="https://betterctx.com" target="_blank">betterctx.com</a> ·
      <a href="https://github.com/jadzeino/betterctx-client" target="_blank">GitHub</a>
    </div>
  `;

  const copyCmd = document.getElementById("copy-cmd");
  if (copyCmd) {
    copyCmd.addEventListener("click", () => {
      const rawCmd = `cd better-ctx/packages/chrome-better-ctx/native-host && chmod +x install.sh bridge.sh && ./install.sh ${extId}`;
      navigator.clipboard.writeText(rawCmd).then(() => {
        const fb = document.getElementById("copy-feedback");
        fb.style.display = "block";
        setTimeout(() => (fb.style.display = "none"), 2000);
      });
    });
  }
}
