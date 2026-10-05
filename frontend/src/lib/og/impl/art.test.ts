import { afterEach, describe, expect, it, vi } from "vitest";
import { fetchToDataURI, inlineArt, ogEntityIconURL, satoriImageType } from "./art";

describe("satoriImageType", () => {
    it("accepts the four types satori decodes, ignoring parameters and case", () => {
        expect(satoriImageType("image/png")).toBe("image/png");
        expect(satoriImageType("IMAGE/JPEG; charset=binary")).toBe("image/jpeg");
        expect(satoriImageType("image/jpg")).toBe("image/jpeg");
        expect(satoriImageType("image/gif")).toBe("image/gif");
        expect(satoriImageType("image/svg+xml")).toBe("image/svg+xml");
    });

    it("rejects WebP, AVIF, non-images and a missing header", () => {
        expect(satoriImageType("image/webp")).toBeUndefined();
        expect(satoriImageType("image/avif")).toBeUndefined();
        expect(satoriImageType("application/octet-stream")).toBeUndefined();
        expect(satoriImageType(null)).toBeUndefined();
        expect(satoriImageType("")).toBeUndefined();
    });
});

describe("ogEntityIconURL", () => {
    it("asks the story sprite thumbnail for PNG, on the default server and another", () => {
        expect(ogEntityIconURL("/story-sprite-thumb/avg_npc_253", "https://b")).toBe("https://b/api/story-sprite-thumb/avg_npc_253?format=png");
        expect(ogEntityIconURL("/story-sprite-thumb/avg_npc_253", "https://b", "cn")).toBe("https://b/api/cn/story-sprite-thumb/avg_npc_253?format=png");
    });

    it("leaves PNG routes alone", () => {
        expect(ogEntityIconURL("/avatar/char_002_amiya", "https://b")).toBe("https://b/api/avatar/char_002_amiya");
        expect(ogEntityIconURL("/assets/textures/x.png", "https://b")).toBe("https://b/api/assets/textures/x.png");
    });
});

describe("fetchToDataURI", () => {
    afterEach(() => {
        vi.unstubAllGlobals();
    });

    function serve(contentType: string | null, ok = true) {
        const headers = new Headers();
        if (contentType) headers.set("content-type", contentType);
        vi.stubGlobal(
            "fetch",
            vi.fn(async () => new Response(ok ? new Uint8Array([1, 2, 3]) : null, { status: ok ? 200 : 404, headers })),
        );
    }

    it("inlines a PNG under its normalised type", async () => {
        serve("image/png");
        expect(await fetchToDataURI("https://b/x")).toBe("data:image/png;base64,AQID");
    });

    it("treats WebP, a missing type and a failed fetch as missing", async () => {
        serve("image/webp");
        expect(await fetchToDataURI("https://b/x")).toBeUndefined();
        serve(null);
        expect(await fetchToDataURI("https://b/x")).toBeUndefined();
        serve("image/png", false);
        expect(await fetchToDataURI("https://b/x")).toBeUndefined();
        expect(await fetchToDataURI("")).toBeUndefined();
    });

    it("inlineArt fetches each distinct URL once and skips empties", async () => {
        serve("image/gif");
        const art = await inlineArt(["https://b/a", "https://b/a", "", null, undefined, "https://b/b"]);
        expect([...art.keys()]).toEqual(["https://b/a", "https://b/b"]);
        expect(art.get("https://b/a")).toBe("data:image/gif;base64,AQID");
        expect(vi.mocked(fetch)).toHaveBeenCalledTimes(2);
    });
});
