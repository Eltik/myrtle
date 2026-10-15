import { useMutation, useQueryClient } from "@tanstack/react-query";
import { invalidateLocaleGrants, invalidateTierListGrants } from "#/components/admin/shell/invalidate";
import { toastError, toastSuccess } from "#/components/admin/shell/toast";
import { useErrorMessage } from "#/components/ui/error-message";
import { grantTierListPermissionFn, grantTranslationPermissionFn, revokeTierListPermissionFn, revokeTranslationPermissionFn, type TierListPermissionLevel } from "#/lib/api/admin";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./Users.messages";

type UsersT = TypedT<typeof messages>;

interface IPersonGrantMutationsInput {
    personId: string;
    /** The person as the toasts name them. */
    name: string;
    localeName: (code: string) => string;
    /** After a grant (not a revoke) lands, before its toast: the sheet closes its picker. */
    onGranted: () => void;
}

/** The sheet's grant and revoke writes, tier-list and language, each with its toasts. */
export function usePersonGrantMutations({ personId, name, localeName, onGranted }: IPersonGrantMutationsInput) {
    const t: UsersT = useT("admin");
    const queryClient = useQueryClient();
    const describeError = useErrorMessage();

    const grantTl = useMutation({
        mutationFn: (input: { slug: string; title: string; permission: TierListPermissionLevel }) => grantTierListPermissionFn({ data: { slug: input.slug, userId: personId, permission: input.permission } }),
        onSuccess: (_data, input) => {
            invalidateTierListGrants(queryClient);
            onGranted();
            toastSuccess("tl-grant", t("users.toast.grantCreated"), t("users.toast.tlGranted.desc", { name, level: input.permission, title: input.title }));
        },
        onError: (err: unknown) => toastError("tl-grant-err", t("users.toast.grantFailed"), describeError(err)),
    });

    const revokeTl = useMutation({
        mutationFn: (input: { slug: string; title: string; permission: TierListPermissionLevel }) => revokeTierListPermissionFn({ data: { slug: input.slug, userId: personId, permission: input.permission } }),
        onSuccess: (_data, input) => {
            invalidateTierListGrants(queryClient);
            toastSuccess("tl-revoke", t("users.toast.tlRevoked"), t("users.toast.revoked.desc", { name, target: input.title }));
        },
        onError: (err: unknown) => toastError("tl-revoke-err", t("users.toast.revokeFailed"), describeError(err)),
    });

    const grantLoc = useMutation({
        mutationFn: (input: { locale: string; permission: TierListPermissionLevel }) => grantTranslationPermissionFn({ data: { locale: input.locale, userId: personId, permission: input.permission } }),
        onSuccess: (_data, input) => {
            invalidateLocaleGrants(queryClient);
            onGranted();
            toastSuccess("loc-grant", t("users.toast.grantCreated"), t("users.toast.locGranted.desc", { name, level: input.permission, language: localeName(input.locale) }));
        },
        onError: (err: unknown) => toastError("loc-grant-err", t("users.toast.grantFailed"), describeError(err)),
    });

    const revokeLoc = useMutation({
        mutationFn: (input: { locale: string; permission: TierListPermissionLevel }) => revokeTranslationPermissionFn({ data: { locale: input.locale, userId: personId, permission: input.permission } }),
        onSuccess: (_data, input) => {
            invalidateLocaleGrants(queryClient);
            toastSuccess("loc-revoke", t("users.toast.locRevoked"), t("users.toast.revoked.desc", { name, target: localeName(input.locale) }));
        },
        onError: (err: unknown) => toastError("loc-revoke-err", t("users.toast.revokeFailed"), describeError(err)),
    });

    return { grantTl, revokeTl, grantLoc, revokeLoc };
}
