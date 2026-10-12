import { describe, expect, it } from "vitest";
import type { ArchiveOperator } from "#/types/generated/ArchiveOperator";
import { archiveRows, buildArchiveOgData, clampExcerpt, firstProseRecord } from "./archive";

const FILE = {
    id: "char_4227_gallus",
    name: "GALLUS²",
    rarity: 1,
    number: { display: "RCX7" },
    factions: [{ id: "rhodes", name: "罗德岛" }],
    profile: {
        infectionState: "unstated",
        exam: [],
        manufacturer: "另一只鸡",
        extra: [
            { label: "编号", value: "GALLUS²" },
            { label: "型号", value: "自称原鸡" },
        ],
    },
} as unknown as ArchiveOperator;

describe("archive card", () => {
    it("lists only fields the file has, never one that repeats the name", () => {
        expect(archiveRows(FILE)).toEqual([
            { label: "Manufacturer", value: "另一只鸡" },
            { label: "型号", value: "自称原鸡" },
        ]);
        const data = buildArchiveOgData(FILE, "cn");
        expect(data.number).toBe("RCX7");
        expect(data.faction).toBe("罗德岛");
    });

    it("takes the first prose section and clamps it", () => {
        const sections = [{ stories: [{ storyText: "[Code Name] Exusiai\n[Gender] Female" }] }, { stories: [{ storyText: "Exusiai is a <@cc.kw>citizen</> of Laterano." }] }];
        expect(firstProseRecord(sections)).toBe("Exusiai is a citizen of Laterano.");
        expect(firstProseRecord([{ stories: [{ storyText: "【代号】佩德洛" }] }])).toBeNull();
        expect(clampExcerpt("alpha beta gamma delta epsilon", 20)).toBe("alpha beta gamma...");
        expect(clampExcerpt("short", 20)).toBe("short");
    });
});
