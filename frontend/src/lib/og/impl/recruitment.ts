/**
 * A recruitment calculator link's social card: the id, the data shape and the
 * hash. Kept apart from the builder (`recruitment.build.ts`), which runs the
 * calculator, so that hashing an og:image URL never bundles it.
 *
 * The id is the selected tag ids, numeric, sorted and deduped, joined with
 * `-` (`11-14`), prefixed `server:` for a server other than the default, as
 * the grid image's id is (`jp:11-14`). Order-free on purpose: `?tags=14,11`
 * and `?tags=11,14` are one selection and share one cached PNG.
 */

import { DEFAULT_GAMEDATA_SERVER, type GamedataServer, isGamedataServer } from "#/lib/api/gamedata";
import type { IDefaultOgData } from "./templates/Default";

/** The most tags the game offers at once, the calculator's `MAX_SELECTED_TAGS`. */
const MAX_TAGS = 5;

export interface IRecruitmentOgTag {
    id: number;
    name: string;
}

export interface IRecruitmentOgOperator {
    id: string;
    name: string;
    rarity: number;
    /** The avatar inlined as a data URI; filled by the OG handler only. */
    avatarURL?: string;
}

/** The calculator's best combination for the selection. */
export interface IRecruitmentOgBest {
    tagIds: number[];
    /** {@link guaranteedFloorRarity} of the combination: 1 is a Robot lock, 2 to 6 stars. */
    floor: number;
    /** The first operators of the pool in the calculator's own order, at most {@link RECRUITMENT_OG_TILES}. */
    operators: IRecruitmentOgOperator[];
    /** The whole pool's size, so the card can say how many it left off. */
    total: number;
}

export interface IRecruitmentOgData {
    server: GamedataServer;
    /** Every selected tag the server knows, in id order. Empty only on the fallback card. */
    tags: IRecruitmentOgTag[];
    /** `null` when no combination of the tags matches any operator. */
    best: IRecruitmentOgBest | null;
    /** Set instead of the above when no selected tag survives: the handler draws the calculator's preset card. */
    preset?: IDefaultOgData;
}

/** Operator tiles the card has room for. */
export const RECRUITMENT_OG_TILES = 8;

/** `raw` (a `-` or `,` separated list, or numbers) as positive integer tag ids, deduped, sorted, at most five. Anything else is dropped. */
export function normalizeRecruitmentTagIds(raw: string | readonly (number | string)[]): number[] {
    const parts = typeof raw === "string" ? raw.split(/[-,]/) : raw.map(String);
    const ids = new Set<number>();
    for (const part of parts) {
        if (!/^\d+$/.test(part.trim())) continue;
        const n = Number(part);
        if (Number.isSafeInteger(n) && n > 0) ids.add(n);
    }
    return [...ids].sort((a, b) => a - b).slice(0, MAX_TAGS);
}

/** The card's URL id for `tagIds` on `server`. */
export function recruitmentOgId(tagIds: readonly number[], server: string = DEFAULT_GAMEDATA_SERVER): string {
    const body = normalizeRecruitmentTagIds(tagIds).join("-");
    return !isGamedataServer(server) || server === DEFAULT_GAMEDATA_SERVER ? body : `${server}:${body}`;
}

/** The inverse of {@link recruitmentOgId}: an unknown prefix is no prefix, and the tag list is normalized. */
export function parseRecruitmentOgId(id: string): { server: GamedataServer; tagIds: number[] } {
    const at = id.indexOf(":");
    if (at > 0) {
        const prefix = id.slice(0, at);
        if (isGamedataServer(prefix)) return { server: prefix, tagIds: normalizeRecruitmentTagIds(id.slice(at + 1)) };
    }
    return { server: DEFAULT_GAMEDATA_SERVER, tagIds: normalizeRecruitmentTagIds(id) };
}

/** The hashed fields of {@link IRecruitmentOgData}: everything drawn but the inlined avatars, which follow from the operator ids. */
export function recruitmentOgHashParts(data: IRecruitmentOgData): unknown[] {
    const best = data.best;
    return [data.server, data.tags.map((t) => `${t.id}=${t.name}`).join("|"), best ? `${best.tagIds.join(",")}:${best.floor}:${best.total}:${best.operators.map((o) => `${o.id}.${o.rarity}.${o.name}`).join(",")}` : "", data.preset ? `${data.preset.title}|${data.preset.subtitle ?? ""}` : ""];
}
