import { Lock } from "lucide-react";
import type React from "react";
import { Slider } from "#/components/ui/slider";
import { Switch } from "#/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { cn } from "#/lib/utils";
import type { OperatorProfession, OperatorRarity } from "#/types/operators";
import { ALL_CLASSES, ALL_RARITIES, CLASS_LABEL } from "../constants";
import type { IRandomizerSettings } from "../types";

interface IOperatorFiltersPanelProps {
    settings: IRandomizerSettings;
    onChange: (next: Partial<IRandomizerSettings>) => void;
    hasProfile: boolean;
}

export function OperatorFiltersPanel({ settings, onChange, hasProfile }: IOperatorFiltersPanelProps): React.ReactElement {
    return (
        <div className="flex flex-col gap-5">
            <FieldGroup label="Class">
                <ToggleGroup
                    aria-label="Allowed classes"
                    multiple
                    value={settings.allowedClasses}
                    onValueChange={(next) => onChange({ allowedClasses: (next as OperatorProfession[]) ?? [] })}
                    variant="outline"
                    className="flex-wrap"
                >
                    {ALL_CLASSES.map((cls) => (
                        <ToggleGroupItem key={cls} value={cls} aria-label={CLASS_LABEL[cls]}>
                            <span className="text-[12px]">{CLASS_LABEL[cls]}</span>
                        </ToggleGroupItem>
                    ))}
                </ToggleGroup>
            </FieldGroup>

            <FieldGroup label="Rarity">
                <ToggleGroup
                    aria-label="Allowed rarities"
                    multiple
                    value={settings.allowedRarities.map(String)}
                    onValueChange={(next) => onChange({ allowedRarities: (next as string[]).map((n) => Number(n) as OperatorRarity) })}
                    variant="outline"
                >
                    {ALL_RARITIES.map((r) => (
                        <ToggleGroupItem key={r} value={String(r)} aria-label={`${r} star`}>
                            <span className="font-mono text-[12px]">{r}★</span>
                        </ToggleGroupItem>
                    ))}
                </ToggleGroup>
            </FieldGroup>

            <FieldGroup label={`Squad size · ${settings.squadSize}`}>
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

            <FieldGroup label="Rules">
                <SwitchRow label="Allow duplicates" description="Same operator can appear twice in a squad." checked={settings.allowDuplicates} onChange={(v) => onChange({ allowDuplicates: v })} />
                <SwitchRow label="Hide unplayable operators" description="Exclude tokens, support-only, and reserve operators." checked={settings.hideUnplayableOperators} onChange={(v) => onChange({ hideUnplayableOperators: v })} />
                <SwitchRow
                    label="Only operators I own"
                    description="Restrict to your roster from the linked profile."
                    checked={settings.onlyOwnedOperators}
                    onChange={(v) => onChange({ onlyOwnedOperators: v, onlyE2Operators: v ? settings.onlyE2Operators : false })}
                    locked={!hasProfile}
                />
                <SwitchRow
                    label="E2 only"
                    description="Restrict further to operators at elite 2 in your roster."
                    checked={settings.onlyE2Operators}
                    onChange={(v) => onChange({ onlyE2Operators: v, onlyOwnedOperators: v ? true : settings.onlyOwnedOperators })}
                    locked={!hasProfile}
                />
            </FieldGroup>
        </div>
    );
}

function FieldGroup({ label, children }: { label: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex flex-col gap-2.5">
            <p className="font-mono text-[10.5px] uppercase tracking-[0.18em] text-muted-foreground/90">{label}</p>
            {children}
        </div>
    );
}

function SwitchRow({ label, description, checked, onChange, locked = false }: { label: string; description: string; checked: boolean; onChange: (v: boolean) => void; locked?: boolean }) {
    return (
        <label className={cn("flex items-start justify-between gap-3 rounded-md border border-border/50 bg-card/60 px-3 py-2.5 transition-colors hover:bg-accent/30", locked && "cursor-not-allowed opacity-60 hover:bg-card/60")}>
            <div className="min-w-0 flex-1">
                <p className="flex items-center gap-1.5 text-[12.5px] font-medium text-foreground">
                    {label}
                    {locked && <Lock aria-hidden="true" className="h-3 w-3 text-muted-foreground/70" />}
                </p>
                <p className="mt-0.5 text-[11.5px] leading-snug text-muted-foreground">{description}</p>
            </div>
            <Switch checked={locked ? false : checked} disabled={locked} onCheckedChange={onChange} />
        </label>
    );
}
