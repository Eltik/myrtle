import { describe, expect, it } from "vitest";
import type { IAlbum, ITrack } from "../impl/model";
import { filterPrepared, filterSongs, prepareAlbums, prepareArtists, prepareSongs } from "./filters";
import { moveTrack } from "./reorder";
import { sortAlbums, sortSongs } from "./sort";

const HOUSE = "House";
const collator = new Intl.Collator("en", { sensitivity: "base", numeric: true });

function album(cid: string, name: string, listOrder: number, trackCount: number): IAlbum {
    return { id: `siren:${cid}`, cid, name, coverUrl: "", artists: [HOUSE], listOrder, trackCount, lyricsCount: 0, instrumentalCount: 0, addedOn: null, showAddedOn: false };
}

function track(key: string, title: string, albumName: string, trackOrder: number, artists: string[] = [HOUSE], instrumental = false): ITrack {
    return { id: `siren:${key}`, kind: "siren", key, title, artists, albumId: `siren:${albumName}`, albumName, coverUrl: null, instrumental, hasLyrics: false, trackOrder };
}

const ALBUMS = [album("a", "Zeta", 0, 3), album("b", "alpha", 1, 9), album("c", "Mid", 2, 9)];
const TRACKS = [track("1", "Speed of Light", "Ost B", 2, [HOUSE, "Mili"]), track("2", "Bonfire (Instrumental)", "Ost A", 1, [HOUSE], true), track("3", "after rain", "Ost B", 1, ["Ave Mujica"]), track("4", "Lullaby", "Ost A", 2)];

const keys = (list: { key?: string; cid?: string }[]) => list.map((x) => x.key ?? x.cid);

describe("sortAlbums", () => {
    it("newest first is list order, and reverses", () => {
        expect(keys(sortAlbums(ALBUMS, "newest", false, collator))).toEqual(["a", "b", "c"]);
        expect(keys(sortAlbums(ALBUMS, "newest", true, collator))).toEqual(["c", "b", "a"]);
    });

    it("title is alphabetical ignoring case", () => {
        expect(keys(sortAlbums(ALBUMS, "title", false, collator))).toEqual(["b", "c", "a"]);
    });

    it("track count is biggest first, ties in list order", () => {
        expect(keys(sortAlbums(ALBUMS, "tracks", false, collator))).toEqual(["b", "c", "a"]);
    });
});

describe("sortSongs", () => {
    it("catalog keeps the input order and reverses", () => {
        expect(keys(sortSongs(TRACKS, "catalog", false, collator, HOUSE))).toEqual(["1", "2", "3", "4"]);
        expect(keys(sortSongs(TRACKS, "catalog", true, collator, HOUSE))).toEqual(["4", "3", "2", "1"]);
    });

    it("title ignores the instrumental marker and case", () => {
        expect(keys(sortSongs(TRACKS, "title", false, collator, HOUSE))).toEqual(["3", "2", "4", "1"]);
    });

    it("album sorts by album name, then track order", () => {
        expect(keys(sortSongs(TRACKS, "album", false, collator, HOUSE))).toEqual(["2", "4", "3", "1"]);
    });

    it("artist sorts by the guests a row shows, else the house name", () => {
        // Ave Mujica, House, House, Mili; the two House rows keep their input order.
        expect(keys(sortSongs(TRACKS, "artist", false, collator, HOUSE))).toEqual(["3", "2", "4", "1"]);
    });
});

describe("filters", () => {
    const songs = prepareSongs(TRACKS);

    it("a blank query keeps everything in order", () => {
        expect(keys(filterSongs(songs, "  ", { hideInstrumentals: false }, true))).toEqual(["1", "2", "3", "4"]);
    });

    it("finds a song by title, artist or album", () => {
        expect(keys(filterSongs(songs, "speed", { hideInstrumentals: false }, true))).toEqual(["1"]);
        expect(keys(filterSongs(songs, "mili", { hideInstrumentals: false }, true))).toEqual(["1"]);
        expect(keys(filterSongs(songs, "ost a", { hideInstrumentals: false }, false))).toEqual(["2", "4"]);
    });

    it("hides instrumentals when asked", () => {
        expect(keys(filterSongs(songs, "", { hideInstrumentals: true }, true))).toEqual(["1", "3", "4"]);
    });

    it("ranks the title match above an alias match when ranking", () => {
        const list = prepareSongs([track("x", "Other", "Lullaby Collection", 1), track("y", "Lullaby", "Ost", 1)]);
        expect(keys(filterSongs(list, "lullaby", { hideInstrumentals: false }, true))).toEqual(["y", "x"]);
        expect(keys(filterSongs(list, "lullaby", { hideInstrumentals: false }, false))).toEqual(["x", "y"]);
    });

    it("searches albums and artists by name", () => {
        expect(keys(filterPrepared(prepareAlbums(ALBUMS), "alp", true))).toEqual(["b"]);
        expect(
            filterPrepared(
                prepareArtists([
                    { name: "Mili", trackCount: 2 },
                    { name: "Low Roar", trackCount: 1 },
                ]),
                "roar",
                true,
            ).map((a) => a.name),
        ).toEqual(["Low Roar"]);
    });
});

describe("moveTrack", () => {
    const ids = ["a", "b", "c", "d"];

    it("moves down to after the target and up to before it", () => {
        expect(moveTrack(ids, "a", "c")).toEqual(["b", "c", "a", "d"]);
        expect(moveTrack(ids, "d", "b")).toEqual(["a", "d", "b", "c"]);
    });

    it("keeps hidden ids in place and ignores unknown ones", () => {
        expect(moveTrack(["a", "ghost", "b", "c"], "c", "b")).toEqual(["a", "ghost", "c", "b"]);
        expect(moveTrack(ids, "a", "zzz")).toEqual(ids);
    });
});
