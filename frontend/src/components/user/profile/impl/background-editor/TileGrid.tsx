import { CheckIcon } from "lucide-react";
import { type KeyboardEvent, useEffect, useRef, useState } from "react";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { type GalleryKind, galleryThumbUrl } from "../background";
import { type IGalleryTile, pagesFor, visibleTiles } from "../gallery";
import type { messages } from "./ArtBrowser.messages";

interface ITileGridProps {
    tiles: readonly IGalleryTile[];
    /** The picture the draft shows, `null` when it is no gallery picture. */
    selected: { kind: GalleryKind; id: string } | null;
    onPick: (kind: GalleryKind, id: string) => void;
    /** What the tiles are (source, filter, search): a new key starts the paging over at one page. */
    pagingKey: string;
    /** The smallest tile width, in CSS px; the tiles grow from it to fill each row. */
    minTile: number;
}

/**
 * The gallery's grid of 16:9 tiles (the backend's 320 px JPEGs), as many columns as fit at
 * `minTile` px or more, so the tiles grow with the editor. It flows in the editor's one scroller
 * rather than scrolling itself. It mounts 120 tiles (`TILE_PAGE`) and adds a page as its
 * end comes within 400 px of view, so the 1,230 CGs never mount at once; on mount it
 * mounts as many pages as the selected tile needs, so the ringed tile is there to scroll
 * to, but it does not scroll the editor away from the preview to show it. Arrow keys move
 * between tiles by the rendered column count, Home and End to either end, and Enter or
 * Space pick.
 */
export function TileGrid({ tiles, selected, onPick, pagingKey, minTile }: ITileGridProps) {
    const t: TypedT<typeof messages> = useT("user");
    const [paging, setPaging] = useState({ key: "", pages: 1 });
    const [revealing, setRevealing] = useState(selected !== null);
    const selectedIndex = selected ? tiles.findIndex((tile) => tile.kind === selected.kind && tile.id === selected.id) : -1;
    const pages = paging.key === pagingKey ? paging.pages : revealing ? pagesFor(selectedIndex) : 1;
    const shown = visibleTiles(tiles, pages);

    useEffect(() => {
        if (!revealing) return;
        setRevealing(false);
        if (selectedIndex >= 0) setPaging({ key: pagingKey, pages });
    }, [revealing, selectedIndex, pagingKey, pages]);

    const sentinel = useRef<HTMLLIElement>(null);
    useEffect(() => {
        const node = sentinel.current;
        if (!node || shown.length >= tiles.length) return;
        const observer = new IntersectionObserver((entries) => entries.some((e) => e.isIntersecting) && setPaging({ key: pagingKey, pages: pages + 1 }), { rootMargin: "400px" });
        observer.observe(node);
        return () => observer.disconnect();
    }, [shown.length, tiles.length, pagingKey, pages]);

    return (
        <ul aria-label={t("profile.background.gallery.grid")} onKeyDown={moveFocus} style={{ gridTemplateColumns: `repeat(auto-fill, minmax(min(${minTile}px, 100%), 1fr))` }} className="m-0 grid list-none gap-2.5 p-0 max-sm:grid-cols-2! max-sm:gap-2">
            {shown.map((tile) => {
                const isSelected = selected !== null && tile.kind === selected.kind && tile.id === selected.id;
                const title = tile.title || tile.groupName;
                const label = tile.title ? t("profile.background.gallery.tileLabel", { title, group: tile.groupName }) : t("profile.background.gallery.storyTileLabel", { group: tile.groupName });
                return (
                    <li key={tile.id}>
                        <button
                            type="button"
                            data-tile=""
                            onClick={() => onPick(tile.kind, tile.id)}
                            aria-label={label}
                            aria-pressed={isSelected}
                            title={label}
                            className={cn(
                                "group flex w-full cursor-pointer scroll-mt-48 scroll-mb-8 flex-col overflow-hidden rounded-lg border bg-card text-start outline-none transition-colors hover:border-primary/70 focus-visible:ring-2 focus-visible:ring-ring",
                                isSelected ? "border-primary ring-2 ring-primary/50" : "border-border",
                            )}
                        >
                            <span className="relative block aspect-video w-full overflow-hidden bg-[oklch(0.2_0.005_285)]">
                                <img src={galleryThumbUrl(tile.kind, tile.id)} alt="" loading="lazy" decoding="async" draggable={false} width={320} height={180} className="absolute inset-0 h-full w-full object-cover transition-transform duration-200 group-hover:scale-[1.03] motion-reduce:transition-none" />
                                {isSelected && (
                                    <span className="absolute top-1.5 right-1.5 inline-flex size-5 items-center justify-center rounded-full bg-primary text-primary-foreground shadow" aria-hidden="true">
                                        <CheckIcon className="size-3.5" strokeWidth={3} />
                                    </span>
                                )}
                            </span>
                            <span className={cn("line-clamp-1 px-2 py-1.5 font-medium font-sans text-[11.5px] leading-tight", isSelected ? "text-primary" : "text-muted-foreground group-hover:text-foreground")}>{title}</span>
                        </button>
                    </li>
                );
            })}
            {shown.length < tiles.length && <li ref={sentinel} aria-hidden="true" className="col-span-full h-px" />}
        </ul>
    );
}

/** The tile-grid keys that move within a row or to either end; Up and Down step by the column count. */
const ROW_STEPS: Record<string, number | "start" | "end"> = { ArrowLeft: -1, ArrowRight: 1, Home: "start", End: "end" };

/** Arrow keys between the tiles by the rendered column count; Home and End jump to the first and last mounted tile. */
function moveFocus(e: KeyboardEvent<HTMLUListElement>) {
    const grid = e.currentTarget;
    const columns = getComputedStyle(grid).gridTemplateColumns.split(" ").filter(Boolean).length || 1;
    const step = e.key === "ArrowUp" ? -columns : e.key === "ArrowDown" ? columns : ROW_STEPS[e.key];
    if (step === undefined) return;
    const buttons = [...grid.querySelectorAll<HTMLButtonElement>("button[data-tile]")];
    const at = buttons.indexOf(document.activeElement as HTMLButtonElement);
    if (at < 0) return;
    const next = step === "start" ? 0 : step === "end" ? buttons.length - 1 : at + step;
    const target = buttons[next];
    if (!target) return;
    e.preventDefault();
    target.focus({ preventScroll: true });
    // The tiles' scroll margins keep a tile clear of the sticky filter bar above and the fade below.
    target.scrollIntoView({ block: "nearest" });
}
