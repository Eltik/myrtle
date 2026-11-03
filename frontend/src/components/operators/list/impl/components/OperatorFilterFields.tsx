import { ChevronDown } from "lucide-react";
import { type ReactNode, useId, useState } from "react";
import { useOperatorFactLabel } from "#/components/operators/OperatorFacts";
import { Badge } from "#/components/ui/badge";
import { FilterChip } from "#/components/ui/filter-chip";
import { Kicker } from "#/components/ui/kicker";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, formatNationId, nationLabel, professionLabel, rarityInk, rarityToNumber, subProfessionLabel, subProfessionToProfession } from "#/lib/utils";
import { GENDERS, MODULE_TYPES, PROFESSION_ORDER, RARITIES } from "../constants";
import type { ArrayFilterKey, IFilterOptions, ISharedFilters } from "../types";
import { ClassPicker, toggle } from "./ClassPicker";
import { FilterDropdown } from "./FilterDropdown";
import { CampIcon, SubProfessionIcon, TeamIcon } from "./Icons";
import type { messages } from "./OperatorFilterFields.messages";
import styles from "./OperatorFilters.module.css";

/** One of several, so a ToggleGroup (accent pressed, no check), not the multi-select FilterChip. */
export function TagRow<T extends string>({ label, options, value, onChange }: { label: string; options: readonly { value: T; label: string }[]; value: T; onChange: (v: T) => void }) {
    const labelId = useId();
    return (
        <div className={styles.field}>
            <div className={styles.fieldLabel} id={labelId}>
                {label}
            </div>
            <ToggleGroup aria-labelledby={labelId} className={cn(styles.tagRow, "w-auto")} onValueChange={(next) => next[0] !== undefined && onChange(next[0] as T)} size="sm" value={[value]} variant="outline">
                {options.map((opt) => (
                    <ToggleGroupItem className="px-3 text-muted-foreground hover:text-foreground data-pressed:text-foreground" key={opt.value} value={opt.value}>
                        {opt.label}
                    </ToggleGroupItem>
                ))}
            </ToggleGroup>
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
    const t: TypedT<typeof messages> = useT("operators");
    const fact = useOperatorFactLabel();
    const [advancedOpen, setAdvancedOpen] = useState(true);

    const advancedCount = filters.subclasses.length + filters.genders.length + filters.nations.length + filters.factions.length + filters.races.length + filters.birthPlaces.length + filters.artists.length + filters.voiceActors.length;

    return (
        <div className="flex flex-col gap-4.5">
            <div className="flex flex-col gap-4">
                <div className={styles.sectionHead}>
                    <Kicker>{t("filters.section.basic")}</Kicker>
                    <span className={styles.line} />
                </div>

                {basicLeading}

                <ClassPicker selected={filters.classes} onChange={(v) => onChange("classes", v)} labels={options.labels?.classes} />

                <div className={styles.field}>
                    <div className={styles.fieldLabel}>{t("filters.rarity")}</div>
                    {/* Three across: six across the 280px sidebar left each chip too narrow for the
                        check mark and the label side by side. */}
                    <div className="grid grid-cols-3 gap-1.5">
                        {RARITIES.map((r) => {
                            const n = rarityToNumber(r);
                            return <FilterChip key={r} active={filters.rarities.includes(r)} className="w-full" label={`${n}★`} onSelect={() => onChange("rarities", toggle(filters.rarities, r))} tone={`var(--rarity-${n})`} toneInk={rarityInk(n)} />;
                        })}
                    </div>
                </div>

                <div className={styles.field}>
                    <div className={styles.fieldLabel}>{t("filters.module")}</div>
                    <div className={styles.tagRow}>
                        {MODULE_TYPES.map((m) => (
                            <FilterChip key={m.value} active={filters.modules.includes(m.value)} label={m.label} onSelect={() => onChange("modules", toggle(filters.modules, m.value))} />
                        ))}
                    </div>
                </div>

                {basicTrailing}
            </div>

            <div className="flex flex-col gap-4">
                <button type="button" className={cn(styles.sectionHead, styles.expandable, advancedOpen && styles.open)} onClick={() => setAdvancedOpen((v) => !v)} aria-expanded={advancedOpen}>
                    <Kicker>{t("filters.section.advanced")}</Kicker>
                    {advancedCount > 0 && (
                        <Badge className="font-mono tabular-nums" size="sm">
                            {advancedCount}
                        </Badge>
                    )}
                    <span className={styles.line} />
                    <ChevronDown className={styles.chev} aria-hidden="true" />
                </button>

                {advancedOpen && (
                    <div className="flex flex-col gap-4">
                        <FilterDropdown
                            label={t("filters.archetype")}
                            placeholder={t("filters.archetype.placeholder")}
                            options={options.subclasses}
                            selected={filters.subclasses}
                            onChange={(v) => onChange("subclasses", v)}
                            formatOption={(id) => subProfessionLabel({ subProfessionId: id, subProfessionName: options.labels?.subclasses[id] })}
                            groupBy={subProfessionToProfession}
                            groupOrder={PROFESSION_ORDER}
                            formatGroup={(p) => professionLabel({ profession: p, professionName: options.labels?.classes[p] })}
                            renderOptionIcon={(v) => <SubProfessionIcon subProfession={v} size={18} />}
                        />

                        <div className={styles.field}>
                            <div className={styles.fieldLabel}>{t("filters.gender")}</div>
                            <div className={styles.tagRow}>
                                {GENDERS.map((g) => (
                                    <FilterChip key={g} active={filters.genders.includes(g)} label={fact.gender(g)} onSelect={() => onChange("genders", toggle(filters.genders, g))} />
                                ))}
                            </div>
                        </div>

                        <FilterDropdown
                            label={t("filters.nation")}
                            placeholder={t("filters.nation.placeholder")}
                            options={options.nations}
                            selected={filters.nations}
                            onChange={(v) => onChange("nations", v)}
                            formatOption={(n) => nationLabel({ nationId: n, nationName: options.labels?.nations[n] }) ?? n}
                            renderOptionIcon={(v) => <TeamIcon teamId={v} size={18} />}
                        />

                        <FilterDropdown
                            label={t("filters.faction")}
                            placeholder={t("filters.faction.placeholder")}
                            options={options.factions}
                            selected={filters.factions}
                            onChange={(v) => onChange("factions", v)}
                            formatOption={(id) => options.labels?.factions[id] ?? formatNationId(id)}
                            renderOptionIcon={(v) => <CampIcon groupId={v} size={18} />}
                        />

                        <FilterDropdown label={t("filters.race")} placeholder={t("filters.race.placeholder")} options={options.races} selected={filters.races} onChange={(v) => onChange("races", v)} formatOption={fact.race} />

                        <FilterDropdown label={t("filters.birthPlace")} placeholder={t("filters.birthPlace.placeholder")} options={options.birthPlaces} selected={filters.birthPlaces} onChange={(v) => onChange("birthPlaces", v)} formatOption={fact.birthPlace} />

                        <FilterDropdown label={t("filters.artist")} placeholder={t("filters.artist.placeholder")} options={options.artists} selected={filters.artists} onChange={(v) => onChange("artists", v)} />

                        <FilterDropdown label={t("filters.voiceActor")} placeholder={t("filters.voiceActor.placeholder")} options={options.voiceActors} selected={filters.voiceActors} onChange={(v) => onChange("voiceActors", v)} />
                    </div>
                )}
            </div>
        </div>
    );
}
