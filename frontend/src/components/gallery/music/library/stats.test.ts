import { describe, expect, it } from "vitest";
import { listeningTime } from "../impl/format";
import type { IAlbum, ITrack } from "../impl/model";
import { statsOf } from "./stats";

const HOUSE = "House";

function track(key: string, albumId: string | null, artists: string[]): ITrack {
    return { id: `siren:${key}`, kind: "siren", key, title: key, artists, albumId, albumName: albumId, coverUrl: null, instrumental: false, hasLyrics: false, trackOrder: 1 };
}

const A: IAlbum = { id: "siren:A", cid: "A", name: "A", coverUrl: "", artists: [], listOrder: 0, trackCount: 2, lyricsCount: 0, instrumentalCount: 0, addedOn: null, showAddedOn: false };
const B: IAlbum = { ...A, id: "siren:B", cid: "B", name: "B", listOrder: 1 };
const tracks = [track("1", "siren:A", [HOUSE, "Mili"]), track("2", "siren:A", [HOUSE]), track("3", "siren:B", ["Mili", "Low Roar"])];
const catalog = { byId: new Map(tracks.map((t) => [t.id, t])), albumById: new Map([A, B].map((a) => [a.id, a])), houseArtist: HOUSE };

describe("statsOf", () => {
    const totals = [
        { trackId: "siren:2", plays: 5, seconds: 600, lastPlayedAt: null },
        { trackId: "siren:3", plays: 3, seconds: 300, lastPlayedAt: null },
        { trackId: "siren:gone", plays: 2, seconds: 100, lastPlayedAt: null },
        { trackId: "siren:1", plays: 1, seconds: 30, lastPlayedAt: null },
    ];
    const s = statsOf(totals, catalog);

    it("counts every play and second, known to the catalogue or not", () => {
        expect(s.plays).toBe(11);
        expect(s.seconds).toBe(1030);
    });

    it("lists only known tracks, most played first", () => {
        expect(s.topTracks.map((r) => [r.track.key, r.plays])).toEqual([
            ["2", 5],
            ["3", 3],
            ["1", 1],
        ]);
    });

    it("leaves the house artist out of the top artists", () => {
        expect(s.topArtists).toEqual([
            { name: "Mili", plays: 4 },
            { name: "Low Roar", plays: 3 },
        ]);
    });

    it("sums plays per album", () => {
        expect(s.topAlbums.map((r) => [r.album.cid, r.plays])).toEqual([
            ["A", 6],
            ["B", 3],
        ]);
    });
});

describe("listeningTime", () => {
    it("prints seconds, minutes, then hours and minutes", () => {
        expect(listeningTime(40)).toEqual({ unit: "seconds", seconds: 40 });
        expect(listeningTime(18 * 60 + 59)).toEqual({ unit: "minutes", minutes: 18 });
        expect(listeningTime(3900)).toEqual({ unit: "hours", hours: 1, minutes: 5 });
        expect(listeningTime(Number.NaN)).toEqual({ unit: "seconds", seconds: 0 });
    });
});
