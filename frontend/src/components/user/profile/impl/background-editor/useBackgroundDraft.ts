import { useMutation } from "@tanstack/react-query";
import { useCallback, useState } from "react";
import { useErrorMessage } from "#/components/ui/error-message";
import { toastManager } from "#/components/ui/toast";
import { useInvalidateProfile } from "#/hooks/use-resync-roster";
import { updateUserSettingsFn } from "#/lib/api/auth";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { ProfileBackground } from "#/types/generated/ProfileBackground";
import type { ProfileBackgroundKind } from "#/types/generated/ProfileBackgroundKind";
import { draftDirty, pickBackground } from "../background";
import type { messages } from "./BackgroundEditor.messages";

/**
 * The editor's unsaved background, from the saved one. Nothing is stored until `save`,
 * which sends the `background` key alone (`{ profile_layout: { background } }`; the
 * backend keeps the stored tabs and showcase), refetches the profile and calls `onSaved`.
 */
export function useBackgroundDraft(saved: ProfileBackground | null, onSaved: () => void) {
    const t: TypedT<typeof messages> = useT("user");
    const describeError = useErrorMessage();
    const invalidateProfile = useInvalidateProfile();
    const [draft, setDraft] = useState<ProfileBackground | null>(saved);

    const mutation = useMutation({
        mutationFn: (background: ProfileBackground | null) => updateUserSettingsFn({ data: { profile_layout: { background } } }),
        onSuccess: async (_ok, background) => {
            await invalidateProfile();
            toastManager.add({ id: `profile-background-${Date.now()}`, title: background ? t("profile.background.saved.title") : t("profile.background.removed.title"), type: "success" });
            onSaved();
        },
        onError: (err: unknown) => {
            toastManager.add({ id: `profile-background-err-${Date.now()}`, title: t("profile.background.saveFailed.title"), description: describeError(err), type: "error" });
        },
    });

    // Stable for the editor's life, so the memoized art browser and strip skip a drag's renders.
    const pick = useCallback((kind: ProfileBackgroundKind, id: string) => setDraft((prev) => pickBackground(prev, kind, id)), []);
    const adjust = useCallback((next: ProfileBackground) => setDraft(next), []);
    const remove = useCallback(() => setDraft(null), []);

    return {
        draft,
        dirty: draftDirty(saved, draft),
        saving: mutation.isPending,
        /** A tile picked: a new art starts at its kind's default crop; the art already shown keeps its crop. */
        pick,
        adjust,
        remove,
        save: () => mutation.mutate(draft),
    };
}
