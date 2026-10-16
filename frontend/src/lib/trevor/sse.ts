/**
 * A server-sent-events parser for a body read with `fetch`.
 *
 * The job stream needs the account's bearer token, which lives in an httpOnly
 * cookie the browser cannot read, so the stream is proxied through a server
 * function (`lib/api/trevor.ts`) and read here as a byte stream. A native
 * `EventSource` cannot send that header; this does what it would have done
 * with the text: `event:` and `data:` fields, multi-line data, `:` comments
 * (the backend's keep-alive) ignored, a blank line ending each event, `\r\n`
 * and `\n` both accepted.
 */

export interface ISseEvent {
    /** `message` when the event named none, as `EventSource` does. */
    event: string;
    data: string;
}

/** Feed it text in any chunking; it calls `onEvent` once per complete event. */
export function createSseParser(onEvent: (e: ISseEvent) => void): { push: (chunk: string) => void; end: () => void } {
    let buffer = "";
    let event = "";
    let data: string[] = [];
    let any = false;

    const dispatch = () => {
        if (any) onEvent({ event: event || "message", data: data.join("\n") });
        event = "";
        data = [];
        any = false;
    };

    const line = (raw: string) => {
        if (raw === "") return dispatch();
        if (raw.startsWith(":")) return;
        const colon = raw.indexOf(":");
        const field = colon < 0 ? raw : raw.slice(0, colon);
        let value = colon < 0 ? "" : raw.slice(colon + 1);
        if (value.startsWith(" ")) value = value.slice(1);
        if (field === "event") {
            event = value;
            any = true;
        } else if (field === "data") {
            data.push(value);
            any = true;
        }
    };

    return {
        push(chunk: string) {
            buffer += chunk;
            for (;;) {
                const nl = buffer.search(/\r\n|\n|\r/);
                if (nl < 0) break;
                const width = buffer.startsWith("\r\n", nl) ? 2 : 1;
                // A lone `\r` at the very end may be the first half of a `\r\n` still in flight.
                if (buffer[nl] === "\r" && width === 1 && nl === buffer.length - 1) break;
                line(buffer.slice(0, nl));
                buffer = buffer.slice(nl + width);
            }
        },
        end() {
            if (buffer !== "") line(buffer);
            buffer = "";
            dispatch();
        },
    };
}

/**
 * Read a `text/event-stream` body to its end, or until `signal` aborts.
 * `onBytes` hears every chunk, keep-alive comments included, so a caller can
 * tell a quiet stream from a dead one.
 */
export async function readSse(body: ReadableStream<Uint8Array>, onEvent: (e: ISseEvent) => void, signal?: AbortSignal, onBytes?: () => void): Promise<void> {
    const reader = body.getReader();
    const decoder = new TextDecoder();
    const parser = createSseParser(onEvent);
    const onAbort = () => {
        void reader.cancel().catch(() => {});
    };
    signal?.addEventListener("abort", onAbort, { once: true });
    try {
        for (;;) {
            if (signal?.aborted) return;
            const { value, done } = await reader.read();
            if (done) break;
            onBytes?.();
            parser.push(decoder.decode(value, { stream: true }));
        }
        parser.push(decoder.decode());
        parser.end();
    } finally {
        signal?.removeEventListener("abort", onAbort);
        try {
            reader.releaseLock();
        } catch {
            // Already released by a cancel; nothing holds the stream.
        }
    }
}
