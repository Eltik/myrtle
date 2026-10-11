import { describe, expect, it } from "vitest";
import { rememberSirenCoverHosts, sizedCover } from "./cover";

describe("sizedCover", () => {
    const siren = "https://web.hycdn.cn/siren/pic/20261010/ab46323f8b0d4580e34b61c837851cc7.jpg";
    const unseen = "https://other.cdn.example/siren/pic/a.jpg";

    it("leaves a host the catalogue never served alone", () => {
        expect(sizedCover(unseen, 28)).toBe(unseen);
    });

    it("rewrites only once the catalogue has served the host", () => {
        rememberSirenCoverHosts([siren, "not a url"]);
        expect(sizedCover(siren, 28)).toBe(`${siren}?x-oss-process=image/resize,w_56/format,webp`);
    });

    it("asks the Siren CDN for a WebP cut at twice the drawn width", () => {
        expect(sizedCover(siren, 28)).toBe(`${siren}?x-oss-process=image/resize,w_56/format,webp`);
    });

    it("never asks for more than the 500 px original", () => {
        expect(sizedCover(siren, 400)).toBe(`${siren}?x-oss-process=image/resize,w_500/format,webp`);
    });

    it("passes other hosts, queried URLs and null through", () => {
        expect(sizedCover("https://example.com/a.jpg", 28)).toBe("https://example.com/a.jpg");
        expect(sizedCover(`${siren}?v=1`, 28)).toBe(`${siren}?v=1`);
        expect(sizedCover(null, 28)).toBeNull();
        expect(sizedCover("not a url", 28)).toBe("not a url");
    });
});
