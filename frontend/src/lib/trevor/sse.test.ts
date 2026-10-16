import { describe, expect, it } from "vitest";
import { createSseParser, type ISseEvent, readSse } from "./sse";

function parse(chunks: string[]): ISseEvent[] {
    const out: ISseEvent[] = [];
    const p = createSseParser((e) => out.push(e));
    for (const c of chunks) p.push(c);
    p.end();
    return out;
}

function body(chunks: string[]): ReadableStream<Uint8Array> {
    const enc = new TextEncoder();
    return new ReadableStream({
        start(c) {
            for (const s of chunks) c.enqueue(enc.encode(s));
            c.close();
        },
    });
}

describe("the SSE parser", () => {
    it("reads named events, ignores keep-alive comments and joins multi-line data", () => {
        const events = parse([': keepalive\n\nevent: delta\ndata: {"seq":0,"text":"Hel"}\n\n', "event: done\ndata: line one\ndata: line two\n\n"]);
        expect(events).toEqual([
            { event: "delta", data: '{"seq":0,"text":"Hel"}' },
            { event: "done", data: "line one\nline two" },
        ]);
    });

    it("is indifferent to how the bytes are chunked, CRLF included", () => {
        const whole = 'event: delta\r\ndata: {"seq":1,"text":"lo"}\r\n\r\nevent: error\r\ndata: {"message":"x"}\r\n\r\n';
        const one = parse([whole]);
        const split = parse(whole.split(""));
        expect(split).toEqual(one);
        expect(one.map((e) => e.event)).toEqual(["delta", "error"]);
    });

    it("names an unnamed event `message`, as EventSource does", () => {
        expect(parse(["data: hi\n\n"])).toEqual([{ event: "message", data: "hi" }]);
    });

    it("reads a fetch body to its end", async () => {
        const out: ISseEvent[] = [];
        await readSse(body(["event: delta\nda", 'ta: {"seq":0,"text":"a"}\n', "\n"]), (e) => out.push(e));
        expect(out).toEqual([{ event: "delta", data: '{"seq":0,"text":"a"}' }]);
    });
});
