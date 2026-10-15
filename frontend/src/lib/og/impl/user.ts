/**
 * A player's social card, as data.
 *
 * Built in two places from the same three documents (`/get-user`,
 * `/get-user-supports`, `/roster`): on the page, by the profile route's
 * loader, to hash the card into its `og:image` URL; and on the server, by the
 * `user` OG handler, when that URL is requested and the PNG is not cached.
 * Both go through {@link buildUserOgData}, so the hash covers every field the
 * card draws. The handler adds the resolved art (`art`) on top; that is a pure
 * function of the hashed ids plus static game data, so it is not hashed.
 *
 * The card shows a visitor's view, whoever renders the page: the handler reads
 * the documents anonymously, and the builder projects an owner's documents
 * the same way the backend projects them for a visitor (no score or grade
 * while the Score tab is private, no roster while the Roster tab is, no LMD
 * while the Inventory tab is). A private profile answers the handler 403, so
 * it has no card at all.
 *
 * What can still drift under one hash, each a re-render-free change to what
 * the same ids draw: static game data (an operator's rarity, a skill's icon),
 * an asset file replaced at the same path (art, avatar, skill icon), and the
 * fonts. The roster enters as a digest of the fields the card reads, not as
 * rows, so the loader data stays small.
 */

import type { IRosterEntry } from "#/lib/api/user";
import type { ProfileBackground } from "#/types/generated/ProfileBackground";
import type { ProfileHeaderField } from "#/types/generated/ProfileHeaderField";
import type { ProfileLayout } from "#/types/generated/ProfileLayout";
import type { ProfileTabId } from "#/types/generated/ProfileTabId";
import type { SupportUnit } from "#/types/generated/SupportUnit";
import type { IUserProfile } from "#/types/user";
import { stableHash } from "./hash";

/** One support unit as the card's hash sees it: the `/get-user-supports` row's drawn fields. */
export interface IUserOgSupport {
    id: string;
    skinId: string | null;
    elite: number;
    level: number;
    skillIndex: number;
    specializeLevel: number;
}

/** One unit tile as the template draws it, resolved by the handler. */
export interface IUserOgTile {
    rarity: number;
    elite: number;
    level: number;
    avatarURL?: string;
    skillIconURL?: string;
    /** Mastery of the drawn skill, 0 to 3. */
    mastery: number;
}

/** What the handler resolves from the hashed fields: art inlined as data URIs, tiles with their rarity. */
export interface IUserOgArt {
    avatarURL?: string;
    heroURL?: string;
    /** CSS `object-position` of the hero art. */
    heroPosition: string;
    /** The hero's zoom past `cover`, 100 to 300. */
    heroZoom: number;
    tiles: IUserOgTile[];
    tilesKind: "supports" | "roster";
    rarityCounts?: Record<number, number>;
}

export interface IUserOgData {
    uid: string;
    nickname: string;
    /** In-game discriminator (`Eltik#1234` -> "1234"). */
    nickNumber: string | null;
    /** `servers.code`, e.g. `EN`. */
    server: string;
    level: number | null;
    /** `null` while the Score tab is private. */
    grade: string | null;
    /** 0 to 1; `null` while the Score tab is private. */
    totalScore: number | null;
    avatarId: string | null;
    operatorCount: number | null;
    /** Outfits beyond the default art, the count the profile's own stat strip shows. */
    skinCount: number | null;
    /** The fourth stat: LMD while the Inventory tab is public, else the item count the profile header always shows. */
    extraStat: { kind: "lmd" | "items"; value: number | null };
    secretary: string | null;
    secretarySkinId: string | null;
    background: ProfileBackground | null;
    supports: IUserOgSupport[];
    /** {@link rosterDigest} of the visible roster; empty when there is none. */
    rosterDigest: string;
    /** Filled by the OG handler only; not part of the hash (see the module note). */
    art?: IUserOgArt;
}

/** Whether a visitor sees `tab`. A visitor's layout lists only the visible tabs; an owner's lists every tab with its flag. No layout shows them all. */
export function userTabVisible(layout: ProfileLayout | null | undefined, tab: ProfileTabId): boolean {
    if (!layout) return true;
    return layout.tabs.some((t) => t.id === tab && t.visible);
}

/** The roster fields the card reads (rarity counts by id, and the top picks' elite, level, outfit and skill), as one short hash. Order-free. */
export function rosterDigest(roster: readonly IRosterEntry[] | null | undefined): string {
    if (!roster || roster.length === 0) return "";
    const rows = roster.map((e) => `${e.operator_id}:${e.elite}:${e.level}:${e.skin_id ?? ""}:${e.default_skill ?? ""}:${(e.masteries ?? []).map((m) => `${m.index}.${m.mastery}`).join(",")}`);
    return `${roster.length}.${stableHash(rows.sort().join("|"))}`;
}

function toSupport(s: SupportUnit): IUserOgSupport {
    return {
        id: s.operator_id,
        skinId: s.skin_id,
        elite: s.elite ?? 0,
        level: s.level ?? 1,
        skillIndex: s.skill_index,
        specializeLevel: s.specialize_level,
    };
}

/** Whether a visitor receives the header number `field`. No layout, or no header in it, hides nothing. */
export function userHeaderShows(layout: ProfileLayout | null | undefined, field: ProfileHeaderField): boolean {
    return !(layout?.header?.hidden.includes(field) ?? false);
}

/**
 * The card's data from the three documents, projected to what a visitor may see: an
 * owner's own profile carries the numbers they hid, so each is dropped here as `/get-user`
 * drops it for a visitor. `supports` and `roster` are `null` when they could not be read.
 */
export function buildUserOgData(profile: IUserProfile, supports: readonly SupportUnit[] | null | undefined, roster: readonly IRosterEntry[] | null | undefined): IUserOgData {
    const layout = profile.profile_layout;
    const scoreVisible = userTabVisible(layout, "score");
    const inventoryVisible = userTabVisible(layout, "inventory");
    const rosterVisible = userTabVisible(layout, "roster");
    const shows = (field: ProfileHeaderField) => userHeaderShows(layout, field);
    return {
        uid: profile.uid,
        nickname: profile.nickname?.trim() || "Player",
        nickNumber: profile.nick_number,
        server: profile.server,
        level: shows("level") ? profile.level : null,
        grade: scoreVisible ? profile.grade : null,
        totalScore: scoreVisible ? profile.total_score : null,
        avatarId: profile.avatar_id,
        operatorCount: shows("operators") ? profile.operator_count : null,
        skinCount: shows("skins") ? (profile.non_default_skin_count ?? profile.skin_count) : null,
        // A hidden LMD balance falls back to the item count, as a private Inventory tab does.
        extraStat: inventoryVisible && shows("lmd") ? { kind: "lmd", value: profile.lmd } : { kind: "items", value: shows("items") ? profile.item_count : null },
        secretary: profile.secretary,
        secretarySkinId: profile.secretary_skin_id,
        background: layout?.background ?? null,
        supports: (supports ?? []).map(toSupport),
        rosterDigest: rosterVisible ? rosterDigest(roster) : "",
    };
}

/** The hashed half of {@link IUserOgData}, in a fixed order; everything but `art`. */
export function userOgHashParts(data: IUserOgData): unknown[] {
    const bg = data.background;
    return [
        data.uid,
        data.nickname,
        data.nickNumber ?? "",
        data.server,
        data.level ?? "",
        data.grade ?? "",
        data.totalScore ?? "",
        data.avatarId ?? "",
        data.operatorCount ?? "",
        data.skinCount ?? "",
        `${data.extraStat.kind}=${data.extraStat.value ?? ""}`,
        data.secretary ?? "",
        data.secretarySkinId ?? "",
        bg ? `${bg.kind}:${bg.id}:${bg.focus_x ?? ""}:${bg.focus_y ?? ""}:${bg.scale ?? ""}:${bg.elite ?? ""}` : "",
        data.supports.map((s) => `${s.id}:${s.skinId ?? ""}:${s.elite}:${s.level}:${s.skillIndex}:${s.specializeLevel}`).join("|"),
        data.rosterDigest,
    ];
}
