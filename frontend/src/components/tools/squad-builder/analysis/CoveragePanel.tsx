import * as React from "react";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { CoverageGroup } from "#/types/generated/CoverageGroup";
import type { CoverageGroupKey } from "#/types/generated/CoverageGroupKey";
import type { CoverageRowKey } from "#/types/generated/CoverageRowKey";
import type { IStageContext, IStageThreat, StageThreatKey } from "../model";
import type { messages } from "./CoveragePanel.messages";
import { type CoverageFilter, filterCoverage, neededRows } from "./coverageFilter";
import { type IPanelBaseProps, OpChip, PanelShell, Pill, Segmented, sourceLabel } from "./shared";
import type { messages as sharedMessages } from "./shared.messages";

type CoverageT = TypedT<typeof messages & typeof sharedMessages>;

interface ICoveragePanelProps extends IPanelBaseProps {
    coverage: ReadonlyArray<CoverageGroup> | undefined;
    stage: IStageContext | null;
}

export function CoveragePanel({ state, stale, collapsed, onToggle, coverage, stage, ctx }: ICoveragePanelProps): React.ReactElement {
    const t: CoverageT = useT("tools");
    const [filter, setFilter] = React.useState<CoverageFilter>("all");
    const activeFilter: CoverageFilter = filter === "stage" && !stage ? "all" : filter;
    const needed = React.useMemo(() => neededRows(stage), [stage]);
    const groups = React.useMemo(() => filterCoverage(coverage ?? [], activeFilter, stage), [coverage, activeFilter, stage]);

    const options: Array<{ value: CoverageFilter; label: string }> = [
        { value: "all", label: t("squad.analysis.coverage.filter.all") },
        { value: "covered", label: t("squad.analysis.coverage.filter.covered") },
        { value: "missing", label: t("squad.analysis.coverage.filter.missing") },
    ];
    if (stage) options.push({ value: "stage", label: t("squad.analysis.coverage.filter.stage") });

    return (
        <PanelShell title={t("squad.analysis.coverage.title")} question={t("squad.analysis.coverage.question")} state={state} stale={stale} collapsed={collapsed} onToggle={onToggle}>
            <div className="space-y-4">
                <Segmented label={t("squad.analysis.coverage.filter")} value={activeFilter} options={options} onChange={setFilter} />
                {groups.length === 0 ? (
                    <p className="text-[13px] text-muted-foreground">{t("squad.analysis.coverage.noMatch")}</p>
                ) : (
                    groups.map((g) => (
                        <section key={g.key}>
                            <h3 className="mb-1.5 font-semibold text-[13px] text-foreground">{groupLabel(g.key, t)}</h3>
                            <ul className="divide-y divide-border/60 rounded-xl border border-border bg-card">
                                {g.rows.map((row) => {
                                    const threats = needed.get(row.key) ?? [];
                                    const text = rowText(row.key, t);
                                    return (
                                        <li key={row.key} className="flex flex-col gap-1.5 px-3 py-2 sm:flex-row sm:items-start sm:gap-3">
                                            <div className="flex min-w-0 items-start gap-2 sm:w-52 sm:shrink-0">
                                                <Bullet covered={row.ops.length > 0} needed={threats.length > 0} />
                                                <div className="min-w-0">
                                                    <div className="font-medium text-[13px] text-foreground leading-snug">{text.label}</div>
                                                    {text.hint && <div className="text-[11.5px] text-muted-foreground leading-snug">{text.hint}</div>}
                                                    {activeFilter === "stage" && threats.length > 0 && (
                                                        <div className="mt-1 flex flex-wrap gap-1">
                                                            {threats.map((th) => (
                                                                <ThreatChip key={th.key} threat={th} t={t} />
                                                            ))}
                                                        </div>
                                                    )}
                                                </div>
                                            </div>
                                            <div className="flex min-w-0 flex-1 flex-wrap items-center gap-1.5 pl-4 sm:pl-0">
                                                {row.ops.length === 0 ? <Pill tone="outline">{t("squad.analysis.coverage.none")}</Pill> : row.ops.map((op) => <OpChip key={`${op.seat}-${op.char_id}`} charId={op.char_id} seat={op.seat} support={op.support} ctx={ctx} why={sourceLabel(op.why, t)} />)}
                                                {threats.length > 0 && <Pill tone="accent">{t("squad.analysis.coverage.needed")}</Pill>}
                                            </div>
                                        </li>
                                    );
                                })}
                            </ul>
                        </section>
                    ))
                )}
            </div>
        </PanelShell>
    );
}

/** Filled when covered, hollow when not; the accent colour when the stage asks for it. */
function Bullet({ covered, needed }: { covered: boolean; needed: boolean }): React.ReactElement {
    return <span aria-hidden="true" className={cn("mt-1.5 inline-block size-2 shrink-0 rounded-full border", needed ? "border-primary" : covered ? "border-foreground/70" : "border-muted-foreground/60", covered && (needed ? "bg-primary" : "bg-foreground/70"))} />;
}

function ThreatChip({ threat, t }: { threat: IStageThreat; t: CoverageT }): React.ReactElement {
    const names = threat.enemies.map((e) => e.name).join(", ");
    return (
        <Pill tone="muted" title={names || undefined}>
            {t("squad.analysis.coverage.threat", { threat: threatLabel(threat.key, t), count: threat.count })}
        </Pill>
    );
}

function threatLabel(key: StageThreatKey, t: CoverageT): string {
    switch (key) {
        case "high_res":
            return t("squad.analysis.threat.high_res");
        case "high_def":
            return t("squad.analysis.threat.high_def");
        case "aerial":
            return t("squad.analysis.threat.aerial");
        case "invisible":
            return t("squad.analysis.threat.invisible");
        case "multi_life":
            return t("squad.analysis.threat.multi_life");
        case "fast":
            return t("squad.analysis.threat.fast");
    }
}

function groupLabel(key: CoverageGroupKey, t: CoverageT): string {
    switch (key) {
        case "getting_on_the_field":
            return t("squad.analysis.coverage.group.getting_on_the_field");
        case "holding_the_line":
            return t("squad.analysis.coverage.group.holding_the_line");
        case "damage":
            return t("squad.analysis.coverage.group.damage");
        case "control":
            return t("squad.analysis.coverage.group.control");
        case "utility":
            return t("squad.analysis.coverage.group.utility");
    }
}

/** Each coverage row's label and the hint under it. */
function rowText(key: CoverageRowKey, t: CoverageT): { label: string; hint: string | null } {
    switch (key) {
        case "dp_gen":
            return { label: t("squad.analysis.coverage.row.dp_gen"), hint: t("squad.analysis.coverage.row.dp_gen.hint") };
        case "openers":
            return { label: t("squad.analysis.coverage.row.openers"), hint: t("squad.analysis.coverage.row.openers.hint") };
        case "fast_redeploy":
            return { label: t("squad.analysis.coverage.row.fast_redeploy"), hint: t("squad.analysis.coverage.row.fast_redeploy.hint") };
        case "block3":
            return { label: t("squad.analysis.coverage.row.block3"), hint: t("squad.analysis.coverage.row.block3.hint") };
        case "heal_single":
            return { label: t("squad.analysis.coverage.row.heal_single"), hint: null };
        case "heal_area":
            return { label: t("squad.analysis.coverage.row.heal_area"), hint: t("squad.analysis.coverage.row.heal_area.hint") };
        case "heal_elemental":
            return { label: t("squad.analysis.coverage.row.heal_elemental"), hint: t("squad.analysis.coverage.row.heal_elemental.hint") };
        case "physical":
            return { label: t("squad.analysis.coverage.row.physical"), hint: null };
        case "arts":
            return { label: t("squad.analysis.coverage.row.arts"), hint: null };
        case "true_elemental":
            return { label: t("squad.analysis.coverage.row.true_elemental"), hint: null };
        case "anti_air":
            return { label: t("squad.analysis.coverage.row.anti_air"), hint: t("squad.analysis.coverage.row.anti_air.hint") };
        case "multi":
            return { label: t("squad.analysis.coverage.row.multi"), hint: t("squad.analysis.coverage.row.multi.hint") };
        case "cc":
            return { label: t("squad.analysis.coverage.row.cc"), hint: null };
        case "shift":
            return { label: t("squad.analysis.coverage.row.shift"), hint: t("squad.analysis.coverage.row.shift.hint") };
        case "debuff":
            return { label: t("squad.analysis.coverage.row.debuff"), hint: t("squad.analysis.coverage.row.debuff.hint") };
        case "invisible":
            return { label: t("squad.analysis.coverage.row.invisible"), hint: null };
        case "summons":
            return { label: t("squad.analysis.coverage.row.summons"), hint: null };
    }
}
