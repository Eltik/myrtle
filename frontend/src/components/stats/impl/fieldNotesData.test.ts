import { describe, expect, it } from "vitest";
import type { IEnemy } from "#/lib/api/enemies";
import { enemyImmunity, keyHasStatus, levelInflicts, mostOwnedSixStar, operatorsInflicting, ownableOperators, rarestSixStar, share } from "./fieldNotesData";

describe("keyHasStatus", () => {
    it("matches the status as a whole segment, or a segment that starts with `status_`", () => {
        for (const key of ["stun", "STUN", "stun_duration", "attack@stun", "attack@stun_main", "sleach_t_1[enemy].stun", "forcer_s_2[brush].stun", "failure.stun"]) {
            expect(keyHasStatus(key, "stun"), key).toBe(true);
        }
        expect(keyHasStatus("attack@sluggish", "sluggish")).toBe(true);
    });

    it("rejects a status that is only a prefix, a suffix or an infix of a segment", () => {
        // `stunned` is a different word: the rule needs the segment to end or continue with `_`.
        for (const key of ["stunned", "sluggish", "attack@sluggish", "max_stun", "attack@s2_stun", "unstun", "stu"]) {
            expect(keyHasStatus(key, "stun"), key).toBe(false);
        }
        expect(keyHasStatus("silence_resist", "sleep")).toBe(false);
    });
});

function enemy(id: string, level: IEnemy["enemyLevel"], stunImmune: boolean | null, extra: Partial<IEnemy> = {}): IEnemy {
    const stats = stunImmune === null ? null : { levels: [{ level: 0, attributes: { stunImmune } }] };
    return { enemyId: id, enemyLevel: level, hideInHandbook: false, stats, ...extra } as unknown as IEnemy;
}

describe("enemyImmunity", () => {
    it("leaves hidden and statless enemies out of the denominator", () => {
        const result = enemyImmunity([enemy("a", "BOSS", true), enemy("b", "NORMAL", false), enemy("c", "ELITE", true), enemy("statless", "NORMAL", null), enemy("hidden", "BOSS", true, { hideInHandbook: true })], "stun");
        expect(result.n).toBe(2);
        expect(result.total).toBe(3);
        expect(result.byLevel).toEqual({ BOSS: { n: 1, total: 1 }, ELITE: { n: 1, total: 1 }, NORMAL: { n: 0, total: 1 } });
    });

    it("gives a zero denominator, and no share, when nothing qualifies", () => {
        const result = enemyImmunity([enemy("statless", "NORMAL", null)], "stun");
        expect(result).toMatchObject({ n: 0, total: 0 });
        expect(share(result)).toBeNull();
    });
});

describe("operatorsInflicting", () => {
    const ops = [
        { id: "op_a", name: "A", rarity: 6 },
        { id: "op_b", name: "B", rarity: 5 },
        { id: "op_c", name: "C", rarity: 4 },
    ];
    const roster = [
        { id: "op_a", skills: [{ static: { levels: [{ blackboard: [{ key: "attack@stun", value: 1 }] }] } }] },
        {
            id: "op_b",
            skills: [
                {
                    static: {
                        levels: [
                            {
                                blackboard: [
                                    { key: "stun", value: 0 },
                                    { key: "attack@sluggish", value: 0.5 },
                                ],
                            },
                        ],
                    },
                },
            ],
        },
        { id: "op_c", skills: [{ static: null }] },
        { id: "not_in_index", skills: [{ static: { levels: [{ blackboard: [{ key: "stun", value: 2 }] }] } }] },
    ];

    it("counts nonzero matches only, over the index", () => {
        expect(operatorsInflicting(ops, roster, "stun")).toEqual({ n: 1, total: 3, ids: ["op_a"] });
    });

    it("has no share over an empty index", () => {
        expect(share(operatorsInflicting([], roster, "stun"))).toBeNull();
    });
});

describe("ownership picks", () => {
    const ops = [
        { id: "char_b", name: "B", rarity: 6 },
        { id: "char_a", name: "A", rarity: 6 },
        { id: "char_c", name: "C", rarity: 5 },
        { id: "char_d", name: "D", rarity: 6, isNotObtainable: true },
    ];

    it("breaks an owner tie by the lower id", () => {
        const ownership = { totalUsers: 10, counts: { char_a: { owners: 10 }, char_b: { owners: 10 }, char_c: { owners: 3 } } };
        expect(mostOwnedSixStar(ownableOperators(ops), ownership)).toEqual({ id: "char_a", name: "A", n: 10, total: 10 });
        expect(rarestSixStar(ownableOperators(ops), { totalUsers: 10, counts: { char_a: { owners: 4 }, char_b: { owners: 4 } } })?.id).toBe("char_a");
    });

    it("reads a missing id as zero owners and skips operators nobody can own", () => {
        const ownership = { totalUsers: 10, counts: { char_a: { owners: 9 } } };
        expect(rarestSixStar(ownableOperators(ops), ownership)).toEqual({ id: "char_b", name: "B", n: 0, total: 10 });
    });

    it("has no share when nobody has synced", () => {
        const pick = mostOwnedSixStar(ops, { totalUsers: 0, counts: {} });
        expect(pick && share(pick)).toBeNull();
        expect(rarestSixStar([], { totalUsers: 0, counts: {} })).toBeNull();
    });
});

describe("levelInflicts", () => {
    it("trusts the description's control tags over the keys", () => {
        // Mudrock S3: `sleep` times her own phase; the text tags only Stun.
        const mudrock = {
            description: "stops attacking for {sleep} seconds, then <$ba.stun>Stuns</> enemies",
            blackboard: [
                { key: "sleep", value: 10 },
                { key: "stun", value: 3 },
            ],
        };
        expect(levelInflicts(mudrock, "sleep")).toBe(false);
        expect(levelInflicts(mudrock, "stun")).toBe(true);
        // Aosta: a `frozen` key timing a Bind.
        expect(levelInflicts({ description: "<$ba.root>Binds</> the targets", blackboard: [{ key: "attack@frozen_duration", value: 2 }] }, "frozen")).toBe(false);
    });

    it("falls back to the keys when nothing is tagged, and always for silence", () => {
        expect(levelInflicts({ description: "chance of stunning targets", blackboard: [{ key: "attack@stun", value: 1 }] }, "stun")).toBe(true);
        expect(levelInflicts({ description: "<$ba.sluggish>Slowed</> and silenced", blackboard: [{ key: "silence", value: 3 }] }, "silence")).toBe(true);
        expect(levelInflicts({ description: "", blackboard: [{ key: "stun", value: 0 }] }, "stun")).toBe(false);
    });
});
