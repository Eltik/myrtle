import { afterEach, describe, expect, it, vi } from "vitest";
import { fetchToDataURI, inlineArt, inlineRemoteImages, ogEntityIconURL, satoriImageType } from "./art";

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

describe("inlineRemoteImages", () => {
    afterEach(() => {
        vi.unstubAllGlobals();
    });

    it("inlines picture URLs at any depth and drops the ones that 404 instead of failing the card", async () => {
        vi.stubGlobal(
            "fetch",
            vi.fn(async (url: string) => (url.endsWith("logo_laterano.png") ? new Response("nope", { status: 404, headers: { "content-type": "text/plain" } }) : new Response(new Uint8Array([1, 2, 3]), { status: 200, headers: { "content-type": "image/png" } }))),
        );
        const data = {
            name: "Lemuen",
            charArtURL: "https://b/api/assets/textures/chararts/x.png",
            factionLogoURL: "https://b/api/assets/textures/spritepack/ui_camp_logo_0/logo_laterano.png",
            units: [{ avatarURL: "https://b/api/avatar/x", skills: [{ iconURL: "https://b/api/skill-icon/s" }] }],
            stats: [{ label: "HP", value: "1,000" }],
            inlined: { artURL: "data:image/png;base64,AA==" },
            siteURL: "https://myrtle.moe",
        };
        const out = await inlineRemoteImages(data);
        expect(out.charArtURL).toBe("data:image/png;base64,AQID");
        expect(out.factionLogoURL).toBeUndefined();
        expect(out.units[0]?.avatarURL).toBe("data:image/png;base64,AQID");
        expect(out.units[0]?.skills[0]?.iconURL).toBe("data:image/png;base64,AQID");
        // Untouched: text, an already-inlined picture, and a URL that is not a picture.
        expect(out.name).toBe("Lemuen");
        expect(out.stats).toEqual(data.stats);
        expect(out.inlined.artURL).toBe("data:image/png;base64,AA==");
        expect(out.siteURL).toBe("https://myrtle.moe");
        // The input is not mutated: it is what the content hash was computed from.
        expect(data.factionLogoURL).toContain("logo_laterano.png");
        expect(vi.mocked(fetch)).toHaveBeenCalledTimes(4);
    });

    it("returns the same object and fetches nothing when there is no remote picture", async () => {
        vi.stubGlobal("fetch", vi.fn());
        const data = { name: "x", artURL: "" };
        expect(await inlineRemoteImages(data)).toBe(data);
        expect(vi.mocked(fetch)).not.toHaveBeenCalled();
    });
});
