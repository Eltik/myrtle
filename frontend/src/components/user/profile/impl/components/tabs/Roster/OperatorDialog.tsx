import { useEffect, useMemo, useRef, useState } from "react";

import { eliteIcon, potentialIcon } from "#/components/operators/detail/impl/assets";
import { DialogContent, DialogTitle } from "#/components/ui/dialog";
import { Separator } from "#/components/ui/separator";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { capitalize, formatProfession } from "#/lib/utils";

import { DynamicArtOverlay } from "../../../DynamicArtOverlay";
import { getAttributeStats, getTrustPercent, moduleIconURL, moduleTypeLabel, ownedHeroURL, rarityIcon, skillIconURL, specializedIcon } from "./helpers.card";
import type { messages as cardMessages } from "./helpers.card.messages";
import type { messages } from "./OperatorDialog.messages";
import type { IOwnedEntry } from "./types";

/** The shared card vocabulary is declared in `helpers.card.messages.ts`. */
type DialogT = TypedT<typeof messages & typeof cardMessages>;

/** `v` is the API's voice-language code; an unknown one is shown as-is. */
function voiceLabel(v: string | null, t: DialogT): string {
    switch (v) {
        case "JP":
            return t("profile.roster.voice.jp");
        case "CN_MANDARIN":
            return t("profile.roster.voice.cn");
        case "EN":
            return t("profile.roster.voice.en");
        case "KR":
            return t("profile.roster.voice.kr");
        case "CN_TOPOLECT":
            return t("profile.roster.voice.cnTopolect");
        case "LINKAGE":
            return t("profile.roster.voice.linkage");
        case "ITA":
            return t("profile.roster.voice.ita");
        case "RUS":
            return t("profile.roster.voice.rus");
        default:
            return v ? capitalize(v.toLowerCase().replace(/_/g, " ")) : t("profile.roster.voice.jp");
    }
}

/** Options matching what this row rendered before it took the page's locale. */
const RECRUITED_AT: Intl.DateTimeFormatOptions = { year: "numeric", month: "short", day: "numeric", hour: "numeric", minute: "2-digit" };

export function OperatorDialog({ entry }: { entry: IOwnedEntry }) {
    const t: DialogT = useT("user");
    const f = useFormatters();
    const op = entry.static;
    const star = entry.rarity;
    const stats = useMemo(() => (op ? getAttributeStats(entry, op) : null), [entry, op]);
    const trustPct = getTrustPercent(entry.favor_point);

    const skills = useMemo(() => {
        const masteryByIndex = new Map(entry.masteries.map((m) => [m.index, m.mastery]));

        return (op?.skills ?? []).map((skill, i) => ({
            skill,
            index: i,
            mastery: masteryByIndex.get(i) ?? 0,
        }));
    }, [entry.masteries, op?.skills]);

    const modules = useMemo(() => {
        const rosterModulesById = new Map(entry.modules.map((m) => [m.id, m]));

        return (op?.modules ?? [])
            .filter((m) => m.typeName1 !== "ORIGINAL")
            .map((module) => {
                const rosterMod = rosterModulesById.get(module.uniEquipId);

                return {
                    module,
                    level: rosterMod?.level ?? 0,
                    locked: rosterMod?.locked ?? true,
                    isEquipped: entry.current_equip === module.uniEquipId,
                };
            })
            .filter((m) => !m.locked && m.level > 0);
    }, [entry.current_equip, entry.modules, op?.modules]);

    const [scroller, setScroller] = useState<HTMLDivElement | null>(null);
    const [dynActive, setDynActive] = useState(false);
    const artRef = useRef<HTMLDivElement>(null);
    const shadeRef = useRef<HTMLDivElement>(null);
    const titleRef = useRef<HTMLDivElement>(null);
    const pillsRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const art = artRef.current;
        const shade = shadeRef.current;
        const title = titleRef.current;
        const pills = pillsRef.current;

        if (!scroller || !art || !shade || !title || !pills) return;
        if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

        let frame = 0;
        let last = -1;

        const render = () => {
            frame = 0;

            const max = scroller.scrollHeight - scroller.clientHeight;
            const progress = max > 0 ? Math.min(1, scroller.scrollTop / max) : 0;
            if (progress === last) return;

            last = progress;

            const artY = Math.round(progress * -64);
            const artScale = 1 - progress * 0.08;
            const shadeOpacity = Math.min(0.78, 0.32 + progress * 0.4);

            // Title fades and lifts faster than the image - gone by ~30% scroll
            const tp = Math.min(1, progress / 0.3);
            const titleY = Math.round(tp * -14);
            const titleOpacity = 1 - tp;

            // Pills lift gently with the page so the hero doesn't feel disconnected
            const pillsY = Math.round(progress * -8);

            art.style.transform = `translate3d(0, ${artY}px, 0) scale(${artScale})`;
            shade.style.opacity = `${shadeOpacity}`;
            title.style.transform = `translate3d(0, ${titleY}px, 0)`;
            title.style.opacity = `${titleOpacity}`;
            pills.style.transform = `translate3d(0, ${pillsY}px, 0)`;
        };

        const onScroll = () => {
            if (!frame) frame = requestAnimationFrame(render);
        };

        scroller.addEventListener("scroll", onScroll, { passive: true });
        render();

        return () => {
            scroller.removeEventListener("scroll", onScroll);
            if (frame) cancelAnimationFrame(frame);
        };
    }, [scroller]);

    return (
        <DialogContent className="max-h-[90vh] max-w-2xl overflow-hidden p-0">
            <DialogTitle className="sr-only">{entry.name}</DialogTitle>
            <div ref={setScroller} className="max-h-[90vh] overflow-y-auto" style={{ scrollbarGutter: "stable" }}>
                <div className="relative h-80 overflow-hidden bg-linear-to-br from-primary/10 via-background to-background">
                    <div ref={artRef} className="absolute inset-x-0 top-0 h-[140%] origin-top" style={{ transform: "translate3d(0, 0, 0) scale(1)" }}>
                        <img alt={entry.name} className={`h-full w-full object-contain object-top transition-opacity duration-500 ${dynActive ? "opacity-0" : "opacity-100"}`} decoding="async" loading="eager" src={ownedHeroURL(entry)} />
                        {/* Composited over the same hero art at the game's authored frame, cropped as the image above (contain, top). */}
                        <DynamicArtOverlay elite={entry.elite} operatorCode={entry.operator_id} skinId={entry.skin_id} fit={{ mode: "contain", align: "top" }} framing="authored" surface="panel" backdrop={ownedHeroURL(entry)} onActiveChange={setDynActive} />
                    </div>
                    <div ref={shadeRef} className="pointer-events-none absolute inset-0 bg-linear-to-t from-background via-background/70 to-transparent" style={{ opacity: 0.32 }} />
                    <div className="pointer-events-none absolute inset-y-0 left-0 w-16 bg-linear-to-r from-background to-transparent" />
                    <div className="pointer-events-none absolute inset-y-0 right-0 w-16 bg-linear-to-l from-background to-transparent" />
                    <div ref={titleRef} className="absolute inset-x-0 bottom-0 px-6 pb-6" style={{ transform: "translate3d(0, 0, 0)", opacity: 1 }}>
                        <img alt={t("profile.roster.card.rarityAlt", { star })} className="mb-2 h-5 w-auto object-contain drop-shadow" decoding="async" src={rarityIcon(star)} />
                        <h2 className="font-bold text-3xl text-foreground tracking-tight">{entry.name}</h2>
                        {op && <p className="mt-1 text-muted-foreground text-sm">{formatProfession(op.profession)}</p>}
                    </div>
                </div>
                <div ref={pillsRef} className="relative z-5 -mt-6 truncate px-6" style={{ transform: "translate3d(0, 0, 0)" }}>
                    <div className="grid grid-cols-4 gap-2 rounded-xl border border-border/60 bg-card/80 p-2 shadow-lg backdrop-blur">
                        <div className="flex flex-col items-center justify-center gap-1 rounded-lg bg-muted/40 px-2 py-2">
                            <img alt={t("profile.roster.card.eliteAlt", { elite: entry.elite })} className="icon-theme-aware h-6 w-6 object-contain" decoding="async" src={eliteIcon(entry.elite)} />
                            <span className="text-[0.625rem] text-muted-foreground uppercase tracking-wide">{t("profile.roster.dialog.elite")}</span>
                        </div>
                        <div className="flex flex-col items-center justify-center gap-1 rounded-lg bg-muted/40 px-2 py-2">
                            <span className="font-semibold text-base text-foreground tabular-nums">{entry.level}</span>
                            <span className="text-[0.625rem] text-muted-foreground uppercase tracking-wide">{t("profile.roster.card.level")}</span>
                        </div>
                        <div className="flex flex-col items-center justify-center gap-1 rounded-lg bg-muted/40 px-2 py-2">
                            <img alt={t("profile.roster.card.potentialAlt", { rank: entry.potential + 1 })} className="h-6 w-6 object-contain" decoding="async" src={potentialIcon(entry.potential)} />
                            <span className="text-[0.625rem] text-muted-foreground uppercase tracking-wide">{t("profile.roster.card.potential")}</span>
                        </div>
                        <div className="flex flex-col items-center justify-center gap-1 rounded-lg bg-muted/40 px-2 py-2">
                            <span className="font-semibold text-base text-foreground tabular-nums">{`${trustPct}%`}</span>
                            <span className="text-[0.625rem] text-muted-foreground uppercase tracking-wide">{t("profile.roster.card.trust")}</span>
                        </div>
                    </div>
                </div>

                <div className="space-y-6 px-6 pt-6 pb-6">
                    {stats && (
                        <section>
                            <header className="mb-3 flex items-center gap-3">
                                <span className="h-4 w-1 rounded-full bg-primary" />
                                <h3 className="font-semibold text-foreground text-xs uppercase tracking-wider">{t("profile.roster.dialog.combatStats")}</h3>
                                <Separator className="flex-1" />
                            </header>
                            <div className="grid grid-cols-2 gap-1.5 sm:grid-cols-3">
                                {(
                                    [
                                        ["profile.roster.card.stat.hp", stats.maxHp],
                                        ["profile.roster.card.stat.atk", stats.atk],
                                        ["profile.roster.card.stat.def", stats.def],
                                        ["profile.roster.card.stat.res", stats.magicResistance],
                                        ["profile.roster.card.stat.dp", stats.cost],
                                        ["profile.roster.card.stat.block", stats.blockCnt],
                                    ] as const
                                ).map(([labelKey, value]) => (
                                    <div key={labelKey} className="flex items-center justify-between rounded-md bg-muted/30 px-2.5 py-1.5">
                                        <span className="text-muted-foreground text-xs">{t(labelKey)}</span>
                                        <span className="font-semibold text-sm tabular-nums">{value}</span>
                                    </div>
                                ))}
                            </div>
                        </section>
                    )}
                    <section>
                        <header className="mb-3 flex items-center gap-3">
                            <span className="h-4 w-1 rounded-full bg-primary" />
                            <h3 className="font-semibold text-foreground text-xs uppercase tracking-wider">{t("profile.roster.card.skills")}</h3>
                            <span className="rounded-full border border-border/60 bg-muted/40 px-2 py-0.5 text-[0.625rem] text-muted-foreground uppercase tracking-wide">{t("profile.roster.dialog.skillLevelBadge", { level: entry.skill_level })}</span>
                            <Separator className="flex-1" />
                        </header>
                        {skills.length > 0 ? (
                            <div className="space-y-1.5">
                                {skills.map(({ skill, index, mastery }) => {
                                    const isDefault = entry.default_skill === index;
                                    const name = skill.static?.levels?.[0]?.name ?? t("profile.roster.card.skillFallback", { n: index + 1 });
                                    return (
                                        <div key={skill.skillId} className={`grid items-center gap-2.5 rounded-md px-2.5 py-1.5 transition-colors ${isDefault ? "border border-primary/40 bg-primary/10" : "bg-muted/30 hover:bg-muted/50"}`} style={{ gridTemplateColumns: "32px minmax(0, 1fr) auto" }}>
                                            <img alt={t("profile.roster.card.skillAlt")} className="h-8 w-8 rounded-sm" decoding="async" src={skillIconURL(skill)} />
                                            <div className="min-w-0">
                                                <p className="truncate font-medium text-sm" title={name}>
                                                    {name}
                                                </p>
                                                {isDefault && <span className="text-[0.625rem] text-primary uppercase tracking-wide">{t("profile.roster.dialog.default")}</span>}
                                            </div>
                                            <div className="flex items-center gap-1.5 text-muted-foreground text-xs">
                                                <span className="tabular-nums">{t("profile.roster.card.skillLevel", { level: entry.skill_level })}</span>
                                                {mastery > 0 && <img alt={t("profile.roster.card.masteryAlt", { mastery })} className="h-4 w-4" decoding="async" src={specializedIcon(mastery)} />}
                                            </div>
                                        </div>
                                    );
                                })}
                            </div>
                        ) : (
                            <p className="rounded-md bg-muted/30 px-3 py-2 text-muted-foreground text-xs">{op ? t("profile.roster.card.noSkills") : t("profile.roster.dialog.loading")}</p>
                        )}
                    </section>
                    <section>
                        <header className="mb-3 flex items-center gap-3">
                            <span className="h-4 w-1 rounded-full bg-primary" />
                            <h3 className="font-semibold text-foreground text-xs uppercase tracking-wider">{t("profile.roster.card.modules")}</h3>
                            <Separator className="flex-1" />
                        </header>
                        {modules.length > 0 ? (
                            <div className="space-y-1.5">
                                {modules.map(({ module, level, isEquipped }) => (
                                    <div key={module.uniEquipId} className={`flex items-center gap-2.5 rounded-md px-2.5 py-1.5 transition-colors ${isEquipped ? "border border-primary/40 bg-primary/10" : "bg-muted/30 hover:bg-muted/50"}`}>
                                        <img alt={t("profile.roster.card.moduleAlt")} className="h-8 w-8 shrink-0 object-contain" decoding="async" src={moduleIconURL(module)} />
                                        <div className="min-w-0 flex-1">
                                            <p className="truncate font-medium text-sm" title={module.uniEquipName}>
                                                {module.uniEquipName}
                                            </p>
                                            {isEquipped && <span className="text-[0.625rem] text-primary uppercase tracking-wide">{t("profile.roster.dialog.equipped")}</span>}
                                        </div>
                                        <div className="flex shrink-0 items-center gap-1.5 text-muted-foreground text-xs">
                                            <span className="rounded bg-background/60 px-1.5 py-0.5">{moduleTypeLabel(module)}</span>
                                            <span className="tabular-nums">{t("profile.roster.card.moduleLevel", { level })}</span>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        ) : (
                            <p className="rounded-md bg-muted/30 px-3 py-2 text-muted-foreground text-xs">{op ? t("profile.roster.card.noModules") : t("profile.roster.dialog.loading")}</p>
                        )}
                    </section>
                    <section>
                        <header className="mb-3 flex items-center gap-3">
                            <span className="h-4 w-1 rounded-full bg-primary" />
                            <h3 className="font-semibold text-foreground text-xs uppercase tracking-wider">{t("profile.roster.dialog.info")}</h3>
                            <Separator className="flex-1" />
                        </header>
                        <div className="grid grid-cols-1 gap-1.5 sm:grid-cols-2">
                            {(
                                [
                                    [t("profile.roster.dialog.recruited"), entry.obtained_at ? f.date(new Date(entry.obtained_at * 1000), RECRUITED_AT) : t("profile.roster.dialog.recruited.unknown")],
                                    [t("profile.roster.dialog.voice"), voiceLabel(entry.voice_lan, t)],
                                ] as const
                            ).map(([label, value]) => (
                                <div key={label} className="flex items-center justify-between gap-2 rounded-md bg-muted/30 px-2.5 py-1.5">
                                    <span className="shrink-0 text-muted-foreground text-xs">{label}</span>
                                    <span className="min-w-0 text-right font-medium text-sm">{value}</span>
                                </div>
                            ))}
                        </div>
                    </section>
                </div>
            </div>
        </DialogContent>
    );
}
