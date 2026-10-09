/**
 * The staff "Needs attention" queue (design `queue`), pure. Items carry data,
 * not text, so the nav badge can count the queue without rendering it and
 * Home's inbox card owns every string.
 */
import { atLeast, type ILocaleGrant, type ILocaleProgress, missingCount } from "./model";

export interface IQueueLocale {
    code: string;
    enabled: boolean;
    is_source: boolean;
}

export interface IQueueOperator {
    id: string;
    name: string;
    rarity: number;
}

/** A language grant whose account is gone: what revoking it needs. */
export interface IOrphanLocaleGrant {
    locale: string;
    user_id: string;
    permission: string;
}

export type InboxItem =
    | { kind: "translationsStale"; locale: string; count: number }
    | { kind: "translationsMissing"; locale: string; count: number }
    | { kind: "notesEmpty"; count: number; sixStarNames: string[]; firstOperatorId: string }
    | { kind: "orphanLocaleGrants"; grants: IOrphanLocaleGrant[] }
    | { kind: "systemDegraded" };

/**
 * The locales whose backlog belongs in the queue, in `locales` order (design
 * `editableLocs`): ones this user can edit, minus the source language, and
 * for a super-admin only the public ones (a hidden locale is nobody's emergency).
 */
export function queueLocales(access: { isSuper: boolean; myLoc: readonly ILocaleGrant[] }, locales: readonly IQueueLocale[]): string[] {
    return locales
        .filter((l) => {
            if (l.is_source) return false;
            const grant = access.myLoc.find((g) => g.code === l.code);
            return grant !== undefined && atLeast(grant.level, "edit") && (l.enabled || !access.isSuper);
        })
        .map((l) => l.code);
}

/** Operators with no note content, by name (design `emptyOps`). */
export function emptyOperators(operators: readonly IQueueOperator[], filled: ReadonlySet<string>): IQueueOperator[] {
    return operators.filter((o) => !filled.has(o.id)).sort((a, b) => a.name.localeCompare(b.name));
}

export interface IInboxInput {
    staff: boolean;
    canNotes: boolean;
    canAssign: boolean;
    /** From `queueLocales`. */
    locales: readonly string[];
    progress: readonly ILocaleProgress[];
    /** From `emptyOperators`. */
    emptyOps: readonly IQueueOperator[];
    /** Language grants left by deleted accounts (`orphanLocaleGrants`). */
    orphanGrants: readonly IOrphanLocaleGrant[];
    healthDegraded: boolean;
}

/**
 * The queue's People rows: one aggregated row per thing a super-admin can
 * act on, never one row per person. The People nav badge counts these too.
 */
export function peopleItems(canAssign: boolean, orphanGrants: readonly IOrphanLocaleGrant[]): InboxItem[] {
    if (!canAssign || orphanGrants.length === 0) return [];
    return [{ kind: "orphanLocaleGrants", grants: orphanGrants.map((g) => ({ locale: g.locale, user_id: g.user_id, permission: g.permission })) }];
}

/** The staff queue, most urgent first, in the design's order. */
export function buildInboxQueue(input: IInboxInput): InboxItem[] {
    if (!input.staff) return [];
    const items: InboxItem[] = [];
    for (const code of input.locales) {
        const p = input.progress.find((row) => row.locale === code);
        if (!p) continue;
        if (p.stale > 0) items.push({ kind: "translationsStale", locale: code, count: p.stale });
        const missing = missingCount(p);
        if (missing > 0) items.push({ kind: "translationsMissing", locale: code, count: missing });
    }
    if (input.canNotes && input.emptyOps.length > 0) {
        items.push({ kind: "notesEmpty", count: input.emptyOps.length, sixStarNames: input.emptyOps.filter((o) => o.rarity === 6).map((o) => o.name), firstOperatorId: input.emptyOps[0].id });
    }
    items.push(...peopleItems(input.canAssign, input.orphanGrants));
    if (input.healthDegraded) items.push({ kind: "systemDegraded" });
    return items;
}

/**
 * Language grants whose account no longer exists. The grant list LEFT JOINs
 * `users`, and `users.uid` is NOT NULL, so a null `user_uid` means the
 * account row is gone.
 */
export function orphanLocaleGrants<T extends { user_uid: string | null }>(locales: readonly T[]): T[] {
    return locales.filter((g) => g.user_uid === null);
}
