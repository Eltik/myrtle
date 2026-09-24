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
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { GLYPH_INK } from "./art";
import type { messages } from "./Browse.messages";
import { type IChapterRange, type IChipModel, rangeIsSingle, SECTION_GLYPHS } from "./chapters";
import type { ISection } from "./sections";

type BrowseT = TypedT<typeof messages>;

/** The heading over the one flat section a sort produces. It carries no glyph and no chapter range, because the list it heads is not a shelf and describing it as one would be a lie. */
export function FlatHead({ title, count }: { title: string; count: string }): React.ReactElement {
    return (
        <div className="mb-3 flex items-end justify-between gap-3 border-border border-b pb-2">
            <div className="flex min-w-0 flex-col gap-0.5">
                <h2 className="m-0 font-bold font-heading text-[13px] text-foreground uppercase tracking-[0.1em] sm:truncate">{title}</h2>
                <span className="font-mono text-[10px] text-muted-foreground uppercase tabular-nums tracking-[0.08em] sm:truncate">{count}</span>
            </div>
        </div>
    );
}

/**
 * "Chapters 0 to 3", or "Chapter 9", as a reader says it. The wording is the
 * caller's, so the range rule stays pure in impl/chapters.ts.
 */
function rangeLabel(range: IChapterRange, t: BrowseT): string {
    return rangeIsSingle(range) ? t("browse.chip.rangeOne", { n: range.from }) : t("browse.chip.range", { from: range.from, to: range.to });
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
 * banner and is sized by HEIGHT (24 px in a chip, 32 px in a heading), a shelf
 * logo is a 108x108 emblem in a 22/28 px square, and a shelf abbreviation is
 * 44x36 in an 18/20 px one.
 *
 * INK. The game art is monochrome and is flattened to the theme's ink
 * (`GLYPH_INK`); on the ACTIVE chip, which is a filled primary pill, that ink
 * would be black on red, so the active chip inks the glyph white instead.
 */
export function SectionGlyph({ chip, place, className, ink = "theme" }: { chip: IChipModel; place: "chip" | "head"; className?: string; ink?: "theme" | "white" }): React.ReactElement {
    if (chip.iconUrl) {
        const size = chip.iconWide ? (place === "chip" ? "h-6 w-auto max-w-[92px]" : "h-8 w-auto max-w-[88px] sm:max-w-[124px]") : chip.iconLogo ? (place === "chip" ? "size-5.5" : "size-7") : place === "chip" ? "size-4.5" : "size-5";
        return <img src={asset(chip.iconUrl)} alt="" loading="lazy" decoding="async" className={cn("shrink-0 object-contain", size, ink === "white" ? "brightness-0 invert" : GLYPH_INK, className)} />;
    }
    const Glyph = SECTION_GLYPHS[chip.glyph] ?? SECTION_GLYPHS[0];
    return <Glyph className={cn("shrink-0", place === "chip" ? "size-3.5" : "size-4", className)} aria-hidden="true" />;
}

/**
 * A left-aligned heading over a thin rule, the page's only section furniture.
 * A mainline run leads with the CHAPTER RANGE and drops the arc's name to the
 * line under it: a reader knows "chapters 0 to 3" and has never heard of
 * "Hour of an Awakening". Only a mainline run does: a themed shelf leads with
 * its own NAME, and any mainline chapters it happens to hold go in the muted
 * line under it, never in the heading.
 */
export function SectionHead({ chip, count, action }: { chip: IChipModel | undefined; count: string; action?: React.ReactNode }): React.ReactElement | null {
    const t: BrowseT = useT("story");
    if (!chip) return null;
    const heading = chip.range ? t("browse.section.mainWith", { range: rangeLabel(chip.range, t) }) : chip.name;
    const under = chip.range ? `${chip.name} · ${count}` : chip.includes ? `${includesLabel(chip.includes, t)} · ${count}` : count;
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
                    <h2 className="m-0 font-bold font-heading text-[13px] text-foreground uppercase tracking-[0.1em] sm:truncate">{heading}</h2>
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
 * The sticky jump bar. Above 640 px it is a scroller of two-line chips that
 * fades at whichever edge still has chips behind it and keeps the active chip
 * centred; under it a native select, because eleven two-line chips on a 375 px
 * phone is a scroller nobody discovers.
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
                    {chips.map((chip) => (
                        <option key={chip.id} value={chip.id}>
                            {chip.range ? `${rangeLabel(chip.range, t)} · ${chip.name}` : chip.includes ? `${chip.name} · ${includesLabel(chip.includes, t)}` : chip.name} ({chip.count})
                        </option>
                    ))}
                </select>
            </label>
            <div ref={scroller} onScroll={measure} className="msv-scroll -my-1 hidden max-w-full gap-1.5 overflow-x-auto py-1 sm:flex" style={{ maskImage: fade, WebkitMaskImage: fade }}>
                {chips.map((chip) => {
                    const on = active === chip.id;
                    return (
                        <a
                            key={chip.id}
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
                                <span className="whitespace-nowrap font-sans font-semibold text-[12px]">{chip.range ? rangeLabel(chip.range, t) : chip.name}</span>
                                {chip.range || chip.includes ? <span className={cn("mt-0.5 whitespace-nowrap font-mono text-[9px] uppercase tracking-[0.1em]", on ? "opacity-80" : "opacity-75")}>{chip.range ? chip.name : chip.includes ? includesLabel(chip.includes, t) : null}</span> : null}
                            </span>
                            <span className={cn("shrink-0 font-mono text-[10px] tabular-nums", on ? "opacity-80" : "text-muted-foreground")}>{chip.count}</span>
                        </a>
                    );
                })}
            </div>
        </nav>
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
