import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("#/hooks/use-auth", () => ({ useAuth: () => ({ user: null }) }));
vi.mock("#/lib/api/music", () => ({
    musicLibraryQueryOptions: () => ({ queryKey: ["music", "library", "anon"], queryFn: () => Promise.resolve(null), enabled: false }),
    getMusicLibraryFn: vi.fn(),
    likeTrackFn: vi.fn(),
    logPlayFn: vi.fn(),
    updatePlayFn: vi.fn(),
    createPlaylistFn: vi.fn(),
    patchPlaylistFn: vi.fn(),
    deletePlaylistFn: vi.fn(),
    addPlaylistTracksFn: vi.fn(),
    removePlaylistTrackFn: vi.fn(),
}));

const store = await import("./store");

function doc(): { likes: Record<string, string>; history: unknown[]; playlists: { id: string; name: string; trackIds: string[] }[] } {
    return JSON.parse(window.localStorage.getItem(store.LOCAL_KEY) ?? "{}");
}

describe("the signed-out library", () => {
    beforeEach(() => {
        window.localStorage.clear();
        store.libraryStore.setState(() => ({ ...store.libraryStore.state, mode: "local", viewerId: null, loaded: true, likes: {}, history: [], totals: [], firstPlayedAt: null, playlists: [] }));
    });

    it("likes are toggled and written to the browser", () => {
        expect(store.toggleLike("siren:1")).toBe(true);
        expect(store.isLiked("siren:1")).toBe(true);
        expect(Object.keys(doc().likes)).toEqual(["siren:1"]);
        expect(store.toggleLike("siren:1")).toBe(false);
        expect(doc().likes).toEqual({});
    });

    it("a play is logged once and its time only grows", async () => {
        const id = await store.logPlay("siren:1", 10);
        expect(id).toBeLessThan(0);
        store.updatePlay(id, 30);
        store.updatePlay(id, 20);
        const s = store.libraryStore.state;
        expect(s.history).toHaveLength(1);
        expect(s.history[0]?.listenedSeconds).toBe(30);
        expect(s.totals).toEqual([{ trackId: "siren:1", plays: 1, seconds: 30, lastPlayedAt: s.history[0]?.playedAt }]);
        expect(s.firstPlayedAt).toBe(s.history[0]?.playedAt);
        expect(doc().history).toHaveLength(1);
    });

    it("totals rank by plays then seconds", () => {
        const plays = [
            { id: 1, trackId: "a", listenedSeconds: 5, playedAt: "2026-01-01T00:00:00Z" },
            { id: 2, trackId: "b", listenedSeconds: 50, playedAt: "2026-01-02T00:00:00Z" },
            { id: 3, trackId: "a", listenedSeconds: 5, playedAt: "2026-01-03T00:00:00Z" },
        ];
        const totals = store.totalsOf(plays);
        expect(totals.map((t) => t.trackId)).toEqual(["a", "b"]);
        expect(totals[0]).toMatchObject({ plays: 2, seconds: 10, lastPlayedAt: "2026-01-03T00:00:00Z" });
    });

    it("playlists are created, filled, reordered, trimmed and deleted", async () => {
        const pl = await store.createPlaylist("  Late night ", ["siren:1", "siren:1", "siren:2"]);
        expect(pl?.name).toBe("Late night");
        expect(pl?.trackIds).toEqual(["siren:1", "siren:2"]);
        const id = pl?.id ?? "";
        expect(store.addToPlaylist(id, ["siren:2", "siren:3"])).toBe(1);
        store.setPlaylistTracks(id, ["siren:3", "siren:1"]);
        store.renamePlaylist(id, "Later");
        let got = store.libraryStore.state.playlists[0];
        expect(got?.trackIds).toEqual(["siren:3", "siren:1"]);
        expect(got?.name).toBe("Later");
        store.removeFromPlaylist(id, "siren:3");
        got = store.libraryStore.state.playlists[0];
        expect(got?.trackIds).toEqual(["siren:1"]);
        expect(doc().playlists[0]?.trackIds).toEqual(["siren:1"]);
        store.deletePlaylist(id);
        expect(store.libraryStore.state.playlists).toEqual([]);
        expect(doc().playlists).toEqual([]);
    });

    it("a blank name makes no playlist", async () => {
        expect(await store.createPlaylist("   ")).toBeNull();
    });
});
