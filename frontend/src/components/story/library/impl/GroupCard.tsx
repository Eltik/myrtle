import { ChevronRightIcon, PauseIcon, PlayIcon } from "lucide-react";
import type React from "react";
import { memo, useMemo } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { useMediaQuery } from "#/hooks/use-media-query";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { humanTime, minutesFor, useReadingSpeed } from "#/lib/story/reading";
import { cn } from "#/lib/utils";
import type { messages as archiveMessages } from "./Archive.messages";
import { plateCrop, plateSource, titleSource } from "./art";
import type { messages } from "./Browse.messages";
import { chapterNumberOf, specModel } from "./chapters";
import { groupWords, type IReadFraction, type LibGroup, type ReadFilter } from "./derive";
import styles from "./GroupCard.module.css";
import { FALLBACK_PALETTE, type ITicketPalette, useTicketPalette } from "./palette";
import { themeTrack, toggle as togglePlayer, useIsSounding } from "./player";
import { cardCode, kindOf, type StoryKind } from "./sections";
import { TICKET_FIT, TICKET_FIT_PHONE, useTitleFit } from "./titleFit";

type BrowseT = TypedT<typeof messages>;
type ThemeT = TypedT<typeof archiveMessages>;

/**
 * ONE NEUTRAL PILL FOR EVERY KIND, and that is a correction, not a preference.
 * The badge used to be tinted per kind on themed surfaces and printed in the
 * cover's own sampled ink on a ticket, so a card's most legible label changed
 * with its art: Stormwatch's c1 came back #4a4a26, and MAIN STORY on it was
 * the reviewer's "poor readability". A label that names a taxonomy has no
 * business carrying a per-card colour, so it is now black on white in light
 * mode and white on near-black in dark mode, which are the two highest
 * contrast ratios the page has.
 */
export function KindBadge({ kind, className }: { kind: StoryKind; className?: string }): React.ReactElement {
    const t: BrowseT = useT("story");
    return <span className={cn(styles.kindBadge, className)}>{t(`browse.badge.${kind}`)}</span>;
}

/**
 * The bookmark at legend size, for the read-state filter pills.
 *
 * It is the one place the ticket's colour code is spelled out: the pills name
 * the three states in words and wear the tab in each state's own ink, so a
 * reader who wonders what an amber tab means finds the answer in the control
 * they are already using. Decorative here, because the pill's own label says
 * the same thing.
 */
export function ReadMark({ state, className }: { state: ReadFilter; className?: string }): React.ReactElement | null {
    // An UNREAD ticket wears no tab, so its pill wears none either: a grey tab
    // on the legend would name a mark no card ever shows.
    if (state === "any" || state === "unread") return null;
    return <span aria-hidden="true" className={cn(styles.mark, state === "progress" && styles.markProgress, state === "done" && styles.markDone, className)} />;
}

/** The card grid. `auto-fill, minmax(200px, 1fr)` at a 12 px gap, which lands six 223.3 x 121.5 tickets across the 1,400 px column at 1440; under 640 the track is 160 px, two across a phone. */
export function GroupGrid({ children }: { children: React.ReactNode }): React.ReactElement {
    return <div className={styles.grid}>{children}</div>;
}

/** The list-mode stack. */
export function GroupRowList({ children }: { children: React.ReactNode }): React.ReactElement {
    return <div className={styles.rows}>{children}</div>;
}

/**
 * THE CARD RECEIVES ITS FRACTION, it does not walk its stories for one. The
 * browse page already computes every group's fraction once per progress change
 * (the read filter and the read sorts need it), and a card that took
 * `progress` and `gameRead` instead recomputed it on every render. The props
 * are all stable between renders that change nothing on the card, which is
 * what lets `memo` skip it: a section change or a keystroke that leaves a card
 * in place no longer re-renders it.
 */
export interface IGroupCardProps {
    group: LibGroup;
    fraction: IReadFraction;
    /** Must be stable (a state setter or a `useCallback`), or `memo` never skips. */
    onOpen: (groupId: string) => void;
    /** Position in its section, for the reveal stagger. */
    index?: number;
}

/**
 * The inks, as inline custom properties the module's rules read.
 *
 * ONLY TWO THINGS ARE INKED NOW. The tape band that used to print all three
 * across the top of every card is gone: it was the reviewer's "palettes at the
 * top of every card", and it was chrome carrying a swatch rather than
 * information. What survives is the spec row's DOT and the progress fill,
 * which the reviewer liked, and nothing else on the card reads `--ticket-c*`.
 */
// The list row carries its inks on the button, because the row's own left
// rule reads `--ticket-c1`. The ticket does not: see `InkDot`.
function inkVars(palette: ITicketPalette, index: number): React.CSSProperties {
    return { "--ticket-c1": palette.c1, "--ticket-c2": palette.c2, "--i": index } as React.CSSProperties;
}

/**
 * THE CHAPTER'S THEME, ONE PRESS FROM THE SHELF.
 *
 * 86 of the 87 EN groups carry a theme and until now the only way to hear one
 * was to open its sheet, which is two presses and a dialog to start a piece of
 * music. The glyph starts it in place; the bar at the foot of the page is
 * where it is then governed, so this control does one thing and shows one
 * state.
 *
 * It is a SIBLING of the card's button, positioned over it by the wrapper,
 * because a nested button is invalid markup. `stopPropagation` is not enough
 * on its own here and is not used: the two controls never overlap in the DOM,
 * so a press on the glyph was never a press on the card.
 *
 * Memoised: its props are the group and a class name, and a card re-rendering
 * for its palette or its fraction has nothing new to tell it.
 */
const ThemeGlyph = memo(function ThemeGlyph({ group, className }: { group: LibGroup; className: string }): React.ReactElement | null {
    const t: ThemeT = useT("story");
    const track = themeTrack(group);
    const sounding = useIsSounding(track?.key ?? null);
    if (track === null) return null;
    const name = track.title ?? t("archive.theme.label");

    return (
        <button type="button" className={className} data-sounding={sounding} onClick={() => togglePlayer(track)} aria-label={sounding ? t("archive.theme.pause", { name }) : t("archive.theme.play", { name })}>
            {sounding ? <PauseIcon className="size-4" aria-hidden="true" /> : <PlayIcon className="size-4" aria-hidden="true" />}
        </button>
    );
});

/**
 * THE TICKET'S INKS LIVE ON THE TWO LEAVES THAT DRAW THEM, and each leaf reads
 * the palette itself.
 *
 * On the button, a custom property is inherited, so a palette landing there
 * restyled all ~39 elements of the card: on load that was one 3,374-element
 * style recalc (89.1 ms at 1440, dev) for 85 dots and 85 fills. And the card
 * held the palette state, so its arrival re-rendered all 85 cards in one
 * commit (49.4 to 54.4 ms in the harness, 168.7 ms under the DevTools MCP,
 * dev). Now the arrival re-renders these two spans per card and restyles two
 * elements. The DOM, the variables and the rules that read them are the
 * same; a card with no art never samples and keeps the fallback ink.
 */
function InkDot({ art }: { art: string | null }): React.ReactElement {
    const palette = useTicketPalette(art);
    return <span aria-hidden="true" className={styles.dot} style={{ "--ticket-c2": palette.c2 } as React.CSSProperties} />;
}

function InkFill({ art, pct }: { art: string | null; pct: number }): React.ReactElement {
    const palette = useTicketPalette(art);
    return <span className={styles.progressFill} style={{ width: `${pct}%`, "--ticket-c1": palette.c1 } as React.CSSProperties} />;
}

/** The stub's blank face, drawn only on a card with no art, whose palette is always the fallback. */
const NO_ART_INK = { "--ticket-c1": FALLBACK_PALETTE.c1 } as React.CSSProperties;

/**
 * The set name, for the 6 groups with no logotype. It is its OWN component so
 * the measurement hooks run on those 6 cards only: in the card they ran on all
 * 87, and each of the 81 logotype cards paid a media-query subscription, a
 * canvas measurement and a state update (two, once the webfont landed) for a
 * title it never draws.
 */
function FittedTitle({ name }: { name: string }): React.ReactElement {
    const phone = useMediaQuery("max-sm");
    const fit = useTitleFit(name, phone ? TICKET_FIT_PHONE : TICKET_FIT);
    return (
        <span className={styles.title} style={{ "--title-size": fit.size, "--title-wrap": fit.breakAnywhere ? "anywhere" : "normal" } as React.CSSProperties}>
            {fit.lines.map((line) => (
                <span key={line} className={styles.titleLine}>
                    {line}
                </span>
            ))}
        </span>
    );
}

/**
 * One chapter as a ticket. The silhouette, the tear at 79%, the notches, the
 * stub's hover tear-away and the drop shadow are the reference page's and are
 * untouched; what is inside them is now the GAME's card.
 *
 * `ui/[uc]mixstory.ab` draws a story set as its KEY VISUAL cover-fitted over
 * the whole card (`stage_mix_story_overall_group_item_view`: a 282x204 image
 * on a 240x180 card, centred, which is 113.0% of the size that would just
 * cover) with the authored 516x260 TITLE LOGOTYPE contained on top of it
 * (197x103, 82.1% of the card's width and 57.2% of its height). So the card is
 * a picture with a logotype on it, and that is what this is.
 *
 * The ticket does not follow the site theme. It is dark art in light mode and
 * in dark mode alike; only the page around it, and the badge, change.
 */
export const GroupCard = memo(function GroupCard({ group, fraction, onOpen, index = 0 }: IGroupCardProps): React.ReactElement {
    const t: BrowseT = useT("story");
    const { wpm } = useReadingSpeed();
    // The KEY VISUAL is the picture, and the cover is only its fallback. The
    // background, the stub and the inks read the SAME choice: a chapter that
    // opens on a fade derives a black cover, and sampling that gives a card
    // with no colour in it at all.
    const plate = plateSource(group);
    const art = plate.kind === "none" ? null : asset(plate.url);
    const title = titleSource(group);
    const pct = fraction.total > 0 ? Math.round((fraction.read / fraction.total) * 100) : 0;
    const spec = specParts(group, t, wpm);
    const stagger = useMemo(() => ({ "--i": index }) as React.CSSProperties, [index]);

    return (
        <div className={styles.cardWrap}>
            <button type="button" onClick={() => onOpen(group.id)} aria-label={t("browse.card.openWithProgress", { name: group.name, read: fraction.read, total: fraction.total })} className={styles.card} style={stagger}>
                <span className={styles.ticket}>
                    <span className={cn(styles.half, styles.body)}>
                        {art ? <img src={art} alt="" crossOrigin="anonymous" data-source={plate.kind} loading="lazy" decoding="async" className={styles.kv} style={{ objectPosition: plateCrop(group) }} /> : null}
                        <span aria-hidden="true" className={styles.scrim} />
                        <span className={styles.content}>
                            <span className={styles.badge}>{t(`browse.badge.${kindOf(group)}`)}</span>
                            {title.kind === "logotype" ? <img src={asset(title.url)} alt={group.name} loading="lazy" decoding="async" className={styles.logotype} /> : <FittedTitle name={group.name} />}
                        </span>
                        <span aria-hidden="true" className={styles.progress}>
                            <InkFill art={art} pct={pct} />
                        </span>
                    </span>
                    <span aria-hidden="true" className={styles.stub}>
                        <span className={cn(styles.half, styles.stubFace)}>{art ? <img src={art} alt="" crossOrigin="anonymous" loading="lazy" decoding="async" className={styles.stubImage} /> : <span className={styles.stubBlank} style={NO_ART_INK} />}</span>
                    </span>
                    {/* THE BOOKMARK SAYS WHAT IT MEANS NOW. It is amber part-read and green
                        finished, and it carried neither a name nor a number, so the colour
                        was a code with no key. `role="img"` plus the fraction gives it one,
                        and the native `title` shows the same words on hover; the card's own
                        `aria-label` already carries the fraction, so this adds no tab stop
                        and no second announcement of the same number. */}
                    {pct > 0 ? <span role="img" aria-label={t("chapter.readFraction", { read: fraction.read, total: fraction.total })} title={t("chapter.readFraction", { read: fraction.read, total: fraction.total })} className={cn(styles.bookmark, fraction.done && styles.bookmarkDone)} /> : null}
                </span>
                <span className={styles.spec}>
                    <InkDot art={art} />
                    <span className={styles.specText}>
                        {spec.map((part, at) => (
                            <span key={part} className={styles.specField}>
                                {at < spec.length - 1 ? `${part} ·` : part}
                            </span>
                        ))}
                    </span>
                </span>
            </button>
            <ThemeGlyph group={group} className={styles.cardPlay} />
        </div>
    );
});

/**
 * The spec line under a card or a row title. A main story chapter prints its
 * NUMBER, which is the only handle a reader has on it; an event prints the
 * operation code it is known by plus the year it ran.
 *
 * The READING TIME closes the line when the wire counts the group's words,
 * because "39 entries" says how many things there are and nothing about how
 * long an evening it is. It is the reader's own words-per-minute setting
 * applied to the group's count, so a card and the sheet behind it always quote
 * the same span. A group with no count simply ends a field earlier.
 */
function specParts(group: LibGroup, t: BrowseT, wpm: number): string[] {
    const model = specModel(group, cardCode(group));
    const entries = t("browse.card.entries", { count: model.entries });
    const parts = model.kind === "chapter" ? [t("browse.card.chapter", { n: model.chapter }), entries] : model.year === null ? [model.code, entries] : [model.code, String(model.year), entries];
    const words = groupWords(group);
    if (words !== null && words > 0) parts.push(humanTime(minutesFor(words, wpm)));
    return parts;
}

/**
 * One chapter as a list row: a 172x92 key visual with the chapter mark burnt
 * into it, a left rule in the cover's dominant ink, badge, title, spec line,
 * the read fraction as a NUMBER, and a chevron.
 *
 * The row title is the group's NAME and never a logotype, because a 516x260
 * plate in an 18.9 px line box is unreadable. It is no longer clipped with an
 * ellipsis either: it wraps to two whole-word lines.
 */
export const GroupRow = memo(function GroupRow({ group, fraction, onOpen, index = 0 }: IGroupCardProps): React.ReactElement {
    const t: BrowseT = useT("story");
    const { wpm } = useReadingSpeed();
    const art = plateSource(group);
    const thumb = art.kind === "none" ? null : asset(art.url);
    const mark = group.iconUrl ? asset(group.iconUrl) : null;
    const palette = useTicketPalette(thumb);
    const pct = fraction.total > 0 ? Math.round((fraction.read / fraction.total) * 100) : 0;
    const code = cardCode(group);
    const chapter = chapterNumberOf(group);
    const inks = useMemo(() => inkVars(palette, index), [palette, index]);

    return (
        <div className={styles.rowWrap} data-music={group.music ? "" : undefined}>
            <button type="button" onClick={() => onOpen(group.id)} aria-label={t("browse.card.openWithProgress", { name: group.name, read: fraction.read, total: fraction.total })} className={styles.row} style={inks}>
                <span className={styles.rowThumb}>
                    {thumb ? <img src={thumb} alt="" crossOrigin="anonymous" loading="lazy" decoding="async" data-source={art.kind} /> : <span className={styles.rowBlank}>{mark ? <img src={mark} alt="" loading="lazy" decoding="async" className={styles.rowMark} /> : code}</span>}
                    {chapter !== null ? (
                        <span aria-hidden="true" className={styles.rowEp}>
                            {t("browse.card.chapterShort", { n: String(chapter).padStart(2, "0") })}
                        </span>
                    ) : null}
                </span>
                <span className={styles.rowText}>
                    <span className={styles.rowBadge}>{t(`browse.badge.${kindOf(group)}`)}</span>
                    <span className={styles.rowTitle}>{group.name}</span>
                    <span className={styles.rowSpec}>
                        <span aria-hidden="true" className={styles.dot} />
                        {specParts(group, t, wpm).join(" · ")}
                    </span>
                </span>
                {fraction.total > 0 ? (
                    <span aria-hidden="true" className={styles.rowMeter}>
                        <span className={styles.rowFraction}>{t("browse.row.fraction", { read: fraction.read, total: fraction.total })}</span>
                        {pct > 0 ? (
                            <span className={styles.rowBar}>
                                <span className={styles.rowBarFill} style={{ width: `${pct}%` }} />
                            </span>
                        ) : null}
                    </span>
                ) : null}
                <ChevronRightIcon aria-hidden="true" className={styles.rowChevron} />
            </button>
            <ThemeGlyph group={group} className={styles.rowPlay} />
        </div>
    );
});
