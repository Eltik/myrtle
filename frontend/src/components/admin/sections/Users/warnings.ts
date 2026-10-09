/**
 * The People model: who can edit what, whose access contradicts their role
 * (design `warnOf` / `accessOf`), and which grants outlived their account.
 * Pure, so the People table, the person sheet, the People nav badge and the
 * Home inbox all read the same answer.
 */
import { atLeast } from "#/components/admin/shell/model";
import { ADMIN_USER_SERVERS } from "#/lib/api/admin/types";
import type { TranslationGrant } from "#/types/generated/TranslationGrant";

/**
 * Why a person's grants and role disagree (design `warnOf`). Only one case is
 * left: any role may hold language or tier-list grants and use them, and a tier
 * list editor with no lists still edits operator notes.
 */
export type AccessWarning = "translatorNoEdit";

/** A tier-list grant as the People model needs it. */
export interface IPersonTierListGrant {
    slug: string;
    title: string;
    permission: string;
    userId: string;
    userUid: string;
    userNickname: string | null;
    grantedBy: string | null;
    grantedByNickname: string | null;
    grantedAt: string;
}

/** A language grant as the People model needs it. Names are null once the account is gone. */
export type IPersonLocaleGrant = TranslationGrant;

export interface IPersonGrants {
    tierLists: IPersonTierListGrant[];
    locales: IPersonLocaleGrant[];
}

const NO_GRANTS: IPersonGrants = { tierLists: [], locales: [] };

/** Every grant, bucketed by the account holding it. */
export function indexGrantsByUser(tierLists: readonly IPersonTierListGrant[], locales: readonly IPersonLocaleGrant[]): Map<string, IPersonGrants> {
    const byUser = new Map<string, IPersonGrants>();
    const bucket = (id: string): IPersonGrants => {
        let entry = byUser.get(id);
        if (!entry) {
            entry = { tierLists: [], locales: [] };
            byUser.set(id, entry);
        }
        return entry;
    };
    for (const g of tierLists) bucket(g.userId).tierLists.push(g);
    for (const g of locales) bucket(g.user_id).locales.push(g);
    return byUser;
}

/** The holder's nickname and UID as their grants carry them, tier-list grants first. */
export function holderOf(grants: IPersonGrants): { nickname: string | null; uid: string | null } {
    const list = grants.tierLists[0];
    const locale = grants.locales[0];
    return { nickname: list?.userNickname ?? locale?.user_nickname ?? null, uid: list?.userUid ?? locale?.user_uid ?? null };
}

export function grantsOf(index: ReadonlyMap<string, IPersonGrants>, userId: string): IPersonGrants {
    return index.get(userId) ?? NO_GRANTS;
}

/** Whether this role's grants contradict it, and how (design `warnOf`). */
export function accessWarning(role: string, grants: IPersonGrants): AccessWarning | null {
    if (role === "translator" && !grants.locales.some((g) => atLeast(g.permission, "edit"))) return "translatorNoEdit";
    return null;
}

/**
 * What the "Can edit" column says, before it is put into words (design
 * `accessOf`). `locales` names the languages the account can edit through a
 * grant, whatever its role: the backend honours a language grant on any role.
 */
export type AccessSummary = { kind: "everything" } | { kind: "allLists"; locales: string[] } | { kind: "editor"; lists: string[]; locales: string[] } | { kind: "translator"; locales: string[] } | { kind: "player"; locales: string[] };

/**
 * `localeName` maps a locale code to the name the column shows. Only Edit and
 * above count; a View grant changes nothing anyone can edit. A super-admin
 * already reaches every language, so their grants add nothing.
 */
export function accessSummary(role: string, grants: IPersonGrants, localeName: (code: string) => string): AccessSummary {
    const locales = grants.locales.filter((g) => atLeast(g.permission, "edit")).map((g) => localeName(g.locale));
    switch (role) {
        case "super_admin":
            return { kind: "everything" };
        case "tier_list_admin":
            return { kind: "allLists", locales };
        case "tier_list_editor":
            return { kind: "editor", lists: grants.tierLists.map((g) => g.title), locales };
        case "translator":
            return { kind: "translator", locales };
        default:
            return { kind: "player", locales };
    }
}

/** True when every grant this id holds belongs to a deleted account (and it holds at least one). */
export function isDeletedAccount(grants: IPersonGrants): boolean {
    return grants.tierLists.length === 0 && grants.locales.length > 0 && grants.locales.every((g) => g.user_uid === null);
}

/**
 * The name a person goes by in the panel: nickname, else UID. Neither is
 * left once the account is deleted, so `deletedLabel` stands in rather than
 * a raw account id.
 */
export function personLabel(person: { nickname: string | null; uid: string | null }, deletedLabel: string): string {
    return person.nickname ?? person.uid ?? deletedLabel;
}

/** Up to two initials for the avatar circle (design `initialsOf`). */
export function initialsOf(name: string): string {
    const words = name
        .replace(/[^\p{L}\p{N} ]/gu, " ")
        .trim()
        .split(/\s+/)
        .filter(Boolean);
    const initials = words
        .map((w) => [...w][0])
        .join("")
        .slice(0, 2);
    return (initials || [...name].slice(0, 2).join("")).toUpperCase();
}

/** Which grant picker the sheet opens on its own, so "Fix access" lands on the fix. */
export function pickerForWarning(warning: AccessWarning | null): "tierLists" | "locales" | null {
    return warning === "translatorNoEdit" ? "locales" : null;
}

/** People tab server filter; the URL keeps the lower-case code the API takes. */
export const PEOPLE_SERVERS = ADMIN_USER_SERVERS;
export type PeopleServer = (typeof PEOPLE_SERVERS)[number];

export function parsePeopleServer(raw: string | undefined): PeopleServer | undefined {
    return (PEOPLE_SERVERS as readonly string[]).includes(raw ?? "") ? (raw as PeopleServer) : undefined;
}
