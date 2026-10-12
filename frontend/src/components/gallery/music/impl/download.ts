import { useStore } from "@tanstack/react-store";
import { Store } from "@tanstack/store";
import { createElement, useCallback } from "react";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { recordMusicDownloadFn, reportMusicStat } from "#/lib/api/music";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "../MusicGallery.messages";
import { DownloadProgress } from "./DownloadProgress";
import { type ITrack, trackSource } from "./model";
import { clipFileName } from "./tracks";

type MusicT = TypedT<typeof messages>;

/**
 * SAVING A TRACK. A label track is the label's own WAV, about 60 MB, reached
 * through the backend's stream redirect to the label's CDN (which allows any
 * origin). An in-game track is its loop clip, a few hundred KB.
 *
 * A cross-origin `<a download>` is ignored by every browser (it would open the
 * WAV in a player tab), so the file is fetched. Where the File System Access
 * save picker exists (Chrome and Edge on desktop) the file is asked for first,
 * inside the click, and the response is piped straight into it: nothing is
 * held in memory. Everywhere else the response becomes a Blob saved through a
 * temporary object URL, which holds the whole file (about 60 MB) until the
 * browser takes it. The bytes pass through a counting stream either way, so
 * the progress is real; nothing is kept in script.
 */

export interface IDownload {
    received: number;
    /** From `Content-Length`; `null` when the response did not say. */
    total: number | null;
}

/** Track id to its download in flight. */
export const downloadStore = new Store<Readonly<Record<string, IDownload>>>({});

export function useDownload(trackId: string | null): IDownload | null {
    return useStore(downloadStore, (s) => (trackId ? (s[trackId] ?? null) : null));
}

/** Characters no common file system takes in a name. */
const UNSAFE = /[\\/:*?"<>|\p{Cc}]+/gu;

function safeName(name: string): string {
    return name.replace(UNSAFE, " ").replace(/\s+/g, " ").trim().slice(0, 180);
}

/** `<Artist> - <Title>.wav` for a label track (every credited name, as the label credits it); the clip's own name for an in-game one. */
export function downloadFileName(track: ITrack): string {
    if (track.kind === "game") return clipFileName(track.game?.loopUrl ?? `${track.key}.ogg`);
    const artists = track.artists.join(", ");
    const title = safeName(track.title) || track.key;
    return `${artists ? `${safeName(artists)} - ` : ""}${title}.wav`;
}

/** The file a download fetches: the label's stream, or the in-game loop clip the player loads. */
function downloadUrl(track: ITrack): string | null {
    const source = trackSource(track);
    if (source.kind === "stream") return source.src;
    return source.kind === "loop" ? source.loop : null;
}

export function isDownloadable(track: ITrack): boolean {
    return downloadUrl(track) !== null;
}

/**
 * Count a download. Signed in, through a server function that sends the
 * session as the bearer, so it counts against the account; signed out (or a
 * session the backend no longer takes), from the browser, counted by address.
 * One path or the other per download. Never throws.
 */
export function countDownload(trackId: string, signedIn: boolean): void {
    const anonymous = () => void reportMusicStat("download", trackId);
    if (!signedIn) {
        anonymous();
        return;
    }
    recordMusicDownloadFn({ data: trackId })
        .then((recorded) => {
            if (recorded === null) anonymous();
        })
        .catch(() => {
            // It may have counted before failing; reporting again could count twice.
        });
}

type SavePicker = (opts: { suggestedName: string; types: { description: string; accept: Record<string, string[]> }[] }) => Promise<FileSystemFileHandle>;

function savePicker(): SavePicker | null {
    if (typeof window === "undefined") return null;
    const picker = (window as unknown as { showSaveFilePicker?: SavePicker }).showSaveFilePicker;
    return typeof picker === "function" ? picker.bind(window) : null;
}

/** Hand `blob` to the browser's own download UI under `fileName`. */
function saveBlob(blob: Blob, fileName: string): void {
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = fileName;
    a.rel = "noopener";
    document.body.append(a);
    a.click();
    a.remove();
    // Revoked late: Safari reads the URL after the click returns.
    window.setTimeout(() => URL.revokeObjectURL(url), 60_000);
}

function setProgress(trackId: string, next: IDownload | null): void {
    downloadStore.setState((s) => {
        const copy = { ...s };
        if (next) copy[trackId] = next;
        else delete copy[trackId];
        return copy;
    });
}

/** `body` with every chunk counted into the download store as it passes. */
function counted(trackId: string, body: ReadableStream<Uint8Array>, total: number | null): ReadableStream<Uint8Array> {
    let received = 0;
    let shownAt = 0;
    return body.pipeThrough(
        new TransformStream<Uint8Array, Uint8Array>({
            transform(chunk, controller) {
                received += chunk.byteLength;
                // A store write per chunk would re-render the rows a thousand times a file;
                // four a second is as often as a reader can follow the toast.
                const now = performance.now();
                if (now - shownAt >= 250) {
                    shownAt = now;
                    setProgress(trackId, { received, total });
                }
                controller.enqueue(chunk);
            },
        }),
    );
}

/** Delete a picked file the download never filled. `remove()` is Chromium's, and newer than the picker. */
async function removeEntry(handle: FileSystemFileHandle): Promise<void> {
    const remove = (handle as { remove?: () => Promise<void> }).remove;
    if (typeof remove !== "function") return;
    await remove.call(handle).catch(() => {});
}

export type DownloadOutcome = "saved" | "handed" | "cancelled" | "failed";

/** Tracks whose save picker is open: a second press then must not start a second download. */
const picking = new Set<string>();

/**
 * Save `track`. MUST be called inside the click: the save picker needs the
 * user's gesture, so nothing may be awaited before it. `onStarted` runs once
 * the audio answered, which is when the download counts.
 */
export async function downloadTrack(track: ITrack, onStarted: () => void): Promise<DownloadOutcome> {
    const url = downloadUrl(track);
    if (!url) return "failed";
    // A second press while it runs is the same download.
    if (downloadStore.state[track.id] || picking.has(track.id)) return "cancelled";
    let fileName = downloadFileName(track);
    const picker = track.kind === "siren" ? savePicker() : null;
    let handle: FileSystemFileHandle | null = null;
    if (picker) {
        picking.add(track.id);
        try {
            handle = await picker({ suggestedName: fileName, types: [{ description: "WAV", accept: { "audio/wav": [".wav"] } }] });
        } catch (err) {
            if (err instanceof DOMException && err.name === "AbortError") return "cancelled";
            // A picker that refuses for another reason (a policy, a sandbox) still leaves the Blob route.
            handle = null;
        } finally {
            picking.delete(track.id);
        }
    }
    setProgress(track.id, { received: 0, total: null });
    let writable: FileSystemWritableFileStream | null = null;
    try {
        const res = await fetch(url);
        if (!res.ok || !res.body) throw new Error(String(res.status));
        const length = Number(res.headers.get("content-length"));
        const total = Number.isFinite(length) && length > 0 ? length : null;
        setProgress(track.id, { received: 0, total });
        // The label serves WAV; should a release ever come as MP3, the Blob route names it so.
        if (track.kind === "siren" && /mpeg|mp3/i.test(res.headers.get("content-type") ?? "")) fileName = fileName.replace(/\.wav$/, ".mp3");
        onStarted();
        const body = counted(track.id, res.body, total);
        if (handle) {
            writable = await handle.createWritable();
            await body.pipeTo(writable);
            writable = null;
            return "saved";
        }
        const blob = await new Response(body).blob();
        saveBlob(blob, fileName);
        return "handed";
    } catch {
        // An aborted writable discards what was written. The picker itself already made the
        // file, empty, so a failure takes the entry off too where the browser can.
        if (writable) await writable.abort().catch(() => {});
        if (handle) await removeEntry(handle);
        return "failed";
    } finally {
        setProgress(track.id, null);
    }
}

/**
 * The download action every surface shares (a row's menu, the player bar, the
 * charts): it saves the track, counts it once it starts, and says how it went
 * in a toast. The picker route ends with a "Saved" toast, since the browser
 * shows nothing; the Blob route hands the file to the browser's own download
 * UI and says nothing more.
 */
export function useTrackDownload(): (track: ITrack) => void {
    const t: MusicT = useT("gallery");
    const { user } = useAuth();
    const signedIn = user != null;
    return useCallback(
        (track: ITrack) => {
            const name = track.title;
            let toastId: string | null = null;
            void downloadTrack(track, () => {
                countDownload(track.id, signedIn);
                // Added once; the progress line reads the store itself (see `DownloadProgress`).
                toastId = toastManager.add({ type: "loading", title: t("music.download.busy", { name }), description: createElement(DownloadProgress, { trackId: track.id }), timeout: 0 });
            }).then((outcome) => {
                if (toastId === null) {
                    if (outcome === "failed") toastManager.add({ type: "error", title: t("music.download.failedTrack", { name }) });
                    return;
                }
                // The same toast turns into the result, so it does not leave and enter again.
                if (outcome === "saved") toastManager.update(toastId, { type: "success", title: t("music.download.saved", { name }), description: undefined, timeout: 5000 });
                else if (outcome === "failed") toastManager.update(toastId, { type: "error", title: t("music.download.failedTrack", { name }), description: undefined, timeout: 5000 });
                else toastManager.close(toastId);
            });
        },
        [t, signedIn],
    );
}
