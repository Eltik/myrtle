import { Link } from "@tanstack/react-router";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "#/components/ui/breadcrumb";
import { Kicker } from "#/components/ui/kicker";
import type { ITierListDetail } from "#/lib/api/tier-lists";
import { formatNumber, formatNumberCompact, formatRelative, getAvatarById } from "#/lib/utils";
import { TierListActions } from "./TierListActions";

interface ITierListHeroProps {
    detail: ITierListDetail;
}

export function TierListHero({ detail }: ITierListHeroProps) {
    const isOfficial = detail.listType === "official";
    const flair = detail.flair;
    const stats = detail.stats;
    const author = detail.author;
    const updatedRel = formatRelative(detail.updatedAt);

    return (
        <header className="border-b border-border/60 bg-linear-to-b from-card/40 to-transparent">
            <div className="mx-auto w-[min(1080px,calc(100%-1.5rem))] pt-5 pb-6 sm:w-[min(1080px,calc(100%-2rem))] sm:pt-10 sm:pb-9">
                <Breadcrumb className="mb-3">
                    <BreadcrumbList className="text-xs">
                        <BreadcrumbItem>
                            <BreadcrumbLink render={<Link to="/tier-lists" search={{ type: "all", sort: "trending", q: "", flair: [] }} />}>Tier Lists</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem>
                            <BreadcrumbPage className="max-w-60 truncate font-medium sm:max-w-none">{detail.title}</BreadcrumbPage>
                        </BreadcrumbItem>
                    </BreadcrumbList>
                </Breadcrumb>

                <div className="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
                    <div className="min-w-0 flex-1">
                        <div className="mb-2 flex flex-wrap items-center gap-1.5">
                            {isOfficial ? (
                                <span className="inline-flex items-center gap-1 rounded-full border border-[oklch(0.92_0.13_85)] bg-[oklch(0.92_0.13_85)] px-2 py-0.5 font-mono text-[10.5px] font-bold uppercase leading-none tracking-wider text-[oklch(0.32_0.12_75)]">
                                    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" className="h-2.5 w-2.5">
                                        <path d="M12 2 9.6 4.4 6.3 4l-.6 3.3L2.5 9 4 12l-1.5 3 3.2 1.7.6 3.3 3.3-.4L12 22l2.4-2.4 3.3.4.6-3.3L21.5 15 20 12l1.5-3-3.2-1.7-.6-3.3L14.4 4.4Zm-1.2 13.4-3.4-3.4 1.4-1.4 2 2 4.4-4.4 1.4 1.4Z" />
                                    </svg>
                                    Official
                                </span>
                            ) : (
                                <span className="inline-flex items-center gap-1 rounded-full border border-border bg-muted px-2 py-0.5 font-mono text-[10.5px] font-medium uppercase leading-none tracking-wider text-muted-foreground">Community</span>
                            )}

                            {flair && (
                                <span
                                    className="inline-flex items-center gap-1 rounded-full px-2 py-0.5 font-mono text-[10.5px] font-bold uppercase leading-none tracking-wider"
                                    style={
                                        flair.color
                                            ? {
                                                  background: `color-mix(in srgb, ${flair.color} 18%, transparent)`,
                                                  color: flair.color,
                                                  border: `1px solid color-mix(in srgb, ${flair.color} 50%, transparent)`,
                                              }
                                            : {
                                                  background: "var(--muted)",
                                                  color: "var(--muted-foreground)",
                                                  border: "1px solid var(--border)",
                                              }
                                    }
                                >
                                    <span className="h-1.5 w-1.5 rounded-full" style={{ background: flair.color ?? "currentColor" }} aria-hidden="true" />
                                    {flair.label}
                                </span>
                            )}

                            {stats?.isTrending && (
                                <span className="inline-flex items-center gap-1 rounded-full border border-[color-mix(in_srgb,var(--primary)_36%,transparent)] bg-[color-mix(in_srgb,var(--primary)_10%,transparent)] px-2 py-0.5 font-mono text-[10.5px] font-bold uppercase leading-none tracking-wider text-primary">
                                    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" className="h-2.5 w-2.5">
                                        <path d="M12 2c.6 4 4.7 5.4 4.7 9.5 0 2.6-2 4.5-4.7 4.5s-4.7-1.9-4.7-4.5C7.3 9.4 9.6 8 9 4c2.4 1.4 3 4 3 4Z" />
                                    </svg>
                                    Trending
                                </span>
                            )}
                        </div>

                        <Kicker>Tier List</Kicker>
                        <h1 className="m-0 font-sans text-2xl font-bold leading-tight tracking-tight text-foreground sm:text-3xl md:text-4xl">{detail.title}</h1>

                        {detail.description && <p className="mt-3 max-w-160 font-sans text-sm leading-relaxed text-muted-foreground sm:text-[15px]">{detail.description}</p>}

                        <div className="mt-4 flex flex-wrap items-center gap-x-4 gap-y-2 font-sans text-[12.5px] text-muted-foreground">
                            {author ? (
                                <Link to="/user/$id" params={{ id: author.uid }} className="group inline-flex min-w-0 items-center gap-2 text-foreground no-underline">
                                    <Avatar className="h-6 w-6 shrink-0 rounded-full border border-border bg-linear-to-br from-muted to-border font-sans text-[10px] font-semibold text-foreground">
                                        {author.avatarId && <AvatarImage src={getAvatarById(author.avatarId)} alt="" />}
                                        <AvatarFallback>{(author.nickname?.charAt(0) || "?").toUpperCase()}</AvatarFallback>
                                    </Avatar>
                                    <span className="min-w-0 truncate font-medium transition-colors group-hover:text-primary">{author.nickname?.trim() || "Doctor"}</span>
                                </Link>
                            ) : (
                                <span className="inline-flex items-center gap-2">
                                    <span className="inline-flex h-6 w-6 items-center justify-center rounded-full border border-border bg-muted font-sans text-[10px] font-semibold text-muted-foreground">?</span>
                                    <span className="font-medium text-foreground">Unknown author</span>
                                </span>
                            )}

                            <span className="opacity-50 max-sm:hidden" aria-hidden="true">
                                ·
                            </span>

                            <span className="inline-flex items-center gap-1.5">
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3.5 w-3.5 opacity-70" aria-hidden="true">
                                    <circle cx="12" cy="12" r="10" />
                                    <path d="M12 6v6l4 2" />
                                </svg>
                                <span>Updated {updatedRel}</span>
                            </span>

                            {stats && (
                                <>
                                    <span className="opacity-50" aria-hidden="true">
                                        ·
                                    </span>
                                    <span className="inline-flex items-center gap-1.5" title={`${formatNumber(stats.viewCount)} views`}>
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3.5 w-3.5 opacity-70" aria-hidden="true">
                                            <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
                                            <circle cx="12" cy="12" r="3" />
                                        </svg>
                                        <span className="font-mono tabular-nums font-semibold text-foreground">{formatNumberCompact(stats.viewCount)}</span>
                                        <span>views</span>
                                    </span>
                                    <span className="inline-flex items-center gap-1.5" title={`${formatNumber(stats.favoriteCount)} favorites`}>
                                        <svg viewBox="0 0 24 24" fill="currentColor" className="h-3 w-3 opacity-70" aria-hidden="true">
                                            <path d="M12 21s-7-4.5-9.5-9C.7 8.7 2.5 5 6 5c2 0 3.5 1 4 2.5C10.5 6 12 5 14 5c3.5 0 5.3 3.7 3.5 7-2.5 4.5-9.5 9-9.5 9Z" />
                                        </svg>
                                        <span className="font-mono tabular-nums font-semibold text-foreground">{formatNumberCompact(stats.favoriteCount)}</span>
                                        <span>favorites</span>
                                    </span>
                                </>
                            )}
                        </div>
                    </div>

                    <TierListActions detail={detail} />
                </div>
            </div>
        </header>
    );
}
