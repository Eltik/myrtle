import { describe, expect, it } from "vitest";
import type { IVoice } from "#/types/voices";
import { countWords, fileSections, type IModuleSource, moduleSections, operatorTabs, operatorVoiceLines, voiceLanguages } from "./operatorText";

function voice(over: Partial<IVoice> & Pick<IVoice, "charWordId">): IVoice {
    return {
        wordKey: over.charWordId,
        charId: "char_102_texas",
        voiceId: over.charWordId,
        voiceText: "",
        voiceTitle: over.charWordId,
        voiceIndex: 0,
        voiceType: "NONE",
        unlockType: "DIRECT",
        unlockParam: [],
        lockDescription: null,
        placeType: "HOME_SHOW",
        voiceAsset: "",
        id: over.charWordId,
        data: null,
        languages: null,
        ...over,
    } as IVoice;
}

function moduleOf(over: Partial<IModuleSource> & Pick<IModuleSource, "uniEquipId">): IModuleSource {
    return {
        uniEquipName: over.uniEquipId,
        uniEquipDesc: "A module.",
        uniEquipIcon: "sol_y",
        image: null,
        typeName1: "SOL",
        typeName2: "Y",
        type: "ADVANCED",
        charEquipOrder: 0,
        ...over,
    };
}

describe("countWords", () => {
    it("counts whitespace-separated tokens of the tag-free text", () => {
        expect(countWords("I don't say much? That's because I'd rather avoid thinking.")).toBe(10);
        expect(countWords("line one\nline two")).toBe(4);
        expect(countWords("   spaced   out   ")).toBe(2);
    });

    it("reads a rich-text span as the words it prints, not as its markup", () => {
        expect(countWords("A <@ba.vup>clear</> plan")).toBe(3);
        expect(countWords("<i>Silence.</i>")).toBe(1);
    });

    it("is zero for nothing at all", () => {
        expect(countWords("")).toBe(0);
        expect(countWords("   ")).toBe(0);
        expect(countWords(null)).toBe(0);
        expect(countWords(undefined)).toBe(0);
    });
});

describe("fileSections", () => {
    it("joins a section's stories into one body and counts its words", () => {
        const sections = fileSections([{ storyTitle: "Basic Info", stories: [{ storyText: "one two" }, { storyText: "three" }] }]);
        expect(sections).toEqual([{ key: "Basic Info", title: "Basic Info", text: "one two\n\nthree", words: 3 }]);
    });

    it("drops a section with no text and keeps the handbook's order", () => {
        const sections = fileSections([
            { storyTitle: "Empty", stories: [{ storyText: "  " }] },
            { storyTitle: "Profile", stories: [{ storyText: "words here" }] },
        ]);
        expect(sections.map((s) => s.title)).toEqual(["Profile"]);
    });

    it("keys a repeated title apart so two sections never collide", () => {
        const sections = fileSections([
            { storyTitle: "Archive File 1", stories: [{ storyText: "a" }] },
            { storyTitle: "Archive File 1", stories: [{ storyText: "b" }] },
        ]);
        expect(sections.map((s) => s.key)).toEqual(["Archive File 1", "Archive File 1#1"]);
    });

    it("is empty when the backend sends no handbook", () => {
        expect(fileSections(null)).toEqual([]);
        expect(fileSections(undefined)).toEqual([]);
    });
});

describe("moduleSections", () => {
    it("keeps the original kit beside the unlockable modules, in equip order", () => {
        const rows = moduleSections([moduleOf({ uniEquipId: "uniequip_002_texas", charEquipOrder: 1 }), moduleOf({ uniEquipId: "uniequip_001_texas", charEquipOrder: 0, type: "INITIAL", typeName1: "ORIGINAL", typeName2: null, uniEquipDesc: "Original gear." })]);
        expect(rows.map((r) => [r.key, r.designator, r.original])).toEqual([
            ["uniequip_001_texas", "ORIGINAL", true],
            ["uniequip_002_texas", "SOL-Y", false],
        ]);
    });

    it("counts each module's description and drops one with no text", () => {
        const rows = moduleSections([moduleOf({ uniEquipId: "a", uniEquipDesc: "four words go here" }), moduleOf({ uniEquipId: "b", uniEquipDesc: "" })]);
        expect(rows.map((r) => [r.key, r.words])).toEqual([["a", 4]]);
    });

    it("falls back to the icon sprite when the module ships no image", () => {
        expect(moduleSections([moduleOf({ uniEquipId: "a", uniEquipIcon: "sol_y" })])[0].iconPath).toBe("/textures/spritepack/ui_equip_big_img_hub_0/sol_y.png");
        expect(moduleSections([moduleOf({ uniEquipId: "a", image: "/textures/x.png" })])[0].iconPath).toBe("/textures/x.png");
    });
});

describe("operatorVoiceLines", () => {
    const words = { a: voice({ charWordId: "char_002_amiya_CN_002", voiceIndex: 2, voiceText: "two words", id: "a" }), b: voice({ charWordId: "char_1001_amiya2_CN_001", voiceIndex: 1, voiceText: "guard line here", id: "b" }), missing: undefined };

    it("filters on the line key, so a form gets its own lines and not the base form's", () => {
        expect(operatorVoiceLines(words, "char_1001_amiya2").map((l) => l.key)).toEqual(["b"]);
        expect(operatorVoiceLines(words, "char_002_amiya").map((l) => l.key)).toEqual(["a"]);
    });

    it("orders by the game's voice index and counts each line's words", () => {
        const lines = operatorVoiceLines({ ...words, c: voice({ charWordId: "char_002_amiya_CN_001", voiceIndex: 1, voiceText: "one", id: "c" }) }, "char_002_amiya");
        expect(lines.map((l) => [l.key, l.words])).toEqual([
            ["c", 1],
            ["a", 2],
        ]);
    });

    it("indexes each line's clip by language", () => {
        const withClips = {
            a: voice({
                charWordId: "char_102_texas_CN_002",
                id: "a",
                data: [
                    { voiceUrl: "/jp.ogg", language: "JP", cvName: null },
                    { voiceUrl: null, language: "EN", cvName: null },
                ],
            }),
        };
        expect(operatorVoiceLines(withClips, "char_102_texas")[0].clips).toEqual({ JP: "/jp.ogg" });
    });

    it("drops a second voice pack's row only when its text and every clip match", () => {
        const twin = {
            a: voice({ charWordId: "char_102_texas_CN_002", id: "a", voiceTitle: "Talk 1", voiceText: "same line", data: [{ voiceUrl: "/jp.ogg", language: "JP", cvName: null }] }),
            b: voice({ charWordId: "char_102_texas_ITA_CN_002", id: "b", voiceTitle: "Talk 1", voiceText: "same line", data: [{ voiceUrl: "/jp.ogg", language: "JP", cvName: null }] }),
            c: voice({ charWordId: "char_102_texas_ITA_CN_003", id: "c", voiceTitle: "Talk 1", voiceText: "same line", data: [{ voiceUrl: "/ita.ogg", language: "ITA", cvName: null }] }),
        };
        expect(operatorVoiceLines(twin, "char_102_texas").map((l) => l.key)).toEqual(["a", "c"]);
    });

    it("is empty without a payload or without an id", () => {
        expect(operatorVoiceLines(null, "char_102_texas")).toEqual([]);
        expect(operatorVoiceLines(words, "")).toEqual([]);
    });
});

describe("voiceLanguages", () => {
    const lines = [
        { key: "a", title: "a", text: "", words: 0, clips: { JP: "/jp.ogg", EN: "/en.ogg" } },
        { key: "b", title: "b", text: "", words: 0, clips: { ITA: "/ita.ogg" } },
    ] as const;

    it("keeps the display order and appends a language that order does not name", () => {
        expect(voiceLanguages(lines, ["EN", "JP"])).toEqual(["EN", "JP", "ITA"]);
    });

    it("names only the languages a clip exists for", () => {
        expect(voiceLanguages([lines[0]], ["EN", "JP", "KR"])).toEqual(["EN", "JP"]);
    });
});

describe("operatorTabs", () => {
    it("says null, not zero, for a shelf whose query has not run", () => {
        expect(operatorTabs({ records: 3, files: null, modules: null, voices: null })).toEqual([
            { key: "records", count: 3, words: null },
            { key: "files", count: null, words: null },
            { key: "modules", count: null, words: null },
            { key: "voices", count: null, words: null },
        ]);
    });

    it("counts rows and sums words once a shelf has loaded", () => {
        const tabs = operatorTabs({
            records: 3,
            files: [
                { key: "a", title: "a", text: "", words: 208 },
                { key: "b", title: "b", text: "", words: 179 },
            ],
            modules: [{ key: "m", name: "m", designator: "SOL-Y", description: "", words: 436, iconPath: "", original: false }],
            voices: [{ key: "v", title: "v", text: "", words: 724, clips: {} }],
        });
        expect(tabs.map((tab) => [tab.key, tab.count, tab.words])).toEqual([
            ["records", 3, null],
            ["files", 2, 387],
            ["modules", 1, 436],
            ["voices", 1, 724],
        ]);
    });

    it("counts an empty shelf as zero once it HAS loaded", () => {
        expect(operatorTabs({ records: 0, files: [], modules: [], voices: [] }).map((tab) => tab.count)).toEqual([0, 0, 0, 0]);
    });
});
