import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { DatabaseIcon, PaletteIcon, ShieldIcon, TriangleAlertIcon, UserRoundIcon } from "lucide-react";
import { useEffect, useState } from "react";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { type IUpdateUserSettingsInput, refreshRosterFn, updateUserSettingsFn } from "#/lib/api/auth";
import type { IUserProfile } from "#/types/user";
import { AppearancePanel } from "./AppearancePanel";
import { DangerPanel } from "./DangerPanel";
import { DataPanel } from "./DataPanel";
import { PrivacyPanel } from "./PrivacyPanel";
import { ProfilePanel } from "./ProfilePanel";
import { type SettingsSectionId, SettingsShell } from "./SettingsShell";

const NAV = [
    { id: "profile" as const, label: "Profile", Icon: UserRoundIcon },
    { id: "appearance" as const, label: "Appearance", Icon: PaletteIcon },
    { id: "privacy" as const, label: "Privacy", Icon: ShieldIcon },
    { id: "data" as const, label: "Account & data", Icon: DatabaseIcon },
    { id: "danger" as const, label: "Danger zone", Icon: TriangleAlertIcon },
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

    // Appearance is a client-side preference (theme, accent, dynamic art) and
    // is available to everyone; the account sections require signing in.
    const nav = user ? NAV : NAV.filter((n) => n.id === "appearance");

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
                title: "Settings saved",
                description: "Your privacy preferences are up to date.",
                type: "success",
            });
        },
        onError: (err: unknown) => {
            setSettings(initialSettings(user));
            toastManager.add({
                id: `settings-err-${Date.now()}`,
                title: "Couldn't save settings",
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
                title: "Roster re-synced",
                description: "Pulled the latest snapshot from Yostar.",
                type: "success",
            });
        },
        onError: (err: unknown) =>
            toastManager.add({
                id: `resync-err-${Date.now()}`,
                title: "Couldn't re-sync",
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
                title: "Couldn't sign out",
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
            {user && active === "data" && <DataPanel user={user} onResync={() => resyncMutation.mutate()} syncing={resyncMutation.isPending} onSignOut={handleSignOut} signingOut={signingOut} />}
            {user && active === "danger" && <DangerPanel />}
        </SettingsShell>
    );
}
