import { Dices, RotateCcw, Star } from "lucide-react";
import * as React from "react";
import { Bar, BarChart, CartesianGrid, ReferenceLine, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { useFormatters, useT } from "#/lib/i18n";
import { cn } from "#/lib/utils";
import { bannerModel, type IBannerModel, ORUNDUM_PER_PULL } from "../pulls/rates";
import { createPuller, type IPuller, type IPullOutcome, type ITargetStats, simulateToTarget } from "../pulls/simulate";
import { type BannerArchetype, BannerModelNote, type PullsT, Stat, useBannerLabel, usePct } from "./PullsShared";
import { SectionTitle } from "./shared";

const ARCHETYPES: { ruleType: BannerArchetype; featuredCount: number }[] = [
    { ruleType: "LIMITED", featuredCount: 2 },
    { ruleType: "SINGLE", featuredCount: 1 },
    { ruleType: "DOUBLE", featuredCount: 2 },
    { ruleType: "LINKAGE", featuredCount: 1 },
    { ruleType: "CLASSIC", featuredCount: 2 },
];

/**
 * A run averages tens of pulls for one copy and a few hundred for six, so this is
 * low millions of iterations on a settings change: fast enough to stay synchronous,
 * and small enough not to stall the tab when someone drags the copies picker.
 */
const RUNS = 10_000;
const MAX_PULLS = 1000;

function isRateUp(o: IPullOutcome): boolean {
    return o.slot === "featuredA" || o.slot === "featuredB";
}

const RARITY_COLOR: Record<number, string> = {
    6: "oklch(0.85 0.18 80)",
    5: "#f7e79e",
    4: "#bcabdb",
    3: "#88c8e3",
};

interface IPullsSimulatorProps {
    budget: number;
    pity: number;
}

export function PullsSimulator({ budget, pity }: IPullsSimulatorProps): React.ReactElement {
    const t: PullsT = useT("tools");
    const f = useFormatters();
    const [ruleType, setRuleType] = React.useState<BannerArchetype>("LIMITED");
    const [copies, setCopies] = React.useState(1);
    const [history, setHistory] = React.useState<IPullOutcome[]>([]);
    const [runId, setRunId] = React.useState(0);
    const bannerLabel = useBannerLabel();
    const pct = usePct();

    const model: IBannerModel = React.useMemo(() => {
        const a = ARCHETYPES.find((x) => x.ruleType === ruleType) ?? ARCHETYPES[0];
        return bannerModel({ ruleType: a.ruleType, featuredCount: a.featuredCount });
    }, [ruleType]);
    const startPity = model.carryOver ? pity : 0;

    /**
     * The puller IS the run: one mutable object holding pity, copies and the
     * once-only guarantee flags. It lives in a ref rather than a memo, because a memo
     * is documented as a cache React may discard, and losing this one would silently
     * reset a run mid-session.
     *
     * A signature covering the banner, the carried pity and the reset counter decides
     * when to build a new one. History is cleared in the same render through React's
     * adjust-state-during-render pattern rather than in an effect, which would paint
     * sixty stale chips beside a counter reading zero for one frame.
     */
    const signature = `${model.ruleType}:${model.featuredCount}:${startPity}:${runId}`;
    const runRef = React.useRef<{ signature: string; puller: IPuller } | null>(null);
    if (runRef.current === null || runRef.current.signature !== signature) {
        runRef.current = { signature, puller: createPuller(model, startPity) };
    }
    const puller = runRef.current.puller;
    const [historySignature, setHistorySignature] = React.useState(signature);
    if (historySignature !== signature) {
        setHistorySignature(signature);
        setHistory([]);
    }

    /**
     * Sampling is the one expensive thing on this tab: 10,000 runs at six copies
     * measures 204 ms, and `startPity` changes on every keystroke in the resources
     * field. Deferring the inputs lets the typed value paint immediately and the
     * distribution catch up, instead of blocking the commit on every character.
     */
    const deferredCopies = React.useDeferredValue(copies);
    const deferredPity = React.useDeferredValue(startPity);
    const stats: ITargetStats = React.useMemo(() => simulateToTarget(model, { runs: RUNS, copies: deferredCopies, maxPulls: MAX_PULLS, startPity: deferredPity, bucket: 10 }), [model, deferredCopies, deferredPity]);

    const withinBudget = React.useMemo(() => {
        if (budget <= 0) return null;
        let reached = 0;
        let seen = 0;
        for (let i = 0; i < stats.histogram.length; i++) {
            seen += stats.histogram[i];
            if ((i + 1) * stats.bucket <= budget) reached = seen;
        }
        return stats.runs > 0 ? reached / stats.runs : 0;
    }, [stats, budget]);

    const chartRows = React.useMemo(
        () =>
            stats.histogram.map((count, i) => ({
                at: (i + 1) * stats.bucket,
                runs: count,
            })),
        [stats],
    );

    const state = puller.state;
    const doPull = (n: number) => {
        const drawn = puller.pullMany(n);
        setHistory((h) => [...drawn].reverse().concat(h).slice(0, 60));
    };
    // Bumping the run id is what rebuilds the puller, because the memo is the only
    // thing holding the run's state and a new identity is the only way to clear it.
    const reset = () => setRunId((n) => n + 1);

    return (
        <Card className="flex flex-col gap-4 p-4">
            <div className="flex flex-col gap-1">
                <SectionTitle>{t("release.pulls.sim.title")}</SectionTitle>
                <p className="m-0 font-sans text-[12px] text-muted-foreground leading-normal">{t("release.pulls.sim.desc")}</p>
            </div>

            <div className="flex flex-wrap items-end gap-x-4 gap-y-3">
                <div className="flex flex-col gap-1">
                    <span className="font-sans text-[12px] text-muted-foreground">{t("release.pulls.odds.banner")}</span>
                    <Select value={ruleType} onValueChange={(v) => v !== null && setRuleType(v as BannerArchetype)}>
                        <SelectTrigger size="sm" className="w-44" aria-label={t("release.pulls.odds.banner")}>
                            <SelectValue>{() => bannerLabel(ruleType)}</SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            {ARCHETYPES.map((a) => (
                                <SelectItem key={a.ruleType} value={a.ruleType}>
                                    {bannerLabel(a.ruleType)}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
                <div className="flex flex-col gap-1">
                    <span className="font-sans text-[12px] text-muted-foreground">{t("release.pulls.sim.target")}</span>
                    <Select value={String(copies)} onValueChange={(v) => v !== null && setCopies(Number(v))}>
                        <SelectTrigger size="sm" className="w-24 font-mono tabular-nums" aria-label={t("release.pulls.sim.target")}>
                            <SelectValue>{() => f.number(copies)}</SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            {[1, 2, 3, 4, 5, 6].map((c) => (
                                <SelectItem key={c} value={String(c)}>
                                    {c}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
            </div>

            <BannerModelNote model={model} />

            <div className="flex flex-col gap-3 rounded-lg border border-border bg-muted/30 p-3">
                <div className="flex flex-wrap items-center gap-2">
                    <Button size="sm" onClick={() => doPull(1)}>
                        <Dices className="mr-1.5 size-4" />
                        {t("release.pulls.sim.pullOne")}
                    </Button>
                    <Button size="sm" variant="outline" onClick={() => doPull(10)}>
                        {t("release.pulls.sim.pullTen")}
                    </Button>
                    <Button size="sm" variant="ghost" onClick={reset} aria-label={t("release.pulls.sim.reset")}>
                        <RotateCcw className="size-4" />
                    </Button>
                </div>

                <div className="flex flex-wrap items-start gap-x-8 gap-y-3">
                    <Stat label={t("release.pulls.sim.spent")} value={f.number(state.pulls)} sub={t("release.pulls.sim.cost", { count: f.number(state.pulls * ORUNDUM_PER_PULL) })} />
                    <Stat label={t("release.pulls.sim.pityNow")} value={f.number(state.pity)} />
                    <Stat label={t("release.pulls.sim.got")} value={f.number(state.copiesA + state.copiesB)} />
                </div>

                {history.length > 0 && (
                    <div className="flex flex-wrap gap-1">
                        {history.map((o) => (
                            <span
                                key={o.index}
                                className={cn(
                                    "relative inline-flex size-6 items-center justify-center rounded font-mono text-[11px] tabular-nums",
                                    o.rarity >= 5 ? "font-bold text-zinc-950" : "text-zinc-950/70",
                                    // A rate-up is marked with a shape, not only a
                                    // colour: the ring alone carried the whole
                                    // meaning and did not clear the 3:1 minimum for
                                    // non-text contrast.
                                    isRateUp(o) ? "ring-2 ring-foreground ring-offset-1 ring-offset-background" : "",
                                )}
                                style={{ background: RARITY_COLOR[o.rarity] }}
                            >
                                {o.rarity}
                                {isRateUp(o) && <Star className="absolute -top-1 -right-1 size-2.5 fill-foreground text-foreground" aria-hidden />}
                                <span className="sr-only">
                                    {isRateUp(o) ? t("release.pulls.sim.rateUpChip") : ""} {o.guaranteed ? t("release.pulls.sim.guaranteed") : ""}
                                </span>
                            </span>
                        ))}
                    </div>
                )}
            </div>

            <div className="flex flex-col gap-2">
                <span className="font-sans font-semibold text-[12px] text-foreground">{t("release.pulls.sim.distribution", { runs: f.number(stats.runs) })}</span>
                <div className="flex flex-wrap items-start gap-x-8 gap-y-3">
                    <Stat label={t("release.pulls.sim.median")} value={f.number(stats.p50)} />
                    <Stat label={t("release.pulls.sim.p90")} value={f.number(stats.p90)} />
                    {withinBudget !== null && <Stat label={t("release.pulls.sim.withinBudget", { count: f.number(budget) })} value={pct(withinBudget, 0)} />}
                </div>
                <div className="h-44 w-full sm:h-52" role="img" aria-label={t("release.pulls.a11y.chartSim", { median: f.number(stats.p50), p90: f.number(stats.p90) })}>
                    <ResponsiveContainer width="100%" height="100%">
                        <BarChart data={chartRows} margin={{ top: 8, right: 8, bottom: 0, left: -20 }}>
                            <CartesianGrid stroke="var(--border)" strokeDasharray="3 3" vertical={false} />
                            <XAxis dataKey="at" tick={{ fill: "var(--muted-foreground)", fontSize: 11 }} stroke="var(--border)" minTickGap={30} />
                            <YAxis tick={{ fill: "var(--muted-foreground)", fontSize: 11 }} stroke="var(--border)" width={44} allowDecimals={false} />
                            <Tooltip
                                cursor={{ fill: "var(--muted)", opacity: 0.4 }}
                                contentStyle={{ background: "var(--popover)", border: "1px solid var(--border)", borderRadius: 8, fontSize: 12 }}
                                labelStyle={{ color: "var(--muted-foreground)" }}
                                labelFormatter={(v) => t("release.pulls.odds.atPulls", { count: Number(v) })}
                                formatter={(value) => [f.number(Number(value)), t("release.pulls.sim.distribution", { runs: f.number(stats.runs) })]}
                            />
                            <Bar dataKey="runs" fill="#bcabdb" isAnimationActive={false} />
                            {budget >= stats.bucket && budget <= MAX_PULLS && <ReferenceLine x={Math.max(stats.bucket, Math.round(budget / stats.bucket) * stats.bucket)} stroke="oklch(0.85 0.18 80)" strokeDasharray="4 3" />}
                        </BarChart>
                    </ResponsiveContainer>
                </div>
            </div>
        </Card>
    );
}
