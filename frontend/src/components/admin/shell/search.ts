/**
 * The search params each admin section reads, parsed once at the route and
 * handed to the section component as its `search` prop. Every field is
 * optional: an absent param is the section's default, so links to a section
 * need no `search` at all and default URLs stay bare.
 *
 * The types live here rather than in the route files so a section component
 * can import its prop type without importing its own route.
 */
import { ADMIN_USER_ROLE_FILTERS } from "#/lib/api/admin/types";

type Raw = Record<string, unknown>;

function str(raw: unknown): string | undefined {
    if (typeof raw === "number" && Number.isFinite(raw)) return String(raw);
    return typeof raw === "string" && raw.length > 0 ? raw : undefined;
}

function oneOf<T extends string>(raw: unknown, allowed: readonly T[]): T | undefined {
    return typeof raw === "string" && (allowed as readonly string[]).includes(raw) ? (raw as T) : undefined;
}

/** Drops the keys whose value is `undefined`, so an unknown param never round-trips as `?x=`. */
function compact<T extends object>(value: T): T {
    return Object.fromEntries(Object.entries(value).filter(([, v]) => v !== undefined)) as T;
}

/**
 * Home reads no search params. It has no `validateSearch` either: a parser
 * returning `Record<string, never>` would join every route's search union and
 * forbid any key in a generic `navigate({ search })` elsewhere.
 */
export type IAdminHomeSearch = object;

export const USERS_FILTERS = ADMIN_USER_ROLE_FILTERS;
export type AdminUsersFilter = (typeof USERS_FILTERS)[number];

export interface IAdminUsersSearch {
    /** Selected user id; opens their sheet. */
    user?: string;
    q?: string;
    filter?: AdminUsersFilter;
    server?: string;
}

export function parseUsersSearch(search: Raw): IAdminUsersSearch {
    return compact({ user: str(search.user), q: str(search.q), filter: oneOf(search.filter, USERS_FILTERS), server: str(search.server) });
}

export const TIER_LISTS_TABS = ["lists", "access"] as const;
export type AdminTierListsTab = (typeof TIER_LISTS_TABS)[number];
export const TIER_LISTS_FILTERS = ["all", "hot"] as const;
export type AdminTierListsFilter = (typeof TIER_LISTS_FILTERS)[number];

export interface IAdminTierListsSearch {
    tab?: AdminTierListsTab;
    filter?: AdminTierListsFilter;
    q?: string;
}

export function parseTierListsSearch(search: Raw): IAdminTierListsSearch {
    return compact({ tab: oneOf(search.tab, TIER_LISTS_TABS), filter: oneOf(search.filter, TIER_LISTS_FILTERS), q: str(search.q) });
}

export const NOTES_STATUSES = ["all", "empty", "incomplete", "filled"] as const;
export type AdminNotesStatus = (typeof NOTES_STATUSES)[number];

export interface IAdminNotesSearch {
    /** Selected operator id. */
    op?: string;
    status?: AdminNotesStatus;
    q?: string;
}

export function parseNotesSearch(search: Raw): IAdminNotesSearch {
    return compact({ op: str(search.op), status: oneOf(search.status, NOTES_STATUSES), q: str(search.q) });
}

export const TRANSLATIONS_FILTERS = ["todo", "stale", "untranslated", "all"] as const;
export type AdminTranslationsFilter = (typeof TRANSLATIONS_FILTERS)[number];

export interface IAdminTranslationsSearch {
    locale?: string;
    filter?: AdminTranslationsFilter;
    /** Namespace; absent = every namespace. */
    ns?: string;
    /** Selected message key. */
    key?: string;
    q?: string;
}

export function parseTranslationsSearch(search: Raw): IAdminTranslationsSearch {
    return compact({ locale: str(search.locale), filter: oneOf(search.filter, TRANSLATIONS_FILTERS), ns: str(search.ns), key: str(search.key), q: str(search.q) });
}

export const SYSTEM_TABS = ["health", "audit", "languages"] as const;
export type AdminSystemTab = (typeof SYSTEM_TABS)[number];

export interface IAdminSystemSearch {
    tab?: AdminSystemTab;
}

export function parseSystemSearch(search: Raw): IAdminSystemSearch {
    return compact({ tab: oneOf(search.tab, SYSTEM_TABS) });
}
