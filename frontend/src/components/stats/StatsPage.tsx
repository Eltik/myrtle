import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useMemo } from "react";
import { Kicker } from "#/components/ui/kicker";
import { statsQueryOptions } from "#/lib/api/stats";
import { formatNumber, formatNumberCompact } from "#/lib/utils";
import { CatalogGrid } from "./impl/CatalogGrid";
import { type IKpiCell, KpiCell, KpiStrip } from "./impl/KpiStrip";
import { PageHeader } from "./impl/PageHeader";
import styles from "./impl/StatsPage.module.css";

const SKELETON = <span className="text-muted-foreground/60">-</span>;

export function StatsPage() {
    const { data, isError, error } = useQuery(statsQueryOptions());

    const headlineCells = useMemo<IKpiCell[]>(() => {
        if (!data) {
            return [
                { featured: true, label: "Operators indexed", value: SKELETON, meta: <span>-</span> },
                { label: "Active tier lists", value: SKELETON, meta: <span>-</span> },
                { label: "Rosters synced", value: SKELETON, meta: <span>-</span> },
            ];
        }
        return [
            {
                featured: true,
                label: "Operators indexed",
                value: formatNumber(data.gameData.operators),
                meta: (
                    <span>
                        {formatNumber(data.gameData.skills)} skills · {formatNumber(data.gameData.modules)} modules
                    </span>
                ),
            },
            {
                label: "Active tier lists",
                value: formatNumber(data.tierLists.active),
                meta: (
                    <span>
                        {formatNumber(data.tierLists.total)} total · {formatNumber(data.tierLists.totalVersions)} versions
                    </span>
                ),
            },
            {
                label: "Rosters synced",
                value: formatNumberCompact(data.rosters.total),
                meta: <span>Yostar-linked doctors</span>,
            },
        ];
    }, [data]);

    return (
        <>
            <div className={styles.pageAmbient} aria-hidden="true" />
            <section className="mx-auto flex w-full max-w-330 flex-col gap-6 px-8 pt-7 pb-15 max-[760px]:px-4 max-[760px]:pt-5 max-[760px]:pb-10">
                <PageHeader computedAt={data?.computedAt} />

                {isError ? (
                    <div className="rounded-[14px] border border-destructive/30 bg-destructive/8 px-5 py-4 font-sans text-foreground/90 text-sm">
                        <strong className="font-semibold text-foreground">Couldn&rsquo;t load site stats.</strong> {(error as Error)?.message ?? "Unknown error."}
                    </div>
                ) : null}

                <KpiStrip cells={headlineCells} />

                <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-4.5 sm:p-6">
                    <header className="flex flex-col gap-1">
                        <Kicker>Game catalog</Kicker>
                        <h2 className="m-0 text-balance font-sans font-semibold text-[22px] text-foreground leading-[1.15] tracking-[-0.02em]">What we know about Terra.</h2>
                        <p className="m-0 font-sans text-[13px] text-muted-foreground leading-normal">A faithful mirror of Hypergryph&rsquo;s gamedata, recomputed on every build.</p>
                    </header>
                    {data ? (
                        <CatalogGrid gameData={data.gameData} tierLists={data.tierLists} rosters={data.rosters} />
                    ) : (
                        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
                            {["a", "b", "c", "d", "e", "f", "g", "h"].map((k) => (
                                <div key={k} className="h-28 animate-pulse rounded-xl border border-border bg-muted/40" />
                            ))}
                        </div>
                    )}
                </section>

                <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-4.5 sm:p-6">
                    <header className="flex flex-col gap-1">
                        <Kicker>Tier lists · community</Kicker>
                        <h2 className="m-0 text-balance font-sans font-semibold text-[22px] text-foreground leading-[1.15] tracking-[-0.02em]">The community&rsquo;s working notes.</h2>
                        <p className="m-0 font-sans text-[13px] text-muted-foreground leading-normal">
                            Counted across every public list. <strong className="font-semibold text-foreground">Versions</strong> are saved revisions; <strong className="font-semibold text-foreground">placements</strong> are individual operator-on-tier rows.
                        </p>
                    </header>
                    <div className="grid grid-cols-[repeat(4,1fr)] overflow-hidden rounded-[14px] border border-border bg-card max-[1180px]:grid-cols-2 max-[520px]:grid-cols-1">
                        {data ? (
                            <>
                                <KpiCell label="Total lists" value={formatNumber(data.tierLists.total)} meta={<span>{formatNumber(Math.max(0, data.tierLists.total - data.tierLists.active))} archived</span>} />
                                <KpiCell label="Active" value={formatNumber(data.tierLists.active)} meta={<span>{data.tierLists.total > 0 ? `${Math.round((data.tierLists.active / data.tierLists.total) * 100)}% of total` : "-"}</span>} />
                                <KpiCell label="Versions saved" value={formatNumberCompact(data.tierLists.totalVersions)} meta={<span>{data.tierLists.total > 0 ? `avg ${(data.tierLists.totalVersions / data.tierLists.total).toFixed(1)} per list` : "-"}</span>} />
                                <KpiCell label="Placements" value={formatNumberCompact(data.tierLists.totalPlacements)} meta={<span>{data.tierLists.total > 0 ? `avg ${Math.round(data.tierLists.totalPlacements / data.tierLists.total)} per list` : "-"}</span>} />
                            </>
                        ) : (
                            <>
                                <KpiCell label="Total lists" value={SKELETON} />
                                <KpiCell label="Active" value={SKELETON} />
                                <KpiCell label="Versions saved" value={SKELETON} />
                                <KpiCell label="Placements" value={SKELETON} />
                            </>
                        )}
                    </div>
                </section>

                <p className="m-0 mt-1 max-w-[62ch] font-sans text-[12.5px] text-muted-foreground leading-[1.55]">
                    Looking for gacha rates, most-pulled operators, or pull timing? Those live on{" "}
                    <Link to="/gacha/community" className="text-primary no-underline hover:underline">
                        /gacha/community
                    </Link>
                    . This page is the meta-count - just the shape of what we host.
                </p>
            </section>
        </>
    );
}
