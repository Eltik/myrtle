import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Skeleton } from "#/components/ui/skeleton";
import { statsQueryOptions } from "#/lib/api/stats";
import { type TypedRichT, useFormatters, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { CatalogGrid } from "./impl/CatalogGrid";
import { FieldNotes } from "./impl/FieldNotes";
import { KpiCell } from "./impl/KpiStrip";
import { PageHeader } from "./impl/PageHeader";
import styles from "./impl/StatsPage.module.css";
import type { messages } from "./StatsPage.messages";

const SKELETON = <span className="text-subtle-foreground">-</span>;

export function StatsPage() {
    const t: TypedT<typeof messages> = useT("stats");
    const rt: TypedRichT<typeof messages> = useRichT("stats");
    const f = useFormatters();
    const { data, isError, error } = useQuery(statsQueryOptions());

    return (
        <>
            <div className={styles.pageAmbient} aria-hidden="true" />
            <section className="page-shell flex flex-col gap-6">
                <PageHeader computedAt={data?.computedAt} />

                {isError ? (
                    <div className="rounded-2xl border border-destructive/30 bg-destructive/8 px-5 py-4 font-sans text-foreground/90 text-sm">
                        <strong className="font-semibold text-foreground">{t("page.error.load")}</strong> {(error as Error)?.message ?? t("page.error.unknown")}
                    </div>
                ) : null}

                <section className="flex flex-col gap-4">
                    <header className="flex flex-col gap-1">
                        <h2 className="m-0 text-balance font-sans font-semibold text-[22px] text-foreground leading-[1.15] tracking-[-0.02em]">{t("page.catalog.title")}</h2>
                        <p className="m-0 font-sans text-[13px] text-muted-foreground leading-normal">{t("page.catalog.blurb")}</p>
                    </header>
                    {data ? <CatalogGrid gameData={data.gameData} rosters={data.rosters} /> : <Skeleton className="h-[328px] rounded-2xl border border-border lg:h-[164px]" />}
                    <FieldNotes />
                </section>

                <section className="flex flex-col gap-4">
                    <header className="flex flex-col gap-1">
                        <h2 className="m-0 text-balance font-sans font-semibold text-[22px] text-foreground leading-[1.15] tracking-[-0.02em]">{t("page.community.title")}</h2>
                        <p className="m-0 font-sans text-[13px] text-muted-foreground leading-normal">
                            {rt("page.community.blurb", {
                                versions: <strong className="font-semibold text-foreground">{t("page.community.blurb.versions")}</strong>,
                                placements: <strong className="font-semibold text-foreground">{t("page.community.blurb.placements")}</strong>,
                            })}
                        </p>
                    </header>
                    <div className="grid grid-cols-[repeat(4,1fr)] overflow-hidden rounded-2xl border border-border bg-card max-[1180px]:grid-cols-2 max-[520px]:grid-cols-1">
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
