import { skinpackFile } from "#/components/operators/detail/impl/assets";
import type { ITierEntity } from "#/lib/api/tier-entities";
import type { IGridEditCell } from "./state";

// The full-screen cell viewer's and the phone cell editor's pure parts: which
// cells the viewer steps through, what a swipe means, where an entity's page
// is, and which art it is drawn with. No React, so the tests import it.

/** A cell the read-only viewer opens: it holds a label or a pick. A cell with neither has nothing to show and stays inert. */
export function isViewableCell(cell: Pick<IGridEditCell, "label" | "kind">): boolean {
    return cell.label.trim().length > 0 || cell.kind !== null;
}

/** The indices of the viewable cells, in board order: what prev and next step through. */
export function viewableIndices(cells: readonly Pick<IGridEditCell, "label" | "kind">[]): number[] {
    const out: number[] = [];
    cells.forEach((cell, index) => {
        if (isViewableCell(cell)) out.push(index);
    });
    return out;
}

/**
 * The cell `step` places from `current` along `order`, or `null` past either
 * end: the viewer does not wrap, so the first and last cells read as ends. A
 * `current` not in `order` steps from where it would sit.
 */
export function neighbourIndex(order: readonly number[], current: number, step: -1 | 1): number | null {
    const at = order.indexOf(current);
    if (at === -1) {
        const next = step === 1 ? order.find((i) => i > current) : [...order].reverse().find((i) => i < current);
        return next ?? null;
    }
    return order[at + step] ?? null;
}

/** How far a finger travels before a swipe counts, in CSS px. */
export const SWIPE_STEP_PX = 48;
export const SWIPE_CLOSE_PX = 80;
/** A swipe must run this many times further along its axis than across it, so a diagonal drag does nothing. */
const SWIPE_DOMINANCE = 1.5;

export type SwipeIntent = "prev" | "next" | "close" | null;

/** What a touch that moved `dx`, `dy` (end minus start) asks for: left is next, right is prev, down is close. */
export function swipeIntent(dx: number, dy: number): SwipeIntent {
    const ax = Math.abs(dx);
    const ay = Math.abs(dy);
    if (ax >= SWIPE_STEP_PX && ax > ay * SWIPE_DOMINANCE) return dx < 0 ? "next" : "prev";
    if (dy >= SWIPE_CLOSE_PX && ay > ax * SWIPE_DOMINANCE) return "close";
    return null;
}

/** A site page an entity links to. Only routes that take the id as `$id`. */
export type EntityLink = { to: "/operators/$id"; id: string } | { to: "/enemies/$id"; id: string };

const LINKED_ROUTES = { operators: "/operators/$id", enemies: "/enemies/$id" } as const;

/**
 * The entity's page, read off the `href` the backend resolves per kind: an
 * operator, skin, skill, module or operator sprite links its operator, an enemy
 * its handbook entry, every other kind nothing. `entityPage` in tier-lists only
 * knows the enemy route, which is why the viewer reads `href` instead.
 */
export function entityLink(entity: ITierEntity | null): EntityLink | null {
    if (!entity?.resolved || !entity.href) return null;
    const match = /^\/(operators|enemies)\/([^/?#]+)$/.exec(entity.href);
    if (!match) return null;
    const route = LINKED_ROUTES[match[1] as keyof typeof LINKED_ROUTES];
    return { to: route, id: decodeURIComponent(match[2] as string) };
}

/**
 * The large art to try for an entity, best first, as asset paths under
 * `/api/assets`; empty when the kind's tile art is already its best (the
 * viewer then draws the tile art large). An operator is its elite 2 art, then
 * elite 1; the reduced `b` variant first (EN 2026-10-06: 323 of 441 operators
 * ship `_2b.png`, 381 `_2.png`, 438 `_1.png`; amiya 1.11 MB against 4.09 MB),
 * `_1b` skipped (161 of 441). A skin is its outfit art, reduced first. Named
 * like `skinThumbnail` and `skinTexture` in operators/detail/impl/assets.ts.
 */
export function heroArtPaths(entity: ITierEntity | null): string[] {
    if (!entity?.resolved) return [];
    if (entity.kind === "operator") {
        const base = `/textures/chararts/${entity.id}/${entity.id}`;
        return [`${base}_2b.png`, `${base}_2.png`, `${base}_1.png`];
    }
    if (entity.kind === "skin") {
        const owner = entity.charId;
        if (entity.id.includes("@")) {
            const file = skinpackFile(entity.id);
            return [`/textures/skinpack/${owner}/${file}b.png`, `/textures/skinpack/${owner}/${file}.png`];
        }
        const file = entity.id.replace("#", "_");
        return [`/textures/chararts/${owner}/${file}.png`];
    }
    return [];
}
