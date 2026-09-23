// Public DOM only. Never inspect WhatsApp's private modules, QR payloads or chats.
(() => {
  if (location.origin !== EXPECTED_ORIGIN) return { state: "unexpected_page" };
  const visible = e => !!e && e.isConnected && e.getClientRects().length > 0 && getComputedStyle(e).visibility !== "hidden" && getComputedStyle(e).display !== "none";
  const qr = [...document.querySelectorAll('[data-ref] canvas, [data-testid="qrcode"] canvas, canvas[aria-label*="QR"], canvas[aria-label*="qr"]')]
    .find(e => visible(e) && e.width >= 100 && e.height >= 100 && e.width <= 1024 && e.height <= 1024);
  // Login evidence takes precedence over a stale sidebar hidden by a modal.
  if (qr) return { state: "awaiting_qr", qr: qr.toDataURL ? true : false };
  if ([...document.querySelectorAll('#pane-side, [data-testid="chat-list"]')].some(visible)) return { state: "ready" };
  if ([...document.querySelectorAll('progress, [role="progressbar"]')].some(visible)) return { state: "syncing" };
  if (document.querySelector('a[href*="google.com/chrome"]') && /Chrome\s+\d+\+/.test(document.body?.innerText || "")) return { state: "browser_incompatible" };
  return { state: "loading" };
})();
