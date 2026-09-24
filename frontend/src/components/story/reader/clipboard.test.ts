/**
 * The copy path has TWO implementations and the second one is the point: an
 * insecure origin has no `navigator.clipboard` at all, so a reader on a plain
 * http LAN address gets the textarea and `execCommand`. These pin which one
 * runs and, more importantly, that a copy which did not happen reports false
 * rather than flashing "Copied".
 */
import { afterEach, describe, expect, it, vi } from "vitest";
import { copyPlainText } from "./clipboard";

function withClipboard(writeText: ((t: string) => Promise<void>) | null): void {
    Object.defineProperty(globalThis.navigator, "clipboard", { value: writeText ? { writeText } : undefined, configurable: true });
}

afterEach(() => {
    withClipboard(null);
    vi.restoreAllMocks();
});

describe("copyPlainText", () => {
    it("uses the async clipboard when there is one, with the text unchanged", async () => {
        const writeText = vi.fn(async () => undefined);
        withClipboard(writeText);
        expect(await copyPlainText("Amiya: Doctor.")).toBe(true);
        expect(writeText).toHaveBeenCalledWith("Amiya: Doctor.");
    });

    it("falls back to execCommand when the clipboard is missing, which is the insecure-context case", async () => {
        withClipboard(null);
        const exec = vi.fn(() => true);
        Object.defineProperty(document, "execCommand", { value: exec, configurable: true });
        expect(await copyPlainText("a line")).toBe(true);
        expect(exec).toHaveBeenCalledWith("copy");
        // The textarea is a scratch element and must not be left in the page.
        expect(document.querySelectorAll("textarea")).toHaveLength(0);
    });

    it("falls back when the clipboard REJECTS, not only when it is absent", async () => {
        withClipboard(async () => {
            throw new Error("denied");
        });
        const exec = vi.fn(() => true);
        Object.defineProperty(document, "execCommand", { value: exec, configurable: true });
        expect(await copyPlainText("a line")).toBe(true);
        expect(exec).toHaveBeenCalledWith("copy");
    });

    it("reports false when both paths fail, so the button stays silent", async () => {
        withClipboard(null);
        Object.defineProperty(document, "execCommand", { value: () => false, configurable: true });
        expect(await copyPlainText("a line")).toBe(false);
    });

    it("never claims to have copied an empty log", async () => {
        const writeText = vi.fn(async () => undefined);
        withClipboard(writeText);
        expect(await copyPlainText("")).toBe(false);
        expect(writeText).not.toHaveBeenCalled();
    });
});
