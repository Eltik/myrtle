import { describe, expect, it } from "vitest";
import { formatProfession, professionLabel, serverNames } from "./utils";

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
