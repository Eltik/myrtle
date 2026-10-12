import { Link } from "@tanstack/react-router";
import type React from "react";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { ServerTag } from "#/components/user/leaderboard/impl/components/ServerTag";
import { playerIdentity } from "#/components/user/leaderboard/impl/identity";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { MusicListenerEntry } from "#/types/generated/MusicListenerEntry";
import type { MusicListenerSort } from "#/types/generated/MusicListenerSort";
import type { MusicStatWindow } from "#/types/generated/MusicStatWindow";
import { listeningTime } from "../impl/format";
import type { messages } from "../MusicGallery.messages";

type MusicT = TypedT<typeof messages>;

const STAT_WINDOWS: readonly MusicStatWindow[] = ["week", "month", "all"];
const LISTENER_SORTS: readonly MusicListenerSort[] = ["listens", "time"];

/**
 * A row of pressed-state buttons, the music page's own segmented control (the
 * Discovery | Library switch), sized for a thumb on a phone.
 */
function Segmented<T extends string>({ value, options, label, onChange, render, size = "md" }: { value: T; options: readonly T[]; label: string; onChange: (next: T) => void; render: (v: T) => string; size?: "sm" | "md" }): React.ReactElement {
    return (
        <fieldset className="flex w-fit gap-0.5 rounded-[9px] border border-border bg-secondary/45 p-0.75" aria-label={label}>
            {options.map((v) => (
                <button
                    key={v}
                    type="button"
                    onClick={() => onChange(v)}
                    aria-pressed={value === v}
                    className={cn("cursor-pointer whitespace-nowrap rounded-md font-sans font-semibold transition-colors max-sm:min-h-11", size === "sm" ? "h-7 px-2.5 text-[12px]" : "h-8 px-3 text-[12.5px]", value === v ? "bg-primary text-primary-foreground shadow-sm/5" : "text-muted-foreground hover:text-foreground")}
                >
                    {render(v)}
                </button>
            ))}
        </fieldset>
    );
}

/** The span a ranking counts over: this week, this month or all time. */
export function StatWindowTabs({ value, onChange, size }: { value: MusicStatWindow; onChange: (next: MusicStatWindow) => void; size?: "sm" | "md" }): React.ReactElement {
    const t: MusicT = useT("gallery");
    return <Segmented size={size} value={value} options={STAT_WINDOWS} label={t("music.charts.windowAria")} onChange={onChange} render={(w) => t(`music.charts.window.${w}`)} />;
}

/** What ranks the listeners: plays or time heard. */
export function ListenerSortTabs({ value, onChange }: { value: MusicListenerSort; onChange: (next: MusicListenerSort) => void }): React.ReactElement {
    const t: MusicT = useT("gallery");
    return <Segmented size="sm" value={value} options={LISTENER_SORTS} label={t("music.charts.sortAria")} onChange={onChange} render={(s) => t(`music.charts.sort.${s}`)} />;
}

/** Seconds heard, in the stats tile's short units. */
function useListeningText(): (seconds: number) => string {
    const t: MusicT = useT("gallery");
    return (seconds) => {
        const time = listeningTime(seconds);
        return time.unit === "seconds" ? t("music.stats.seconds", { seconds: time.seconds }) : time.unit === "minutes" ? t("music.stats.minutes", { minutes: time.minutes }) : t("music.stats.hours", { hours: time.hours, minutes: time.minutes });
    };
}

/**
 * Ranked listeners, each a link to the profile, with the avatar, name and
 * server the user leaderboard shows. The sorted value leads; the other one
 * follows it, smaller.
 */
export function ListenerRows({ entries, sort }: { entries: readonly MusicListenerEntry[]; sort: MusicListenerSort }): React.ReactElement {
    return (
        <ol className="m-0 flex list-none flex-col p-0">
            {entries.map((entry) => (
                <ListenerRow key={`${entry.server}:${entry.uid}`} entry={entry} sort={sort} />
            ))}
        </ol>
    );
}

function ListenerRow({ entry, sort }: { entry: MusicListenerEntry; sort: MusicListenerSort }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const f = useFormatters();
    const time = useListeningText();
    const { nickname, initials, avatarSrc } = playerIdentity({ uid: entry.uid, nickname: entry.nickname, avatar_id: entry.avatarId });
    const listens = t("music.charts.listens", { count: entry.listens });
    const heard = time(entry.listenedSeconds);
    const [lead, rest] = sort === "time" ? [heard, listens] : [f.number(entry.listens), heard];
    return (
        <li>
            <Link to="/user/$id" params={{ id: entry.uid }} aria-label={t("music.charts.profile", { rank: entry.rank, name: nickname, listens, time: heard })} className="group flex min-h-12 items-center gap-2.5 rounded-md px-1.5 py-1 no-underline transition-colors hover:bg-primary/6">
                <span className="w-6 shrink-0 text-right font-mono text-[11px] text-muted-foreground tabular-nums">{entry.rank}</span>
                <Avatar className="size-8 shrink-0 rounded-lg">
                    <AvatarImage src={avatarSrc} alt="" />
                    <AvatarFallback className="rounded-lg text-[10px]">{initials}</AvatarFallback>
                </Avatar>
                <span className="flex min-w-0 flex-1 flex-col gap-0.5 leading-tight">
                    <span className="truncate font-sans font-semibold text-[13px] text-foreground transition-colors group-hover:text-primary" title={nickname}>
                        {nickname}
                    </span>
                    <span className="flex min-w-0 items-center gap-1.5">
                        <ServerTag server={entry.server} />
                        {entry.level != null ? <span className="truncate font-mono text-[10.5px] text-muted-foreground">{t("music.charts.level", { level: entry.level })}</span> : null}
                    </span>
                </span>
                <span className="flex shrink-0 flex-col items-end gap-0.5 leading-tight">
                    <span className="font-mono font-semibold text-[13px] text-primary tabular-nums">{lead}</span>
                    <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{rest}</span>
                </span>
            </Link>
        </li>
    );
}

interface IRankingQuery<T> {
    isPending: boolean;
    isError: boolean;
    isFetching: boolean;
    data: T | null | undefined;
}

/**
 * A ranking's rows, or the skeleton, the failure or the quiet note in their
 * place: `null` data is a backend that predates the route. Busy while the
 * query fetches.
 */
export function RankingBody<T>({ query, isEmpty, emptyText, skeletonRows, children }: { query: IRankingQuery<T>; isEmpty: (data: T) => boolean; emptyText: string; skeletonRows?: number; children: (data: T) => React.ReactNode }): React.ReactElement {
    const t: MusicT = useT("gallery");
    let body: React.ReactNode;
    if (query.isPending) body = <RowsSkeleton rows={skeletonRows} />;
    else if (query.isError) body = <ChartNote tone="error">{t("music.charts.failed")}</ChartNote>;
    else if (!query.data) body = <ChartNote>{t("music.charts.unavailable")}</ChartNote>;
    else if (isEmpty(query.data)) body = <ChartNote>{emptyText}</ChartNote>;
    else body = children(query.data);
    return <div aria-busy={query.isFetching}>{body}</div>;
}

function RowsSkeleton({ rows = 8 }: { rows?: number }): React.ReactElement {
    return (
        <div className="flex flex-col gap-1.5" aria-hidden="true">
            {Array.from({ length: rows }, (_, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: a placeholder has no identity beyond its position
                <Skeleton key={i} className="h-12 w-full rounded-md" />
            ))}
        </div>
    );
}

/** The quiet line a ranking shows when it has nothing, or could not load. */
function ChartNote({ children, tone = "muted" }: { children: React.ReactNode; tone?: "muted" | "error" }): React.ReactElement {
    return <p className={cn("m-0 rounded-lg border border-border border-dashed px-4 py-8 text-center font-sans text-[12.5px]", tone === "error" ? "text-destructive-foreground" : "text-muted-foreground")}>{children}</p>;
}
