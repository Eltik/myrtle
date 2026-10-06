import { useState } from "react";

/** The gallery tile sizes: each one's smallest tile width, in CSS px. */
export const TILE_SIZES = { s: 180, m: 260, l: 340 } as const;
export type TileSize = keyof typeof TILE_SIZES;

const KEY = "myrtle:profile-background:tile-size";

function read(): TileSize {
    try {
        const raw = window.localStorage.getItem(KEY);
        return raw === "s" || raw === "m" || raw === "l" ? raw : "m";
    } catch {
        return "m";
    }
}

/**
 * The viewer's gallery tile size, remembered in this browser. M (260 px at least) is the
 * default: at a 2000 px window the results column holds six tiles of about 280 px.
 * Storage that throws or is empty reads as M.
 */
export function useTileSize(): [TileSize, (next: TileSize) => void] {
    const [size, setSize] = useState<TileSize>(read);
    const set = (next: TileSize) => {
        setSize(next);
        try {
            window.localStorage.setItem(KEY, next);
        } catch {
            // Blocked storage: the size holds for this visit.
        }
    };
    return [size, set];
}
