"use client";

import { Filter } from "lucide-react";
import Image from "next/image";
import { CLASS_DISPLAY, CLASS_ICON, CLASSES, RARITIES } from "~/components/collection/operators/list/constants";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import { Label } from "~/components/ui/shadcn/label";
import { Slider } from "~/components/ui/shadcn/slider";
import { Switch } from "~/components/ui/shadcn/switch";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { cn } from "~/lib/utils";
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

    const handleUnplayableOperators = (checked: boolean) => {
        setSettings({ ...settings, allowUnplayableOperators: checked });
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
            onE2Enabled?.();
        } else {
            onE2Disabled?.();
        }
    };

    return (
        <Card className="overflow-hidden border-border/50 bg-linear-to-br from-card/40 to-card/20 py-0 shadow-lg backdrop-blur-md">
            <CardContent className="space-y-5 p-6">
                <div className="flex items-center gap-3">
                    <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-linear-to-br from-primary/20 to-primary/5 shadow-md">
                        <Filter className="h-4.5 w-4.5 text-primary" />
                    </div>
                    <div>
                        <h2 className="font-semibold text-foreground text-lg">Filters</h2>
                        <p className="text-muted-foreground text-xs">Customize your randomization criteria</p>
                    </div>
                </div>

                {/* Classes Filter */}
                <div className="space-y-3">
                    <div className="flex items-center justify-between">
                        <Label className="font-medium text-muted-foreground text-sm">Class</Label>
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
                    <div className="flex flex-wrap gap-1.5">
                        {CLASSES.map((profession) => {
                            const isSelected = settings.allowedClasses.includes(profession);
                            const displayName = CLASS_DISPLAY[profession] ?? profession;

                            return (
                                <Tooltip key={profession}>
                                    <TooltipTrigger asChild>
                                        <button className={cn("flex h-10 w-10 items-center justify-center rounded-lg border transition-all", isSelected ? "border-primary bg-primary/20" : "border-border bg-secondary/50 hover:border-primary/50")} onClick={() => handleClassToggle(profession)} type="button">
                                            <Image alt={displayName} className={cn("icon-theme-aware h-6 w-6", isSelected ? "opacity-100" : "opacity-60")} height={24} src={`/api/cdn/upk/arts/ui/[uc]charcommon/icon_profession_${CLASS_ICON[profession] ?? profession.toLowerCase()}.png`} width={24} />
                                        </button>
                                    </TooltipTrigger>
                                    <TooltipContent>
                                        <p>{displayName}</p>
                                    </TooltipContent>
                                </Tooltip>
                            );
                        })}
                    </div>
                </div>

                {/* Rarities Filter */}
                <div className="space-y-3">
                    <div className="flex items-center justify-between">
                        <Label className="font-medium text-muted-foreground text-sm">Rarity</Label>
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

                            return (
                                <button
                                    className={cn("flex items-center justify-center rounded-lg border px-3 py-1.5 font-medium text-sm transition-all", isSelected ? "border-amber-500 bg-amber-500/20 text-amber-400" : "border-border bg-secondary/50 text-muted-foreground hover:border-amber-500/50 hover:text-amber-400")}
                                    key={rarity}
                                    onClick={() => handleRarityToggle(rarity)}
                                    type="button"
                                >
                                    {rarity}★
                                </button>
                            );
                        })}
                    </div>
                </div>

                {/* Squad Size */}
                <div className="space-y-3">
                    <div className="flex items-center justify-between">
                        <Label className="font-semibold text-foreground text-sm">Squad Size</Label>
                        <span className="rounded-lg bg-secondary/50 px-3 py-1 font-mono font-semibold text-foreground text-sm">{settings.squadSize}</span>
                    </div>
                    <Slider className="w-full" max={20} min={1} onValueChange={handleSquadSizeChange} step={1} value={[settings.squadSize]} />
                </div>

                {/* Allow Duplicates */}
                <div className="flex items-center justify-between rounded-xl border border-border/30 bg-linear-to-br from-secondary/50 to-secondary/30 p-4 shadow-sm backdrop-blur-sm">
                    <div className="space-y-0.5">
                        <Label className="font-semibold text-foreground text-sm" htmlFor="allow-duplicates">
                            Allow Duplicate Operators
                        </Label>
                        <p className="text-muted-foreground text-xs">Same operator can appear multiple times</p>
                    </div>
                    <Switch checked={settings.allowDuplicates} id="allow-duplicates" onCheckedChange={handleDuplicatesToggle} />
                </div>

                {/* Allow Unplayable Operators */}
                <div className="flex items-center justify-between rounded-xl border border-border/30 bg-linear-to-br from-secondary/50 to-secondary/30 p-4 shadow-sm backdrop-blur-sm">
                    <div className="space-y-0.5">
                        <Label className="font-semibold text-foreground text-sm" htmlFor="allow-duplicates">
                            Allow Unplayable Operators
                        </Label>
                        <p className="text-muted-foreground text-xs">Include Reserve Operators and unplayable operators in randomized squads</p>
                    </div>
                    <Switch checked={settings.allowUnplayableOperators} id="allow-duplicates" onCheckedChange={handleUnplayableOperators} />
                </div>

                {/* Only Elite 2 Operators */}
                {hasProfile && (
                    <div className="flex items-center justify-between rounded-xl border border-border/30 bg-linear-to-br from-secondary/50 to-secondary/30 p-4 shadow-sm backdrop-blur-sm">
                        <div className="space-y-0.5">
                            <Label className="font-semibold text-foreground text-sm" htmlFor="only-e2">
                                Only Elite 2 Operators
                            </Label>
                            <p className="text-muted-foreground text-xs">Only include operators you've promoted to E2</p>
                        </div>
                        <Switch checked={settings.onlyE2Operators} id="only-e2" onCheckedChange={handleE2Toggle} />
                    </div>
                )}
            </CardContent>
        </Card>
    );
}
