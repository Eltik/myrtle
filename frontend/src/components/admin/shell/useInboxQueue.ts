import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { useAuth } from "#/hooks/use-auth";
import { healthQueryOptions, localesQueryOptions, translationPermissionsQueryOptions, translationProgressQueryOptions } from "#/lib/api/admin";
import { noteHasContent, operatorNotesListQueryOptions } from "#/lib/api/operator-notes";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { useGamedataServer } from "#/lib/i18n";
import type { TranslationGrant } from "#/types/generated/TranslationGrant";
import type { IAdminAccess } from "./access";
import { buildInboxQueue, emptyOperators, type InboxItem, type IQueueOperator, orphanLocaleGrants, queueLocales } from "./inbox";

/**
 * Operators with no note content, by name, for a role that can open notes.
 * `undefined` while loading or when `enabled` is false.
 */
export function useEmptyOperators(enabled: boolean): { empty: IQueueOperator[]; total: number } | undefined {
    const opsQuery = useQuery({ ...operatorsIndexQueryOptions(useGamedataServer()), enabled });
    const notesQuery = useQuery({ ...operatorNotesListQueryOptions(), enabled });
    return useMemo(() => {
        if (!enabled || !opsQuery.data || !notesQuery.data) return undefined;
        const filled = new Set(notesQuery.data.filter(noteHasContent).map((n) => n.operator_id));
        return { empty: emptyOperators(opsQuery.data, filled), total: opsQuery.data.length };
    }, [enabled, opsQuery.data, notesQuery.data]);
}

/**
 * The staff "Needs attention" queue (design `queue`). `undefined` for
 * non-staff and while any source it needs is still loading, so the Inbox
 * badge never counts a half-built queue.
 */
export function useInboxQueue(access: IAdminAccess): InboxItem[] | undefined {
    const { isAuthenticated } = useAuth();
    const staff = isAuthenticated && access.staff;
    // Only a super-admin holds locales (`myLoc`), so only they get translation items.
    const wantTranslations = staff && access.isSuper;
    const localesQuery = useQuery({ ...localesQueryOptions(isAuthenticated), enabled: wantTranslations });
    const progressQuery = useQuery({ ...translationProgressQueryOptions(isAuthenticated), enabled: wantTranslations });
    const empty = useEmptyOperators(staff && access.can.notes);
    const orphanGrants = useOrphanLocaleGrants(staff && access.canAssign);
    const healthQuery = useQuery({ ...healthQueryOptions(), enabled: staff });

    return useMemo(() => {
        if (!staff) return undefined;
        if (wantTranslations && (!localesQuery.data || !progressQuery.data)) return undefined;
        if (access.can.notes && !empty) return undefined;
        if (access.canAssign && !orphanGrants) return undefined;
        if (healthQuery.isPending) return undefined;
        return buildInboxQueue({
            staff,
            canNotes: access.can.notes,
            canAssign: access.canAssign,
            locales: wantTranslations ? queueLocales(access, localesQuery.data ?? []) : [],
            progress: progressQuery.data ?? [],
            emptyOps: empty?.empty ?? [],
            orphanGrants: orphanGrants ?? [],
            // A health check that fails outright is as degraded as one that says so.
            healthDegraded: healthQuery.isError || (healthQuery.data !== undefined && healthQuery.data.status !== "ok"),
        });
    }, [staff, wantTranslations, access, localesQuery.data, progressQuery.data, empty, orphanGrants, healthQuery.isPending, healthQuery.isError, healthQuery.data]);
}

/**
 * Language grants left behind by deleted accounts. Feeds the People nav badge
 * and the Home inbox's clean-up row, both super-admin only: pass `canAssign`
 * so no other role fetches every grant.
 */
export function useOrphanLocaleGrants(enabled: boolean): TranslationGrant[] | undefined {
    const { isAuthenticated } = useAuth();
    const on = enabled && isAuthenticated;
    const query = useQuery({ ...translationPermissionsQueryOptions(undefined, on), enabled: on });
    return useMemo(() => (on && query.data ? orphanLocaleGrants(query.data) : undefined), [on, query.data]);
}
