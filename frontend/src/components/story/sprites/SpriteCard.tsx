import { Link } from "@tanstack/react-router";
import type React from "react";
import { memo } from "react";
import { Body } from "#/components/story/reader/StageSprites";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { StorySpriteEntry } from "#/types/generated/StorySpriteEntry";
import type { StorySpriteVariant } from "#/types/generated/StorySpriteVariant";
import type { messages } from "./CharactersTab.messages";
import { aliasesOf, CARD_CROP, cropForVariant, type ICrop, type ICropWindow, primaryName } from "./gallery";
import { spriteThumbUrl } from "./thumb";

type SpritesT = TypedT<typeof messages>;

const WHOLE_PLATE: ICrop = { size: 100, left: 0, top: 0 };

/**
 * One expression drawn through the reader's own `Body` (body plate plus face
 * patch at `facePos`), or through the backend's composed `thumb` of the same
 * plate when one is given, inside a window cropped to the head (`crop`, or the
 * whole plate when null). The square plate is sized and offset in percent of
 * the frame, so the same markup serves a 150 px card and a 600 px preview, and
 * nothing is measured at runtime.
 */
export function SpriteFigure({ variant, name, crop, lazy, thumb, className }: { variant: StorySpriteVariant; name: string; crop: ICropWindow | null; lazy?: boolean; thumb?: string; className?: string }): React.ReactElement {
    const box = crop ? cropForVariant(variant, crop) : WHOLE_PLATE;
    return (
        <span className={cn("relative block overflow-hidden", className)}>
            <span className="absolute block aspect-square" style={{ width: `${box.size}%`, left: `${box.left}%`, top: `${box.top}%` }}>
                {thumb ? <img src={thumb} alt="" loading="lazy" decoding="async" draggable={false} className="absolute inset-0 size-full select-none" /> : <Body state={{ sprite: variant, name }} sec={0} mode="hold" lazy={lazy} />}
            </span>
        </span>
    );
}

export interface ISpriteCardProps {
    entry: StorySpriteEntry;
    /** Opens the sheet in place; the card is still a real link, so a modified click opens the page. */
    onOpen: (base: string) => void;
}

/** The props a card renders from, derived once: exported so a test can pin them without a DOM. */
export function cardView(entry: StorySpriteEntry): { name: string; aliases: number; stories: number; kind: StorySpriteEntry["kind"]; thumbKey: string | null; base: string } {
    return { name: primaryName(entry), aliases: aliasesOf(entry).length, stories: entry.storyCount, kind: entry.kind, thumbKey: entry.thumb?.key ?? null, base: entry.base };
}

/**
 * One character in the grid. Memoised on its entry and the stable `onOpen`,
 * so a keystroke that keeps a card on screen does not re-render it; the thumb
 * is lazy and nothing on it samples a palette.
 */
export const SpriteCard = memo(function SpriteCard({ entry, onOpen }: ISpriteCardProps): React.ReactElement {
    const t: SpritesT = useT("story");
    const view = cardView(entry);
    const server = useGamedataServer();
    return (
        <Link
            to="/stories"
            search={{ tab: "characters", sprite: entry.base }}
            onClick={(e) => {
                if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
                e.preventDefault();
                onOpen(entry.base);
            }}
            aria-label={t("sprites.card.open", { name: view.name })}
            className="group flex min-w-0 flex-col overflow-hidden rounded-xl border border-border bg-card text-left no-underline ring-inset transition-colors hover:border-primary/45 focus-visible:ring-2 focus-visible:ring-ring/60"
        >
            {entry.thumb ? (
                <SpriteFigure variant={entry.thumb} name={entry.base} crop={CARD_CROP} lazy thumb={spriteThumbUrl(entry.base, entry.thumb.key, server)} className="aspect-3/4 w-full bg-secondary/55" />
            ) : (
                <span className="flex aspect-3/4 w-full items-center justify-center bg-secondary/30 font-mono text-[10px] text-muted-foreground/70">{entry.base}</span>
            )}
            <span className="flex min-w-0 flex-col gap-0.5 px-2.5 pt-2 pb-2.5">
                <span className="truncate font-sans font-semibold text-[13px] text-foreground" title={view.name}>
                    {view.name}
                </span>
                <span className="flex min-w-0 flex-wrap items-center gap-x-1.5 gap-y-0.5 font-mono text-[10px] text-muted-foreground tabular-nums">
                    <span className={cn("shrink-0 rounded-[4px] px-1 py-px font-sans font-semibold text-[9.5px] uppercase tracking-[0.04em]", view.kind === "operator" ? "bg-primary/15 text-primary" : "bg-secondary text-muted-foreground")}>{t(`sprites.badge.${view.kind}`)}</span>
                    <span className="whitespace-nowrap">{t("sprites.card.stories", { count: view.stories })}</span>
                    {view.aliases > 0 ? <span className="whitespace-nowrap">{t("sprites.card.aliases", { count: view.aliases })}</span> : null}
                </span>
            </span>
        </Link>
    );
});
