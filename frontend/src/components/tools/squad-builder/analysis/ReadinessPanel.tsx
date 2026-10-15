import { useQueries } from "@tanstack/react-query";
import type * as React from "react";
import { eliteIcon, moduleIconURL, potentialIcon, skillIconURL, specializedIcon } from "#/components/tools/shared/icons";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { operatorQueryOptions } from "#/lib/api/operators";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { Readiness } from "#/types/generated/Readiness";
import type { ReadinessFlag } from "#/types/generated/ReadinessFlag";
import type { ReadinessRow } from "#/types/generated/ReadinessRow";
import type { IOperatorListItem } from "#/types/operators";
import type { messages } from "./ReadinessPanel.messages";
import { type IChipContext, type IPanelBaseProps, OpChip, PanelShell, Pill, StatCell, StatStrip } from "./shared";
import type { messages as sharedMessages } from "./shared.messages";

type ReadinessT = TypedT<typeof messages & typeof sharedMessages>;

interface IReadinessPanelProps extends IPanelBaseProps {
    readiness: Readiness | undefined;
}

export function ReadinessPanel({ state, stale, collapsed, onToggle, readiness, ctx }: IReadinessPanelProps): React.ReactElement {
    const t: ReadinessT = useT("tools");
    return (
        <PanelShell title={t("squad.analysis.readiness.title")} question={t("squad.analysis.readiness.question")} state={state} stale={stale} collapsed={collapsed} onToggle={onToggle}>
            {readiness && <ReadinessBody readiness={readiness} ctx={ctx} t={t} />}
        </PanelShell>
    );
}

function ReadinessBody({ readiness, ctx, t }: { readiness: Readiness; ctx: IChipContext; t: ReadinessT }): React.ReactElement {
    const f = useFormatters();
    const server = useGamedataServer();
    const ids = [...new Set(readiness.rows.map((r) => r.char_id))];
    const details = useQueries({ queries: ids.map((id) => operatorQueryOptions(id, server)) });
    const detailById = new Map<string, IOperatorListItem>();
    ids.forEach((id, i) => {
        const d = details[i]?.data;
        if (d) detailById.set(id, d);
    });
    const s = readiness.stats;

    return (
        <div className="space-y-4">
            <StatStrip>
                <StatCell label={t("squad.analysis.readiness.usual")} value={t("squad.analysis.readiness.elite", { n: s.usual_phase })} caption={t("squad.analysis.readiness.usual.caption", { count: s.usual_phase_count, seats: s.seats })} />
                <StatCell label={t("squad.analysis.readiness.levels", { n: s.usual_phase })} value={s.median_level === null ? "-" : f.number(s.median_level)} caption={t("squad.analysis.readiness.levels.caption")} />
                <StatCell label={t("squad.analysis.readiness.mastery")} value={f.number(s.mastery3)} caption={t("squad.analysis.readiness.mastery.caption", { count: s.chosen_skills })} />
                <StatCell label={t("squad.analysis.readiness.modules")} value={f.number(s.modules_on)} caption={t("squad.analysis.readiness.modules.caption", { count: s.modules_possible })} />
            </StatStrip>
            <ul className="divide-y divide-border/60 rounded-xl border border-border bg-card">
                {readiness.rows.map((row) => (
                    <ReadinessRowView key={row.seat} row={row} detail={detailById.get(row.char_id)} ctx={ctx} t={t} />
                ))}
            </ul>
            <p className="text-[12px] text-muted-foreground">{t("squad.analysis.readiness.footer")}</p>
        </div>
    );
}

function ReadinessRowView({ row, detail, ctx, t }: { row: ReadinessRow; detail: IOperatorListItem | undefined; ctx: IChipContext; t: ReadinessT }): React.ReactElement {
    const skill = detail?.skills?.[row.skill_index];
    const mod = row.module_id ? detail?.modules?.find((m) => m.uniEquipId === row.module_id) : undefined;
    const modType = mod ? (mod.typeName2 ? `${mod.typeName1}-${mod.typeName2}` : mod.typeName1) : null;
    const skillTitle = row.mastery > 0 ? t("squad.analysis.readiness.skillMastery", { n: row.skill_index + 1, mastery: row.mastery }) : t("squad.analysis.readiness.skill", { n: row.skill_index + 1, level: row.skill_level });

    return (
        <li className="px-3 py-2" onMouseEnter={() => ctx.onHoverSeat(row.seat)} onMouseLeave={() => ctx.onHoverSeat(null)}>
            <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
                <div className="min-w-0 sm:w-40 sm:shrink-0">
                    <OpChip charId={row.char_id} seat={row.seat} support={row.support} ctx={ctx} />
                </div>
                <div className="flex flex-wrap items-center gap-2.5">
                    <Tooltip>
                        <TooltipTrigger render={<img alt={t("squad.analysis.readiness.elite", { n: row.phase })} className="icon-theme-aware size-6 object-contain" decoding="async" loading="lazy" src={eliteIcon(row.phase)} />} />
                        <TooltipPopup>{t("squad.analysis.readiness.elite", { n: row.phase })}</TooltipPopup>
                    </Tooltip>
                    <LevelRing level={row.level} cap={row.level_cap} t={t} />
                    <Tooltip>
                        <TooltipTrigger render={<span className="flex items-center gap-1" />}>
                            {skill ? <img alt="" className="size-6 rounded-sm object-contain" decoding="async" loading="lazy" src={skillIconURL(skill)} /> : <span className="flex size-6 items-center justify-center rounded-sm bg-muted font-mono text-[10px] text-muted-foreground">S{row.skill_index + 1}</span>}
                            {row.mastery > 0 ? <img alt="" className="icon-theme-aware size-5 object-contain" decoding="async" loading="lazy" src={specializedIcon(row.mastery)} /> : <span className="font-mono text-[11px] text-muted-foreground">{row.skill_level}</span>}
                        </TooltipTrigger>
                        <TooltipPopup>{skillTitle}</TooltipPopup>
                    </Tooltip>
                    {row.module_id ? (
                        <Tooltip disabled={!mod?.uniEquipName}>
                            <TooltipTrigger render={<span className="flex items-center gap-1" />}>
                                {mod && <img alt="" className="size-6 object-contain" decoding="async" loading="lazy" src={moduleIconURL(mod)} />}
                                {modType && <Pill tone="outline">{modType}</Pill>}
                                <span className="font-mono text-[11px] text-foreground">{t("squad.analysis.readiness.moduleRank", { level: row.module_level })}</span>
                            </TooltipTrigger>
                            <TooltipPopup>{mod?.uniEquipName}</TooltipPopup>
                        </Tooltip>
                    ) : (
                        <span className="text-[11px] text-muted-foreground">{t("squad.analysis.readiness.noModule")}</span>
                    )}
                    <Tooltip>
                        <TooltipTrigger render={<img alt={t("squad.analysis.readiness.potential", { n: row.potential + 1 })} className="icon-theme-aware size-5 object-contain" decoding="async" loading="lazy" src={potentialIcon(row.potential)} />} />
                        <TooltipPopup>{t("squad.analysis.readiness.potential", { n: row.potential + 1 })}</TooltipPopup>
                    </Tooltip>
                </div>
            </div>
            {row.flags.length > 0 && (
                <div className="mt-1.5 flex flex-wrap gap-1.5 sm:pl-43">
                    {row.flags.map((flag) => (
                        <Pill key={flag.kind} tone={flag.kind === "support_left_out" ? "muted" : "accent"} className="whitespace-normal">
                            {flagSentence(flag, row, t)}
                        </Pill>
                    ))}
                </div>
            )}
        </li>
    );
}

/** The level as an arc against the promotion's cap; no percentage, just the level. */
function LevelRing({ level, cap, t }: { level: number; cap: number; t: ReadinessT }): React.ReactElement {
    const r = 9;
    const c = 2 * Math.PI * r;
    const frac = cap > 0 ? Math.max(0, Math.min(1, level / cap)) : 0;
    return (
        <Tooltip>
            <TooltipTrigger render={<span className="flex items-center gap-1" />}>
                <svg viewBox="0 0 24 24" className="size-6 -rotate-90" aria-hidden="true">
                    <circle cx="12" cy="12" r={r} fill="none" strokeWidth="3" className="stroke-muted" />
                    <circle cx="12" cy="12" r={r} fill="none" strokeWidth="3" strokeLinecap="round" className="stroke-primary" strokeDasharray={`${frac * c} ${c}`} />
                </svg>
                <span className="font-mono text-[11px] text-foreground tabular-nums">{t("squad.analysis.readiness.level", { level })}</span>
            </TooltipTrigger>
            <TooltipPopup>{t("squad.analysis.readiness.levelOf", { level, cap })}</TooltipPopup>
        </Tooltip>
    );
}

function flagSentence(flag: ReadinessFlag, row: ReadinessRow, t: ReadinessT): string {
    const value = flag.value ?? 0;
    const reference = flag.reference ?? 0;
    switch (flag.kind) {
        case "level":
            return t("squad.analysis.readiness.flag.level", { value, phase: row.phase, reference });
        case "promotion":
            return t("squad.analysis.readiness.flag.promotion", { value, reference });
        case "skill_locked":
            return t("squad.analysis.readiness.flag.skillLocked", { n: row.skill_index + 1, value, reference });
        case "skill_rank_locked":
            return t("squad.analysis.readiness.flag.skillRankLocked", { value, reference });
        case "module_locked":
            return t("squad.analysis.readiness.flag.moduleLocked", { value, reference });
        case "support_left_out":
            return t("squad.analysis.readiness.flag.supportLeftOut");
    }
}
