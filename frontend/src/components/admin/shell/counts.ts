import { useQuery } from "@tanstack/react-query";
import { useAuth } from "#/hooks/use-auth";
import { localesQueryOptions, translationProgressQueryOptions } from "#/lib/api/admin";
import type { IAdminAccess } from "./access";
import { peopleItems } from "./inbox";
import { translationsTodo } from "./model";
import { useEmptyOperators, useInboxQueue, useOrphanLocaleGrants } from "./useInboxQueue";

/*
 * Nav badge counts. Each hook answers `undefined` while loading or when the
 * count does not apply, and the bar hides a badge that is `undefined` or 0.
 * Sections that compute the same number should reuse the pure helpers in
 * `./model` so the badge and the list it points at cannot disagree.
 */

export function useNotesMissingCount(access: IAdminAccess): number | undefined {
    return useEmptyOperators(access.can.notes)?.empty.length;
}

export function useTranslationsTodoCount(access: IAdminAccess): number | undefined {
    const { isAuthenticated } = useAuth();
    const enabled = isAuthenticated && access.can.translations;
    const progressQuery = useQuery({ ...translationProgressQueryOptions(isAuthenticated), enabled });
    // Only a super-admin's count depends on which locales are public.
    const localesQuery = useQuery({ ...localesQueryOptions(isAuthenticated), enabled: enabled && access.isSuper });
    if (!enabled || !progressQuery.data) return undefined;
    if (access.isSuper && !localesQuery.data) return undefined;
    const publicLocales = new Set((localesQuery.data ?? []).filter((l) => l.enabled).map((l) => l.code));
    return translationsTodo(access, progressQuery.data, publicLocales);
}

/**
 * The People rows of the Home inbox (one per thing to act on, not one per
 * person), shown to super-admins on the People tab.
 */
export function usePeopleWarnedCount(access: IAdminAccess): number | undefined {
    const orphans = useOrphanLocaleGrants(access.canAssign);
    return orphans === undefined ? undefined : peopleItems(access.canAssign, orphans).length;
}

/** Length of the staff Inbox "Needs attention" queue (design `queue.length`). */
export function useInboxQueueCount(access: IAdminAccess): number | undefined {
    return useInboxQueue(access)?.length;
}
