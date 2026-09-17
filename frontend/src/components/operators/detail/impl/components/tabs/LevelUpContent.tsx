import { useQuery } from "@tanstack/react-query";
import { ArrowUpRight, Box, ChevronDown, ChevronsUp, Sigma, Sparkles, Star, TrendingUp } from "lucide-react";
import { memo, useMemo, useState } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useOperatorName } from "#/hooks/use-operator-name";
import { operatorGamedataServer } from "#/lib/api/gamedata";
import { type IMaterials, materialsQueryOptions } from "#/lib/api/materials";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { ILevelUpCostItem, IOperatorListItem, IUnlockCondition } from "#/types/operators";
import { asset, itemIcon } from "../../assets";
import type { messages } from "./LevelUpContent.messages";

interface ILevelUpContentProps {
    operator: IOperatorListItem;
}

type LevelUpT = TypedT<typeof messages>;

const PHASE_MESSAGE_KEY: Record<IUnlockCondition["phase"], keyof typeof messages & string> = {
    PHASE_0: "levelup.phase.e0",
    PHASE_1: "levelup.phase.e1",
    PHASE_2: "levelup.phase.e2",
};

function aggregate(items: ILevelUpCostItem[]): ILevelUpCostItem[] {
    const map = new Map<string, ILevelUpCostItem>();
    for (const it of items) {
        const prev = map.get(it.id);
        if (prev) prev.count += it.count;
        else map.set(it.id, { ...it });
    }
    return [...map.values()].sort((a, b) => {
        const aPrio = a.id === "4001" ? 0 : a.id === "5001" ? 1 : 2;
        const bPrio = b.id === "4001" ? 0 : b.id === "5001" ? 1 : 2;
        if (aPrio !== bPrio) return aPrio - bPrio;
        return b.count - a.count;
    });
}

function ItemTile({ item, materials, server }: { item: ILevelUpCostItem; materials: IMaterials | undefined; server?: "en" | "cn" }) {
    const t: LevelUpT = useT("operators");
    const f = useFormatters();
    const name = materials?.items[item.id]?.name ?? item.id;
    const src = itemIcon(item.id, item.iconId, item.image, server);
    // Five-figure counts are shortened so the badge on a 56px tile stays legible.
    const shortCount = item.count >= 10_000 ? t("levelup.countThousands", { value: f.number(Math.round(item.count / 1000)) }) : f.number(item.count);
    return (
        <Tooltip>
            <TooltipTrigger
                render={(props) => (
                    <div {...props} className="group relative h-14 w-14 cursor-help overflow-hidden rounded-lg border border-border bg-muted/40 transition-colors hover:border-primary/60">
                        <img
                            alt=""
                            className="h-full w-full object-contain p-1"
                            loading="lazy"
                            src={src}
                            onError={(e) => {
                                (e.currentTarget as HTMLImageElement).style.visibility = "hidden";
                            }}
                        />
                        <span className="absolute right-0 bottom-0 whitespace-nowrap rounded-tl-md bg-background/85 px-1 font-semibold text-[10px] text-foreground leading-tight">{t("levelup.itemCount", { count: shortCount })}</span>
                    </div>
                )}
            />
            <TooltipPopup className="p-0">
                <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-md border border-border bg-muted/40">
                        <img alt="" className="h-full w-full object-contain p-0.5" loading="lazy" src={src} />
                    </div>
                    <div className="flex flex-col">
                        <span className="font-semibold text-foreground text-sm leading-tight">{name}</span>
                        <span className="text-muted-foreground text-xs">{t("levelup.itemCount", { count: f.number(item.count) })}</span>
                    </div>
                </div>
            </TooltipPopup>
        </Tooltip>
    );
}

function CostRow({ label, hint, items, materials, server }: { label: string; hint?: string; items: ILevelUpCostItem[]; materials: IMaterials | undefined; server?: "en" | "cn" }) {
    if (!items.length) return null;
    return (
        <div className="flex flex-col gap-2 border-border/60 border-b px-5 py-3 last:border-b-0 sm:flex-row sm:items-center sm:gap-5">
            <div className="flex w-full shrink-0 items-baseline justify-between gap-2 sm:w-32 sm:flex-col sm:items-start sm:justify-start">
                <span className="font-medium text-foreground text-sm">{label}</span>
                {hint && <span className="text-muted-foreground text-xs">{hint}</span>}
            </div>
            <div className="flex flex-wrap gap-2.5">
                {items.map((it) => (
                    <ItemTile item={it} key={it.id} materials={materials} server={server} />
                ))}
            </div>
        </div>
    );
}

function Section({ icon, title, subtitle, children, className }: { icon: React.ReactNode; title: string; subtitle?: string; children: React.ReactNode; className?: string }) {
    const [open, setOpen] = useState(true);
    return (
        <Collapsible className={cn("overflow-hidden rounded-xl border border-border bg-card", className)} onOpenChange={setOpen} open={open} render={<section />}>
            <CollapsibleTrigger className={cn("flex w-full items-center gap-3 px-5 py-3.5 text-left transition-colors hover:bg-muted/30", open && "border-border border-b")}>
                <div className="flex h-7 w-7 items-center justify-center rounded-md bg-primary/10 text-primary">{icon}</div>
                <div className="min-w-0 flex-1">
                    <h3 className="font-semibold text-foreground text-sm tracking-tight">{title}</h3>
                    {subtitle && <p className="truncate text-muted-foreground text-xs">{subtitle}</p>}
                </div>
                <ChevronDown className={cn("h-4 w-4 shrink-0 text-muted-foreground transition-transform", open && "rotate-180")} />
            </CollapsibleTrigger>
            <CollapsibleContent>{children}</CollapsibleContent>
        </Collapsible>
    );
}

export const LevelUpContent = memo(function LevelUpContent({ operator }: ILevelUpContentProps) {
    const t: LevelUpT = useT("operators");
    const operatorName = useOperatorName();
    const [activeSkill, setActiveSkill] = useState(0);
    const localeServer = useGamedataServer();
    const { data: materials } = useQuery(materialsQueryOptions(operatorGamedataServer(operator.server, localeServer)));

    const elitePromotions = useMemo(() => operator.phases.map((p, idx) => ({ to: idx, cost: p.evolveCost ?? [] })).filter((r) => r.cost.length > 0), [operator.phases]);
    const levelingRows = useMemo(() => operator.phases.map((p, idx) => ({ phase: idx, maxLevel: p.maxLevel, cost: p.levelUpCost ?? [] })).filter((r) => r.cost.length > 0), [operator.phases]);
    const allSkillRows = useMemo(() => operator.allSkillLevelUp.filter((l) => l.lvlUpCost?.length), [operator.allSkillLevelUp]);
    const masterySkills = useMemo(() => operator.skills.filter((s) => s.levelUpCostCond?.length), [operator.skills]);
    const validModules = useMemo(() => operator.modules.filter((m) => m.itemCost && Object.keys(m.itemCost).length > 0), [operator.modules]);

    const grandTotal = useMemo(() => {
        const all: ILevelUpCostItem[] = [
            ...operator.phases.flatMap((p) => p.evolveCost ?? []),
            ...operator.phases.flatMap((p) => p.levelUpCost ?? []),
            ...operator.allSkillLevelUp.flatMap((l) => l.lvlUpCost ?? []),
            ...operator.skills.flatMap((s) => s.levelUpCostCond.flatMap((c) => c.levelUpCost ?? [])),
            ...operator.modules.flatMap((m) =>
                m.itemCost
                    ? Object.values(m.itemCost)
                          .flat()
                          .filter((c) => c !== undefined)
                    : [],
            ),
        ];
        return aggregate(all);
    }, [operator.phases, operator.allSkillLevelUp, operator.skills, operator.modules]);

    if (!grandTotal.length) {
        return (
            <div className="min-w-0 overflow-hidden p-4 md:p-6">
                <section className="overflow-hidden rounded-xl border border-border bg-card">
                    <div className="flex flex-col items-center justify-center gap-3 px-6 py-16 text-center">
                        <div className="grid h-12 w-12 place-items-center rounded-full bg-muted text-muted-foreground">
                            <Star className="h-5 w-5" />
                        </div>
                        <div>
                            <h3 className="font-semibold text-foreground text-sm">{t("levelup.empty.title")}</h3>
                            <p className="mt-1 text-muted-foreground text-xs">{t("levelup.empty.body", { name: operatorName(operator) })}</p>
                        </div>
                    </div>
                </section>
            </div>
        );
    }

    const activeMastery = masterySkills[Math.min(activeSkill, masterySkills.length - 1)];
    const activeMasteryName = activeMastery?.static?.levels?.[0]?.name ?? t("levelup.skillFallback", { index: activeSkill + 1 });

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">{t("levelup.title")}</h2>
                <p className="text-muted-foreground text-sm">{t("levelup.subtitle")}</p>
            </div>

            <div className="flex flex-col gap-5">
                {levelingRows.length > 0 && (
                    <Section icon={<TrendingUp className="h-4 w-4" />} subtitle={t("levelup.section.operatorLevel.desc")} title={t("levelup.section.operatorLevel")}>
                        {levelingRows.map((r) => (
                            <CostRow hint={t("levelup.row.levelRange", { max: r.maxLevel })} items={r.cost} key={r.phase} label={t("levelup.row.elite", { phase: r.phase })} materials={materials} />
                        ))}
                    </Section>
                )}

                {elitePromotions.length > 0 && (
                    <Section icon={<ChevronsUp className="h-4 w-4" />} subtitle={t("levelup.section.elite.desc")} title={t("levelup.section.elite")}>
                        {elitePromotions.map((r) => (
                            <CostRow hint={t("levelup.row.promoteTo", { to: r.to })} items={r.cost} key={r.to} label={t("levelup.row.elite", { phase: r.to })} materials={materials} server={operator.server} />
                        ))}
                    </Section>
                )}

                {allSkillRows.length > 0 && (
                    <Section icon={<ArrowUpRight className="h-4 w-4" />} subtitle={t("levelup.section.skillLevels.desc")} title={t("levelup.section.skillLevels")}>
                        {allSkillRows.map((r, i) => (
                            <CostRow
                                hint={t("levelup.row.unlockAt", { phase: t(PHASE_MESSAGE_KEY[r.unlockCond.phase]), level: r.unlockCond.level })}
                                items={r.lvlUpCost}
                                key={`${r.unlockCond.phase}-${r.unlockCond.level}`}
                                label={t("levelup.row.skillStep", { from: i + 1, to: i + 2 })}
                                materials={materials}
                                server={operator.server}
                            />
                        ))}
                    </Section>
                )}

                {activeMastery && (
                    <Section icon={<Sparkles className="h-4 w-4" />} subtitle={t("levelup.section.mastery.desc")} title={t("levelup.section.mastery")}>
                        {masterySkills.length > 1 && (
                            <div className="flex gap-1.5 overflow-x-auto border-border border-b bg-muted/30 px-5 py-2.5">
                                {masterySkills.map((s, i) => (
                                    <button
                                        className={cn("flex shrink-0 items-center gap-2 whitespace-nowrap rounded-md px-3 py-1.5 font-medium text-xs transition-colors", i === activeSkill ? "bg-primary/15 text-primary" : "text-muted-foreground hover:bg-accent hover:text-foreground")}
                                        key={s.skillId}
                                        onClick={() => setActiveSkill(i)}
                                        type="button"
                                    >
                                        <span className="grid h-5 w-5 shrink-0 place-items-center rounded bg-background/60 font-bold text-[10px]">{i + 1}</span>
                                        <span>{s.static?.levels?.[0]?.name ?? t("levelup.skillFallback", { index: i + 1 })}</span>
                                    </button>
                                ))}
                            </div>
                        )}
                        <div className="px-5 pt-3 pb-1 text-muted-foreground text-xs uppercase tracking-wider">{activeMasteryName}</div>
                        {activeMastery.levelUpCostCond.map((c, i) => (
                            <CostRow
                                hint={t("levelup.row.unlockAt", { phase: t(PHASE_MESSAGE_KEY[c.unlockCond.phase]), level: c.unlockCond.level })}
                                items={c.levelUpCost}
                                key={`${c.unlockCond.phase}-${c.unlockCond.level}`}
                                label={t("levelup.row.mastery", { index: i + 1 })}
                                materials={materials}
                                server={operator.server}
                            />
                        ))}
                    </Section>
                )}

                {validModules.length > 0 && (
                    <Section icon={<Box className="h-4 w-4" />} subtitle={t("levelup.section.modules.desc")} title={t("levelup.section.modules")}>
                        <div className="divide-y divide-border">
                            {validModules.map((m) => (
                                <div className="px-5 py-4" key={m.uniEquipId}>
                                    <div className="mb-3 flex items-center gap-3">
                                        <div className="grid h-9 w-9 shrink-0 place-items-center overflow-hidden rounded-md border border-border bg-muted/40">
                                            {m.image ? <img alt="" className="h-full w-full object-contain" loading="lazy" src={asset(m.image, operator.server)} /> : <span className="font-bold text-[10px] text-primary uppercase">{m.typeName2 ?? m.typeName1}</span>}
                                        </div>
                                        <div className="min-w-0">
                                            <div className="truncate font-semibold text-foreground text-sm">{m.uniEquipName}</div>
                                            <div className="text-muted-foreground text-xs">
                                                {m.typeName1}
                                                {m.typeName2 ? ` · ${m.typeName2}` : ""}
                                            </div>
                                        </div>
                                    </div>
                                    <div className="space-y-1 rounded-lg border border-border/60 bg-background/40">
                                        {Object.entries(m.itemCost ?? {})
                                            .sort(([a], [b]) => Number(a) - Number(b))
                                            .map(([stage, items]) => (
                                                <CostRow items={items ?? []} key={stage} label={t("levelup.row.stage", { stage })} materials={materials} server={operator.server} />
                                            ))}
                                    </div>
                                </div>
                            ))}
                        </div>
                    </Section>
                )}

                <Section icon={<Sigma className="h-4 w-4" />} subtitle={t("levelup.section.total.desc")} title={t("levelup.section.total")}>
                    <div className="px-5 py-4">
                        <div className="flex flex-wrap gap-2.5">
                            {grandTotal.map((it) => (
                                <ItemTile item={it} key={it.id} materials={materials} server={operator.server} />
                            ))}
                        </div>
                    </div>
                </Section>
            </div>
        </div>
    );
});
