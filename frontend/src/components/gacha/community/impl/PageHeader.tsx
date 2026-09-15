import { Kicker } from "#/components/ui/kicker";
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
        <div className="flex flex-col gap-3.5 pt-1.5">
            <div className="flex max-w-180 flex-col items-start">
                <Kicker>{t("community.header.kicker")}</Kicker>
                <h1 className="m-0 mb-3 text-balance font-bold font-sans text-[32px] text-foreground leading-[1.05] tracking-[-0.03em] sm:text-[38px] sm:leading-[1.03] sm:tracking-[-0.035em] lg:text-[44px] lg:leading-[1.02]">
                    {rt("community.header.title", { emphasis: <em className="text-primary not-italic">{t("community.header.titleEmphasis")}</em> })}
                </h1>
                <p className="m-0 max-w-[60ch] font-sans text-muted-foreground">
                    {rt("community.header.blurb", {
                        doctors: <strong className="font-semibold text-foreground">{isLoading || totalUsers == null ? "-" : t("community.header.blurbUsers", { count: f.number(totalUsers) })}</strong>,
                    })}
                </p>
            </div>
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
