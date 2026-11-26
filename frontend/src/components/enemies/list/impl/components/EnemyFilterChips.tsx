import type React from "react";
import { FilterChip } from "#/components/ui/filter-chip";
import { Kicker } from "#/components/ui/kicker";
import type { IEnemyDamageType, IEnemyLevel } from "#/lib/api/enemies";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { APPLY_WAY_LABEL_KEY, APPLY_WAYS, DAMAGE_TYPES, ENEMY_LEVEL_LABEL_KEY, ENEMY_LEVELS, IMMUNITY_KEYS, IMMUNITY_LABEL_KEY } from "../constants";
import type { messages as listConstantsMessages } from "../constants.messages";
import { DAMAGE_TOKENS, LEVEL_TOKENS } from "../tokens";
import type { ApplyWay, IEnemyImmunities, IFilterState } from "../types";
import type { messages } from "./EnemyFilterChips.messages";

interface IChipGroupProps {
    label: string;
    children: React.ReactNode;
}

function ChipGroup({ label, children }: IChipGroupProps) {
    return (
        <div className="flex w-full flex-col gap-1.5 sm:w-auto sm:flex-row sm:items-center sm:gap-2.5">
            <Kicker className="shrink-0 whitespace-nowrap leading-none">{label}</Kicker>
            <div className="flex flex-wrap gap-1.5">{children}</div>
        </div>
    );
}

function toggleArray<T>(list: T[], v: T): T[] {
    return list.includes(v) ? list.filter((x) => x !== v) : [...list, v];
}

export interface IRaceOption {
    id: string;
    label: string;
}

interface IEnemyFilterChipsProps {
    filters: IFilterState;
    setLevels: (v: IEnemyLevel[]) => void;
    setDamageTypes: (v: IEnemyDamageType[]) => void;
    setAttackTypes: (v: ApplyWay[]) => void;
    setRaces: (v: string[]) => void;
    setImmunities: (v: (keyof IEnemyImmunities)[]) => void;
    races: IRaceOption[];
}

export function EnemyFilterChips({ filters, setLevels, setDamageTypes, setAttackTypes, setRaces, setImmunities, races }: IEnemyFilterChipsProps) {
    const t: TypedT<typeof messages> = useT("enemies");
    const tConst: TypedT<typeof listConstantsMessages> = useT("enemies");
    return (
        <div className="flex flex-wrap items-center gap-x-3.5 gap-y-2.5 rounded-2xl border border-border bg-[color-mix(in_oklch,var(--card)_60%,transparent)] px-3.5 py-3">
            <ChipGroup label={t("chips.threat")}>
                {ENEMY_LEVELS.map((lv) => (
                    <FilterChip key={lv} active={filters.levels.includes(lv)} tone={LEVEL_TOKENS[lv].accent} toneInk={LEVEL_TOKENS[lv].ink} onSelect={() => setLevels(toggleArray(filters.levels, lv))} label={tConst(ENEMY_LEVEL_LABEL_KEY[lv])} />
                ))}
            </ChipGroup>

            <ChipGroup label={t("chips.damage")}>
                {DAMAGE_TYPES.map((d) => (
                    <FilterChip
                        key={d}
                        active={filters.damageTypes.includes(d)}
                        tone={DAMAGE_TOKENS[d].color}
                        toneInk={DAMAGE_TOKENS[d].ink}
                        onSelect={() => setDamageTypes(toggleArray(filters.damageTypes, d))}
                        label={
                            <span className="inline-flex items-center gap-1.75">
                                <span aria-hidden="true" className="inline-block h-1.5 w-1.5 rounded-[1.5px]" style={{ background: DAMAGE_TOKENS[d].color }} />
                                {tConst(DAMAGE_TOKENS[d].labelKey)}
                            </span>
                        }
                    />
                ))}
            </ChipGroup>

            <ChipGroup label={t("chips.range")}>
                {APPLY_WAYS.map((a) => (
                    <FilterChip key={a} active={filters.attackTypes.includes(a)} onSelect={() => setAttackTypes(toggleArray(filters.attackTypes, a))} label={tConst(APPLY_WAY_LABEL_KEY[a])} />
                ))}
            </ChipGroup>

            {/* AND, unlike the other groups: several picks keep only enemies immune to all of them. */}
            <ChipGroup label={t("chips.immune")}>
                {IMMUNITY_KEYS.map((k) => (
                    <FilterChip key={k} active={filters.immunities.includes(k)} onSelect={() => setImmunities(toggleArray(filters.immunities, k))} label={tConst(IMMUNITY_LABEL_KEY[k])} />
                ))}
            </ChipGroup>

            {races.length > 0 && (
                <>
                    <ChipGroup label={t("chips.race")}>
                        {races.map((r) => (
                            <FilterChip key={r.id} active={filters.races.includes(r.id)} onSelect={() => setRaces(toggleArray(filters.races, r.id))} label={r.label} />
                        ))}
                    </ChipGroup>
                </>
            )}
        </div>
    );
}
