/**
 * A minimal WebM writer for one video track of encoded VP8/VP9 frames, enough for the chibi
 * exporter to save what `VideoEncoder` produces without a muxer dependency. The whole file is
 * built in memory once every frame is in, so every element size is known and written exactly;
 * a new cluster starts at each keyframe and whenever a block's offset would outgrow 16 bits.
 */

export type WebmCodec = "V_VP8" | "V_VP9";

export interface IWebmFrame {
    data: Uint8Array;
    /** Microseconds from the start, as `EncodedVideoChunk.timestamp` gives it. */
    timestamp: number;
    key: boolean;
}

export interface IWebmOptions {
    codec: WebmCodec;
    width: number;
    height: number;
    /** Microseconds; written as the segment's duration. */
    duration: number;
    frames: readonly IWebmFrame[];
}

const ID = {
    EBML: 0x1a45dfa3,
    EBMLVersion: 0x4286,
    EBMLReadVersion: 0x42f7,
    EBMLMaxIDLength: 0x42f2,
    EBMLMaxSizeLength: 0x42f3,
    DocType: 0x4282,
    DocTypeVersion: 0x4287,
    DocTypeReadVersion: 0x4285,
    Segment: 0x18538067,
    Info: 0x1549a966,
    TimecodeScale: 0x2ad7b1,
    MuxingApp: 0x4d80,
    WritingApp: 0x5741,
    Duration: 0x4489,
    Tracks: 0x1654ae6b,
    TrackEntry: 0xae,
    TrackNumber: 0xd7,
    TrackUID: 0x73c5,
    TrackType: 0x83,
    FlagLacing: 0x9c,
    CodecID: 0x86,
    Video: 0xe0,
    PixelWidth: 0xb0,
    PixelHeight: 0xba,
    Cluster: 0x1f43b675,
    Timecode: 0xe7,
    SimpleBlock: 0xa3,
} as const;

/** Timecodes count milliseconds: the WebM default TimecodeScale of 1,000,000 ns. */
const TIMECODE_SCALE = 1_000_000;
const MAX_BLOCK_OFFSET = 0x7fff;

type Part = Uint8Array;

function idBytes(id: number): Part {
    const bytes: number[] = [];
    for (let v = id; v > 0; v = Math.floor(v / 256)) bytes.unshift(v & 0xff);
    return Uint8Array.from(bytes);
}

/** EBML variable-length size: the shortest width whose all-ones value (reserved for "unknown") stays clear. */
export function sizeBytes(size: number): Part {
    let length = 1;
    while (length < 8 && size >= 2 ** (7 * length) - 1) length++;
    const out = new Uint8Array(length);
    let v = size;
    for (let i = length - 1; i >= 0; i--) {
        out[i] = v & 0xff;
        v = Math.floor(v / 256);
    }
    out[0] = (out[0] ?? 0) | (1 << (8 - length));
    return out;
}

function uintBytes(value: number): Part {
    const bytes: number[] = [];
    for (let v = value; v > 0; v = Math.floor(v / 256)) bytes.unshift(v & 0xff);
    return Uint8Array.from(bytes.length > 0 ? bytes : [0]);
}

function concat(parts: readonly Part[]): Uint8Array<ArrayBuffer> {
    const out = new Uint8Array(parts.reduce((sum, part) => sum + part.length, 0));
    let offset = 0;
    for (const part of parts) {
        out.set(part, offset);
        offset += part.length;
    }
    return out;
}

function element(id: number, payload: Part | readonly Part[]): Part {
    const body = payload instanceof Uint8Array ? payload : concat(payload);
    return concat([idBytes(id), sizeBytes(body.length), body]);
}

const uint = (id: number, value: number): Part => element(id, uintBytes(value));
const text = (id: number, value: string): Part => element(id, new TextEncoder().encode(value));

function float64(id: number, value: number): Part {
    const body = new Uint8Array(8);
    new DataView(body.buffer).setFloat64(0, value);
    return element(id, body);
}

function simpleBlock(frame: IWebmFrame, offsetMs: number): Part {
    const header = new Uint8Array(4);
    header[0] = 0x81; // track 1, as a one-byte vint
    new DataView(header.buffer).setInt16(1, offsetMs);
    header[3] = frame.key ? 0x80 : 0x00;
    return element(ID.SimpleBlock, [header, frame.data]);
}

export function muxWebm({ codec, width, height, duration, frames }: IWebmOptions): Uint8Array<ArrayBuffer> {
    const header = element(ID.EBML, [uint(ID.EBMLVersion, 1), uint(ID.EBMLReadVersion, 1), uint(ID.EBMLMaxIDLength, 4), uint(ID.EBMLMaxSizeLength, 8), text(ID.DocType, "webm"), uint(ID.DocTypeVersion, 2), uint(ID.DocTypeReadVersion, 2)]);

    const info = element(ID.Info, [uint(ID.TimecodeScale, TIMECODE_SCALE), text(ID.MuxingApp, "myrtle"), text(ID.WritingApp, "myrtle"), float64(ID.Duration, duration / 1000)]);

    const tracks = element(ID.Tracks, element(ID.TrackEntry, [uint(ID.TrackNumber, 1), uint(ID.TrackUID, 1), uint(ID.TrackType, 1), uint(ID.FlagLacing, 0), text(ID.CodecID, codec), element(ID.Video, [uint(ID.PixelWidth, width), uint(ID.PixelHeight, height)])]));

    const clusters: Part[] = [];
    let blocks: Part[] = [];
    let clusterStart = 0;
    const flush = () => {
        if (blocks.length > 0) clusters.push(element(ID.Cluster, [uint(ID.Timecode, clusterStart), ...blocks]));
        blocks = [];
    };
    for (const frame of frames) {
        const ms = Math.round(frame.timestamp / 1000);
        if (blocks.length === 0 || frame.key || ms - clusterStart > MAX_BLOCK_OFFSET) {
            flush();
            clusterStart = ms;
        }
        blocks.push(simpleBlock(frame, ms - clusterStart));
    }
    flush();

    return concat([header, element(ID.Segment, [info, tracks, ...clusters])]);
}
