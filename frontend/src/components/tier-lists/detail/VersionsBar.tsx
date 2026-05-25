import { Link } from "@tanstack/react-router";
import { CheckIcon, ChevronDownIcon, HistoryIcon, RotateCcwIcon } from "lucide-react";
import { memo, useMemo } from "react";
import { Menu, MenuItem, MenuPopup, MenuTrigger } from "#/components/ui/menu";
import type { ITierListVersion } from "#/lib/api/tier-lists";
import { Markdown, stripMarkdown } from "#/lib/markdown";
import { cn, formatRelative } from "#/lib/utils";

interface IVersionsBarProps {
    slug: string;
    versions: ITierListVersion[];
    selectedVersion: ITierListVersion | null;
    isLatestView: boolean;
}

const CHANGELOG_PREVIEW_CHARS = 140;

function formatPublishedAt(iso: string): string {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return "-";
    return d.toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}

function previewChangelog(text: string | null): string | null {
    if (!text) return null;
    const collapsed = stripMarkdown(text).replace(/\s+/g, " ").trim();
    if (!collapsed) return null;
    if (collapsed.length <= CHANGELOG_PREVIEW_CHARS) return collapsed;
    return `${collapsed.slice(0, CHANGELOG_PREVIEW_CHARS).trimEnd()}…`;
}

interface IVersionMenuItemProps {
    slug: string;
    version: ITierListVersion;
    selected: boolean;
}

const VersionMenuItem = memo(function VersionMenuItem({ slug, version, selected }: IVersionMenuItemProps) {
    const changelogPreview = useMemo(() => previewChangelog(version.changelog), [version.changelog]);
    const publishedRel = useMemo(() => formatRelative(version.publishedAt), [version.publishedAt]);

    return (
        <MenuItem render={<Link to="/tier-lists/$id" params={{ id: slug }} search={{ v: version.version }} replace resetScroll={false} preload={false} />} className={cn("flex-col items-start gap-0.5 py-2", selected && "bg-accent/60 text-accent-foreground")}>
            <span className="flex w-full items-center gap-2">
                <span className="h-1.5 w-1.5 rounded-full bg-muted-foreground" aria-hidden="true" />
                <span className="font-medium tabular-nums">v{version.version}</span>
                <span className="font-mono text-[10.5px] text-muted-foreground">{publishedRel}</span>
                {selected && <CheckIcon className="ml-auto h-3.5 w-3.5" aria-hidden="true" />}
            </span>
            {changelogPreview && <span className="wrap-anywhere line-clamp-2 pl-3.5 font-sans text-[11.5px] text-muted-foreground leading-snug">{changelogPreview}</span>}
        </MenuItem>
    );
});

export function VersionsBar({ slug, versions, selectedVersion, isLatestView }: IVersionsBarProps) {
    if (versions.length === 0) return null;

    const latestVersion = versions[0] ?? null;
    const triggerLabel = isLatestView ? "Latest (live)" : selectedVersion ? `v${selectedVersion.version}` : `v${latestVersion?.version ?? "?"}`;
    const triggerHint = isLatestView ? "live state" : selectedVersion?.publishedAt ? formatRelative(selectedVersion.publishedAt) : null;

    return (
        <section aria-label="Tier list versions" className="mx-auto mt-4 w-[min(1200px,calc(100%-1.5rem))] sm:mt-6 sm:w-[min(1200px,calc(100%-2rem))]">
            <div className="flex flex-wrap items-center gap-2 rounded-xl border border-border bg-card/60 px-3 py-2 backdrop-blur-sm">
                <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">
                    <HistoryIcon className="h-3.5 w-3.5" aria-hidden="true" />
                    Version
                </span>

                <Menu>
                    <MenuTrigger className="inline-flex h-8 min-w-44 cursor-pointer items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none transition-colors hover:bg-accent" aria-label={`Select version, currently ${triggerLabel}`}>
                        <span className="flex min-w-0 items-center gap-2">
                            <span className={cn("h-1.5 w-1.5 shrink-0 rounded-full", isLatestView ? "bg-primary shadow-[0_0_6px_color-mix(in_srgb,var(--primary)_60%,transparent)]" : "bg-muted-foreground")} aria-hidden="true" />
                            <span className="truncate">{triggerLabel}</span>
                            {triggerHint && <span className="hidden text-muted-foreground sm:inline">· {triggerHint}</span>}
                        </span>
                        <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" aria-hidden="true" />
                    </MenuTrigger>
                    <MenuPopup align="start" sideOffset={6} className="w-80 max-w-[calc(100vw-1.5rem)]">
                        <MenuItem render={<Link to="/tier-lists/$id" params={{ id: slug }} search={{}} replace resetScroll={false} preload={false} />} className={cn("flex-col items-start gap-0.5 py-2", isLatestView && "bg-accent/60 text-accent-foreground")}>
                            <span className="flex w-full items-center gap-2">
                                <span className="h-1.5 w-1.5 rounded-full bg-primary shadow-[0_0_6px_color-mix(in_srgb,var(--primary)_60%,transparent)]" aria-hidden="true" />
                                <span className="font-medium">Latest (live)</span>
                                {isLatestView && <CheckIcon className="ml-auto h-3.5 w-3.5" aria-hidden="true" />}
                            </span>
                            <span className="pl-3.5 font-mono text-[10.5px] text-muted-foreground">Always reflects the most recent edits</span>
                        </MenuItem>
                        {versions.map((v) => (
                            <VersionMenuItem key={v.id} slug={slug} version={v} selected={!isLatestView && selectedVersion?.id === v.id} />
                        ))}
                    </MenuPopup>
                </Menu>

                <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                    <span className="text-foreground">{versions.length}</span> published
                </span>

                {!isLatestView && (
                    <Link
                        to="/tier-lists/$id"
                        params={{ id: slug }}
                        search={{}}
                        replace
                        resetScroll={false}
                        className="ml-auto inline-flex items-center gap-1 rounded-lg border border-border bg-popover px-2.5 py-1.5 font-medium font-sans text-[11.5px] text-foreground leading-none no-underline transition-colors hover:bg-accent"
                    >
                        <RotateCcwIcon className="h-3 w-3" aria-hidden="true" />
                        Back to latest
                    </Link>
                )}
            </div>

            {!isLatestView && selectedVersion && (
                <output aria-live="polite" className="mt-2 block rounded-xl border border-[color-mix(in_srgb,var(--primary)_30%,transparent)] bg-[color-mix(in_srgb,var(--primary)_8%,transparent)] px-3 py-2.5">
                    <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1 font-sans text-[12.5px] text-foreground">
                        <span className="font-bold font-mono text-[10.5px] text-primary uppercase tracking-[0.12em]">Snapshot</span>
                        <span className="font-medium">
                            v{selectedVersion.version} · published <time dateTime={selectedVersion.publishedAt}>{formatPublishedAt(selectedVersion.publishedAt)}</time>
                        </span>
                    </div>
                    {selectedVersion.changelog && <Markdown text={selectedVersion.changelog} className="wrap-anywhere mt-1.5 font-sans text-[13px] text-foreground leading-relaxed" flush />}
                </output>
            )}
        </section>
    );
}
