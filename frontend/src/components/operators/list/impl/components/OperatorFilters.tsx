import { ChevronDown, Filter, X } from "lucide-react";
import { useState } from "react";
import { Sheet, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useMediaQuery } from "#/hooks/use-media-query";
import { cn, formatNationId, formatProfession, formatSubProfession, rarityToNumber, subProfessionToProfession } from "#/lib/utils";
import type { OperatorRarityTier } from "#/types/operators";
import { CLASSES, GENDERS, HAS_NOTES_OPTIONS, PROFESSION_ORDER, RARITIES } from "../constants";
import type { AvailabilityFilter, HasNotesFilter, IFilterOptions } from "../types";
import { FilterDropdown } from "./FilterDropdown";
import { CampIcon, ClassIcon, SubProfessionIcon, TeamIcon } from "./Icons";
import styles from "./OperatorFilters.module.css";

interface IOperatorFiltersProps {
    selectedClasses: string[];
    selectedSubclasses: string[];
    selectedRarities: OperatorRarityTier[];
    selectedGenders: string[];
    selectedNations: string[];
    selectedFactions: string[];
    selectedRaces: string[];
    selectedBirthPlaces: string[];
    selectedArtists: string[];
    selectedVoiceActors: string[];
    selectedHasNotes: HasNotesFilter;
    selectedAvailability: AvailabilityFilter;
    options: IFilterOptions;
    onClassesChange: (v: string[]) => void;
    onSubclassesChange: (v: string[]) => void;
    onRaritiesChange: (v: OperatorRarityTier[]) => void;
    onGendersChange: (v: string[]) => void;
    onNationsChange: (v: string[]) => void;
    onFactionsChange: (v: string[]) => void;
    onRacesChange: (v: string[]) => void;
    onBirthPlacesChange: (v: string[]) => void;
    onArtistsChange: (v: string[]) => void;
    onVoiceActorsChange: (v: string[]) => void;
    onHasNotesChange: (v: HasNotesFilter) => void;
    onAvailabilityChange: (v: AvailabilityFilter) => void;
    onClearAll: () => void;
    hasActiveFilters: boolean;
    collapsed?: boolean;
    onToggle?: () => void;
    activeFilterCount?: number;
}

function toggle<T>(list: T[], value: T): T[] {
    return list.includes(value) ? list.filter((v) => v !== value) : [...list, value];
}

const AVAILABILITY_OPTIONS: { value: AvailabilityFilter; label: string }[] = [
    { value: "global", label: "Global" },
    { value: "upcoming", label: "Upcoming (CN)" },
];

function FiltersContent(props: Omit<IOperatorFiltersProps, "collapsed" | "onToggle" | "activeFilterCount">) {
    const [advancedOpen, setAdvancedOpen] = useState(true);

    const advancedCount = props.selectedSubclasses.length + props.selectedGenders.length + props.selectedNations.length + props.selectedFactions.length + props.selectedRaces.length + props.selectedBirthPlaces.length + props.selectedArtists.length + props.selectedVoiceActors.length;

    return (
        <div className="flex flex-col gap-4.5">
            <div className="flex flex-col gap-4">
                <div className={styles.sectionHead}>
                    <span className={styles.lbl}>Basic</span>
                    <span className={styles.line} />
                </div>

                <div className={styles.field}>
                    <div className={styles.fieldLabel}>Availability</div>
                    <div className={styles.tagRow}>
                        {AVAILABILITY_OPTIONS.map((opt) => {
                            const on = props.selectedAvailability === opt.value;
                            return (
                                <button key={opt.value} type="button" className={cn(styles.tg, on && styles.on)} onClick={() => props.onAvailabilityChange(opt.value)} aria-pressed={on}>
                                    {opt.label}
                                </button>
                            );
                        })}
                    </div>
                </div>

                <div className={styles.field}>
                    <div className={styles.fieldLabel}>Class</div>
                    <div className={styles.classRow}>
                        {CLASSES.map((cls) => {
                            const on = props.selectedClasses.includes(cls);
                            return (
                                <Tooltip key={`tooltip-${cls}`}>
                                    <TooltipTrigger
                                        render={
                                            <button key={cls} type="button" title={formatProfession(cls)} className={cn(styles.classBtn, on && styles.on)} onClick={() => props.onClassesChange(toggle(props.selectedClasses, cls))} aria-pressed={on}>
                                                <ClassIcon profession={cls} size={20} />
                                            </button>
                                        }
                                    />
                                    <TooltipPopup side="top" sideOffset={8}>
                                        {formatProfession(cls)}
                                    </TooltipPopup>
                                </Tooltip>
                            );
                        })}
                    </div>
                </div>

                <div className={styles.field}>
                    <div className={styles.fieldLabel}>Rarity</div>
                    <div className={styles.rarityRow}>
                        {RARITIES.map((r) => {
                            const on = props.selectedRarities.includes(r);
                            return (
                                <button key={r} type="button" data-rarity={rarityToNumber(r)} className={cn(styles.rarityBtn, on && styles.on)} onClick={() => props.onRaritiesChange(toggle(props.selectedRarities, r))} aria-pressed={on}>
                                    {rarityToNumber(r)}★
                                </button>
                            );
                        })}
                    </div>
                </div>

                <div className={styles.field}>
                    <div className={styles.fieldLabel}>Notes</div>
                    <div className={styles.tagRow}>
                        {HAS_NOTES_OPTIONS.map((opt) => {
                            const on = props.selectedHasNotes === opt.value;
                            return (
                                <button key={opt.value} type="button" className={cn(styles.tg, on && styles.on)} onClick={() => props.onHasNotesChange(opt.value)} aria-pressed={on}>
                                    {opt.label}
                                </button>
                            );
                        })}
                    </div>
                </div>
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
                            options={props.options.subclasses}
                            selected={props.selectedSubclasses}
                            onChange={props.onSubclassesChange}
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
                                    const on = props.selectedGenders.includes(g);
                                    return (
                                        <button key={g} type="button" className={cn(styles.tg, on && styles.on)} onClick={() => props.onGendersChange(toggle(props.selectedGenders, g))} aria-pressed={on}>
                                            {g}
                                        </button>
                                    );
                                })}
                            </div>
                        </div>

                        <FilterDropdown label="Nation" placeholder="Select nation" options={props.options.nations} selected={props.selectedNations} onChange={props.onNationsChange} formatOption={(n) => formatNationId(n) ?? n} renderOptionIcon={(v) => <TeamIcon teamId={v} size={18} />} />

                        <FilterDropdown label="Faction" placeholder="Select faction" options={props.options.factions} selected={props.selectedFactions} onChange={props.onFactionsChange} formatOption={formatNationId} renderOptionIcon={(v) => <CampIcon groupId={v} size={18} />} />

                        <FilterDropdown label="Race" placeholder="Select race" options={props.options.races} selected={props.selectedRaces} onChange={props.onRacesChange} />

                        <FilterDropdown label="Place of Birth" placeholder="Select birth place" options={props.options.birthPlaces} selected={props.selectedBirthPlaces} onChange={props.onBirthPlacesChange} />

                        <FilterDropdown label="Artist" placeholder="Select artist" options={props.options.artists} selected={props.selectedArtists} onChange={props.onArtistsChange} />

                        <FilterDropdown label="Voice Actor" placeholder="Select voice actor" options={props.options.voiceActors} selected={props.selectedVoiceActors} onChange={props.onVoiceActorsChange} />
                    </div>
                )}
            </div>
        </div>
    );
}

export function OperatorFilters(props: IOperatorFiltersProps) {
    const isMobile = useMediaQuery("max-md");
    const isOpen = !props.collapsed;

    if (isMobile) {
        return (
            <Sheet open={isOpen} onOpenChange={(open) => !open && props.onToggle?.()}>
                <SheetPopup side="left" variant="inset" className="max-w-80">
                    <SheetHeader className="pb-4">
                        <div className="flex items-center justify-between">
                            <SheetTitle className="flex items-center gap-2 text-base">
                                <Filter className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
                                Filters
                                {props.activeFilterCount && props.activeFilterCount > 0 ? <span className="inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-primary px-1.5 font-mono font-semibold text-[10px] text-primary-foreground">{props.activeFilterCount}</span> : null}
                            </SheetTitle>
                            {props.hasActiveFilters && (
                                <button type="button" className="inline-flex items-center gap-1 font-medium text-[11.5px] text-muted-foreground hover:text-foreground" onClick={props.onClearAll}>
                                    <X className="h-2.5 w-2.5" aria-hidden="true" />
                                    Clear all
                                </button>
                            )}
                        </div>
                    </SheetHeader>
                    <SheetPanel className="px-5 pb-6">
                        <FiltersContent {...props} />
                    </SheetPanel>
                </SheetPopup>
            </Sheet>
        );
    }

    return (
        <aside className={cn(styles.filterSidebar, props.collapsed && styles.filterSidebarCollapsed)} aria-label="Operator filters" aria-hidden={props.collapsed || undefined} {...(props.collapsed ? { inert: "" as unknown as boolean } : {})}>
            <div className={styles.filterSidebarInner}>
                <div className={styles.fpHead}>
                    <h3>
                        <Filter className="h-3.5 w-3.5" aria-hidden="true" />
                        Filters
                    </h3>
                    {props.hasActiveFilters && (
                        <button type="button" className={styles.clear} onClick={props.onClearAll}>
                            <X className="h-2.5 w-2.5" aria-hidden="true" />
                            Clear all
                        </button>
                    )}
                </div>
                <FiltersContent {...props} />
            </div>
        </aside>
    );
}
