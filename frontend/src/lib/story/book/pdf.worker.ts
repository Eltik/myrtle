/**
 * THE PDF OFF THE MAIN THREAD. react-pdf lays out and writes the whole
 * document in one synchronous stretch (seconds for a chapter), so it runs
 * here: the worker loads the images and fonts itself (fetch, createImageBitmap
 * and OffscreenCanvas all exist in a worker), renders, and posts the Blob
 * back. The main thread only builds the IR and posts it.
 */
import { browserPdfDeps } from "./browser";
import { type PaperSize, toPdf } from "./pdf";
import type { BookLabels } from "./render";
import type { Book, Typeface } from "./types";

export type PdfWorkerRequest = { type: "start"; book: Book; typeface: Typeface; paper: PaperSize; labels: Omit<BookLabels, "ifChose" | "earlier"> & { ifChose: string; earlier: string } } | { type: "cancel" };
export type PdfWorkerReply = { type: "progress"; done: number; total: number; label: string } | { type: "done"; blob: Blob; images: number; failedImages: string[]; renderMs: number } | { type: "error"; message: string; aborted: boolean };

const controller = new AbortController();

self.onmessage = async (event: MessageEvent<PdfWorkerRequest>) => {
    const msg = event.data;
    if (msg.type === "cancel") {
        controller.abort();
        return;
    }
    const post = (reply: PdfWorkerReply) => (self as unknown as Worker).postMessage(reply);
    // Functions do not cross postMessage: the two templated labels arrive as
    // strings with a `{options}` hole.
    const fill = (template: string) => (options: string[]) => template.replace("{options}", options.map((o) => `“${o}”`).join(" / "));
    const labels: BookLabels = { ...msg.labels, ifChose: fill(msg.labels.ifChose), earlier: fill(msg.labels.earlier) };
    const t = performance.now();
    try {
        const out = await toPdf(msg.book, { typeface: msg.typeface, paper: msg.paper, labels, signal: controller.signal, onProgress: (p) => post({ type: "progress", ...p }) }, browserPdfDeps);
        post({ type: "done", blob: out.blob, images: out.images, failedImages: out.failedImages, renderMs: Math.round(performance.now() - t) });
    } catch (err) {
        post({ type: "error", message: err instanceof Error ? err.message : String(err), aborted: controller.signal.aborted });
    }
};
