import { Info } from "lucide-react";
import { formatNumberCompact } from "#/lib/utils";

interface IHeroProps {
    rankedDoctors: number | null;
    topScore: number | null;
    updatedAt: string | null;
    isLoading?: boolean;
}

function formatRelative(iso: string | null): string {
    if (!iso) return "-";
    const ts = Date.parse(iso);
    if (Number.isNaN(ts)) return "-";
    const seconds = Math.max(0, Math.floor((Date.now() - ts) / 1000));
    if (seconds < 60) return "moments ago";
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
}

export function Hero({ rankedDoctors, topScore, updatedAt, isLoading }: IHeroProps) {
    return (
        <section className="relative overflow-hidden rounded-2xl border border-border bg-card shadow-sm">
            <div aria-hidden className="pointer-events-none absolute inset-x-0 -top-px h-px bg-linear-to-r from-transparent via-primary/60 to-transparent" />
            <div
                aria-hidden
                className="pointer-events-none absolute inset-0"
                style={{
                    background: "radial-gradient(ellipse 80% 60% at 60% 40%, color-mix(in oklab, var(--primary) 7%, transparent), transparent 70%), radial-gradient(ellipse 60% 80% at 80% 20%, oklch(0.696 0.17 162 / 0.06), transparent 70%)",
                }}
            />

            <div className="relative grid gap-x-7 gap-y-4 px-5 py-5 sm:grid-cols-[1fr_auto] sm:gap-y-5 sm:px-6 sm:py-6">
                <div className="flex min-w-0 flex-col gap-1.5">
                    <span className="font-medium font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-[0.16em]">Players · Leaderboard</span>
                    <h1 className="m-0 flex flex-wrap items-center gap-2.5 font-bold font-sans text-2xl text-foreground leading-[1.1] tracking-tight sm:text-[28px]">
                        Top Doctors
                        <span className="inline-flex h-5.5 items-center rounded-full border border-primary/30 bg-primary/10 px-2 font-mono font-semibold text-[10.5px] text-primary uppercase leading-none tracking-[0.18em]">All-time</span>
                    </h1>
                    <p className="m-0 max-w-[60ch] font-sans text-[13.5px] text-muted-foreground leading-snug">A composite ranking across roster depth, base efficiency, and combat scoring. Pulled from public profiles only.</p>
                </div>

                <div className="flex items-center gap-4 self-center sm:gap-5">
                    <HeroStat label="Doctors ranked" value={rankedDoctors == null ? "-" : formatNumberCompact(rankedDoctors)} loading={isLoading} />
                    <span aria-hidden className="h-10 w-px bg-border" />
                    <HeroStat label="Top score" value={topScore == null ? "-" : formatNumberCompact(topScore)} loading={isLoading} />
                    <span aria-hidden className="h-10 w-px bg-border" />
                    <HeroStat label="Updated" value={formatRelative(updatedAt)} small loading={isLoading} />
                </div>

                <div role="note" className="col-span-full flex items-start gap-2 rounded-lg border border-info/25 bg-info/8 px-3 py-2.5 font-medium font-sans text-[12px] text-foreground/90 leading-snug">
                    <Info className="mt-px size-3.5 shrink-0 text-info" aria-hidden />
                    <span>
                        <b className="font-semibold text-foreground">Heads up</b> - the leaderboard is in active development and the ranking formula is subject to change.
                    </span>
                </div>
            </div>
        </section>
    );
}

function HeroStat({ label, value, small, loading }: { label: string; value: string; small?: boolean; loading?: boolean }) {
    return (
        <div className="flex min-w-0 flex-col gap-1">
            <span className={`font-bold font-sans text-foreground tabular-nums tracking-tight ${small ? "font-semibold text-sm" : "text-[22px] leading-none"}`}>{loading ? "…" : value}</span>
            <span className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.16em]">{label}</span>
        </div>
    );
}
