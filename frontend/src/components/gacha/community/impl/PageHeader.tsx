import { PageHeader as BasePageHeader } from "#/components/ui/page-header";
import type { IGachaEnhancedStats } from "#/lib/api/gacha";
import { type TypedRichT, useFormatters, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { fmtRelative, fmtUTCStamp } from "./format";
import type { messages as formatMessages } from "./format.messages";
import type { messages } from "./PageHeader.messages";

interface IPageHeaderProps {
    data: IGachaEnhancedStats | null;
    isLoading: boolean;
}

export function PageHeader({ data, isLoading }: IPageHeaderProps) {
    const t: TypedT<typeof messages & typeof formatMessages> = useT("gacha");
    const rt: TypedRichT<typeof messages> = useRichT("gacha");
    const f = useFormatters();
    const totalUsers = data?.collectiveStats.totalUsers;
    const updatedAt = data?.computedAt;
    const cached = data?.cached;

    return (
        <div className="flex flex-col gap-3.5">
            <BasePageHeader
                breadcrumbLabel={t("community.breadcrumb.label")}
                breadcrumb={[t("community.breadcrumb.gacha"), t("community.breadcrumb.current")]}
                title={t("community.header.title")}
                description={rt("community.header.blurb", {
                    players: <strong className="font-semibold text-foreground">{isLoading || totalUsers == null ? "-" : t("community.header.blurbUsers", { count: f.number(totalUsers) })}</strong>,
                })}
            />
            <div className="flex flex-wrap gap-2">
                <span className="inline-flex h-6 items-center gap-1.5 rounded-md border border-border bg-card/80 px-2.5 font-sans text-[11.5px] text-muted-foreground">
                    {/* Static, and green only for a fresh computation: a looping pulse said "live" even
                        over figures served from the cache. The Cached chip beside it says which. */}
                    <span aria-hidden className={`size-1.5 shrink-0 rounded-full ${updatedAt != null && !cached ? "bg-success" : "bg-muted-foreground"}`} />
                    {t("community.header.updated", { when: fmtRelative(updatedAt, t) })}
                </span>
                {cached ? <span className="inline-flex h-6 items-center rounded-md border border-border bg-card/80 px-2.5 font-mono text-[10.5px] text-muted-foreground">{t("community.header.cached")}</span> : null}
                <span className="inline-flex h-6 items-center rounded-md border border-border bg-card/80 px-2.5 font-mono text-[10.5px] text-muted-foreground">{fmtUTCStamp(updatedAt)}</span>
            </div>
        </div>
    );
}
