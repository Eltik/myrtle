import { MonitorIcon, MoonIcon, RotateCcwIcon, SunIcon } from "lucide-react";
import { useId } from "react";
import { GamedataServerSelect, useGamedataServerOptions } from "#/components/GamedataServerSwitcher";
import type { messages as gamedataServerMessages } from "#/components/GamedataServerSwitcher.messages";
import { LocaleOptionLabel, useLocaleSwitch } from "#/components/LocaleSwitcher";
import type { messages as localeSwitcherMessages } from "#/components/LocaleSwitcher.messages";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Switch } from "#/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useHydrated } from "#/hooks/use-hydrated";
import { useTheme } from "#/hooks/use-theme";
import { useI18n, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { accentToRenderedHex, COLOR_PRESETS, PRESET_MATCH_TOLERANCE } from "#/lib/theme/color-utils";
import { SERVER_THEME_STATE } from "#/lib/theme/store";
import { cn } from "#/lib/utils";
import type { messages } from "./AppearancePanel.messages";
import { SectionLabel, SettingRow } from "./SettingsShell";

export function AppearancePanel() {
    const theme = useTheme();
    const { setMode, setPresetHue, setCustomHex, resetAccent, setDynamicArtwork, setLatinNames, setSpoilers, setMaintenanceCountdown, setDataSaver, setCnUpdates } = theme;
    const customInputId = useId();
    const t: TypedT<typeof messages> = useT("settings");

    // This panel hydrates after the header has already filled the theme store from
    // localStorage, so on its hydrating pass the store disagreed with the server HTML: the
    // custom hex text, every swatch, and the pressed mode button and switches (React does not
    // patch attributes, so the wrong mode stayed pressed). Until hydration it renders the
    // server's view, the defaults, and the next render shows the stored preferences.
    // Theme-dependent colours are given for BOTH themes as variables and the `.dark` class
    // picks one, so they need no second pass.
    const hydrated = useHydrated();
    const { mode, accent, dynamicArtwork, latinNames, spoilers, maintenanceCountdown, dataSaver, cnUpdates } = hydrated ? theme : SERVER_THEME_STATE;
    const isCustom = accent?.type === "custom";
    const lightHex = accentToRenderedHex(accent, false);
    const darkHex = accentToRenderedHex(accent, true);
    const inputHex = hydrated && theme.resolved === "dark" ? darkHex : lightHex;

    return (
        <div className="flex flex-col gap-4">
            <Card>
                <CardHeader>
                    <CardTitle>{t("appearance.theme.title")}</CardTitle>
                    <CardDescription>{t("appearance.theme.desc")}</CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <ToggleGroup aria-label={t("appearance.theme.title")} className="grid w-full max-w-90 grid-cols-3 gap-2" value={[mode]} onValueChange={(next) => next[0] && setMode(next[0] as typeof mode)}>
                        <ModeItem value="light" icon={<SunIcon />} label={t("appearance.mode.light")} />
                        <ModeItem value="dark" icon={<MoonIcon />} label={t("appearance.mode.dark")} />
                        <ModeItem value="auto" icon={<MonitorIcon />} label={t("appearance.mode.auto")} />
                    </ToggleGroup>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>{t("appearance.accent.title")}</CardTitle>
                    <CardDescription>{t("appearance.accent.desc")} </CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <SectionLabel>{t("appearance.accent.presetHue")}</SectionLabel>
                    <div className="mb-5 grid max-w-110 grid-cols-6 gap-2.5 sm:grid-cols-12">
                        {COLOR_PRESETS.map((preset) => {
                            const selected = accent?.type === "preset" && Math.abs(accent.hue - preset.hue) < PRESET_MATCH_TOLERANCE;
                            return (
                                <Tooltip key={preset.name}>
                                    <TooltipTrigger
                                        render={
                                            <button
                                                type="button"
                                                onClick={() => setPresetHue(preset.hue)}
                                                aria-label={t("appearance.accent.setTo", { name: preset.name })}
                                                aria-pressed={selected}
                                                className={cn(
                                                    "pointer-coarse:after:-translate-1/2 relative aspect-square w-full cursor-pointer rounded-full border-2 border-background bg-(--swatch-light) outline-none ring-1 ring-border transition-transform pointer-coarse:after:absolute pointer-coarse:after:top-1/2 pointer-coarse:after:left-1/2 pointer-coarse:after:size-11 hover:scale-110 focus-visible:ring-2 focus-visible:ring-ring dark:bg-(--swatch-dark)",
                                                    selected && "ring-2 ring-foreground",
                                                )}
                                                style={swatchVars(accentToRenderedHex({ type: "preset", hue: preset.hue }, false), accentToRenderedHex({ type: "preset", hue: preset.hue }, true))}
                                            />
                                        }
                                    />
                                    <TooltipPopup>{preset.name}</TooltipPopup>
                                </Tooltip>
                            );
                        })}
                    </div>

                    <SectionLabel>{t("appearance.accent.customColor")}</SectionLabel>
                    <div className="flex flex-wrap items-center gap-3">
                        {/* The colour input is `sr-only`, so keyboard focus lands on something
                            invisible; the ring goes on the label that wraps it. */}
                        <label
                            htmlFor={customInputId}
                            className={cn("group inline-flex cursor-pointer items-center gap-2.5 rounded-lg border bg-card px-2.5 py-1.5 font-medium font-sans text-[13px] transition-colors hover:bg-accent/40 has-focus-visible:ring-2 has-focus-visible:ring-ring", isCustom ? "border-foreground/40" : "border-input")}
                        >
                            <span className="inline-block size-5 rounded-full border border-border bg-(--swatch-light) dark:bg-(--swatch-dark)" style={swatchVars(lightHex, darkHex)} aria-hidden="true" />
                            <span>{isCustom ? t("appearance.accent.customActive") : t("appearance.accent.pickCustomHex")}</span>
                            <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                {lightHex === darkHex ? (
                                    lightHex.toUpperCase()
                                ) : (
                                    <>
                                        <span className="dark:hidden">{lightHex.toUpperCase()}</span>
                                        <span className="not-dark:hidden">{darkHex.toUpperCase()}</span>
                                    </>
                                )}
                            </span>
                            <input id={customInputId} type="color" value={inputHex} onChange={(e) => setCustomHex(e.target.value)} className="sr-only" aria-label={t("appearance.accent.chooseCustom")} />
                        </label>
                        {accent !== null ? (
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

            <ToggleCard title={t("appearance.latinNames.title")} description={t("appearance.latinNames.desc")} rowTitle={t("appearance.latinNames.rowTitle")} rowDescription={t("appearance.latinNames.rowDesc")} checked={latinNames} onCheckedChange={setLatinNames} />

            <ToggleCard title={t("appearance.spoilers.title")} description={t("appearance.spoilers.desc")} rowTitle={t("appearance.spoilers.rowTitle")} rowDescription={t("appearance.spoilers.rowDesc")} checked={spoilers} onCheckedChange={setSpoilers} />

            <ToggleCard title={t("appearance.cnUpdates.title")} description={t("appearance.cnUpdates.desc")} rowTitle={t("appearance.cnUpdates.rowTitle")} rowDescription={t("appearance.cnUpdates.rowDesc")} checked={cnUpdates} onCheckedChange={setCnUpdates} />

            <ToggleCard title={t("appearance.topBar.title")} description={t("appearance.topBar.desc")} rowTitle={t("appearance.topBar.maintenance.rowTitle")} rowDescription={t("appearance.topBar.maintenance.rowDesc")} checked={maintenanceCountdown} onCheckedChange={setMaintenanceCountdown} />

            <ToggleCard title={t("appearance.dataSaver.title")} description={t("appearance.dataSaver.desc")} rowTitle={t("appearance.dataSaver.rowTitle")} rowDescription={t("appearance.dataSaver.rowDesc")} checked={dataSaver} onCheckedChange={setDataSaver} />
            <LanguageCard />
        </div>
    );
}

/** One colour per theme as CSS variables, read by `bg-(--swatch-light)` and `dark:bg-(--swatch-dark)`. */
function swatchVars(light: string, dark: string): React.CSSProperties {
    return { "--swatch-light": light, "--swatch-dark": dark } as React.CSSProperties;
}

/** A card holding one on/off setting: the card names the area, the row names the switch. */
function ToggleCard({ title, description, rowTitle, rowDescription, checked, onCheckedChange }: { title: string; description: string; rowTitle: string; rowDescription: string; checked: boolean; onCheckedChange: (checked: boolean) => void }) {
    return (
        <Card>
            <CardHeader>
                <CardTitle>{title}</CardTitle>
                <CardDescription>{description}</CardDescription>
            </CardHeader>
            <CardContent className="pt-0">
                <SettingRow layout="inline" title={rowTitle} description={rowDescription} control={<Switch checked={checked} onCheckedChange={onCheckedChange} aria-label={rowTitle} />} />
            </CardContent>
        </Card>
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
                <SectionLabel>{t("appearance.language.displayLanguage")}</SectionLabel>
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
                                    <LocaleOptionLabel entry={entry} />
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                )}
                <GameTextSection />
            </CardContent>
        </Card>
    );
}

/**
 * Which client's game text the site shows, under the display language because
 * that is what it defaults to: a Korean reader gets the Korean client unless
 * they pick another. Absent below two loaded servers.
 */
function GameTextSection(): React.ReactElement | null {
    const t: TypedT<typeof messages> = useT("settings");
    const tCommon: TypedT<typeof gamedataServerMessages> = useT("common");
    const options = useGamedataServerOptions();
    if (options.length === 0) return null;

    return (
        <div className="mt-5">
            <SectionLabel>{tCommon("gamedataServer.label")}</SectionLabel>
            <p className="mt-0 mb-2 font-sans text-[13px] text-muted-foreground leading-normal">{t("appearance.language.gameTextDesc")}</p>
            <GamedataServerSelect className="max-w-60" />
        </div>
    );
}

function ModeItem({ value, icon, label }: { value: string; icon: React.ReactNode; label: string }) {
    return (
        <ToggleGroupItem value={value} className="h-15 flex-col gap-1.5 border-input bg-card font-sans text-[13px] sm:h-15 sm:text-[13px]">
            {icon}
            {label}
        </ToggleGroupItem>
    );
}
