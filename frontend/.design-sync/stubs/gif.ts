// Design-bundle stand-in for `gif.js`: only the chibi recorder's "record GIF" click
// reaches it, and the bundle must stay under the 12 MiB upload cap.
export default class GIF {
    constructor() {
        throw new Error("Recording isn't available in this preview.");
    }
}
