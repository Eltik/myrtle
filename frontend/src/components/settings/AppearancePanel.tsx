import { LanguagesIcon, MonitorIcon, MoonIcon, PaletteIcon, RotateCcwIcon, SunIcon } from "lucide-react";
import { useId } from "react";
import { useLocaleSwitch } from "#/components/LocaleSwitcher";
import type { messages as localeSwitcherMessages } from "#/components/LocaleSwitcher.messages";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Switch } from "#/components/ui/switch";
import { useTheme } from "#/hooks/use-theme";
import { useI18n, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { accentToRenderedHex, COLOR_PRESETS, PRESET_MATCH_TOLERANCE } from "#/lib/theme/color-utils";
import { cn } from "#/lib/utils";
import type { messages } from "./AppearancePanel.messages";
import { SectionLabel, SettingRow } from "./SettingsShell";

export function AppearancePanel() {
    const { mode, resolved, accent, isDefaultAccent, dynamicArtwork, setMode, setPresetHue, setCustomHex, resetAccent, setDynamicArtwork } = useTheme();
    const customInputId = useId();
    const t: TypedT<typeof messages> = useT("settings");

    const renderedHex = accentToRenderedHex(accent, resolved === "dark");
    const customLabel = accent?.type === "custom" ? accent.hex.toUpperCase() : renderedHex.toUpperCase();

    return (
        <div className="flex flex-col gap-4">
            <Card>
                <CardHeader>
                    <CardTitle>{t("appearance.theme.title")}</CardTitle>
                    <CardDescription>{t("appearance.theme.desc")}</CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <div className="grid max-w-90 grid-cols-3 gap-2">
                        <ModeButton active={mode === "light"} icon={<SunIcon />} label={t("appearance.mode.light")} onClick={() => setMode("light")} />
                        <ModeButton active={mode === "dark"} icon={<MoonIcon />} label={t("appearance.mode.dark")} onClick={() => setMode("dark")} />
                        <ModeButton active={mode === "auto"} icon={<MonitorIcon />} label={t("appearance.mode.auto")} onClick={() => setMode("auto")} />
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>{t("appearance.accent.title")}</CardTitle>
                    <CardDescription>{t("appearance.accent.desc")} </CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <SectionLabel icon={<PaletteIcon />}>{t("appearance.accent.presetHue")}</SectionLabel>
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
                                    aria-label={t("appearance.accent.setTo", { name: preset.name })}
                                    aria-pressed={selected}
                                    className={cn("relative aspect-square w-full cursor-pointer rounded-full border-2 border-background outline-none ring-1 ring-border transition-transform hover:scale-110 focus-visible:ring-2 focus-visible:ring-ring", selected && "ring-2 ring-foreground")}
                                    style={{ backgroundColor: swatch }}
                                />
                            );
                        })}
                    </div>

                    <SectionLabel icon={<PaletteIcon />}>{t("appearance.accent.customColor")}</SectionLabel>
                    <div className="flex flex-wrap items-center gap-3">
                        <label htmlFor={customInputId} className={cn("group inline-flex cursor-pointer items-center gap-2.5 rounded-lg border bg-card px-2.5 py-1.5 font-medium font-sans text-[13px] outline-none transition-colors hover:bg-accent/40", accent?.type === "custom" ? "border-foreground/40" : "border-input")}>
                            <span className="inline-block size-5 rounded-full border border-border" style={{ backgroundColor: renderedHex }} aria-hidden="true" />
                            <span>{accent?.type === "custom" ? customLabel : t("appearance.accent.pickCustomHex")}</span>
                            <span className="font-mono text-[10px] text-muted-foreground/70 tabular-nums">{customLabel}</span>
                            <input id={customInputId} type="color" value={renderedHex} onChange={(e) => setCustomHex(e.target.value)} className="sr-only" aria-label={t("appearance.accent.chooseCustom")} />
                        </label>
                        {!isDefaultAccent ? (
                            <Button variant="ghost" size="sm" onClick={resetAccent}>
                                <RotateCcwIcon className="size-3.5" /> {t("appearance.accent.reset")}
                            </Button>
                        ) : null}
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>{t("appearance.dynamicArt.title")}</CardTitle>
                    <CardDescription>{t("appearance.dynamicArt.desc")} </CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <SettingRow layout="inline" title={t("appearance.dynamicArt.rowTitle")} description={t("appearance.dynamicArt.rowDesc")} control={<Switch checked={dynamicArtwork} onCheckedChange={setDynamicArtwork} aria-label={t("appearance.dynamicArt.rowTitle")} />} />
                </CardContent>
            </Card>

            <LanguageCard />
        </div>
    );
}

/**
 * Language lives under Appearance rather than in a section of its own: it is a
 * display preference, and Appearance is the one section a signed-out visitor
 * can reach, so putting it here keeps the choice available without loosening
 * the nav filter in `SettingsPage`.
 *
 * With one enabled language the picker would be a control that cannot do
 * anything, so the card names the language as a fact instead. It disappears
 * entirely only when the provider has no languages at all, which is the
 * nothing-to-say case rather than the one-choice case.
 */
function LanguageCard(): React.ReactElement | null {
    const { locale, available } = useI18n();
    const t: TypedT<typeof messages> = useT("settings");
    const tCommon: TypedT<typeof localeSwitcherMessages> = useT("common");
    const switchLocale = useLocaleSwitch();

    if (available.length === 0) return null;

    const currentName = available.find((entry) => entry.code === locale)?.nativeName ?? locale;

    return (
        <Card>
            <CardHeader>
                <CardTitle>{tCommon("localeSwitcher.language")}</CardTitle>
                <CardDescription>{t("appearance.language.desc")}</CardDescription>
            </CardHeader>
            <CardContent className="pt-0">
                <SectionLabel icon={<LanguagesIcon />}>{t("appearance.language.displayLanguage")}</SectionLabel>
                {available.length < 2 ? (
                    <p className="m-0 font-sans text-[13px] text-muted-foreground leading-normal" lang={locale}>
                        {t("appearance.language.onlyOne", { language: currentName })}
                    </p>
                ) : (
                    <Select
                        value={locale}
                        onValueChange={(next: string | null) => {
                            if (next !== null && next !== locale) switchLocale(next);
                        }}
                    >
                        <SelectTrigger className="max-w-60" aria-label={t("appearance.language.choose")}>
                            <SelectValue>{() => currentName}</SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            {available.map((entry) => (
                                <SelectItem key={entry.code} value={entry.code} lang={entry.code}>
                                    {entry.nativeName}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                )}
            </CardContent>
        </Card>
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
