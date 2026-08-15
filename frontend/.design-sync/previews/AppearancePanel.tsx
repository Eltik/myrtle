import { PaletteIcon, ShieldIcon, TriangleAlertIcon, UserRoundIcon } from "lucide-react";
import { AppearancePanel, SettingsShell } from "frontend";

const NAV = [
    { id: "profile" as const, label: "Profile", Icon: UserRoundIcon },
    { id: "appearance" as const, label: "Appearance", Icon: PaletteIcon },
    { id: "privacy" as const, label: "Privacy", Icon: ShieldIcon },
    { id: "danger" as const, label: "Danger zone", Icon: TriangleAlertIcon },
];

const noop = () => {};

export const Default = () => <AppearancePanel />;

/** Where it actually lives: the Appearance tab of /settings. */
export const InSettingsShell = () => (
    <SettingsShell nav={NAV} active="appearance" onChange={noop}>
        <AppearancePanel />
    </SettingsShell>
);
