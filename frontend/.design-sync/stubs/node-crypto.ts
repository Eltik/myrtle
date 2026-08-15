// Design-sync stub for `node:crypto`, imported by src/lib/api/tier-lists.ts for
// a share-token generator. Node builtins can't be bundled for the browser, and
// the only consumer is a server-side code path a preview never reaches.
//
// Wired in through `compilerOptions.paths` in .design-sync/tsconfig.ds.json.

export function randomBytes(size: number) {
    const bytes = new Uint8Array(size);
    if (typeof globalThis.crypto?.getRandomValues === "function") globalThis.crypto.getRandomValues(bytes);
    return {
        toString: (_encoding?: string) => Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join(""),
    };
}
