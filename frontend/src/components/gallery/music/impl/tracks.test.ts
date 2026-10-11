import { describe, expect, it } from "vitest";
import type { MusicTrack } from "#/types/generated/MusicTrack";
import { clipFileName, displayName, filterTracks, prepareMusicSearch } from "./tracks";

function track(over: Partial<MusicTrack> & { id: string }): MusicTrack {
    return { kinds: ["battle"], banks: [], places: [], ...over };
}

const TRACKS: MusicTrack[] = [
    track({ id: "m_bat_indust", loopUrl: "/audio/x/m_bat_indust_loop.ogg", banks: ["battle.ON_GAME_READY.indust"] }),
    track({ id: "m_sys_void", title: "Lifeflow", kinds: ["lobby"], loopUrl: "/audio/x/m_sys_void_loop.ogg" }),
    track({ id: "m_sys_act13side", title: "The Grand Knight Territory", kinds: ["lobby"], origin: "Near Light", loopUrl: "/a.ogg" }),
    track({ id: "m_avg_escape", kinds: ["story"], places: [{ kind: "story", name: "Evil Time Part 1", items: [] }] }),
    track({ id: "m_bat_ccs1" }),
];

const collator = new Intl.Collator("en", { sensitivity: "base", numeric: true });

describe("filterTracks", () => {
    const search = prepareMusicSearch(TRACKS);

    it("orders titled before untitled and playable before unavailable on a blank query", () => {
        expect(filterTracks(search, "", "all", collator).map((t) => t.id)).toEqual(["m_sys_void", "m_sys_act13side", "m_bat_indust", "m_avg_escape", "m_bat_ccs1"]);
    });

    it("keeps only the kind asked for", () => {
        expect(filterTracks(search, "", "story", collator).map((t) => t.id)).toEqual(["m_avg_escape"]);
        expect(filterTracks(search, "", "lobby", collator)).toHaveLength(2);
    });

    it("finds a track by its event and by its place", () => {
        expect(filterTracks(search, "near light", "all", collator).map((t) => t.id)).toEqual(["m_sys_act13side"]);
        expect(filterTracks(search, "evil time", "all", collator).map((t) => t.id)).toEqual(["m_avg_escape"]);
    });

    it("finds a track by a bank or clip id substring", () => {
        expect(filterTracks(search, "indust", "all", collator)[0]?.id).toBe("m_bat_indust");
    });
});

describe("names", () => {
    it("leads with the title and falls back to the clip id", () => {
        expect(displayName({ id: "m_sys_void", title: "Lifeflow" })).toBe("Lifeflow");
        expect(displayName({ id: "m_bat_indust" })).toBe("m_bat_indust");
    });

    it("saves a clip under its own file name", () => {
        expect(clipFileName("/audio/audio/sound_beta_2/music/beta1_180603/m_bat_indust/m_bat_indust_loop.ogg")).toBe("m_bat_indust_loop.ogg");
    });
});
