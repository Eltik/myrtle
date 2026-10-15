import { describe, expect, it } from "vitest";
import { formatProfession, getAvatarById, getSecretaryAvatarURL, professionLabel, serverNames, toAvatarStem } from "./utils";

describe("professionLabel", () => {
    it("shows the server's own class name verbatim, suffix and all", () => {
        expect(professionLabel({ profession: "WARRIOR", professionName: "近卫干员" })).toBe("近卫干员");
        expect(professionLabel({ profession: "WARRIOR", professionName: "前衛タイプ" })).toBe("前衛タイプ");
    });

    it("falls back to the English formatter without a server name", () => {
        expect(professionLabel({ profession: "PIONEER" })).toBe("Vanguard");
        expect(professionLabel({ profession: "TANK", professionName: null })).toBe("Defender");
        expect(professionLabel({ profession: "SUPPORT", professionName: "" })).toBe("Supporter");
    });

    it("leaves EN unchanged: the EN tag names are the formatter's words", () => {
        const en: Record<string, string> = { WARRIOR: "Guard", SNIPER: "Sniper", TANK: "Defender", MEDIC: "Medic", SUPPORT: "Supporter", CASTER: "Caster", SPECIAL: "Specialist", PIONEER: "Vanguard" };
        for (const [profession, professionName] of Object.entries(en)) {
            expect(professionLabel({ profession, professionName })).toBe(formatProfession(profession));
        }
    });
});

describe("serverNames", () => {
    it("keys class names by the bare profession code", () => {
        const names = serverNames(
            [
                { profession: "WARRIOR", professionName: "가드" },
                { profession: "MEDIC", professionName: null },
            ],
            "profession",
        );
        expect([...names]).toEqual([["WARRIOR", "가드"]]);
    });
});

describe("toAvatarStem", () => {
    it("maps base-art ids to the avatar file stems the backend serves", () => {
        expect(toAvatarStem("char_002_amiya")).toBe("char_002_amiya");
        expect(toAvatarStem("char_002_amiya#1")).toBe("char_002_amiya");
        expect(toAvatarStem("char_002_amiya#2")).toBe("char_002_amiya_2");
        expect(toAvatarStem("char_002_amiya#1+")).toBe("char_002_amiya_1%2B");
    });

    it("keeps a skin's # and encodes it", () => {
        expect(toAvatarStem("char_002_amiya@winter#1")).toBe("char_002_amiya_winter%231");
        expect(toAvatarStem("char_4193_lemuen@ambienceSynesthesia#7")).toBe("char_4193_lemuen_ambienceSynesthesia%237");
    });

    it("builds the secretary and id avatar URLs from the same stem", () => {
        expect(getSecretaryAvatarURL({ secretary: "char_002_amiya", secretary_skin_id: "char_002_amiya#1+" })).toMatch(/\/api\/avatar\/char_002_amiya_1%2B$/);
        expect(getAvatarById("char_002_amiya#2", "cn")).toMatch(/\/api\/cn\/avatar\/char_002_amiya_2$/);
    });
});
