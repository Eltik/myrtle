// Design-bundle stand-in for `mp4-muxer`: only the chibi recorder's "record MP4"
// click reaches it (a lazy import in use-recorder.ts), and the bundle must stay
// under the 12 MiB upload cap. See story-book-pdf.ts.
const unavailable = () => new Error("Recording isn't available in this preview.");

export class ArrayBufferTarget {
    buffer: ArrayBuffer = new ArrayBuffer(0);
}

export class Muxer {
    constructor() {
        throw unavailable();
    }
}
