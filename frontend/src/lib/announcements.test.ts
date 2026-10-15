import { describe, expect, it } from "vitest";
import { validateAnnouncementsSearch } from "#/components/announcements/search";
import type { AnnouncementSummary } from "#/types/generated/AnnouncementSummary";
import { boardDate, filterAnnouncements, groupsOf, mergeSeen, preferredServer, unreadOf } from "./announcements";

/** The EN board as the backend serves it on 2026-10-11 (ids, titles, groups, dates from the real meta). */
const EN: AnnouncementSummary[] = [
    ["1608", "Displayed Operators Rate Up !!!", "ACTIVITY", "2026-10-09"],
    ["1629", "Critical Phase Transition Open", "ACTIVITY", "2026-10-08"],
    ["1627", "Recommended Monthly Card Available", "ACTIVITY", "2026-10-07"],
    ["1599", "Kernel Headhunting", "ACTIVITY", "2026-10-06"],
    ["1624", "Ambience Synesthesia New Arrivals", "ACTIVITY", "2026-09-23"],
    ["1107", "Official Statement on Reclamation Algorithm", "SYSTEM", "2026-08-07"],
    ["1090", "Official Statement on Fair Play", "SYSTEM", "2026-07-31"],
].map(([id, title, group, published_on]) => ({ id, title, group, published_on, banner_url: null, notice_url: `https://example.invalid/${id}.html` }));

describe("groupsOf", () => {
    it("lists the board's own groups in order of first appearance, with counts", () => {
        expect(groupsOf(EN)).toEqual([
            { group: "ACTIVITY", count: 5 },
            { group: "SYSTEM", count: 2 },
        ]);
    });
});

describe("filterAnnouncements", () => {
    it("keeps board order with no query", () => {
        expect(filterAnnouncements(EN, null, "").map((a) => a.id)).toEqual(EN.map((a) => a.id));
    });
    it("filters by group", () => {
        expect(filterAnnouncements(EN, "SYSTEM", "").map((a) => a.id)).toEqual(["1107", "1090"]);
    });
    it("fuzzy-matches titles inside the group", () => {
        expect(filterAnnouncements(EN, null, "kernel").map((a) => a.id)).toEqual(["1599"]);
        expect(
            filterAnnouncements(EN, "SYSTEM", "statement")
                .map((a) => a.id)
                .sort(),
        ).toEqual(["1090", "1107"]);
        expect(filterAnnouncements(EN, "SYSTEM", "kernel")).toEqual([]);
    });
});

describe("seen state", () => {
    it("is empty until storage has been read", () => {
        expect(unreadOf(EN, "en", { status: "pending", seen: {} })).toEqual([]);
        expect(unreadOf(EN, "en", { status: "unavailable", seen: {} })).toEqual([]);
    });
    it("counts every notice unread on a first visit, and none once marked", () => {
        expect(unreadOf(EN, "en", { status: "ready", seen: {} })).toHaveLength(7);
        const seen = mergeSeen({}, "en", ["1608", "1599"]);
        expect(unreadOf(EN, "en", { status: "ready", seen }).map((a) => a.id)).toEqual(["1629", "1627", "1624", "1107", "1090"]);
        expect(unreadOf(EN, "jp", { status: "ready", seen })).toHaveLength(7);
    });
    it("dedupes, returns the same object when nothing changes, and caps at 200 ids", () => {
        const once = mergeSeen({}, "en", ["1", "2"]);
        expect(mergeSeen(once, "en", ["2", "1"])).toBe(once);
        const many = mergeSeen(
            {},
            "en",
            Array.from({ length: 250 }, (_, i) => String(i)),
        );
        expect(many.en).toHaveLength(200);
        expect(many.en?.[0]).toBe("50");
    });
});

describe("helpers", () => {
    it("picks the first known server", () => {
        expect(preferredServer(undefined, null, "", "jp", "en")).toBe("jp");
        expect(preferredServer(undefined, null)).toBeUndefined();
    });
    it("reads a board date as a local calendar day", () => {
        const d = boardDate("2026-10-09");
        expect([d?.getFullYear(), d?.getMonth(), d?.getDate()]).toEqual([2026, 9, 9]);
        expect(boardDate(null)).toBeNull();
        expect(boardDate("garbage")).toBeNull();
    });
    it("validates the page's search params", () => {
        expect(validateAnnouncementsSearch({ server: "jp", id: " 1427 " })).toEqual({ server: "jp", id: "1427" });
        expect(validateAnnouncementsSearch({ id: 1608 })).toEqual({ id: "1608" });
        expect(validateAnnouncementsSearch({ server: "<script>", id: "" })).toEqual({});
    });
});
