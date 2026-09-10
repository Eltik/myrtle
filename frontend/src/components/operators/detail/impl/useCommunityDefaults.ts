import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { useAuth } from "#/hooks/use-auth";
import { operatorBuildStatsQueryOptions } from "#/lib/api/operators";
import { userRosterOperatorQueryOptions } from "#/lib/api/user";
import type { BuildChoice } from "#/types/generated/BuildChoice";
import type { ModuleLevelStats } from "#/types/generated/ModuleLevelStats";
import type { SkillMasteryStats } from "#/types/generated/SkillMasteryStats";
import type { IOperatorListItem } from "#/types/operators";

/**
 * Kill switch. `?defaults=0` on `/operators/{id}` restores the pre-feature
 * openings exactly: the last skill on the Skills tab, the first non-INITIAL
 * module on Information. Checked for an explicit "0" rather than falsiness,
 * because an absent parameter and a disabled one are different states.
 */
function communityDefaultsEnabled(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("defaults") !== "0";
}

/** `RosterEntry.masteries` and `.modules` arrive as raw `jsonb_agg` output, so
 *  they are typed as unknown JSON. Narrow them to row objects once, here,
 *  rather than casting at every read. */
function asRows(value: unknown): Record<string, unknown>[] {
    return Array.isArray(value) ? (value.filter((v) => typeof v === "object" && v !== null) as Record<string, unknown>[]) : [];
}

/** How many people picked one option, and what share of the cohort that is. */
export interface IChoiceShare {
    users: number;
    /** Fraction of the cohort, in [0, 1]. */
    share: number;
    /** 0 for the most-picked option. Ordering comes from the backend, which
     *  applies an explicit `users DESC, id ASC` tie-break, so this is stable
     *  across processes even when two options tie exactly. */
    rank: number;
}

export interface ICommunityDefaults {
    /** The viewer's own default skill index, 0-based, or `null` when they are
     *  signed out, do not own the operator, or the operator has no skills (the
     *  game stores -1 for that, which is a sentinel and not an index). */
    ownSkillIndex: number | null;
    /** The viewer's own equipped module id, or `null`. ORIGINAL and unequipped
     *  both resolve to `null`: neither is a chosen module. */
    ownModuleId: string | null;
    /** Index of the community's most-chosen skill, resolved through the stable
     *  `skillId` rather than by array position, or `null` when the cohort is
     *  below the reporting floor. */
    communitySkillIndex: number | null;
    /** Share of the E2 cohort that picked `communitySkillIndex`, in [0, 1]. */
    communitySkillShare: number | null;
    communityModuleId: string | null;
    /** Share of owners with a module equipped who picked `communityModuleId`. */
    communityModuleShare: number | null;

    /** Every skill's share of the E2 cohort, keyed by the stable `skillId`.
     *  Empty when the cohort is below the reporting floor. */
    skillShares: ReadonlyMap<string, IChoiceShare>;
    /** Size of the E2 cohort this operator's skill shares are drawn from. */
    skillTotal: number;
    /** Every module's share, keyed by `uniEquipId`. The denominator is owners
     *  with an ADVANCED module equipped, NOT all owners: 72.75% of roster rows
     *  have nothing equipped and a further 13.62% carry ORIGINAL, and counting
     *  those would report a preference nobody expressed. */
    moduleShares: ReadonlyMap<string, IChoiceShare>;
    moduleTotal: number;

    /** Mastery histogram per skill, keyed by `skillId`. */
    masteries: ReadonlyMap<string, SkillMasteryStats>;
    /** Level histogram per module, keyed by `uniEquipId`. */
    moduleLevels: ReadonlyMap<string, ModuleLevelStats>;
    /** The viewer's own mastery per skill, keyed by `skillId`. Only populated
     *  at E2, the same cohort the histogram counts. */
    ownMasteries: ReadonlyMap<string, number>;
    /** The viewer's own level per module, keyed by `uniEquipId`, where 0 means
     *  unlocked-not-yet or locked. Only populated at E2. */
    ownModuleLevels: ReadonlyMap<string, number>;
}

const NONE: ICommunityDefaults = {
    ownSkillIndex: null,
    ownModuleId: null,
    communitySkillIndex: null,
    communitySkillShare: null,
    communityModuleId: null,
    communityModuleShare: null,
    skillShares: new Map(),
    skillTotal: 0,
    moduleShares: new Map(),
    moduleTotal: 0,
    masteries: new Map(),
    moduleLevels: new Map(),
    ownMasteries: new Map(),
    ownModuleLevels: new Map(),
};

/**
 * What this operator's tabs should open on.
 *
 * Precedence is the viewer's own setting, then the community's, then the
 * component's existing fallback. The viewer's own comes first because it is not
 * a corner case: across 166,719 owned-operator pairs the viewer's own default
 * differs from the community modal 24.48% of the time, so deferring to the
 * community would contradict their explicit in-game choice on roughly a quarter
 * of visits.
 *
 * Every field can be `null`, and callers must treat that as "keep doing what you
 * did before" rather than substituting a zero.
 */
export function useCommunityDefaults(operator: IOperatorListItem): ICommunityDefaults {
    const enabled = communityDefaultsEnabled();
    const operatorId = operator.id ?? "";
    const { user } = useAuth();

    const { data: stats } = useQuery({
        ...operatorBuildStatsQueryOptions(operatorId),
        enabled: enabled && operatorId.length > 0,
    });
    const { data: own } = useQuery({
        ...userRosterOperatorQueryOptions(user?.uid ?? "", operatorId),
        enabled: enabled && !!user?.uid && operatorId.length > 0,
    });

    return useMemo(() => {
        if (!enabled) return NONE;

        const skills = operator.skills ?? [];
        const modules = operator.modules ?? [];

        // The viewer's own default counts only once they have E2'd this
        // operator, the same cohort the community aggregate uses.
        //
        // Below E2 the value is the game's placeholder, not a choice: 100.0% of
        // E0 rows carry `default_skill = 0` (all 323,521 of them) because S1 is
        // the only skill unlocked, and 65.0% of E1 rows do. Honouring it meant
        // that for any operator the viewer owned but had not promoted, the
        // Skills tab opened on S1 and the community statistic was discarded.
        // Measured across owned-but-not-E2 rows, that value disagrees with the
        // community modal 68.75% of the time, against 24.54% at E2.
        //
        // -1 is separately the game's "this operator has no skills" sentinel.
        // Reading it as an index would select the last skill via negative
        // indexing, so it is excluded explicitly rather than by a falsy check,
        // which would also have swallowed a legitimate 0 (S1).
        const rawOwnSkill = own?.elite === 2 ? own.default_skill : null;
        const ownSkillIndex = typeof rawOwnSkill === "number" && rawOwnSkill >= 0 && rawOwnSkill < skills.length ? rawOwnSkill : null;

        // A null current_equip and a `uniequip_001_*` id both mean "no module
        // chosen" - 72.75% and 13.62% of roster rows respectively. Only a module
        // this operator actually has counts.
        //
        // No elite gate here, unlike the skill above: a module cannot be
        // equipped before E2, and the data agrees with the rule. Zero of the
        // 448,046 rows below E2 name an ADVANCED module, so the filter already
        // implies the cohort and an explicit check could never fire.
        const rawOwnModule = own?.current_equip ?? null;
        const ownModuleId = rawOwnModule && !rawOwnModule.startsWith("uniequip_001") && modules.some((m) => m.uniEquipId === rawOwnModule) ? rawOwnModule : null;

        // The viewer's own investment, read from the roster view's jsonb
        // aggregates. Gated on E2 for the same reason the community histogram
        // is: below E2 neither a mastery nor a module level can exist, so any
        // value there is a placeholder rather than a stopping point.
        const ownMasteries = new Map<string, number>();
        const ownModuleLevels = new Map<string, number>();
        if (own?.elite === 2) {
            for (const entry of asRows(own.masteries)) {
                const index = Number(entry.index);
                const skillId = Number.isInteger(index) ? skills[index]?.skillId : undefined;
                if (skillId) ownMasteries.set(skillId, Number(entry.mastery) || 0);
            }
            for (const entry of asRows(own.modules)) {
                const id = typeof entry.id === "string" ? entry.id : undefined;
                // `locked` mirrors the backend remap: a locked module reports
                // level 1 in the source but has not actually been unlocked.
                if (id) ownModuleLevels.set(id, entry.locked ? 0 : Number(entry.level) || 0);
            }
        }

        // The backend ordered these most-picked first with an explicit
        // `users DESC, id ASC` tie-break, so [0] is the modal choice and is the
        // same across processes. It also withheld the list entirely when the
        // cohort was under its floor, so a present entry is already above it.
        const topSkill = stats?.defaultSkills?.[0];
        const topModule = stats?.defaultModules?.[0];

        const communityIdx = topSkill ? skills.findIndex((s) => s.skillId === topSkill.id) : -1;
        const communityModuleId = topModule && modules.some((m) => m.uniEquipId === topModule.id) ? topModule.id : null;

        // The backend withholds a distribution entirely below its floor, so an
        // entry existing already means it is reportable. Ranks come from array
        // position because the ordering is the backend's tie-broken one.
        const toShares = (choices: BuildChoice[] | undefined, total: number): ReadonlyMap<string, IChoiceShare> => {
            const out = new Map<string, IChoiceShare>();
            if (!choices || total <= 0) return out;
            choices.forEach((c, rank) => {
                out.set(c.id, { users: c.users, share: c.users / total, rank });
            });
            return out;
        };

        return {
            ownSkillIndex,
            ownModuleId,
            communitySkillIndex: communityIdx >= 0 ? communityIdx : null,
            communitySkillShare: topSkill && stats && stats.skillTotal > 0 ? topSkill.users / stats.skillTotal : null,
            communityModuleId,
            communityModuleShare: topModule && stats && stats.moduleTotal > 0 ? topModule.users / stats.moduleTotal : null,
            skillShares: toShares(stats?.defaultSkills, stats?.skillTotal ?? 0),
            skillTotal: stats?.skillTotal ?? 0,
            moduleShares: toShares(stats?.defaultModules, stats?.moduleTotal ?? 0),
            moduleTotal: stats?.moduleTotal ?? 0,
            masteries: new Map((stats?.masteries ?? []).map((m) => [m.skillId, m])),
            moduleLevels: new Map((stats?.moduleLevels ?? []).map((m) => [m.uniEquipId, m])),
            ownMasteries,
            ownModuleLevels,
        };
    }, [enabled, operator, own, stats]);
}
