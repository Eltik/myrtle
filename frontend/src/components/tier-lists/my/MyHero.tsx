import { PlusIcon } from "lucide-react";
import { Button } from "#/components/ui/button";
import { Kicker } from "#/components/ui/kicker";
import { formatNumber, formatNumberCompact } from "#/lib/utils";

interface IMyHeroProps {
    total: number;
    communityCount: number;
    communityQuota: number;
    officialCount: number;
    totalViews: number;
    totalFavorites: number;
    onCreate: () => void;
}

interface IStatPillProps {
    label: string;
    value: string;
}

function StatPill({ label, value }: IStatPillProps) {
    return (
        <div className="flex min-w-0 items-center gap-2 rounded-lg border border-border bg-card px-3 py-2 shadow-[0_1px_2px_oklch(0_0_0/0.04)]">
            <div className="flex flex-col gap-1">
                <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-wider">{label}</span>
                <span className="font-sans font-semibold text-[15px] text-foreground tabular-nums leading-none">{value}</span>
            </div>
        </div>
    );
}

export function MyHero({ total, communityCount, communityQuota, officialCount, totalViews, totalFavorites, onCreate }: IMyHeroProps) {
    const atQuota = communityCount >= communityQuota;
    const quotaPct = Math.min(100, Math.round((communityCount / communityQuota) * 100));

    return (
        <section className="mx-auto w-[min(1080px,calc(100%-2rem))] pt-10 pb-6 sm:pt-14 sm:pb-8">
            <div className="flex flex-wrap items-end justify-between gap-4">
                <div className="min-w-0 flex-1">
                    <Kicker>My Tier Lists</Kicker>
                    <h1 className="m-0 font-bold font-sans text-3xl text-foreground leading-tight tracking-tight sm:text-4xl">Your workshop.</h1>
                    <p className="mt-2 max-w-130 font-sans text-muted-foreground text-sm leading-relaxed">
                        {total > 0 ? (
                            <>
                                Manage, refine, and share <span className="font-mono text-foreground tabular-nums">{formatNumber(total)}</span> {total === 1 ? "list" : "lists"} you've built. Edits sync to anyone who visits.
                            </>
                        ) : (
                            <>You haven't created any tier lists yet. Spin up your first one - drafts publish instantly to your share link.</>
                        )}
                    </p>
                </div>

                <Button onClick={onCreate} disabled={atQuota} title={atQuota ? `You've reached the community list cap (${communityQuota})` : undefined}>
                    <PlusIcon />
                    <span>New tier list</span>
                </Button>
            </div>

            <div className="mt-6 grid grid-cols-2 gap-2 sm:grid-cols-4">
                <StatPill label="Total lists" value={formatNumber(total)} />
                <div className="relative overflow-hidden rounded-lg border border-border bg-card px-3 py-2 shadow-[0_1px_2px_oklch(0_0_0/0.04)]">
                    <div className="flex flex-col gap-1">
                        <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-wider">Community quota</span>
                        <span className="font-sans font-semibold text-[15px] text-foreground tabular-nums leading-none">
                            {communityCount}
                            <span className="text-muted-foreground"> / {communityQuota}</span>
                        </span>
                    </div>
                    <div aria-hidden="true" className="mt-2 h-1 w-full overflow-hidden rounded-full bg-muted">
                        <div
                            className="h-full rounded-full transition-[width] duration-300"
                            style={{
                                width: `${quotaPct}%`,
                                background: atQuota ? "var(--destructive)" : "var(--primary)",
                                boxShadow: atQuota ? "none" : "0 0 8px color-mix(in srgb, var(--primary) 50%, transparent)",
                            }}
                        />
                    </div>
                </div>
                <StatPill label="Total views" value={formatNumberCompact(totalViews)} />
                <StatPill label="Total favorites" value={formatNumberCompact(totalFavorites)} />
            </div>

            {officialCount > 0 && (
                <p className="mt-3 inline-flex items-center gap-1.5 font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-wider">
                    <span className="inline-block h-1.5 w-1.5 rounded-full bg-[oklch(0.78_0.15_92)]" aria-hidden="true" />
                    {officialCount} official {officialCount === 1 ? "list" : "lists"} included
                </p>
            )}
        </section>
    );
}
