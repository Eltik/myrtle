import { Kicker } from "#/components/ui/kicker";
import type { IDatePullData, IDayOfWeekPullData, IHourlyPullData, IPullTimingData } from "#/lib/api/gacha";
import styles from "./CommunityPage.module.css";
import { fmtPct } from "./format";

interface ITimingPanelProps {
    timing: IPullTimingData | null | undefined;
}

export function TimingPanel({ timing }: ITimingPanelProps) {
    if (!timing) return null;

    return (
        <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-[18px_18px] sm:p-[22px_24px]">
            <header className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between sm:gap-4">
                <div>
                    <Kicker className="mb-1.5">Pull timing</Kicker>
                    <h2 className="m-0 font-sans text-[20px] font-semibold leading-[1.15] tracking-[-0.02em] text-foreground text-balance sm:text-[22px]">When the community pulls.</h2>
                </div>
                <div className="inline-flex items-center gap-3.5 font-mono text-[10.5px] uppercase tracking-[0.12em] text-muted-foreground">
                    <span className="inline-flex items-center">
                        <span aria-hidden className="mr-1.5 inline-block h-2 w-2 rounded-sm bg-primary align-middle" />
                        Total pulls
                    </span>
                </div>
            </header>

            <div className="grid gap-y-4 gap-x-6 grid-cols-1 md:grid-cols-2">
                {timing.byDate && timing.byDate.length > 0 ? (
                    <div className="md:col-span-2">
                        <div className="mb-2.5 font-mono text-[10.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground">By date · last {timing.byDate.length} days</div>
                        <TimingByDate rows={timing.byDate} />
                    </div>
                ) : null}

                <div>
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

function TimingByDate({ rows }: { rows: IDatePullData[] }) {
    const series = rows.map((r) => r.pullCount);
    const days = rows.map((r) => {
        const d = new Date(r.date);
        if (Number.isNaN(d.getTime())) return r.date;
        return `${d.toLocaleString("en", { month: "short" })} ${d.getDate()}`;
    });
    return <ActivityChart days={days} data={series} color="var(--primary)" height={180} />;
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

function ActivityChart({ data, days, color, height }: { data: number[]; days: string[]; color: string; height: number }) {
    const PAD_T = 8;
    const PAD_B = 24;
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

    return (
        <div className={styles.activityChart} style={{ height }}>
            <div className={styles.acYaxis} style={{ paddingTop: PAD_T, paddingBottom: PAD_B }}>
                {yticks.map((v) => (
                    <span key={v} className={styles.acYtick}>
                        {fmtY(v)}
                    </span>
                ))}
            </div>
            <div className={styles.acPlot} style={{ paddingTop: PAD_T, paddingBottom: PAD_B }}>
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
