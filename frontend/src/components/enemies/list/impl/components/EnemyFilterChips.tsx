import type React from "react";
import type { IEnemyDamageType, IEnemyLevel } from "#/lib/api/enemies";
import { APPLY_WAY_DISPLAY, APPLY_WAYS, DAMAGE_TYPES, ENEMY_LEVEL_DISPLAY, ENEMY_LEVELS } from "../constants";
import { DAMAGE_TOKENS, LEVEL_TOKENS } from "../tokens";
import type { ApplyWay, IFilterState } from "../types";

interface IChipProps {
    on: boolean;
    color?: string;
    onClick: () => void;
    children: React.ReactNode;
}

function Chip({ on, color, onClick, children }: IChipProps) {
    const tint = color ?? "var(--primary)";
    return (
        <button
            type="button"
            onClick={onClick}
            aria-pressed={on}
            className="inline-flex h-7 cursor-pointer items-center gap-1.75 rounded-full border px-2.75 font-medium font-sans text-[12px] leading-none transition-all duration-150"
            style={{
                borderColor: on ? tint : "var(--border)",
                background: on ? `color-mix(in oklch, ${tint} 14%, transparent)` : "color-mix(in oklch, var(--secondary) 50%, transparent)",
                color: on ? tint : "var(--muted-foreground)",
            }}
            onMouseEnter={(e) => {
                if (!on) {
                    e.currentTarget.style.borderColor = `color-mix(in oklch, ${tint} 50%, var(--border))`;
                    e.currentTarget.style.color = "var(--foreground)";
                }
            }}
            onMouseLeave={(e) => {
                if (!on) {
                    e.currentTarget.style.borderColor = "var(--border)";
                    e.currentTarget.style.color = "var(--muted-foreground)";
                }
            }}
        >
            {children}
        </button>
    );
}

interface IChipGroupProps {
    label: string;
    children: React.ReactNode;
}

function ChipGroup({ label, children }: IChipGroupProps) {
    return (
        <div className="flex w-full flex-col gap-1.5 sm:w-auto sm:flex-row sm:items-center sm:gap-2.5">
            <span className="shrink-0 whitespace-nowrap font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.14em]">{label}</span>
            <div className="flex flex-wrap gap-1.5">{children}</div>
        </div>
    );
}

function ChipDivider() {
    return <span aria-hidden="true" className="mx-1 hidden h-6 w-px self-stretch bg-border sm:block" />;
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
    races: IRaceOption[];
}

export function EnemyFilterChips({ filters, setLevels, setDamageTypes, setAttackTypes, setRaces, races }: IEnemyFilterChipsProps) {
    return (
        <div className="flex flex-wrap items-center gap-x-3.5 gap-y-2.5 rounded-xl border border-border bg-[color-mix(in_oklch,var(--card)_60%,transparent)] px-3.5 py-3">
            <ChipGroup label="Threat">
                {ENEMY_LEVELS.map((lv) => (
                    <Chip key={lv} on={filters.levels.includes(lv)} color={LEVEL_TOKENS[lv].accent} onClick={() => setLevels(toggleArray(filters.levels, lv))}>
                        {ENEMY_LEVEL_DISPLAY[lv]}
                    </Chip>
                ))}
            </ChipGroup>

            <ChipDivider />

            <ChipGroup label="Damage">
                {DAMAGE_TYPES.map((d) => (
                    <Chip key={d} on={filters.damageTypes.includes(d)} color={DAMAGE_TOKENS[d].color} onClick={() => setDamageTypes(toggleArray(filters.damageTypes, d))}>
                        <span className="inline-block h-1.5 w-1.5 rounded-[1.5px]" style={{ background: DAMAGE_TOKENS[d].color }} />
                        {DAMAGE_TOKENS[d].label}
                    </Chip>
                ))}
            </ChipGroup>

            <ChipDivider />

            <ChipGroup label="Range">
                {APPLY_WAYS.map((a) => (
                    <Chip key={a} on={filters.attackTypes.includes(a)} onClick={() => setAttackTypes(toggleArray(filters.attackTypes, a))}>
                        {APPLY_WAY_DISPLAY[a]}
                    </Chip>
                ))}
            </ChipGroup>

            {races.length > 0 && (
                <>
                    <ChipDivider />
                    <ChipGroup label="Race">
                        {races.map((r) => (
                            <Chip key={r.id} on={filters.races.includes(r.id)} onClick={() => setRaces(toggleArray(filters.races, r.id))}>
                                {r.label}
                            </Chip>
                        ))}
                    </ChipGroup>
                </>
            )}
        </div>
    );
}
