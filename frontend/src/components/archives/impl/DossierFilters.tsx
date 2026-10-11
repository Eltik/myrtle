import type * as React from "react";
import { ClassPicker, toggle } from "#/components/operators/list/impl/components/ClassPicker";
import { FilterDropdown } from "#/components/operators/list/impl/components/FilterDropdown";
import { SubProfessionIcon } from "#/components/operators/list/impl/components/Icons";
import type { messages as fieldMessages } from "#/components/operators/list/impl/components/OperatorFilterFields.messages";
import styles from "#/components/operators/list/impl/components/OperatorFilters.module.css";
import { PROFESSION_ORDER, RARITIES } from "#/components/operators/list/impl/constants";
import { Slider } from "#/components/ui/slider";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, professionLabel, rarityToNumber, subProfessionToProfession } from "#/lib/utils";
import { type DossierListKey, type IDossierFilters, type IDossierOptions, INFECTION_ORDER, type IOption, type Range } from "./dossierMatch";
import { type ArchivesT, FactionLogo, INFECTION_KEY } from "./shared";

const STARS = RARITIES.map((r) => rarityToNumber(r));

interface IDossierFilterFieldsProps {
    filters: IDossierFilters;
    options: IDossierOptions;
    /** Rows per infection state, over every dossier. */
    infectionCounts: Map<string, number>;
    onChange: <K extends keyof IDossierFilters>(key: K, value: IDossierFilters[K]) => void;
    t: ArchivesT;
}

/**
 * The Dossiers filter fields, built from the /operators sidebar's own parts
 * (class picker, rarity buttons, multi-select dropdowns, tag rows) so the two
 * pages read and behave alike. A field whose values no row carries is left
 * out rather than shown empty.
 */
export function DossierFilterFields({ filters, options, infectionCounts, onChange, t }: IDossierFilterFieldsProps): React.ReactElement {
    const ft: TypedT<typeof fieldMessages> = useT("operators");
    const labelOf = (list: IOption[]) => {
        const m = new Map(list.map((o) => [o.value, o.label]));
        return (v: string) => m.get(v) ?? v;
    };
    const dropdown = (key: DossierListKey & ("subclasses" | "nations" | "factions" | "races" | "birthplaces"), label: string, placeholder: string, list: IOption[], extra?: Partial<React.ComponentProps<typeof FilterDropdown>>) =>
        list.length === 0 ? null : <FilterDropdown label={label} placeholder={placeholder} options={list.map((o) => o.value)} selected={filters[key]} onChange={(v) => onChange(key, v)} formatOption={labelOf(list)} {...extra} />;

    return (
        <div className="flex flex-col gap-4.5">
            <div className="flex flex-col gap-4">
                <SectionHead label={ft("filters.section.basic")} />
                <ClassPicker selected={filters.classes} onChange={(v) => onChange("classes", v)} />
                <div className={styles.field}>
                    <div className={styles.fieldLabel}>{ft("filters.rarity")}</div>
                    <div className={styles.rarityRow}>
                        {STARS.map((r) => {
                            const on = filters.rarities.includes(r);
                            return (
                                <button key={r} type="button" data-rarity={r} className={cn(styles.rarityBtn, on && styles.on)} onClick={() => onChange("rarities", toggle(filters.rarities, r))} aria-pressed={on}>
                                    {r}★
                                </button>
                            );
                        })}
                    </div>
                </div>
                <MultiTagRow label={t("dossiers.filter.infection")} options={INFECTION_ORDER.map((s) => ({ value: s, label: t(INFECTION_KEY[s]), count: infectionCounts.get(s) ?? 0 })).filter((o) => o.count > 0)} selected={filters.infection} onChange={(v) => onChange("infection", v as IDossierFilters["infection"])} />
                {options.genders.length > 0 ? <MultiTagRow label={ft("filters.gender")} options={options.genders} selected={filters.genders} onChange={(v) => onChange("genders", v)} /> : null}
            </div>

            <div className="flex flex-col gap-4">
                <SectionHead label={t("dossiers.filter.profile")} />
                <RangeField label={t("dossiers.filter.height")} extent={options.height} value={filters.height} onChange={(v) => onChange("height", v)} format={(lo, hi) => t("dossiers.filter.rangeCm", { min: lo, max: hi })} />
                <RangeField label={t("dossiers.filter.experience")} extent={options.experience} value={filters.experience} onChange={(v) => onChange("experience", v)} format={(lo, hi) => t("dossiers.filter.rangeYears", { min: lo, max: hi })} step={0.5} />
                {dropdown("races", ft("filters.race"), ft("filters.race.placeholder"), options.races)}
                {dropdown("birthplaces", ft("filters.birthPlace"), ft("filters.birthPlace.placeholder"), options.birthplaces)}
            </div>

            <div className="flex flex-col gap-4">
                <SectionHead label={ft("filters.section.advanced")} />
                {dropdown("subclasses", ft("filters.archetype"), ft("filters.archetype.placeholder"), options.subclasses, {
                    groupBy: subProfessionToProfession,
                    groupOrder: PROFESSION_ORDER,
                    formatGroup: (p) => professionLabel({ profession: p }),
                    renderOptionIcon: (v) => <SubProfessionIcon subProfession={v} size={18} />,
                })}
                {dropdown("nations", ft("filters.nation"), ft("filters.nation.placeholder"), options.nations, { renderOptionIcon: (v) => <FactionLogo id={v} className="size-4.5 text-foreground" /> })}
                {dropdown("factions", ft("filters.faction"), ft("filters.faction.placeholder"), options.factions, { renderOptionIcon: (v) => <FactionLogo id={v} className="size-4.5 text-foreground" /> })}
            </div>
        </div>
    );
}

function SectionHead({ label }: { label: string }): React.ReactElement {
    return (
        <div className={styles.sectionHead}>
            <span className={styles.lbl}>{label}</span>
            <span className={styles.line} />
        </div>
    );
}

/** The /operators tag row, any number on at once, each tag with its row count. */
function MultiTagRow({ label, options, selected, onChange }: { label: string; options: readonly IOption[]; selected: readonly string[]; onChange: (v: string[]) => void }): React.ReactElement {
    const f = useFormatters();
    return (
        <div className={styles.field}>
            <div className={styles.fieldLabel}>{label}</div>
            <div className={styles.tagRow}>
                {options.map((o) => {
                    const on = selected.includes(o.value);
                    return (
                        <button key={o.value} type="button" className={cn(styles.tg, on && styles.on)} onClick={() => onChange(toggle([...selected], o.value))} aria-pressed={on}>
                            {o.label}
                            <span className="ml-1.5 font-mono text-[10px] tabular-nums opacity-70">{f.number(o.count)}</span>
                        </button>
                    );
                })}
            </div>
        </div>
    );
}

/** A two-thumb range over the rows' own extremes; the full extent is no filter. */
function RangeField({ label, extent, value, onChange, format, step = 1 }: { label: string; extent: Range; value: Range; onChange: (v: Range) => void; format: (lo: number, hi: number) => string; step?: number }): React.ReactElement | null {
    if (!extent) return null;
    const [lo, hi] = value ?? extent;
    return (
        <div className={styles.field}>
            <div className={cn(styles.fieldLabel, "flex items-center justify-between gap-2")}>
                <span>{label}</span>
                <span className={cn("font-mono tabular-nums", value ? "text-primary" : "")}>{format(lo, hi)}</span>
            </div>
            <Slider
                className="px-1 pt-1"
                min={extent[0]}
                max={extent[1]}
                step={step}
                value={[lo, hi]}
                aria-label={label}
                onValueChange={(next) => {
                    if (!Array.isArray(next)) return;
                    const [a, b] = next as readonly number[];
                    onChange(a <= extent[0] && b >= extent[1] ? null : [a, b]);
                }}
            />
        </div>
    );
}
