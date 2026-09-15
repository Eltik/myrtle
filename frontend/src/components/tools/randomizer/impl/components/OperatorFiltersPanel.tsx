import { Lock } from "lucide-react";
import type React from "react";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { Slider } from "#/components/ui/slider";
import { Switch } from "#/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { OperatorProfession, OperatorRarity } from "#/types/operators";
import { ALL_CLASSES, ALL_RARITIES, CLASS_LABEL } from "../constants";
import type { IRandomizerSettings } from "../types";
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

function FieldGroup({ label, children }: { label: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex flex-col gap-2.5">
            <p className="font-mono text-[10.5px] text-muted-foreground/90 uppercase tracking-[0.18em]">{label}</p>
            {children}
        </div>
    );
}

function SwitchRow({ label, description, checked, onChange, locked = false }: { label: string; description: string; checked: boolean; onChange: (v: boolean) => void; locked?: boolean }) {
    return (
        // biome-ignore lint/a11y/noLabelWithoutControl: Switch is a Base UI primitive; wrapping label provides click target and is correctly associated at runtime
        <label className={cn("flex items-start justify-between gap-3 rounded-md border border-border/50 bg-card/60 px-3 py-2.5 transition-colors hover:bg-accent/30", locked && "cursor-not-allowed opacity-60 hover:bg-card/60")}>
            <div className="min-w-0 flex-1">
                <p className="flex items-center gap-1.5 font-medium text-[12.5px] text-foreground">
                    {label}
                    {locked && <Lock aria-hidden="true" className="h-3 w-3 text-muted-foreground/70" />}
                </p>
                <p className="mt-0.5 text-[11.5px] text-muted-foreground leading-snug">{description}</p>
            </div>
            <Switch checked={locked ? false : checked} disabled={locked} onCheckedChange={onChange} />
        </label>
    );
}
