import type { UseQueryResult } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Newspaper, Search, SearchX } from "lucide-react";
import { Button } from "#/components/ui/button";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { FilterChip } from "#/components/ui/filter-chip";
import { Input } from "#/components/ui/input";
import { Skeleton } from "#/components/ui/skeleton";
import { boardDate, type IGroupCount } from "#/lib/announcements";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { AnnouncementList as Board } from "#/types/generated/AnnouncementList";
import type { AnnouncementSummary } from "#/types/generated/AnnouncementSummary";
import type { messages } from "../Announcements.messages";
import { AnnouncementsError, RetryButton, retryOf } from "./AnnouncementsError";
import { useGroupLabel } from "./group-label";

interface IAnnouncementListProps {
    query: UseQueryResult<Board>;
    server: string | undefined;
    all: readonly AnnouncementSummary[];
    shown: readonly AnnouncementSummary[];
    groups: readonly IGroupCount[];
    group: string | null;
    onGroup: (group: string | null) => void;
    text: string;
    onText: (text: string) => void;
    unreadIds: ReadonlySet<string>;
    openId: string | undefined;
    checkedAt: number | null;
    onMarkAllRead: () => void;
}

export function AnnouncementList({ query, server, all, shown, groups, group, onGroup, text, onText, unreadIds, openId, checkedAt, onMarkAllRead }: IAnnouncementListProps) {
    const t: TypedT<typeof messages> = useT("announcements");
    const f = useFormatters();
    const groupLabel = useGroupLabel();

    if (query.isPending) return <ListSkeleton />;
    if (query.isError) return <AnnouncementsError {...retryOf(query)} />;

    const unreadCount = all.filter((a) => unreadIds.has(a.id)).length;

    return (
        <div className="flex min-w-0 flex-col gap-3">
            <div className="relative">
                <Search className="pointer-events-none absolute start-3 top-1/2 z-10 size-4 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
                <Input type="search" value={text} onChange={(e) => onText(e.target.value)} placeholder={t("search.placeholder")} aria-label={t("search.label")} className="ps-6 max-lg:[&_input]:h-11 max-lg:[&_input]:text-base max-lg:[&_input]:leading-11" />
            </div>

            {groups.length > 1 ? (
                // biome-ignore lint/a11y/useSemanticElements: a chip row, not a form field group
                <div role="group" aria-label={t("filter.label")} className="flex flex-wrap gap-2">
                    <FilterChip label={t("filter.all")} count={all.length} active={group === null} onSelect={() => onGroup(null)} className="max-lg:h-11! max-lg:px-4" />
                    {groups.map((g) => (
                        <FilterChip key={g.group} label={groupLabel(g.group)} count={g.count} active={group === g.group} onSelect={() => onGroup(group === g.group ? null : g.group)} className="max-lg:h-11! max-lg:px-4" />
                    ))}
                </div>
            ) : null}

            <div className="flex min-h-8 flex-wrap items-center justify-between gap-x-3 gap-y-1 text-[12.5px] text-muted-foreground">
                <span>
                    {t("list.count", { count: shown.length })}
                    {checkedAt ? <> · {t("list.checked", { time: f.relative(new Date(checkedAt * 1000).toISOString()) })}</> : null}
                </span>
                {unreadCount > 0 ? (
                    <Button variant="ghost" size="sm" onClick={onMarkAllRead} className="-me-2 max-lg:h-11!">
                        {t("list.markAllRead")}
                    </Button>
                ) : null}
            </div>

            {all.length === 0 ? (
                <Empty className="rounded-xl border border-dashed py-10">
                    <EmptyHeader>
                        <EmptyTitle>{t("empty.title")}</EmptyTitle>
                        <EmptyDescription>{t("empty.body")}</EmptyDescription>
                    </EmptyHeader>
                    <EmptyContent>
                        <RetryButton {...retryOf(query)} />
                    </EmptyContent>
                </Empty>
            ) : shown.length === 0 ? (
                <Empty className="rounded-xl border border-dashed py-10">
                    <EmptyHeader>
                        <EmptyMedia variant="icon">
                            <SearchX aria-hidden="true" />
                        </EmptyMedia>
                        <EmptyTitle>{t("empty.filtered.title")}</EmptyTitle>
                        <EmptyDescription>{t("empty.filtered.body")}</EmptyDescription>
                    </EmptyHeader>
                    <EmptyContent>
                        <Button
                            variant="outline"
                            onClick={() => {
                                onText("");
                                onGroup(null);
                            }}
                            className="max-lg:h-11!"
                        >
                            {t("empty.filtered.clear")}
                        </Button>
                    </EmptyContent>
                </Empty>
            ) : (
                <ul className="flex min-w-0 flex-col gap-1.5">
                    {shown.map((a) => {
                        const date = boardDate(a.published_on);
                        const unread = unreadIds.has(a.id);
                        const active = a.id === openId;
                        return (
                            <li key={a.id} className="min-w-0">
                                <Link
                                    to="/announcements"
                                    search={{ server, id: a.id }}
                                    aria-current={active ? "page" : undefined}
                                    className={cn(
                                        "group flex min-h-16 min-w-0 items-center gap-3 rounded-xl border border-transparent p-2.5 text-foreground no-underline outline-none transition-colors hover:bg-accent/60 focus-visible:ring-2 focus-visible:ring-ring",
                                        active && "border-primary/40 bg-primary/8 hover:bg-primary/10",
                                    )}
                                >
                                    <span className="relative block aspect-video w-24 shrink-0 overflow-hidden rounded-lg bg-muted sm:w-28">
                                        {a.banner_url ? <img src={a.banner_url} alt="" loading="lazy" decoding="async" className="size-full object-cover" /> : <Newspaper className="absolute inset-0 m-auto size-5 text-muted-foreground/60" aria-hidden="true" />}
                                    </span>
                                    <span className="flex min-w-0 flex-1 flex-col gap-1">
                                        <span className={cn("wrap-break-word line-clamp-2 text-[14px] leading-snug", unread ? "font-semibold" : "font-medium")}>{a.title}</span>
                                        <span className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-0.5 text-[12px] text-muted-foreground">
                                            <span>{groupLabel(a.group)}</span>
                                            {date ? (
                                                <>
                                                    <span aria-hidden="true">·</span>
                                                    <time dateTime={a.published_on ?? undefined}>{f.calendarDay(date, { month: "short", day: "numeric" })}</time>
                                                </>
                                            ) : null}
                                        </span>
                                    </span>
                                    {unread ? (
                                        <span className="shrink-0">
                                            <span aria-hidden="true" className="block size-2 rounded-full bg-primary" />
                                            <span className="sr-only">{t("list.unread")}</span>
                                        </span>
                                    ) : null}
                                </Link>
                            </li>
                        );
                    })}
                </ul>
            )}
        </div>
    );
}

function ListSkeleton() {
    return (
        <div className="flex flex-col gap-3" aria-busy="true">
            <Skeleton className="h-11 w-full rounded-lg lg:h-9" />
            <div className="flex gap-2">
                <Skeleton className="h-8 w-16 rounded-full" />
                <Skeleton className="h-8 w-20 rounded-full" />
                <Skeleton className="h-8 w-20 rounded-full" />
            </div>
            {[0, 1, 2, 3, 4].map((i) => (
                <div key={i} className="flex items-center gap-3 p-2.5">
                    <Skeleton className="aspect-video w-24 shrink-0 rounded-lg sm:w-28" />
                    <div className="flex flex-1 flex-col gap-2">
                        <Skeleton className="h-4 w-4/5" />
                        <Skeleton className="h-3 w-1/3" />
                    </div>
                </div>
            ))}
        </div>
    );
}
