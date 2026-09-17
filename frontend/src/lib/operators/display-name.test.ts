import { describe, expect, it } from "vitest";
import { hasCjk, operatorDisplayName } from "./display-name";

describe("operatorDisplayName", () => {
    const aglna2 = { name: "予愿安洁莉娜", appellation: "Angelina the Mellow Wish" };
    const reed2 = { name: "Reed the Flame Shadow", appellation: "Reed The Flame Shadow" };
    const zima = { name: "Zima", appellation: "Зима" };
    const turdus = { name: "乌啾", appellation: "Укусик" };

    it("swaps a Han name for its appellation when the preference is on", () => {
        expect(operatorDisplayName(aglna2, true)).toBe("Angelina the Mellow Wish");
    });

    it("is inert for EN operators even where the appellation differs", () => {
        expect(operatorDisplayName(reed2, true)).toBe("Reed the Flame Shadow");
        expect(operatorDisplayName(zima, true)).toBe("Zima");
    });

    it("returns the stored name when the preference is off", () => {
        expect(operatorDisplayName(aglna2, false)).toBe("予愿安洁莉娜");
    });

    it("keeps a Cyrillic appellation: that is the game's own Latin column", () => {
        expect(operatorDisplayName(turdus, true)).toBe("Укусик");
    });

    it("falls back to the name when the appellation is blank or missing", () => {
        expect(operatorDisplayName({ name: "预备干员-近战", appellation: " " }, true)).toBe("预备干员-近战");
        expect(operatorDisplayName({ name: "预备干员-近战" }, true)).toBe("预备干员-近战");
    });
});

describe("hasCjk", () => {
    it("detects Han, kana and hangul but not Cyrillic or Latin", () => {
        expect(hasCjk("凯尔希·思衡托")).toBe(true);
        expect(hasCjk("ケルシー")).toBe(true);
        expect(hasCjk("켈시")).toBe(true);
        expect(hasCjk("Зима")).toBe(false);
        expect(hasCjk("Kal'tsit")).toBe(false);
    });
});
