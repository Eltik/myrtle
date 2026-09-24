import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import type React from "react";
import { useMemo, useState } from "react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Skeleton } from "#/components/ui/skeleton";
import { storyCommunityQueryOptions } from "#/lib/api/story";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./CommunityTab.messages";
import { type ICommunityDepthGroup, type ICommunityGroupRow, type ICommunityStoryRow, rankCommunity } from "./community";
import type { LibIndex } from "./derive";
import { KindBadge } from "./GroupCard";

type CommunityT = TypedT<typeof messages>;

export interface ICommunityTabProps {
    index: LibIndex;
    /** Opens the chapter sheet on the Browse surface, the same seam the continue card uses. */
    onViewChapter: (groupId: string) => void;
}

/**
 * What everyone else has read: the chapters and events with the most readers,
 * the single stories with the most readers, the ones with the fewest, and how
 * far into one chapter people get.
 *
 * EVERY PERCENTAGE HERE IS A SHARE OF SYNCED PLAYERS, `StoryCommunity.players`,
 * and each one says so beside the number. Nothing on this tab is a rank, a
 * probability or a share of a narrower set, because none of those was measured.
 *
 * The route this reads answers 404 on a backend older than it, so the tab owns
 * an empty state and the query answers `null` rather than throwing: a library
 * that took the whole page down because one aggregate is missing would be a
 * worse failure than a missing tab.
 */
export function CommunityTab({ index, onViewChapter }: ICommunityTabProps): React.ReactElement {
    const t: CommunityT = useT("story");
    const f = useFormatters();
    const server = useGamedataServer();
    const { data, isPending } = useQuery(storyCommunityQueryOptions(server));
    const ranked = useMemo(() => rankCommunity(index, data), [index, data]);

    if (isPending) {
        return (
            <div className="flex flex-col gap-3 pt-4">
                <Skeleton className="h-4 w-64" />
                <Skeleton className="h-48 w-full" />
                <Skeleton className="h-48 w-full" />
            </div>
        );
    }

    if (!ranked) {
        return (
            <div className="mt-4 rounded-[14px] border border-border bg-card p-5 text-center">
                <h3 className="m-0 font-sans font-semibold text-[14px] text-foreground">{t("community.empty.title")}</h3>
                <p className="mx-auto mt-1.5 mb-0 max-w-prose font-sans text-[12.5px] text-muted-foreground">{t("community.empty.body")}</p>
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-6 pt-4">
            <p className="m-0 font-mono text-[11px] text-muted-foreground tracking-[0.02em]">{t("community.header", { players: f.number(ranked.players), when: f.relativeLong(new Date(ranked.computedAt * 1000).toISOString()) })}</p>

            <Section heading={t("community.section.top")} blurb={t("community.section.top.blurb")}>
                {ranked.top.length === 0 ? <Empty text={t("community.empty.rows")} /> : ranked.top.map((row, at) => <GroupRankRow key={row.id} row={row} rank={at + 1} onOpen={onViewChapter} t={t} f={f} />)}
            </Section>

            <Section heading={t("community.section.stories")} blurb={t("community.section.stories.blurb")}>
                {ranked.stories.length === 0 ? <Empty text={t("community.empty.rows")} /> : ranked.stories.map((row, at) => <StoryRankRow key={row.id} row={row} rank={at + 1} t={t} f={f} />)}
            </Section>

            <Section heading={t("community.section.bottom")} blurb={t("community.section.bottom.blurb")}>
                {ranked.bottom.length === 0 ? <Empty text={t("community.empty.rows")} /> : ranked.bottom.map((row, at) => <GroupRankRow key={row.id} row={row} rank={at + 1} onOpen={onViewChapter} t={t} f={f} />)}
                {ranked.skippedZero > 0 ? <p className="mt-2 mb-0 px-1 font-sans text-[12px] text-muted-foreground">{t("community.skipped", { count: ranked.skippedZero })}</p> : null}
            </Section>

            <DepthStrip groups={ranked.depth} initial={ranked.defaultDepth} t={t} f={f} />
        </div>
    );
}

function Section({ heading, blurb, children }: { heading: string; blurb: string; children: React.ReactNode }): React.ReactElement {
    return (
        <section className="rounded-[14px] border border-border bg-card p-4 sm:p-5">
            <h3 className="m-0 font-sans font-semibold text-[14px] text-foreground">{heading}</h3>
            <p className="mt-1 mb-3 max-w-prose font-sans text-[12.5px] text-muted-foreground">{blurb}</p>
            <div className="flex flex-col">{children}</div>
        </section>
    );
}

function Empty({ text }: { text: string }): React.ReactElement {
    return <p className="m-0 px-1 py-2 font-sans text-[12.5px] text-muted-foreground">{text}</p>;
}

/**
 * The shared grid: rank, name, readers, finished. One definition so the two
 * group lists and the story list line up column for column.
 *
 * UNDER 640 IT IS TWO COLUMNS AND THE FIGURES TAKE A SECOND LINE. The `auto`
 * third track is the readers-plus-finished block, which measures about 200 px;
 * on a 390 px phone that left the name 68 px and "Necessary Solutions" read as
 * "Necessa...". The figures now sit under the name in the same column, which
 * gives the title the full 270.
 */
const ROW = "grid w-full grid-cols-[1.75rem_minmax(0,1fr)] items-center gap-x-3 gap-y-1.5 rounded-md px-2 py-2 text-left max-sm:min-h-11 sm:grid-cols-[1.75rem_minmax(0,1fr)_auto] sm:gap-y-0.5";

/** Where a row's figures go once the grid is two columns wide: under the name, never under the rank. */
const ROW_FIGURES = "max-sm:col-start-2 max-sm:justify-self-end";

function Rank({ rank }: { rank: number }): React.ReactElement {
    return <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{rank}</span>;
}

/** Readers and the share of players they are, stacked so the label never leaves the number unqualified. */
function Readers({ readers, readerShare, label, className, t, f }: { readers: number; readerShare: number; label: string; className?: string; t: CommunityT; f: ReturnType<typeof useFormatters> }): React.ReactElement {
    return (
        <span className={cn("flex shrink-0 flex-col items-end", className)}>
            <span className="font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">{label}</span>
            <span className="font-mono text-[14px] text-foreground tabular-nums">{f.number(readers)}</span>
            <span className="font-mono text-[10px] text-muted-foreground tabular-nums">{t("community.ofPlayers", { share: f.percent(readerShare) })}</span>
        </span>
    );
}

function GroupRankRow({ row, rank, onOpen, t, f }: { row: ICommunityGroupRow; rank: number; onOpen: (groupId: string) => void; t: CommunityT; f: ReturnType<typeof useFormatters> }): React.ReactElement {
    return (
        <button type="button" onClick={() => onOpen(row.id)} aria-label={t("community.open", { name: row.name })} className={cn(ROW, "transition-colors hover:bg-accent")}>
            <Rank rank={rank} />
            <span className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1">
                <span className="min-w-0 truncate font-sans text-[13px] text-foreground">{row.name}</span>
                <KindBadge kind={row.kind} />
            </span>
            <span className={cn("flex items-start gap-4 sm:gap-6", ROW_FIGURES)}>
                <Readers readers={row.readers} readerShare={row.readerShare} label={t("community.readers")} t={t} f={f} />
                <span className="flex w-24 shrink-0 flex-col items-end sm:w-28">
                    <span className="font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">{t("community.finished")}</span>
                    {row.finished === null || row.finishedShare === null ? (
                        <span className="mt-0.5 text-right font-sans text-[11px] text-muted-foreground" title={t("community.notMeasurable.why")}>
                            {t("community.notMeasurable")}
                        </span>
                    ) : (
                        <>
                            <span className="font-mono text-[14px] text-foreground tabular-nums">{f.number(row.finished)}</span>
                            <span className="font-mono text-[10px] text-muted-foreground tabular-nums">{t("community.ofPlayers", { share: f.percent(row.finishedShare) })}</span>
                        </>
                    )}
                </span>
            </span>
        </button>
    );
}

/**
 * One story. The 25 listed stories with no script have no reader route to
 * point at, so those rows are plain text rather than a link that 404s.
 */
function StoryRankRow({ row, rank, t, f }: { row: ICommunityStoryRow; rank: number; t: CommunityT; f: ReturnType<typeof useFormatters> }): React.ReactElement {
    const inner = (
        <>
            <Rank rank={rank} />
            <span className="flex min-w-0 flex-col">
                <span className="min-w-0 truncate font-sans text-[13px] text-foreground">
                    {row.code ? <span className="mr-2 font-mono text-[10.5px] text-muted-foreground tabular-nums">{row.code}</span> : null}
                    {row.name}
                </span>
                <span className="min-w-0 truncate font-sans text-[11.5px] text-muted-foreground">{row.groupName}</span>
            </span>
            <Readers readers={row.readers} readerShare={row.readerShare} label={t("community.readers")} className={ROW_FIGURES} t={t} f={f} />
        </>
    );
    if (!row.hasScript) return <div className={cn(ROW, "opacity-60")}>{inner}</div>;
    return (
        <Link to="/stories/$storyId" params={{ storyId: row.id }} className={cn(ROW, "transition-colors hover:bg-accent")}>
            {inner}
        </Link>
    );
}

/**
 * Readers per story of one chapter, as BARS and never a line: `depth` is not
 * monotone, because a `spst_*` special stage or an interlude is entered by its
 * own door and can outrun the story before it. A line between the points would
 * draw a funnel the data does not describe.
 *
 * Each bar's height is its share of PLAYERS, the same denominator the rest of
 * the tab uses, so a chapter that half the site has read draws at half height
 * rather than being stretched to fill the box.
 */
function DepthStrip({ groups, initial, t, f }: { groups: ICommunityDepthGroup[]; initial: string | null; t: CommunityT; f: ReturnType<typeof useFormatters> }): React.ReactElement {
    const [pick, setPick] = useState<string | null>(initial);
    const group = groups.find((g) => g.id === pick) ?? groups[0] ?? null;

    return (
        <section className="rounded-[14px] border border-border bg-card p-4 sm:p-5">
            <h3 className="m-0 font-sans font-semibold text-[14px] text-foreground">{t("community.section.depth")}</h3>
            <p className="mt-1 mb-3 max-w-prose font-sans text-[12.5px] text-muted-foreground">{t("community.section.depth.blurb")}</p>
            {group === null ? (
                <Empty text={t("community.depth.none")} />
            ) : (
                <>
                    <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
                        <span className="font-sans text-[12.5px] text-muted-foreground">{t("community.depth.picker")}</span>
                        <Select value={group.id} onValueChange={(v: string | null) => v && setPick(v)}>
                            <SelectTrigger size="sm" className="w-auto min-w-48 max-sm:h-11" aria-label={t("community.depth.picker")}>
                                <SelectValue>{() => group.name}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                {groups.map((g) => (
                                    <SelectItem key={g.id} value={g.id}>
                                        {g.name}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div className="mt-4 flex h-32 items-end gap-0.75 border-border border-b">
                        {group.bars.map((bar) => {
                            const label = t("community.depth.bar", { name: bar.code ? `${bar.code} ${bar.name}` : bar.name, readers: f.number(bar.readers), share: f.percent(bar.readerShare) });
                            return (
                                <span key={bar.id} role="img" title={label} aria-label={label} className="flex min-w-0 flex-1 items-end self-stretch">
                                    <span className="w-full rounded-t-xs bg-primary/80" style={{ height: `${Math.max(1, Math.round(bar.readerShare * 1000) / 10)}%` }} />
                                </span>
                            );
                        })}
                    </div>
                    <p className="mt-2 mb-0 font-sans text-[12.5px] text-muted-foreground">{t("community.depth.drop", { first: f.number(group.bars[0]?.readers ?? 0), last: f.number(group.bars[group.bars.length - 1]?.readers ?? 0) })}</p>
                </>
            )}
        </section>
    );
}
