import { Lock } from "lucide-react";
import type React from "react";
import { Switch } from "#/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { cn } from "#/lib/utils";
import type { IRandomizerSettings } from "../types";

interface IStageFiltersPanelProps {
    settings: IRandomizerSettings;
    onChange: (next: Partial<IRandomizerSettings>) => void;
    hasProfile: boolean;
}

const ZONE_OPTIONS: Array<{ value: string; label: string; hint: string }> = [
    { value: "MAINLINE", label: "Mainline", hint: "Episodes 1–N" },
    { value: "ACTIVITY", label: "Events", hint: "Side stories, ongoing & permanent" },
];

export function StageFiltersPanel({ settings, onChange, hasProfile }: IStageFiltersPanelProps): React.ReactElement {
    return (
        <div className="flex flex-col gap-5">
            <div className="flex flex-col gap-2.5">
                <p className="font-mono text-[10.5px] uppercase tracking-[0.18em] text-muted-foreground/90">Source</p>
                <ToggleGroup
                    aria-label="Allowed zone types"
                    multiple
                    value={settings.allowedZoneTypes}
                    onValueChange={(next) => onChange({ allowedZoneTypes: (next as string[]) ?? [] })}
                    variant="outline"
                    className="flex-wrap"
                >
                    {ZONE_OPTIONS.map((opt) => (
                        <ToggleGroupItem key={opt.value} value={opt.value} className="h-auto px-3 py-2">
                            <div className="text-left">
                                <p className="text-[12.5px] font-medium leading-none">{opt.label}</p>
                                <p className="mt-1 text-[10.5px] text-muted-foreground">{opt.hint}</p>
                            </div>
                        </ToggleGroupItem>
                    ))}
                </ToggleGroup>
            </div>

            <div className="flex flex-col gap-2.5">
                <p className="font-mono text-[10.5px] uppercase tracking-[0.18em] text-muted-foreground/90">Eligibility</p>
                <SwitchRow label="Only currently available" description="Permanent zones plus events open right now." checked={settings.onlyAvailableStages} onChange={(v) => onChange({ onlyAvailableStages: v })} />
                <SwitchRow label="Only stages I've cleared" description="Restrict to stages with at least one clear in your profile." checked={settings.onlyCompletedStages} onChange={(v) => onChange({ onlyCompletedStages: v })} locked={!hasProfile} />
            </div>
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
