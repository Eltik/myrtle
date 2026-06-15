import { useSuspenseQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import type { IEnemyStageRef } from "#/lib/api/enemies";
import { enemyStagesQueryOptions } from "#/lib/api/enemies";
import { zonesQueryOptions } from "#/lib/api/stages";
import type { IZone } from "#/types/stages";
import { SectionHead } from "./sections";

/** A single zone with its enemy appearances, ready to render. */
interface IAppearanceGroup {
    zoneId: string;
    title: string;
    subtitle: string | null;
    type: IZone["type"] | null;
    zoneIndex: number;
    refs: IEnemyStageRef[];
}

function zoneTitle(zone: IZone | undefined, zoneId: string): { title: string; subtitle: string | null } {
    if (!zone) return { title: zoneId, subtitle: null };
    const title = zone.zoneNameSecond || zone.zoneNameFirst || zone.zoneNameTitleCurrent || zoneId;
    // Surface the chapter/episode prefix as a subtitle when it differs from the title.
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
        const { title, subtitle } = zoneTitle(zone, zoneId);
        groups.push({
            zoneId,
            title,
            subtitle,
            type: zone?.type ?? null,
            zoneIndex: zone?.zoneIndex ?? Number.MAX_SAFE_INTEGER,
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
    CAMPAIGN: "Campaign",
    CLIMB_TOWER: "S.S.S.",
    ROGUELIKE: "I.S.",
    GUIDE: "Guide",
    SPECIAL: "Special",
};

export function AppearsInTab({ enemyId }: { enemyId: string }) {
    const { data: index } = useSuspenseQuery(enemyStagesQueryOptions());
    const { data: zones } = useSuspenseQuery(zonesQueryOptions());

    const groups = useMemo(() => {
        const refs = index[enemyId] ?? [];
        if (refs.length === 0) return [];
        const zonesById = new Map(zones.map((z) => [z.zoneId, z]));
        return groupByZone(refs, zonesById);
    }, [index, enemyId, zones]);

    const totalStages = useMemo(() => (index[enemyId] ?? []).length, [index, enemyId]);

    if (groups.length === 0) {
        return (
            <div className="flex flex-col items-center justify-center rounded-[14px] border border-border border-dashed bg-card/50 px-6 py-14 text-center">
                <h3 className="m-0 font-sans font-semibold text-[15px] text-foreground leading-none">No stage appearances recorded</h3>
                <p className="m-0 mt-2 max-w-96 text-pretty font-sans text-[13px] text-muted-foreground leading-normal">This enemy isn't listed in any of the currently extracted stage level files. Coverage grows as more stages are processed.</p>
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-5 sm:gap-5.5">
            <section>
                <SectionHead>
                    Appears in {totalStages} stage{totalStages === 1 ? "" : "s"} · {groups.length} zone{groups.length === 1 ? "" : "s"}
                </SectionHead>
                <div className="flex flex-col gap-4">
                    {groups.map((g) => (
                        <ZoneGroup key={g.zoneId} group={g} />
                    ))}
                </div>
            </section>
        </div>
    );
}

function ZoneGroup({ group }: { group: IAppearanceGroup }) {
    const typeLabel = group.type ? ZONE_TYPE_LABEL[group.type] : null;
    return (
        <div className="rounded-[12px] border border-border bg-[color-mix(in_oklch,var(--muted)_22%,transparent)] p-3 sm:p-3.5">
            <div className="mb-2.5 flex items-baseline justify-between gap-2">
                <div className="flex min-w-0 flex-col gap-0.5">
                    <h4 className="m-0 truncate font-sans font-semibold text-[13.5px] text-foreground leading-tight">{group.title}</h4>
                    {group.subtitle && <span className="truncate font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{group.subtitle}</span>}
                </div>
                {typeLabel && <span className="shrink-0 rounded-full border border-border bg-card px-2 py-0.75 font-medium font-mono text-[9.5px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{typeLabel}</span>}
            </div>
            <div className="flex flex-wrap gap-1.5">
                {group.refs.map((r) => (
                    <StageChip key={`${r.stageId}-${r.isHard ? "h" : "n"}`} stage={r} />
                ))}
            </div>
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
