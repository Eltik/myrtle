/**
 * THE JUMP BAR'S BEHAVIOUR, kept apart from its markup in `BrowseSections.tsx`.
 *
 * - `useScrollSpy`: which section the reader is in.
 * - `useJumpHold`: the chip a click lights, held while the page travels to it.
 * - `useRailSlide`: the rail sliding to centre the lit chip as it expands.
 * - `useRailEdges`: which edges of the rail have chips scrolled behind them.
 *
 * A chip is any element in the rail carrying `data-chip="<section id>"`; the
 * section it names is the element with that id.
 */

import type React from "react";
import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { scrollBehaviorFor } from "./toolbar";

const CHIP_SELECTOR = "[data-chip]";

/** How long a section change takes to slide the rail. */
const SLIDE_MS = 260;

/** The longest a click holds its chip lit, for a browser without `scrollend`. */
const HOLD_MAX_MS = 3000;

/** How long after the page stops the hold waits for the spy before letting go anyway. */
const HOLD_GRACE_MS = 300;

/** Sub-pixel differences in a width or a scroll offset are not movement. */
const EPSILON_PX = 0.5;

/** How far the rail must be scrolled from an edge before that edge fades. */
const EDGE_SLACK_PX = 2;

/** `smooth`, or instant for a reader whose system asks for reduced motion. Read at the moment of the jump, so a setting changed mid-visit is honoured. */
export function jumpBehavior(): ScrollBehavior {
    return scrollBehaviorFor(typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches);
}

const easeOutCubic = (t: number): number => 1 - (1 - t) ** 3;

function chipsIn(rail: HTMLElement): HTMLElement[] {
    return [...rail.querySelectorAll<HTMLElement>(CHIP_SELECTOR)];
}

function chipId(chip: HTMLElement): string {
    return chip.dataset.chip ?? "";
}

/**
 * The rail's `scrollLeft` that centres the chip for `id`, clamped to what the
 * rail can scroll, or the start when nothing is active. Above the first
 * section nothing is, and the row goes back to 0; leaving it where the last
 * active chip had centred it put the first chip at -390 under the mask at 768.
 */
function centredLeft(rail: HTMLElement, id: string | null): number {
    const chip = id ? rail.querySelector<HTMLElement>(`[data-chip="${CSS.escape(id)}"]`) : null;
    if (!chip) return 0;
    const left = chip.offsetLeft - (rail.clientWidth - chip.offsetWidth) / 2;
    return Math.max(0, Math.min(rail.scrollWidth - rail.clientWidth, left));
}

function unpin(chip: HTMLElement): void {
    chip.style.width = "";
    chip.style.overflow = "";
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
    // The last id handed to React. `pick` runs on every scroll event, and a
    // setter called with an unchanged value still costs a bail-out render once
    // the hook has updated before, so the spy only calls it on a change.
    const last = useRef<string | null>(null);

    useEffect(() => {
        if (typeof IntersectionObserver === "undefined") return;
        const list = idsKey === "" ? [] : idsKey.split("|");
        visible.current = new Set();
        const atBottom = () => window.scrollY + window.innerHeight >= document.documentElement.scrollHeight - 4;
        const set = (id: string | null) => {
            if (last.current === id) return;
            last.current = id;
            setActive(id);
        };
        const pick = () => {
            if (atBottom() && list.length > 0) {
                set(list[list.length - 1] ?? null);
                return;
            }
            // ABOVE THE FIRST SECTION THE FIRST SECTION IS CURRENT. The spy used
            // to keep whichever section it last saw, so a reader who scrolled back
            // to the head found the phone's section pill naming a shelf far down
            // the page ("The Blessed" over the top of Act 0). A first cut made
            // "nothing" current up there, and that FLAPPED: the 2026-09-25
            // recording shows the first chip expanding and collapsing every few
            // frames as the heading crossed the band's lower edge (40% down the
            // viewport, the `-60%` below), each flip re-laying out the whole row.
            // The first section is the right answer on both sides of that line,
            // so nothing changes when it is crossed. Between headings, mid-page,
            // the last heading that passed the band stays current, which is the
            // section the reader is in.
            const head = list[0] ? document.getElementById(list[0]) : null;
            if (head && head.getBoundingClientRect().top >= window.innerHeight * 0.4) {
                set(list[0] ?? null);
                return;
            }
            const first = list.find((id) => visible.current.has(id));
            if (first) set(first);
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

/**
 * The lit chip, given the spy's answer, and `jump` to send the page to a
 * section.
 *
 * A JUMP IS SMOOTH, AND ITS CHIP IS HELD LIT UNTIL THE PAGE ARRIVES. The
 * scroll crosses every section between here and there, and following the spy
 * meanwhile expanded and collapsed each chip on the way. The hold ends:
 *
 * - when the spy names the held section, which is the normal arrival;
 * - HOLD_GRACE_MS after the page stops, for a section the spy never reaches;
 * - on the reader's own wheel or touch;
 * - after HOLD_MAX_MS whatever happens. A shorter 1.5 s was outrun by a jump
 *   across the long acts, and the bar lit the act being passed.
 *
 * THE HOLD NEVER ENDS AT A `scrollend` ITSELF. Chrome fires one as a smooth
 * scroll STARTS, at the old scrollY and before any scroll event (213 ms after
 * the click, measured), so only a `scrollend` after the page has moved counts.
 * And at the real `scrollend` the spy's IntersectionObserver has not reported
 * yet, so it still names the old section; releasing there lit the old chip for
 * a frame, and the clicked chip flickered on every jump.
 *
 * The hash is REPLACED rather than pushed, through the router's own history
 * state: Back leaves the page instead of walking its sections.
 */
export function useJumpHold(spied: string | null): { active: string | null; jump: (id: string) => void } {
    const [held, setHeld] = useState<string | null>(null);

    useEffect(() => {
        if (held !== null && spied === held) setHeld(null);
    }, [held, spied]);

    useEffect(() => {
        if (held === null) return;
        const release = () => setHeld(null);
        let moved = false;
        let grace = 0;
        const onScroll = () => {
            moved = true;
        };
        const onScrollEnd = () => {
            if (!moved) return;
            window.clearTimeout(grace);
            grace = window.setTimeout(release, HOLD_GRACE_MS);
        };
        const timeout = window.setTimeout(release, HOLD_MAX_MS);
        window.addEventListener("scroll", onScroll, { passive: true });
        window.addEventListener("scrollend", onScrollEnd);
        window.addEventListener("wheel", release, { once: true, passive: true });
        window.addEventListener("touchstart", release, { once: true, passive: true });
        return () => {
            window.clearTimeout(timeout);
            window.clearTimeout(grace);
            window.removeEventListener("scroll", onScroll);
            window.removeEventListener("scrollend", onScrollEnd);
            window.removeEventListener("wheel", release);
            window.removeEventListener("touchstart", release);
        };
    }, [held]);

    const jump = useCallback((id: string) => {
        const section = document.getElementById(id);
        if (!section) return;
        const behavior = jumpBehavior();
        window.history.replaceState(window.history.state, "", `#${id}`);
        if (behavior === "smooth") setHeld(id);
        section.scrollIntoView({ behavior, block: "start" });
    }, []);

    return { active: held ?? spied, jump };
}

/**
 * Centres the `active` chip in the rail, sliding there on a change.
 *
 * ONE CLOCK. The chip that collapses and the chip that expands tween their
 * widths while the rail tweens its `scrollLeft`, all written in the same frame.
 * Run as a WAAPI width animation beside a native smooth scroll, the two curves
 * disagreed and a second scroll took up what the first was clamped short of:
 * the active chip swam and then settled.
 *
 * A layout effect, so a chip is pinned to its old width before the first frame
 * paints the new one. The final widths and the centring target are read from
 * the final layout before anything is pinned; a change mid-slide starts from
 * where the slide is. Under reduced motion the rail moves instantly.
 *
 * THE RAIL IGNORES THE POINTER WHILE IT SLIDES. A click centres the chip it
 * hit, which on the wide act banners moves the row a long way under a still
 * cursor, and every chip that passed under it lit its hover and its tooltip.
 *
 * `scrollLeft` is written directly, never through `scrollIntoView`: that walks
 * every scrollable ancestor and would drag the PAGE to the section the reader
 * has not asked for yet.
 */
export function useRailSlide(rail: React.RefObject<HTMLElement | null>, active: string | null): void {
    // Every chip's width as of the last commit: the width a chip slides FROM.
    const widths = useRef(new Map<string, number>());
    const frame = useRef(0);
    const activeRef = useRef(active);

    const record = useCallback(() => {
        const node = rail.current;
        if (!node) return;
        for (const chip of chipsIn(node)) widths.current.set(chipId(chip), chip.getBoundingClientRect().width);
    }, [rail]);

    useLayoutEffect(() => {
        activeRef.current = active;
        const node = rail.current;
        if (!node) return;
        const chips = chipsIn(node);
        const froms = chips.map((chip) => (chip.style.width ? chip.getBoundingClientRect().width : widths.current.get(chipId(chip))));
        cancelAnimationFrame(frame.current);
        node.style.pointerEvents = "";
        for (const chip of chips) unpin(chip);
        record();
        const left = centredLeft(node, active);
        if (jumpBehavior() !== "smooth") {
            node.scrollLeft = left;
            return;
        }

        const moves = chips.flatMap((chip, i) => {
            const from = froms[i];
            const to = widths.current.get(chipId(chip));
            return from === undefined || to === undefined || Math.abs(to - from) < EPSILON_PX ? [] : [{ chip, from, to }];
        });
        const start = node.scrollLeft;
        if (moves.length === 0 && Math.abs(start - left) < EPSILON_PX) return;

        const step = (progress: number) => {
            for (const { chip, from, to } of moves) chip.style.width = `${from + (to - from) * progress}px`;
            node.scrollLeft = start + (left - start) * progress;
        };
        for (const { chip } of moves) chip.style.overflow = "hidden";
        node.style.pointerEvents = "none";
        step(0);
        const began = performance.now();
        const tick = (now: number) => {
            const t = Math.min(1, (now - began) / SLIDE_MS);
            if (t < 1) {
                step(easeOutCubic(t));
                frame.current = requestAnimationFrame(tick);
                return;
            }
            for (const { chip } of moves) unpin(chip);
            node.scrollLeft = left;
            node.style.pointerEvents = "";
        };
        frame.current = requestAnimationFrame(tick);
    }, [active, rail, record]);

    useEffect(() => () => cancelAnimationFrame(frame.current), []);

    // The webfonts and a resize move every chip, so the widths are re-read.
    // A RELOAD ALSO RESTORES THE RAIL'S OLD SCROLLLEFT after the layout effect
    // has run: Chrome's history restoration puts nested scrollers back too, so
    // a reload at the top of the page came up at 406 with nothing active. The
    // rail is re-centred, instantly, once the document has loaded and on every
    // `pageshow` (the back-forward cache).
    useEffect(() => {
        const settle = () =>
            requestAnimationFrame(() => {
                const node = rail.current;
                if (!node) return;
                record();
                node.scrollLeft = centredLeft(node, activeRef.current);
            });
        if (document.readyState === "complete") settle();
        else window.addEventListener("load", settle, { once: true });
        window.addEventListener("pageshow", settle);
        window.addEventListener("resize", record);
        return () => {
            window.removeEventListener("load", settle);
            window.removeEventListener("pageshow", settle);
            window.removeEventListener("resize", record);
        };
    }, [rail, record]);
}

/**
 * Whether the rail has chips scrolled out of view at its start and its end,
 * and `measure`, for the rail's `onScroll`.
 *
 * The rail fires a scroll event per frame while it slides, and a fresh object
 * each time re-rendered the bar per frame for two booleans that had not
 * changed; the previous state is kept unless one flips.
 */
export function useRailEdges(rail: React.RefObject<HTMLElement | null>): { edges: { start: boolean; end: boolean }; measure: () => void } {
    const [edges, setEdges] = useState({ start: false, end: false });

    const measure = useCallback(() => {
        const node = rail.current;
        if (!node) return;
        const start = node.scrollLeft > EDGE_SLACK_PX;
        const end = node.scrollLeft + node.clientWidth < node.scrollWidth - EDGE_SLACK_PX;
        setEdges((prev) => (prev.start === start && prev.end === end ? prev : { start, end }));
    }, [rail]);

    useEffect(() => {
        measure();
        window.addEventListener("resize", measure);
        return () => window.removeEventListener("resize", measure);
    }, [measure]);

    return { edges, measure };
}
