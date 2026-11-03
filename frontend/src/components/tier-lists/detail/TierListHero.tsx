import { Link } from "@tanstack/react-router";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Badge } from "#/components/ui/badge";
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "#/components/ui/breadcrumb";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import type { ITierListDetail } from "#/lib/api/tier-lists";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { getAvatarById } from "#/lib/utils";
import { ExpandableDescription } from "../ExpandableDescription";
import { TierListActions } from "./TierListActions";
import type { messages } from "./TierListHero.messages";

interface ITierListHeroProps {
    detail: ITierListDetail;
}

export function TierListHero({ detail }: ITierListHeroProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const f = useFormatters();
    const isOfficial = detail.listType === "official";
    const flair = detail.flair;
    const stats = detail.stats;
    const author = detail.author;
    const updatedRel = f.relative(detail.updatedAt);

    return (
        <header className="border-border/60 border-b bg-linear-to-b from-card/40 to-transparent">
            <div className="page-gutter pt-5 pb-6 [--page-max:1200px] sm:pt-10 sm:pb-9">
                <Breadcrumb className="mb-3">
                    <BreadcrumbList className="text-xs">
                        <BreadcrumbItem>
                            <BreadcrumbLink render={<Link to="/tier-lists" search={{ type: "all", sort: "recent", q: "", flair: [] }} />}>{t("detail.hero.breadcrumb")}</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem>
                            <BreadcrumbPage className="max-w-60 truncate font-medium sm:max-w-none">{detail.title}</BreadcrumbPage>
                        </BreadcrumbItem>
                    </BreadcrumbList>
                </Breadcrumb>

                <div className="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
                    <div className="min-w-0 flex-1">
                        <h1 className="m-0 font-bold font-sans text-2xl text-foreground leading-tight tracking-tight sm:text-3xl md:text-4xl">{detail.title}</h1>

                        {detail.description && <ExpandableDescription text={detail.description} markdown className="mt-3 max-w-160 font-sans text-muted-foreground text-sm leading-relaxed sm:text-[15px]" />}

                        {/* The byline is its own band under a hairline, so the description's "Show more"
                            ends the description block instead of sitting on the byline's edge. */}
                        <div className="mt-5 flex flex-wrap items-center gap-x-4 gap-y-2 border-border/60 border-t pt-4 font-sans text-[12.5px] text-muted-foreground">
                            {author ? (
                                <Link to="/user/$id" params={{ id: author.uid }} className="group inline-flex min-w-0 items-center gap-2 text-foreground no-underline">
                                    <Avatar className="h-6 w-6 shrink-0 rounded-full border border-border bg-linear-to-br from-muted to-border font-sans font-semibold text-[10px] text-foreground">
                                        {author.avatarId && <AvatarImage src={getAvatarById(author.avatarId)} alt="" />}
                                        <AvatarFallback>{(author.nickname?.charAt(0) || "?").toUpperCase()}</AvatarFallback>
                                    </Avatar>
                                    <span className="min-w-0 truncate font-medium transition-colors group-hover:underline">{author.nickname?.trim() || t("detail.hero.authorFallback")}</span>
                                </Link>
                            ) : (
                                <span className="inline-flex items-center gap-2">
                                    <span className="inline-flex h-6 w-6 items-center justify-center rounded-full border border-border bg-muted font-sans font-semibold text-[10px] text-muted-foreground">?</span>
                                    <span className="font-medium text-foreground">{t("detail.hero.authorUnknown")}</span>
                                </span>
                            )}

                            {/* No dot separators: the row's gap separates the items, so a wrap never starts a line with an orphan "·". */}
                            <span className="inline-flex items-center gap-1.5">
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3.5 w-3.5 opacity-70" aria-hidden="true">
                                    <circle cx="12" cy="12" r="10" />
                                    <path d="M12 6v6l4 2" />
                                </svg>
                                <span>{t("detail.hero.updated", { when: updatedRel })}</span>
                            </span>

                            {stats && (
                                <>
                                    <Tooltip>
                                        <TooltipTrigger render={<span className="inline-flex items-center gap-1.5" />}>
                                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3.5 w-3.5 opacity-70" aria-hidden="true">
                                                <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
                                                <circle cx="12" cy="12" r="3" />
                                            </svg>
                                            <span className="font-mono font-semibold text-foreground tabular-nums">{f.compact(stats.viewCount)}</span>
                                            <span>{t("detail.hero.views")}</span>
                                        </TooltipTrigger>
                                        <TooltipPopup>{t("detail.hero.views.title", { count: f.number(stats.viewCount) })}</TooltipPopup>
                                    </Tooltip>
                                    <Tooltip>
                                        <TooltipTrigger render={<span className="inline-flex items-center gap-1.5" />}>
                                            <svg viewBox="0 0 24 24" fill="currentColor" className="h-3 w-3 opacity-70" aria-hidden="true">
                                                <path d="M12 21s-7-4.5-9.5-9C.7 8.7 2.5 5 6 5c2 0 3.5 1 4 2.5C10.5 6 12 5 14 5c3.5 0 5.3 3.7 3.5 7-2.5 4.5-9.5 9-9.5 9Z" />
                                            </svg>
                                            <span className="font-mono font-semibold text-foreground tabular-nums">{f.compact(stats.favoriteCount)}</span>
                                            <span>{t("detail.hero.favorites")}</span>
                                        </TooltipTrigger>
                                        <TooltipPopup>{t("detail.hero.favorites.title", { count: f.number(stats.favoriteCount) })}</TooltipPopup>
                                    </Tooltip>
                                </>
                            )}

                            {/* Kind, flair and trending sit in the meta row, under the title they describe: official in gold, the flair in its own colour, trending in the accent. */}
                            <span className="inline-flex flex-wrap items-center gap-1.5">
                                <Badge variant="outline" className={isOfficial ? "border-transparent bg-[oklch(0.92_0.13_85)] text-[oklch(0.32_0.12_75)]" : undefined}>
                                    {isOfficial ? t("detail.hero.official") : t("detail.hero.community")}
                                </Badge>
                                {flair && (
                                    // Free-form flair colours measured 2.46 to 2.94 as text on their own tint; the tint and border carry the colour, the text stays foreground.
                                    <Badge variant={flair.color ? "tint" : "outline"} style={flair.color ? ({ "--tone": flair.color } as React.CSSProperties) : undefined}>
                                        {flair.label}
                                    </Badge>
                                )}
                                {stats?.isTrending && (
                                    <Badge variant="tint" style={{ "--tone": "var(--primary)" } as React.CSSProperties}>
                                        {t("detail.hero.trending")}
                                    </Badge>
                                )}
                            </span>
                        </div>
                    </div>

                    <TierListActions detail={detail} />
                </div>
            </div>
        </header>
    );
}
