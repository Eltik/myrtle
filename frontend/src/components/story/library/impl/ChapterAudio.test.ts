/**
 * WHO RE-RENDERS WHEN THE PLAYHEAD MOVES. The now-playing bar's clock writes
 * `positionSeconds` into the store once a frame while a cue sounds. The chapter
 * sheet reads one key or null from the channel and the bar reads everything
 * but the playhead, so a frame of the clock must change neither snapshot; only
 * the seek slider reads the playhead itself.
 */
import { describe, expect, it } from "vitest";
import { playingKeyOf } from "./ChapterAudio";
import { barSnapshot, type IPlayerState } from "./player";

const idle: IPlayerState = { track: null, paused: false, volume: 0.5, muted: false, positionSeconds: 0, introSeconds: 0, lengthSeconds: 0 };

function sounding(key: string, over: Partial<IPlayerState> = {}): IPlayerState {
    return { ...idle, track: { key, groupId: key.split("::")[0] ?? "", groupName: "", title: null, cue: { loop: "loop.wav" } }, ...over };
}

describe("playingKeyOf", () => {
    it("is null on an idle channel", () => {
        expect(playingKeyOf(idle, "act31side::")).toBeNull();
    });

    it("is the LOCAL key of this group's sounding cue", () => {
        expect(playingKeyOf(sounding("act31side::theme"), "act31side::")).toBe("theme");
        expect(playingKeyOf(sounding("act31side::track:bgm_1"), "act31side::")).toBe("track:bgm_1");
    });

    it("is null while this group's cue is paused, and null for another group's cue", () => {
        expect(playingKeyOf(sounding("act31side::theme", { paused: true }), "act31side::")).toBeNull();
        expect(playingKeyOf(sounding("main_8::theme"), "act31side::")).toBeNull();
    });

    it("does not move with the playhead, which is the whole reason the sheet subscribes to it", () => {
        const a = playingKeyOf(sounding("act31side::theme", { positionSeconds: 1.25, lengthSeconds: 90 }), "act31side::");
        const b = playingKeyOf(sounding("act31side::theme", { positionSeconds: 1.266, lengthSeconds: 90 }), "act31side::");
        expect(a).toBe(b);
    });
});

describe("barSnapshot", () => {
    it("keeps its identity while only the playhead moves", () => {
        // The store spreads each frame's playhead over the SAME track object.
        const base = sounding("act31side::theme", { positionSeconds: 1, lengthSeconds: 90 });
        const first = barSnapshot(null, base);
        expect(barSnapshot(first, { ...base, positionSeconds: 1.016 })).toBe(first);
        expect("positionSeconds" in first).toBe(false);
    });

    it("is a new object when anything the bar draws changes", () => {
        const base = sounding("act31side::theme", { lengthSeconds: 90 });
        const first = barSnapshot(null, base);
        for (const change of [{ paused: true }, { volume: 0.2 }, { muted: true }, { introSeconds: 4 }, { lengthSeconds: 91 }, { track: null }] as Partial<IPlayerState>[]) {
            expect(barSnapshot(first, { ...base, ...change })).not.toBe(first);
        }
    });
});
