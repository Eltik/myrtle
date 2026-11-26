import { describe, expect, it } from "vitest";
import { muxWebm, sizeBytes } from "./webm-mux";

/** Reads one EBML element at `offset`: its id, payload range and the offset after it. */
function readElement(bytes: Uint8Array, offset: number): { id: number; start: number; end: number } {
    const first = bytes[offset] ?? 0;
    const idLength = Math.clz32(first) - 23;
    let id = 0;
    for (let i = 0; i < idLength; i++) id = id * 256 + (bytes[offset + i] ?? 0);
    const sizeAt = offset + idLength;
    const sizeFirst = bytes[sizeAt] ?? 0;
    const sizeLength = Math.clz32(sizeFirst) - 23;
    let size = sizeFirst & (0xff >> sizeLength);
    for (let i = 1; i < sizeLength; i++) size = size * 256 + (bytes[sizeAt + i] ?? 0);
    const start = sizeAt + sizeLength;
    return { id, start, end: start + size };
}

function children(bytes: Uint8Array, start: number, end: number): { id: number; start: number; end: number }[] {
    const out = [];
    for (let offset = start; offset < end; ) {
        const el = readElement(bytes, offset);
        out.push(el);
        offset = el.end;
    }
    return out;
}

describe("sizeBytes", () => {
    it("never writes the reserved all-ones value", () => {
        expect([...sizeBytes(0)]).toEqual([0x80]);
        expect([...sizeBytes(126)]).toEqual([0xfe]);
        expect([...sizeBytes(127)]).toEqual([0x40, 0x7f]);
        expect([...sizeBytes(300)]).toEqual([0x41, 0x2c]);
    });
});

describe("muxWebm", () => {
    const frame = (timestamp: number, key: boolean) => ({ data: new Uint8Array([1, 2, 3]), timestamp, key });
    const bytes = muxWebm({ codec: "V_VP9", width: 320, height: 240, duration: 100_000, frames: [frame(0, true), frame(33_333, false), frame(66_667, true), frame(100_000, false)] });

    it("writes an EBML header and one segment that spans the rest of the file exactly", () => {
        const top = children(bytes, 0, bytes.length);
        expect(top.map((el) => el.id)).toEqual([0x1a45dfa3, 0x18538067]);
        expect(top[1]?.end).toBe(bytes.length);
    });

    it("starts a cluster at each keyframe with the blocks' offsets inside it", () => {
        const segment = readElement(bytes, readElement(bytes, 0).end);
        const parts = children(bytes, segment.start, segment.end);
        expect(parts.map((el) => el.id)).toEqual([0x1549a966, 0x1654ae6b, 0x1f43b675, 0x1f43b675]);
        const second = parts[3];
        if (!second) throw new Error("no second cluster");
        const inner = children(bytes, second.start, second.end);
        // Timecode 67 ms, then two SimpleBlocks at +0 (key) and +33.
        expect(inner.map((el) => el.id)).toEqual([0xe7, 0xa3, 0xa3]);
        const [timecode, keyBlock, deltaBlock] = inner;
        if (!timecode || !keyBlock || !deltaBlock) throw new Error("cluster incomplete");
        expect(bytes[timecode.start]).toBe(67);
        expect(bytes[keyBlock.start + 3]).toBe(0x80);
        expect(new DataView(bytes.buffer, deltaBlock.start + 1, 2).getInt16(0)).toBe(33);
        expect(bytes[deltaBlock.start + 3]).toBe(0x00);
    });
});
