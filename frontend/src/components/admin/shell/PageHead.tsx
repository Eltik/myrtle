export interface IPageHeadProps {
    kicker: string;
    title: string;
    sub?: React.ReactNode;
    action?: React.ReactNode;
}

/** The heading every admin section opens with: kicker, title, optional sub line and actions. */
export function PageHead({ kicker, title, sub, action }: IPageHeadProps): React.ReactElement {
    return (
        <div className="mb-5 flex flex-col gap-3 border-border border-b pb-4 sm:flex-row sm:items-end sm:justify-between sm:gap-4">
            <div className="flex min-w-0 flex-col gap-1.5">
                <span className="font-bold text-[11px] text-primary uppercase tracking-[0.22em]">{kicker}</span>
                <h1 className="font-semibold text-[22px] leading-tight tracking-[-0.02em] sm:text-[26px]">{title}</h1>
                {sub ? <p className="max-w-[64ch] text-[13px] text-muted-foreground leading-[1.55] sm:text-[13.5px]">{sub}</p> : null}
            </div>
            {action ? <div className="flex flex-wrap items-center gap-2">{action}</div> : null}
        </div>
    );
}
