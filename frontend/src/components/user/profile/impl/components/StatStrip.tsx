import { useState } from "react";

import { type IFormatters, useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";

import type { IUserProfile } from "#/types/user";
import { type HeaderField, isHidden, stripCards, stripGridClass } from "../header";
import { HiddenMarker } from "./HiddenMarker";
import type { messages } from "./StatStrip.messages";

interface IStatCardProps {
    kicker: string;
    /** The owner hid this number from visitors: they alone see the card, marked. */
    hidden?: boolean;
    value: string | number | null;
    accent?: boolean;
    live?: boolean;
    compactOnMobile?: boolean;
    unformatted?: boolean;
    onClick?: () => void;
}

function formatValue(value: string | number | null, f: IFormatters) {
    if (value === null || value === undefined) return "-";
    if (typeof value === "number") return f.number(value);
    return value;
}

function formatCompact(value: string | number | null) {
    if (value === null || value === undefined) return "-";
    if (typeof value !== "number") return value;
    const abs = Math.abs(value);
    const sign = value < 0 ? "-" : "";
    if (abs >= 1_000_000) return `${sign}${(abs / 1_000_000).toFixed(1)}mil`;
    if (abs >= 1_000) return `${sign}${(abs / 1_000).toFixed(1)}k`;
    return `${sign}${abs}`;
}

function StatCard({ kicker, hidden, value, accent, live, compactOnMobile, unformatted, onClick }: IStatCardProps) {
    const f = useFormatters();
    const interactive = typeof onClick === "function";
    const className = cn(
        "group relative flex flex-col overflow-hidden rounded-xl border border-border bg-card text-left sm:rounded-2xl",
        "shadow-[0_1px_2px_rgb(0_0_0/0.04)] transition-all duration-200",
        "hover:border-foreground/15 hover:shadow-[0_8px_24px_-12px_rgb(0_0_0/0.15)]",
        interactive && "cursor-pointer select-none",
    );
    const style = {
        padding: "clamp(0.75rem, 1.2vw + 0.5rem, 1.25rem)",
        gap: "clamp(0.375rem, 0.4vw + 0.25rem, 0.625rem)",
    };
    const inner = (
        <>
            {accent && <div aria-hidden className="pointer-events-none absolute inset-x-0 -top-px h-px bg-linear-to-r from-transparent via-primary/60 to-transparent" />}
            <div className="flex items-center gap-1.5 sm:gap-2">
                {live && (
                    <span className="relative inline-flex h-1.5 w-1.5 shrink-0">
                        <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-500 opacity-70" />
                        <span className="relative inline-flex h-1.5 w-1.5 rounded-full bg-emerald-500" />
                    </span>
                )}
                <span
                    className={cn("truncate font-medium font-mono text-muted-foreground uppercase tracking-wider sm:tracking-widest", hidden && "pr-5")}
                    style={{
                        fontSize: "clamp(0.625rem, 0.25vw + 0.55rem, 0.6875rem)",
                        lineHeight: 1.2,
                    }}
                >
                    {kicker}
                </span>
            </div>
            <div
                className={cn("whitespace-nowrap font-bold font-sans tabular-nums tracking-tight", accent ? "text-primary" : "text-foreground")}
                style={{
                    fontSize: "clamp(1.125rem, 1.6vw + 0.75rem, 1.875rem)",
                    lineHeight: 1.05,
                }}
            >
                {unformatted ? (
                    (value ?? "-")
                ) : compactOnMobile ? (
                    <>
                        <span className="sm:hidden">{formatCompact(value)}</span>
                        <span className="hidden sm:inline">{formatValue(value, f)}</span>
                    </>
                ) : (
                    formatValue(value, f)
                )}
            </div>
        </>
    );

    const card = interactive ? (
        <button className={className} onClick={onClick} style={style} type="button">
            {inner}
        </button>
    ) : (
        <div className={className} style={style}>
            {inner}
        </div>
    );
    if (!hidden) return card;
    // A sibling of the card, not inside it: the card may be a button, and the marker
    // carries its own tooltip and accessible name.
    return (
        <div className="relative grid">
            {card}
            <span className="absolute" style={{ top: style.padding, right: style.padding }}>
                <HiddenMarker />
            </span>
        </div>
    );
}

interface IStatStripProps {
    profile: IUserProfile;
    rosterCount?: number;
    /** The reader owns the profile: their hidden cards render, marked. */
    isOwner?: boolean;
}

/** Cards whose value reads compact on a phone until tapped, so large balances fit. */
type CompactCard = "orundum" | "originite" | "lmd";

export function StatStrip({ profile, rosterCount, isOwner = false }: IStatStripProps) {
    const t: TypedT<typeof messages> = useT("user");
    const [raw, setRaw] = useState<ReadonlySet<CompactCard>>(() => new Set());
    const layout = profile.profile_layout ?? null;
    const cards = stripCards(layout, isOwner);
    if (cards.length === 0) return null;

    const compact = (card: CompactCard) => ({
        compactOnMobile: !raw.has(card),
        unformatted: raw.has(card),
        onClick: () =>
            setRaw((prev) => {
                const next = new Set(prev);
                if (!next.delete(card)) next.add(card);
                return next;
            }),
    });

    const render = (card: HeaderField) => {
        const hidden = isOwner && isHidden(layout, card);
        switch (card) {
            case "operators":
                return <StatCard key={card} hidden={hidden} kicker={t("profile.strip.operators.kicker")} value={rosterCount ?? profile.operator_count} live />;
            case "skins":
                return <StatCard key={card} hidden={hidden} kicker={t("profile.strip.skins.kicker")} value={profile.non_default_skin_count} />;
            case "items":
                return <StatCard key={card} hidden={hidden} kicker={t("profile.strip.items.kicker")} value={profile.item_count} />;
            // The currency names below are the game's own and stay as game vocabulary.
            case "orundum":
                return <StatCard key={card} hidden={hidden} kicker="Orundum" value={profile.orundum} {...compact(card)} />;
            // `null` until the account syncs again: the field arrived on 2026-09-15 and is filled per refresh.
            case "originite":
                return <StatCard key={card} hidden={hidden} kicker="Originite Prime" value={profile.originite} {...compact(card)} />;
            case "lmd":
                return <StatCard key={card} hidden={hidden} kicker="LMD" value={profile.lmd} {...compact(card)} />;
            case "headhunting_permits":
                return <StatCard key={card} hidden={hidden} kicker="Headhunting Permits" value={profile.gacha_tickets} />;
            case "ten_pull_permits":
                return <StatCard key={card} hidden={hidden} kicker="Ten-roll Permits" value={profile.ten_pull_tickets} />;
            default:
                return null;
        }
    };

    return (
        <div className={cn("grid", stripGridClass(cards.length))} style={{ gap: "clamp(0.5rem, 0.6vw + 0.375rem, 0.875rem)" }}>
            {cards.map(render)}
        </div>
    );
}
