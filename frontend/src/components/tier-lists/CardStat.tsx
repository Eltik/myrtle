import type { ReactNode } from "react";

type StatKind = "views" | "favorites" | "views24h";

interface ICardStatProps {
    kind: StatKind;
    /** Already formatted for display (compact number). */
    value: string;
    /** Tooltip carrying the full, unabbreviated count. */
    title: string;
}

const ICONS: Record<StatKind, ReactNode> = {
    views: (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3 opacity-70" aria-hidden="true">
            <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
            <circle cx="12" cy="12" r="3" />
        </svg>
    ),
    favorites: (
        <svg viewBox="0 0 24 24" fill="currentColor" className="h-2.75 w-2.75 opacity-70" aria-hidden="true">
            <path d="M12 21s-7-4.5-9.5-9C.7 8.7 2.5 5 6 5c2 0 3.5 1 4 2.5C10.5 6 12 5 14 5c3.5 0 5.3 3.7 3.5 7-2.5 4.5-9.5 9-9.5 9Z" />
        </svg>
    ),
    views24h: (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3" aria-hidden="true">
            <path d="m6 14 6-6 6 6" />
        </svg>
    ),
};

/** One icon-plus-number stat in a tier-list card footer. The 24h trend reads in the accent colour. */
export function CardStat({ kind, value, title }: ICardStatProps) {
    const trend = kind === "views24h";
    return (
        <span className={trend ? "inline-flex items-center gap-0.5 text-primary" : "inline-flex items-center gap-1"} title={title}>
            {ICONS[kind]}
            <span className={trend ? "font-bold" : "font-semibold text-foreground"}>{value}</span>
        </span>
    );
}
