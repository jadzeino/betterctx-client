const MIN_LENGTH = 200;
let isCompressing = false;
let extensionSettings = { enabled: true, autoCompressPaste: true };

const SITE_CONFIG = {
  "chatgpt.com": {
    input: '#prompt-textarea, div[contenteditable="true"]#prompt-textarea',
    send: 'button[data-testid="send-button"], button[data-testid="composer-send-button"], form button[type="button"]:last-child',
  },
  "chat.openai.com": {
    input: '#prompt-textarea, div[contenteditable="true"]#prompt-textarea',
    send: 'button[data-testid="send-button"]',
  },
  "claude.ai": {
    input: 'div.ProseMirror[contenteditable="true"]',
    send: 'button[aria-label="Send Message"], button[aria-label="Send message"], fieldset button:last-child',
  },
  "gemini.google.com": {
    input: '.ql-editor[contenteditable="true"], rich-textarea .ql-editor',
    send: 'button[aria-label="Send message"], button.send-button',
  },
  "github.com": {
    input: 'textarea[name="message"], textarea.js-copilot-chat-input',
    send: 'button[type="submit"]',
  },
  "lovable.dev": {
    input: 'textarea',
    send: 'button[type="submit"]',
  },
  "bolt.new": {
    input: 'textarea',
    send: 'button[type="submit"]',
  },
  "v0.dev": {
    input: 'textarea',
    send: 'button[type="submit"]',
  },
  "poe.com": {
    input: 'textarea',
    send: 'button[class*="send"], button[class*="Send"]',
  },
  "aistudio.google.com": {
    input: 'textarea',
    send: 'button[aria-label*="Send"], button[aria-label*="Run"]',
  },
  "labs.perplexity.ai": {
    input: 'textarea',
    send: 'button[aria-label*="Submit"], button[aria-label*="Send"]',
  },
};

function getSiteConfig() {
  const host = window.location.hostname;
  for (const [domain, config] of Object.entries(SITE_CONFIG)) {
    if (host.includes(domain)) return config;
  }
  return null;
}

function getActiveInput(config) {
  const els = document.querySelectorAll(config.input);
  for (const el of els) {
    if (el.offsetParent !== null) return el;
  }
  return els[0] || null;
}

function getInputText(el) {
  if (el.tagName === "TEXTAREA" || el.tagName === "INPUT") return el.value;
  return el.innerText || el.textContent || "";
}

function setTextareaValue(el, text) {
  const proto = Object.getOwnPropertyDescriptor(
    window.HTMLTextAreaElement.prototype,
    "value"
  );
  if (proto && proto.set) {
    proto.set.call(el, text);
  } else {
    el.value = text;
  }
  el.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertText" }));
  el.dispatchEvent(new Event("change", { bubbles: true }));
}

function setContentEditableText(el, text) {
  el.focus();
  const sel = window.getSelection();
  const range = document.createRange();
  range.selectNodeContents(el);
  sel.removeAllRanges();
  sel.addRange(range);
  document.execCommand("insertText", false, text);
}

function setInputText(el, text) {
  if (el.tagName === "TEXTAREA" || el.tagName === "INPUT") {
    setTextareaValue(el, text);
  } else {
    setContentEditableText(el, text);
  }
}

async function compressText(text) {
  return new Promise((resolve) => {
    chrome.runtime.sendMessage({ action: "compress", text }, (response) => {
      if (chrome.runtime.lastError) {
        resolve(null);
        return;
      }
      resolve(response);
    });
  });
}

function findSendButton(config) {
  if (config.send) {
    const btns = document.querySelectorAll(config.send);
    for (const btn of btns) {
      if (btn.offsetParent !== null) return btn;
    }
  }
  const fallbacks = [
    'button[type="submit"]',
    'button[aria-label*="Send"]',
    'button[aria-label*="send"]',
  ];
  for (const sel of fallbacks) {
    const btns = document.querySelectorAll(sel);
    for (const btn of btns) {
      if (btn.offsetParent !== null) return btn;
    }
  }
  return null;
}

function triggerSend(input, config) {
  const form = input.closest("form");
  if (form) {
    try {
      form.requestSubmit();
      return;
    } catch {
      try {
        form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
        return;
      } catch { /* fall through */ }
    }
  }

  const btn = findSendButton(config);
  if (btn) {
    btn.click();
    return;
  }

  const opts = {
    key: "Enter",
    code: "Enter",
    keyCode: 13,
    which: 13,
    bubbles: true,
  };
  input.dispatchEvent(new KeyboardEvent("keydown", opts));
  input.dispatchEvent(new KeyboardEvent("keypress", opts));
  input.dispatchEvent(new KeyboardEvent("keyup", opts));
}

async function handleSubmit(input, config) {
  const text = getInputText(input).trim();

  if (text.length < MIN_LENGTH) {
    triggerSend(input, config);
    return;
  }

  isCompressing = true;
  showCompressing();

  const response = await compressText(text);

  if (response && !response.skipped && response.compressed && response.compressed !== text) {
    setInputText(input, response.compressed);
    showSavings(response.inputTokens || 0, response.outputTokens || 0, response.savings || 0);
    updateStats(response);
  }

  hideCompressing();
  isCompressing = false;

  await new Promise((resolve) => {
    requestAnimationFrame(() => setTimeout(resolve, 200));
  });

  const freshInput = getActiveInput(config) || input;
  triggerSend(freshInput, config);
}

let hookedInputs = new WeakSet();

function hookInput(input, config) {
  if (hookedInputs.has(input)) return;
  hookedInputs.add(input);

  input.addEventListener(
    "keydown",
    (e) => {
      if (e.key !== "Enter" || e.shiftKey || e.isComposing || e.metaKey || e.ctrlKey) return;
      if (!extensionSettings.enabled) return;
      if (isCompressing) {
        e.preventDefault();
        e.stopImmediatePropagation();
        return;
      }
      if (e._leanCtxPassthrough) return;

      const text = getInputText(input).trim();
      if (text.length < MIN_LENGTH) return;

      e.preventDefault();
      e.stopImmediatePropagation();
      handleSubmit(input, config);
    },
    true
  );

  input.addEventListener("paste", async (e) => {
    if (!extensionSettings.enabled || !extensionSettings.autoCompressPaste) return;

    const text = e.clipboardData?.getData("text/plain");
    if (!text || text.length < MIN_LENGTH) return;

    const response = await compressText(text);
    if (!response || response.skipped || !response.compressed || response.compressed === text)
      return;

    e.preventDefault();

    if (input.tagName === "TEXTAREA" || input.tagName === "INPUT") {
      const start = input.selectionStart || 0;
      const before = input.value.substring(0, start);
      const after = input.value.substring(input.selectionEnd || start);
      setTextareaValue(input, before + response.compressed + after);
    } else {
      document.execCommand("insertText", false, response.compressed);
    }

    showSavings(response.inputTokens || 0, response.outputTokens || 0, response.savings || 0);
    updateStats(response);
  });
}

let badge = null;

function createBadge() {
  if (badge && document.body.contains(badge)) return badge;
  badge = document.createElement("div");
  badge.id = "better-ctx-badge";
  document.body.appendChild(badge);
  return badge;
}

function showCompressing() {
  const b = createBadge();
  b.textContent = "better-ctx: compressing...";
  b.classList.add("visible");
}

function hideCompressing() {
  if (badge) badge.classList.remove("visible");
}

function showSavings(inputTokens, outputTokens, savings) {
  const b = createBadge();
  b.textContent = `better-ctx: ${inputTokens}\u2192${outputTokens} tok (-${savings.toFixed(0)}%)`;
  b.classList.add("visible");
  setTimeout(() => b.classList.remove("visible"), 4000);
}

function updateStats(response) {
  chrome.storage.local.get(["stats"], (result) => {
    const stats = result.stats || { totalSaved: 0, totalCommands: 0 };
    stats.totalSaved += (response.inputTokens || 0) - (response.outputTokens || 0);
    stats.totalCommands += 1;
    chrome.storage.local.set({ stats });
  });
}

function loadSettings() {
  chrome.runtime.sendMessage({ action: "getSettings" }, (s) => {
    if (chrome.runtime.lastError) return;
    if (s) extensionSettings = { ...extensionSettings, ...s };
  });
  chrome.storage.onChanged.addListener((changes) => {
    if (changes.settings?.newValue) {
      extensionSettings = { ...extensionSettings, ...changes.settings.newValue };
    }
  });
}

function observeAndHook(config) {
  const tryHook = () => {
    const inputs = document.querySelectorAll(config.input);
    inputs.forEach((input) => hookInput(input, config));
  };

  tryHook();

  const observer = new MutationObserver(tryHook);
  observer.observe(document.body, { childList: true, subtree: true });
}

function init() {
  const config = getSiteConfig();
  if (!config) return;

  loadSettings();
  observeAndHook(config);
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
