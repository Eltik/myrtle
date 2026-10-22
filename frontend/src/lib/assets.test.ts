import { afterEach, describe, expect, it, vi } from "vitest";

const mockEnv = vi.hoisted(() => ({ VITE_BACKEND_URL: "https://api.example" as string | undefined, VITE_ASSET_BASE_URL: undefined as string | undefined }));
vi.mock("#/env", () => ({ env: mockEnv }));

import { assetStoreURL, assetURL, encodeAssetPath, isAssetStoreURL } from "./assets";

const STORE = "https://cdn.example/assets";

/** What `asset()` in operators/detail/impl/assets.ts built before this module existed. */
function oldAsset(path: string, server?: string): string {
    const prefix = server && server !== "en" ? `${server}/` : "";
    return `${mockEnv.VITE_BACKEND_URL}/api/${prefix}assets${path}`;
}

afterEach(() => {
    mockEnv.VITE_ASSET_BASE_URL = undefined;
});

describe("assetURL with the store off", () => {
    it("is exactly the string the old asset() built", () => {
        const inputs: Array<[string, string | undefined]> = [
            ["/textures/chararts/char_002_amiya/char_002_amiya_2.png", undefined],
            ["/textures/skinpack/char_002_amiya/char_002_amiya_epoque%234b.png", "cn"],
            ["/textures/arts/ui/[uc]classicgachapool/classic_gacha%230.png", "en"],
        ];
        for (const [path, server] of inputs) expect(assetURL(path, server)).toBe(oldAsset(path, server));
    });

    it("prefixes non-default servers and accepts a path without a leading slash", () => {
        expect(assetURL("textures/x.png")).toBe("https://api.example/api/assets/textures/x.png");
        expect(assetURL("textures/x.png", "cn")).toBe("https://api.example/api/cn/assets/textures/x.png");
        expect(assetURL("/textures/x.png", "jp")).toBe("https://api.example/api/jp/assets/textures/x.png");
    });

    it("never returns a store URL", () => {
        expect(assetStoreURL("/textures/x.png")).toBeNull();
        expect(isAssetStoreURL("https://cdn.example/assets/en/textures/x.png")).toBe(false);
    });
});

describe("assetURL with the store on", () => {
    it("carries the server for en and cn alike", () => {
        mockEnv.VITE_ASSET_BASE_URL = STORE;
        expect(assetURL("/textures/x.png")).toBe(`${STORE}/en/textures/x.png`);
        expect(assetURL("/textures/x.png", "en")).toBe(`${STORE}/en/textures/x.png`);
        expect(assetURL("textures/x.png", "cn")).toBe(`${STORE}/cn/textures/x.png`);
    });

    it("encodes brackets and spaces per segment and keeps existing escapes", () => {
        mockEnv.VITE_ASSET_BASE_URL = STORE;
        expect(assetURL("/spine/DynIllust/char_1012_skadi2/[scene].json")).toBe(`${STORE}/en/spine/DynIllust/char_1012_skadi2/%5Bscene%5D.json`);
        expect(assetURL("/textures/pkgrps/btl_pfb_fx_mapeff_5/water flow_01.png")).toBe(`${STORE}/en/textures/pkgrps/btl_pfb_fx_mapeff_5/water%20flow_01.png`);
        expect(assetURL("/textures/skinpack/char_002_amiya/char_002_amiya_epoque%234b.png", "cn")).toBe(`${STORE}/cn/textures/skinpack/char_002_amiya/char_002_amiya_epoque%234b.png`);
        expect(assetURL("/textures/avg/characters/avg_225_haak_1/1$1.png")).toBe(`${STORE}/en/textures/avg/characters/avg_225_haak_1/1%241.png`);
    });

    it("is idempotent, so a pre-encoded path is never double-encoded", () => {
        const raw = "/spine/a b/[particles]/x%23y.png";
        expect(encodeAssetPath(encodeAssetPath(raw))).toBe(encodeAssetPath(raw));
        expect(encodeAssetPath(raw)).toBe("/spine/a%20b/%5Bparticles%5D/x%23y.png");
    });

    it("keeps a trailing slash and a query", () => {
        mockEnv.VITE_ASSET_BASE_URL = STORE;
        expect(assetURL("/spine/x/[scene]/")).toBe(`${STORE}/en/spine/x/%5Bscene%5D/`);
        expect(assetURL("/audio/a/b.ogg?v=2")).toBe(`${STORE}/en/audio/a/b.ogg?v=2`);
    });

    it("does not produce a double slash from a trailing slash on the base", () => {
        mockEnv.VITE_ASSET_BASE_URL = `${STORE}/`;
        expect(assetURL("/textures/x.png")).toBe(`${STORE}/en/textures/x.png`);
        expect(isAssetStoreURL(`${STORE}/en/textures/x.png`)).toBe(true);
    });

    it("leaves servers with partial trees and paths outside the store roots on the backend", () => {
        mockEnv.VITE_ASSET_BASE_URL = STORE;
        expect(assetURL("/textures/x.png", "jp")).toBe("https://api.example/api/jp/assets/textures/x.png");
        expect(assetURL("/derived/x.json")).toBe("https://api.example/api/assets/derived/x.json");
        expect(assetStoreURL("x.png")).toBeNull();
    });

    it("recognises store URLs only under the base", () => {
        mockEnv.VITE_ASSET_BASE_URL = STORE;
        expect(isAssetStoreURL(`${STORE}/cn/spine/a.skel`)).toBe(true);
        expect(isAssetStoreURL(`${STORE}-other/en/a.png`)).toBe(false);
        expect(isAssetStoreURL("https://api.example/api/assets/textures/x.png")).toBe(false);
    });
});
