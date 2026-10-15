import { describe, expect, it } from "vitest";
import type { IRosterEntry } from "#/lib/api/user";
import type { SupportUnit } from "#/types/generated/SupportUnit";
import type { IUserProfile } from "#/types/user";
import { ogHashers } from "./hashers";
import { buildUserOgData } from "./user";

const PROFILE = {
    id: "u",
    uid: "12345678",
    nickname: "Eltik",
    nick_number: "1234",
    level: 120,
    avatar_id: "char_002_amiya",
    secretary: "char_002_amiya",
    secretary_skin_id: null,
    server: "EN",
    total_score: 0.8,
    grade: "S",
    public_profile: true,
    lmd: 1_000_000,
    item_count: 300,
    operator_count: 400,
    skin_count: 500,
    non_default_skin_count: 100,
    profile_layout: null,
} as unknown as IUserProfile;

function support(over: Partial<SupportUnit> = {}): SupportUnit {
    return { slot: 0, operator_id: "char_103_angel", skin_id: null, skill_index: 2, current_equip: null, elite: 2, level: 90, potential: 5, skill_level: 7, favor_point: 0, specialize_level: 3, ...over };
}

const hash = (supports: SupportUnit[], roster: IRosterEntry[] | null = null, profile: IUserProfile = PROFILE) => ogHashers.user.hash(buildUserOgData(profile, supports, roster));

describe("user card hash", () => {
    it("changes when a support unit changes", () => {
        const base = hash([support()]);
        expect(hash([support()])).toBe(base);
        expect(hash([support({ level: 80 })])).not.toBe(base);
        expect(hash([support({ specialize_level: 2 })])).not.toBe(base);
        expect(hash([support({ skill_index: 1 })])).not.toBe(base);
        expect(hash([support({ operator_id: "char_002_amiya" })])).not.toBe(base);
        expect(hash([])).not.toBe(base);
    });

    it("changes with the background and the roster, not with fields the card does not draw", () => {
        const base = hash([]);
        const withBg = { ...PROFILE, profile_layout: { tabs: [], background: { kind: "skin", id: "char_002_amiya@epoque#4", focus_y: 25 } } } as unknown as IUserProfile;
        expect(hash([], null, withBg)).not.toBe(base);
        const roster = [{ operator_id: "char_103_angel", elite: 2, level: 90, skin_id: null, default_skill: 2, masteries: [] }] as unknown as IRosterEntry[];
        expect(hash([], roster)).not.toBe(base);
        expect(hash([], null, { ...PROFILE, orundum: 99_999 } as IUserProfile)).toBe(base);
    });

    it("projects an owner's profile to what a visitor sees", () => {
        const layout = {
            tabs: [
                { id: "score", visible: false },
                { id: "roster", visible: false },
                { id: "inventory", visible: false },
            ],
        };
        const roster = [{ operator_id: "char_103_angel", elite: 2, level: 90, skin_id: null, default_skill: 2, masteries: [] }] as unknown as IRosterEntry[];
        const card = buildUserOgData({ ...PROFILE, profile_layout: layout } as unknown as IUserProfile, [], roster);
        expect(card.grade).toBeNull();
        expect(card.totalScore).toBeNull();
        expect(card.rosterDigest).toBe("");
        expect(card.extraStat).toEqual({ kind: "items", value: 300 });
    });

    it("drops the header numbers the owner hid, and hashes as before without a header", () => {
        const withTabs = { ...PROFILE, profile_layout: { tabs: [{ id: "inventory", visible: true }] } } as unknown as IUserProfile;
        expect(hash([], null, withTabs)).toBe(hash([], null, { ...PROFILE, profile_layout: { ...withTabs.profile_layout, header: { hidden: [] } } } as unknown as IUserProfile));
        const hidden = { ...withTabs, profile_layout: { ...withTabs.profile_layout, header: { hidden: ["level", "operators", "lmd"] } } } as unknown as IUserProfile;
        const card = buildUserOgData(hidden, [], null);
        expect(card.level).toBeNull();
        expect(card.operatorCount).toBeNull();
        expect(card.skinCount).toBe(100);
        expect(card.extraStat).toEqual({ kind: "items", value: 300 });
        const both = { ...withTabs, profile_layout: { ...withTabs.profile_layout, header: { hidden: ["lmd", "items"] } } } as unknown as IUserProfile;
        expect(buildUserOgData(both, [], null).extraStat).toEqual({ kind: "items", value: null });
        // A visitor's copy (nulls from the backend) draws the same card as the owner's projected one.
        expect(ogHashers.user.hash(card)).toBe(ogHashers.user.hash(buildUserOgData({ ...hidden, level: null, operator_count: null, lmd: null } as unknown as IUserProfile, [], null)));
    });
});
