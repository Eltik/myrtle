import { SearchIcon } from "lucide-react";
import type * as React from "react";
import { FilterDropdown } from "#/components/operators/list/impl/components/FilterDropdown";
import { ClassIcon, TeamIcon } from "#/components/operators/list/impl/components/Icons";
import { FilterChip } from "#/components/ui/filter-chip";
import { Input } from "#/components/ui/input";
import { InputGroup, InputGroupAddon } from "#/components/ui/input-group";
import { Kicker } from "#/components/ui/kicker";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, formatNationId, professionLabel } from "#/lib/utils";
import type { OperatorProfession } from "#/types/operators";
import { PROFESSIONS, RARITIES } from "../constants";
import { rarityInkVar, rarityVar } from "../helpers";
import type { IBirthdayFilters } from "../types";
import type { messages } from "./FilterControls.messages";

type FilterT = TypedT<typeof messages>;

interface IFilterControlsProps {
    filters: IBirthdayFilters;
    onChange: (next: IBirthdayFilters) => void;
    nations: [string, string][];
    /** Class code -> the server's own class name; a class it lacks gets the English formatter. */
    classNames: ReadonlyMap<string, string>;
}

export function FilterControls({ filters, onChange, nations, classNames }: IFilterControlsProps): React.ReactElement {
    const t: FilterT = useT("tools");
    const toggleRarity = (r: number) => {
        const rarities = new Set(filters.rarities);
        rarities.has(r) ? rarities.delete(r) : rarities.add(r);
        onChange({ ...filters, rarities });
    };
    const toggleProfession = (p: OperatorProfession) => {
        const professions = new Set(filters.professions);
        professions.has(p) ? professions.delete(p) : professions.add(p);
        onChange({ ...filters, professions });
    };

    return (
        <div className="flex flex-col">
            <Section label={t("birthdays.filters.search")} t={t}>
                <InputGroup>
                    <InputGroupAddon>
                        <SearchIcon />
                    </InputGroupAddon>
                    <Input placeholder={t("birthdays.filters.search.placeholder")} value={filters.query} onChange={(e) => onChange({ ...filters, query: e.target.value })} />
                </InputGroup>
            </Section>

            <Section label={t("birthdays.filters.class")} t={t} onClear={filters.professions.size > 0 ? () => onChange({ ...filters, professions: new Set() }) : undefined}>
                <div className="grid grid-cols-4 gap-1.5">
                    {PROFESSIONS.map((p) => {
                        const on = filters.professions.has(p);
                        const label = professionLabel({ profession: p, professionName: classNames.get(p) });
                        return (
                            <Tooltip key={p}>
                                <TooltipTrigger
                                    render={
                                        <button
                                            type="button"
                                            aria-label={label}
                                            aria-pressed={on}
                                            onClick={() => toggleProfession(p)}
                                            className={cn(
                                                "inline-flex aspect-square pointer-coarse:min-h-11 cursor-pointer items-center justify-center rounded-md border transition-colors",
                                                on
                                                    ? "border-primary bg-[color-mix(in_oklch,var(--primary)_16%,transparent)] text-primary"
                                                    : "border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] text-muted-foreground hover:border-[color-mix(in_oklch,var(--primary)_45%,var(--border))] hover:text-foreground",
                                            )}
                                        />
                                    }
                                >
                                    <ClassIcon profession={p} size={20} />
                                </TooltipTrigger>
                                <TooltipPopup>{label}</TooltipPopup>
                            </Tooltip>
                        );
                    })}
                </div>
            </Section>

            <Section label={t("birthdays.filters.rarity")} t={t} onClear={filters.rarities.size > 0 ? () => onChange({ ...filters, rarities: new Set() }) : undefined}>
                <div className="flex flex-wrap gap-1.5">
                    {RARITIES.map((r) => (
                        <FilterChip key={r} active={filters.rarities.has(r)} label={t("birthdays.filters.rarityChip", { rarity: r })} onSelect={() => toggleRarity(r)} tone={rarityVar(r)} toneInk={rarityInkVar(r)} />
                    ))}
                </div>
            </Section>

            <Section label={t("birthdays.filters.nation")} t={t} onClear={filters.nations.size > 0 ? () => onChange({ ...filters, nations: new Set() }) : undefined}>
                <FilterDropdown
                    placeholder={t("birthdays.filters.nation.placeholder")}
                    options={nations.map(([id]) => id)}
                    selected={[...filters.nations]}
                    onChange={(values) => onChange({ ...filters, nations: new Set(values) })}
                    formatOption={(n) => nations.find(([id]) => id === n)?.[1] ?? formatNationId(n) ?? n}
                    renderOptionIcon={(v) => <TeamIcon teamId={v} size={18} />}
                />
            </Section>
        </div>
    );
}

interface ISectionProps {
    label: string;
    onClear?: () => void;
    children: React.ReactNode;
    t: FilterT;
}

function Section({ label, onClear, children, t }: ISectionProps): React.ReactElement {
    return (
        <div className="border-border border-t py-3.5 first:border-t-0 first:pt-0">
            <div className="mb-2.5 flex items-center justify-between">
                <Kicker className="mb-0">{label}</Kicker>
                {onClear && (
                    <button
                        type="button"
                        className="pointer-coarse:after:-translate-1/2 relative -my-1 cursor-pointer rounded-sm px-1 py-1 text-[11px] text-primary underline-offset-2 pointer-coarse:after:absolute pointer-coarse:after:top-1/2 pointer-coarse:after:left-1/2 pointer-coarse:after:size-11 hover:underline"
                        onClick={onClear}
                    >
                        {t("birthdays.filters.clear")}
                    </button>
                )}
            </div>
            {children}
        </div>
    );
}
