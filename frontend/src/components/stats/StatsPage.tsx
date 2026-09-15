import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useMemo } from "react";
import { Kicker } from "#/components/ui/kicker";
import { statsQueryOptions } from "#/lib/api/stats";
import { type TypedRichT, useFormatters, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { CatalogGrid } from "./impl/CatalogGrid";
import { type IKpiCell, KpiCell, KpiStrip } from "./impl/KpiStrip";
import { PageHeader } from "./impl/PageHeader";
import styles from "./impl/StatsPage.module.css";
import type { messages } from "./StatsPage.messages";

const SKELETON = <span className="text-muted-foreground/60">-</span>;

export function StatsPage() {
    const t: TypedT<typeof messages> = useT("stats");
    const rt: TypedRichT<typeof messages> = useRichT("stats");
    const f = useFormatters();
    const { data, isError, error } = useQuery(statsQueryOptions());

    const headlineCells = useMemo<IKpiCell[]>(() => {
        if (!data) {
            return [
                { featured: true, label: t("page.kpi.operators"), value: SKELETON, meta: <span>-</span> },
                { label: t("page.kpi.tierLists"), value: SKELETON, meta: <span>-</span> },
                { label: t("page.kpi.rosters"), value: SKELETON, meta: <span>-</span> },
            ];
        }
        return [
            {
                featured: true,
                label: t("page.kpi.operators"),
                value: f.number(data.gameData.operators),
                meta: <span>{t("page.kpi.operators.meta", { skills: f.number(data.gameData.skills), modules: f.number(data.gameData.modules) })}</span>,
            },
            {
                label: t("page.kpi.tierLists"),
                value: f.number(data.tierLists.active),
                meta: <span>{t("page.kpi.tierLists.meta", { total: f.number(data.tierLists.total), versions: f.number(data.tierLists.totalVersions) })}</span>,
            },
            {
                label: t("page.kpi.rosters"),
                value: f.compact(data.rosters.total),
                meta: <span>{t("page.kpi.rosters.meta")}</span>,
            },
        ];
    }, [data, t, f]);

    return (
        <>
            <div className={styles.pageAmbient} aria-hidden="true" />
            <section className="mx-auto flex w-full max-w-330 flex-col gap-6 px-8 pt-7 pb-15 max-[760px]:px-4 max-[760px]:pt-5 max-[760px]:pb-10">
                <PageHeader computedAt={data?.computedAt} />

                {isError ? (
                    <div className="rounded-[14px] border border-destructive/30 bg-destructive/8 px-5 py-4 font-sans text-foreground/90 text-sm">
                        <strong className="font-semibold text-foreground">{t("page.error.load")}</strong> {(error as Error)?.message ?? t("page.error.unknown")}
                    </div>
                ) : null}

                <KpiStrip cells={headlineCells} />

                <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-4.5 sm:p-6">
                    <header className="flex flex-col gap-1">
                        <Kicker>{t("page.catalog.kicker")}</Kicker>
                        <h2 className="m-0 text-balance font-sans font-semibold text-[22px] text-foreground leading-[1.15] tracking-[-0.02em]">{t("page.catalog.title")}</h2>
                        <p className="m-0 font-sans text-[13px] text-muted-foreground leading-normal">{t("page.catalog.blurb")}</p>
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
                        <Kicker>{t("page.community.kicker")}</Kicker>
                        <h2 className="m-0 text-balance font-sans font-semibold text-[22px] text-foreground leading-[1.15] tracking-[-0.02em]">{t("page.community.title")}</h2>
                        <p className="m-0 font-sans text-[13px] text-muted-foreground leading-normal">
                            {rt("page.community.blurb", {
                                versions: <strong className="font-semibold text-foreground">{t("page.community.blurb.versions")}</strong>,
                                placements: <strong className="font-semibold text-foreground">{t("page.community.blurb.placements")}</strong>,
                            })}
                        </p>
                    </header>
                    <div className="grid grid-cols-[repeat(4,1fr)] overflow-hidden rounded-[14px] border border-border bg-card max-[1180px]:grid-cols-2 max-[520px]:grid-cols-1">
                        {data ? (
                            <>
                                <KpiCell label={t("page.cell.totalLists")} value={f.number(data.tierLists.total)} meta={<span>{t("page.cell.totalLists.meta", { count: f.number(Math.max(0, data.tierLists.total - data.tierLists.active)) })}</span>} />
                                <KpiCell label={t("page.cell.active")} value={f.number(data.tierLists.active)} meta={<span>{data.tierLists.total > 0 ? t("page.cell.active.meta", { percent: Math.round((data.tierLists.active / data.tierLists.total) * 100) }) : "-"}</span>} />
                                <KpiCell label={t("page.cell.versions")} value={f.compact(data.tierLists.totalVersions)} meta={<span>{data.tierLists.total > 0 ? t("page.cell.avgPerList", { avg: (data.tierLists.totalVersions / data.tierLists.total).toFixed(1) }) : "-"}</span>} />
                                <KpiCell label={t("page.cell.placements")} value={f.compact(data.tierLists.totalPlacements)} meta={<span>{data.tierLists.total > 0 ? t("page.cell.avgPerList", { avg: Math.round(data.tierLists.totalPlacements / data.tierLists.total) }) : "-"}</span>} />
                            </>
                        ) : (
                            <>
                                <KpiCell label={t("page.cell.totalLists")} value={SKELETON} />
                                <KpiCell label={t("page.cell.active")} value={SKELETON} />
                                <KpiCell label={t("page.cell.versions")} value={SKELETON} />
                                <KpiCell label={t("page.cell.placements")} value={SKELETON} />
                            </>
                        )}
                    </div>
                </section>

                <p className="m-0 mt-1 max-w-[62ch] font-sans text-[12.5px] text-muted-foreground leading-[1.55]">
                    {rt("page.footer", {
                        link: (
                            <Link to="/gacha/community" className="text-primary no-underline hover:underline">
                                /gacha/community
                            </Link>
                        ),
                    })}
                </p>
            </section>
        </>
    );
}
