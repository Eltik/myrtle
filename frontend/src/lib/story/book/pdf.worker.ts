/**
 * THE PDF OFF THE MAIN THREAD. The worker loads the images and fonts itself
 * (fetch, createImageBitmap and OffscreenCanvas all exist in a worker), lays
 * the book out one story at a time (`toPdf`), and posts progress per story
 * and the Blob at the end. Every failure is POSTED, with the story it
 * happened in: an uncaught throw or a rejected promise anywhere in here is
 * caught by the two global handlers below, so the sheet never waits on a
 * worker that died quietly.
 */
import { browserPdfDeps } from "./browser";
import { type PaperSize, PdfStoryError, toPdf } from "./pdf";
import type { BookLabels } from "./render";
import type { Book, Typeface } from "./types";

export type PdfWorkerRequest = { type: "start"; book: Book; typeface: Typeface; paper: PaperSize; labels: Omit<BookLabels, "ifChose" | "earlier"> & { ifChose: string; earlier: string }; failStoryId?: string } | { type: "cancel" };
export type PdfWorkerReply =
    | { type: "ready" }
    | { type: "progress"; phase: "images" | "layout" | "merge"; done: number; total: number; label: string }
    | { type: "done"; blob: Blob; images: number; failedImages: string[]; renderMs: number; pages: number }
    | { type: "error"; message: string; story?: string; aborted: boolean };

const controller = new AbortController();
const post = (reply: PdfWorkerReply) => (self as unknown as Worker).postMessage(reply);
const fail = (err: unknown) => post({ type: "error", message: err instanceof Error ? err.message : String(err), story: err instanceof PdfStoryError ? err.story : undefined, aborted: controller.signal.aborted });

self.addEventListener("error", (e) => fail(e.error ?? e.message));
self.addEventListener("unhandledrejection", (e) => fail(e.reason));

self.onmessage = async (event: MessageEvent<PdfWorkerRequest>) => {
    const msg = event.data;
    if (msg.type === "cancel") {
        controller.abort();
        return;
    }
    // Functions do not cross postMessage: the two templated labels arrive as
    // strings with an `{options}` hole.
    const fill = (template: string) => (options: string[]) => template.replace("{options}", options.map((o) => `“${o}”`).join(" / "));
    const labels: BookLabels = { ...msg.labels, ifChose: fill(msg.labels.ifChose), earlier: fill(msg.labels.earlier) };
    const t = performance.now();
    try {
        const out = await toPdf(msg.book, { typeface: msg.typeface, paper: msg.paper, labels, signal: controller.signal, failStoryId: msg.failStoryId, onProgress: (p) => post({ type: "progress", ...p }) }, browserPdfDeps);
        post({ type: "done", blob: out.blob, images: out.images, failedImages: out.failedImages, renderMs: Math.round(performance.now() - t), pages: out.pages });
    } catch (err) {
        fail(err);
    }
};

// The module loaded: from here on a failure is the book's, not the worker's.
post({ type: "ready" });
