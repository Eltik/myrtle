/**
 * Home's pure model: the focus cards a translator or editor sees instead of
 * the staff queue (`#/components/admin/shell/inbox`), and the merged
 * recent-changes feed. Cards carry data, not text: the component owns every
 * string.
 */

import type { IQueueOperator } from "#/components/admin/shell/inbox";
import { atLeast, currentPercent, type ILocaleGrant, type ILocaleProgress, missingCount } from "#/components/admin/shell/model";

export interface IFocusTierList {
    slug: string;
    title: string;
    permission: string;
    /** Absent when the list is not in the browse feed. */
    tiers?: number;
    placements?: number;
    updatedAtMs?: number;
}

export type FocusCard = { kind: "locale"; code: string; level: string; editable: boolean; stale: number; missing: number; percent: number } | { kind: "notes"; empty: number; total: number; sixStarNames: string[]; firstOperatorId: string } | { kind: "tierList"; list: IFocusTierList; canPublish: boolean };

export interface IFocusInput {
    role: string | null;
    /** The translator's locale grants, in display order. */
    localeGrants: readonly ILocaleGrant[];
    progress: readonly ILocaleProgress[];
    emptyOps: readonly IQueueOperator[];
    totalOps: number;
    tierLists: readonly IFocusTierList[];
}

/** What a translator or editor opens Home to find (design `focusCards`). */
export function buildFocusCards(input: IFocusInput): FocusCard[] {
    if (input.role === "translator") {
        const cards: FocusCard[] = [];
        for (const g of input.localeGrants) {
            const p = input.progress.find((row) => row.locale === g.code);
            if (!p) continue;
            cards.push({ kind: "locale", code: g.code, level: g.level, editable: atLeast(g.level, "edit"), stale: p.stale, missing: missingCount(p), percent: currentPercent(p) });
        }
        return cards;
    }
    if (input.role === "tier_list_editor") {
        const cards: FocusCard[] = [];
        if (input.emptyOps.length > 0) {
            cards.push({ kind: "notes", empty: input.emptyOps.length, total: input.totalOps, sixStarNames: input.emptyOps.filter((o) => o.rarity === 6).map((o) => o.name), firstOperatorId: input.emptyOps[0].id });
        }
        for (const list of input.tierLists) cards.push({ kind: "tierList", list, canPublish: atLeast(list.permission, "publish") });
        return cards;
    }
    return [];
}

/** At most `max` names; the rest is a count the caller words. */
export function truncateNames(names: readonly string[], max = 3): { shown: string[]; rest: number } {
    return { shown: names.slice(0, max), rest: Math.max(0, names.length - max) };
}

export interface INoteAuditRow {
    id: number;
    operator_id: string;
    field_name: string;
    changed_at: string;
    actor: { user_id: string; nickname: string | null; uid: string | null };
}

export interface ITranslationAuditRow {
    id: number;
    message_key: string;
    locale: string;
    changed_at: string;
    actor: { user_id: string; nickname: string | null; uid: string | null };
}

/** `actorName` is null when the actor's account was deleted; the component words that. */
export type RecentChange = { kind: "note"; id: string; at: string; actorName: string | null; operatorId: string; field: string } | { kind: "translation"; id: string; at: string; actorName: string | null; locale: string; key: string };

/**
 * Notes and translation edits merged newest first (design `recent`).
 * `onlyActorId` keeps one account's rows, for the non-staff "Your recent changes".
 */
export function mergeRecentChanges(notes: readonly INoteAuditRow[], translations: readonly ITranslationAuditRow[], options: { onlyActorId?: string; limit?: number } = {}): RecentChange[] {
    const name = (a: INoteAuditRow["actor"]) => a.nickname ?? a.uid ?? null;
    const keep = (a: INoteAuditRow["actor"]) => options.onlyActorId === undefined || a.user_id === options.onlyActorId;
    const rows: RecentChange[] = [
        ...notes.filter((n) => keep(n.actor)).map((n): RecentChange => ({ kind: "note", id: `n${n.id}`, at: n.changed_at, actorName: name(n.actor), operatorId: n.operator_id, field: n.field_name })),
        ...translations.filter((r) => keep(r.actor)).map((r): RecentChange => ({ kind: "translation", id: `t${r.id}`, at: r.changed_at, actorName: name(r.actor), locale: r.locale, key: r.message_key })),
    ];
    rows.sort((a, b) => (Date.parse(b.at) || 0) - (Date.parse(a.at) || 0));
    return rows.slice(0, options.limit ?? 5);
}
