import { describe, expect, it } from "vitest";
import type { IOperatorListItem } from "#/types/operators";
import { cnModuleTexts, placeholderKeys, rebuildFromBase, swapJustifiedNumbers, templateWithCnValues } from "./cnText";
import { descriptionToHtml } from "./description";

const bb = (rows: [string, number][]) => rows.map(([key, value]) => ({ key, value }));
const STRUCK = (s: string) => `<span class="rounded bg-red-500/15 px-0.5 text-destructive-foreground line-through decoration-red-400/70">${s}</span>`;
const MARKED = (s: string) => `<span class="rounded bg-emerald-500/15 px-0.5 text-success-foreground">${s}</span>`;

describe("placeholderKeys", () => {
    it("reads every placeholder, lowercased, with or without a format or sign", () => {
        expect(placeholderKeys("ATK +{atk:0%}, DEF -{-Def}, every {interval}s")).toEqual(["atk", "def", "interval"]);
    });
});

describe("templateWithCnValues", () => {
    it("fills the template with CN's values when the keys match, marking the changed number", () => {
        const html = templateWithCnValues(
            "ATK +{atk:0%} for {duration} seconds",
            bb([
                ["atk", 0.3],
                ["duration", 10],
            ]),
            bb([
                ["atk", 0.35],
                ["duration", 10],
            ]),
        );
        expect(html).toBe(`ATK ${STRUCK("+30%")}${MARKED("+35%")} for 10 seconds`);
    });

    it("fills from CN's blackboard when the template reads only some of its keys", () => {
        const html = templateWithCnValues(
            "ATK +{atk:0%}",
            bb([["atk", 0.3]]),
            bb([
                ["atk", 0.4],
                ["talent@atk", 0.4],
                ["max_stack_cnt", 3],
            ]),
        );
        expect(html).toBe(`ATK ${STRUCK("+30%")}${MARKED("+40%")}`);
    });

    it("is null when CN's blackboard lacks a key the template reads, so the caller falls back to CN's text", () => {
        expect(
            templateWithCnValues(
                "ATK +{atk:0%}, DEF +{def:0%}",
                bb([
                    ["atk", 0.3],
                    ["def", 0.2],
                ]),
                bb([
                    ["atk", 0.35],
                    ["defense", 0.25],
                ]),
            ),
        ).toBeNull();
        expect(templateWithCnValues("", [], [])).toBeNull();
    });

    it("formats percents and one-decimal values as the game does", () => {
        const html = templateWithCnValues(
            "{scale:0%} and {interval:0.0}s",
            bb([
                ["scale", 1.2],
                ["interval", 1.5],
            ]),
            bb([
                ["scale", 1.25],
                ["interval", 2],
            ]),
        );
        expect(html).toBe(`${STRUCK("120%")}${MARKED("125%")} and ${STRUCK("1.5s")}${MARKED("2.0s")}`);
    });

    it("keeps the rich-text tags around the numbers", () => {
        const html = templateWithCnValues("ATK <@ba.vup>+{atk:0%}</>, <$ba.root>Bind</>", bb([["atk", 0.3]]), bb([["atk", 0.35]]));
        expect(html).toContain(`<span style="color:var(--game-vup-ink)">+35%</span>`);
        expect(html).toContain(`<span style="color:var(--game-vup-ink)">+30%</span>`);
        expect(html).toContain(`<span style="color:var(--game-kw-ink)">Bind</span>`);
        expect(html).not.toContain("<@");
    });

    it("renders unchanged when nothing the template shows moved", () => {
        expect(
            templateWithCnValues(
                "ATK +{atk:0%}",
                bb([
                    ["atk", 0.3],
                    ["hidden", 1],
                ]),
                bb([
                    ["atk", 0.3],
                    ["hidden", 2],
                ]),
            ),
        ).toBe(descriptionToHtml("ATK +{atk:0%}", bb([["atk", 0.3]])));
    });

    it("carries over numbers written into the text when CN's text is given and a changed key accounts for them", () => {
        const html = templateWithCnValues(
            "Attacks have a 20% chance to Bind for 1.5s",
            bb([
                ["prob", 0.2],
                ["duration", 1.5],
            ]),
            bb([
                ["prob", 0.23],
                ["duration", 1.5],
            ]),
            "攻击时有23%的几率使目标束缚1.5秒",
        );
        expect(html).toBe(`Attacks have a ${STRUCK("20%")}${MARKED("23%")} chance to Bind for 1.5s`);
    });

    it("is null when a written-in number moves that no changed key accounts for", () => {
        expect(
            templateWithCnValues(
                "Attacks have a 20% chance to Bind for 1.5s",
                bb([["prob", 0.2]]),
                bb([
                    ["prob", 0.2],
                    ["duration", 2],
                ]),
                "攻击时有20%的几率使目标束缚2秒",
            ),
        ).toBeNull();
    });
});

describe("swapJustifiedNumbers", () => {
    it("reads a value above 100% the way the game writes it", () => {
        expect(swapJustifiedNumbers("DP regeneration +10%", "部署费用回复速度+13%", bb([["rate", 1.1]]), bb([["rate", 1.13]]))).toBe("DP regeneration +13%");
    });

    it("is null when the texts carry a different count of numbers", () => {
        expect(swapJustifiedNumbers("3 enemies, +12%", "三名敌人，+14%", bb([["atk", 0.12]]), bb([["atk", 0.14]]))).toBeNull();
    });
});

describe("rebuildFromBase", () => {
    it("carries CN's changed numbers into this server's text by position", () => {
        expect(rebuildFromBase("有30%概率：攻击力-20%、34%的脆弱，持续7秒", "有30%概率：攻击力-20%、30%的脆弱，持续6秒", "30% chance: ATK -20%, 30% Fragility, lasts 6s")).toEqual({
            text: "30% chance: ATK -20%, 34% Fragility, lasts 7s",
            zh: null,
        });
    });

    it("keeps what CN adds after the base as Chinese", () => {
        expect(rebuildFromBase("攻击时有23%的几率束缚1.5秒，首次命中时提升至100%", "攻击时有20%的几率束缚1.5秒", "Attacks have a 20% chance to Bind for 1.5s")).toEqual({
            text: "Attacks have a 23% chance to Bind for 1.5s",
            zh: "，首次命中时提升至100%",
        });
    });

    it("matches by value where the translation reorders the numbers or writes some in words", () => {
        expect(rebuildFromBase("三名干员时攻击力+14%且4秒获得1点技力", "三名干员时攻击力+12%且4秒获得1点技力", "With 3 Operators, ATK +12%, and 1 SP every 4 seconds")).toEqual({
            text: "With 3 Operators, ATK +14%, and 1 SP every 4 seconds",
            zh: null,
        });
    });

    it("gives up matching by value when one occurrence of a value moves and another stays", () => {
        expect(rebuildFromBase("一30，二30，三35", "一30，二30，三30", "in 1 tile, 30 then 30 then 30")).toEqual({ text: null, zh: "一30，二30，三35" });
    });

    it("gives up when CN rewrote the base's words", () => {
        expect(rebuildFromBase("在场时，每次击倒敌人时攻击力提高35", "每次击倒敌人时攻击力提高30", "ATK +30 per defeat")).toEqual({ text: null, zh: "在场时，每次击倒敌人时攻击力提高35" });
    });

    it("leaves digits inside rich-text tag names alone", () => {
        expect(rebuildFromBase("造成40%的<$ba.dt.burning2>灼燃损伤</>", "造成30%的<$ba.dt.burning2>灼燃损伤</>", "deals 30% <$ba.dt.burning2>Burn</>")).toEqual({ text: "deals 40% <$ba.dt.burning2>Burn</>", zh: null });
    });
});

describe("cnModuleTexts", () => {
    const talent = (phase: string, name: string, description: string) => ({ candidates: [{ unlockCondition: { phase, level: 1 }, requiredPotentialRank: 0, prefabKey: null, name, description, rangeId: null, blackboard: [], tokenKey: null, isHideTalent: false }] });
    const operator = (description: string, talentName: string, talentText: string, trait: string) =>
        ({
            description,
            trait: null,
            talents: [talent("PHASE_2", talentName, talentText)],
            modules: [{ uniEquipId: "shared", data: { phases: [{ parts: [{ addOrOverrideTalentDataBundle: { candidates: null }, overrideTraitDataBundle: { candidates: [{ additionalDescription: trait, overrideDescription: null, blackboard: [] }] } }] }] } }],
        }) as unknown as IOperatorListItem;
    const en = operator("Blocks {cnt} enemies", "Falling Petals", "Attacks have a 20% chance to Bind", "ASPD <@ba.vup>+{attack_speed}</> with 2+ enemies");
    const cn = operator("阻挡{cnt}个敌人", "落英", "攻击时有20%的几率束缚", "攻击范围内存在2名及以上敌人时攻击速度<@ba.vup>+{attack_speed}</>");
    const texts = cnModuleTexts(en, cn);

    it("rebuilds a talent rewrite from the talent it rewrites, names included", () => {
        expect(texts.talent({ talentIndex: 0, requiredPotentialRank: 0, name: "落英", description: "攻击时有23%的几率束缚，首次命中时提升至100%", blackboard: [] })).toEqual({
            name: { text: "Falling Petals", zh: null },
            description: { text: "Attacks have a 23% chance to Bind", zh: "，首次命中时提升至100%" },
        });
    });

    it("keeps a talent with no counterpart here Chinese", () => {
        expect(texts.talent({ talentIndex: 3, requiredPotentialRank: 0, name: "新天赋", description: "攻击力+10%", blackboard: [] })).toEqual({ name: { text: null, zh: "新天赋" }, description: { text: null, zh: "攻击力+10%" } });
    });

    it("takes a trait addition another module of the operator shares, when CN's values fill it", () => {
        const zh = "攻击范围内存在2名及以上敌人时攻击速度<@ba.vup>+{attack_speed}</>";
        expect(texts.trait(zh, "addition", bb([["attack_speed", 12]]))).toEqual({ text: "ASPD <@ba.vup>+{attack_speed}</> with 2+ enemies", zh: null });
        expect(texts.trait(zh, "addition", bb([["aspd", 12]]))).toEqual({ text: null, zh });
        expect(texts.trait("受到的伤害减少15%", "addition", [])).toEqual({ text: null, zh: "受到的伤害减少15%" });
    });

    it("takes a text another operator's shared module translates, its own pairs first", () => {
        const withShared = cnModuleTexts(en, cn, [
            { cn: "受到的伤害减少<@ba.kw>15%</>", en: "Reduces damage taken by <@ba.kw>15%</>" },
            { cn: "攻击时有{prob:0%}的几率束缚", en: "{prob:0%} chance to Bind" },
            { cn: "落英", en: "Fallen Petals" },
        ]);
        expect(withShared.trait("受到的伤害减少<@ba.kw>15%</>", "addition", [])).toEqual({ text: "Reduces damage taken by <@ba.kw>15%</>", zh: null });
        expect(withShared.talent({ talentIndex: 3, requiredPotentialRank: 0, name: "落英", description: "攻击时有{prob:0%}的几率束缚", blackboard: bb([["prob", 0.25]]) })).toEqual({
            name: { text: "Falling Petals", zh: null },
            description: { text: "{prob:0%} chance to Bind", zh: null },
        });
    });

    it("keeps a translated text Chinese when CN's blackboard cannot fill it", () => {
        const withShared = cnModuleTexts(en, cn, [{ cn: "攻击力+{atk}", en: "ATK +{atk}" }]);
        expect(withShared.trait("攻击力+{atk}", "addition", bb([["def", 10]]))).toEqual({ text: null, zh: "攻击力+{atk}" });
    });
});
