import { ChevronRightIcon, PauseIcon, PlayIcon } from "lucide-react";
import type React from "react";
import { memo, useMemo } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
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
 * One neutral pill for every kind. Tinting it per kind or in the cover's
 * sampled ink made legibility depend on the art (MAIN STORY on Stormwatch's
 * #4a4a26).
 */
export function KindBadge({ kind, className }: { kind: StoryKind; className?: string }): React.ReactElement {
    const t: BrowseT = useT("story");
    return <span className={cn(styles.kindBadge, className)}>{t(`browse.badge.${kind}`)}</span>;
}

/**
 * The bookmark at legend size, so the read-state pills double as the legend
 * for the ticket's tab colours. Decorative: the pill's label says the same.
 */
export function ReadMark({ state, className }: { state: ReadFilter; className?: string }): React.ReactElement | null {
    // An UNREAD ticket wears no tab, so its pill wears none either: a grey tab
    // on the legend would name a mark no card ever shows.
    if (state === "any" || state === "unread") return null;
    return <span aria-hidden="true" className={cn(styles.mark, state === "progress" && styles.markProgress, state === "done" && styles.markDone, className)} />;
}

/** `auto-fill, minmax(200px, 1fr)` on whole pixels; the outer box is the container `.grid` reads its column count from. */
export function GroupGrid({ children }: { children: React.ReactNode }): React.ReactElement {
    return (
        <div className={styles.shelf}>
            <div className={styles.grid}>{children}</div>
        </div>
    );
}

/** The page header's class for the whole-pixel title line; see `.shelfHead`. */
export const SHELF_HEAD = styles.shelfHead;

export function GroupRowList({ children }: { children: React.ReactNode }): React.ReactElement {
    return <div className={styles.rows}>{children}</div>;
}

/**
 * The card takes its fraction instead of `progress` and `gameRead`: the browse
 * page already computes it per progress change for the read filter and sorts,
 * and stable props let `memo` skip cards a keystroke or section change left alone.
 */
export interface IGroupCardProps {
    group: LibGroup;
    fraction: IReadFraction;
    /** Must be stable (a state setter or a `useCallback`), or `memo` never skips. */
    onOpen: (groupId: string) => void;
    /** Position in its section, for the reveal stagger. */
    index?: number;
}

/** Row inks go on the button because the row's left rule reads `--ticket-c1`; the ticket's live on `InkDot` / `InkFill`. */
function inkVars(palette: ITicketPalette, index: number): React.CSSProperties {
    return { "--ticket-c1": palette.c1, "--ticket-c2": palette.c2, "--i": index } as React.CSSProperties;
}

/**
 * Plays the chapter theme from the shelf (86 of 87 EN groups have one); the
 * bar at the foot of the page controls it after that. A sibling of the card
 * button since nested buttons are invalid, so no stopPropagation is needed.
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
 * The ticket's inks live on the two leaves that draw them, each reading the
 * palette itself. On the button the inherited property restyled all ~39
 * elements per card (one 3,374-element recalc, 89.1 ms at 1440, dev), and a
 * palette held by the card re-rendered all 85 cards in one commit (49.4-54.4 ms
 * in the harness, 168.7 ms under the DevTools MCP, dev).
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

/** Own component so the fit hooks (media query, canvas measure, state updates) run on the 6 logotype-less cards, not all 87. */
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
 * One chapter as a ticket. Silhouette, 79% tear, notches, stub tear-away and
 * shadow are the reference page's; the contents copy the game's
 * `stage_mix_story_overall_group_item_view` (`ui/[uc]mixstory.ab`): key visual
 * cover-fitted at 113%, 516x260 logotype on top. Dark in both themes.
 */
export const GroupCard = memo(function GroupCard({ group, fraction, onOpen, index = 0 }: IGroupCardProps): React.ReactElement {
    const t: BrowseT = useT("story");
    const { wpm } = useReadingSpeed();
    // Key visual first, cover as fallback, and the inks sample the same pick:
    // a chapter that opens on a fade derives a black cover with no colour in it.
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
                    {/* role="img" + the tooltip give the colour a name; the card's aria-label
                        already carries the fraction, so no extra tab stop. */}
                    {pct > 0 ? (
                        <Tooltip>
                            <TooltipTrigger render={<span role="img" aria-label={t("chapter.readFraction", { read: fraction.read, total: fraction.total })} className={cn(styles.bookmark, fraction.done && styles.bookmarkDone)} />} />
                            <TooltipPopup>{t("chapter.readFraction", { read: fraction.read, total: fraction.total })}</TooltipPopup>
                        </Tooltip>
                    ) : null}
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
 * Main story prints its chapter number, an event its code and year. Reading
 * time (the reader's wpm over the group's word count) is appended when the wire
 * has a count, so card and sheet quote the same span.
 */
function specParts(group: LibGroup, t: BrowseT, wpm: number): string[] {
    const model = specModel(group, cardCode(group));
    const entries = t("browse.card.entries", { count: model.entries });
    const parts = model.kind === "chapter" ? [t("browse.card.chapter", { n: model.chapter }), entries] : model.year === null ? [model.code, entries] : [model.code, String(model.year), entries];
    const words = groupWords(group);
    if (words !== null && words > 0) parts.push(humanTime(minutesFor(words, wpm)));
    return parts;
}

/** One chapter as a list row. Title is the name, never the logotype: a 516x260 plate in an 18.9 px line is unreadable. */
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
