import { MonitorIcon, MoonIcon, PaletteIcon, RotateCcwIcon, SunIcon } from "lucide-react";
import { useId } from "react";
import { Button } from "#/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "#/components/ui/popover";
import { useTheme } from "#/hooks/use-theme";
import { accentToRenderedHex, COLOR_PRESETS, PRESET_MATCH_TOLERANCE } from "#/lib/theme/color-utils";
import { cn } from "#/lib/utils";

export default function ThemeToggle() {
    const { mode, resolved, accent, isDefaultAccent, setMode, setPresetHue, setCustomHex, resetAccent } = useTheme();
    const customInputId = useId();

    const TriggerIcon = mode === "auto" ? MonitorIcon : mode === "dark" ? MoonIcon : SunIcon;
    const triggerLabel = mode === "auto" ? "Theme: System" : `Theme: ${mode === "dark" ? "Dark" : "Light"}`;

    const renderedHex = accentToRenderedHex(accent, resolved === "dark");
    const customLabel = accent?.type === "custom" ? accent.hex.toUpperCase() : renderedHex.toUpperCase();

    function onPickCustom(event: React.ChangeEvent<HTMLInputElement>) {
        setCustomHex(event.target.value);
    }

    return (
        <Popover>
            <PopoverTrigger
                render={
                    <Button variant="ghost" size="icon" aria-label={`${triggerLabel}. Open appearance settings.`} title={triggerLabel}>
                        <TriggerIcon className="h-4.5 w-4.5" />
                    </Button>
                }
            />
            <PopoverContent align="end" sideOffset={8} className="w-64">
                <div className="flex flex-col gap-3">
                    <div className="flex items-center gap-1.5 font-medium text-muted-foreground text-xs">
                        <SunIcon className="h-3.5 w-3.5" />
                        <span>Appearance</span>
                    </div>
                    <div className="grid grid-cols-3 gap-1">
                        <ModeButton active={mode === "light"} icon={<SunIcon className="h-3.5 w-3.5" />} label="Light" onClick={() => setMode("light")} />
                        <ModeButton active={mode === "dark"} icon={<MoonIcon className="h-3.5 w-3.5" />} label="Dark" onClick={() => setMode("dark")} />
                        <ModeButton active={mode === "auto"} icon={<MonitorIcon className="h-3.5 w-3.5" />} label="Auto" onClick={() => setMode("auto")} />
                    </div>

                    <div className="-mx-1 h-px bg-border" />

                    <div className="flex items-center justify-between font-medium text-muted-foreground text-xs">
                        <span className="flex items-center gap-1.5">
                            <PaletteIcon className="h-3.5 w-3.5" />
                            Accent
                        </span>
                        {!isDefaultAccent && (
                            <button type="button" onClick={resetAccent} className="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] text-muted-foreground transition-colors hover:bg-accent hover:text-foreground" aria-label="Reset accent color to default">
                                <RotateCcwIcon className="h-3 w-3" />
                                Reset
                            </button>
                        )}
                    </div>

                    <div className="grid grid-cols-6 gap-1.5">
                        {COLOR_PRESETS.map((preset) => {
                            const selected = accent?.type === "preset" && Math.abs(accent.hue - preset.hue) < PRESET_MATCH_TOLERANCE;
                            return (
                                <button
                                    key={preset.name}
                                    type="button"
                                    onClick={() => setPresetHue(preset.hue)}
                                    title={preset.name}
                                    aria-label={`Set accent color to ${preset.name}`}
                                    aria-pressed={selected}
                                    className={cn("relative aspect-square w-full rounded-full ring-offset-2 ring-offset-popover transition-transform hover:scale-110 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring", selected && "ring-2 ring-foreground")}
                                    style={{ backgroundColor: preset.color }}
                                />
                            );
                        })}
                    </div>

                    <label htmlFor={customInputId} className={cn("group flex cursor-pointer items-center justify-between rounded-md border bg-popover px-2 py-1.5 text-xs transition-colors hover:bg-accent/40", accent?.type === "custom" ? "border-foreground/40" : "border-border")}>
                        <span className="flex items-center gap-2 text-foreground">
                            <span className="relative inline-block size-5 overflow-hidden rounded-full border border-border" aria-hidden="true">
                                <span className="absolute inset-0" style={{ backgroundColor: renderedHex }} />
                            </span>
                            Custom color
                        </span>
                        <span className="font-mono text-[10px] text-muted-foreground/70 tabular-nums">{customLabel}</span>
                        <input id={customInputId} type="color" value={renderedHex} onChange={onPickCustom} className="sr-only" aria-label="Choose custom accent color" />
                    </label>
                </div>
            </PopoverContent>
        </Popover>
    );
}

function ModeButton({ active, icon, label, onClick }: { active: boolean; icon: React.ReactNode; label: string; onClick: () => void }) {
    return (
        <button
            type="button"
            onClick={onClick}
            aria-pressed={active}
            className={cn("inline-flex h-8 items-center justify-center gap-1.5 rounded-md border border-transparent px-1.5 font-medium text-foreground text-xs transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring", active ? "border-border bg-accent" : "hover:bg-accent/60")}
        >
            {icon}
            {label}
        </button>
    );
}
