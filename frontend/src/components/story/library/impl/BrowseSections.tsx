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
 * INK. The game art is monochrome and is flattened to the theme's ink
 * (`GLYPH_INK`); on the ACTIVE chip, which is a filled primary pill, that ink
 * would be black on red, so the active chip inks the glyph white instead.
 */
export function SectionGlyph({ chip, place, className, ink = "theme" }: { chip: IChipModel; place: "chip" | "head"; className?: string; ink?: "theme" | "white" }): React.ReactElement {
    if (chip.iconUrl) {
        const size = chip.iconWide ? (place === "chip" ? "h-5 w-auto max-w-19.5" : "h-6.5 w-auto max-w-22 sm:max-w-26") : chip.iconLogo ? (place === "chip" ? "size-5.5" : "size-7") : place === "chip" ? "size-5" : "size-6";
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
 * name. A collapsed chip now prints one line, and on a mainline arc that line
 * is the compact range alone because the 184x52 banner beside it reads ACT I;
 * the active chip alone expands to the name over the range and keeps its
 * count. The hover tooltip carries the unabbreviated pair for every collapsed
 * chip, so nothing the bar stops printing becomes unreachable.
 *
 * THE BAR STILL SCROLLS AT 1440 AND THAT IS NOT FIXABLE HERE: 3,411 px ->
 * 3,014 against the same 1,400 px rail. The four mainline chips paid 1,025.9
 * -> 722.2 and the count came off every collapsed chip, but the thirteen
 * themed shelves are 2,036.3 px of that total on their NAMES, which no
 * collapse may touch: a shelf's 108x108 logo is a monogram (RL, UR, LA) and
 * names nothing, so "The Ark" is the only thing its chip says. Eighteen chips
 * on a 1,400 px rail needs the shelf names abbreviated or dropped, which is a
 * decision about what a reader can recognise and not a layout tweak.
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
 * One chip. Collapsed it is the glyph plus a single line; active it is the
 * glyph, the name, the range under it and the count.
 *
 * The tooltip is on the COLLAPSED chip only, and it carries the pair the
 * collapsed form leaves out: the name a banner-only chip does not print, and
 * the range in full words rather than in the "Ch. 4-8" shorthand. An expanded
 * chip is already showing both, so tipping it would repeat the screen.
 */
function JumpChip({ chip, on, t }: { chip: IChipModel; on: boolean; t: BrowseT }): React.ReactElement {
    const lines = chipLines(chip, on);
    const mono = monoLabel(lines, t);
    const link = (
        <a
            data-chip={chip.id}
            href={`#${chip.id}`}
            aria-current={on ? "true" : undefined}
            className={cn(
                "flex shrink-0 items-center gap-2 rounded-[10px] border px-2.5 py-1.5 transition-colors focus-visible:ring-2 focus-visible:ring-ring/60",
                on ? "border-primary bg-primary text-primary-foreground" : "border-transparent text-muted-foreground hover:border-border hover:bg-secondary/50 hover:text-foreground",
            )}
        >
            <SectionGlyph chip={chip} place="chip" ink={on ? "white" : "theme"} className={on ? "opacity-95" : "opacity-70"} />
            <span className="flex min-w-0 flex-col items-start leading-none">
                {lines.name ? <span className="whitespace-nowrap font-sans font-semibold text-[13px]">{lines.name}</span> : null}
                {mono ? <span className={cn("whitespace-nowrap font-mono text-[10px] uppercase tabular-nums tracking-[0.08em]", lines.name ? "mt-1" : "", on ? "opacity-80" : "opacity-75")}>{mono}</span> : null}
            </span>
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
