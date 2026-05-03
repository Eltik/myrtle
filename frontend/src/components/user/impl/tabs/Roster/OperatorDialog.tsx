import { useEffect, useMemo, useRef, useState } from "react";

import { eliteIcon, potentialIcon } from "#/components/operators/detail/impl/assets";
import { DialogContent, DialogTitle } from "#/components/ui/dialog";
import { Separator } from "#/components/ui/separator";
import { capitalize, formatProfession } from "#/lib/utils";

import { getAttributeStats, getTrustPercent, moduleIconURL, ownedHeroURL, rarityIcon, skillIconURL, specializedIcon } from "./helpers.card";
import type { IOwnedEntry } from "./types";

function voiceLabel(v: string | null): string {
    switch (v) {
        case "JP":
            return "Japanese";
        case "CN_MANDARIN":
            return "Chinese";
        case "EN":
            return "English";
        case "KR":
            return "Korean";
        case "CN_TOPOLECT":
            return "CN Regional";
        case "LINKAGE":
            return "Collab";
        case "ITA":
            return "Italian";
        case "RUS":
            return "Russian";
        default:
            return capitalize(v?.toLowerCase().replace(/_/g, " ") ?? "Japanese");
    }
}

export function OperatorDialog({ entry }: { entry: IOwnedEntry }) {
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

            // Title fades and lifts faster than the image — gone by ~30% scroll
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
                        <img alt={entry.name} className="h-full w-full object-contain object-top" decoding="async" loading="eager" src={ownedHeroURL(entry)} />
                    </div>
                    <div ref={shadeRef} className="pointer-events-none absolute inset-0 bg-linear-to-t from-background via-background/70 to-transparent" style={{ opacity: 0.32 }} />
                    <div className="pointer-events-none absolute inset-y-0 left-0 w-16 bg-linear-to-r from-background to-transparent" />
                    <div className="pointer-events-none absolute inset-y-0 right-0 w-16 bg-linear-to-l from-background to-transparent" />
                    <div ref={titleRef} className="absolute inset-x-0 bottom-0 px-6 pb-6" style={{ transform: "translate3d(0, 0, 0)", opacity: 1 }}>
                        <img alt={`${star} Star`} className="mb-2 h-5 w-auto object-contain drop-shadow" decoding="async" src={rarityIcon(star)} />
                        <h2 className="text-3xl font-bold tracking-tight text-foreground">{entry.name}</h2>
                        {op && <p className="mt-1 text-sm text-muted-foreground">{formatProfession(op.profession)}</p>}
                    </div>
                </div>
                <div ref={pillsRef} className="relative z-5 -mt-6 px-6" style={{ transform: "translate3d(0, 0, 0)" }}>
                    <div className="grid grid-cols-4 gap-2 rounded-xl border border-border/60 bg-card/80 p-2 shadow-lg backdrop-blur">
                        <div className="flex flex-col items-center justify-center gap-1 rounded-lg bg-muted/40 px-2 py-2">
                            <img alt={`Elite ${entry.elite}`} className="h-6 w-6 object-contain icon-theme-aware" decoding="async" src={eliteIcon(entry.elite)} />
                            <span className="text-[0.625rem] uppercase tracking-wide text-muted-foreground">Elite</span>
                        </div>
                        <div className="flex flex-col items-center justify-center gap-1 rounded-lg bg-muted/40 px-2 py-2">
                            <span className="text-base font-semibold tabular-nums text-foreground">{entry.level}</span>
                            <span className="text-[0.625rem] uppercase tracking-wide text-muted-foreground">Level</span>
                        </div>
                        <div className="flex flex-col items-center justify-center gap-1 rounded-lg bg-muted/40 px-2 py-2">
                            <img alt={`Potential ${entry.potential + 1}`} className="h-6 w-6 object-contain" decoding="async" src={potentialIcon(entry.potential)} />
                            <span className="text-[0.625rem] uppercase tracking-wide text-muted-foreground">Potential</span>
                        </div>
                        <div className="flex flex-col items-center justify-center gap-1 rounded-lg bg-muted/40 px-2 py-2">
                            <span className="text-base font-semibold tabular-nums text-foreground">{`${trustPct}%`}</span>
                            <span className="text-[0.625rem] uppercase tracking-wide text-muted-foreground">Trust</span>
                        </div>
                    </div>
                </div>

                <div className="space-y-6 px-6 pt-6 pb-6">
                    {stats && (
                        <section>
                            <header className="mb-3 flex items-center gap-3">
                                <span className="h-4 w-1 rounded-full bg-primary" />
                                <h3 className="text-xs font-semibold uppercase tracking-wider text-foreground">Combat Stats</h3>
                                <Separator className="flex-1" />
                            </header>
                            <div className="grid grid-cols-2 gap-1.5 sm:grid-cols-3">
                                {(
                                    [
                                        ["HP", stats.maxHp],
                                        ["ATK", stats.atk],
                                        ["DEF", stats.def],
                                        ["RES", stats.magicResistance],
                                        ["DP", stats.cost],
                                        ["Block", stats.blockCnt],
                                    ] as const
                                ).map(([label, value]) => (
                                    <div key={label} className="flex items-center justify-between rounded-md bg-muted/30 px-2.5 py-1.5">
                                        <span className="text-xs text-muted-foreground">{label}</span>
                                        <span className="text-sm font-semibold tabular-nums">{value}</span>
                                    </div>
                                ))}
                            </div>
                        </section>
                    )}
                    <section>
                        <header className="mb-3 flex items-center gap-3">
                            <span className="h-4 w-1 rounded-full bg-primary" />
                            <h3 className="text-xs font-semibold uppercase tracking-wider text-foreground">Skills</h3>
                            <span className="rounded-full border border-border/60 bg-muted/40 px-2 py-0.5 text-[0.625rem] uppercase tracking-wide text-muted-foreground">Lv. {entry.skill_level}</span>
                            <Separator className="flex-1" />
                        </header>
                        {skills.length > 0 ? (
                            <div className="space-y-1.5">
                                {skills.map(({ skill, index, mastery }) => {
                                    const isDefault = entry.default_skill === index;
                                    const name = skill.static?.levels?.[0]?.name ?? `Skill ${index + 1}`;
                                    return (
                                        <div key={skill.skillId} className={`grid items-center gap-2.5 rounded-md px-2.5 py-1.5 transition-colors ${isDefault ? "border border-primary/40 bg-primary/10" : "bg-muted/30 hover:bg-muted/50"}`} style={{ gridTemplateColumns: "32px minmax(0, 1fr) auto" }}>
                                            <img alt="Skill" className="h-8 w-8 rounded-sm" decoding="async" src={skillIconURL(skill)} />
                                            <div className="min-w-0">
                                                <p className="truncate text-sm font-medium" title={name}>
                                                    {name}
                                                </p>
                                                {isDefault && <span className="text-[0.625rem] uppercase tracking-wide text-primary">Default</span>}
                                            </div>
                                            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                                                <span className="tabular-nums">Lv.{entry.skill_level}</span>
                                                {mastery > 0 && <img alt={`M${mastery}`} className="h-4 w-4" decoding="async" src={specializedIcon(mastery)} />}
                                            </div>
                                        </div>
                                    );
                                })}
                            </div>
                        ) : (
                            <p className="rounded-md bg-muted/30 px-3 py-2 text-xs text-muted-foreground">{op ? "No skills found." : "Loading…"}</p>
                        )}
                    </section>
                    <section>
                        <header className="mb-3 flex items-center gap-3">
                            <span className="h-4 w-1 rounded-full bg-primary" />
                            <h3 className="text-xs font-semibold uppercase tracking-wider text-foreground">Modules</h3>
                            <Separator className="flex-1" />
                        </header>
                        {modules.length > 0 ? (
                            <div className="space-y-1.5">
                                {modules.map(({ module, level, isEquipped }) => (
                                    <div key={module.uniEquipId} className={`flex items-center gap-2.5 rounded-md px-2.5 py-1.5 transition-colors ${isEquipped ? "border border-primary/40 bg-primary/10" : "bg-muted/30 hover:bg-muted/50"}`}>
                                        <img alt="Module" className="h-8 w-8 shrink-0 object-contain" decoding="async" src={moduleIconURL(module)} />
                                        <div className="min-w-0 flex-1">
                                            <p className="truncate text-sm font-medium" title={module.uniEquipName}>
                                                {module.uniEquipName}
                                            </p>
                                            {isEquipped && <span className="text-[0.625rem] uppercase tracking-wide text-primary">Equipped</span>}
                                        </div>
                                        <div className="flex shrink-0 items-center gap-1.5 text-xs text-muted-foreground">
                                            <span className="rounded bg-background/60 px-1.5 py-0.5">{module.typeName1}</span>
                                            <span className="tabular-nums">Lv.{level}</span>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        ) : (
                            <p className="rounded-md bg-muted/30 px-3 py-2 text-xs text-muted-foreground">{op ? "No modules unlocked." : "Loading…"}</p>
                        )}
                    </section>
                    <section>
                        <header className="mb-3 flex items-center gap-3">
                            <span className="h-4 w-1 rounded-full bg-primary" />
                            <h3 className="text-xs font-semibold uppercase tracking-wider text-foreground">Info</h3>
                            <Separator className="flex-1" />
                        </header>
                        <div className="grid grid-cols-2 gap-1.5">
                            {(
                                [
                                    ["Recruited", entry.obtained_at ? new Date(entry.obtained_at * 1000).toLocaleDateString() : "Unknown"],
                                    ["Voice", voiceLabel(entry.voice_lan)],
                                ] as const
                            ).map(([label, value]) => (
                                <div key={label} className="flex items-center justify-between rounded-md bg-muted/30 px-2.5 py-1.5">
                                    <span className="text-xs text-muted-foreground">{label}</span>
                                    <span className="text-sm font-medium">{value}</span>
                                </div>
                            ))}
                        </div>
                    </section>
                </div>
            </div>
        </DialogContent>
    );
}
