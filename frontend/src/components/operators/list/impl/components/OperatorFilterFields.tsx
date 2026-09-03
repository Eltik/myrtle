import { ChevronDown } from "lucide-react";
import { type ReactNode, useState } from "react";
import { cn, formatNationId, formatProfession, formatSubProfession, rarityToNumber, subProfessionToProfession } from "#/lib/utils";
import { GENDERS, PROFESSION_ORDER, RARITIES } from "../constants";
import type { ArrayFilterKey, IFilterOptions, ISharedFilters } from "../types";
import { ClassPicker, toggle } from "./ClassPicker";
import { FilterDropdown } from "./FilterDropdown";
import { CampIcon, SubProfessionIcon, TeamIcon } from "./Icons";
import styles from "./OperatorFilters.module.css";

export function TagRow<T extends string>({ label, options, value, onChange }: { label: string; options: readonly { value: T; label: string }[]; value: T; onChange: (v: T) => void }) {
    return (
        <div className={styles.field}>
            <div className={styles.fieldLabel}>{label}</div>
            <div className={styles.tagRow}>
                {options.map((opt) => {
                    const on = value === opt.value;
                    return (
                        <button key={opt.value} type="button" className={cn(styles.tg, on && styles.on)} onClick={() => onChange(opt.value)} aria-pressed={on}>
                            {opt.label}
                        </button>
                    );
                })}
            </div>
        </div>
    );
}

export function OperatorFilterFields({
    filters,
    options,
    onChange,
    basicLeading,
    basicTrailing,
}: {
    filters: ISharedFilters;
    options: IFilterOptions;
    onChange: <K extends ArrayFilterKey>(key: K, value: ISharedFilters[K]) => void;
    /** Page-specific rows rendered first in Basic: Availability on /operators, Ownership on the roster. */
    basicLeading?: ReactNode;
    /** Page-specific rows rendered last in Basic: Notes on /operators. */
    basicTrailing?: ReactNode;
}) {
    const [advancedOpen, setAdvancedOpen] = useState(true);

    const advancedCount = filters.subclasses.length + filters.genders.length + filters.nations.length + filters.factions.length + filters.races.length + filters.birthPlaces.length + filters.artists.length + filters.voiceActors.length;

    return (
        <div className="flex flex-col gap-4.5">
            <div className="flex flex-col gap-4">
                <div className={styles.sectionHead}>
                    <span className={styles.lbl}>Basic</span>
                    <span className={styles.line} />
                </div>

                {basicLeading}

                <ClassPicker selected={filters.classes} onChange={(v) => onChange("classes", v)} />

                <div className={styles.field}>
                    <div className={styles.fieldLabel}>Rarity</div>
                    <div className={styles.rarityRow}>
                        {RARITIES.map((r) => {
                            const on = filters.rarities.includes(r);
                            return (
                                <button key={r} type="button" data-rarity={rarityToNumber(r)} className={cn(styles.rarityBtn, on && styles.on)} onClick={() => onChange("rarities", toggle(filters.rarities, r))} aria-pressed={on}>
                                    {rarityToNumber(r)}★
                                </button>
                            );
                        })}
                    </div>
                </div>

                {basicTrailing}
            </div>

            <div className="flex flex-col gap-4">
                <button type="button" className={cn(styles.sectionHead, styles.expandable, advancedOpen && styles.open)} onClick={() => setAdvancedOpen((v) => !v)} aria-expanded={advancedOpen}>
                    <span className={styles.lbl}>Advanced</span>
                    {advancedCount > 0 && <span className={styles.n}>{advancedCount}</span>}
                    <span className={styles.line} />
                    <ChevronDown className={styles.chev} aria-hidden="true" />
                </button>

                {advancedOpen && (
                    <div className="flex flex-col gap-4">
                        <FilterDropdown
                            label="Archetype"
                            placeholder="Select archetype"
                            options={options.subclasses}
                            selected={filters.subclasses}
                            onChange={(v) => onChange("subclasses", v)}
                            formatOption={formatSubProfession}
                            groupBy={subProfessionToProfession}
                            groupOrder={PROFESSION_ORDER}
                            formatGroup={formatProfession}
                            renderOptionIcon={(v) => <SubProfessionIcon subProfession={v} size={18} />}
                        />

                        <div className={styles.field}>
                            <div className={styles.fieldLabel}>Gender</div>
                            <div className={styles.tagRow}>
                                {GENDERS.map((g) => {
                                    const on = filters.genders.includes(g);
                                    return (
                                        <button key={g} type="button" className={cn(styles.tg, on && styles.on)} onClick={() => onChange("genders", toggle(filters.genders, g))} aria-pressed={on}>
                                            {g}
                                        </button>
                                    );
                                })}
                            </div>
                        </div>

                        <FilterDropdown label="Nation" placeholder="Select nation" options={options.nations} selected={filters.nations} onChange={(v) => onChange("nations", v)} formatOption={(n) => formatNationId(n) ?? n} renderOptionIcon={(v) => <TeamIcon teamId={v} size={18} />} />

                        <FilterDropdown label="Faction" placeholder="Select faction" options={options.factions} selected={filters.factions} onChange={(v) => onChange("factions", v)} formatOption={formatNationId} renderOptionIcon={(v) => <CampIcon groupId={v} size={18} />} />

                        <FilterDropdown label="Race" placeholder="Select race" options={options.races} selected={filters.races} onChange={(v) => onChange("races", v)} />

                        <FilterDropdown label="Place of Birth" placeholder="Select birth place" options={options.birthPlaces} selected={filters.birthPlaces} onChange={(v) => onChange("birthPlaces", v)} />

                        <FilterDropdown label="Artist" placeholder="Select artist" options={options.artists} selected={filters.artists} onChange={(v) => onChange("artists", v)} />

                        <FilterDropdown label="Voice Actor" placeholder="Select voice actor" options={options.voiceActors} selected={filters.voiceActors} onChange={(v) => onChange("voiceActors", v)} />
                    </div>
                )}
            </div>
        </div>
    );
}
