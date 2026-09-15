import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { DatabaseIcon, PaletteIcon, ShieldIcon, TriangleAlertIcon, UserRoundIcon } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { disconnectGameAccountFn, type IUpdateUserSettingsInput, refreshRosterFn, updateUserSettingsFn } from "#/lib/api/auth";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IUserProfile } from "#/types/user";
import { AppearancePanel } from "./AppearancePanel";
import { DangerPanel } from "./DangerPanel";
import { DataPanel } from "./DataPanel";
import { PrivacyPanel } from "./PrivacyPanel";
import { ProfilePanel } from "./ProfilePanel";
import type { messages } from "./SettingsPage.messages";
import { type SettingsSectionId, SettingsShell } from "./SettingsShell";

/**
 * Section order and icons. The labels are message keys rather than text, so
 * this table can stay a module constant; `SettingsPage` resolves them.
 */
const NAV = [
    { id: "profile" as const, labelKey: "nav.profile" as const, Icon: UserRoundIcon },
    { id: "appearance" as const, labelKey: "nav.appearance" as const, Icon: PaletteIcon },
    { id: "privacy" as const, labelKey: "nav.privacy" as const, Icon: ShieldIcon },
    { id: "data" as const, labelKey: "nav.data" as const, Icon: DatabaseIcon },
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
    const queryClient = useQueryClient();
    const t: TypedT<typeof messages> = useT("settings");

    // Appearance is a client-side preference (theme, accent, dynamic art) and
    // is available to everyone; the account sections require signing in.
    const nav = useMemo(() => (user ? NAV : NAV.filter((n) => n.id === "appearance")).map(({ id, labelKey, Icon }) => ({ id, label: t(labelKey), Icon })), [t, user]);

    const [active, setActive] = useState<SettingsSectionId>(user ? "profile" : "appearance");
    const [settings, setSettings] = useState<IUpdateUserSettingsInput>(() => initialSettings(user));
    const [signingOut, setSigningOut] = useState(false);

    useEffect(() => {
        setSettings(initialSettings(user));
    }, [user]);

    const settingsMutation = useMutation({
        mutationFn: (next: IUpdateUserSettingsInput) => updateUserSettingsFn({ data: next }),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["user"] });
            toastManager.add({
                id: `settings-saved-${Date.now()}`,
                title: t("toast.saved.title"),
                description: t("toast.saved.body"),
                type: "success",
            });
        },
        onError: (err: unknown) => {
            setSettings(initialSettings(user));
            toastManager.add({
                id: `settings-err-${Date.now()}`,
                title: t("toast.saveFailed.title"),
                description: err instanceof Error ? err.message : String(err),
                type: "error",
            });
        },
    });

    const resyncMutation = useMutation({
        mutationFn: () => refreshRosterFn(),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["user"] });
            toastManager.add({
                id: `resync-${Date.now()}`,
                title: t("toast.resynced.title"),
                description: t("toast.resynced.body"),
                type: "success",
            });
        },
        onError: (err: unknown) =>
            toastManager.add({
                id: `resync-err-${Date.now()}`,
                title: t("toast.resyncFailed.title"),
                description: err instanceof Error ? err.message : String(err),
                type: "error",
            }),
    });

    const disconnectMutation = useMutation({
        mutationFn: () => disconnectGameAccountFn(),
        onSuccess: ({ removed }) => {
            toastManager.add({
                id: `disconnect-${Date.now()}`,
                title: removed ? t("toast.disconnected.title") : t("toast.nothingToDisconnect.title"),
                description: removed ? t("toast.disconnected.body") : t("toast.nothingToDisconnect.body"),
                type: "success",
            });
        },
        onError: (err: unknown) =>
            toastManager.add({
                id: `disconnect-err-${Date.now()}`,
                title: t("toast.disconnectFailed.title"),
                description: err instanceof Error ? err.message : String(err),
                type: "error",
            }),
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
            toastManager.add({
                id: `signout-err-${Date.now()}`,
                title: t("toast.signOutFailed.title"),
                description: err instanceof Error ? err.message : String(err),
                type: "error",
            });
        }
    };

    return (
        <SettingsShell nav={nav} active={active} onChange={setActive}>
            {user && active === "profile" && <ProfilePanel user={user} onResync={() => resyncMutation.mutate()} syncing={resyncMutation.isPending} />}
            {active === "appearance" && <AppearancePanel />}
            {user && active === "privacy" && <PrivacyPanel settings={settings} onChange={handleSettingsChange} saving={settingsMutation.isPending} />}
            {user && active === "data" && <DataPanel user={user} onResync={() => resyncMutation.mutate()} syncing={resyncMutation.isPending} onSignOut={handleSignOut} signingOut={signingOut} onDisconnect={() => disconnectMutation.mutate()} disconnecting={disconnectMutation.isPending} />}
            {user && active === "danger" && <DangerPanel />}
        </SettingsShell>
    );
}
