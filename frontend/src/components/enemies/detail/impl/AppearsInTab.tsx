import { useQuery } from "@tanstack/react-query";
import { ChevronDown } from "lucide-react";
import { useMemo, useState } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Skeleton } from "#/components/ui/skeleton";
import { enemyAppearsInQueryOptions, type IEnemyStageRef } from "#/lib/api/enemies";
import { zonesQueryOptions } from "#/lib/api/stages";
import { cn } from "#/lib/utils";
import type { IZone } from "#/types/stages";
import { SectionHead } from "./sections";

type Category = IEnemyStageRef["category"];

/** A single zone/event/season with its enemy appearances. */
interface IAppearanceGroup {
    zoneId: string;
    title: string;
    subtitle: string | null;
    type: IZone["type"] | null;
    zoneIndex: number;
    category: Category;
    refs: IEnemyStageRef[];
}

/** Top-level buckets, in display order. `defaultOpen` controls whether each
 *  event/season dropdown inside the bucket starts expanded. */
const CATEGORY_SECTIONS: { key: Category; label: string; defaultOpen: boolean }[] = [
    { key: "stages", label: "Story Stages", defaultOpen: true },
    { key: "events", label: "Events", defaultOpen: true },
    { key: "modes", label: "Permanent Game Modes", defaultOpen: true },
];

/** Within an event, split stages into named sub-sections (Story vs EX). */
function eventSubgroups(refs: IEnemyStageRef[]): { label: string; refs: IEnemyStageRef[] }[] {
    const story: IEnemyStageRef[] = [];
    const ex: IEnemyStageRef[] = [];
    for (const r of refs) {
        const isEx = /(?:^|[-_ ])ex[-_ ]?\d/i.test(r.code) || /_ex/i.test(r.stageId);
        (isEx ? ex : story).push(r);
    }
    const out: { label: string; refs: IEnemyStageRef[] }[] = [];
    if (story.length) out.push({ label: "Story", refs: story });
    if (ex.length) out.push({ label: "EX Stages", refs: ex });
    return out;
}

function zoneTitle(zone: IZone | undefined, zoneId: string): { title: string; subtitle: string | null } {
    if (!zone) return { title: zoneId, subtitle: null };
    const title = zone.zoneNameSecond || zone.zoneNameFirst || zone.zoneNameTitleCurrent || zoneId;
    const subtitle = zone.zoneNameFirst && zone.zoneNameFirst !== title ? zone.zoneNameFirst : null;
    return { title, subtitle };
}

function groupByZone(refs: IEnemyStageRef[], zonesById: Map<string, IZone>): IAppearanceGroup[] {
    const byZone = new Map<string, IEnemyStageRef[]>();
    for (const r of refs) {
        const list = byZone.get(r.zoneId);
        if (list) list.push(r);
        else byZone.set(r.zoneId, [r]);
    }

    const groups: IAppearanceGroup[] = [];
    for (const [zoneId, zoneRefs] of byZone) {
        const zone = zonesById.get(zoneId);
        const fallback = zoneTitle(zone, zoneId);
        // The backend resolves the display name across zone/activity/mode
        // tables; prefer it so events and modes never show a raw id.
        const title = zoneRefs.find((r) => r.zoneName)?.zoneName ?? fallback.title;
        groups.push({
            zoneId,
            title,
            subtitle: title === fallback.title ? fallback.subtitle : null,
            type: zone?.type ?? null,
            zoneIndex: zone?.zoneIndex ?? Number.MAX_SAFE_INTEGER,
            category: zoneRefs[0]?.category ?? "modes",
            refs: zoneRefs,
        });
    }

    groups.sort((a, b) => a.zoneIndex - b.zoneIndex || a.title.localeCompare(b.title));
    return groups;
}

const ZONE_TYPE_LABEL: Partial<Record<IZone["type"], string>> = {
    MAINLINE: "Main",
    MAINLINE_ACTIVITY: "Main",
    MAINLINE_RETRO: "Main",
    SIDESTORY: "Side Story",
    BRANCHLINE: "Branch",
    ACTIVITY: "Event",
    WEEKLY: "Weekly",
    CAMPAIGN: "Annihilation",
    CLIMB_TOWER: "S.S.S.",
    ROGUELIKE: "I.S.",
    GUIDE: "Guide",
    SPECIAL: "Special",
};

export function AppearsInTab({ enemyId }: { enemyId: string }) {
    // Fetched lazily when this tab is opened (the panel mounts on demand), rather
    // than warmed by the route loader. The refs carry a pre-resolved `zoneName`;
    // the zones table is still needed for group `type` labels + `zoneIndex` sort.
    const { data: refs = [], isLoading: refsLoading } = useQuery(enemyAppearsInQueryOptions(enemyId));
    const { data: zones = [], isLoading: zonesLoading } = useQuery(zonesQueryOptions());

    const { sections, totalStages, totalZones } = useMemo(() => {
        const zonesById = new Map(zones.map((z) => [z.zoneId, z]));
        const groups = groupByZone(refs, zonesById);
        const sections = CATEGORY_SECTIONS.map((s) => ({
            ...s,
            groups: groups.filter((g) => g.category === s.key),
        })).filter((s) => s.groups.length > 0);
        return { sections, totalStages: refs.length, totalZones: groups.length };
    }, [refs, zones]);

    if (refsLoading || zonesLoading) {
        return (
            <div className="flex flex-col gap-2">
                <Skeleton className="h-12 w-full rounded-xl" />
                <Skeleton className="h-12 w-full rounded-xl" />
                <Skeleton className="h-12 w-full rounded-xl" />
            </div>
        );
    }

    if (sections.length === 0) {
        return (
            <div className="flex flex-col items-center justify-center rounded-[14px] border border-border border-dashed bg-card/50 px-6 py-14 text-center">
                <h3 className="m-0 font-sans font-semibold text-[15px] text-foreground leading-none">No appearances recorded</h3>
                <p className="m-0 mt-2 max-w-96 text-pretty font-sans text-[13px] text-muted-foreground leading-normal">This enemy isn't listed in any of the currently extracted level files. Coverage grows as more content is processed.</p>
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-6">
            <p className="m-0 font-sans text-[12.5px] text-muted-foreground leading-normal">
                Found in <strong className="text-foreground">{totalStages}</strong> {totalStages === 1 ? "location" : "locations"} across <strong className="text-foreground">{totalZones}</strong> {totalZones === 1 ? "zone" : "zones"}.
            </p>
            {sections.map((section) => (
                <section key={section.key}>
                    <SectionHead>
                        {section.label} · {section.groups.length}
                    </SectionHead>
                    <div className="flex flex-col gap-2">
                        {section.groups.map((g) => (
                            <ZoneCollapsible key={g.zoneId} group={g} defaultOpen={section.defaultOpen} />
                        ))}
                    </div>
                </section>
            ))}
        </div>
    );
}

/** One event/season/mode as a collapsible: name in the header, stages inside. */
function ZoneCollapsible({ group, defaultOpen }: { group: IAppearanceGroup; defaultOpen: boolean }) {
    const [open, setOpen] = useState(defaultOpen);
    const typeLabel = group.type ? ZONE_TYPE_LABEL[group.type] : null;

    return (
        <Collapsible open={open} onOpenChange={setOpen}>
            <div className="overflow-hidden rounded-xl border border-border bg-[color-mix(in_oklch,var(--muted)_22%,transparent)]">
                <CollapsibleTrigger className="flex w-full items-center gap-2.5 px-3 py-2.5 text-left transition-colors hover:bg-[color-mix(in_oklch,var(--muted)_36%,transparent)] sm:px-3.5">
                    <ChevronDown className={cn("h-4 w-4 shrink-0 text-muted-foreground transition-transform duration-200 motion-reduce:transition-none", open && "rotate-180")} aria-hidden="true" />
                    <div className="flex min-w-0 flex-1 flex-col gap-0.5">
                        <h4 className="m-0 truncate font-sans font-semibold text-[13.5px] text-foreground leading-tight">{group.title}</h4>
                        {group.subtitle && <span className="truncate font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{group.subtitle}</span>}
                    </div>
                    {typeLabel && <span className="shrink-0 rounded-full border border-border bg-card px-2 py-0.75 font-medium font-mono text-[9.5px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{typeLabel}</span>}
                    <span className="shrink-0 rounded-full bg-[color-mix(in_oklch,var(--muted)_60%,transparent)] px-1.75 py-0.75 font-medium font-mono text-[10px] text-muted-foreground tabular-nums leading-none">{group.refs.length}</span>
                </CollapsibleTrigger>
                <CollapsibleContent>
                    <div className="border-border/60 border-t px-3 py-3 sm:px-3.5">
                        <ZoneBody group={group} />
                    </div>
                </CollapsibleContent>
            </div>
        </Collapsible>
    );
}

/** Event bodies split into Story / EX sub-sections; other zones are a flat
 *  chip grid. */
function ZoneBody({ group }: { group: IAppearanceGroup }) {
    const subgroups = group.category === "events" ? eventSubgroups(group.refs) : [];
    if (subgroups.length > 1) {
        return (
            <div className="flex flex-col gap-3">
                {subgroups.map((sg) => (
                    <div key={sg.label}>
                        <div className="mb-1.5 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{sg.label}</div>
                        <ChipRow refs={sg.refs} />
                    </div>
                ))}
            </div>
        );
    }
    return <ChipRow refs={group.refs} />;
}

function ChipRow({ refs }: { refs: IEnemyStageRef[] }) {
    return (
        <div className="flex flex-wrap gap-1.5">
            {refs.map((r) => (
                <StageChip key={`${r.stageId}-${r.code}-${r.isHard ? "h" : "n"}`} stage={r} />
            ))}
        </div>
    );
}

function StageChip({ stage }: { stage: IEnemyStageRef }) {
    const title = [stage.stageName, stage.isHard ? "(Adverse)" : null, stage.count > 0 ? `${stage.count} spawn${stage.count === 1 ? "" : "s"}` : "summoned / conditional"].filter(Boolean).join(" · ");
    return (
        <span title={title} className="inline-flex items-center gap-1.5 rounded-md border bg-card px-2 py-1 font-medium font-mono text-[11.5px] text-foreground leading-none" style={{ borderColor: stage.isHard ? "color-mix(in oklch, var(--destructive) 45%, var(--border))" : "var(--border)" }}>
            <span className="tabular-nums">{stage.code || stage.stageId}</span>
            {stage.isHard && <span className="rounded-[3px] bg-[color-mix(in_oklch,var(--destructive)_18%,transparent)] px-1 py-px font-mono text-[8.5px] text-destructive uppercase leading-none tracking-[0.08em]">Adv</span>}
            {stage.count > 0 && <span className="text-muted-foreground tabular-nums">×{stage.count}</span>}
        </span>
    );
}
