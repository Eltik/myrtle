/**
 * A squad builder share link's social card: the id, the data shape and the
 * hash.
 *
 * The id carries the whole squad so the handler can redraw it from the URL
 * alone: the canonical share params (`encodeShare` of the decoded link, in
 * the fixed order `s`, `sup`, `n`, `st`) as a query string, base64url
 * encoded, prefixed `server:` off the default server. Canonical, so two links
 * that decode to the same squad share one cached PNG. Capped at
 * {@link SQUAD_OG_ID_MAX} characters: a link over it is retried without the
 * per-seat builds, and failing that unfurls as the tool's preset card.
 *
 * What the card draws beyond the share (operator rarity, the stage's code and
 * name) is static game data read by the handler; it is not hashed, so a renamed
 * stage keeps its old card until the design version moves.
 */

import { decodeShare, encodeShare, type IShareSearch, type ISquadDraft } from "#/components/tools/squad-builder/model";
import type { GamedataServer } from "#/lib/api/gamedata";
import type { SquadMember } from "#/types/generated/SquadMember";
import { parseServerScopedOgId, serverScopedOgId } from "./ids";
import type { IDefaultOgData } from "./templates/Default";

/** The longest id accepted. Twelve seats with full builds, a support and a 40-character name encode to about 1,000. */
export const SQUAD_OG_ID_MAX = 1400;

export interface ISquadOgSeat {
    id: string;
    /** 0-based skill index. */
    skill: number;
    /** Elite phase and mastery when the link carries a build; `null` otherwise. */
    elite: number | null;
    mastery: number | null;
    /** Filled by the OG handler only. */
    rarity?: number;
    avatarURL?: string;
}

export interface ISquadOgData {
    server: GamedataServer;
    /** The canonical share query string the id encodes; the hash's identity for the squad. */
    share: string;
    name: string;
    stageId: string | null;
    /** Exactly twelve, `null` for an empty seat. */
    seats: (ISquadOgSeat | null)[];
    support: ISquadOgSeat | null;
    /** The stage's code and name; filled by the OG handler when `stageId` resolves. */
    stage?: { code: string; name: string };
    /** Set instead of the squad when the id does not decode: the handler draws the tool's preset card. */
    preset?: IDefaultOgData;
}

function toBase64Url(text: string): string {
    let bin = "";
    for (const b of new TextEncoder().encode(text)) bin += String.fromCharCode(b);
    return btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function fromBase64Url(text: string): string | null {
    if (!/^[A-Za-z0-9_-]+$/.test(text)) return null;
    try {
        const padded = text.replace(/-/g, "+").replace(/_/g, "/") + "=".repeat((4 - (text.length % 4)) % 4);
        return new TextDecoder("utf-8", { fatal: true }).decode(Uint8Array.from(atob(padded), (c) => c.charCodeAt(0)));
    } catch {
        return null;
    }
}

/** The share params as one query string in a fixed key order. */
function shareString(search: IShareSearch): string {
    const params = new URLSearchParams();
    for (const key of ["s", "sup", "n", "st"] as const) {
        const value = search[key];
        if (value) params.set(key, value);
    }
    return params.toString();
}

function hasSeat(draft: ISquadDraft): boolean {
    return draft.support !== null || draft.members.some((m) => m !== null);
}

/** The card id for a share link's params, or `null` when they carry no seat or do not fit the cap even without builds. */
export function squadOgId(search: IShareSearch, server?: string): string | null {
    const draft = decodeShare(search);
    if (!draft || !hasSeat(draft)) return null;
    for (const includeBuilds of [true, false]) {
        const id = serverScopedOgId(toBase64Url(shareString(encodeShare(draft, includeBuilds))), server);
        if (id.length <= SQUAD_OG_ID_MAX) return id;
    }
    return null;
}

/** The inverse of {@link squadOgId}: the server and the decoded squad, or `null` for an id that is over the cap, not base64url, or carries no seat. */
export function parseSquadOgId(raw: string): { server: GamedataServer; draft: ISquadDraft; share: string } | null {
    if (raw.length > SQUAD_OG_ID_MAX) return null;
    const { server, id } = parseServerScopedOgId(raw);
    const text = fromBase64Url(id);
    if (text === null) return null;
    const params = new URLSearchParams(text);
    const search: IShareSearch = {};
    for (const key of ["s", "sup", "n", "st"] as const) {
        const value = params.get(key);
        if (value !== null) search[key] = value;
    }
    const draft = decodeShare(search);
    if (!draft || !hasSeat(draft)) return null;
    return { server, draft, share: shareString(encodeShare(draft, true)) };
}

function toSeat(m: SquadMember | null): ISquadOgSeat | null {
    if (!m) return null;
    return { id: m.char_id, skill: m.skill_index, elite: m.build?.phase ?? null, mastery: m.build?.mastery ?? null };
}

/** The card's data for a decoded squad. Both the route's `head()` and the handler build it from the same draft. */
export function buildSquadOgData(draft: ISquadDraft, server: GamedataServer): ISquadOgData {
    return {
        server,
        share: shareString(encodeShare(draft, true)),
        name: draft.name.trim(),
        stageId: draft.stageId,
        seats: draft.members.map(toSeat),
        support: toSeat(draft.support),
    };
}

/** The share string is the squad; the server picks the art tree and the stage's text. */
export function squadOgHashParts(data: ISquadOgData): unknown[] {
    return [data.server, data.share, data.preset ? `${data.preset.title}|${data.preset.subtitle ?? ""}` : ""];
}
