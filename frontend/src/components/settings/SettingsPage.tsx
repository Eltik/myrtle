import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { PaletteIcon, ShieldIcon, TriangleAlertIcon, UserRoundIcon } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useErrorMessage } from "#/components/ui/error-message";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { useInvalidateProfile, useResyncRoster } from "#/hooks/use-resync-roster";
import { disconnectGameAccountFn, type IUpdateUserSettingsInput, updateUserSettingsFn } from "#/lib/api/auth";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { forgetSession } from "#/lib/root-context";
import type { IUserProfile } from "#/types/user";
import { AccountPanel } from "./AccountPanel";
import { AppearancePanel } from "./AppearancePanel";
import { DangerPanel } from "./DangerPanel";
import { PrivacyPanel } from "./PrivacyPanel";
import type { messages } from "./SettingsPage.messages";
import { type SettingsSectionId, SettingsShell } from "./SettingsShell";

/**
 * Section order and icons. The labels are message keys rather than text, so
 * this table can stay a module constant; `SettingsPage` resolves them.
 */
const NAV = [
    // First and the default.
    { id: "account" as const, labelKey: "nav.account" as const, Icon: UserRoundIcon },
    { id: "appearance" as const, labelKey: "nav.appearance" as const, Icon: PaletteIcon },
    { id: "privacy" as const, labelKey: "nav.privacy" as const, Icon: ShieldIcon },
    { id: "danger" as const, labelKey: "nav.danger" as const, Icon: TriangleAlertIcon },
];

function initialSettings(user: IUserProfile | null): IUpdateUserSettingsInput {
    return {
        public_profile: user?.public_profile ?? true,
        store_gacha: user?.store_gacha ?? true,
        share_stats: user?.share_stats ?? true,
    };
}

export function SettingsPage({ user }: { user: IUserProfile | null }) {
    const navigate = useNavigate();
    const { logout } = useAuth();
    const t: TypedT<typeof messages> = useT("settings");
    const describeError = useErrorMessage();

    // Appearance is a client-side preference (theme, accent, dynamic art) and
    // is available to everyone; the account sections require signing in.
    const nav = useMemo(() => (user ? NAV : NAV.filter((n) => n.id === "appearance")).map(({ id, labelKey, Icon }) => ({ id, label: t(labelKey), Icon })), [t, user]);

    const [active, setActive] = useState<SettingsSectionId>(user ? "account" : "appearance");
    const [settings, setSettings] = useState<IUpdateUserSettingsInput>(() => initialSettings(user));
    const [signingOut, setSigningOut] = useState(false);

    useEffect(() => {
        setSettings(initialSettings(user));
    }, [user]);

    const invalidateProfile = useInvalidateProfile();
    const toastSuccess = (id: string, title: string, description: string) => toastManager.add({ id: `${id}-${Date.now()}`, title, description, type: "success" });
    const toastError = (id: string, title: string, err: unknown) => toastManager.add({ id: `${id}-err-${Date.now()}`, title, description: describeError(err), type: "error" });

    const settingsMutation = useMutation({
        mutationFn: (next: IUpdateUserSettingsInput) => updateUserSettingsFn({ data: next }),
        onSuccess: () => {
            void invalidateProfile();
            toastSuccess("settings-saved", t("toast.saved.title"), t("toast.saved.body"));
        },
        onError: (err: unknown) => {
            setSettings(initialSettings(user));
            toastError("settings", t("toast.saveFailed.title"), err);
        },
    });

    const resyncMutation = useResyncRoster({
        onSuccess: () => toastSuccess("resync", t("toast.resynced.title"), t("toast.resynced.body")),
        onError: (err) => toastError("resync", t("toast.resyncFailed.title"), err),
    });

    const disconnectMutation = useMutation({
        mutationFn: () => disconnectGameAccountFn(),
        onSuccess: ({ removed }) => {
            forgetSession();
            if (removed) toastSuccess("disconnect", t("toast.disconnected.title"), t("toast.disconnected.body"));
            else toastSuccess("disconnect", t("toast.nothingToDisconnect.title"), t("toast.nothingToDisconnect.body"));
        },
        onError: (err: unknown) => toastError("disconnect", t("toast.disconnectFailed.title"), err),
    });

    const handleSettingsChange = (next: IUpdateUserSettingsInput) => {
        setSettings(next);
        settingsMutation.mutate(next);
    };

    const handleSignOut = async () => {
        setSigningOut(true);
        try {
            await logout();
            await navigate({ to: "/" });
        } catch (err) {
            setSigningOut(false);
            toastError("signout", t("toast.signOutFailed.title"), err);
        }
    };

    return (
        <SettingsShell nav={nav} active={active} onChange={setActive}>
            {user && active === "account" && <AccountPanel user={user} onResync={() => resyncMutation.mutate()} syncing={resyncMutation.isPending} onSignOut={handleSignOut} signingOut={signingOut} onDisconnect={() => disconnectMutation.mutate()} disconnecting={disconnectMutation.isPending} />}
            {active === "appearance" && <AppearancePanel />}
            {user && active === "privacy" && <PrivacyPanel settings={settings} onChange={handleSettingsChange} saving={settingsMutation.isPending} />}
            {user && active === "danger" && <DangerPanel />}
        </SettingsShell>
    );
}
