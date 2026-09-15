import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { Locale } from "#/types/generated/Locale";
import type { LocaleProgress } from "#/types/generated/LocaleProgress";
import type { TranslationAuditResponse } from "#/types/generated/TranslationAuditResponse";
import type { TranslationEntry } from "#/types/generated/TranslationEntry";
import type { TranslationListResponse } from "#/types/generated/TranslationListResponse";
import type { TranslationPermission } from "#/types/generated/TranslationPermission";
import type { UiMessageAuditEntry } from "#/types/generated/UiMessageAuditEntry";
import { type IBackendStatus, parseError } from "../_shared";
import { requireSiteToken } from "../_shared.server";
import type { TierListPermissionLevel, UserRole } from "./types";

/** `all` | `untranslated` | `stale` | `translated` - mirrors the backend's filter. */
export type TranslationFilter = "all" | "untranslated" | "stale" | "translated";

export interface IListTranslationsInput {
    locale: string;
    namespace?: string;
    search?: string;
    filter?: TranslationFilter;
    limit?: number;
    offset?: number;
}

export interface IUpdateTranslationInput {
    locale: string;
    key: string;
    value: string;
}

export interface IClearTranslationInput {
    locale: string;
    key: string;
}

export interface ITranslationAuditInput {
    locale?: string;
    limit?: number;
    before?: string;
}

export interface IEntryAuditInput {
    locale: string;
    key: string;
}

export interface IGrantTranslationPermissionInput {
    locale: string;
    userId: string;
    permission: TierListPermissionLevel;
}

export interface ISetUserRoleInput {
    userId: string;
    role: UserRole;
}

/** One `details[]` item from a 422 `VALIDATION_FAILED` body. */
export interface ITranslationFieldError {
    field: string;
    message: string;
}

/**
 * The outcome of a translation write. A placeholder mismatch is not an
 * exceptional condition here - it is the editor's main feedback loop, and the
 * `details[]` it carries have to reach the field rather than a toast. Server
 * function rejections cross the boundary as bare messages, so the 422 comes
 * back as a value and every other status still throws.
 */
export type ITranslationSaveResult = { ok: true; entry: TranslationEntry } | { ok: false; message: string; details: ITranslationFieldError[] };

interface IValidationErrorBody {
    error?: {
        code?: string;
        message?: string;
        details?: ITranslationFieldError[];
    };
}

// ---------------------------------------------------------------- locales

export const getLocalesFn = createServerFn({ method: "GET" }).handler(async (): Promise<Locale[]> => {
    const token = requireSiteToken();
    const res = await backendFetch("/admin/i18n/locales", { bearerToken: token });
    if (!res.ok) throw await parseError(res);
    return (await res.json()) as Locale[];
});

export function localesQueryOptions(authed: boolean) {
    return queryOptions({
        queryKey: ["admin", "i18n", "locales", authed ? "auth" : "anon"],
        queryFn: () => getLocalesFn(),
        enabled: authed,
        staleTime: 5 * 60 * 1000,
        gcTime: 30 * 60 * 1000,
    });
}

export interface IUpsertLocaleInput {
    code: string;
    englishName: string;
    nativeName: string;
    fallbackLocale: string | null;
    gamedataServer: string;
    enabled: boolean;
    sortOrder?: number;
}

export const upsertLocaleFn = createServerFn({ method: "POST" })
    .inputValidator((data: IUpsertLocaleInput) => data)
    .handler(async ({ data }): Promise<Locale> => {
        const token = requireSiteToken();
        const res = await backendFetch("/admin/i18n/locales", {
            method: "PUT",
            bearerToken: token,
            body: JSON.stringify({
                code: data.code,
                english_name: data.englishName,
                native_name: data.nativeName,
                fallback_locale: data.fallbackLocale,
                gamedata_server: data.gamedataServer,
                enabled: data.enabled,
                sort_order: data.sortOrder,
            }),
        });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as Locale;
    });

/** The locale codes this caller may actually write, for the editor's picker. */
export const getWritableLocalesFn = createServerFn({ method: "GET" }).handler(async (): Promise<string[]> => {
    const token = requireSiteToken();
    const res = await backendFetch("/admin/i18n/writable-locales", { bearerToken: token });
    if (!res.ok) throw await parseError(res);
    return (await res.json()) as string[];
});

export function writableLocalesQueryOptions(authed: boolean) {
    return queryOptions({
        queryKey: ["admin", "i18n", "writable-locales", authed ? "auth" : "anon"],
        queryFn: () => getWritableLocalesFn(),
        enabled: authed,
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

// ---------------------------------------------------------------- messages

export const listTranslationsFn = createServerFn({ method: "GET" })
    .inputValidator((data: IListTranslationsInput) => data)
    .handler(async ({ data }): Promise<TranslationListResponse> => {
        const token = requireSiteToken();
        const params = new URLSearchParams({ locale: data.locale });
        if (data.namespace) params.set("namespace", data.namespace);
        if (data.search) params.set("search", data.search);
        if (data.filter) params.set("filter", data.filter);
        if (data.limit !== undefined) params.set("limit", String(data.limit));
        if (data.offset !== undefined) params.set("offset", String(data.offset));
        const res = await backendFetch(`/admin/i18n/messages?${params.toString()}`, { bearerToken: token });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as TranslationListResponse;
    });

export function translationsListQueryOptions(input: IListTranslationsInput, authed: boolean) {
    return queryOptions({
        queryKey: ["admin", "i18n", "messages", input, authed ? "auth" : "anon"],
        queryFn: () => listTranslationsFn({ data: input }),
        enabled: authed && input.locale.length > 0,
        staleTime: 15 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const updateTranslationFn = createServerFn({ method: "POST" })
    .inputValidator((data: IUpdateTranslationInput) => data)
    .handler(async ({ data }): Promise<ITranslationSaveResult> => {
        const token = requireSiteToken();
        const res = await backendFetch("/admin/i18n/message", {
            method: "PUT",
            bearerToken: token,
            body: JSON.stringify({ locale: data.locale, key: data.key, value: data.value }),
        });
        if (res.status === 422) {
            const body = (await res.json().catch(() => null)) as IValidationErrorBody | null;
            const details = body?.error?.details ?? [];
            return {
                ok: false,
                message: body?.error?.message ?? "The translation failed validation.",
                details: details.length > 0 ? details : [{ field: "value", message: body?.error?.message ?? "The translation failed validation." }],
            };
        }
        if (!res.ok) throw await parseError(res);
        return { ok: true, entry: (await res.json()) as TranslationEntry };
    });

export const clearTranslationFn = createServerFn({ method: "POST" })
    .inputValidator((data: IClearTranslationInput) => data)
    .handler(async ({ data }): Promise<IBackendStatus> => {
        const token = requireSiteToken();
        const res = await backendFetch("/admin/i18n/message/clear", {
            method: "POST",
            bearerToken: token,
            body: JSON.stringify({ locale: data.locale, key: data.key }),
        });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as IBackendStatus;
    });

// ---------------------------------------------------------------- namespaces

/**
 * `GET /admin/i18n/namespaces`. The whole namespace list, read from
 * `ui_message_keys` and independent of paging - a namespace list derived from
 * the rows on screen can only ever offer the namespaces the reader already
 * paged onto, which is the opposite of what the filter is for.
 */
export const getTranslationNamespacesFn = createServerFn({ method: "GET" }).handler(async (): Promise<string[]> => {
    const token = requireSiteToken();
    const res = await backendFetch("/admin/i18n/namespaces", { bearerToken: token });
    if (!res.ok) throw await parseError(res);
    return (await res.json()) as string[];
});

export function translationNamespacesQueryOptions(authed: boolean) {
    return queryOptions({
        queryKey: ["admin", "i18n", "namespaces", authed ? "auth" : "anon"],
        queryFn: () => getTranslationNamespacesFn(),
        enabled: authed,
        // The set only moves when the extractor syncs new keys, so this can sit
        // for minutes rather than being refetched alongside the message list.
        staleTime: 10 * 60 * 1000,
        gcTime: 30 * 60 * 1000,
    });
}

// ---------------------------------------------------------------- progress + audit

export const getTranslationProgressFn = createServerFn({ method: "GET" }).handler(async (): Promise<LocaleProgress[]> => {
    const token = requireSiteToken();
    const res = await backendFetch("/admin/i18n/progress", { bearerToken: token });
    if (!res.ok) throw await parseError(res);
    return (await res.json()) as LocaleProgress[];
});

export function translationProgressQueryOptions(authed: boolean) {
    return queryOptions({
        queryKey: ["admin", "i18n", "progress", authed ? "auth" : "anon"],
        queryFn: () => getTranslationProgressFn(),
        enabled: authed,
        staleTime: 30 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const getTranslationAuditLogFn = createServerFn({ method: "GET" })
    .inputValidator((data: ITranslationAuditInput) => data)
    .handler(async ({ data }): Promise<TranslationAuditResponse> => {
        const token = requireSiteToken();
        const params = new URLSearchParams();
        if (data.locale) params.set("locale", data.locale);
        if (data.limit !== undefined) params.set("limit", String(data.limit));
        if (data.before) params.set("before", data.before);
        const query = params.toString();
        const res = await backendFetch(`/admin/i18n/audit${query ? `?${query}` : ""}`, { bearerToken: token });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as TranslationAuditResponse;
    });

export function translationAuditLogQueryOptions(input: ITranslationAuditInput, authed: boolean) {
    return queryOptions({
        queryKey: ["admin", "i18n", "audit", input, authed ? "auth" : "anon"],
        queryFn: () => getTranslationAuditLogFn({ data: input }),
        enabled: authed,
        staleTime: 30 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

/**
 * Per-key history. Every row carries the value it replaced, which is what
 * makes the editor's revert an ordinary write of `old_value`.
 */
export const getTranslationEntryAuditFn = createServerFn({ method: "GET" })
    .inputValidator((data: IEntryAuditInput) => data)
    .handler(async ({ data }): Promise<UiMessageAuditEntry[]> => {
        const token = requireSiteToken();
        const params = new URLSearchParams({ locale: data.locale, key: data.key });
        const res = await backendFetch(`/admin/i18n/audit/entry?${params.toString()}`, { bearerToken: token });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as UiMessageAuditEntry[];
    });

export function translationEntryAuditQueryOptions(input: IEntryAuditInput, authed: boolean) {
    return queryOptions({
        queryKey: ["admin", "i18n", "audit", "entry", input.locale, input.key, authed ? "auth" : "anon"],
        queryFn: () => getTranslationEntryAuditFn({ data: input }),
        enabled: authed && input.locale.length > 0 && input.key.length > 0,
        staleTime: 15 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

// ---------------------------------------------------------------- per-locale grants

export const getTranslationPermissionsFn = createServerFn({ method: "GET" })
    .inputValidator((locale: string | undefined) => locale)
    .handler(async ({ data: locale }): Promise<TranslationPermission[]> => {
        const token = requireSiteToken();
        const query = locale ? `?locale=${encodeURIComponent(locale)}` : "";
        const res = await backendFetch(`/admin/i18n/permissions${query}`, { bearerToken: token });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as TranslationPermission[];
    });

export function translationPermissionsQueryOptions(locale: string | undefined, authed: boolean) {
    return queryOptions({
        queryKey: ["admin", "i18n", "permissions", locale ?? "all", authed ? "auth" : "anon"],
        queryFn: () => getTranslationPermissionsFn({ data: locale }),
        enabled: authed,
        staleTime: 30 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const grantTranslationPermissionFn = createServerFn({ method: "POST" })
    .inputValidator((data: IGrantTranslationPermissionInput) => data)
    .handler(async ({ data }): Promise<IBackendStatus> => {
        const token = requireSiteToken();
        const res = await backendFetch("/admin/i18n/permissions", {
            method: "POST",
            bearerToken: token,
            body: JSON.stringify({ locale: data.locale, user_id: data.userId, permission: data.permission }),
        });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as IBackendStatus;
    });

export const revokeTranslationPermissionFn = createServerFn({ method: "POST" })
    .inputValidator((data: IGrantTranslationPermissionInput) => data)
    .handler(async ({ data }): Promise<IBackendStatus> => {
        const token = requireSiteToken();
        const res = await backendFetch("/admin/i18n/permissions/revoke", {
            method: "POST",
            bearerToken: token,
            body: JSON.stringify({ locale: data.locale, user_id: data.userId, permission: data.permission }),
        });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as IBackendStatus;
    });

// ---------------------------------------------------------------- global role

/**
 * `PUT /admin/users/{user_id}/role`. Super-admin only, and the backend
 * refuses to change your own row with a 400 rather than letting you
 * self-demote out of the panel.
 */
export const setUserRoleFn = createServerFn({ method: "POST" })
    .inputValidator((data: ISetUserRoleInput) => data)
    .handler(async ({ data }): Promise<IBackendStatus> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/admin/users/${encodeURIComponent(data.userId)}/role`, {
            method: "PUT",
            bearerToken: token,
            body: JSON.stringify({ role: data.role }),
        });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as IBackendStatus;
    });
