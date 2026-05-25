import { ChevronDown, GitCommitHorizontal } from "lucide-react";
import type { IChangelogCommit } from "#/lib/api/changelog";
import { cn, formatRelative } from "#/lib/utils";
import { commitTypeStyle } from "./commit-types";

const BODY_PREVIEW_LIMIT = 220;

function absoluteDate(iso: string): string {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return "";
    return d.toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}

export function CommitItem({ commit }: { commit: IChangelogCommit }) {
    const style = commitTypeStyle(commit.type);
    const { Icon } = style;
    const hasBody = commit.body.length > 0;
    const longBody = commit.body.length > BODY_PREVIEW_LIMIT;

    return (
        <li className="group/item relative grid grid-cols-[1.25rem_minmax(0,1fr)] gap-x-3 pb-5 last:pb-0 sm:grid-cols-[1.5rem_minmax(0,1fr)] sm:gap-x-4">
            {/* Timeline rail */}
            <div className="relative flex justify-center">
                <span aria-hidden="true" className="absolute top-4 -bottom-5 w-px bg-linear-to-b from-border to-border/50 group-last/item:hidden" />
                <span className="relative z-1 mt-1 size-3.5 rounded-full bg-background ring-[3px] ring-background">
                    <span className={cn("block size-full rounded-full ring-2 ring-background", style.dotClass)} />
                </span>
            </div>

            {/* Commit body */}
            <div className="min-w-0 rounded-xl border border-border bg-card px-3.5 py-3 transition-colors hover:border-border/70 sm:px-4">
                <div className="flex flex-wrap items-center gap-x-2 gap-y-1.5">
                    <span className={cn("inline-flex shrink-0 items-center gap-1 rounded-md px-1.5 py-0.5 font-medium text-[11px] uppercase leading-none tracking-wide ring-1 ring-inset", style.pillClass)}>
                        <Icon className="size-3" strokeWidth={2} aria-hidden="true" />
                        {style.label}
                    </span>
                    {commit.scope ? <span className="inline-flex shrink-0 items-center rounded-md bg-muted px-1.5 py-0.5 font-medium font-mono text-[11px] text-muted-foreground leading-none">{commit.scope}</span> : null}
                    {commit.breaking ? <span className="inline-flex shrink-0 items-center rounded-md bg-destructive/12 px-1.5 py-0.5 font-semibold text-[11px] text-destructive-foreground uppercase leading-none tracking-wide ring-1 ring-destructive/25 ring-inset">Breaking</span> : null}
                </div>

                <p className="m-0 mt-2 font-sans font-semibold text-[15px] text-foreground leading-snug">{commit.title}</p>

                {hasBody ? (
                    longBody ? (
                        <details className="group/body mt-1.5">
                            <summary className="inline-flex cursor-pointer list-none items-center gap-1 font-sans text-[13px] text-muted-foreground leading-none transition-colors hover:text-foreground [&::-webkit-details-marker]:hidden">
                                <ChevronDown className="size-3.5 transition-transform duration-200 group-open/body:rotate-180" strokeWidth={2} aria-hidden="true" />
                                Details
                            </summary>
                            <p className="m-0 mt-2 whitespace-pre-line font-sans text-[13.5px] text-muted-foreground leading-[1.6]">{commit.body}</p>
                        </details>
                    ) : (
                        <p className="m-0 mt-1.5 whitespace-pre-line font-sans text-[13.5px] text-muted-foreground leading-[1.6]">{commit.body}</p>
                    )
                ) : null}

                <div className="mt-2.5 flex flex-wrap items-center gap-x-2.5 gap-y-1.5 font-sans text-[12.5px] text-muted-foreground">
                    <span className="inline-flex items-center gap-1.5">
                        {commit.author.avatarUrl ? (
                            <img src={commit.author.avatarUrl} alt="" width={18} height={18} loading="lazy" className="size-4.5 shrink-0 rounded-full bg-muted object-cover ring-1 ring-border" />
                        ) : (
                            <span className="inline-flex size-4.5 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground ring-1 ring-border">
                                <GitCommitHorizontal className="size-3" strokeWidth={2} aria-hidden="true" />
                            </span>
                        )}
                        {commit.author.profileUrl ? (
                            <a href={commit.author.profileUrl} target="_blank" rel="noreferrer" className="font-medium text-foreground/80 no-underline transition-colors hover:text-primary">
                                {commit.author.name}
                            </a>
                        ) : (
                            <span className="font-medium text-foreground/80">{commit.author.name}</span>
                        )}
                    </span>
                    <span aria-hidden="true" className="text-border">
                        ·
                    </span>
                    <time dateTime={commit.date} title={absoluteDate(commit.date)} className="tabular-nums">
                        {formatRelative(commit.date)}
                    </time>
                    <a
                        href={commit.url}
                        target="_blank"
                        rel="noreferrer"
                        className="ml-auto inline-flex items-center gap-1 rounded-md bg-muted/60 px-1.5 py-0.5 font-medium font-mono text-[11.5px] text-muted-foreground no-underline transition-colors hover:bg-muted hover:text-foreground"
                        title={`View ${commit.shortSha} on GitHub`}
                    >
                        {commit.shortSha}
                    </a>
                </div>
            </div>
        </li>
    );
}
