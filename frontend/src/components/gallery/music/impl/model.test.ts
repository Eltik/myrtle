import { describe, expect, it } from "vitest";
import type { MusicIndex } from "#/types/generated/MusicIndex";
import type { SirenAlbum } from "#/types/generated/SirenAlbum";
import type { SirenCatalog } from "#/types/generated/SirenCatalog";
import type { SirenSong } from "#/types/generated/SirenSong";
import { buildCatalog, radioFor } from "./model";

const HOUSE = "House Label";
const BULK = "2021-03-22";

function album(cid: string, listOrder: number, addedOn: string | null, over: Partial<SirenAlbum> = {}): SirenAlbum {
    return { cid, name: `Album ${cid}`, coverUrl: `https://cdn/${cid}.jpg`, artists: [HOUSE], listOrder, trackCount: 0, lyricsCount: 0, instrumentalCount: 0, addedOn, ...over };
}

function song(cid: string, albumCid: string, trackOrder: number, artists: string[] = [HOUSE], over: Partial<SirenSong> = {}): SirenSong {
    return { cid, name: `Song ${cid}`, title: `Song ${cid}`, albumCid, artists, hasLyrics: false, instrumental: false, hasMv: false, trackOrder, ...over };
}

// Listed out of order on purpose: list order 0 is the newest release.
const SIREN: SirenCatalog = {
    albums: [album("old", 2, BULK), album("new", 0, "2026-10-10"), album("mid", 1, "2025-05-01")],
    songs: [song("n2", "new", 2), song("n1", "new", 1, [HOUSE, "Guest A"]), song("m1", "mid", 1, ["Guest B"]), song("m2", "mid", 2), song("o1", "old", 1), song("o2", "old", 2, ["Guest A"])],
    syncedAt: null,
    bulkImportDay: BULK,
};

const GAME: MusicIndex = {
    totals: { tracks: 3, titled: 3, unavailable: 0, withAudio: 3, fromOtherServer: 0 },
    tracks: [
        { id: "m_a", title: "A", origin: "Event X", kinds: ["lobby"], banks: [], places: [], loopUrl: "/a.ogg" },
        { id: "m_b", title: "B", origin: "Event Y", kinds: ["battle"], banks: [], places: [], loopUrl: "/b.ogg" },
        { id: "m_c", title: "C", origin: "Event X", kinds: ["battle"], banks: [], places: [], loopUrl: "/c.ogg" },
    ],
};

const collator = new Intl.Collator("en", { sensitivity: "base", numeric: true });

describe("buildCatalog", () => {
    const c = buildCatalog(SIREN, GAME, collator);

    it("orders albums newest first by list order", () => {
        expect(c.albums.map((a) => a.cid)).toEqual(["new", "mid", "old"]);
    });

    it("derives the house artist as the most credited name", () => {
        expect(c.houseArtist).toBe(HOUSE);
    });

    it("lists only guest artists, alphabetically, with their track counts", () => {
        expect(c.artists).toEqual([
            { name: "Guest A", trackCount: 2 },
            { name: "Guest B", trackCount: 1 },
        ]);
    });

    it("hides the added date on the bulk import day only", () => {
        const shown = Object.fromEntries(c.albums.map((a) => [a.cid, a.showAddedOn]));
        expect(shown).toEqual({ new: true, mid: true, old: false });
    });

    it("puts label tracks in album then track order, then the game tracks", () => {
        expect(c.tracks.map((t) => t.id)).toEqual(["siren:n1", "siren:n2", "siren:m1", "siren:m2", "siren:o1", "siren:o2", "game:m_a", "game:m_b", "game:m_c"]);
        expect(c.byId.get("siren:m1")?.albumName).toBe("Album mid");
    });

    it("reads an empty catalogue as not ready", () => {
        const empty = buildCatalog(null, null, collator);
        expect(empty.sirenReady).toBe(false);
        expect(empty.houseArtist).toBeNull();
        expect(empty.tracks).toEqual([]);
    });
});

describe("radioFor", () => {
    const c = buildCatalog(SIREN, GAME, collator);
    const ids = (list: { id: string }[]) => list.map((t) => t.id);

    it("plays the rest of the album first, then the neighbours newer before older", () => {
        const from = c.byId.get("siren:m1");
        if (!from) throw new Error("fixture");
        expect(ids(radioFor(c, from, new Set()))).toEqual(["siren:m2", "siren:n1", "siren:n2", "siren:o1", "siren:o2"]);
    });

    it("alternates outward from the newest album", () => {
        const from = c.byId.get("siren:n2");
        if (!from) throw new Error("fixture");
        expect(ids(radioFor(c, from, new Set()))).toEqual(["siren:m1", "siren:m2", "siren:o1", "siren:o2"]);
    });

    it("leaves out what is already queued, and stops at the limit", () => {
        const from = c.byId.get("siren:m1");
        if (!from) throw new Error("fixture");
        expect(ids(radioFor(c, from, new Set(["siren:m2", "siren:n1"])))).toEqual(["siren:n2", "siren:o1", "siren:o2"]);
        expect(ids(radioFor(c, from, new Set(), 2))).toEqual(["siren:m2", "siren:n1"]);
    });

    it("plays game tracks of the same origin first", () => {
        const from = c.byId.get("game:m_a");
        if (!from) throw new Error("fixture");
        expect(ids(radioFor(c, from, new Set()))).toEqual(["game:m_c", "game:m_b"]);
    });
});
