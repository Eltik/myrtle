import { Kicker } from "#/components/ui/kicker";
import type { IGachaEnhancedStats } from "#/lib/api/gacha";
import { formatNumber } from "#/lib/utils";
import styles from "./CommunityPage.module.css";
import { fmtRelative, fmtUTCStamp } from "./format";

interface IPageHeaderProps {
    data: IGachaEnhancedStats | null;
    isLoading: boolean;
}

export function PageHeader({ data, isLoading }: IPageHeaderProps) {
    const totalUsers = data?.collectiveStats.totalUsers;
    const updatedAt = data?.computedAt;
    const cached = data?.cached;

    return (
        <div className="flex flex-col gap-3.5 pt-1.5">
            <div className="flex max-w-180 flex-col items-start">
                <Kicker>Live · community statistics</Kicker>
                <h1 className="m-0 mb-3 font-sans text-[32px] font-bold leading-[1.05] tracking-[-0.03em] text-foreground text-balance sm:text-[38px] sm:leading-[1.03] sm:tracking-[-0.035em] lg:text-[44px] lg:leading-[1.02]">
                    What everyone&rsquo;s <em className="not-italic text-primary">pulling</em>.
                </h1>
                <p className="m-0 max-w-[60ch] font-sans text-muted-foreground">
                    Aggregated across <strong className="font-semibold text-foreground">{isLoading || totalUsers == null ? "-" : `${formatNumber(totalUsers)} doctors`}</strong> who opted into anonymous sharing. Rates are observational, not official.
                </p>
            </div>
            <div className="flex flex-wrap gap-2">
                <span className="inline-flex h-6 items-center gap-1.5 rounded-md border border-border bg-card/80 px-2.5 font-sans text-[11.5px] text-muted-foreground">
                    <span className={styles.dotPulse} aria-hidden />
                    updated {fmtRelative(updatedAt)}
                </span>
                {cached ? <span className="inline-flex h-6 items-center rounded-md border border-border bg-card/80 px-2.5 font-mono text-[10.5px] uppercase tracking-widest text-muted-foreground">cached</span> : null}
                <span className="inline-flex h-6 items-center rounded-md border border-border bg-card/80 px-2.5 font-mono text-[10.5px] uppercase tracking-widest text-muted-foreground">{fmtUTCStamp(updatedAt)}</span>
            </div>
        </div>
    );
}
