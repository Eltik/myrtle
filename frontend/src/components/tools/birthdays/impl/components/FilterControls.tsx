import { SearchIcon } from "lucide-react";
import type * as React from "react";
import { FilterDropdown } from "#/components/operators/list/impl/components/FilterDropdown";
import { ClassIcon, TeamIcon } from "#/components/operators/list/impl/components/Icons";
import { Input } from "#/components/ui/input";
import { InputGroup, InputGroupAddon } from "#/components/ui/input-group";
import { cn, formatNationId, formatProfession } from "#/lib/utils";
import type { OperatorProfession } from "#/types/operators";
import { PROFESSIONS, RARITIES } from "../constants";
import { rarityVar } from "../helpers";
import type { IBirthdayFilters } from "../types";

interface IFilterControlsProps {
    filters: IBirthdayFilters;
    onChange: (next: IBirthdayFilters) => void;
    nations: [string, string][];
}

/** The filter form: search · class · rarity · nation. */
export function FilterControls({ filters, onChange, nations }: IFilterControlsProps): React.ReactElement {
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
            <Section label="Search">
                <InputGroup>
                    <InputGroupAddon>
                        <SearchIcon />
                    </InputGroupAddon>
                    <Input placeholder="Operator name…" value={filters.query} onChange={(e) => onChange({ ...filters, query: e.target.value })} />
                </InputGroup>
            </Section>

            <Section label="Class" onClear={filters.professions.size > 0 ? () => onChange({ ...filters, professions: new Set() }) : undefined}>
                <div className="grid grid-cols-4 gap-1.5">
                    {PROFESSIONS.map((p) => {
                        const on = filters.professions.has(p);
                        return (
                            <button
                                key={p}
                                type="button"
                                title={formatProfession(p)}
                                aria-pressed={on}
                                onClick={() => toggleProfession(p)}
                                className={cn(
                                    "inline-flex aspect-square cursor-pointer items-center justify-center rounded-md border transition-colors",
                                    on ? "border-primary bg-[color-mix(in_oklch,var(--primary)_16%,transparent)] text-primary" : "border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] text-muted-foreground hover:border-[color-mix(in_oklch,var(--primary)_45%,var(--border))] hover:text-foreground",
                                )}
                            >
                                <ClassIcon profession={p} size={20} />
                            </button>
                        );
                    })}
                </div>
            </Section>

            <Section label="Rarity" onClear={filters.rarities.size > 0 ? () => onChange({ ...filters, rarities: new Set() }) : undefined}>
                <div className="grid grid-cols-6 gap-1.5">
                    {RARITIES.map((r) => {
                        const on = filters.rarities.has(r);
                        return (
                            <button
                                key={r}
                                type="button"
                                data-rarity={r}
                                aria-pressed={on}
                                onClick={() => toggleRarity(r)}
                                style={{ "--rc": rarityVar(r) } as React.CSSProperties}
                                className={cn(
                                    "inline-flex h-7 cursor-pointer items-center justify-center rounded-md border font-sans font-semibold text-[12.5px] transition-colors",
                                    on ? "border-[var(--rc)] bg-[color-mix(in_oklch,var(--rc)_18%,transparent)] text-[var(--rc)]" : "border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] text-muted-foreground hover:border-[color-mix(in_oklch,var(--rc)_55%,var(--border))] hover:text-foreground",
                                )}
                            >
                                {r}★
                            </button>
                        );
                    })}
                </div>
            </Section>

            <Section label="Nation" onClear={filters.nations.size > 0 ? () => onChange({ ...filters, nations: new Set() }) : undefined}>
                <FilterDropdown placeholder="All nations" options={nations.map(([id]) => id)} selected={[...filters.nations]} onChange={(values) => onChange({ ...filters, nations: new Set(values) })} formatOption={(n) => formatNationId(n) ?? n} renderOptionIcon={(v) => <TeamIcon teamId={v} size={18} />} />
            </Section>
        </div>
    );
}

interface ISectionProps {
    label: string;
    onClear?: () => void;
    children: React.ReactNode;
}

function Section({ label, onClear, children }: ISectionProps): React.ReactElement {
    return (
        <div className="border-border border-t py-3.5 first:border-t-0 first:pt-0">
            <div className="mb-2.5 flex items-center justify-between font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.08em]">
                <span>{label}</span>
                {onClear && (
                    <button type="button" className="-my-1 cursor-pointer px-1 py-1 font-sans text-[11px] text-primary normal-case tracking-normal transition-colors hover:text-primary/80" onClick={onClear}>
                        clear
                    </button>
                )}
            </div>
            {children}
        </div>
    );
}
