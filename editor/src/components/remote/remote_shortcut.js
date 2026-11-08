// Sends "toggle" to Rust when R is pressed outside text fields.
// Remembers the last focused element outside modals, so closing the remote can restore it.
const isTyping = (el) =>
  !!el && (["INPUT", "TEXTAREA", "SELECT"].includes(el.tagName) || el.isContentEditable);

if (window.__remoteKeydown) document.removeEventListener("keydown", window.__remoteKeydown);
if (window.__remoteFocusin) document.removeEventListener("focusin", window.__remoteFocusin);

window.__remoteKeydown = (e) => {
  if (e.key !== "r" && e.key !== "R") return;
  if (e.metaKey || e.ctrlKey || e.altKey || e.repeat || isTyping(document.activeElement)) return;
  e.preventDefault();
  dioxus.send("toggle");
};
window.__remoteFocusin = (e) => {
  if (!e.target.closest || !e.target.closest(".scrim")) window.__remoteReturnFocus = e.target;
};

document.addEventListener("keydown", window.__remoteKeydown);
document.addEventListener("focusin", window.__remoteFocusin);
