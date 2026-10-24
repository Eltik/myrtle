/**
 * Batch plumbing for `POST /{dps,hps}/batch`. One call carries at most
 * {@link BATCH_LIMIT} requests (the backend's `MAX_BATCH`, a 400 above it), so a
 * longer list is split in order and the answers are concatenated back in order.
 */

/** `MAX_BATCH` in `backend/src/app/services/dps.rs`. */
export const BATCH_LIMIT = 64;

/** `items` in consecutive runs of at most `size`, order kept. An empty list gives no chunks. */
export function chunk<T>(items: readonly T[], size: number = BATCH_LIMIT): T[][] {
    if (!Number.isInteger(size) || size < 1) throw new Error(`chunk size must be a positive integer (got ${size})`);
    const out: T[][] = [];
    for (let i = 0; i < items.length; i += size) out.push(items.slice(i, i + size));
    return out;
}

/** The outcome of one request inside a chunked batch, in the order the requests were given. */
export interface IBatchSlot<R> {
    data: R | undefined;
    isPending: boolean;
    error: Error | null;
}

/**
 * Reassemble per-request slots from the per-chunk query states. A chunk that failed as a
 * whole (network, 400) fails each of its requests with that error; a chunk that answered
 * carries a per-request `error` string for a request the engine could not compute.
 */
export function flattenChunks<I extends { error?: string }, R>(chunkSizes: readonly number[], chunks: readonly { data?: { results: I[] }; isPending: boolean; error: Error | null }[], pick: (item: I) => R | undefined): IBatchSlot<R>[] {
    const out: IBatchSlot<R>[] = [];
    chunkSizes.forEach((size, c) => {
        const q = chunks[c];
        for (let i = 0; i < size; i++) {
            const item = q?.data?.results[i];
            if (item?.error) out.push({ data: undefined, isPending: false, error: new Error(item.error) });
            else out.push({ data: item ? pick(item) : undefined, isPending: q?.isPending ?? true, error: q?.error ?? null });
        }
    });
    return out;
}
