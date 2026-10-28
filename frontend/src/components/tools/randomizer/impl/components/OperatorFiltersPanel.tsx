import type React from "react";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Slider } from "#/components/ui/slider";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { ACQUISITION_FILTERS, type AcquisitionFilter } from "#/lib/operators/acquisition";
import type { OperatorProfession, OperatorRarity } from "#/types/operators";
import { ALL_CLASSES, ALL_RARITIES, CLASS_LABEL } from "../constants";
import { type ITierFilterState, UNRANKED_TIER_ID } from "../tier-filter";
import type { IRandomizerSettings } from "../types";
import { FieldGroup, SwitchRow } from "./FilterControls";
import type { messages } from "./OperatorFiltersPanel.messages";

interface IOperatorFiltersPanelProps {
    settings: IRandomizerSettings;
    onChange: (next: Partial<IRandomizerSettings>) => void;
    hasProfile: boolean;
    tierFilter: ITierFilterState;
}

/** Select value for "no tier list"; slugs never start with an underscore. */
const NO_TIER_LIST = "__none";

const CHIP_CLASS = "[&:not([data-pressed])>span]:opacity-35 [&:not([data-pressed])]:bg-input/64 [&:not([data-pressed])]:before:shadow-none! dark:[&:not([data-pressed])]:bg-input";

export function OperatorFiltersPanel({ settings, onChange, hasProfile, tierFilter }: IOperatorFiltersPanelProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    return (
        <div className="flex flex-col gap-5">
            <FieldGroup label={t("randomizer.ops.class")}>
                <ToggleGroup aria-label={t("randomizer.ops.class.aria")} multiple value={settings.allowedClasses} onValueChange={(next) => onChange({ allowedClasses: (next as OperatorProfession[]) ?? [] })} variant="outline" size="lg" className="flex-wrap">
                    {ALL_CLASSES.map((cls) => (
                        <Tooltip key={cls}>
                            <TooltipTrigger render={<ToggleGroupItem value={cls} aria-label={CLASS_LABEL[cls]} className="px-2 [&:not([data-pressed])]:bg-input/64 [&:not([data-pressed])]:before:shadow-none! dark:[&:not([data-pressed])]:bg-input [&:not([data-pressed])_img]:opacity-35" />}>
                                <ClassIcon profession={cls} size={24} className="size-6!" />
                            </TooltipTrigger>
                            <TooltipPopup>{CLASS_LABEL[cls]}</TooltipPopup>
                        </Tooltip>
                    ))}
                </ToggleGroup>
            </FieldGroup>

            <FieldGroup label={t("randomizer.ops.rarity")}>
                <ToggleGroup aria-label={t("randomizer.ops.rarity.aria")} multiple value={settings.allowedRarities.map(String)} onValueChange={(next) => onChange({ allowedRarities: (next as string[]).map((n) => Number(n) as OperatorRarity) })} variant="outline">
                    {ALL_RARITIES.map((r) => (
                        <ToggleGroupItem key={r} value={String(r)} aria-label={t("randomizer.ops.rarityChip.aria", { rarity: r })} className={CHIP_CLASS}>
                            <span className="font-mono text-[12px]">{t("randomizer.ops.rarityChip", { rarity: r })}</span>
                        </ToggleGroupItem>
                    ))}
                </ToggleGroup>
            </FieldGroup>

            <FieldGroup label={t("randomizer.ops.source")}>
                {/* One source at a time; pressing the active one again keeps it rather than leaving none. */}
                <ToggleGroup aria-label={t("randomizer.ops.source.aria")} value={[settings.acquisition]} onValueChange={(next) => next[0] && onChange({ acquisition: next[0] as AcquisitionFilter })} variant="outline" className="flex-wrap">
                    {ACQUISITION_FILTERS.map((f) => (
                        <ToggleGroupItem key={f} value={f} className={CHIP_CLASS}>
                            <span className="text-[12px]">{sourceLabel(t, f)}</span>
                        </ToggleGroupItem>
                    ))}
                </ToggleGroup>
            </FieldGroup>

            <FieldGroup label={t("randomizer.ops.squadSize", { size: settings.squadSize })}>
                <div className="flex items-center gap-3">
                    <Slider
                        min={1}
                        max={12}
                        step={1}
                        value={[settings.squadSize]}
                        onValueChange={(value) => {
                            const n = Array.isArray(value) ? value[0] : value;
                            if (typeof n === "number") onChange({ squadSize: n });
                        }}
                    />
                    <span className="w-8 shrink-0 text-right font-mono text-[12px] text-muted-foreground">{settings.squadSize}</span>
                </div>
            </FieldGroup>

            <FieldGroup label={t("randomizer.ops.rules")}>
                <SwitchRow label={t("randomizer.ops.allowDuplicates")} description={t("randomizer.ops.allowDuplicates.desc")} checked={settings.allowDuplicates} onChange={(v) => onChange({ allowDuplicates: v })} />
                <SwitchRow label={t("randomizer.ops.hideUnplayable")} description={t("randomizer.ops.hideUnplayable.desc")} checked={settings.hideUnplayableOperators} onChange={(v) => onChange({ hideUnplayableOperators: v })} />
                <SwitchRow label={t("randomizer.ops.onlyOwned")} description={t("randomizer.ops.onlyOwned.desc")} checked={settings.onlyOwnedOperators} onChange={(v) => onChange({ onlyOwnedOperators: v, onlyE2Operators: v ? settings.onlyE2Operators : false })} locked={!hasProfile} />
                <SwitchRow label={t("randomizer.ops.onlyE2")} description={t("randomizer.ops.onlyE2.desc")} checked={settings.onlyE2Operators} onChange={(v) => onChange({ onlyE2Operators: v, onlyOwnedOperators: v ? true : settings.onlyOwnedOperators })} locked={!hasProfile} />
                {settings.onlyE2Operators && <SwitchRow label={t("randomizer.ops.e2SixStarOnly")} description={t("randomizer.ops.e2SixStarOnly.desc")} checked={settings.e2SixStarOnly} onChange={(v) => onChange({ e2SixStarOnly: v })} locked={!hasProfile} />}
            </FieldGroup>

            <TierListField settings={settings} onChange={onChange} tierFilter={tierFilter} />
        </div>
    );
}

function sourceLabel(t: TypedT<typeof messages>, filter: AcquisitionFilter): string {
    const labels: Record<AcquisitionFilter, string> = { any: t("randomizer.ops.source.any"), gachaless: t("randomizer.ops.source.gachaless"), welfare: t("randomizer.ops.source.welfare"), recruitment: t("randomizer.ops.source.recruitment") };
    return labels[filter];
}

function TierListField({ settings, onChange, tierFilter }: Pick<IOperatorFiltersPanelProps, "settings" | "onChange" | "tierFilter">): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    const { choices, slug, index, loading } = tierFilter;
    const chips = index ? [...index.tiers, { id: UNRANKED_TIER_ID, name: t("randomizer.ops.tierList.unranked") }] : [];
    const selected = chips.filter((c) => !settings.deselectedTierIds.includes(c.id)).map((c) => c.id);

    return (
        <FieldGroup label={t("randomizer.ops.tierList")}>
            <div className="flex flex-col gap-3 rounded-md border border-border/50 bg-card/60 px-3 py-2.5">
                <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between sm:gap-3">
                    <div className="min-w-0">
                        <p className="font-medium text-[12.5px] text-foreground">{t("randomizer.ops.tierList.title")}</p>
                        <p className="mt-0.5 text-[11.5px] text-muted-foreground leading-snug">{t("randomizer.ops.tierList.desc")}</p>
                    </div>
                    <Select value={slug ?? NO_TIER_LIST} onValueChange={(v) => onChange({ tierListSlug: !v || v === NO_TIER_LIST ? null : v, deselectedTierIds: [] })} disabled={choices.length === 0}>
                        <SelectTrigger size="sm" className="w-full sm:w-48 sm:shrink-0" aria-label={t("randomizer.ops.tierList.title")}>
                            <SelectValue>{(v: string | null) => (!v || v === NO_TIER_LIST ? t("randomizer.ops.tierList.none") : (choices.find((c) => c.slug === v)?.title ?? v))}</SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value={NO_TIER_LIST}>{t("randomizer.ops.tierList.none")}</SelectItem>
                            {choices.map((c) => (
                                <SelectItem key={c.slug} value={c.slug}>
                                    {c.title}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
                {loading && <p className="text-[11.5px] text-muted-foreground">{t("randomizer.ops.tierList.loading")}</p>}
                {chips.length > 0 && (
                    <ToggleGroup
                        aria-label={t("randomizer.ops.tierList.aria")}
                        multiple
                        value={selected}
                        // Keep at least one tier: an empty pick would draw nothing behind a filter that reads as set.
                        onValueChange={(next) => {
                            const keep = next as string[];
                            if (keep.length === 0) return;
                            onChange({ deselectedTierIds: chips.filter((c) => !keep.includes(c.id)).map((c) => c.id) });
                        }}
                        variant="outline"
                        size="sm"
                        className="flex-wrap"
                    >
                        {chips.map((c) => (
                            <ToggleGroupItem key={c.id} value={c.id} className={CHIP_CLASS}>
                                <span className="text-[12px]">{c.name}</span>
                            </ToggleGroupItem>
                        ))}
                    </ToggleGroup>
                )}
            </div>
        </FieldGroup>
    );
}
