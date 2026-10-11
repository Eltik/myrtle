import { describe, expect, it } from "vitest";
import { searchFor, selectionOf, tabOf, validateMusicSearch } from "./route";

describe("validateMusicSearch", () => {
    it("keeps known values and drops the rest", () => {
        expect(validateMusicSearch({ tab: "library", view: "liked", album: " 123 ", junk: 1 })).toEqual({ tab: "library", view: "liked", album: "123" });
        expect(validateMusicSearch({ tab: "discovery", view: "nope", playlist: "", artist: 5 })).toEqual({ artist: "5" });
    });

    it("reads any Library param as the Library tab", () => {
        expect(tabOf({})).toBe("discovery");
        expect(tabOf({ view: "stats" })).toBe("library");
        expect(tabOf({ artist: "Mili" })).toBe("library");
    });

    it("prefers a playlist, then an artist, then the view", () => {
        expect(selectionOf({ view: "songs", artist: "Mili", playlist: "p1" }, "albums")).toEqual({ kind: "playlist", id: "p1" });
        expect(selectionOf({ view: "songs", artist: "Mili" }, "albums")).toEqual({ kind: "artist", name: "Mili" });
        expect(selectionOf({}, "albums")).toEqual({ kind: "view", view: "albums" });
    });

    it("keeps an open album when the view changes", () => {
        expect(searchFor({ kind: "artist", name: "Mili" }, { view: "songs", album: "9" })).toEqual({ tab: "library", album: "9", artist: "Mili" });
    });
});
