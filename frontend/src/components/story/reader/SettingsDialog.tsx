/**
 * THE SETTINGS SHEET: the rows that write one field each, and the three
 * sections that do not.
 *
 * What this file OWNS is the order the sheet reads in and the two writers every
 * row goes through. `apply` re-derives the reading style on every write, which
 * is why "Custom" is a value the select READS and never one a reader picks;
 * `set` is `apply` for a single field. The sections that carry state of their
 * own live beside it: `SettingsFont` (the uploaded face and its IndexedDB
 * record), `SettingsPosition` (the pad and its presets) and `SettingsProgress`
 * (backup, restore and the one destructive control in the reader).
 *
 * Every bound below is the one `settings.ts` coerces to. Nothing is clamped
 * twice: a slider that could hand back a value the document refuses would be a
 * control that lies about its own range.
 */
import type React from "react";
import { Dialog, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Input } from "#/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Slider } from "#/components/ui/slider";
import { Switch } from "#/components/ui/switch";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { applyReadingStyle, CUTSCENE_PLAYERS, type CutscenePlayer, clampBoxPosition, DEFAULT_NICKNAME, NICKNAME_MAX, READING_STYLES, type ReadingPreset, readingStyleOf, SPEAKER_TINTS, type SpeakerTint, type StorySettings, sliderValue } from "#/lib/story/settings";
import { cn } from "#/lib/utils";
import { CHROME_IDLE_MAX_SEC, CHROME_IDLE_MIN_SEC, CHROME_IDLE_STEP_SEC } from "./chrome";
import { isHexColor, swatchColor, TEXT_SWATCHES } from "./colors";
import type { messages } from "./reader.messages";
import { SettingsFont } from "./SettingsFont";
import { BoxPositionControl } from "./SettingsPosition";
import { ProgressSection, ResetProgressDialog, useProgressActions } from "./SettingsProgress";
import { Row, TOUCH_CLOSE, TOUCH_INPUT, TOUCH_SLIDER, TOUCH_SWITCH } from "./SettingsRow";
import type { messages as styleMessages } from "./styles.messages";

export interface ISettingsDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    settings: StorySettings;
    onChange: (next: StorySettings) => void;
}

export function SettingsDialog({ open, onOpenChange, settings, onChange }: ISettingsDialogProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const ts: TypedT<typeof styleMessages> = useT("story");
    // The reading style is DERIVED on every write, never left behind: the five
    // fields a preset owns are ordinary settings, so editing one of them by
    // hand is what turns the select to "Custom", and setting all five back to a
    // preset's own values turns it back into that preset.
    const apply = (next: StorySettings) => onChange({ ...next, style: readingStyleOf(next) });
    // Derived at RENDER too, never read off the stored field: another tab, a
    // hand-edit or an older stored shape can all leave the name behind, and a
    // select naming a preset whose numbers are not on screen is worse than no
    // select at all.
    const style = readingStyleOf(settings);
    const set = <K extends keyof StorySettings>(key: K, value: StorySettings[K]) => apply({ ...settings, [key]: value });
    const boxPosition = clampBoxPosition({ x: settings.boxX, y: settings.boxY });
    const progressActions = useProgressActions();

    return (
        <>
            <Dialog open={open} onOpenChange={onOpenChange}>
                <DialogPopup className="max-w-xl" closeProps={{ className: TOUCH_CLOSE }}>
                    <DialogHeader>
                        <DialogTitle>{t("settings.title")}</DialogTitle>
                    </DialogHeader>
                    <DialogPanel className="flex max-h-[70dvh] flex-col gap-4 overflow-y-auto">
                        {/* THE READING STYLE, first because it writes the five rows
                        under it in one click. "Custom" is not a choice a reader
                        makes: it is what the select reads back once one of those
                        five has been edited, so it is in the list to be SHOWN
                        and disabled to stop it being picked, which would mean
                        nothing. */}
                        <Row label={t("settings.style")} hint={t("settings.styleHint")}>
                            <Select value={style} onValueChange={(v: string | null) => v && v !== "custom" && apply(applyReadingStyle(settings, v as ReadingPreset))}>
                                <SelectTrigger className="max-sm:h-11" data-story-reading-style>
                                    <SelectValue>{() => ts(`settings.style.${style}`)}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {READING_STYLES.map((name) => (
                                        <SelectItem key={name} value={name} disabled={name === "custom"} data-story-reading-style-option={name}>
                                            {ts(`settings.style.${name}`)}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </Row>
                        <Row label={t("settings.cps")} value={t("settings.cpsValue", { value: Math.round(settings.cps) })}>
                            <Slider className={TOUCH_SLIDER} min={10} max={120} step={5} value={[settings.cps]} onValueChange={(v) => set("cps", sliderValue(v, settings.cps))} />
                        </Row>
                        <Row label={t("settings.textSize")} value={t("settings.percent", { value: Math.round(settings.textSize) })}>
                            <Slider className={TOUCH_SLIDER} min={70} max={200} step={5} value={[settings.textSize]} onValueChange={(v) => set("textSize", sliderValue(v, settings.textSize))} />
                        </Row>
                        <Row label={t("settings.lineWidth")} value={t("settings.lineWidthValue", { value: Math.round(settings.lineWidth) })}>
                            <Slider className={TOUCH_SLIDER} min={40} max={120} step={5} value={[settings.lineWidth]} onValueChange={(v) => set("lineWidth", sliderValue(v, settings.lineWidth))} />
                        </Row>
                        <Row label={t("settings.lineHeight")} value={settings.lineHeight.toFixed(2)}>
                            <Slider className={TOUCH_SLIDER} min={1.2} max={2.2} step={0.05} value={[settings.lineHeight]} onValueChange={(v) => set("lineHeight", sliderValue(v, settings.lineHeight))} />
                        </Row>
                        <Row label={t("settings.sideMargin")} value={t("settings.percent", { value: Math.round(settings.sideMargin) })}>
                            <Slider className={TOUCH_SLIDER} min={0} max={20} step={1} value={[settings.sideMargin]} onValueChange={(v) => set("sideMargin", sliderValue(v, settings.sideMargin))} />
                        </Row>
                        <Row label={t("settings.bottomMargin")} value={t("settings.percent", { value: Math.round(settings.bottomMargin) })}>
                            <Slider className={TOUCH_SLIDER} min={0} max={20} step={1} value={[settings.bottomMargin]} onValueChange={(v) => set("bottomMargin", sliderValue(v, settings.bottomMargin))} />
                        </Row>
                        <Row label={t("settings.lightBox")} hint={t("settings.lightBoxHint")}>
                            <Switch className={TOUCH_SWITCH} checked={settings.lightBox} onCheckedChange={(c) => set("lightBox", c)} />
                        </Row>
                        <Row label={t("settings.letterbox")} hint={t("settings.letterboxHint")}>
                            <Switch className={TOUCH_SWITCH} checked={settings.letterbox} onCheckedChange={(c) => set("letterbox", c)} />
                        </Row>
                        <Row label={t("settings.animateRatio")} value={t("settings.multiplier", { value: settings.animateRatio.toFixed(1) })} hint={t("settings.animateRatioHint")}>
                            <Slider className={TOUCH_SLIDER} min={0} max={3} step={0.1} value={[settings.animateRatio]} onValueChange={(v) => set("animateRatio", sliderValue(v, settings.animateRatio))} />
                        </Row>
                        <Row label={t("settings.autoPace")} value={t("settings.multiplier", { value: settings.autoPace.toFixed(1) })}>
                            <Slider className={TOUCH_SLIDER} min={0.5} max={3} step={0.1} value={[settings.autoPace]} onValueChange={(v) => set("autoPace", sliderValue(v, settings.autoPace))} />
                        </Row>
                        <Row label={t("settings.minLineSec")} value={t("settings.seconds", { value: settings.minLineSec.toFixed(1) })}>
                            <Slider className={TOUCH_SLIDER} min={0} max={10} step={0.5} value={[settings.minLineSec]} onValueChange={(v) => set("minLineSec", sliderValue(v, settings.minLineSec))} />
                        </Row>
                        <Row label={t("settings.musicVolume")} value={t("settings.percent", { value: Math.round(settings.musicVolume * 100) })}>
                            <Slider className={TOUCH_SLIDER} min={0} max={1} step={0.05} value={[settings.musicVolume]} onValueChange={(v) => set("musicVolume", sliderValue(v, settings.musicVolume))} />
                        </Row>
                        <Row label={t("settings.sfxVolume")} value={t("settings.percent", { value: Math.round(settings.sfxVolume * 100) })}>
                            <Slider className={TOUCH_SLIDER} min={0} max={1} step={0.05} value={[settings.sfxVolume]} onValueChange={(v) => set("sfxVolume", sliderValue(v, settings.sfxVolume))} />
                        </Row>
                        <Row label={t("settings.progressBar")}>
                            <Switch className={TOUCH_SWITCH} checked={settings.progressBar} onCheckedChange={(c) => set("progressBar", c)} />
                        </Row>
                        <Row label={t("settings.playVideos")} hint={t("settings.playVideosHint")}>
                            <Switch className={TOUCH_SWITCH} checked={settings.playVideos} onCheckedChange={(c) => set("playVideos", c)} />
                        </Row>
                        {/* Under the toggle that turns cutscenes on, because it is a
                        setting ABOUT the layer that toggle draws: with videos
                        off there is nothing for it to choose. */}
                        <Row label={t("settings.cutscenePlayer")} hint={t("settings.cutscenePlayerHint")}>
                            <Select value={settings.cutscenePlayer} onValueChange={(v: string | null) => v && set("cutscenePlayer", v as CutscenePlayer)}>
                                <SelectTrigger className="max-sm:h-11" data-story-cutscene-player>
                                    <SelectValue>{() => t(`settings.cutscenePlayer.${settings.cutscenePlayer}`)}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {CUTSCENE_PLAYERS.map((v: CutscenePlayer) => (
                                        <SelectItem key={v} value={v} data-story-cutscene-player-option={v}>
                                            {t(`settings.cutscenePlayer.${v}`)}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </Row>
                        {/* The field holds exactly what was typed, spaces and all,
                        so a name being typed is never rewritten under the
                        cursor; `resolveNickname` trims it where it is READ, and
                        the placeholder says what an empty field resolves to. */}
                        <Row label={t("settings.nickname")} hint={t("settings.nicknameHint")}>
                            <Input className={TOUCH_INPUT} value={settings.nickname} maxLength={NICKNAME_MAX} placeholder={DEFAULT_NICKNAME} data-story-nickname onChange={(e) => set("nickname", e.target.value)} />
                        </Row>
                        <SettingsFont open={open} settings={settings} onChange={onChange} apply={apply} />
                        <Row label={t("settings.textColor")} hint={t("settings.textColorHint")}>
                            <div className="flex flex-wrap items-center gap-1.5">
                                <button
                                    type="button"
                                    aria-label={t("settings.textColor.default")}
                                    title={t("settings.textColor.default")}
                                    aria-pressed={settings.textColor === ""}
                                    onClick={() => set("textColor", "")}
                                    className={cn("size-7 rounded-full border bg-linear-to-br from-white to-neutral-900 max-sm:size-11", settings.textColor === "" && "ring-2 ring-ring ring-offset-2 ring-offset-background")}
                                />
                                {TEXT_SWATCHES.map((sw, i) => (
                                    <button
                                        key={sw.id}
                                        type="button"
                                        data-story-swatch={sw.id}
                                        aria-label={t("settings.textColor.swatch", { number: i + 1 })}
                                        title={t("settings.textColor.swatch", { number: i + 1 })}
                                        aria-pressed={settings.textColor === sw.id}
                                        onClick={() => set("textColor", sw.id)}
                                        className={cn("size-7 rounded-full border border-black/20 max-sm:size-11", settings.textColor === sw.id && "ring-2 ring-ring ring-offset-2 ring-offset-background")}
                                        style={{ backgroundColor: swatchColor(sw, settings.lightBox) }}
                                    />
                                ))}
                                <input
                                    type="color"
                                    aria-label={t("settings.textColor.custom")}
                                    title={t("settings.textColor.custom")}
                                    data-story-color-input
                                    value={isHexColor(settings.textColor) ? settings.textColor : swatchColor(TEXT_SWATCHES[0], settings.lightBox)}
                                    onChange={(e) => set("textColor", e.target.value)}
                                    className="size-7 cursor-pointer rounded-full border bg-transparent p-0 max-sm:size-11"
                                />
                            </div>
                        </Row>
                        <Row label={t("settings.speakerTint")} hint={t("settings.speakerTintHint")}>
                            <Select value={settings.speakerTint} onValueChange={(v: string | null) => v && set("speakerTint", v as SpeakerTint)}>
                                <SelectTrigger className="max-sm:h-11" data-story-speaker-tint>
                                    <SelectValue>{() => t(`settings.speakerTint.${settings.speakerTint}`)}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {SPEAKER_TINTS.map((v: SpeakerTint) => (
                                        <SelectItem key={v} value={v} data-story-speaker-tint-option={v}>
                                            {t(`settings.speakerTint.${v}`)}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </Row>
                        <Row label={t("settings.boxPosition")} value={t("settings.boxPosition.value", { x: `${Math.round(((boxPosition.x + 1) / 2) * 100)}%`, y: `${Math.round(boxPosition.y * 100)}%` })} hint={t("settings.boxPositionHint")}>
                            <BoxPositionControl position={boxPosition} onChange={(p) => onChange({ ...settings, boxX: p.x, boxY: p.y })} />
                        </Row>
                        <Row label={t("settings.toolbarHidden")} hint={t("settings.toolbarHiddenHint")}>
                            <Switch className={TOUCH_SWITCH} checked={settings.toolbarHidden} onCheckedChange={(c) => set("toolbarHidden", c)} />
                        </Row>
                        <Row label={t("settings.autoHideToolbar")} hint={t("settings.autoHideToolbarHint")}>
                            <Switch className={TOUCH_SWITCH} checked={settings.autoHideToolbar} onCheckedChange={(c) => set("autoHideToolbar", c)} />
                        </Row>
                        {/* The delay only exists while the fade does. A slider for a
                        behaviour that is off is a question with no answer. */}
                        {settings.autoHideToolbar ? (
                            <Row label={t("settings.toolbarIdle")} value={t("settings.seconds", { value: settings.toolbarIdleSec.toFixed(1) })} hint={t("settings.toolbarIdleHint")}>
                                <Slider className={TOUCH_SLIDER} min={CHROME_IDLE_MIN_SEC} max={CHROME_IDLE_MAX_SEC} step={CHROME_IDLE_STEP_SEC} value={[settings.toolbarIdleSec]} onValueChange={(v) => set("toolbarIdleSec", sliderValue(v, settings.toolbarIdleSec))} />
                            </Row>
                        ) : null}

                        <ProgressSection actions={progressActions} />
                        <div className="border-t pt-4">
                            <div className="mb-1 font-heading font-semibold text-sm">{t("settings.hotkeys.heading")}</div>
                            <p className="text-muted-foreground text-xs">{t("settings.hotkeys")}</p>
                        </div>
                    </DialogPanel>
                </DialogPopup>
            </Dialog>
            <ResetProgressDialog actions={progressActions} />
        </>
    );
}
