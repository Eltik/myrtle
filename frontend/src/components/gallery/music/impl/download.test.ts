import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("#/hooks/use-auth", () => ({ useAuth: () => ({ user: null }) }));
vi.mock("#/lib/api/music", () => ({
    sirenStreamUrl: (cid: string) => `https://api.test/api/music/siren/songs/${cid}/stream`,
    reportMusicStat: vi.fn(() => Promise.resolve({ counted: true })),
    recordMusicDownloadFn: vi.fn(),
}));

const api = await import("#/lib/api/music");
const { countDownload, downloadFileName, downloadTrack, downloadStore } = await import("./download");

import type { ITrack } from "./model";

function siren(over: Partial<ITrack> = {}): ITrack {
    return { id: "siren:1", kind: "siren", key: "1", title: "Song", artists: ["House"], albumId: "siren:a", albumName: "Album", coverUrl: null, instrumental: false, hasLyrics: false, trackOrder: 1, ...over };
}

describe("the saved file's name", () => {
    it("is Artist - Title.wav for a label track", () => {
        expect(downloadFileName(siren({ artists: ["House", "Guest"], title: "Song" }))).toBe("House, Guest - Song.wav");
    });

    it("drops what a file system refuses", () => {
        expect(downloadFileName(siren({ artists: ["A/B"], title: 'What? "Now"' }))).toBe("A B - What Now.wav");
    });

    it("is the loop clip's own name for an in-game track", () => {
        const game: ITrack = { ...siren(), id: "game:m_a", kind: "game", key: "m_a", artists: [], game: { id: "m_a", title: "A", kinds: [], banks: [], places: [], loopUrl: "/audio/m_a_loop.ogg" } };
        expect(downloadFileName(game)).toBe("m_a_loop.ogg");
    });
});

describe("counting a download", () => {
    beforeEach(() => {
        vi.mocked(api.reportMusicStat).mockClear();
        vi.mocked(api.recordMusicDownloadFn).mockReset();
    });

    it("signed out, the browser reports it", () => {
        countDownload("siren:1", false);
        expect(api.reportMusicStat).toHaveBeenCalledExactlyOnceWith("download", "siren:1");
        expect(api.recordMusicDownloadFn).not.toHaveBeenCalled();
    });

    it("signed in, the account counts it and the browser stays quiet", async () => {
        vi.mocked(api.recordMusicDownloadFn).mockResolvedValue({ counted: true });
        countDownload("siren:1", true);
        await vi.waitFor(() => expect(api.recordMusicDownloadFn).toHaveBeenCalledOnce());
        await Promise.resolve();
        expect(api.reportMusicStat).not.toHaveBeenCalled();
    });

    it("a session the backend no longer takes falls back to the browser", async () => {
        vi.mocked(api.recordMusicDownloadFn).mockResolvedValue(null);
        countDownload("siren:1", true);
        await vi.waitFor(() => expect(api.reportMusicStat).toHaveBeenCalledExactlyOnceWith("download", "siren:1"));
    });
});

describe("a download", () => {
    it("counts once the audio answers, and clears its progress when done", async () => {
        const bytes = new Uint8Array(1000);
        const fetchMock = vi.fn(() => Promise.resolve(new Response(bytes, { status: 200, headers: { "content-length": "1000" } })));
        vi.stubGlobal("fetch", fetchMock);
        const createObjectURL = vi.fn(() => "blob:x");
        vi.stubGlobal("URL", Object.assign(URL, { createObjectURL, revokeObjectURL: vi.fn() }));
        const started = vi.fn();
        const outcome = await downloadTrack(siren(), started);
        expect(outcome).toBe("handed");
        expect(started).toHaveBeenCalledOnce();
        expect(fetchMock).toHaveBeenCalledWith("https://api.test/api/music/siren/songs/1/stream");
        expect(downloadStore.state).toEqual({});
        vi.unstubAllGlobals();
    });

    it("a failed answer counts nothing", async () => {
        vi.stubGlobal(
            "fetch",
            vi.fn(() => Promise.resolve(new Response(null, { status: 404 }))),
        );
        const started = vi.fn();
        expect(await downloadTrack(siren(), started)).toBe("failed");
        expect(started).not.toHaveBeenCalled();
        vi.unstubAllGlobals();
    });

    it("a failed answer after the picker takes the empty file off again", async () => {
        vi.stubGlobal(
            "fetch",
            vi.fn(() => Promise.resolve(new Response(null, { status: 503 }))),
        );
        const createWritable = vi.fn();
        const remove = vi.fn(() => Promise.resolve());
        vi.stubGlobal(
            "showSaveFilePicker",
            vi.fn(() => Promise.resolve({ createWritable, remove })),
        );
        expect(await downloadTrack(siren(), vi.fn())).toBe("failed");
        expect(createWritable).not.toHaveBeenCalled();
        expect(remove).toHaveBeenCalledOnce();
        vi.unstubAllGlobals();
    });
});
