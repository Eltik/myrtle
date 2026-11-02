import { describe, expect, it } from "vitest";
import type { ISkin } from "#/lib/api/skins";
import { buildOperatorSkinList } from "./skins";

const t = ((key: string) => key) as unknown as Parameters<typeof buildOperatorSkinList>[0]["t"];

function skin(skinId: string, skinName: string): ISkin {
    return { skinId, charId: "char_4039_horn", displaySkin: { skinName } } as unknown as ISkin;
}

function build(skinsFromBackend: ISkin[], cnSkins?: ISkin[]) {
    return buildOperatorSkinList({ skinsFromBackend, operatorId: "char_4039_horn", operatorSkin: null, operatorPortrait: null, phasesLength: 3, server: "en", cnSkins, t });
}

describe("buildOperatorSkinList with CN's skins", () => {
    const en = [skin("char_4039_horn#1", "default"), skin("char_4039_horn@cfa#1", "Spotlight")];

    it("appends only the outfits this server lacks, after its own, with CN's art", () => {
        const list = build(en, [...en, skin("char_4039_horn@epoque#59", "锋光初现")]);
        expect(list.map((s) => s.id)).toEqual(["char_4039_horn_default", "char_4039_horn_e2", "char_4039_horn@cfa#1", "char_4039_horn@epoque#59"]);
        const cnOnly = list[3];
        expect(cnOnly.fromCn).toBe(true);
        expect(cnOnly.name).toBe("锋光初现");
        expect(cnOnly.image).toContain("cn/");
        expect(list.filter((s) => s.fromCn)).toHaveLength(1);
    });

    it("leaves the list as it was without CN's skins", () => {
        expect(build(en).map((s) => s.id)).toEqual(build(en, []).map((s) => s.id));
        expect(build(en).some((s) => s.fromCn)).toBe(false);
    });
});
