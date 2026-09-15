import { Kicker } from "#/components/ui/kicker";
import type { IGachaEnhancedStats } from "#/lib/api/gacha";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { compareRate, fmtPct } from "./format";
import type { messages as formatMessages } from "./format.messages";
import type { messages } from "./RarityPanel.messages";

/** This panel renders its own chrome plus the labels `format.ts` derives. */
type RarityT = TypedT<typeof messages & typeof formatMessages>;

interface IRarityPanelProps {
    data: IGachaEnhancedStats | null;
    /** When present, render a personal column comparing to community + advertised. */
    personal?: IPersonalRarityRates | null;
}

export interface IPersonalRarityRates {
    totalPulls: number;
    totalSixStars: number;
    totalFiveStars: number;
    totalFourStars: number;
    totalThreeStars: number;
}

interface IRarityRow {
    rarity: 6 | 5 | 4 | 3;
    countKey: "totalSixStars" | "totalFiveStars" | "totalFourStars" | "totalThreeStars";
    color: string;
    starColor: string;
    expected: number;
    rateHintKey: keyof typeof messages & string;
}

// `expected` derivation: 6★ chance is 2% for pulls 1–50 since the last 6★, then
// +2% per pull until one drops. Expected pulls per 6★ = Σ n·P(first 6★ at n) ≈ 34.59,
// so the long-run 6★ rate is 1/34.59 ≈ 2.89%. On every pull the non-6★ share splits
// 8:50:40, so lower rarities scale by (1 − 0.0289)/0.98 - a pity 6★ takes their slot.
const RARITY_ROWS: IRarityRow[] = [
    { rarity: 6, countKey: "totalSixStars", color: "oklch(0.85 0.18 80)", starColor: "#f7a452", expected: 0.0289, rateHintKey: "community.rarity.hint.sixStar" },
    { rarity: 5, countKey: "totalFiveStars", color: "#f7e79e", starColor: "#f7e79e", expected: 0.0793, rateHintKey: "community.rarity.hint.fiveStar" },
    { rarity: 4, countKey: "totalFourStars", color: "#bcabdb", starColor: "#bcabdb", expected: 0.4955, rateHintKey: "community.rarity.hint.fourStar" },
    { rarity: 3, countKey: "totalThreeStars", color: "#88c8e3", starColor: "#88c8e3", expected: 0.3964, rateHintKey: "community.rarity.hint.threeStar" },
];

export function RarityPanel({ data, personal }: IRarityPanelProps) {
    const t: RarityT = useT("gacha");
    const f = useFormatters();
    const cs = data?.collectiveStats;
    const total = cs?.totalPulls ?? 0;
    const personalTotal = personal?.totalPulls ?? 0;
    const showPersonal = personal != null && personalTotal > 0;

    return (
        <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-[18px_18px] sm:p-[22px_24px]">
            <header className="flex flex-col gap-1 sm:flex-row sm:items-start sm:justify-between sm:gap-4">
                <div>
                    <Kicker className="mb-1.5">{cs ? t("community.rarity.kicker.withTotal", { count: f.compact(total) }) : t("community.rarity.kicker")}</Kicker>
                    <h2 className="m-0 text-balance font-sans font-semibold text-[20px] text-foreground leading-[1.15] tracking-[-0.02em] sm:text-[22px]">{t("community.rarity.title")}</h2>
                </div>
                {showPersonal ? (
                    <div className="inline-flex shrink-0 items-center gap-1.5 self-start rounded-md border border-primary/30 bg-primary/8 px-2.5 py-1 font-mono text-[10px] text-foreground/80 uppercase tracking-[0.14em] sm:self-auto">
                        <span className="block h-1.5 w-1.5 rounded-full bg-primary" aria-hidden />
                        {t("community.rarity.you", { count: f.compact(personalTotal) })}
                    </div>
                ) : null}
            </header>

            <div className="flex min-w-0 flex-col gap-4">
                {RARITY_ROWS.map((row) => {
                    const count = cs ? cs[row.countKey] : 0;
                    const observed = total > 0 ? count / total : 0;
                    const personalCount = personal ? personal[row.countKey] : 0;
                    const personalObserved = personalTotal > 0 ? personalCount / personalTotal : 0;

                    return (
                        <RarityRow
                            key={row.rarity}
                            row={row}
                            count={count}
                            observed={observed}
                            hasData={cs != null && total > 0}
                            personal={
                                showPersonal
                                    ? {
                                          observed: personalObserved,
                                          count: personalCount,
                                      }
                                    : null
                            }
                        />
                    );
                })}
            </div>

            <Legend showPersonal={showPersonal} />
        </section>
    );
}

function RarityRow({ row, count, observed, hasData, personal }: { row: IRarityRow; count: number; observed: number; hasData: boolean; personal: { observed: number; count: number } | null }) {
    const t: RarityT = useT("gacha");
    const f = useFormatters();
    const vsExpected = hasData ? compareRate(observed, row.expected, t) : null;
    const personalVsCommunity = personal != null ? compareRate(personal.observed, observed, t) : null;
    const personalVsExpected = personal != null ? compareRate(personal.observed, row.expected, t) : null;

    return (
        <div className="flex flex-col gap-1.5">
            <div className="flex items-center justify-between gap-3">
                <div className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-0.5">
                    <span style={{ color: row.starColor }} className="whitespace-nowrap text-base tracking-wider">
                        {"★".repeat(row.rarity)}
                    </span>
                    <span className="font-mono text-[10px] text-muted-foreground uppercase tracking-[0.14em]">{t(row.rateHintKey)}</span>
                </div>
                <div className="flex shrink-0 items-baseline gap-3 font-mono text-[11px] text-muted-foreground tabular-nums">
                    <span className="whitespace-nowrap font-sans font-semibold text-base text-foreground">{hasData ? fmtPct(observed, 2) : "-"}</span>
                </div>
            </div>

            <BaselineBar observed={observed} baseline={row.expected} fillColor={row.color} personalObserved={personal?.observed} hasData={hasData} />

            <div className="flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[10.5px] text-muted-foreground">
                <span className="tabular-nums">{hasData ? t("community.rarity.pulls", { count: f.number(count) }) : "-"}</span>
                <span className="opacity-50">·</span>
                <ComparisonChip label={t("community.rarity.chip.vsExpected")} baseline={row.expected} comparison={vsExpected} />
                {personal != null && personalVsCommunity != null ? (
                    <>
                        <span className="opacity-50">·</span>
                        <ComparisonChip label={t("community.rarity.chip.youVsCommunity")} baseline={observed} comparison={personalVsCommunity} highlight />
                    </>
                ) : null}
                {personal != null && personalVsExpected != null ? (
                    <>
                        <span className="opacity-50">·</span>
                        <ComparisonChip label={t("community.rarity.chip.youVsExpected")} baseline={row.expected} comparison={personalVsExpected} highlight />
                    </>
                ) : null}
            </div>
        </div>
    );
}

function ComparisonChip({ label, baseline, comparison, highlight }: { label: string; baseline: number; comparison: ReturnType<typeof compareRate>; highlight?: boolean }) {
    if (!comparison) {
        return (
            <span className="inline-flex items-center gap-1.5">
                <span className="uppercase tracking-[0.12em] opacity-70">{label}</span>
                <span className="text-muted-foreground/60">-</span>
            </span>
        );
    }
    const colorClass = comparison.direction === "on" ? "text-foreground/80" : comparison.direction === "above" ? "text-[oklch(0.78_0.18_152)]" : "text-[oklch(0.78_0.18_25)]";
    const baseClass = highlight ? "rounded-md border border-primary/20 bg-primary/6 px-1.5 py-0.5" : "";

    return (
        <span className={`inline-flex items-center gap-1.5 ${baseClass}`}>
            <span className="uppercase tracking-[0.12em] opacity-70">{label}</span>
            <span className={`font-semibold tabular-nums ${colorClass}`}>{comparison.ratioLabel}</span>
            <span className="tabular-nums opacity-60">({fmtPct(baseline, baseline >= 0.1 ? 0 : 2)})</span>
        </span>
    );
}

function BaselineBar({ observed, baseline, fillColor, personalObserved, hasData }: { observed: number; baseline: number; fillColor: string; personalObserved?: number; hasData: boolean }) {
    const t: RarityT = useT("gacha");
    // Bar scales to whichever value is largest, so a 65% 3★ bar sits next to a 2% 6★ bar without dwarfing it. Bars share a max within their own row.
    const max = Math.max(observed, baseline, personalObserved ?? 0, 0.001);
    const observedPct = hasData ? (observed / max) * 100 : 0;
    const baselineLeftPct = (baseline / max) * 100;
    const personalLeftPct = personalObserved != null ? (personalObserved / max) * 100 : null;

    return (
        <div className="relative h-2.5 w-full overflow-visible rounded-full bg-muted">
            <div className="h-full overflow-hidden rounded-full">
                <div className="h-full" style={{ width: `${observedPct}%`, background: fillColor }} />
            </div>
            <div className="pointer-events-none absolute inset-y-0 w-px bg-foreground/60" style={{ left: `${baselineLeftPct}%` }} title={t("community.rarity.bar.expected", { value: fmtPct(baseline, 2) })} aria-hidden />
            {personalLeftPct != null ? <div className="pointer-events-none absolute -top-1 h-4 w-0.5 rounded-full bg-primary shadow-[0_0_0_2px_var(--card)]" style={{ left: `calc(${personalLeftPct}% - 1px)` }} title={t("community.rarity.bar.you", { value: fmtPct(personalObserved ?? 0, 2) })} aria-hidden /> : null}
        </div>
    );
}

function Legend({ showPersonal }: { showPersonal: boolean }) {
    const t: RarityT = useT("gacha");

    return (
        <div className="flex flex-wrap items-center gap-x-4 gap-y-1.5 border-border/60 border-t pt-3 font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">
            <span className="inline-flex items-center gap-1.5">
                <span className="block h-2 w-2 rounded-full bg-foreground/60" aria-hidden /> {t("community.rarity.legend.expected")}
            </span>
            {showPersonal ? (
                <span className="inline-flex items-center gap-1.5">
                    <span className="block h-2 w-2 rounded-full bg-primary" aria-hidden /> {t("community.rarity.legend.you")}
                </span>
            ) : null}
            <span className="opacity-60">{t("community.rarity.legend.note")}</span>
            <span className="opacity-60">{t("community.rarity.legend.scaleNote")}</span>
        </div>
    );
}
