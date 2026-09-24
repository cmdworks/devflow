/**
 * Robust clipboard utility supporting modern Async Clipboard API
 * with transparent DOM execCommand fallback for desktop WebViews and iframe sandboxes.
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  if (!text) return false;

  // 1. Primary: Modern Async Clipboard API
  try {
    if (typeof navigator !== "undefined" && navigator.clipboard && navigator.clipboard.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch (err) {
    console.warn("navigator.clipboard.writeText failed, using DOM fallback:", err);
  }

  // 2. Secondary: Hidden DOM textarea with document.execCommand
  try {
    const textarea = document.createElement("textarea");
    textarea.value = text;
    textarea.setAttribute("readonly", "");
    textarea.style.position = "fixed";
    textarea.style.top = "-9999px";
    textarea.style.left = "-9999px";
    textarea.style.opacity = "0";
    textarea.style.pointerEvents = "none";
    document.body.appendChild(textarea);
    textarea.focus();
    textarea.select();
    textarea.setSelectionRange(0, text.length);
    const successful = document.execCommand("copy");
    document.body.removeChild(textarea);
    if (successful) return true;
  } catch (err) {
    console.error("Fallback execCommand copy failed:", err);
  }

  return false;
}
