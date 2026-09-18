import { MonitorIcon, MoonIcon, PaletteIcon, RotateCcwIcon, SparklesIcon, SunIcon } from "lucide-react";
import { useId } from "react";
import { Button } from "#/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "#/components/ui/popover";
import { Switch } from "#/components/ui/switch";
import { useTheme } from "#/hooks/use-theme";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { accentToRenderedHex, COLOR_PRESETS, PRESET_MATCH_TOLERANCE } from "#/lib/theme/color-utils";
import { cn } from "#/lib/utils";
import type { messages } from "./ThemeToggle.messages";

export default function ThemeToggle() {
    const { mode, resolved, accent, isDefaultAccent, dynamicArtwork, setMode, setPresetHue, setCustomHex, resetAccent, setDynamicArtwork } = useTheme();
    const customInputId = useId();
    const dynamicArtId = useId();
    const t: TypedT<typeof messages> = useT("nav");

    const TriggerIcon = mode === "auto" ? MonitorIcon : mode === "dark" ? MoonIcon : SunIcon;
    const triggerLabel = t("themeToggle.trigger", { mode });

    const renderedHex = accentToRenderedHex(accent, resolved === "dark");
    const customLabel = accent?.type === "custom" ? accent.hex.toUpperCase() : renderedHex.toUpperCase();

    function onPickCustom(event: React.ChangeEvent<HTMLInputElement>) {
        setCustomHex(event.target.value);
    }

    return (
        <Popover>
            <PopoverTrigger
                render={
                    <Button variant="ghost" size="icon" aria-label={t("themeToggle.triggerAria", { label: triggerLabel })} title={triggerLabel}>
                        <TriggerIcon className="h-4 w-4" />
                    </Button>
                }
            />
            {/* w-72 rather than w-64: at 256px the three mode labels came to 222px in
                Russian against 222px of content box, so every one of them ellipsised by
                a character. 288px leaves 32px of slack there, and the truncate on the
                labels is the floor for a language that still does not fit. */}
            <PopoverContent align="end" sideOffset={8} className="w-72">
                <div className="flex flex-col gap-3">
                    <div className="flex items-center gap-1.5 font-medium text-muted-foreground text-xs">
                        <SunIcon className="h-3.5 w-3.5 shrink-0" />
                        <span className="min-w-0">{t("themeToggle.appearance")}</span>
                    </div>
                    {/* Three equal columns gave each button a third of the popover
                        whatever its label needed, so a word wider than the cell
                        broke inside the fixed h-8 box and spilled out of it:
                        French "Sombre" rendered as "Sombr" over "e". The row is
                        now content-sized with the slack shared out, so each
                        button is at least as wide as its own word and the labels
                        never wrap. */}
                    <div className="flex gap-1">
                        <ModeButton active={mode === "light"} icon={<SunIcon className="h-3.5 w-3.5 shrink-0" />} label={t("themeToggle.light")} onClick={() => setMode("light")} />
                        <ModeButton active={mode === "dark"} icon={<MoonIcon className="h-3.5 w-3.5 shrink-0" />} label={t("themeToggle.dark")} onClick={() => setMode("dark")} />
                        <ModeButton active={mode === "auto"} icon={<MonitorIcon className="h-3.5 w-3.5 shrink-0" />} label={t("themeToggle.auto")} onClick={() => setMode("auto")} />
                    </div>

                    <div className="-mx-1 h-px bg-border" />

                    {/* Same shape as the mode row above: the heading is a phrase and
                        may wrap, the button is one word and may not. Without
                        shrink-0 the two squeezed each other and French broke
                        "Reinitialiser" across two lines inside the button. */}
                    <div className="flex items-center justify-between gap-2 font-medium text-muted-foreground text-xs">
                        <span className="flex min-w-0 items-center gap-1.5">
                            <PaletteIcon className="h-3.5 w-3.5 shrink-0" />
                            {t("themeToggle.accent")}
                        </span>
                        {!isDefaultAccent && (
                            <button type="button" onClick={resetAccent} className="inline-flex shrink-0 items-center gap-1 whitespace-nowrap rounded px-1.5 py-0.5 text-[10px] text-muted-foreground transition-colors hover:bg-accent hover:text-foreground" aria-label={t("themeToggle.resetAccentAria")}>
                                <RotateCcwIcon className="h-3 w-3 shrink-0" />
                                {t("themeToggle.reset")}
                            </button>
                        )}
                    </div>

                    <div className="grid grid-cols-6 gap-1.5">
                        {COLOR_PRESETS.map((preset) => {
                            const selected = accent?.type === "preset" && Math.abs(accent.hue - preset.hue) < PRESET_MATCH_TOLERANCE;
                            const swatch = accentToRenderedHex({ type: "preset", hue: preset.hue }, resolved === "dark");
                            return (
                                <button
                                    key={preset.name}
                                    type="button"
                                    onClick={() => setPresetHue(preset.hue)}
                                    title={preset.name}
                                    aria-label={t("themeToggle.setAccent", { name: preset.name })}
                                    aria-pressed={selected}
                                    className={cn("relative aspect-square w-full rounded-full ring-offset-2 ring-offset-popover transition-transform hover:scale-110 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring", selected && "ring-2 ring-foreground")}
                                    style={{ backgroundColor: swatch }}
                                />
                            );
                        })}
                    </div>

                    <label htmlFor={customInputId} className={cn("group flex cursor-pointer items-center justify-between rounded-md border bg-popover px-2 py-1.5 text-xs transition-colors hover:bg-accent/40", accent?.type === "custom" ? "border-foreground/40" : "border-border")}>
                        <span className="flex min-w-0 items-center gap-2 text-foreground">
                            <span className="relative inline-block size-5 shrink-0 overflow-hidden rounded-full border border-border" aria-hidden="true">
                                <span className="absolute inset-0" style={{ backgroundColor: renderedHex }} />
                            </span>
                            {t("themeToggle.customColor")}
                        </span>
                        <span className="shrink-0 font-mono text-[10px] text-muted-foreground/70 tabular-nums">{customLabel}</span>
                        <input id={customInputId} type="color" value={renderedHex} onChange={onPickCustom} className="sr-only" aria-label={t("themeToggle.chooseCustomColor")} />
                    </label>

                    <div className="-mx-1 h-px bg-border" />

                    <label htmlFor={dynamicArtId} className="flex cursor-pointer items-center justify-between gap-2">
                        <span className="flex min-w-0 items-center gap-1.5 font-medium text-muted-foreground text-xs">
                            <SparklesIcon className="h-3.5 w-3.5 shrink-0" />
                            {t("themeToggle.dynamicArt")}
                        </span>
                        <Switch id={dynamicArtId} checked={dynamicArtwork} onCheckedChange={setDynamicArtwork} aria-label={t("themeToggle.dynamicArtAria")} className="shrink-0" />
                    </label>
                    <p className="text-[10px] text-muted-foreground/70 leading-snug">{t("themeToggle.dynamicArtNote")}</p>
                </div>
            </PopoverContent>
        </Popover>
    );
}

function ModeButton({ active, icon, label, lang, onClick }: { active: boolean; icon?: React.ReactNode; label: string; lang?: string; onClick: () => void }) {
    return (
        <button
            type="button"
            onClick={onClick}
            aria-pressed={active}
            lang={lang}
            className={cn(
                "inline-flex h-8 min-w-0 grow items-center justify-center gap-1.5 rounded-md border border-transparent px-1.5 font-medium text-foreground text-xs transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                active ? "border-border bg-accent" : "hover:bg-accent/60",
            )}
        >
            {icon}
            {/* A language whose three words genuinely cannot fit clips with an
                ellipsis, which is still readable; wrapping inside a fixed-height
                button is not. */}
            <span className="truncate">{label}</span>
        </button>
    );
}
