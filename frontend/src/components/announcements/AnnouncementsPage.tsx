import { useNavigate, useSearch } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { useGamedataServerName } from "#/components/GamedataServerSwitcher";
import { PageHeader } from "#/components/ui/page-header";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { useAnnouncementBoard } from "#/hooks/use-announcements";
import { filterAnnouncements, groupsOf, markSeen } from "#/lib/announcements";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./Announcements.messages";
import { AnnouncementDetail, DetailPlaceholder } from "./impl/AnnouncementDetail";
import { AnnouncementList } from "./impl/AnnouncementList";
import { AnnouncementsError } from "./impl/AnnouncementsError";

/**
 * The in-game notice board. Two panes from `lg` up (list beside the open notice), one below it:
 * the list until a notice is opened, then the notice with a link back. `server` and `id` live in
 * the URL, so a notice deep-links and the browser's back button closes it on a phone.
 */
export function AnnouncementsPage() {
    const t: TypedT<typeof messages> = useT("announcements");
    const serverName = useGamedataServerName();
    const search = useSearch({ from: "/announcements" });
    const navigate = useNavigate({ from: "/announcements" });
    const { query, board, unread } = useAnnouncementBoard(search.server);
    const [group, setGroup] = useState<string | null>(null);
    const [text, setText] = useState("");

    const all = board?.announcements ?? [];
    const groups = useMemo(() => groupsOf(all), [all]);
    const shown = useMemo(() => filterAnnouncements(all, group, text), [all, group, text]);
    const unreadIds = useMemo(() => new Set(unread.map((a) => a.id)), [unread]);
    const openId = search.id;
    const server = board?.server;

    const pickServer = (value: string) => {
        setGroup(null);
        void navigate({ search: { server: value } });
    };

    return (
        <main className="page-shell [--page-max:1200px]">
            <PageHeader
                title={t("page.title")}
                description={t("page.description")}
                actions={
                    board && board.servers.length > 1 ? (
                        <Tabs value={board.server} onValueChange={(v) => pickServer(String(v))}>
                            <TabsList aria-label={t("server.label")}>
                                {board.servers.map((s) => (
                                    <TabsTab key={s} value={s} className="max-lg:h-11! max-lg:px-4">
                                        {serverName(s)}
                                    </TabsTab>
                                ))}
                            </TabsList>
                        </Tabs>
                    ) : null
                }
            />

            <div className="mt-6 grid min-w-0 gap-6 lg:grid-cols-[minmax(0,22rem)_minmax(0,1fr)] lg:items-start xl:grid-cols-[minmax(0,24rem)_minmax(0,1fr)]">
                <section aria-label={t("list.label")} className={cn("min-w-0 lg:sticky lg:top-[calc(var(--site-header-height)+1rem)] lg:max-h-[calc(100dvh-var(--site-header-height)-2rem)] lg:overflow-y-auto lg:pe-1", openId && "max-lg:hidden")}>
                    <AnnouncementList
                        query={query}
                        server={server}
                        all={all}
                        shown={shown}
                        groups={groups}
                        group={group}
                        onGroup={setGroup}
                        text={text}
                        onText={setText}
                        unreadIds={unreadIds}
                        openId={openId}
                        checkedAt={board?.checked_at ?? null}
                        onMarkAllRead={() =>
                            server &&
                            markSeen(
                                server,
                                all.map((a) => a.id),
                            )
                        }
                    />
                </section>

                <section className={cn("min-w-0", !openId && "max-lg:hidden")}>
                    {openId && server ? (
                        <AnnouncementDetail key={`${server}:${openId}`} server={server} id={openId} />
                    ) : query.isError ? (
                        // The list column shows this on wide screens; a phone with a notice open has only this column.
                        <div className="lg:hidden">
                            <AnnouncementsError onRetry={() => void query.refetch()} retrying={query.isFetching} />
                        </div>
                    ) : (
                        <DetailPlaceholder loading={query.isPending} />
                    )}
                </section>
            </div>
        </main>
    );
}
