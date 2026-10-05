/**
 * A grid's PNG, as data. Built in two places from the same `Grid` document:
 * on the page, by the view route's `head()`, to hash the image into its
 * `og:image` URL; and on the server, by the `grid-image` OG handler, when the
 * URL is requested and the PNG is not cached. Both go through
 * {@link buildGridImageData}, so the hash and the picture read the same cells.
 */

import { DEFAULT_GAMEDATA_SERVER, type GamedataServer, isGamedataServer, resolveGamedataServer } from "#/lib/api/gamedata";
import { type ArtFit, entityArtFit, entityIconURL, entityKey, toTierEntity, UNPLACED } from "#/lib/api/tier-entities";
import type { Grid } from "#/types/generated/Grid";

export interface IGridImageCell {
    label: string;
    /** `${kind}:${id}` of the pick, the hash's identity for it; `null` for an empty cell. */
    key: string | null;
    /** The pick's name, drawn as initials when it has no art. */
    name: string | null;
    artURL: string | null;
    fit: ArtFit;
}

export interface IGridImageData {
    title: string;
    slug: string;
    /** The server the cells were resolved against: the same pick can draw different art per server. */
    server: GamedataServer;
    rows: number;
    cols: number;
    /** Row-major, `rows * cols` long. */
    cells: IGridImageCell[];
}

/** `iconBase` is the backend origin the image is fetched from: the public one on the page, the internal one on the server. */
export function buildGridImageData(grid: Pick<Grid, "title" | "slug" | "rows" | "cols" | "cells">, iconBase: string, server?: string): IGridImageData {
    const cells: IGridImageCell[] = [];
    for (let i = 0; i < grid.rows * grid.cols; i++) {
        const cell = grid.cells[i];
        if (!cell?.entity_kind || !cell.entity_id) {
            cells.push({ label: cell?.label ?? "", key: null, name: null, artURL: null, fit: "cover" });
            continue;
        }
        const entity = toTierEntity(cell.entity_kind, cell.entity_id, cell.entity, UNPLACED);
        cells.push({
            label: cell.label,
            key: entityKey(cell.entity_kind, cell.entity_id),
            name: entity.name,
            artURL: entity.icon && iconBase ? entityIconURL(entity.icon, iconBase, server) : null,
            fit: entityArtFit(entity),
        });
    }
    return { title: grid.title, slug: grid.slug, server: resolveGamedataServer(server), rows: grid.rows, cols: grid.cols, cells };
}

/** The image's id in the OG and download routes: the bare slug on the default server, `cn:{slug}` on another, as {@link storyOgId} does. */
export function gridOgId(slug: string, server: string = DEFAULT_GAMEDATA_SERVER): string {
    return !isGamedataServer(server) || server === DEFAULT_GAMEDATA_SERVER ? slug : `${server}:${slug}`;
}

/** The PNG's download name: the slug made filename-safe, `grid` when nothing survives. The download button and the image route's attachment both use it. */
export function gridImageFilename(slug: string): string {
    const stem = slug.replace(/[^a-z0-9_-]+/gi, "-").replace(/^-+|-+$/g, "") || "grid";
    return `${stem}-grid.png`;
}

/** The inverse of {@link gridOgId}. */
export function parseGridOgId(id: string): { server: GamedataServer; slug: string } {
    const at = id.indexOf(":");
    if (at > 0) {
        const prefix = id.slice(0, at);
        if (isGamedataServer(prefix)) return { server: prefix, slug: id.slice(at + 1) };
    }
    return { server: DEFAULT_GAMEDATA_SERVER, slug: id };
}
