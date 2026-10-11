import type React from "react";
import { Fragment, useEffect, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Kbd } from "#/components/ui/kbd";
import { Slider, SliderPrimitive } from "#/components/ui/slider";
import { Switch } from "#/components/ui/switch";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { clockTime } from "#/lib/story/reading";
import { sliderValue } from "#/lib/story/settings";
import { cn } from "#/lib/utils";
import { preloadIsOn, SHORTCUTS, setSettingsOpen, setShortcutsOpen, setSleepTimer, updateSettings, usePlayer } from "./engine";
import type { PlayerT } from "./parts";
import type { messages as shortcutMessages } from "./Shortcuts.messages";
import { CROSSFADE_MAX, CROSSFADE_MIN, SLEEP_CHOICES, type SleepMinutes } from "./types";

function Section({ title, children }: { title: string; children: React.ReactNode }): React.ReactElement {
    return (
        <section className="flex flex-col gap-3 border-border border-t py-4 first:border-t-0 first:pt-0">
            <h3 className="font-mono text-[10px] text-primary uppercase tracking-[0.16em]">{title}</h3>
            {children}
        </section>
    );
}

function Row({ id, label, hint, control, dim }: { id: string; label: string; hint?: React.ReactNode; control: React.ReactNode; dim?: boolean }): React.ReactElement {
    return (
        <div className={cn("flex items-center justify-between gap-4 transition-opacity", dim && "opacity-45")}>
            <div className="flex min-w-0 flex-col gap-0.5">
                <label className="font-medium text-foreground text-sm" htmlFor={id} id={`${id}-label`}>
                    {label}
                </label>
                {hint && <span className="text-muted-foreground text-xs">{hint}</span>}
            </div>
            <div className="shrink-0">{control}</div>
        </div>
    );
}

/** `Pauses in m:ss`, ticking once a second while the dialog is open. */
function SleepStatus({ t }: { t: PlayerT }): React.ReactElement {
    const endsAt = usePlayer((s) => s.sleepEndsAt);
    const [now, setNow] = useState(() => Date.now());
    useEffect(() => {
        if (endsAt === null) return;
        setNow(Date.now());
        const id = setInterval(() => setNow(Date.now()), 1000);
        return () => clearInterval(id);
    }, [endsAt]);
    if (endsAt === null) return <>{t("music.settings.sleepOff")}</>;
    return <>{t("music.settings.sleepIn", { time: clockTime(Math.max(0, Math.ceil((endsAt - now) / 1000))) })}</>;
}

export function SettingsDialog(): React.ReactElement {
    const t: PlayerT = useT("gallery");
    const open = usePlayer((s) => s.settingsOpen);
    const settings = usePlayer((s) => s.settings);
    const endsAt = usePlayer((s) => s.sleepEndsAt);
    const [sleepPick, setSleepPick] = useState<SleepMinutes>(0);
    const choice: SleepMinutes = endsAt === null ? 0 : sleepPick;

    const pickSleep = (m: SleepMinutes) => {
        setSleepPick(m);
        setSleepTimer(m);
    };

    return (
        <Dialog onOpenChange={setSettingsOpen} open={open}>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>{t("music.settings.title")}</DialogTitle>
                </DialogHeader>
                <DialogPanel>
                    <Section title={t("music.settings.section.transitions")}>
                        <Row control={<Switch aria-labelledby="music-crossfade-label" checked={settings.crossfadeOn} id="music-crossfade" onCheckedChange={(v) => updateSettings({ crossfadeOn: v })} />} hint={t("music.settings.crossfadeHint")} id="music-crossfade" label={t("music.settings.crossfade")} />
                        <Row
                            control={
                                <div className="flex w-36 items-center gap-2">
                                    <Slider className="[&_[data-slot=slider-control]]:min-w-0" disabled={!settings.crossfadeOn} max={CROSSFADE_MAX} min={CROSSFADE_MIN} onValueChange={(v) => updateSettings({ crossfadeSeconds: sliderValue(v, settings.crossfadeSeconds) })} step={1} value={[settings.crossfadeSeconds]}>
                                        <SliderPrimitive.Label className="sr-only">{t("music.settings.duration")}</SliderPrimitive.Label>
                                    </Slider>
                                    <span className="w-7 text-right font-mono text-muted-foreground text-xs tabular-nums">{t("music.settings.durationValue", { seconds: settings.crossfadeSeconds })}</span>
                                </div>
                            }
                            dim={!settings.crossfadeOn}
                            hint={t("music.settings.durationHint")}
                            id="music-crossfade-seconds"
                            label={t("music.settings.duration")}
                        />
                        <p className="text-muted-foreground text-xs italic">{t("music.settings.note")}</p>
                    </Section>

                    <Section title={t("music.settings.section.extras")}>
                        <Row control={<Switch aria-labelledby="music-radio-label" checked={settings.radioOn} id="music-radio" onCheckedChange={(v) => updateSettings({ radioOn: v })} />} hint={t("music.settings.radioHint")} id="music-radio" label={t("music.settings.radio")} />
                        <div className="flex flex-col gap-2">
                            <div className="flex flex-col gap-0.5">
                                <span className="font-medium text-foreground text-sm">{t("music.settings.sleep")}</span>
                                <span aria-live="off" className="text-muted-foreground text-xs tabular-nums">
                                    <SleepStatus t={t} />
                                </span>
                            </div>
                            <fieldset aria-label={t("music.settings.sleepGroup")} className="inline-flex w-fit overflow-hidden rounded-lg border border-border">
                                {[...SLEEP_CHOICES, 0 as const].map((m) => (
                                    <button
                                        aria-pressed={choice === m}
                                        className={cn(
                                            "cursor-pointer border-border border-l px-3 py-1.5 font-mono text-xs uppercase outline-none transition-colors first:border-l-0 hover:bg-secondary/60 focus-visible:ring-2 focus-visible:ring-ring/40 focus-visible:ring-inset max-sm:min-h-11",
                                            choice === m ? "bg-primary text-primary-foreground hover:bg-primary" : "text-muted-foreground",
                                        )}
                                        key={m}
                                        onClick={() => pickSleep(m)}
                                        type="button"
                                    >
                                        {m === 0 ? t("music.settings.sleepChoiceOff") : t("music.settings.sleepChoice", { minutes: m })}
                                    </button>
                                ))}
                            </fieldset>
                        </div>
                    </Section>

                    <Section title={t("music.settings.section.volume")}>
                        <Row control={<Switch aria-labelledby="music-leveling-label" checked={settings.levelingOn} id="music-leveling" onCheckedChange={(v) => updateSettings({ levelingOn: v })} />} hint={t("music.settings.levelingHint")} id="music-leveling" label={t("music.settings.leveling")} />
                    </Section>

                    <Section title={t("music.settings.section.playback")}>
                        <Row control={<Switch aria-labelledby="music-preload-label" checked={preloadIsOn(settings)} id="music-preload" onCheckedChange={(v) => updateSettings({ preload: v ? "on" : "off" })} />} hint={t("music.settings.preloadHint")} id="music-preload" label={t("music.settings.preload")} />
                        <Row control={<Switch aria-labelledby="music-resume-label" checked={settings.resumePosition} id="music-resume" onCheckedChange={(v) => updateSettings({ resumePosition: v })} />} hint={t("music.settings.resumeHint")} id="music-resume" label={t("music.settings.resume")} />
                    </Section>

                    <Section title={t("music.settings.section.keyboard")}>
                        <div className="flex items-center justify-between gap-4">
                            <span className="text-foreground text-sm">{t("music.settings.keyboardHint")}</span>
                            <Button
                                onClick={() => {
                                    setSettingsOpen(false);
                                    setShortcutsOpen(true);
                                }}
                                size="sm"
                                variant="outline"
                            >
                                {t("music.settings.show")}
                            </Button>
                        </div>
                    </Section>
                </DialogPanel>
            </DialogPopup>
        </Dialog>
    );
}

function Keys({ keys }: { keys: readonly string[] }): React.ReactElement {
    return (
        <span className="inline-flex items-center gap-1">
            {keys.map((k, i) => (
                <Fragment key={k}>
                    {i > 0 && <span className="text-muted-foreground text-xs">+</span>}
                    <Kbd>{k}</Kbd>
                </Fragment>
            ))}
        </span>
    );
}

/** Rendered from `SHORTCUTS`, the list the keydown handler implements, so the two cannot drift apart. */
export function ShortcutsDialog(): React.ReactElement {
    const t: PlayerT = useT("gallery");
    const label: TypedT<typeof shortcutMessages> = useT("gallery");
    const open = usePlayer((s) => s.shortcutsOpen);
    return (
        <Dialog onOpenChange={setShortcutsOpen} open={open}>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>{t("music.shortcut.title")}</DialogTitle>
                </DialogHeader>
                <DialogPanel>
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="text-left font-mono text-[10px] text-muted-foreground uppercase tracking-[0.14em]">
                                <th className="pb-2 font-normal">{t("music.shortcut.action")}</th>
                                <th className="pb-2 text-right font-normal">{t("music.shortcut.keys")}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {SHORTCUTS.map((s) => (
                                <tr className="border-border border-t" key={s.id}>
                                    <td className="py-2 pr-4 text-foreground">{label(s.id as keyof typeof shortcutMessages & string)}</td>
                                    <td className="py-2 text-right">
                                        <span className="inline-flex flex-wrap items-center justify-end gap-1.5">
                                            <Keys keys={s.keys} />
                                            {s.alt && (
                                                <>
                                                    <span className="text-muted-foreground text-xs">{t("music.shortcut.or")}</span>
                                                    <Keys keys={s.alt} />
                                                </>
                                            )}
                                        </span>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </DialogPanel>
            </DialogPopup>
        </Dialog>
    );
}
