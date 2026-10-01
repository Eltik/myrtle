/**
 * ONE EXPORT, START TO FINISH: fetch the scope's scripts through the same
 * react-query entries the reader uses (four at a time), build the book, render
 * the chosen format, and either stream it into a file the reader picked or
 * hand back a Blob. Cancellable at every await through `signal`.
 *
 * STREAMING. Where `showSaveFilePicker` exists (Chrome, Edge) the sheet asks
 * for the file FIRST, inside the click, and an EPUB is zipped straight into
 * the file's writable as each story completes, so a 100 MB arc never sits in
 * memory whole. A cancelled or failed stream is ABORTED (the File System
 * Access spec discards everything written through that writable) and the
 * handle is then removed where the browser supports `remove()`, so no empty
 * or partial file is left behind; `onPartial` says which happened. The PDF
 * cannot stream (react-pdf writes a Blob in the browser), so it is written to
 * the picked file in one piece at the end.
 */
import type { QueryClient } from "@tanstack/react-query";
import { asset } from "#/components/operators/detail/impl/assets";
import { storyQueryOptions } from "#/lib/api/story";
import { type BookIndex, bookFileName, bookOf, scopeStories, storyLabel } from "#/lib/story/book/book";
import { browserEpubDeps, browserPdfDeps } from "#/lib/story/book/browser";
import { toEpub } from "#/lib/story/book/epub";
import type { BookFormat } from "#/lib/story/book/estimate";
import type { PaperSize } from "#/lib/story/book/pdf";
import type { PdfWorkerReply, PdfWorkerRequest } from "#/lib/story/book/pdf.worker";
import { PdfStoryError } from "#/lib/story/book/pdfError";
import { toMarkdown, toText } from "#/lib/story/book/plain";
import type { BookLabels } from "#/lib/story/book/render";
import type { Book, BookOptions, BookScope, Typeface } from "#/lib/story/book/types";
import type { StoryScript } from "#/types/generated/StoryScript";
import { createWatchdog, ExportStallError, STALL_MS } from "./watchdog";

/** Scripts fetched at once. The median script is 61 KB; four keeps a chapter moving without a burst of 40 requests. */
export const SCRIPT_CONCURRENCY = 4;
/** Above this a Blob-path export warns: the whole file sits in memory before it is saved. */
export const BLOB_WARN_BYTES = 150_000_000;

export interface ExportProgress {
    phase: "scripts" | "book" | "layout" | "merge";
    done: number;
    total: number;
    label: string;
}

/** What is left on disk after a streamed export stopped early. */
export type PartialFile = "removed" | "kept";

export interface ExportInput {
    queryClient: QueryClient;
    server: string;
    index: BookIndex;
    scope: BookScope;
    options: BookOptions;
    format: BookFormat;
    typeface: Typeface;
    paper: PaperSize;
    labels: BookLabels;
    /** The two templated labels as strings with an `{options}` hole, for the PDF worker. */
    labelTemplates: { ifChose: string; earlier: string };
    /** A file the reader picked; null downloads a Blob instead. */
    target?: FileSystemFileHandle | null;
    /** `?pdfworker=0` renders the PDF on the main thread. */
    pdfWorker?: boolean;
    /** `?pdffail=<story id>`: that story's layout throws, to prove the error line. */
    pdfFailStory?: string;
    /** Silence after which a PDF export fails; default `STALL_MS`. */
    stallMs?: number;
    signal: AbortSignal;
    onProgress: (p: ExportProgress) => void;
    onPartial?: (state: PartialFile) => void;
}

export interface ExportOutput {
    /** The file, or null when it was written to `target`. */
    blob: Blob | null;
    fileName: string;
    failedImages: string[];
    /** Where the PDF was laid out and how long that took, for the register. */
    pdf?: { thread: "worker" | "main"; renderMs: number };
}

function abortIfNeeded(signal: AbortSignal): void {
    if (signal.aborted) throw new DOMException("Export cancelled", "AbortError");
}

async function fetchScripts(input: ExportInput): Promise<Map<string, StoryScript>> {
    const { queryClient, server, scope, signal, onProgress } = input;
    const stories = scopeStories(input.index, scope);
    const scripts = new Map<string, StoryScript>();
    let next = 0;
    let done = 0;
    onProgress({ phase: "scripts", done: 0, total: stories.length, label: stories[0] ? storyLabel(stories[0].entry) : "" });
    const worker = async () => {
        while (next < stories.length) {
            const { entry } = stories[next];
            next += 1;
            abortIfNeeded(signal);
            const script = await queryClient.fetchQuery(storyQueryOptions(entry.id, server));
            abortIfNeeded(signal);
            if (script) scripts.set(entry.id, script);
            done += 1;
            onProgress({ phase: "scripts", done, total: stories.length, label: storyLabel(entry) });
        }
    };
    await Promise.all(Array.from({ length: Math.min(SCRIPT_CONCURRENCY, stories.length) }, worker));
    abortIfNeeded(signal);
    return scripts;
}

/** The worker could not be started here (no module workers, a CSP): the main thread takes over. */
class WorkerStartError extends Error {}

/**
 * Lay the PDF out in a worker. Only a failure to START falls back to the main
 * thread; a story that fails, a stall and a cancel all propagate, because
 * laying the same book out again on the main thread would freeze the page to
 * reach the same error.
 */
function pdfInWorker(book: Book, input: ExportInput): Promise<{ blob: Blob; failedImages: string[]; renderMs: number }> {
    return new Promise((resolve, reject) => {
        let worker: Worker;
        try {
            worker = new Worker(new URL("../../../lib/story/book/pdf.worker.ts", import.meta.url), { type: "module" });
        } catch (err) {
            reject(new WorkerStartError(err instanceof Error ? err.message : String(err)));
            return;
        }
        let started = false;
        let last = "";
        const stop = () => worker.postMessage({ type: "cancel" } satisfies PdfWorkerRequest);
        input.signal.addEventListener("abort", stop, { once: true });
        const dog = createWatchdog(input.stallMs ?? STALL_MS, () => {
            finish();
            reject(new ExportStallError(last, Math.round((input.stallMs ?? STALL_MS) / 1000)));
        });
        const finish = () => {
            dog.stop();
            input.signal.removeEventListener("abort", stop);
            worker.terminate();
        };
        worker.onerror = (e) => {
            finish();
            reject(started ? new Error(e.message || "PDF worker failed") : new WorkerStartError(e.message || "PDF worker did not start"));
        };
        worker.onmessageerror = () => {
            finish();
            reject(new Error("PDF worker sent a message that could not be read"));
        };
        worker.onmessage = (e: MessageEvent<PdfWorkerReply>) => {
            const msg = e.data;
            dog.kick();
            if (msg.type === "ready") started = true;
            else if (msg.type === "progress") {
                if (msg.label) last = msg.label;
                input.onProgress(pdfProgress(msg));
            } else if (msg.type === "done") {
                finish();
                resolve({ blob: msg.blob, failedImages: msg.failedImages, renderMs: msg.renderMs });
            } else {
                finish();
                reject(msg.aborted ? new DOMException("Export cancelled", "AbortError") : msg.story ? new PdfStoryError(msg.story, new Error(msg.message)) : new Error(msg.message));
            }
        };
        const { ifChose: _i, earlier: _e, ...plainLabels } = input.labels;
        worker.postMessage({ type: "start", book, typeface: input.typeface, paper: input.paper, labels: { ...plainLabels, ...input.labelTemplates }, failStoryId: input.pdfFailStory } satisfies PdfWorkerRequest);
    });
}

/** The PDF's own phases onto the sheet's: pictures loading is "book", laying out a story is "layout", writing the file is "merge". */
function pdfProgress(p: { phase: "images" | "layout" | "merge"; done: number; total: number; label: string }): ExportProgress {
    return { phase: p.phase === "images" ? "book" : p.phase, done: p.done, total: p.total, label: p.label };
}

export async function runExport(input: ExportInput): Promise<ExportOutput> {
    const { server, scope, signal, onProgress, target } = input;
    let writable: FileSystemWritableFileStream | null = null;
    try {
        if (target) writable = await target.createWritable();
        const scripts = await fetchScripts(input);
        const book = bookOf({ index: input.index, scripts, server }, scope, input.options);

        if (input.format === "epub") {
            const stream = writable;
            const result = await toEpub(book, { typeface: input.typeface, labels: input.labels, signal, onProgress: (p) => onProgress({ phase: "book", ...p }), sink: stream ? { write: (chunk) => stream.write(chunk as Uint8Array<ArrayBuffer>) } : undefined }, browserEpubDeps);
            if (stream) await stream.close();
            return { blob: result.blob, fileName: bookFileName(book, "epub"), failedImages: result.failedImages };
        }

        let blob: Blob;
        let failedImages: string[] = [];
        let pdfInfo: ExportOutput["pdf"];
        let ext: string;
        if (input.format === "pdf") {
            ext = "pdf";
            let onMain = input.pdfWorker === false;
            if (!onMain) {
                try {
                    const out = await pdfInWorker(book, input);
                    blob = out.blob;
                    failedImages = out.failedImages;
                    pdfInfo = { thread: "worker", renderMs: out.renderMs };
                } catch (err) {
                    if (!(err instanceof WorkerStartError)) throw err;
                    // The worker could not run here; the same code runs on the main thread.
                    console.warn("PDF worker did not start, laying out on the main thread", err);
                    onMain = true;
                }
            }
            if (onMain) {
                const t = performance.now();
                // Loaded on demand: react-pdf is ~1 MB of script, and a sheet that never makes a PDF on the main thread should not pay for it.
                const { toPdf } = await import("#/lib/story/book/pdf");
                // The same stall rule as the worker's; the layout yields between stories, so the timer can fire.
                let last = "";
                let stalled: ExportStallError | null = null;
                const stallCtl = new AbortController();
                const relay = () => stallCtl.abort();
                signal.addEventListener("abort", relay, { once: true });
                const dog = createWatchdog(input.stallMs ?? STALL_MS, () => {
                    stalled = new ExportStallError(last, Math.round((input.stallMs ?? STALL_MS) / 1000));
                    stallCtl.abort();
                });
                let out: Awaited<ReturnType<typeof toPdf>>;
                try {
                    out = await toPdf(
                        book,
                        {
                            typeface: input.typeface,
                            paper: input.paper,
                            labels: input.labels,
                            signal: stallCtl.signal,
                            failStoryId: input.pdfFailStory,
                            onProgress: (p) => {
                                dog.kick();
                                if (p.label) last = p.label;
                                onProgress(pdfProgress(p));
                            },
                        },
                        browserPdfDeps,
                    );
                } catch (err) {
                    if (stalled) throw stalled;
                    throw err;
                } finally {
                    dog.stop();
                    signal.removeEventListener("abort", relay);
                }
                blob = out.blob;
                failedImages = out.failedImages;
                pdfInfo = { thread: "main", renderMs: Math.round(performance.now() - t) };
            }
        } else {
            const imageUrl = (path: string) => asset(path);
            const text = input.format === "markdown" ? toMarkdown(book, { labels: input.labels, imageUrl }) : toText(book, { labels: input.labels, imageUrl });
            blob = new Blob([text], { type: input.format === "markdown" ? "text/markdown;charset=utf-8" : "text/plain;charset=utf-8" });
            ext = input.format === "markdown" ? "md" : "txt";
        }
        // biome-ignore lint/style/noNonNullAssertion: every branch above assigns it or throws.
        const whole = blob!;
        if (writable) {
            await writable.write(whole);
            await writable.close();
            return { blob: null, fileName: bookFileName(book, ext), failedImages, pdf: pdfInfo };
        }
        return { blob: whole, fileName: bookFileName(book, ext), failedImages, pdf: pdfInfo };
    } catch (err) {
        if (writable && target) {
            // Abort discards what this writable wrote; then the picked file itself goes, where the browser can remove it.
            await writable.abort().catch(() => undefined);
            const removable = target as FileSystemFileHandle & { remove?: () => Promise<void> };
            let state: PartialFile = "kept";
            if (removable.remove) {
                try {
                    await removable.remove();
                    state = "removed";
                } catch {
                    state = "kept";
                }
            }
            input.onPartial?.(state);
        }
        throw err;
    }
}

/** Save a Blob under a name, through a temporary object URL. */
export function saveBlob(blob: Blob, fileName: string): void {
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = fileName;
    a.rel = "noopener";
    document.body.appendChild(a);
    a.click();
    a.remove();
    // Revoked late: Safari reads the URL after the click returns.
    window.setTimeout(() => URL.revokeObjectURL(url), 60_000);
}

const PICKER_TYPES: Record<BookFormat, { description: string; accept: Record<string, string[]> }> = {
    epub: { description: "EPUB", accept: { "application/epub+zip": [".epub"] } },
    pdf: { description: "PDF", accept: { "application/pdf": [".pdf"] } },
    markdown: { description: "Markdown", accept: { "text/markdown": [".md"] } },
    text: { description: "Text", accept: { "text/plain": [".txt"] } },
};

type SavePicker = (opts: { suggestedName: string; types: { description: string; accept: Record<string, string[]> }[] }) => Promise<FileSystemFileHandle>;

/** The File System Access save picker, when this browser has one. */
export function savePicker(): SavePicker | null {
    if (typeof window === "undefined") return null;
    const picker = (window as unknown as { showSaveFilePicker?: SavePicker }).showSaveFilePicker;
    return typeof picker === "function" ? picker.bind(window) : null;
}

/** Ask for the file. Null when the reader dismissed the picker. */
export async function pickTarget(fileName: string, format: BookFormat): Promise<FileSystemFileHandle | null> {
    const picker = savePicker();
    if (!picker) return null;
    try {
        return await picker({ suggestedName: fileName, types: [PICKER_TYPES[format]] });
    } catch (err) {
        if (err instanceof DOMException && err.name === "AbortError") return null;
        throw err;
    }
}
