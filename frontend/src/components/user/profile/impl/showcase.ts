import { type ITierEntity, type TierEntityKind, toTierEntity, UNPLACED } from "#/lib/api/tier-entities";
import type { ProfileShowcase } from "#/types/generated/ProfileShowcase";
import type { ShowcaseBlock } from "#/types/generated/ShowcaseBlock";
import type { ShowcaseView } from "#/types/generated/ShowcaseView";

// The Showcase tab's working copy, and the rules the backend applies to what is
// saved, mirrored so the editor never offers a save the server would refuse or
// silently cut. The backend is the authority: `ProfileShowcase::normalize` in
// `backend/src/database/models/profile_layout.rs` and `services/showcase.rs`.

/** Most blocks one showcase holds (`SHOWCASE_MAX_BLOCKS`). */
export const SHOWCASE_MAX_BLOCKS = 12;
/** Most entities one favourites block holds (`SHOWCASE_MAX_IDS`). */
export const SHOWCASE_MAX_IDS = 24;
/** Longest favourites title, in characters (`SHOWCASE_TITLE_MAX`). */
export const SHOWCASE_TITLE_MAX = 40;

export type ShowcaseBlockType = ShowcaseBlock["type"];

/** One favourite in the editor: the id, and what the reader's server (or `server`) resolved it to. */
export interface IShowcaseEntity {
    id: string;
    /** `null` when no loaded game data knows the id: shown to the owner, dropped on save. */
    entity: ITierEntity | null;
    /** The server the art comes from when it is not the reader's (a CN-only operator), else `null`. */
    server: string | null;
}

interface IDraftBase {
    /** Stable React key for the editor's list; never sent. */
    key: string;
    /** What the block points at is gone. Shown to the owner, dropped on save. */
    removed: boolean;
}

export type ShowcaseDraftBlock = (IDraftBase & { type: "favourites"; kind: TierEntityKind; title: string; entities: IShowcaseEntity[] }) | (IDraftBase & { type: "grid"; slug: string }) | (IDraftBase & { type: "tier_list"; slug: string }) | (IDraftBase & { type: "plan"; id: string; operatorId?: string });

let nextKey = 0;
/** A fresh editor key. */
export function draftKey(): string {
    nextKey += 1;
    return `showcase-${nextKey}`;
}

/** The view as the editor's working copy. */
export function draftFromView(view: ShowcaseView): ShowcaseDraftBlock[] {
    return view.blocks.map(({ block, removed, entities }): ShowcaseDraftBlock => {
        const key = draftKey();
        switch (block.type) {
            case "favourites":
                return {
                    key,
                    removed,
                    type: "favourites",
                    kind: block.entity_kind,
                    title: block.title ?? "",
                    entities: entities.map((e) => ({ id: e.id, entity: e.entity ? toTierEntity(block.entity_kind, e.id, e.entity, UNPLACED) : null, server: e.entity_server })),
                };
            case "grid":
                return { key, removed, type: "grid", slug: block.slug };
            case "tier_list":
                return { key, removed, type: "tier_list", slug: block.slug };
            default:
                return { key, removed, type: "plan", id: block.id, operatorId: block.operator_id };
        }
    });
}

/** `title` as the backend stores it: control characters dropped, trimmed, at most {@link SHOWCASE_TITLE_MAX} characters. */
export function cleanTitle(title: string): string {
    // biome-ignore lint/suspicious/noControlCharactersInRegex: stripping them is the point
    const stripped = title.replace(/[\u0000-\u001f\u007f-\u009f]/g, "").trim();
    return Array.from(stripped).slice(0, SHOWCASE_TITLE_MAX).join("").trimEnd();
}

/** How many blocks and favourites a save will drop: gone ones, and favourites blocks with no known pick. */
export function droppedOnSave(draft: readonly ShowcaseDraftBlock[]): number {
    let n = 0;
    for (const block of draft) {
        if (block.removed) n += 1;
        else if (block.type === "favourites") {
            const unknown = block.entities.filter((e) => e.entity === null).length;
            // A block left with no known pick is dropped whole, an empty new one included.
            n += unknown === block.entities.length ? 1 : unknown;
        }
    }
    return n;
}

/**
 * The blocks to send. Removed blocks and unknown favourites are dropped (the backend
 * would refuse them), an emptied favourites block with them; titles are cleaned; ids
 * are deduplicated and capped; a repeated grid, tier list or plan keeps the first;
 * the list is cut to {@link SHOWCASE_MAX_BLOCKS}.
 */
export function blocksForSave(draft: readonly ShowcaseDraftBlock[]): ShowcaseBlock[] {
    const out: ShowcaseBlock[] = [];
    const seen = new Set<string>();
    for (const block of draft) {
        if (out.length === SHOWCASE_MAX_BLOCKS) break;
        if (block.removed) continue;
        if (block.type === "favourites") {
            const ids: string[] = [];
            for (const e of block.entities) {
                if (e.entity !== null && !ids.includes(e.id) && ids.length < SHOWCASE_MAX_IDS) ids.push(e.id);
            }
            if (ids.length === 0) continue;
            const title = cleanTitle(block.title);
            out.push(title ? { type: "favourites", entity_kind: block.kind, ids, title } : { type: "favourites", entity_kind: block.kind, ids });
            continue;
        }
        const referent = block.type === "plan" ? `plan:${block.id.toLowerCase()}` : `${block.type}:${block.slug}`;
        if (seen.has(referent)) continue;
        seen.add(referent);
        out.push(block.type === "plan" ? { type: "plan", id: block.id, operator_id: block.operatorId } : { type: block.type, slug: block.slug });
    }
    return out;
}

/**
 * The `profile_layout` a showcase save sends: the `showcase` key and nothing else, an
 * empty showcase as `{ blocks: [] }`. The backend keeps every key a save leaves out,
 * so the tabs and the header background stay exactly as stored, never this page's
 * possibly stale copy of them. Never `null`: that resets the whole layout, which only
 * the tab editor's Reset may do.
 */
export function showcaseForSave(blocks: readonly ShowcaseBlock[]): { showcase: ProfileShowcase } {
    return { showcase: { blocks: [...blocks] } };
}

/** Whether the visitor-facing showcase differs from the saved one: anything to save. */
export function isDraftDirty(saved: readonly ShowcaseBlock[], draft: readonly ShowcaseDraftBlock[]): boolean {
    return JSON.stringify(saved) !== JSON.stringify(blocksForSave(draft)) || draft.some((b) => b.removed);
}

/** The blocks a reader is shown. The owner sees every block, the removed ones marked; a visitor never sees one, which the backend already guarantees. */
export function shownBlocks(view: ShowcaseView, isOwner: boolean): ShowcaseView["blocks"] {
    return isOwner ? view.blocks : view.blocks.filter((b) => !b.removed);
}

/** Where on the site a grid or tier list lives, so a pasted link can be matched. */
const PATH_PREFIX: Record<"grid" | "tier_list", string> = { grid: "/grids/", tier_list: "/tier-lists/" };

/**
 * The slug in `input`: a bare slug, or a link to the grid or tier list (any host, with
 * or without a query or hash). `null` when the input is blank or links somewhere else.
 */
export function slugFromInput(input: string, type: "grid" | "tier_list"): string | null {
    const raw = input.trim();
    if (!raw) return null;
    const prefix = PATH_PREFIX[type];
    let path: string | null = null;
    if (/^https?:\/\//i.test(raw)) {
        try {
            path = new URL(raw).pathname;
        } catch {
            return null;
        }
    } else if (raw.startsWith("/")) {
        path = raw.split(/[?#]/)[0] ?? raw;
    }
    if (path === null) return /^[^\s/?#]+$/.test(raw) ? raw : null;
    if (!path.startsWith(prefix)) return null;
    const slug = path.slice(prefix.length).split("/")[0] ?? "";
    if (!slug) return null;
    try {
        return decodeURIComponent(slug);
    } catch {
        return slug;
    }
}

/** A new, empty favourites block of `kind`. */
export function newFavourites(kind: TierEntityKind): ShowcaseDraftBlock {
    return { key: draftKey(), removed: false, type: "favourites", kind, title: "", entities: [] };
}

/** `entities` with `entity` added at the end, or taken out when it is already there. Capped at {@link SHOWCASE_MAX_IDS}. */
export function toggleFavourite(entities: readonly IShowcaseEntity[], entity: ITierEntity, server: string | null): IShowcaseEntity[] {
    if (entities.some((e) => e.id === entity.id)) return entities.filter((e) => e.id !== entity.id);
    if (entities.length >= SHOWCASE_MAX_IDS) return [...entities];
    return [...entities, { id: entity.id, entity, server }];
}
