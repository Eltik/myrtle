import * as React from "react";
import { Area, AreaChart, CartesianGrid, ReferenceLine, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { useFormatters, useT } from "#/lib/i18n";
import { oddsCurve, pullOdds } from "../pulls/odds";
import { bannerModel, type IBannerModel } from "../pulls/rates";
import { type BannerArchetype, BannerModelNote, PullsNumber, type PullsT, Stat, useBannerLabel, usePct } from "./PullsShared";
import { SectionTitle } from "./shared";

/**
 * The banner archetypes a player can ask about directly, rather than through a
 * specific forecast row. `featuredCount` is what the rule type actually ships with:
 * SINGLE and LINKAGE run one rate-up, the rest run two.
 */
const ARCHETYPES: { ruleType: BannerArchetype; featuredCount: number }[] = [
    { ruleType: "LIMITED", featuredCount: 2 },
    { ruleType: "SINGLE", featuredCount: 1 },
    { ruleType: "DOUBLE", featuredCount: 2 },
    { ruleType: "NORMAL", featuredCount: 2 },
    { ruleType: "LINKAGE", featuredCount: 1 },
    { ruleType: "CLASSIC", featuredCount: 2 },
    { ruleType: "ATTAIN", featuredCount: 1 },
];

const MAX_COPIES = 6;

interface IPullsOddsProps {
    /** Rolls the projection expects at the horizon, offered as a shortcut. */
    budget: number;
    pity: number;
}

export function PullsOdds({ budget, pity }: IPullsOddsProps): React.ReactElement {
    const t: PullsT = useT("tools");
    const pct = usePct();
    const f = useFormatters();
    const bannerLabel = useBannerLabel();
    const [ruleType, setRuleType] = React.useState<BannerArchetype>("LIMITED");
    // `budget || 100` would be a falsy check on a number where 0 is a real value
    // meaning "nothing banked", silently substituting 100.
    const [pulls, setPulls] = React.useState(() => Math.max(10, Math.min(600, budget > 0 ? budget : 100)));

    const model: IBannerModel = React.useMemo(() => {
        const a = ARCHETYPES.find((x) => x.ruleType === ruleType) ?? ARCHETYPES[0];
        return bannerModel({ ruleType: a.ruleType, featuredCount: a.featuredCount });
    }, [ruleType]);

    const startPity = model.carryOver ? pity : 0;
    const result = React.useMemo(() => pullOdds(model, pulls, { startPity, maxCopies: MAX_COPIES }), [model, pulls, startPity]);
    const curve = React.useMemo(() => oddsCurve(model, Math.max(pulls, 300), { startPity }), [model, pulls, startPity]);
    const chartRows = React.useMemo(() => curve.filter((_, i) => i % 2 === 0).map((p) => ({ pulls: p.pulls, specific: p.specific * 100, any: p.any * 100 })), [curve]);

    return (
        <Card className="flex flex-col gap-4 p-4">
            <SectionTitle>{t("release.pulls.odds.title")}</SectionTitle>

            <div className="flex flex-wrap items-end gap-x-4 gap-y-3">
                <div className="flex flex-col gap-1">
                    <span className="font-sans text-[11.5px] text-muted-foreground">{t("release.pulls.odds.banner")}</span>
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
                <PullsNumber id="odds-pulls" label={t("release.pulls.odds.pulls")} value={pulls} onChange={setPulls} min={0} max={1200} step={10} className="w-24" />
                {budget > 0 && (
                    <Button size="sm" variant="outline" onClick={() => setPulls(Math.min(1200, budget))}>
                        {t("release.pulls.odds.fromBudget")}
                    </Button>
                )}
            </div>

            <BannerModelNote model={model} />

            <div className="flex flex-wrap items-start gap-x-8 gap-y-4">
                <Stat label={model.featuredCount > 1 ? t("release.pulls.odds.specific") : t("release.pulls.odds.any")} value={pct(result.specific)} />
                {model.featuredCount > 1 && <Stat label={t("release.pulls.odds.any")} value={pct(result.any)} />}
                {model.featuredCount > 1 && <Stat label={t("release.pulls.odds.both")} value={pct(result.both)} />}
                <Stat label={t("release.pulls.odds.expectedSix")} value={f.number(Math.round(result.expectedSix * 100) / 100)} />
            </div>

            <div className="h-52 w-full sm:h-60" role="img" aria-label={t("release.pulls.a11y.chartOdds", { max: Math.max(pulls, 300) })}>
                <ResponsiveContainer width="100%" height="100%">
                    <AreaChart data={chartRows} margin={{ top: 8, right: 8, bottom: 0, left: -20 }}>
                        <defs>
                            <linearGradient id="pullsOddsFill" x1="0" y1="0" x2="0" y2="1">
                                <stop offset="0%" stopColor="#bcabdb" stopOpacity={0.45} />
                                <stop offset="100%" stopColor="#bcabdb" stopOpacity={0.02} />
                            </linearGradient>
                        </defs>
                        <CartesianGrid stroke="var(--border)" strokeDasharray="3 3" vertical={false} />
                        <XAxis dataKey="pulls" tick={{ fill: "var(--muted-foreground)", fontSize: 11 }} stroke="var(--border)" minTickGap={30} />
                        <YAxis domain={[0, 100]} tickFormatter={(v: number) => pct(v / 100, 0)} tick={{ fill: "var(--muted-foreground)", fontSize: 11 }} stroke="var(--border)" width={44} />
                        <Tooltip
                            contentStyle={{ background: "var(--popover)", border: "1px solid var(--border)", borderRadius: 8, fontSize: 12 }}
                            labelStyle={{ color: "var(--muted-foreground)" }}
                            labelFormatter={(v) => t("release.pulls.odds.atPulls", { count: Number(v) })}
                            formatter={(value, name) => [pct(Number(value) / 100), String(name)]}
                        />
                        <Area type="monotone" dataKey="specific" name={model.featuredCount > 1 ? t("release.pulls.odds.specific") : t("release.pulls.odds.any")} stroke="#bcabdb" strokeWidth={2} fill="url(#pullsOddsFill)" isAnimationActive={false} />
                        <ReferenceLine x={pulls} stroke="var(--muted-foreground)" strokeDasharray="4 3" />
                    </AreaChart>
                </ResponsiveContainer>
            </div>

            <div className="flex flex-col gap-1.5">
                <span className="font-sans font-semibold text-[12px] text-foreground">{t("release.pulls.odds.copies")}</span>
                {result.copies.map((p, k) => (
                    <div key={`copies-${k}`} className="flex items-center gap-2">
                        <span className="w-20 flex-none font-sans text-[11.5px] text-muted-foreground">{k === result.copies.length - 1 ? t("release.pulls.odds.copiesTail", { count: k }) : t("release.pulls.odds.copiesRow", { count: k })}</span>
                        <div className="h-2 min-w-0 flex-1 overflow-hidden rounded-full bg-muted">
                            <div className="h-full rounded-full bg-primary/70" style={{ width: `${Math.max(p * 100, p > 0 ? 0.5 : 0)}%` }} />
                        </div>
                        <span className="w-14 flex-none text-right font-mono text-[11.5px] text-muted-foreground tabular-nums">{pct(p, 1)}</span>
                    </div>
                ))}
            </div>

            <p className="m-0 font-sans text-[11.5px] text-muted-foreground leading-normal">{t("release.pulls.odds.method")}</p>
        </Card>
    );
}
