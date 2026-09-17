import { PageHeader as BasePageHeader } from "#/components/ui/page-header";
import type { IGachaEnhancedStats } from "#/lib/api/gacha";
import { type TypedRichT, useFormatters, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import styles from "./CommunityPage.module.css";
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
            {/* The page ran its own 32/38/44px hero title behind a kicker while
                every other page ran the shared 24/30px header, which is the
                inconsistency raised in #ui-ux. It uses the shared
                with-description design now; the freshness chips stay below. */}
            <BasePageHeader
                breadcrumbLabel={t("community.breadcrumb.label")}
                breadcrumb={[t("community.breadcrumb.gacha"), t("community.breadcrumb.current")]}
                title={rt("community.header.title", { emphasis: <em className="text-primary not-italic">{t("community.header.titleEmphasis")}</em> })}
                description={rt("community.header.blurb", {
                    doctors: <strong className="font-semibold text-foreground">{isLoading || totalUsers == null ? "-" : t("community.header.blurbUsers", { count: f.number(totalUsers) })}</strong>,
                })}
            />
            <div className="flex flex-wrap gap-2">
                <span className="inline-flex h-6 items-center gap-1.5 rounded-md border border-border bg-card/80 px-2.5 font-sans text-[11.5px] text-muted-foreground">
                    <span className={styles.dotPulse} aria-hidden />
                    {t("community.header.updated", { when: fmtRelative(updatedAt, t) })}
                </span>
                {cached ? <span className="inline-flex h-6 items-center rounded-md border border-border bg-card/80 px-2.5 font-mono text-[10.5px] text-muted-foreground uppercase tracking-widest">{t("community.header.cached")}</span> : null}
                <span className="inline-flex h-6 items-center rounded-md border border-border bg-card/80 px-2.5 font-mono text-[10.5px] text-muted-foreground uppercase tracking-widest">{fmtUTCStamp(updatedAt)}</span>
            </div>
        </div>
    );
}
