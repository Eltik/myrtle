import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ArrowLeft, ExternalLink, FileQuestion, Newspaper } from "lucide-react";
import { useEffect, useRef } from "react";
import { Button } from "#/components/ui/button";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { Skeleton } from "#/components/ui/skeleton";
import { boardDate, markSeen } from "#/lib/announcements";
import { announcementQueryOptions } from "#/lib/api/announcements";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "../Announcements.messages";
import { AnnouncementsError } from "./AnnouncementsError";
import { useGroupLabel } from "./group-label";
import styles from "./NoticeBody.module.css";

/** Two titles are the same headline when they differ only in case and spacing. */
function sameTitle(a: string, b: string): boolean {
    const norm = (s: string) => s.replace(/\s+/g, " ").trim().toLowerCase();
    return norm(a) === norm(b);
}

function BackLink({ server }: { server: string }) {
    const t: TypedT<typeof messages> = useT("announcements");
    return (
        <Link to="/announcements" search={{ server }} className="-ms-2 inline-flex min-h-11 items-center gap-1.5 rounded-md px-2 font-medium text-[14px] text-muted-foreground no-underline outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring lg:hidden">
            <ArrowLeft className="size-4" aria-hidden="true" />
            {t("detail.back")}
        </Link>
    );
}

export function AnnouncementDetail({ server, id }: { server: string; id: string }) {
    const t: TypedT<typeof messages> = useT("announcements");
    const f = useFormatters();
    const groupLabel = useGroupLabel();
    const query = useQuery(announcementQueryOptions(id, server));
    const heading = useRef<HTMLHeadingElement>(null);
    const notice = query.data;

    // Opening a notice is what reads it; focus moves to it so a screen reader starts there.
    useEffect(() => {
        if (!notice) return;
        markSeen(server, [notice.id]);
        heading.current?.focus({ preventScroll: true });
    }, [notice, server]);

    if (query.isPending) {
        return (
            <div aria-busy="true" className="flex flex-col gap-4">
                <BackLink server={server} />
                <Skeleton className="aspect-[16/7] w-full rounded-xl" />
                <Skeleton className="h-7 w-3/4" />
                <Skeleton className="h-4 w-1/3" />
                <div className="flex flex-col gap-2 pt-2">
                    <Skeleton className="h-4 w-full" />
                    <Skeleton className="h-4 w-11/12" />
                    <Skeleton className="h-4 w-4/5" />
                </div>
            </div>
        );
    }

    if (query.isError) {
        return (
            <div className="flex flex-col gap-3">
                <BackLink server={server} />
                <AnnouncementsError onRetry={() => void query.refetch()} retrying={query.isFetching} />
            </div>
        );
    }

    if (!notice) {
        return (
            <div className="flex flex-col gap-3">
                <BackLink server={server} />
                <Empty className="rounded-xl border border-dashed py-10">
                    <EmptyHeader>
                        <EmptyMedia variant="icon">
                            <FileQuestion aria-hidden="true" />
                        </EmptyMedia>
                        <EmptyTitle>{t("detail.gone.title")}</EmptyTitle>
                        <EmptyDescription>{t("detail.gone.body")}</EmptyDescription>
                    </EmptyHeader>
                    <EmptyContent className="lg:hidden">
                        <Button variant="outline" className="h-11" render={<Link to="/announcements" search={{ server }} />}>
                            {t("detail.back")}
                        </Button>
                    </EmptyContent>
                </Empty>
            </div>
        );
    }

    const date = boardDate(notice.published_on);
    const subtitle = notice.headline && !sameTitle(notice.headline, notice.title) ? notice.headline : null;

    return (
        <article className="flex min-w-0 flex-col gap-4">
            <BackLink server={server} />
            {notice.banner_url ? <img src={notice.banner_url} alt={t("detail.bannerAlt", { title: notice.title })} className="block h-auto w-full max-w-full rounded-xl border border-border bg-muted" /> : null}
            <header className="flex min-w-0 flex-col gap-1.5">
                <h2 ref={heading} tabIndex={-1} className="wrap-break-word m-0 font-bold text-[22px] text-foreground leading-tight tracking-tight outline-none sm:text-[26px]">
                    {notice.title}
                </h2>
                {subtitle ? <p className="wrap-break-word m-0 text-[14px] text-muted-foreground">{subtitle}</p> : null}
                <p className="m-0 flex flex-wrap items-center gap-x-2 text-[13px] text-muted-foreground">
                    <span>{groupLabel(notice.group)}</span>
                    {date ? (
                        <>
                            <span aria-hidden="true">·</span>
                            <time dateTime={notice.published_on ?? undefined}>{f.calendarDay(date, { dateStyle: "long" })}</time>
                        </>
                    ) : null}
                </p>
            </header>
            {notice.body_html ? (
                <div
                    className={styles.body}
                    // biome-ignore lint/security/noDangerouslySetInnerHtml: sanitized server-side by ammonia against an allowlist (backend/src/core/announcements/page.rs)
                    dangerouslySetInnerHTML={{ __html: notice.body_html }}
                />
            ) : (
                <p className="m-0 text-[14px] text-muted-foreground">{t("detail.imageOnly")}</p>
            )}
            <div className="border-border border-t pt-4">
                <Button
                    variant="outline"
                    className="max-lg:h-11!"
                    render={
                        <a href={notice.notice_url} target="_blank" rel="noopener noreferrer">
                            {t("detail.official")}
                            <ExternalLink className="size-4" aria-hidden="true" />
                        </a>
                    }
                />
            </div>
        </article>
    );
}

/** The reading pane before a notice is picked (wide screens only; phones show the list instead). */
export function DetailPlaceholder({ loading }: { loading: boolean }) {
    const t: TypedT<typeof messages> = useT("announcements");
    if (loading) return <Skeleton className="aspect-[16/9] w-full rounded-xl" />;
    return (
        <Empty className="hidden rounded-xl border border-dashed py-16 lg:flex">
            <EmptyHeader>
                <EmptyMedia variant="icon">
                    <Newspaper aria-hidden="true" />
                </EmptyMedia>
                <EmptyDescription>{t("detail.pick")}</EmptyDescription>
            </EmptyHeader>
        </Empty>
    );
}
