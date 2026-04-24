import { ChevronDown, Filter, X } from "lucide-react";
import { useState } from "react";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { cn, formatNationId, formatProfession, formatSubProfession, rarityToNumber, subProfessionToProfession } from "#/lib/utils";
import type { OperatorRarityTier } from "#/types/operators";
import { CLASSES, GENDERS, PROFESSION_ORDER, RARITIES } from "../constants";
import type { IFilterOptions } from "../types";
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
    onClearAll: () => void;
    hasActiveFilters: boolean;
    collapsed?: boolean;
}

function toggle<T>(list: T[], value: T): T[] {
    return list.includes(value) ? list.filter((v) => v !== value) : [...list, value];
}

export function OperatorFilters(props: IOperatorFiltersProps) {
    const [advancedOpen, setAdvancedOpen] = useState(true);

    const advancedCount = props.selectedSubclasses.length + props.selectedGenders.length + props.selectedNations.length + props.selectedFactions.length + props.selectedRaces.length + props.selectedBirthPlaces.length + props.selectedArtists.length + props.selectedVoiceActors.length;

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

                <div className="flex flex-col gap-4">
                    <div className={styles.sectionHead}>
                        <span className={styles.lbl}>Basic</span>
                        <span className={styles.line} />
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
        </aside>
    );
}
