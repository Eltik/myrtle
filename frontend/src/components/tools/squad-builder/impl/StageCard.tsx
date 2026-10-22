import { Link } from "@tanstack/react-router";
import { ExternalLink, Map as MapIcon, RefreshCw, X } from "lucide-react";
import * as React from "react";
import { tallyEnemies } from "#/components/stages/detail/impl/helpers";
import { Button } from "#/components/ui/button";
import { Skeleton } from "#/components/ui/skeleton";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { enemyIconURL } from "#/lib/api/enemies";
import type { IStageDetail } from "#/lib/api/stages";
import { useFormatters, useGamedataServer, useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IStageContext, StageThreatKey } from "../model";
import type { messages } from "./StageCard.messages";

type StageT = TypedT<typeof messages>;

function threatLabel(key: StageThreatKey, t: StageT): string {
    switch (key) {
        case "high_res":
            return t("squad.stage.threat.high_res");
        case "high_def":
            return t("squad.stage.threat.high_def");
        case "aerial":
            return t("squad.stage.threat.aerial");
        case "invisible":
            return t("squad.stage.threat.invisible");
        case "multi_life":
            return t("squad.stage.threat.multi_life");
        case "fast":
            return t("squad.stage.threat.fast");
    }
}

export interface IStageCardProps {
    stageId: string | null;
    detail: IStageDetail | null | undefined;
    context: IStageContext | null;
    isLoading: boolean;
    /** The challenge variant's id when the normal stage has one. */
    challengeId: string | null;
    normalId: string | null;
    readOnly: boolean;
    onPick: () => void;
    onSwitch: (stageId: string) => void;
    onRemove: () => void;
}

export function StageCard({ stageId, detail, context, isLoading, challengeId, normalId, readOnly, onPick, onSwitch, onRemove }: IStageCardProps): React.ReactElement {
    const t: StageT = useT("tools");
    const fmt = useFormatters();
    const locale = useLocale();
    const server = useGamedataServer();
    const [showEnemies, setShowEnemies] = React.useState(false);
    const tally = React.useMemo(() => (detail ? tallyEnemies(detail.levelData, detail.enemies).filter((r) => r.enemy && !r.enemy.hideInStage) : []), [detail]);
    const list = React.useMemo(() => new Intl.ListFormat(locale, { style: "narrow", type: "conjunction" }), [locale]);

    if (!stageId) {
        return (
            <section className="flex flex-col items-start gap-3 rounded-xl border border-border border-dashed bg-card/60 p-4 sm:flex-row sm:items-center">
                <MapIcon className="size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
                <p className="flex-1 text-[13px] text-muted-foreground">{t("squad.stage.empty")}</p>
                {!readOnly && (
                    <Button variant="outline" size="sm" onClick={onPick}>
                        {t("squad.stage.pick")}
                    </Button>
                )}
            </section>
        );
    }

    if (isLoading || !context) {
        return (
            <section className="rounded-xl border border-border bg-card p-4">
                {isLoading ? <Skeleton className="h-16 w-full" /> : <p className="text-[13px] text-muted-foreground">{t("squad.stage.missing")}</p>}
                {!isLoading && !readOnly && (
                    <div className="mt-3 flex gap-2">
                        <Button variant="outline" size="sm" onClick={onPick}>
                            {t("squad.stage.change")}
                        </Button>
                        <Button variant="ghost" size="sm" onClick={onRemove}>
                            {t("squad.stage.remove")}
                        </Button>
                    </div>
                )}
            </section>
        );
    }

    const none = t("squad.stage.none");
    const facts: Array<{ label: string; value: string }> = [
        { label: t("squad.stage.sanity"), value: context.apCost !== null ? fmt.number(context.apCost) : none },
        { label: t("squad.stage.deployLimit"), value: context.deployLimit !== null ? fmt.number(context.deployLimit) : none },
        { label: t("squad.stage.startDp"), value: context.startDp !== null ? fmt.number(context.startDp) : none },
        { label: t("squad.stage.dpRate"), value: context.dpPerSecond !== null ? t("squad.stage.dpValue", { value: fmt.number(Math.round(context.dpPerSecond * 100) / 100) }) : none },
        { label: t("squad.stage.lifePoints"), value: context.lifePoints !== null ? fmt.number(context.lifePoints) : none },
        { label: t("squad.stage.enemies"), value: fmt.number(context.enemyCount) },
    ];
    const isChallenge = challengeId !== null && stageId === challengeId;

    return (
        <section className="rounded-xl border border-border bg-card p-4">
            <div className="flex flex-wrap items-start gap-3">
                <span className="flex h-10 min-w-14 items-center justify-center rounded-md border border-border bg-muted px-2 font-bold font-mono text-[14px]">{context.code}</span>
                <div className="min-w-0 flex-1">
                    <h3 className="truncate font-semibold text-[15px] leading-tight">{context.name || context.code}</h3>
                    {context.zoneName && <p className="truncate text-[12px] text-muted-foreground">{context.zoneName}</p>}
                </div>
                <div className="flex flex-wrap items-center gap-2">
                    {challengeId && normalId && !readOnly && (
                        <ToggleGroup aria-label={t("squad.stage.mode")} value={[isChallenge ? "challenge" : "normal"]} onValueChange={(next) => next[0] && onSwitch(next[0] === "challenge" ? challengeId : normalId)} variant="outline" size="sm">
                            <ToggleGroupItem value="normal" className="px-2 text-[12px]">
                                {t("squad.stage.normal")}
                            </ToggleGroupItem>
                            <ToggleGroupItem value="challenge" className="px-2 text-[12px]">
                                {t("squad.stage.challenge")}
                            </ToggleGroupItem>
                        </ToggleGroup>
                    )}
                    {!readOnly && (
                        <Button variant="outline" size="sm" onClick={onPick}>
                            <RefreshCw />
                            {t("squad.stage.change")}
                        </Button>
                    )}
                    <Button variant="ghost" size="sm" render={<Link to="/stages/$stageId" params={{ stageId }} />}>
                        <ExternalLink />
                        {t("squad.stage.open")}
                    </Button>
                    {!readOnly && (
                        <Button variant="ghost" size="icon-sm" aria-label={t("squad.stage.remove")} onClick={onRemove}>
                            <X />
                        </Button>
                    )}
                </div>
            </div>

            {/* Without a level file every fact below would read as "none" and the enemy count as 0, which looks like a stage that failed to load. */}
            {!context.hasLevel ? (
                <p className="mt-3 rounded-lg border border-border border-dashed bg-background/40 px-3 py-2 text-[12.5px] text-muted-foreground">{t("squad.stage.noLevel")}</p>
            ) : (
                <>
                    <dl className="mt-3 grid grid-cols-3 gap-2 sm:grid-cols-6">
                        {facts.map((f) => (
                            <div key={f.label} className="rounded-lg border border-border bg-background/40 px-2.5 py-1.5">
                                <dt className="truncate text-[11px] text-muted-foreground">{f.label}</dt>
                                <dd className="font-mono font-semibold text-[14px] tabular-nums">{f.value}</dd>
                            </div>
                        ))}
                    </dl>

                    <div className="mt-3">
                        <h4 className="mb-1.5 font-medium text-[12px] text-muted-foreground">{t("squad.stage.threats")}</h4>
                        {context.threats.length === 0 ? (
                            <p className="text-[12.5px] text-muted-foreground">{t("squad.stage.noThreats")}</p>
                        ) : (
                            <ul className="flex flex-wrap gap-1.5">
                                {context.threats.map((th) => (
                                    <li key={th.key}>
                                        <Tooltip>
                                            <TooltipTrigger
                                                render={(p) => (
                                                    <button {...p} type="button" className="inline-flex cursor-default items-center rounded-md border border-border bg-background/40 px-2 py-1 font-medium text-[12px] outline-none focus-visible:ring-2 focus-visible:ring-ring">
                                                        {t("squad.stage.threatCount", { label: threatLabel(th.key, t), count: th.count })}
                                                    </button>
                                                )}
                                            />
                                            <TooltipPopup className="max-w-72">{list.format(th.enemies.map((e) => e.name))}</TooltipPopup>
                                        </Tooltip>
                                    </li>
                                ))}
                            </ul>
                        )}
                    </div>

                    {tally.length > 0 && (
                        <div className="mt-3">
                            <Button variant="ghost" size="xs" onClick={() => setShowEnemies((v) => !v)} aria-expanded={showEnemies}>
                                {showEnemies ? t("squad.stage.hideEnemies") : t("squad.stage.showEnemies")}
                            </Button>
                            {showEnemies && (
                                <ul className="mt-2 grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-2">
                                    {tally.map((row) => (
                                        <li key={row.id} className="flex items-center gap-2 rounded-lg border border-border bg-background/40 p-1.5">
                                            <span className={cn("size-9 shrink-0 overflow-hidden rounded-md border bg-muted", row.enemy?.enemyLevel === "BOSS" ? "border-destructive/60" : row.enemy?.enemyLevel === "ELITE" ? "border-warning/60" : "border-border")}>
                                                <img alt="" src={enemyIconURL(row.id, server)} className="size-full object-contain" loading="lazy" />
                                            </span>
                                            <span className="min-w-0 flex-1 truncate text-[12px]">{row.enemy?.name ?? row.id}</span>
                                            <span className="shrink-0 font-mono text-[11px] text-muted-foreground">{t("squad.stage.enemyCount", { count: row.count })}</span>
                                        </li>
                                    ))}
                                </ul>
                            )}
                        </div>
                    )}
                </>
            )}
        </section>
    );
}
