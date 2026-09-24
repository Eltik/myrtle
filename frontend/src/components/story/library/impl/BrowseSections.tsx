/**
 * THE PAGE'S SECTION FURNITURE: the heading over a shelf, the glyph beside it,
 * and the sticky jump bar over the whole page.
 *
 * `Browse.tsx` owns which groups end up in which section and what a filter
 * does; this file owns how a section ANNOUNCES itself, which is a separate
 * question and the one most of the measured layout notes below are about.
 *
 * The jump bar carries a scroll spy, so it is the one piece here that is not
 * pure: `useScrollSpy` watches the headings through an IntersectionObserver
 * inset by the sticky header's own height.
 */
import type React from "react";
import { useCallback, useEffect, useRef, useState } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { GLYPH_INK } from "./art";
import type { messages } from "./Browse.messages";
import { chipLines, type IChapterRange, type IChipLines, type IChipModel, rangeIsSingle, SECTION_GLYPHS } from "./chapters";
import { type ISection, underLine } from "./sections";

type BrowseT = TypedT<typeof messages>;

/** The heading over the one flat section a sort produces. It carries no glyph and no chapter range, because the list it heads is not a shelf and describing it as one would be a lie. */
export function FlatHead({ title, count }: { title: string; count: string }): React.ReactElement {
    return (
        <div className="mb-3 flex items-end justify-between gap-3 border-border border-b pb-2">
            <div className="flex min-w-0 flex-col gap-0.5">
                <h2 className="m-0 font-bold font-heading text-[13px] text-foreground uppercase tracking-widest sm:truncate">{title}</h2>
                <span className="font-mono text-[10px] text-muted-foreground uppercase tabular-nums tracking-[0.08em] sm:truncate">{count}</span>
            </div>
        </div>
    );
}

/**
 * "Chapters 0 to 3", or "Chapter 9", as a reader says it. The wording is the
 * caller's, so the range rule stays pure in impl/chapters.ts.
 *
 * This is the UNABBREVIATED form and it is now the tooltip's alone: a bar that
 * overflows by 2,011 px at 1440 cannot afford "Chapters 0 to 3" eighteen
 * times, and a reader who wants the words gets them on hover.
 */
function rangeLabel(range: IChapterRange, t: BrowseT): string {
    return rangeIsSingle(range) ? t("browse.chip.rangeOne", { n: range.from }) : t("browse.chip.range", { from: range.from, to: range.to });
}

/** "Ch. 4-8", or "Ch. 9" over one chapter: the compact mono form every printed range now takes. A plain hyphen, not an en dash, because it sits in a monospace run beside digits. */
function compactLabel(range: IChapterRange, t: BrowseT): string {
    return rangeIsSingle(range) ? t("browse.chip.compactOne", { n: range.from }) : t("browse.chip.compact", { from: range.from, to: range.to });
}

/** The mono line under a chip's name: the chapter run, behind the act ordinal wherever the section's own name carries one. */
function monoLabel(lines: IChipLines, t: BrowseT): string | null {
    if (!lines.range) return null;
    if (lines.holds) return includesLabel(lines.range, t);
    const compact = compactLabel(lines.range, t);
    return lines.ordinal ? t("browse.chip.act", { ordinal: lines.ordinal, range: compact }) : compact;
}

/**
 * The muted second line of a themed shelf that HOLDS mainline chapters without
 * being a run of them. It is a secondary, never a heading: "The Ark" is a shelf
 * called The Ark, not "Chapters 7 to 14".
 */
function includesLabel(range: IChapterRange, t: BrowseT): string {
    return rangeIsSingle(range) ? t("browse.chip.includesOne", { n: range.from }) : t("browse.chip.includes", { from: range.from, to: range.to });
}

/** What a collapsed chip is called for a reader who cannot read the mark it collapsed to: the tooltip's own pair, in one line. */
function chipLabel(chip: IChipModel, t: BrowseT): string {
    return underLine([chip.name, chip.range ? rangeLabel(chip.range, t) : chip.includes ? includesLabel(chip.includes, t) : null]);
}

/**
 * The section's own icon when the wire sends one, else the glyph its id hashes
 * to. The three image shapes are NOT the same slot: an arc icon is a 184x52
 * banner and is sized by HEIGHT (20 px in a chip, 26 in a heading), a shelf
 * logo is a 108x108 emblem in a 22/28 px square, and a shelf abbreviation is
 * 44x36 in a 20/24 px one.
 *
 * THE BANNER CAME DOWN AND THE MONOGRAM STAYED PUT, which is a correction. At
 * 24/32 the banner rendered 84.9 px wide in a chip and 113.2 in a heading
 * against a 22/28 px monogram beside it, so RL, UR and LA read as a lesser
 * kind of mark than the four act banners. The banner now measures 70.8 px in a
 * chip and 92.0 in a heading, and the monogram is the LARGER of the two by
 * height in both places.
 *
 * A MONOGRAM ALONE IN A CHIP IS 24 PX, not the 22 it is beside a name. A
 * collapsed shelf chip is the logo and nothing else, and at 22 px in a 36 px
 * pill it read as an icon rather than as the shelf's mark.
 *
 * INK. The game art is monochrome and is flattened to the theme's ink
 * (`GLYPH_INK`); on the ACTIVE chip, which is a filled primary pill, that ink
 * would be black on red, so the active chip inks the glyph white instead.
 */
export function SectionGlyph({ chip, place, className, ink = "theme", alone = false }: { chip: IChipModel; place: "chip" | "head"; className?: string; ink?: "theme" | "white"; alone?: boolean }): React.ReactElement {
    if (chip.iconUrl) {
        const size = chip.iconWide ? (place === "chip" ? "h-5 w-auto max-w-19.5" : "h-6.5 w-auto max-w-22 sm:max-w-26") : chip.iconLogo ? (place === "chip" ? (alone ? "size-6" : "size-5.5") : "size-7") : place === "chip" ? "size-5" : "size-6";
        return <img src={asset(chip.iconUrl)} alt="" loading="lazy" decoding="async" className={cn("shrink-0 object-contain", size, ink === "white" ? "brightness-0 invert" : GLYPH_INK, className)} />;
    }
    const Glyph = SECTION_GLYPHS[chip.glyph] ?? SECTION_GLYPHS[0];
    return <Glyph className={cn("shrink-0", place === "chip" ? "size-4" : "size-4.5", className)} aria-hidden="true" />;
}

/**
 * A left-aligned heading over a thin rule, the page's only section furniture.
 *
 * THE NAME IS THE HEADING, on every section and not only on the themed
 * shelves. It led with the range instead until two readers said the same thing
 * about the same line: "Shatter of a Vision" is what the section is called and
 * "Main story · Chapters 4 to 8" is what is true about it, and the page had
 * them the wrong way round. Neither is dropped, because many readers know the
 * mainline only as "chapter 8": the range moves to the muted line, where it now
 * sits behind the kind the section is all of and in front of the count,
 * reading "Main story · Ch. 4-8 · 6 chapters".
 */
export function SectionHead({ chip, count, action }: { chip: IChipModel | undefined; count: string; action?: React.ReactNode }): React.ReactElement | null {
    const t: BrowseT = useT("story");
    if (!chip) return null;
    const heading = chip.name;
    const kind = chip.filter ? t(`browse.filter.${chip.filter}`) : null;
    const chapters = chip.range ? compactLabel(chip.range, t) : chip.includes ? includesLabel(chip.includes, t) : null;
    const under = underLine([kind, chapters, count]);
    return (
        <div className="mb-3 flex items-end justify-between gap-3 border-border border-b pb-2">
            <div className="flex min-w-0 items-center gap-2.5">
                <SectionGlyph chip={chip} place="head" className="text-muted-foreground" />
                {/* A HEADING WRAPS ON A PHONE RATHER THAN LOSING ITS RANGE. At 320 the
                    wide arc banner leaves the text column 173 px against a 245 px
                    "Main story · Chapters 0 to 3", so the ellipsis ate the numbers a
                    reader navigates by. `truncate` is kept from 640 up, where the
                    column is 245 px and the string has never been cut. */}
                <div className="flex min-w-0 flex-col gap-0.5">
                    <h2 className="m-0 font-bold font-heading text-[15px] text-foreground uppercase tracking-widest sm:truncate">{heading}</h2>
                    <span className="font-mono text-[10px] text-muted-foreground uppercase tabular-nums tracking-[0.08em] sm:truncate">{under}</span>
                </div>
            </div>
            {action}
        </div>
    );
}

/** How much of the scroller is faded at each edge once there is something to scroll to. */
const FADE = 28;

/**
 * The sticky jump bar. Above 640 px it is a scroller of chips that fades at
 * whichever edge still has chips behind it and keeps the active chip centred;
 * under it a native select, because eighteen chips on a 390 px phone is a
 * scroller nobody discovers.
 *
 * ONE CHIP IS EXPANDED AND SEVENTEEN ARE COLLAPSED. Every chip used to print
 * both its lines and its count, and the row measured 3,411 px of scroll width
 * against a 1,400 px rail at 1440: the bar was a horizontal list nobody could
 * see the end of, and its first line was "Chapters 0 to 3" rather than the
 * name. A collapsed chip now prints one line at most, and on a mainline arc
 * that line is the compact range alone because the 184x52 banner beside it
 * reads ACT I; the active chip alone expands to the name over the range and
 * keeps its count. The tooltip carries the unabbreviated pair for every
 * collapsed chip and the `aria-label` carries the name, so nothing the bar
 * stops printing becomes unreachable.
 *
 * THE EIGHTEEN CHIPS FIT THE RAIL AT 1440, which the last pass said was not
 * fixable here: 2,877 px of content -> 1,359.5 against a 1,400 px rail, no
 * horizontal scroll and both fade masks off. The shelf names were the whole of
 * it. Twelve themed chips were 1,881.0 px on their names and are 504.0 px as
 * 42 px monograms, and the two chips the game marks with nothing were 280.1 px
 * as "Other events" and "Operator records" against 139.6 as MISC and REC. The
 * four mainline chips are untouched at 613.9 px. Two of those numbers are
 * load-bearing and were measured, not chosen: keeping the two full names is
 * 1,500.0 px and still scrolls, and leaving the bare monogram chips on the
 * 10 px side padding the text chips use is 1,407.5 and still scrolls, which is
 * why a chip with no text is padded 8.
 *
 * THE ACTIVE CHIP CAN STILL PUT THE BAR OVER THE RAIL, by design and only
 * while it is active: the widest shelf, "Snow and Silver Steel", expands to
 * 201.7 px and takes the row to 1,519.2. The reader is looking at the section
 * that chip names, and the bar centres it.
 *
 * The active chip is centred by writing `scrollLeft` directly, never by
 * `scrollIntoView`: that walks every scrollable ancestor and would drag the
 * PAGE to the section the reader has not asked for yet.
 */
export function JumpBar({ chips, active }: { chips: readonly IChipModel[]; active: string | null }): React.ReactElement {
    const t: BrowseT = useT("story");
    const scroller = useRef<HTMLDivElement | null>(null);
    const [edges, setEdges] = useState<{ start: boolean; end: boolean }>({ start: false, end: false });

    const measure = useCallback(() => {
        const node = scroller.current;
        if (!node) return;
        setEdges({ start: node.scrollLeft > 2, end: node.scrollLeft + node.clientWidth < node.scrollWidth - 2 });
    }, []);

    useEffect(() => {
        measure();
        window.addEventListener("resize", measure);
        return () => window.removeEventListener("resize", measure);
    }, [measure]);

    useEffect(() => {
        const node = scroller.current;
        if (!node || !active) return;
        const chip = node.querySelector<HTMLElement>(`[data-chip="${CSS.escape(active)}"]`);
        if (!chip) return;
        node.scrollTo({ left: Math.max(0, chip.offsetLeft - (node.clientWidth - chip.offsetWidth) / 2), behavior: "smooth" });
    }, [active]);

    const fade = `linear-gradient(to right, transparent 0, #000 ${edges.start ? FADE : 0}px, #000 calc(100% - ${edges.end ? FADE : 0}px), transparent 100%)`;

    return (
        <nav aria-label={t("browse.jump.aria")} className="page-bleed sticky top-14 z-20 mt-4 border-border/60 border-b bg-background/85 px-(--page-gutter) py-2 backdrop-blur-md sm:top-16">
            <label className="flex items-center gap-2 sm:hidden">
                <span className="shrink-0 font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">{t("browse.jump.select")}</span>
                <select
                    value={active ?? ""}
                    onChange={(e) => document.getElementById(e.target.value)?.scrollIntoView({ behavior: "smooth", block: "start" })}
                    className="h-11 min-w-0 flex-1 rounded-[9px] border border-border bg-secondary/50 px-2.5 font-sans text-[13px] text-foreground outline-none focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-ring/40"
                >
                    {/* The same wording rule as a heading, in one line: the NAME first,
                        then the compact range, then how many chapters are under it. */}
                    {chips.map((chip) => (
                        <option key={chip.id} value={chip.id}>
                            {underLine([chip.name, chip.range ? compactLabel(chip.range, t) : chip.includes ? includesLabel(chip.includes, t) : null])} ({chip.count})
                        </option>
                    ))}
                </select>
            </label>
            <div ref={scroller} onScroll={measure} className="msv-scroll -my-1 hidden max-w-full gap-1.5 overflow-x-auto py-1 sm:flex" style={{ maskImage: fade, WebkitMaskImage: fade }}>
                {chips.map((chip) => (
                    <JumpChip key={chip.id} chip={chip} on={active === chip.id} t={t} />
                ))}
            </div>
        </nav>
    );
}

/**
 * One chip. Collapsed it is the mark plus at most one line; active it is the
 * mark, the name, the range under it and the count.
 *
 * A COLLAPSED SHELF CHIP HAS NO TEXT AT ALL, so it says what it is twice over
 * where a sighted reader cannot read the monogram: in the tooltip, and in the
 * `aria-label`, which is the only one of the two a screen reader or a keyboard
 * reader is guaranteed. The label is the tooltip's own pair in one line, and it
 * is set on every chip whose name the collapsed form leaves out, the mainline
 * arcs included: "CH. 4-8" names the run and not the act.
 *
 * The tooltip is on the COLLAPSED chip only, and it carries what the collapsed
 * form leaves out: the name, and the range in full words rather than in the
 * "Ch. 4-8" shorthand. An expanded chip is already showing both, so tipping it
 * would repeat the screen.
 */
function JumpChip({ chip, on, t }: { chip: IChipModel; on: boolean; t: BrowseT }): React.ReactElement {
    const lines = chipLines(chip, on);
    const mono = monoLabel(lines, t);
    // A chip that prints nothing is a square around its mark, and the 10 px of
    // side padding a chip with text wears is 4 px a chip the rail has not got:
    // the twelve shelves are 504.0 px at 8 and 552.0 at 10, and only the first
    // of those fits at 1440.
    const bare = lines.name === null && lines.abbr === null && mono === null;
    const link = (
        <a
            data-chip={chip.id}
            href={`#${chip.id}`}
            aria-current={on ? "true" : undefined}
            aria-label={lines.name === null ? chipLabel(chip, t) : undefined}
            className={cn(
                "flex shrink-0 items-center gap-2 rounded-[10px] border py-1.5 transition-colors focus-visible:ring-2 focus-visible:ring-ring/60",
                bare ? "px-2" : "px-2.5",
                on ? "border-primary bg-primary text-primary-foreground" : "border-transparent text-muted-foreground hover:border-border hover:bg-secondary/50 hover:text-foreground",
            )}
        >
            <SectionGlyph chip={chip} place="chip" alone={bare} ink={on ? "white" : "theme"} className={on ? "opacity-95" : "opacity-70"} />
            {bare ? null : (
                <span className="flex min-w-0 flex-col items-start leading-none">
                    {lines.name ? <span className="whitespace-nowrap font-sans font-semibold text-[13px]">{lines.name}</span> : null}
                    {lines.abbr ? <span className="whitespace-nowrap font-mono font-semibold text-[10px] uppercase tracking-[0.08em] opacity-75">{lines.abbr}</span> : null}
                    {mono ? <span className={cn("whitespace-nowrap font-mono text-[10px] uppercase tabular-nums tracking-[0.08em]", lines.name ? "mt-1" : "", on ? "opacity-80" : "opacity-75")}>{mono}</span> : null}
                </span>
            )}
            {on ? <span className="shrink-0 font-mono text-[10px] tabular-nums opacity-80">{chip.count}</span> : null}
        </a>
    );
    if (on) return link;
    return (
        <Tooltip>
            <TooltipTrigger render={link} />
            <TooltipContent>
                <span className="flex flex-col gap-0.5 py-0.5">
                    <span className="font-semibold">{chip.name}</span>
                    <span className="font-mono text-[10px] text-muted-foreground uppercase tabular-nums">{underLine([chip.range ? rangeLabel(chip.range, t) : chip.includes ? includesLabel(chip.includes, t) : null, t("browse.section.count", { count: chip.count })])}</span>
                </span>
            </TooltipContent>
        </Tooltip>
    );
}

/** The section's own name, or the translated name for the sections the data does not name. */
export function sectionTitle(section: ISection, t: BrowseT): string {
    switch (section.kind) {
        case "other":
            return t("browse.section.other");
        case "main":
            return t("browse.section.main");
        case "year":
            return section.title === "" ? t("browse.section.undated") : section.title;
        default:
            return section.title;
    }
}

/**
 * Which section the reader is in. The observer's root is inset from the top by
 * the sticky header plus the chip row, so a section counts as current once its
 * heading clears the furniture rather than while it is still under it.
 *
 * The LAST section is a special case: the collapsed records section is 62 px
 * tall at the very bottom of the page, and no amount of scrolling can push it
 * into the band, so the spy left the previous chip lit on a page scrolled all
 * the way down. At the bottom the last chip wins outright.
 */
export function useScrollSpy(ids: readonly string[]): string | null {
    const [active, setActive] = useState<string | null>(null);
    const idsKey = ids.join("|");
    const visible = useRef<Set<string>>(new Set());

    useEffect(() => {
        if (typeof IntersectionObserver === "undefined") return;
        const list = idsKey === "" ? [] : idsKey.split("|");
        visible.current = new Set();
        const atBottom = () => window.scrollY + window.innerHeight >= document.documentElement.scrollHeight - 4;
        const pick = () => {
            if (atBottom() && list.length > 0) {
                setActive(list[list.length - 1] ?? null);
                return;
            }
            const first = list.find((id) => visible.current.has(id));
            if (first) setActive(first);
        };
        const observer = new IntersectionObserver(
            (entries) => {
                for (const entry of entries) {
                    if (entry.isIntersecting) visible.current.add(entry.target.id);
                    else visible.current.delete(entry.target.id);
                }
                pick();
            },
            { rootMargin: "-140px 0px -60% 0px", threshold: 0 },
        );
        for (const id of list) {
            const node = document.getElementById(id);
            if (node) observer.observe(node);
        }
        window.addEventListener("scroll", pick, { passive: true });
        return () => {
            observer.disconnect();
            window.removeEventListener("scroll", pick);
        };
    }, [idsKey]);

    return active;
}
