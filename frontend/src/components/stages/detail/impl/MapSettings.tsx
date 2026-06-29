import { Footprints, type LucideIcon, PersonStanding, Skull, Spline, Timer } from "lucide-react";
import { Switch } from "#/components/ui/switch";
import { cn } from "#/lib/utils";
import { Kicker } from "./primitives";

export interface IMapSettings {
    /** Draw the enemy route lines. */
    showRoutes: boolean;
    /** Show the enemy's icon on its route. */
    showEnemyIcons: boolean;
    /** Animate the enemy icon walking along its route (vs. parked at its spawn). */
    showMovement: boolean;
    /** Render the wait-timer badges on routes. */
    showTimers: boolean;
    /** In 3D view, render the enemy's animated chibi walking the route. */
    walkingChibis: boolean;
}

export const DEFAULT_MAP_SETTINGS: IMapSettings = {
    showRoutes: true,
    showEnemyIcons: false,
    showMovement: false,
    showTimers: true,
    walkingChibis: true,
};

interface ISettingDef {
    key: keyof IMapSettings;
    icon: LucideIcon;
    label: string;
    description: string;
    /** Other setting that must be on for this one to apply. */
    requires?: keyof IMapSettings;
}

const SETTINGS: ISettingDef[] = [
    { key: "showRoutes", icon: Spline, label: "Route Lines", description: "Draw the paths enemies follow across the map." },
    { key: "showEnemyIcons", icon: Skull, label: "Enemy Icons", description: "Show each spawning enemy's icon on its route." },
    { key: "showMovement", icon: Footprints, label: "Enemy Movement", description: "Animate the icon along its route instead of parking it at the spawn.", requires: "showEnemyIcons" },
    { key: "showTimers", icon: Timer, label: "Wait Timers", description: "Show how long enemies pause at points on their route." },
    { key: "walkingChibis", icon: PersonStanding, label: "Walking Chibis", description: "In 3D view, replace the icon with the enemy's animated chibi walking the route." },
];

function SettingRow({ def, checked, disabled, onChange }: { def: ISettingDef; checked: boolean; disabled: boolean; onChange: (v: boolean) => void }) {
    const id = `map-setting-${def.key}`;
    const Icon = def.icon;
    return (
        <label htmlFor={id} className={cn("flex items-center gap-3 py-1.5", disabled ? "cursor-not-allowed opacity-50" : "cursor-pointer")}>
            <span aria-hidden="true" className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md border border-border bg-[color-mix(in_oklch,var(--muted)_40%,transparent)] text-muted-foreground [&_svg]:h-3.5 [&_svg]:w-3.5">
                <Icon />
            </span>
            <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                <span className="font-medium font-sans text-[12.5px] text-foreground leading-none">{def.label}</span>
                <span className="font-sans text-[11px] text-muted-foreground leading-tight">{def.description}</span>
            </span>
            <Switch id={id} checked={checked} disabled={disabled} onCheckedChange={onChange} />
        </label>
    );
}

export function MapSettings({ settings, onChange }: { settings: IMapSettings; onChange: (next: IMapSettings) => void }) {
    return (
        <div className="flex flex-col rounded-[10px] border border-border bg-card px-3.5 py-2.5">
            <div className="mb-1 flex items-center">
                <Kicker>Settings</Kicker>
            </div>
            <div className="grid gap-x-6 sm:grid-cols-2">
                {SETTINGS.map((def) => (
                    <SettingRow
                        key={def.key}
                        def={def}
                        checked={settings[def.key]}
                        disabled={def.requires ? !settings[def.requires] : false}
                        onChange={(v) => {
                            const next = { ...settings, [def.key]: v };
                            if (def.key === "walkingChibis" && v) next.showEnemyIcons = false;
                            onChange(next);
                        }}
                    />
                ))}
            </div>
        </div>
    );
}
