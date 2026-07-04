import { MonitorIcon, MoonIcon, PaletteIcon, RotateCcwIcon, SunIcon } from "lucide-react";
import { useId } from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { useTheme } from "#/hooks/use-theme";
import { accentToRenderedHex, COLOR_PRESETS, PRESET_MATCH_TOLERANCE } from "#/lib/theme/color-utils";
import { cn } from "#/lib/utils";
import { SectionLabel } from "./SettingsShell";

export function AppearancePanel() {
    const { mode, resolved, accent, isDefaultAccent, setMode, setPresetHue, setCustomHex, resetAccent } = useTheme();
    const customInputId = useId();

    const renderedHex = accentToRenderedHex(accent, resolved === "dark");
    const customLabel = accent?.type === "custom" ? accent.hex.toUpperCase() : renderedHex.toUpperCase();

    return (
        <div className="flex flex-col gap-4">
            <Card>
                <CardHeader>
                    <CardTitle>Theme</CardTitle>
                    <CardDescription>Pick light, dark, or follow your system. The mobile picker in the header controls the same setting.</CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <div className="grid max-w-90 grid-cols-3 gap-2">
                        <ModeButton active={mode === "light"} icon={<SunIcon />} label="Light" onClick={() => setMode("light")} />
                        <ModeButton active={mode === "dark"} icon={<MoonIcon />} label="Dark" onClick={() => setMode("dark")} />
                        <ModeButton active={mode === "auto"} icon={<MonitorIcon />} label="Auto" onClick={() => setMode("auto")} />
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Accent color</CardTitle>
                    <CardDescription>
                        Tints buttons, links, focus rings, and the active state across the whole app.{" "}
                        <Badge variant="outline" size="sm" className="ml-1 font-mono">
                            localStorage · myrtle-accent
                        </Badge>
                    </CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <SectionLabel icon={<PaletteIcon />}>Preset hue</SectionLabel>
                    <div className="mb-5 grid max-w-110 grid-cols-6 gap-2.5 sm:grid-cols-12">
                        {COLOR_PRESETS.map((preset) => {
                            const selected = accent?.type === "preset" && Math.abs(accent.hue - preset.hue) < PRESET_MATCH_TOLERANCE;
                            const swatch = accentToRenderedHex({ type: "preset", hue: preset.hue }, resolved === "dark");
                            return (
                                <button
                                    key={preset.name}
                                    type="button"
                                    onClick={() => setPresetHue(preset.hue)}
                                    title={preset.name}
                                    aria-label={`Set accent color to ${preset.name}`}
                                    aria-pressed={selected}
                                    className={cn("relative aspect-square w-full cursor-pointer rounded-full border-2 border-background outline-none ring-1 ring-border transition-transform hover:scale-110 focus-visible:ring-2 focus-visible:ring-ring", selected && "ring-2 ring-foreground")}
                                    style={{ backgroundColor: swatch }}
                                />
                            );
                        })}
                    </div>

                    <SectionLabel icon={<PaletteIcon />}>Custom color</SectionLabel>
                    <div className="flex flex-wrap items-center gap-3">
                        <label htmlFor={customInputId} className={cn("group inline-flex cursor-pointer items-center gap-2.5 rounded-lg border bg-card px-2.5 py-1.5 font-medium font-sans text-[13px] outline-none transition-colors hover:bg-accent/40", accent?.type === "custom" ? "border-foreground/40" : "border-input")}>
                            <span className="inline-block size-5 rounded-full border border-border" style={{ backgroundColor: renderedHex }} aria-hidden="true" />
                            <span>{accent?.type === "custom" ? customLabel : "Pick custom hex"}</span>
                            <span className="font-mono text-[10px] text-muted-foreground/70 tabular-nums">{customLabel}</span>
                            <input id={customInputId} type="color" value={renderedHex} onChange={(e) => setCustomHex(e.target.value)} className="sr-only" aria-label="Choose custom accent color" />
                        </label>
                        {!isDefaultAccent ? (
                            <Button variant="ghost" size="sm" onClick={resetAccent}>
                                <RotateCcwIcon className="size-3.5" /> Reset to default
                            </Button>
                        ) : null}
                    </div>
                </CardContent>
            </Card>
        </div>
    );
}

function ModeButton({ active, icon, label, onClick }: { active: boolean; icon: React.ReactNode; label: string; onClick: () => void }) {
    return (
        <button
            type="button"
            onClick={onClick}
            aria-pressed={active}
            className={cn(
                "inline-flex h-15 cursor-pointer flex-col items-center justify-center gap-1.5 rounded-lg border bg-card font-medium font-sans text-[13px] text-foreground outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background [&_svg]:size-4.5",
                active ? "border-foreground bg-accent" : "border-input hover:bg-accent/50",
            )}
        >
            {icon}
            {label}
        </button>
    );
}
