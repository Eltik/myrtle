import { Link } from "@tanstack/react-router";
import { CheckIcon, ChevronDownIcon, MinusIcon, PlusIcon } from "lucide-react";
import type React from "react";
import { useMemo, useState } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { StoryProgress } from "#/lib/story/progress";
import { humanTime, minutesFor, useReadingSpeed } from "#/lib/story/reading";
import { cn } from "#/lib/utils";
import type { messages as browseMessages } from "./Browse.messages";
import { type LibGroup, readFraction, sortedStories } from "./derive";
import { KindBadge } from "./GroupCard";
import type { messages } from "./ReadingOrderTab.messages";
import { type IMainlineDates, type IOrderRow, type IOrderSection, mainlineDateSplit, nextInOrder, orderRowModel, orderTotal, READING_ORDER_MODES, type ReadingOrderMode, readingOrder } from "./readingOrder";
import { StoryRow } from "./StoryRow";
import { sectionTotals } from "./stats";

/** The chapter label and the badge are the Browse cards' own strings, so the two surfaces cannot drift. */
type ReadingT = TypedT<typeof messages & typeof browseMessages>;

export interface IReadingOrderTabProps {
    groups: LibGroup[];
    progress: StoryProgress;
    /** The game's own read verdict, weighed beside the document. */
    gameRead: ReadonlySet<string>;
}

/**
 * The library's third view: one list of every chapter, in whichever order the
 * reader picks.
 *
 * Two of the three orders are DERIVED and shipped. The third, an in-world
 * timeline, is a hand-written curation with no field behind it, so its tab
 * carries the reason and no list; see the head of `readingOrder.ts` for what
 * the data does and does not carry.
 */
export function ReadingOrderTab({ groups, progress, gameRead }: IReadingOrderTabProps): React.ReactElement {
    const t: ReadingT = useT("story");
    const [mode, setMode] = useState<ReadingOrderMode>("release");
    const [expanded, setExpanded] = useState<string | null>(null);

    const sections = useMemo(() => readingOrder(mode, groups), [mode, groups]);
    const dates = useMemo(() => mainlineDateSplit(groups), [groups]);
    const next = useMemo(() => nextInOrder(sections, progress, gameRead), [sections, progress, gameRead]);
    const total = orderTotal(sections);

    return (
        <div className="pt-4">
            <fieldset className="msv-scroll mb-3 flex max-w-full gap-0.5 overflow-x-auto rounded-[9px] border border-border bg-secondary/45 p-0.75" aria-label={t("reading.mode.aria")}>
                {READING_ORDER_MODES.map((key) => (
                    <button
                        key={key}
                        type="button"
                        onClick={() => setMode(key)}
                        aria-pressed={mode === key}
                        className={cn("h-11 pointer-coarse:h-11 shrink-0 cursor-pointer rounded-md px-3 font-sans font-semibold text-[12px] transition-colors sm:h-8", mode === key ? "bg-background text-foreground shadow-sm/5" : "text-muted-foreground hover:text-foreground")}
                    >
                        {t(`reading.mode.${key}`)}
                    </button>
                ))}
            </fieldset>

            {mode === "timeline" ? (
                <div className="rounded-[14px] border border-border border-dashed p-6 sm:p-8">
                    <h3 className="m-0 font-sans font-semibold text-[14px] text-foreground">{t("reading.timeline.title")}</h3>
                    <p className="mt-2 mb-0 max-w-160 font-sans text-[13px] text-muted-foreground leading-relaxed">{t("reading.timeline.body")}</p>
                </div>
            ) : (
                <>
                    <div className="mb-4 grid gap-2 sm:grid-cols-2">
                        <Blurb tone="pro" text={mode === "release" ? t("reading.release.pros") : t("reading.storyline.pros")} />
                        <Blurb tone="con" text={mode === "release" ? t("reading.release.cons") : t("reading.storyline.cons")} />
                    </div>

                    <p className="mt-0 mb-2 font-mono text-[10.5px] text-muted-foreground tabular-nums">{t("reading.total", { count: total })}</p>

                    {/* THE POINTER IS THIS ORDER'S OWN. Release order and
                        storyline order disagree about what comes next by design,
                        so it is recomputed per order rather than shared with the
                        continue card at the top of the page. */}
                    <div className="mb-4 rounded-xl border border-primary/35 bg-primary/5 px-3 py-2.5">
                        {next ? (
                            <p className="m-0 flex flex-wrap items-baseline gap-x-2 gap-y-0.5 font-sans text-[12.5px]">
                                <span className="font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">{t("reading.next")}</span>
                                <Link to="/stories/$storyId" params={{ storyId: next.entry.id }} className="min-w-0 font-semibold text-foreground underline-offset-4 hover:underline">
                                    {next.group.name}
                                </Link>
                                <span className="min-w-0 text-muted-foreground">{next.entry.name}</span>
                            </p>
                        ) : (
                            <p className="m-0 font-sans text-[12.5px] text-muted-foreground">{t("reading.next.done")}</p>
                        )}
                    </div>

                    <div className="flex flex-col gap-5">
                        {sections.map((section) => (
                            <OrderSection key={section.key} section={section} dates={dates} progress={progress} gameRead={gameRead} expanded={expanded} onExpand={setExpanded} />
                        ))}
                    </div>
                </>
            )}
        </div>
    );
}

function Blurb({ tone, text }: { tone: "pro" | "con"; text: string }): React.ReactElement {
    const Icon = tone === "pro" ? PlusIcon : MinusIcon;
    return (
        <div className={cn("flex gap-2.5 rounded-xl border p-3", tone === "pro" ? "border-primary/35 bg-primary/5" : "border-border bg-secondary/30")}>
            <Icon className={cn("mt-0.5 size-3.5 shrink-0", tone === "pro" ? "text-primary" : "text-muted-foreground")} aria-hidden="true" />
            <p className="m-0 font-sans text-[12.5px] text-muted-foreground leading-relaxed">{text}</p>
        </div>
    );
}

function OrderSection({ section, dates, progress, gameRead, expanded, onExpand }: { section: IOrderSection; dates: IMainlineDates; progress: StoryProgress; gameRead: ReadonlySet<string>; expanded: string | null; onExpand: (id: string | null) => void }): React.ReactElement {
    const t: ReadingT = useT("story");
    const heading = section.kind === "arc" ? section.title : t(`reading.section.${section.kind}`);
    // The undated note quotes BOTH halves of the mainline, counted off the same
    // groups the list is built from: 12 chapters sit in zones the game dates -1
    // and are shelved here, 5 carry a real zone open time and are interleaved
    // with the events below rather than shelved with these.
    const note = section.kind === "undated" ? t("reading.section.undated.note", { undated: dates.undated, dated: dates.dated }) : section.kind === "unlinked" ? t("reading.section.unlinked.note") : null;
    const f = useFormatters();
    const { wpm } = useReadingSpeed();
    const totals = sectionTotals(
        section.rows.map((row) => row.group),
        progress,
        gameRead,
    );

    return (
        <section>
            <h3 className="m-0 font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">{heading}</h3>
            <p className="mt-1 mb-0 font-mono text-[10.5px] text-muted-foreground tabular-nums">
                {totals.total === null ? t("reading.section.totalsUncounted", { done: totals.done, groups: totals.groups }) : t("reading.section.totals", { words: f.number(totals.total), time: humanTime(minutesFor(totals.total, wpm)), done: totals.done, groups: totals.groups })}
            </p>
            {note ? <p className="mt-1 mb-0 max-w-160 font-sans text-[11.5px] text-muted-foreground/85 leading-relaxed">{note}</p> : null}
            <ul className="m-0 mt-2 flex list-none flex-col gap-1.5 p-0">
                {section.rows.map((row) => (
                    <OrderRow key={row.group.id} row={row} progress={progress} gameRead={gameRead} open={expanded === row.group.id} onToggle={() => onExpand(expanded === row.group.id ? null : row.group.id)} />
                ))}
            </ul>
        </section>
    );
}

/**
 * ONE ROW. The picture is the chapter's KEY VISUAL, not the derived cover: a
 * chapter that opens on a fade derives `bg_black.png`, and 81 of the 87 groups
 * carry an authored `bannerUrl` that is the picture the game itself draws.
 * The badge is the Browse card's own `KindBadge`, so MAIN STORY reads the same
 * here as it does two tabs over. The 56x68 chapter DECO is deliberately not
 * drawn: at the 18 px this row could give it, monochrome line art authored for
 * a 108 px game chrome reads as a smudge, and the chapter NUMBER is what a
 * reader knows the chapter by anyway.
 */
function OrderRow({ row, progress, gameRead, open, onToggle }: { row: IOrderRow; progress: StoryProgress; gameRead: ReadonlySet<string>; open: boolean; onToggle: () => void }): React.ReactElement {
    const t: ReadingT = useT("story");
    const f = useFormatters();
    const group = row.group;
    const fraction = readFraction(group.stories, progress, gameRead);
    const model = orderRowModel(group);
    const plate = model.plate.kind === "none" ? null : asset(model.plate.url);
    const pct = fraction.total > 0 ? Math.round((fraction.read / fraction.total) * 100) : 0;

    return (
        <li>
            <button
                type="button"
                onClick={onToggle}
                aria-expanded={open}
                aria-label={open ? t("reading.row.collapse", { name: group.name }) : t("reading.row.expand", { name: group.name })}
                className={cn("flex w-full cursor-pointer items-center gap-3 rounded-xl border border-border bg-card p-2 text-left ring-inset transition-colors hover:border-primary/45 focus-visible:ring-2 focus-visible:ring-ring/60 max-sm:min-h-11", open && "border-primary/70")}
            >
                <span className="relative block aspect-16/9 w-18 shrink-0 overflow-hidden rounded-lg bg-secondary/55 sm:w-24">
                    {plate ? (
                        <img src={plate} alt="" aria-hidden="true" loading="lazy" decoding="async" data-source={model.plate.kind} className="h-full w-full object-cover" />
                    ) : (
                        <span data-source="none" className="flex h-full w-full items-center justify-center bg-linear-to-br from-secondary/70 to-card px-1 text-center font-black font-heading text-[10px] text-muted-foreground/70 uppercase tracking-tight sm:text-[12px]">
                            {model.code}
                        </span>
                    )}
                </span>

                <span className="flex min-w-0 flex-1 flex-col gap-1">
                    <span className="flex min-w-0 items-center gap-2">
                        <KindBadge kind={model.kind} className="shrink-0" />
                        {/* NO ELLIPSIS UNDER 640, the same correction the list-mode row
                            already carries. At 390 the text column is 177 px and
                            "Come Catastrophes or Wakes of Vultures" measures 249, so the
                            clip landed mid-word and four of the 87 titles became
                            unidentifiable. Above 640 the column is wide enough that the
                            rule has never fired, and it is kept there. */}
                        <span className="min-w-0 flex-1 font-sans font-semibold text-[12.5px] text-foreground sm:truncate sm:text-[13.5px]">{group.name}</span>
                    </span>
                    <span className="flex flex-wrap items-baseline gap-x-2.5 gap-y-0.5 font-mono text-[10px] text-muted-foreground tabular-nums">
                        {model.chapter !== null ? <span className="text-foreground/75">{t("browse.card.chapter", { n: model.chapter })}</span> : null}
                        <span>{row.time !== null ? f.date(row.time * 1000) : t("reading.row.noDate")}</span>
                        <span>{fraction.total === 0 ? t("reading.row.noneExtracted", { count: fraction.listed }) : t("reading.row.read", { read: fraction.read, total: fraction.total })}</span>
                        {fraction.done ? <CheckIcon className="size-3 text-primary" aria-hidden="true" /> : null}
                    </span>
                    <span className="block h-0.75 w-full overflow-hidden rounded-full bg-input">
                        <span className="block h-full rounded-full bg-primary transition-[width] duration-300" style={{ width: `${pct}%` }} />
                    </span>
                </span>

                <ChevronDownIcon className={cn("size-4 shrink-0 text-muted-foreground transition-transform", open && "rotate-180")} aria-hidden="true" />
            </button>

            {open ? (
                <ul className="m-0 mt-1 flex list-none flex-col rounded-xl border border-primary/45 bg-card/70 p-2 sm:p-2.5">
                    {sortedStories(group.stories).map((story) => (
                        <li key={story.id}>
                            <StoryRow story={story} progress={progress} gameRead={gameRead} />
                        </li>
                    ))}
                </ul>
            ) : null}
        </li>
    );
}
