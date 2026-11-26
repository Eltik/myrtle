import { tagTokenRegex } from "#/lib/gamedata/richtext";
import type { CnTranslation } from "#/types/generated/CnTranslation";
import type { IOperatorListItem, ITalent, ITalentCandidate } from "#/types/operators";
import { renderDescriptionDiffHtml } from "./description";
import { getActiveTalentCandidate } from "./helpers";

type Blackboard = readonly { key: string; value: number }[];

/** A number written into the text, or anything else: one character, a rich-text tag, or a `{key}` placeholder. */
type Token = { kind: "num"; text: string; value: number } | { kind: "lit"; text: string };

const placeholderRegex = /-?\{-?([^}:]+)(?::[^}]+)?\}/g;

function tokenize(text: string): Token[] {
    const re = new RegExp(`${tagTokenRegex().source}|-?\\{[^}]*\\}|\\d+(?:\\.\\d+)?|[\\s\\S]`, "g");
    return Array.from(text.matchAll(re), ([t]) => (/^\d/.test(t) ? { kind: "num", text: t, value: Number.parseFloat(t) } : { kind: "lit", text: t }));
}

function numbers(tokens: readonly Token[]): Extract<Token, { kind: "num" }>[] {
    return tokens.filter((t) => t.kind === "num");
}

function near(a: number, b: number): boolean {
    return Math.abs(a - b) <= 1e-3 * Math.max(1, Math.abs(a), Math.abs(b));
}

/** The blackboard keys a description's `{key}` placeholders read, lowercased. */
export function placeholderKeys(text: string): string[] {
    return Array.from(text.matchAll(placeholderRegex), (m) => m[1].toLowerCase());
}

function hasKeys(text: string, blackboard: Blackboard): boolean {
    const keys = new Set(blackboard.map((b) => b.key.toLowerCase()));
    return placeholderKeys(text).every((k) => keys.has(k));
}

/** The ways the game writes a blackboard value into text: as is, as a percent, and as a percent above 100%. */
const DISPLAY_FORMS: ((v: number) => number)[] = [(v) => v, (v) => v * 100, (v) => (v - 1) * 100];

/**
 * `enText` with the numbers written into it (not its placeholders) set to `cnText`'s, by position,
 * where every number that moves is a value of a blackboard key that moved between `enBlackboard`
 * and `cnBlackboard`. Null when the two texts carry a different count of numbers or a number moves
 * that no changed key accounts for: the texts no longer line up, and a swap would guess.
 */
export function swapJustifiedNumbers(enText: string, cnText: string, enBlackboard: Blackboard, cnBlackboard: Blackboard): string | null {
    const tokens = tokenize(enText);
    const en = numbers(tokens);
    const cn = numbers(tokenize(cnText));
    if (en.length !== cn.length) return null;
    const cnValues = new Map(cnBlackboard.map((b) => [b.key.toLowerCase(), b.value] as const));
    const moved = enBlackboard.flatMap((b) => {
        const next = cnValues.get(b.key.toLowerCase());
        return next === undefined || near(b.value, next) ? [] : [[b.value, next] as const];
    });
    const justified = (from: number, to: number) => moved.some(([a, b]) => DISPLAY_FORMS.some((f) => near(Math.abs(f(a)), from) && near(Math.abs(f(b)), to)));
    for (let i = 0; i < en.length; i++) {
        if (!near(en[i].value, cn[i].value) && !justified(en[i].value, cn[i].value)) return null;
    }
    let n = 0;
    return tokens.map((t) => (t.kind === "num" ? cn[n++].text : t.text)).join("");
}

/**
 * This server's `template` filled with CN's values, as HTML with each changed number shown as
 * this server's struck through and CN's highlighted. `cnText`, CN's own text of the same
 * description, carries the numbers the game writes into the text instead of a placeholder; with
 * it, those are carried over too (see {@link swapJustifiedNumbers}).
 *
 * Null when the template cannot be filled from CN's data: a placeholder key CN's blackboard lacks,
 * or written-in numbers that no longer line up. The caller then shows CN's own text.
 */
export function templateWithCnValues(template: string, blackboard: Blackboard, cnBlackboard: Blackboard, cnText?: string | null): string | null {
    if (!template || !hasKeys(template, cnBlackboard)) return null;
    const next = cnText ? swapJustifiedNumbers(template, cnText, blackboard, cnBlackboard) : template;
    if (next === null) return null;
    return renderDescriptionDiffHtml(template, next, [...blackboard], [...cnBlackboard]);
}

/**
 * A text from CN's data in this server's language, as far as this server's own text allows. `text`
 * is this server's text with CN's numbers, null when nothing here matches; `zh` is the Chinese CN
 * has with no counterpart here: the whole text when `text` is null, else what follows `text`.
 */
export interface ICnText {
    text: string | null;
    zh: string | null;
}

/** Where each of `from`'s numbers that `moved` sits in `to`, a translation of the same text; null when that is ambiguous. */
function alignNumbers(from: readonly number[], to: readonly number[], moved: (k: number) => boolean): number[][] | null {
    if (from.length === to.length && from.every((v, i) => near(v, to[i]))) return from.map((_, i) => [i]);
    // Reordered by the translation, or a number one side writes in words (CN's 三名, "3 Operators"):
    // match by value, which holds only when every occurrence of a value that moves moves together,
    // and both texts carry that value equally often.
    const count = (xs: readonly number[], v: number) => xs.filter((x) => near(x, v)).length;
    for (let k = 0; k < from.length; k++) {
        if (!moved(k)) continue;
        if (count(from, from[k]) !== count(to, from[k])) return null;
        if (from.some((w, j) => !moved(j) && near(from[k], w))) return null;
    }
    return from.map((v, k) => {
        if (!moved(k)) return [];
        return to.flatMap((x, i) => (near(x, v) ? [i] : []));
    });
}

/**
 * `cnText`, CN's rewrite of `cnBase`, rebuilt from `enBase`, this server's text of `cnBase`. Holds
 * when `cnText` is `cnBase` with only its numbers changed, or that followed by more text; the
 * changed numbers are carried into `enBase` and the added text stays Chinese in `zh`. Otherwise
 * the whole of `cnText` is `zh`.
 */
export function rebuildFromBase(cnText: string, cnBase: string, enBase: string): ICnText {
    const whole: ICnText = { text: null, zh: cnText };
    const next = tokenize(cnText);
    const base = tokenize(cnBase);
    if (base.length === 0 || base.length > next.length) return whole;
    for (let i = 0; i < base.length; i++) {
        const a = base[i];
        const b = next[i];
        if (a.kind !== b.kind || (a.kind === "lit" && a.text !== b.text)) return whole;
    }
    const enTokens = tokenize(enBase);
    const baseNums = numbers(base);
    const nextNums = numbers(next.slice(0, base.length));
    const moved = (k: number) => !near(baseNums[k].value, nextNums[k].value);
    const enNums = numbers(enTokens);
    const at = alignNumbers(
        baseNums.map((t) => t.value),
        enNums.map((t) => t.value),
        moved,
    );
    if (!at) return whole;
    const replaced = new Map<number, string>();
    for (let k = 0; k < baseNums.length; k++) {
        if (!moved(k)) continue;
        for (const i of at[k]) {
            const prior = replaced.get(i);
            if (prior !== undefined && prior !== nextNums[k].text) return whole;
            replaced.set(i, nextNums[k].text);
        }
    }
    let n = 0;
    const text = enTokens.map((t) => (t.kind === "num" ? (replaced.get(n++) ?? t.text) : t.text)).join("");
    const tail = next
        .slice(base.length)
        .map((t) => t.text)
        .join("");
    return { text, zh: tail.length > 0 ? tail : null };
}

/** A module candidate's name and description, rebuilt in this server's language where its text allows. */
export interface ICnTalentText {
    name: ICnText;
    description: ICnText;
}

/** Reads a CN module's talent and trait text in this server's language where this server's own text allows. */
export interface ICnModuleTexts {
    talent(candidate: { talentIndex: number; requiredPotentialRank: number; name: string | null; description: string; blackboard: Blackboard }): ICnTalentText;
    /** A trait part: an override of the whole trait, or an addition after it. */
    trait(text: string, kind: "override" | "addition", blackboard: Blackboard): ICnText;
}

/** The talent candidate a module rewrites at the second promotion and `potentialRank`. */
function talentAnchor(talent: ITalent | undefined, potentialRank: number): ITalentCandidate | null {
    return talent ? getActiveTalentCandidate(talent, 2, Number.MAX_SAFE_INTEGER, potentialRank) : null;
}

function sameUnlock(a: ITalentCandidate, b: ITalentCandidate): boolean {
    return a.unlockCondition.phase === b.unlockCondition.phase && a.unlockCondition.level === b.unlockCondition.level && a.requiredPotentialRank === b.requiredPotentialRank;
}

const zhOnly = (text: string): ICnText => ({ text: null, zh: text });

/**
 * Texts for the modules merged in from `cn`, read against `en`, this server's own record (before
 * the merge). A CN text is this server's where the two records pair it with one: the same text on
 * a talent, trait or module both servers have, or else one of `translations`, the same text on
 * another operator's module both servers have. Failing that, a talent rewrite is rebuilt from the
 * talent it rewrites and a trait override from the trait it replaces (see {@link rebuildFromBase}),
 * when both servers have that talent or trait in the same shape. A trait addition has nothing here
 * to be rebuilt from. Either way this server's text must take only keys `blackboard` carries.
 */
export function cnModuleTexts(en: IOperatorListItem, cn: IOperatorListItem, translations: readonly CnTranslation[] = []): ICnModuleTexts {
    const pairs = new Map<string, string>();
    const pair = (zh: string | null | undefined, here: string | null | undefined) => {
        if (zh && here && !pairs.has(zh)) pairs.set(zh, here);
    };
    pair(cn.description, en.description);
    const cnTraits = cn.trait?.candidates ?? [];
    const enTraits = en.trait?.candidates ?? [];
    cnTraits.forEach((c, i) => {
        pair(c.overrideDescription, enTraits[i]?.overrideDescription);
    });
    (cn.talents ?? []).forEach((talent, t) => {
        talent.candidates?.forEach((c, i) => {
            const here = en.talents?.[t]?.candidates?.[i];
            pair(c.name, here?.name);
            pair(c.description, here?.description);
        });
    });
    for (const module of cn.modules) {
        const here = en.modules.find((m) => m.uniEquipId === module.uniEquipId);
        module.data?.phases?.forEach((phase, p) => {
            phase.parts.forEach((part, i) => {
                const herePart = here?.data?.phases?.[p]?.parts?.[i];
                part.addOrOverrideTalentDataBundle.candidates?.forEach((c, k) => {
                    const h = herePart?.addOrOverrideTalentDataBundle.candidates?.[k];
                    pair(c.name, h?.name);
                    pair(c.upgradeDescription, h?.upgradeDescription);
                    pair(c.description, h?.description);
                });
                part.overrideTraitDataBundle.candidates?.forEach((c, k) => {
                    const h = herePart?.overrideTraitDataBundle.candidates?.[k];
                    pair(c.additionalDescription, h?.additionalDescription);
                    pair(c.overrideDescription, h?.overrideDescription);
                });
            });
        });
    }
    for (const t of translations) pair(t.cn, t.en);

    const resolve = (zh: string, blackboard: Blackboard, rebuild: () => ICnText): ICnText => {
        const paired = pairs.get(zh);
        if (paired !== undefined && hasKeys(paired, blackboard)) return { text: paired, zh: null };
        const rebuilt = rebuild();
        return rebuilt.text !== null && hasKeys(rebuilt.text, blackboard) ? rebuilt : zhOnly(zh);
    };

    return {
        talent({ talentIndex, requiredPotentialRank, name, description, blackboard }) {
            const cnTalent = cn.talents?.[talentIndex];
            const enTalent = en.talents?.[talentIndex];
            const cnBase = talentAnchor(cnTalent, requiredPotentialRank);
            const enBase = talentAnchor(enTalent, requiredPotentialRank);
            const sameShape = !!cnBase && !!enBase && sameUnlock(cnBase, enBase) && cnTalent?.candidates?.length === enTalent?.candidates?.length;
            const nameText: ICnText = !name ? { text: null, zh: null } : pairs.has(name) ? { text: pairs.get(name) ?? null, zh: null } : sameShape && name === cnBase.name && enBase.name ? { text: enBase.name, zh: null } : zhOnly(name);
            return {
                name: nameText,
                description: resolve(description, blackboard, () => (sameShape && cnBase.description && enBase.description ? rebuildFromBase(description, cnBase.description, enBase.description) : zhOnly(description))),
            };
        },
        trait(text, kind, blackboard) {
            if (kind === "addition") return resolve(text, blackboard, () => zhOnly(text));
            const cnLast = cnTraits[cnTraits.length - 1];
            const enLast = enTraits[enTraits.length - 1];
            const cnBase = cnLast?.overrideDescription ?? cn.description;
            const enBase = enLast?.overrideDescription ?? en.description;
            const sameShape = cnTraits.length === enTraits.length && !!cnLast?.overrideDescription === !!enLast?.overrideDescription;
            return resolve(text, blackboard, () => (sameShape && cnBase && enBase ? rebuildFromBase(text, cnBase, enBase) : zhOnly(text)));
        },
    };
}
