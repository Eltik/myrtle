import { useMemo, useRef, useState } from "react";
import { Kicker } from "#/components/ui/kicker";
import { type ClientGachaGroup, classifyBannerGroup, type IBanner, type IDatePullData, type IDayOfWeekPullData, type IHourlyPullData, type IPullTimingData } from "#/lib/api/gacha";
import styles from "./CommunityPage.module.css";
import { fmtPct } from "./format";

interface ITimingPanelProps {
    timing: IPullTimingData | null | undefined;
    firstPullAt?: number;
    banners: IBanner[];
}

const TYPE_LABELS: Record<ClientGachaGroup, string> = {
    limited: "Limited",
    linkage: "Collab",
    regular: "Standard",
    special: "Kernel",
};

const TYPE_COLORS: Record<ClientGachaGroup, string> = {
    limited: "oklch(0.85 0.18 80)",
    linkage: "oklch(0.78 0.16 320)",
    regular: "#bcabdb",
    special: "#88c8e3",
};

// Track render order — top row down. Same order as the rest of the UI.
const TRACK_ORDER: ClientGachaGroup[] = ["limited", "linkage", "special", "regular"];

export function TimingPanel({ timing, firstPullAt, banners }: ITimingPanelProps) {
    if (!timing) return null;

    const byDate = [...(timing.byDate ?? [])].sort((a, b) => a.date.localeCompare(b.date));
    const dayCount = firstPullAt ? Math.floor((Date.now() / 1000 - firstPullAt) / 86_400) + 1 : 0;

    return (
        <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-[18px_18px] sm:p-[22px_24px]">
            <header className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between sm:gap-4">
                <div>
                    <Kicker className="mb-1.5">Pull timing</Kicker>
                    <h2 className="m-0 font-sans text-[20px] font-semibold leading-[1.15] tracking-[-0.02em] text-foreground text-balance sm:text-[22px]">When the community pulls.</h2>
                </div>
                <div className="inline-flex flex-wrap items-center gap-x-3.5 gap-y-1 font-mono text-[10.5px] uppercase tracking-[0.12em] text-muted-foreground">
                    <span className="inline-flex items-center">
                        <span aria-hidden className="mr-1.5 inline-block h-2 w-2 rounded-sm bg-primary align-middle" />
                        Total pulls
                    </span>
                    {TRACK_ORDER.map((g) => (
                        <span key={g} className="inline-flex items-center">
                            <span aria-hidden className="mr-1.5 inline-block h-2 w-2 rounded-sm align-middle" style={{ background: TYPE_COLORS[g] }} />
                            {TYPE_LABELS[g]}
                        </span>
                    ))}
                </div>
            </header>

            <div className="grid gap-y-4 gap-x-6 grid-cols-1 md:grid-cols-2">
                {byDate.length > 0 ? (
                    <div className="md:col-span-2">
                        <div className="mb-2.5 font-mono text-[10.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground">By date · last {dayCount} days</div>
                        <TimingByDate rows={byDate} banners={banners} />
                    </div>
                ) : null}

                <div className="flex flex-col">
                    <div className="mb-2.5 font-mono text-[10.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground">By hour of day · UTC</div>
                    <TimingByHour rows={timing.byHour} />
                </div>

                <div>
                    <div className="mb-2.5 font-mono text-[10.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground">By day of week</div>
                    <TimingByDow rows={timing.byDayOfWeek} />
                </div>
            </div>
        </section>
    );
}

interface IBannerBand {
    poolId: string;
    name: string;
    group: ClientGachaGroup;
    /** Inclusive row indices into the data series. */
    startIdx: number;
    endIdx: number;
}

/**
 * Map banners onto the date-indexed chart x-axis.
 *
 * The line chart uses uniform row-index spacing (one column per date that has
 * pulls). We approximate banner bands by snapping each banner's open/close
 * timestamps to the nearest row indices — slightly inaccurate when the
 * underlying series has multi-day gaps, but the community corpus is dense
 * enough day-to-day that this is invisible in practice.
 *
 * Bands are clamped to the chart window; banners with zero overlap are
 * dropped. Tracks are stacked by rule-type (TRACK_ORDER), at most 4 rows.
 */
function buildBannerBands(rows: IDatePullData[], banners: IBanner[]): { byTrack: Record<ClientGachaGroup, IBannerBand[]>; nonEmptyTracks: ClientGachaGroup[]; daySecs: number[] } {
    const daySecs = rows.map((r) => Date.UTC(...parseISODate(r.date)) / 1000);
    const firstSec = daySecs[0];
    const lastSec = daySecs[daySecs.length - 1];

    const byTrack: Record<ClientGachaGroup, IBannerBand[]> = { limited: [], linkage: [], regular: [], special: [] };

    if (daySecs.length === 0) return { byTrack, nonEmptyTracks: [], daySecs };

    for (const b of banners) {
        if (b.endTime < firstSec || b.openTime > lastSec) continue;
        const startIdx = lowerBoundIdx(daySecs, b.openTime);
        const endIdx = upperBoundIdx(daySecs, b.endTime);
        if (endIdx < startIdx) continue;
        const group = classifyBannerGroup(b);
        byTrack[group].push({ poolId: b.gachaPoolId, name: b.gachaPoolName, group, startIdx, endIdx });
    }

    const nonEmptyTracks = TRACK_ORDER.filter((g) => byTrack[g].length > 0);
    return { byTrack, nonEmptyTracks, daySecs };
}

function parseISODate(s: string): [number, number, number] {
    const [y, m, d] = s.split("-").map(Number);
    return [y, (m ?? 1) - 1, d ?? 1];
}

/** First index i where arr[i] >= target, clamped to [0, len-1]. */
function lowerBoundIdx(arr: number[], target: number): number {
    let lo = 0;
    let hi = arr.length;
    while (lo < hi) {
        const mid = (lo + hi) >> 1;
        if (arr[mid] < target) lo = mid + 1;
        else hi = mid;
    }
    return Math.min(arr.length - 1, Math.max(0, lo));
}

/** Last index i where arr[i] <= target, clamped to [0, len-1]. */
function upperBoundIdx(arr: number[], target: number): number {
    let lo = 0;
    let hi = arr.length;
    while (lo < hi) {
        const mid = (lo + hi) >> 1;
        if (arr[mid] <= target) lo = mid + 1;
        else hi = mid;
    }
    return Math.min(arr.length - 1, Math.max(0, lo - 1));
}

function TimingByDate({ rows, banners }: { rows: IDatePullData[]; banners: IBanner[] }) {
    const series = rows.map((r) => r.pullCount);
    const days = rows.map((r) => {
        const d = new Date(r.date);
        if (Number.isNaN(d.getTime())) return r.date;
        return `${d.toLocaleString("en", { month: "short" })} ${d.getDate()}`;
    });
    const bands = useMemo(() => buildBannerBands(rows, banners), [rows, banners]);
    return <ActivityChart days={days} data={series} color="var(--primary)" height={220} bands={bands} />;
}

function TimingByHour({ rows }: { rows: IHourlyPullData[] }) {
    const max = Math.max(1, ...rows.map((r) => r.pullCount));
    return (
        <div className={styles.hourGrid}>
            {rows.map((r) => {
                const h = r.pullCount / max;
                return (
                    <div key={r.hour} className={styles.hourCell}>
                        <div className={styles.hourBar} style={{ height: `${Math.max(4, h * 100)}%`, background: `oklch(0.58 0.22 25 / ${0.25 + h * 0.7})` }} />
                        <div className={styles.hourLabel}>{String(r.hour).padStart(2, "0")}</div>
                    </div>
                );
            })}
        </div>
    );
}

function TimingByDow({ rows }: { rows: IDayOfWeekPullData[] }) {
    const max = Math.max(1, ...rows.map((r) => r.pullCount));
    return (
        <div className={styles.dowList}>
            {rows.map((r) => (
                <div key={r.day} className={styles.dowRow}>
                    <span className={styles.dowName}>{r.dayName.slice(0, 3)}</span>
                    <div className={styles.dowBar}>
                        <div style={{ width: `${(r.pullCount / max) * 100}%` }} />
                    </div>
                    <span className={styles.dowPct}>{fmtPct(r.percentage, 1)}</span>
                </div>
            ))}
        </div>
    );
}

function ActivityChart({ data, days, color, height, bands }: { data: number[]; days: string[]; color: string; height: number; bands: ReturnType<typeof buildBannerBands> }) {
    const PAD_T = 8;
    const X_AXIS_H = 24;
    const TRACK_H = 8;
    const TRACK_GAP = 3;
    const TRACKS_TOP_GAP = 6; // gap between line-chart bottom and first track
    const tracksH = bands.nonEmptyTracks.length === 0 ? 0 : bands.nonEmptyTracks.length * (TRACK_H + TRACK_GAP) - TRACK_GAP + TRACKS_TOP_GAP;
    const PAD_B = X_AXIS_H + tracksH;

    const VBW = 1000;
    const VBH = 100;
    const max = Math.max(1, ...data);
    const yticks = [max, max * 0.75, max * 0.5, max * 0.25, 0];
    const fmtY = (v: number) => (v >= 1e6 ? `${(v / 1e6).toFixed(1)}M` : v >= 1e3 ? `${Math.round(v / 1e3)}k` : `${Math.round(v)}`);

    const linePath = (vals: number[]) => {
        if (vals.length === 0) return "";
        const pts = vals.map((v, i) => [(i / Math.max(1, vals.length - 1)) * VBW, VBH - (v / max) * VBH] as const);
        return pts.map(([x, y], i) => `${i === 0 ? "M" : "L"}${x.toFixed(2)} ${y.toFixed(2)}`).join(" ");
    };

    const xTickCount = Math.min(6, days.length);
    const xTickIdx = xTickCount > 1 ? Array.from({ length: xTickCount }, (_, i) => Math.round((i * (days.length - 1)) / (xTickCount - 1))) : [0];

    const fillId = "ac-fill-gacha-community";

    const plotRef = useRef<HTMLDivElement>(null);
    const [hoverIdx, setHoverIdx] = useState<number | null>(null);

    const onPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
        if (data.length === 0 || !plotRef.current) return;
        const rect = plotRef.current.getBoundingClientRect();
        if (rect.width <= 0) return;
        const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
        setHoverIdx(data.length > 1 ? Math.round(ratio * (data.length - 1)) : 0);
    };
    const onPointerLeave = () => setHoverIdx(null);

    const plotInnerH = Math.max(0, height - PAD_T - PAD_B);
    const hoverLeftPct = hoverIdx != null && data.length > 1 ? (hoverIdx / (data.length - 1)) * 100 : hoverIdx != null ? 50 : 0;
    const hoverTopPx = hoverIdx != null ? PAD_T + (1 - data[hoverIdx] / max) * plotInnerH : 0;
    const cardTransform = hoverIdx == null ? "" : hoverLeftPct < 12 ? "translate(0, calc(-100% - 10px))" : hoverLeftPct > 88 ? "translate(-100%, calc(-100% - 10px))" : "translate(-50%, calc(-100% - 10px))";

    // Banner bands active on the hovered column. Used to enrich the tooltip.
    const activeBands: IBannerBand[] = hoverIdx == null ? [] : bands.nonEmptyTracks.flatMap((track) => bands.byTrack[track].filter((b) => hoverIdx >= b.startIdx && hoverIdx <= b.endIdx));

    return (
        <div className={styles.activityChart} style={{ height }}>
            <div className={styles.acYaxis} style={{ paddingTop: PAD_T, paddingBottom: PAD_B }}>
                {yticks.map((v) => (
                    <span key={v} className={styles.acYtick}>
                        {fmtY(v)}
                    </span>
                ))}
            </div>
            <div className={styles.acPlot} style={{ paddingTop: PAD_T, paddingBottom: PAD_B }} ref={plotRef} onPointerMove={onPointerMove} onPointerLeave={onPointerLeave}>
                <svg viewBox={`0 0 ${VBW} ${VBH}`} preserveAspectRatio="none" style={{ width: "100%", height: "100%", display: "block", overflow: "visible" }} role="presentation" aria-hidden="true">
                    <defs>
                        <linearGradient id={fillId} x1="0" x2="0" y1="0" y2="1">
                            <stop offset="0%" stopColor={color} stopOpacity="0.25" />
                            <stop offset="100%" stopColor={color} stopOpacity="0" />
                        </linearGradient>
                    </defs>
                    {yticks.map((_, i) => {
                        const y = (i / (yticks.length - 1)) * VBH;
                        return (
                            <line
                                // biome-ignore lint/suspicious/noArrayIndexKey: gridline index is stable
                                key={i}
                                x1="0"
                                x2={VBW}
                                y1={y}
                                y2={y}
                                stroke="var(--border)"
                                strokeWidth="1"
                                strokeDasharray={i === yticks.length - 1 ? "0" : "2 3"}
                                vectorEffect="non-scaling-stroke"
                            />
                        );
                    })}
                    {data.length > 0 ? <path d={`${linePath(data)} L ${VBW} ${VBH} L 0 ${VBH} Z`} fill={`url(#${fillId})`} /> : null}
                    {data.length > 0 ? <path d={linePath(data)} fill="none" stroke={color} strokeWidth="1.75" strokeLinejoin="round" strokeLinecap="round" vectorEffect="non-scaling-stroke" /> : null}
                </svg>
                {hoverIdx != null ? (
                    <>
                        <div className={styles.acGuide} style={{ left: `${hoverLeftPct}%`, top: PAD_T, bottom: PAD_B }} />
                        <div className={styles.acDot} style={{ left: `${hoverLeftPct}%`, top: hoverTopPx, background: color }} />
                        <div className={styles.acHoverCard} style={{ left: `${hoverLeftPct}%`, top: hoverTopPx, transform: cardTransform }}>
                            <div className={styles.acHoverCardDate}>{days[hoverIdx]}</div>
                            <div className={styles.acHoverCardValue}>
                                <span aria-hidden className={styles.acHoverCardSwatch} style={{ background: color }} />
                                {data[hoverIdx].toLocaleString()}
                                <span className={styles.acHoverCardLabel}>pulls</span>
                            </div>
                            {activeBands.length > 0 ? (
                                <div className={styles.acHoverCardBanners}>
                                    {activeBands.slice(0, 5).map((b) => (
                                        <div key={b.poolId} className={styles.acHoverCardBannerRow}>
                                            <span aria-hidden className={styles.acHoverCardSwatch} style={{ background: TYPE_COLORS[b.group] }} />
                                            <span className={styles.acHoverCardBannerName}>{b.name}</span>
                                        </div>
                                    ))}
                                    {activeBands.length > 5 ? <div className={styles.acHoverCardBannerMore}>+{activeBands.length - 5} more</div> : null}
                                </div>
                            ) : null}
                        </div>
                    </>
                ) : null}

                {/* Banner-run tracks: one row per rule-type that has banners in window. */}
                {bands.nonEmptyTracks.length > 0 ? (
                    <div className={styles.acBannerTracks} style={{ bottom: X_AXIS_H, height: tracksH - TRACKS_TOP_GAP }}>
                        {bands.nonEmptyTracks.map((track, ti) => (
                            <div
                                key={track}
                                className={styles.acBannerTrack}
                                style={{
                                    top: ti * (TRACK_H + TRACK_GAP),
                                    height: TRACK_H,
                                }}
                            >
                                {bands.byTrack[track].map((b) => {
                                    const leftPct = data.length > 1 ? (b.startIdx / (data.length - 1)) * 100 : 0;
                                    const rightPct = data.length > 1 ? (b.endIdx / (data.length - 1)) * 100 : 100;
                                    // Single-day banners would render zero width; give them a minimum of 0.6% of the chart so they remain visible.
                                    const widthPct = Math.max(0.6, rightPct - leftPct);
                                    return (
                                        <div
                                            key={b.poolId}
                                            className={styles.acBannerBand}
                                            style={{
                                                left: `${leftPct}%`,
                                                width: `${widthPct}%`,
                                                background: TYPE_COLORS[b.group],
                                            }}
                                            title={`${b.name} — ${TYPE_LABELS[b.group]}`}
                                        />
                                    );
                                })}
                            </div>
                        ))}
                    </div>
                ) : null}

                <div className={styles.acXaxis}>
                    {xTickIdx.map((idx) => {
                        const left = days.length > 1 ? (idx / (days.length - 1)) * 100 : 0;
                        const transform = idx === 0 ? "translateX(0)" : idx === days.length - 1 ? "translateX(-100%)" : "translateX(-50%)";
                        return (
                            <span key={idx} className={styles.acXtick} style={{ left: `${left}%`, transform }}>
                                {days[idx]}
                            </span>
                        );
                    })}
                </div>
            </div>
        </div>
    );
}
