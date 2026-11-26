import { fmtRelative, fmtUTCStamp } from "#/components/gacha/community/impl/format";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./PageHeader.messages";
import styles from "./StatsPage.module.css";

interface IPageHeaderProps {
    computedAt: string | null | undefined;
}

export function PageHeader({ computedAt }: IPageHeaderProps) {
    const t: TypedT<typeof messages> = useT("stats");
    return (
        <div className="flex flex-col gap-3.5 pt-1.5">
            <div className="flex max-w-180 flex-col items-start">
                <h1 className="m-0 mb-3 text-balance font-bold font-sans text-[32px] text-foreground leading-[1.05] tracking-[-0.03em] sm:text-[38px] sm:leading-[1.03] sm:tracking-[-0.035em] lg:text-[44px] lg:leading-[1.02]">{t("header.title")}</h1>
                <p className="m-0 max-w-[62ch] font-sans text-muted-foreground">{t("header.blurb")}</p>
            </div>
            <div className="flex flex-wrap gap-2">
                <span className="inline-flex h-6 items-center gap-1.5 rounded-md border border-border bg-card px-2.5 font-sans text-[11.5px] text-muted-foreground">
                    <span className={styles.dotPulse} aria-hidden="true" />
                    {t("header.updated", { when: fmtRelative(computedAt) })}
                </span>
                <span className="inline-flex h-6 items-center rounded-md border border-border bg-card px-2.5 font-mono text-[11.5px] text-muted-foreground tabular-nums">{fmtUTCStamp(computedAt)}</span>
            </div>
        </div>
    );
}
