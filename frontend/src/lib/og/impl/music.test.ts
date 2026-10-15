import { describe, expect, it } from "vitest";
import type { SirenAlbumDetail } from "#/types/generated/SirenAlbumDetail";
import { ogHashers } from "./hashers";
import { buildMusicOgData, isMusicAlbumId, sirenCoverAt } from "./music";

const ALBUM = {
    cid: "0242",
    name: "Fleeting Wish (Monster Siren Records) ",
    intro: "",
    belong: "arknights",
    coverUrl: "https://web.hycdn.cn/siren/pic/20231125/1b0b.jpg",
    coverDeUrl: null,
    artists: [" 塞壬唱片-MSR "],
    listOrder: 133,
    addedOn: "2023-11-25",
    songs: [
        { cid: "2", name: "B", trackOrder: 1 },
        { cid: "1", name: "A", trackOrder: 0 },
    ],
} as unknown as SirenAlbumDetail;

describe("music card", () => {
    it("accepts only Monster Siren album cids", () => {
        expect(isMusicAlbumId("0242")).toBe(true);
        expect(isMusicAlbumId("8913")).toBe(true);
        expect(isMusicAlbumId("../etc")).toBe(false);
        expect(isMusicAlbumId("")).toBe(false);
        expect(isMusicAlbumId("12a")).toBe(false);
    });

    it("orders tracks, trims text, and hashes the cover source", () => {
        const data = buildMusicOgData(ALBUM);
        expect(data.tracks).toEqual(["A", "B"]);
        expect(data.name).toBe("Fleeting Wish (Monster Siren Records)");
        expect(data.artists).toEqual(["塞壬唱片-MSR"]);
        const moved = buildMusicOgData({ ...ALBUM, coverUrl: "https://web.hycdn.cn/siren/pic/x/other.jpg" });
        expect(ogHashers.music.hash(moved)).not.toBe(ogHashers.music.hash(data));
    });

    it("asks the label's CDN for a resized cover, and nothing else", () => {
        expect(sirenCoverAt("https://web.hycdn.cn/siren/pic/a.jpg", 8)).toBe("https://web.hycdn.cn/siren/pic/a.jpg?x-oss-process=image%2Fresize%2Cw_8");
        expect(sirenCoverAt("https://example.com/a.jpg", 8)).toBeNull();
        expect(sirenCoverAt("not a url", 8)).toBeNull();
    });
});
