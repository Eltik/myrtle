import type React from "react";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { Slider } from "#/components/ui/slider";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { OperatorProfession, OperatorRarity } from "#/types/operators";
import { ALL_CLASSES, ALL_RARITIES, CLASS_LABEL } from "../constants";
import type { IRandomizerSettings } from "../types";
import { FieldGroup, SwitchRow } from "./FilterControls";
import type { messages } from "./OperatorFiltersPanel.messages";

interface IOperatorFiltersPanelProps {
    settings: IRandomizerSettings;
    onChange: (next: Partial<IRandomizerSettings>) => void;
    hasProfile: boolean;
}

export function OperatorFiltersPanel({ settings, onChange, hasProfile }: IOperatorFiltersPanelProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    return (
        <div className="flex flex-col gap-5">
            <FieldGroup label={t("randomizer.ops.class")}>
                <ToggleGroup aria-label={t("randomizer.ops.class.aria")} multiple value={settings.allowedClasses} onValueChange={(next) => onChange({ allowedClasses: (next as OperatorProfession[]) ?? [] })} variant="outline" size="lg" className="flex-wrap">
                    {ALL_CLASSES.map((cls) => (
                        <ToggleGroupItem key={cls} value={cls} aria-label={CLASS_LABEL[cls]} title={CLASS_LABEL[cls]} className="px-2 [&:not([data-pressed])]:bg-input/64 [&:not([data-pressed])]:before:shadow-none! dark:[&:not([data-pressed])]:bg-input [&:not([data-pressed])_img]:opacity-35">
                            <ClassIcon profession={cls} size={24} className="size-6!" />
                        </ToggleGroupItem>
                    ))}
                </ToggleGroup>
            </FieldGroup>

            <FieldGroup label={t("randomizer.ops.rarity")}>
                <ToggleGroup aria-label={t("randomizer.ops.rarity.aria")} multiple value={settings.allowedRarities.map(String)} onValueChange={(next) => onChange({ allowedRarities: (next as string[]).map((n) => Number(n) as OperatorRarity) })} variant="outline">
                    {ALL_RARITIES.map((r) => (
                        <ToggleGroupItem key={r} value={String(r)} aria-label={t("randomizer.ops.rarityChip.aria", { rarity: r })} className="[&:not([data-pressed])>span]:opacity-35 [&:not([data-pressed])]:bg-input/64 [&:not([data-pressed])]:before:shadow-none! dark:[&:not([data-pressed])]:bg-input">
                            <span className="font-mono text-[12px]">{t("randomizer.ops.rarityChip", { rarity: r })}</span>
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
            </FieldGroup>
        </div>
    );
}
