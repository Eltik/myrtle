/**
 * Copying text out of the reader.
 *
 * `navigator.clipboard` is the whole answer on a secure origin, and it is
 * MISSING rather than failing everywhere else: the async clipboard is gated on
 * a secure context, so a reader on a plain-http LAN address has no
 * `navigator.clipboard` at all and an `await` on it throws a TypeError before
 * any permission is involved. The fallback is the old one, a hidden textarea
 * and `document.execCommand("copy")`, which is deprecated and still the only
 * thing that works there.
 *
 * Returns whether the text actually reached the clipboard, so the button can
 * stay silent instead of flashing "Copied" over a copy that did not happen.
 */
export async function copyPlainText(text: string): Promise<boolean> {
    if (text === "") return false;
    try {
        if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
            await navigator.clipboard.writeText(text);
            return true;
        }
    } catch {
        // Blocked, denied or insecure: fall through to the textarea.
    }
    return execCommandCopy(text);
}

/**
 * The insecure-context path. The textarea has to be IN the document and
 * focusable for the selection to exist, so it is placed off-screen rather than
 * hidden: `display: none` and `visibility: hidden` both make the selection
 * empty and the copy a silent no-op.
 */
function execCommandCopy(text: string): boolean {
    if (typeof document === "undefined") return false;
    const area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.setAttribute("aria-hidden", "true");
    area.style.position = "fixed";
    area.style.top = "0";
    area.style.left = "-9999px";
    document.body.appendChild(area);
    try {
        area.select();
        area.setSelectionRange(0, text.length);
        return document.execCommand("copy");
    } catch {
        return false;
    } finally {
        area.remove();
    }
}
