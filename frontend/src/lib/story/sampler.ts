/**
 * ONE IMAGE, SAMPLED ONCE, KEPT IN THREE PLACES.
 *
 * The story feature samples two different kinds of picture, a ticket's cover
 * art and a character's body PNG, and it used to do it with two hand-written
 * copies of the same pipeline: a memory map, an in-flight map, a localStorage
 * record, an `Image` load, a `drawImage` into an offscreen canvas, and a
 * `getImageData` into the rule that reads it. The two differed in four values
 * and in nothing else, so they are one factory now and the four values are its
 * arguments.
 *
 * THREE CACHES, and each answers a different question. `hot` is memory only and
 * is what a React initial state may read, because localStorage on the client
 * and nothing on the server is a hydration mismatch. `cached` adds the stored
 * record, for a caller that wants an answer without starting work. `sample`
 * does the work, and shares one job between every caller that asks while it
 * runs, so eight cards over one cover decode it once.
 *
 * It never REJECTS. A failure resolves to the fallback, including a tainted
 * canvas: `/api/assets` is same-site today, but a deployment that moved it to
 * another origin would otherwise throw a SecurityError on `getImageData` for
 * every card on the page at once, and the right answer there is a grey ticket.
 */

export interface ISampler<T> {
    /** Memory only. Safe in a `useState` initialiser, because the server and the client agree on it. */
    hot(url: string): T | undefined;
    /** Memory, then the stored record, promoting a hit into memory. Never starts a sample. */
    cached(url: string): T | undefined;
    /** The full path: cache, then in-flight job, then a decode. Resolves to the fallback on any failure. */
    sample(url: string): Promise<T>;
}

export interface ISamplerOptions<T> {
    /** The versioned localStorage key. A bump invalidates every cached record at once. */
    storageKey: string;
    /** The sample grid the image is drawn into. */
    width: number;
    height: number;
    /** The rule that turns sampled pixels into the value. Pure, and unit-tested on its own. */
    derive: (data: Uint8ClampedArray) => T;
    fallback: T;
}

export function createSampler<T>({ storageKey, width, height, derive, fallback }: ISamplerOptions<T>): ISampler<T> {
    const memory = new Map<string, T>();
    /** In flight, so several callers sharing a url sample it once. */
    const pending = new Map<string, Promise<T>>();

    function readStore(): Record<string, T> {
        try {
            const raw = globalThis.localStorage?.getItem(storageKey);
            if (!raw) return {};
            const parsed: unknown = JSON.parse(raw);
            return parsed && typeof parsed === "object" ? (parsed as Record<string, T>) : {};
        } catch {
            return {};
        }
    }

    function writeStore(url: string, value: T): void {
        try {
            const all = readStore();
            all[url] = value;
            globalThis.localStorage?.setItem(storageKey, JSON.stringify(all));
        } catch {
            // A private window or a full quota is not an error here: the memory cache still holds.
        }
    }

    function cached(url: string): T | undefined {
        const hot = memory.get(url);
        if (hot) return hot;
        const stored = readStore()[url];
        if (stored) {
            memory.set(url, stored);
            return stored;
        }
        return undefined;
    }

    return {
        hot(url) {
            return memory.get(url);
        },
        cached,
        sample(url) {
            const known = cached(url);
            if (known) return Promise.resolve(known);
            const inFlight = pending.get(url);
            if (inFlight) return inFlight;

            const job = (async (): Promise<T> => {
                try {
                    const image = new Image();
                    image.crossOrigin = "anonymous";
                    image.decoding = "async";
                    image.src = url;
                    await new Promise<void>((resolve, reject) => {
                        image.addEventListener("load", () => resolve(), { once: true });
                        image.addEventListener("error", () => reject(new Error("image failed")), { once: true });
                    });
                    const canvas = document.createElement("canvas");
                    canvas.width = width;
                    canvas.height = height;
                    const ctx = canvas.getContext("2d", { willReadFrequently: true });
                    if (!ctx) return fallback;
                    ctx.drawImage(image, 0, 0, width, height);
                    const value = derive(ctx.getImageData(0, 0, width, height).data);
                    memory.set(url, value);
                    writeStore(url, value);
                    return value;
                } catch {
                    memory.set(url, fallback);
                    return fallback;
                } finally {
                    pending.delete(url);
                }
            })();
            pending.set(url, job);
            return job;
        },
    };
}
