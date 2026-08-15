import { DatabaseIcon, PaletteIcon, ShieldIcon, TriangleAlertIcon, UserRoundIcon } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, DangerPanel, SettingRow, SettingsShell, Switch } from "frontend";

const NAV = [
    { id: "profile" as const, label: "Profile", Icon: UserRoundIcon },
    { id: "appearance" as const, label: "Appearance", Icon: PaletteIcon },
    { id: "privacy" as const, label: "Privacy", Icon: ShieldIcon },
    { id: "data" as const, label: "Account & data", Icon: DatabaseIcon },
    { id: "danger" as const, label: "Danger zone", Icon: TriangleAlertIcon },
];

const noop = () => {};

export const ProfileSection = () => (
    <SettingsShell nav={NAV} active="profile" onChange={noop}>
        <Card>
            <CardHeader>
                <CardTitle>Game-synced info</CardTitle>
                <CardDescription>Read-only. Pulled from Yostar when you sync - change it in-game.</CardDescription>
            </CardHeader>
            <CardContent className="pt-0">
                <SettingRow layout="inline" title="Arknights nickname" description="Shown on your profile, leaderboard, and tier lists you publish." control={<span className="font-mono text-[13px] text-muted-foreground">Eltik#1734</span>} />
                <SettingRow layout="inline" title="Account level" description="Doctor level from the in-game profile." control={<span className="font-mono text-[13px] text-muted-foreground">Lv. 120</span>} />
                <SettingRow layout="inline" title="Assistant operator" description="The operator displayed on your in-game and Myrtle profile." control={<span className="font-mono text-[13px] text-muted-foreground">char_4064_mlynar</span>} />
            </CardContent>
        </Card>
    </SettingsShell>
);

export const PrivacySection = () => (
    <SettingsShell nav={NAV} active="privacy" onChange={noop}>
        <Card>
            <CardHeader>
                <CardTitle>Profile visibility</CardTitle>
                <CardDescription>Who can see your roster and account stats.</CardDescription>
            </CardHeader>
            <CardContent className="pt-0">
                <SettingRow layout="inline" title="Public profile" description="Lets anyone with your UID open your profile, see your roster, and look up your scores." control={<Switch checked={true} onCheckedChange={noop} />} />
                <SettingRow layout="inline" title="Show on leaderboards" description="Opt in to ranked appearance on /user/leaderboard. Scores are still calculated either way." control={<Switch checked={true} onCheckedChange={noop} />} />
                <SettingRow layout="inline" title="Store gacha history" description="Saves your synced pulls so you can browse them in Gacha → History." control={<Switch checked={false} onCheckedChange={noop} />} />
            </CardContent>
        </Card>
    </SettingsShell>
);

/** The danger nav item is separated by a rule and tints destructive when active. */
export const DangerSection = () => (
    <SettingsShell nav={NAV} active="danger" onChange={noop}>
        <DangerPanel />
    </SettingsShell>
);

/** Signed out: appearance is a client-side preference, so it is the only section offered. */
export const AppearanceOnly = () => (
    <SettingsShell nav={NAV.filter((n) => n.id === "appearance")} active="appearance" onChange={noop}>
        <Card>
            <CardHeader>
                <CardTitle>Theme</CardTitle>
                <CardDescription>Pick light, dark, or follow your system. Sign in to manage profile, privacy and account data.</CardDescription>
            </CardHeader>
        </Card>
    </SettingsShell>
);
