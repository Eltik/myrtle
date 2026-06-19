import { CalendarCheck, CalendarDays, Check, Clock } from "lucide-react";
import { cn } from "#/lib/utils";
import type { IUserCheckin } from "#/types/user";
import { PALETTE } from "../palette";
import { CARD_PADDING, KICKER_TEXT, Kicker, MetricRow, StatCard, Tile } from "../primitives";

interface ICardProps {
    checkin: IUserCheckin | null | undefined;
}

const SIGNIN = PALETTE.signin;
const DAY_MS = 86_400_000;
const WEEKDAYS = [
    { k: "sun", l: "S" },
    { k: "mon", l: "M" },
    { k: "tue", l: "T" },
    { k: "wed", l: "W" },
    { k: "thu", l: "T" },
    { k: "fri", l: "F" },
    { k: "sat", l: "S" },
] as const;

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
export function SignInOverviewCard({ checkin }: ICardProps) {
    if (!checkin) return null;

    const { history, cumulative_signin, register_ts, last_online_ts } = checkin;

    // Floor (not round) so e.g. 915/919 reads 99%, never a misleading 100%.
    const ageDays = register_ts ? Math.max(0, Math.floor((Date.now() - register_ts * 1000) / DAY_MS)) : null;
    const rate = ageDays && ageDays > 0 ? Math.min(100, Math.floor((cumulative_signin / ageDays) * 100)) : null;
    const missed = ageDays !== null ? Math.max(0, ageDays - cumulative_signin) : null;
    const claimedThisMonth = history.filter((d) => d === 1).length;

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
                    <StatRow label="This month" title="Days claimed this month" value={`${claimedThisMonth} / ${history.length}`} />
                    {missed !== null && <StatRow label="Days missed" title="Days since joining without a sign-in" value={missed.toLocaleString()} />}
                    {last_online_ts ? <StatRow label="Last online" title={new Date(last_online_ts * 1000).toLocaleString()} value={relativeTime(new Date(last_online_ts * 1000))} /> : null}
                </div>
            </div>
        </StatCard>
    );
}

type DayState = "claimed" | "missed" | "upcoming";

const CELL_BASE = "relative flex aspect-square items-center justify-center rounded-md border font-medium text-[10.5px] tabular-nums";

function claimedStyle() {
    return {
        background: `color-mix(in oklch, ${SIGNIN} 16%, transparent)`,
        borderColor: `color-mix(in oklch, ${SIGNIN} 30%, transparent)`,
        color: SIGNIN,
    };
}

function LegendSwatch({ state }: { state: DayState }) {
    return <span aria-hidden className={cn("size-2.5 rounded-[3px] border", state === "missed" && "border-border/60 border-dashed", state === "upcoming" && "border-transparent bg-muted/40", state === "claimed" && "border-transparent")} style={state === "claimed" ? claimedStyle() : undefined} />;
}

/** Right card: the current month's sign-in calendar (snapshot as of last sync). */
export function SignInCalendarCard({ checkin }: ICardProps) {
    if (!checkin) return null;

    const { history, can_check_in, updated_at } = checkin;

    const sync = new Date(updated_at);
    const year = sync.getFullYear();
    const month = sync.getMonth();
    const monthLabel = sync.toLocaleDateString("en-US", { month: "long", year: "numeric" });
    const monthShort = sync.toLocaleDateString("en-US", { month: "short" });
    const syncAbsolute = sync.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
    const firstWeekday = new Date(year, month, 1).getDay();
    const daysInMonth = new Date(year, month + 1, 0).getDate();

    const elapsed = history.length;
    const claimedThisMonth = history.filter((d) => d === 1).length;

    // Current streak: trailing claimed days (the final day may be today-still-
    // pending, so a single trailing 0 doesn't break it).
    let streak = 0;
    let i = history.length - 1;
    if (i >= 0 && history[i] === 0) i--;
    for (; i >= 0 && history[i] === 1; i--) streak++;
    // The game only exposes the CURRENT month's calendar, so a streak that fills
    // the whole month certainly began earlier than we can see -> show "N+".
    const extendsBeyond = elapsed > 0 && claimedThisMonth === elapsed;

    const summary = `${monthLabel}: ${claimedThisMonth} of ${elapsed} days claimed, as of ${syncAbsolute}`;
    const leadingBlanks = Array.from({ length: firstWeekday }, (_, b) => `blank-${b}`);
    const days = Array.from({ length: daysInMonth }, (_, d) => d + 1);

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
                            {claimedThisMonth}
                        </span>{" "}
                        / {elapsed} claimed
                    </span>
                </div>

                <div className="mx-auto w-full max-w-68">
                    <div aria-hidden className="mb-1 grid grid-cols-7 gap-1">
                        {WEEKDAYS.map((w) => (
                            <span className="text-center font-mono font-semibold text-[9px] text-muted-foreground/50 uppercase" key={w.k}>
                                {w.l}
                            </span>
                        ))}
                    </div>
                    {/* role=img + summary so screen readers get the gist, not 30+ cells.
                        State is conveyed by fill + check + border-style, not color alone. */}
                    <div aria-label={summary} className="grid grid-cols-7 gap-1" role="img">
                        {leadingBlanks.map((id) => (
                            <span key={id} />
                        ))}
                        {days.map((day) => {
                            const state: DayState = day > elapsed ? "upcoming" : history[day - 1] === 1 ? "claimed" : "missed";
                            const isSyncDay = day === sync.getDate();
                            return (
                                <div
                                    className={cn(CELL_BASE, state === "missed" && "border-border/60 border-dashed text-muted-foreground/55", state === "upcoming" && "border-transparent text-muted-foreground/30", state === "claimed" && "border-transparent")}
                                    key={day}
                                    style={{
                                        ...(state === "claimed" && claimedStyle()),
                                        ...(isSyncDay && { boxShadow: `inset 0 0 0 1.5px color-mix(in oklch, ${SIGNIN} 55%, transparent)` }),
                                    }}
                                    title={`${monthShort} ${day}${state === "claimed" ? " · Claimed" : state === "missed" ? " · Missed" : ""}${isSyncDay ? " · Last synced" : ""}`}
                                >
                                    {day}
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
                        <LegendSwatch state="missed" /> Missed
                    </span>
                    <span className="inline-flex items-center gap-1.5">
                        <LegendSwatch state="upcoming" /> Upcoming
                    </span>
                </div>

                <div className="mt-auto flex items-center justify-between gap-2 border-border/40 border-t pt-2.5">
                    <span className={KICKER_TEXT}>Current streak</span>
                    <span className="font-mono text-[11px] text-foreground tabular-nums" title={extendsBeyond ? "Only the current month's daily record is available - your streak likely extends further back." : undefined}>
                        <span className="font-semibold" style={{ color: SIGNIN }}>
                            {streak}
                            {extendsBeyond ? "+" : ""}
                        </span>{" "}
                        {streak === 1 && !extendsBeyond ? "day" : "days"}
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
