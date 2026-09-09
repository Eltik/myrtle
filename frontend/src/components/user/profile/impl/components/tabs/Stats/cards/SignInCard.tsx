import { CalendarCheck, CalendarDays, Check, Clock } from "lucide-react";
import { countGameDays, gameDate } from "#/lib/registry/server-time";
import { cn } from "#/lib/utils";
import type { IUserCheckin } from "#/types/user";
import { PALETTE } from "../palette";
import { CARD_PADDING, KICKER_TEXT, Kicker, MetricRow, StatCard, Tile } from "../primitives";

interface ICardProps {
    checkin: IUserCheckin | null | undefined;
    server: string;
}

const SIGNIN = PALETTE.signin;

/**
 * The month's sign-in state as of the last sync, from the one thing the game
 * actually tells us: how many of the month's reward slots have been claimed.
 *
 * The sign-in calendar is a sequential list of reward slots, not a dated
 * calendar - miss a day and you fall one slot behind, you never forfeit the
 * slot. So "which days were claimed" is not answerable and must not be drawn;
 * "N of D slots claimed, as of the sync" is.
 */
function monthState(checkin: IUserCheckin, server: string) {
    const sync = new Date(checkin.updated_at);
    // The snapshot belongs to the game day it was taken on, which lives in
    // SERVER time (04:00 reset) - the viewer's local date may be a day ahead.
    const { year, month, day } = gameDate(Math.floor(sync.getTime() / 1000), server);
    const daysInMonth = new Date(year, month + 1, 0).getDate();
    const claimed = Math.max(0, Math.min(checkin.claimed_this_month, daysInMonth));
    // Days elapsed in the sync's month. `claimed` can never exceed it, but a
    // stale row shouldn't be able to render "10 / 9".
    const elapsed = Math.min(daysInMonth, Math.max(day, claimed));
    return { sync, year, month, daysInMonth, claimed, elapsed, behind: elapsed - claimed };
}

/** "3 weeks ago", "yesterday", etc. Client-rendered, so `Date.now()` is fine. */
function relativeTime(date: Date): string {
    const diffSec = Math.round((date.getTime() - Date.now()) / 1000);
    const abs = Math.abs(diffSec);
    const rtf = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
    if (abs < 60) return rtf.format(diffSec, "second");
    if (abs < 3600) return rtf.format(Math.round(diffSec / 60), "minute");
    if (abs < 86_400) return rtf.format(Math.round(diffSec / 3600), "hour");
    if (abs < 2_592_000) return rtf.format(Math.round(diffSec / 86_400), "day");
    if (abs < 31_536_000) return rtf.format(Math.round(diffSec / 2_592_000), "month");
    return rtf.format(Math.round(diffSec / 31_536_000), "year");
}

/** "2y 7mo" / "8mo" / "12d" from a day count. */
function humanAge(days: number): string {
    if (days < 31) return `${days}d`;
    const years = Math.floor(days / 365);
    const months = Math.floor((days % 365) / 30.44);
    if (years > 0) return months > 0 ? `${years}y ${months}mo` : `${years}y`;
    return `${months}mo`;
}

function StatRow({ label, value, title }: { label: string; value: string; title?: string }) {
    return (
        <div className="flex items-center justify-between gap-2 border-border/40 border-t pt-2.5" title={title}>
            <span className={KICKER_TEXT}>{label}</span>
            <span className="font-mono text-[11px] text-foreground tabular-nums">{value}</span>
        </div>
    );
}

/** Left card: lifetime / account-level sign-in engagement. */
export function SignInOverviewCard({ checkin, server }: ICardProps) {
    if (!checkin) return null;

    const { cumulative_signin, register_ts, last_online_ts } = checkin;

    const ageDays = register_ts ? countGameDays(register_ts, last_online_ts ?? Math.floor(Date.now() / 1000), server) : null;
    // Floor (not round) so e.g. 915/919 reads 99%, never a misleading 100%.
    const rate = ageDays !== null ? Math.min(100, Math.floor((cumulative_signin / ageDays) * 100)) : null;
    const missed = ageDays !== null ? Math.max(0, ageDays - cumulative_signin) : null;
    const { claimed, elapsed } = monthState(checkin, server);

    return (
        <StatCard color={SIGNIN}>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <Kicker icon={CalendarDays} label="Sign-In Record" />

                <div className="grid grid-cols-2 gap-3">
                    <Tile color={SIGNIN} sub="total sign-ins" tooltip="Lifetime cumulative sign-in days" value={cumulative_signin.toLocaleString()} />
                    {ageDays !== null && <Tile color={SIGNIN} sub="days since joining" value={ageDays.toLocaleString()} />}
                </div>

                {rate !== null && <MetricRow color={SIGNIN} label="Sign-in rate" pct={rate} value={`${rate}%`} />}

                <div className="flex flex-col gap-2.5">
                    {register_ts ? <StatRow label="Member since" value={new Date(register_ts * 1000).toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" })} /> : null}
                    {ageDays !== null && <StatRow label="Account age" value={humanAge(ageDays)} />}
                    <StatRow label="This month" title="Sign-ins claimed this month, out of the days elapsed" value={`${claimed} / ${elapsed}`} />
                    {missed !== null && <StatRow label="Days missed" title="Days since joining without a sign-in" value={missed.toLocaleString()} />}
                    {last_online_ts ? <StatRow label="Last online" title={new Date(last_online_ts * 1000).toLocaleString()} value={relativeTime(new Date(last_online_ts * 1000))} /> : null}
                </div>
            </div>
        </StatCard>
    );
}

/** A reward slot is claimed, still open (you're behind), or not yet reachable. */
type SlotState = "claimed" | "open" | "upcoming";

const CELL_BASE = "relative flex aspect-square items-center justify-center rounded-md border font-medium text-[10.5px] tabular-nums";

function claimedStyle() {
    return {
        background: `color-mix(in oklch, ${SIGNIN} 16%, transparent)`,
        borderColor: `color-mix(in oklch, ${SIGNIN} 30%, transparent)`,
        color: SIGNIN,
    };
}

function LegendSwatch({ state }: { state: SlotState }) {
    return <span aria-hidden className={cn("size-2.5 rounded-[3px] border", state === "open" && "border-border/60 border-dashed", state === "upcoming" && "border-transparent bg-muted/40", state === "claimed" && "border-transparent")} style={state === "claimed" ? claimedStyle() : undefined} />;
}

/**
 * Right card: the current month's sign-in progress (snapshot as of last sync).
 *
 * The cells are the month's reward SLOTS, claimed front-to-back - the same
 * thing the game itself draws. They are not dates, so there is no weekday
 * header and no "you missed the 4th": being behind shows up as open slots at
 * the end of the claimed run.
 */
export function SignInCalendarCard({ checkin, server }: ICardProps) {
    if (!checkin) return null;

    const { monthly_card_flags, can_check_in } = checkin;
    const { sync, year, month, daysInMonth, claimed, elapsed, behind } = monthState(checkin, server);

    const monthAnchor = new Date(year, month, 1);
    const monthLabel = monthAnchor.toLocaleDateString("en-US", { month: "long", year: "numeric" });
    const syncAbsolute = sync.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });

    // The next slot the player can take - the one the game highlights.
    const nextSlot = claimed < daysInMonth ? claimed + 1 : null;
    const cardDays = monthly_card_flags.filter((f) => f === 1).length;

    const summary = `${monthLabel}: ${claimed} of ${elapsed} days claimed, as of ${syncAbsolute}`;
    const slots = Array.from({ length: daysInMonth }, (_, d) => d + 1);

    return (
        <StatCard color={SIGNIN}>
            <div className={cn("flex h-full flex-col gap-4", CARD_PADDING)}>
                <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
                    <Kicker icon={CalendarCheck} label="Daily Sign-In" />
                    <span className="inline-flex items-center gap-1.5 font-medium text-[11px] text-muted-foreground" title={`Last synced ${syncAbsolute}`}>
                        <Clock aria-hidden className="size-3" />
                        Synced {relativeTime(sync)}
                    </span>
                </div>

                <div className="flex items-baseline justify-between">
                    <span className={KICKER_TEXT}>{monthLabel}</span>
                    <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                        <span className="font-semibold" style={{ color: SIGNIN }}>
                            {claimed}
                        </span>{" "}
                        / {elapsed} claimed
                    </span>
                </div>

                <div className="mx-auto w-full max-w-68">
                    {/* role=img + summary so screen readers get the gist, not 30+ cells.
                        State is conveyed by fill + check + border-style, not color alone. */}
                    <div aria-label={summary} className="grid grid-cols-7 gap-1" role="img">
                        {slots.map((slot) => {
                            const state: SlotState = slot <= claimed ? "claimed" : slot <= elapsed ? "open" : "upcoming";
                            const withCard = state === "claimed" && monthly_card_flags[slot - 1] === 1;
                            return (
                                <div
                                    className={cn(CELL_BASE, state === "open" && "border-border/60 border-dashed text-muted-foreground/55", state === "upcoming" && "border-transparent text-muted-foreground/30", state === "claimed" && "border-transparent")}
                                    key={slot}
                                    style={{
                                        ...(state === "claimed" && claimedStyle()),
                                        ...(slot === nextSlot && { boxShadow: `inset 0 0 0 1.5px color-mix(in oklch, ${SIGNIN} 55%, transparent)` }),
                                    }}
                                    title={`Day ${slot}${state === "claimed" ? " · Claimed" : state === "open" ? " · Unclaimed" : ""}${withCard ? " · Monthly card" : ""}${slot === nextSlot ? " · Next up" : ""}`}
                                >
                                    {slot}
                                    {state === "claimed" && <Check aria-hidden className="absolute top-0.5 right-0.5 size-2" style={{ color: SIGNIN }} />}
                                </div>
                            );
                        })}
                    </div>
                </div>

                {/* Legend - state is not conveyed by color alone. */}
                <div className="flex flex-wrap items-center justify-center gap-x-3 gap-y-1 font-mono text-[9.5px] text-muted-foreground/70 uppercase tracking-wide">
                    <span className="inline-flex items-center gap-1.5">
                        <LegendSwatch state="claimed" /> Claimed
                    </span>
                    <span className="inline-flex items-center gap-1.5">
                        <LegendSwatch state="open" /> Unclaimed
                    </span>
                    <span className="inline-flex items-center gap-1.5">
                        <LegendSwatch state="upcoming" /> Upcoming
                    </span>
                </div>

                <div className="mt-auto flex items-center justify-between gap-2 border-border/40 border-t pt-2.5">
                    <span className={KICKER_TEXT}>Days behind</span>
                    <span className="font-mono text-[11px] text-foreground tabular-nums" title={cardDays > 0 ? `${cardDays} of this month's ${claimed} claims came with the monthly card` : undefined}>
                        {behind === 0 ? (
                            <span className="font-semibold" style={{ color: SIGNIN }}>
                                All caught up
                            </span>
                        ) : (
                            <>
                                <span className="font-semibold" style={{ color: SIGNIN }}>
                                    {behind}
                                </span>{" "}
                                {behind === 1 ? "day" : "days"}
                            </>
                        )}
                        {can_check_in && (
                            <span className="ml-2 font-semibold" style={{ color: SIGNIN }}>
                                · claim ready
                            </span>
                        )}
                    </span>
                </div>
            </div>
        </StatCard>
    );
}
