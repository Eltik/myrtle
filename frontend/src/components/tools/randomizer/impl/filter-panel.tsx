"use client";

import { Filter } from "lucide-react";
import { CLASS_DISPLAY, CLASSES, RARITIES, RARITY_COLORS } from "~/components/operators/list/constants";
import { ClassIcon } from "~/components/operators/list/ui/impl/class-icon";
import { Label } from "~/components/ui/shadcn/label";
import { Slider } from "~/components/ui/shadcn/slider";
import { Switch } from "~/components/ui/shadcn/switch";
import type { RandomizerSettings } from "./types";

interface FilterPanelProps {
    settings: RandomizerSettings;
    setSettings: (settings: RandomizerSettings) => void;
    hasProfile?: boolean;
    onFiltersChanged?: () => void;
    onE2Disabled?: () => void;
    onE2Enabled?: () => void;
}

export function FilterPanel({ settings, setSettings, hasProfile, onFiltersChanged, onE2Disabled, onE2Enabled }: FilterPanelProps) {
    const handleClassToggle = (profession: string) => {
        const newClasses = settings.allowedClasses.includes(profession) ? settings.allowedClasses.filter((c) => c !== profession) : [...settings.allowedClasses, profession];
        setSettings({ ...settings, allowedClasses: newClasses });
        onFiltersChanged?.();
    };

    const handleRarityToggle = (rarity: number) => {
        const newRarities = settings.allowedRarities.includes(rarity) ? settings.allowedRarities.filter((r) => r !== rarity) : [...settings.allowedRarities, rarity];
        setSettings({ ...settings, allowedRarities: newRarities });
        onFiltersChanged?.();
    };

    const handleSquadSizeChange = (value: number[]) => {
        setSettings({ ...settings, squadSize: value[0] ?? 12 });
        onFiltersChanged?.();
    };

    const handleDuplicatesToggle = (checked: boolean) => {
        setSettings({ ...settings, allowDuplicates: checked });
        onFiltersChanged?.();
    };

    const handleSelectAllClasses = () => {
        setSettings({ ...settings, allowedClasses: [...CLASSES] });
        onFiltersChanged?.();
    };

    const handleDeselectAllClasses = () => {
        setSettings({ ...settings, allowedClasses: [] });
        onFiltersChanged?.();
    };

    const handleSelectAllRarities = () => {
        setSettings({ ...settings, allowedRarities: [...RARITIES] });
        onFiltersChanged?.();
    };

    const handleDeselectAllRarities = () => {
        setSettings({ ...settings, allowedRarities: [] });
        onFiltersChanged?.();
    };

    const handleE2Toggle = (checked: boolean) => {
        setSettings({ ...settings, onlyE2Operators: checked });
        onFiltersChanged?.();
        if (checked) {
            // When E2 filter is enabled, filter roster to only E2 operators
            onE2Enabled?.();
        } else {
            // When E2 filter is disabled, auto-import profile operators
            onE2Disabled?.();
        }
    };

    return (
        <div className="space-y-4 rounded-xl border border-border bg-card/30 p-4 backdrop-blur-sm sm:p-6">
            <div className="flex items-center gap-2">
                <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/10">
                    <Filter className="h-4 w-4 text-primary" />
                </div>
                <h2 className="font-semibold text-foreground">Filters</h2>
            </div>

            {/* Classes Filter */}
            <div className="space-y-3">
                <div className="flex items-center justify-between">
                    <Label className="font-medium text-foreground text-sm">Classes</Label>
                    <div className="flex gap-2 text-xs">
                        <button className="text-muted-foreground transition-colors hover:text-foreground" onClick={handleSelectAllClasses} type="button">
                            All
                        </button>
                        <span className="text-border">•</span>
                        <button className="text-muted-foreground transition-colors hover:text-foreground" onClick={handleDeselectAllClasses} type="button">
                            None
                        </button>
                    </div>
                </div>
                <div className="grid grid-cols-4 gap-2">
                    {CLASSES.map((profession) => {
                        const isSelected = settings.allowedClasses.includes(profession);
                        const displayName = CLASS_DISPLAY[profession] ?? profession;

                        return (
                            <button className={`flex flex-col items-center gap-1 rounded-lg border p-2 transition-all ${isSelected ? "border-primary bg-primary/10" : "border-border bg-secondary/50 hover:border-border/80"}`} key={profession} onClick={() => handleClassToggle(profession)} type="button">
                                <ClassIcon profession={profession} size={24} />
                                <span className="text-foreground text-xs">{displayName}</span>
                            </button>
                        );
                    })}
                </div>
            </div>

            {/* Rarities Filter */}
            <div className="space-y-3">
                <div className="flex items-center justify-between">
                    <Label className="font-medium text-foreground text-sm">Rarities</Label>
                    <div className="flex gap-2 text-xs">
                        <button className="text-muted-foreground transition-colors hover:text-foreground" onClick={handleSelectAllRarities} type="button">
                            All
                        </button>
                        <span className="text-border">•</span>
                        <button className="text-muted-foreground transition-colors hover:text-foreground" onClick={handleDeselectAllRarities} type="button">
                            None
                        </button>
                    </div>
                </div>
                <div className="flex flex-wrap gap-2">
                    {RARITIES.map((rarity) => {
                        const isSelected = settings.allowedRarities.includes(rarity);
                        const rarityColor = RARITY_COLORS[rarity] ?? RARITY_COLORS[1];

                        return (
                            <button className={`flex items-center gap-1 rounded-lg border px-3 py-2 transition-all ${isSelected ? "border-primary bg-primary/10" : "border-border bg-secondary/50 hover:border-border/80"}`} key={rarity} onClick={() => handleRarityToggle(rarity)} type="button">
                                <span className="font-medium text-sm" style={{ color: rarityColor }}>
                                    {rarity}★
                                </span>
                            </button>
                        );
                    })}
                </div>
            </div>

            {/* Squad Size */}
            <div className="space-y-3">
                <div className="flex items-center justify-between">
                    <Label className="font-medium text-foreground text-sm">Squad Size</Label>
                    <span className="font-mono text-foreground text-sm">{settings.squadSize}</span>
                </div>
                <Slider className="w-full" max={20} min={1} onValueChange={handleSquadSizeChange} step={1} value={[settings.squadSize]} />
            </div>

            {/* Allow Duplicates */}
            <div className="flex items-center justify-between rounded-lg border border-border/50 bg-secondary/30 p-3">
                <Label className="font-medium text-foreground text-sm" htmlFor="allow-duplicates">
                    Allow Duplicate Operators
                </Label>
                <Switch checked={settings.allowDuplicates} id="allow-duplicates" onCheckedChange={handleDuplicatesToggle} />
            </div>

            {/* Only Elite 2 Operators */}
            {hasProfile && (
                <div className="flex items-center justify-between rounded-lg border border-border/50 bg-secondary/30 p-3">
                    <div className="space-y-0.5">
                        <Label className="font-medium text-foreground text-sm" htmlFor="only-e2">
                            Only Elite 2 Operators
                        </Label>
                        <p className="text-muted-foreground text-xs">Only include operators you've promoted to E2</p>
                    </div>
                    <Switch checked={settings.onlyE2Operators} id="only-e2" onCheckedChange={handleE2Toggle} />
                </div>
            )}
        </div>
    );
}
