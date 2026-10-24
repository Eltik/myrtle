import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { CONDITIONAL_NAME_KEYS, CONDITIONAL_OVERRIDE_KEYS, conditionalFlagFor, conditionalLabelKey, conditionalSwitchOn, humaniseConditionalName } from "./conditionals";
import { messages } from "./conditionals.messages";

interface IFormulaEntry {
    conditionals: { type: string; name: string }[];
}

/** The engine's own configs: the names `GET /{dps,hps}/operators` serves come from here. */
function engineConditionals(): { charId: string; type: string; name: string }[] {
    const out: { charId: string; type: string; name: string }[] = [];
    for (const file of ["operator_formulas.json", "heal_formulas.json"]) {
        // Joined by hand: Vite rewrites `new URL(literal, import.meta.url)` into an asset import.
        const path = join(dirname(fileURLToPath(import.meta.url)), "../../../../../backend/src/dps/config", file);
        const config = JSON.parse(readFileSync(path, "utf8")) as Record<string, IFormulaEntry>;
        for (const [charId, entry] of Object.entries(config)) for (const c of entry.conditionals) out.push({ charId, type: c.type, name: c.name });
    }
    return out;
}

describe("conditional labels", () => {
    const all = engineConditionals();

    it("has a label for every conditional name in the engine configs", () => {
        const unlabelled = [...new Set(all.map((c) => c.name))].filter((n) => !Object.hasOwn(CONDITIONAL_NAME_KEYS, n));
        expect(unlabelled).toEqual([]);
    });

    it("has no override for an operator conditional the engine no longer serves", () => {
        const present = new Set(all.map((c) => `${c.charId}|${c.type}|${c.name}`));
        expect(Object.keys(CONDITIONAL_OVERRIDE_KEYS).filter((k) => !present.has(k))).toEqual([]);
    });

    it("points every table entry at a declared message", () => {
        for (const key of [...Object.values(CONDITIONAL_NAME_KEYS), ...Object.values(CONDITIONAL_OVERRIDE_KEYS)]) expect(Object.hasOwn(messages, key)).toBe(true);
    });

    it("prefers the per-operator override for a truncated name", () => {
        expect(conditionalLabelKey("char_250_phatom", { conditionalType: "talent", name: "with" })).toBe("calc.cond.withClone");
        expect(conditionalLabelKey("char_x", { conditionalType: "talent", name: "with" })).toBe(CONDITIONAL_NAME_KEYS.with);
        expect(conditionalLabelKey("char_x", { conditionalType: "talent", name: "constructor" })).toBeNull();
    });
});

describe("humaniseConditionalName", () => {
    it.each([
        ["vsBlockedTarget", "vs. blocked target"],
        ["noTalent2Stacks", "No talent 2 stacks"],
        ["maxRange", "Max range"],
        ["3inRange", "3 in range"],
        ["noDP", "No DP"],
        ["inIS3", "In IS3"],
        ["NotBlocking", "Not blocking"],
        ["", ""],
    ])("%s -> %s", (raw, label) => {
        expect(humaniseConditionalName(raw)).toBe(label);
    });
});

describe("conditional switch polarity", () => {
    const on = { inverted: false };
    const inverted = { inverted: true };

    it("shows an inverted conditional on when its flag is off", () => {
        expect(conditionalSwitchOn(on, true)).toBe(true);
        expect(conditionalSwitchOn(on, false)).toBe(false);
        expect(conditionalSwitchOn(inverted, true)).toBe(false);
        expect(conditionalSwitchOn(inverted, false)).toBe(true);
    });

    it("reads an unset flag as on, the way the engine does", () => {
        expect(conditionalSwitchOn(on, undefined)).toBe(true);
        expect(conditionalSwitchOn(inverted, undefined)).toBe(false);
    });

    it("round-trips switch -> flag -> switch for both polarities", () => {
        for (const cond of [on, inverted]) for (const s of [true, false]) expect(conditionalSwitchOn(cond, conditionalFlagFor(cond, s))).toBe(s);
    });
});
