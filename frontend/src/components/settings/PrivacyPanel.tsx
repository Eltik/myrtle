import { EyeIcon } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "#/components/ui/alert";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Switch } from "#/components/ui/switch";
import type { IUpdateUserSettingsInput } from "#/lib/api/auth";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./PrivacyPanel.messages";
import { Mono, SettingRow } from "./SettingsShell";

interface IPrivacyPanelProps {
    settings: IUpdateUserSettingsInput;
    onChange: (next: IUpdateUserSettingsInput) => void;
    saving: boolean;
}

export function PrivacyPanel({ settings, onChange, saving }: IPrivacyPanelProps) {
    const t: TypedT<typeof messages> = useT("settings");
    const rt: TypedRichT<typeof messages> = useRichT("settings");

    return (
        <div className="flex flex-col gap-4">
            <Alert variant="info">
                <EyeIcon />
                <AlertTitle>{t("privacy.alert.title")}</AlertTitle>
                <AlertDescription>{t("privacy.alert.body")}</AlertDescription>
            </Alert>

            <Card>
                <CardHeader>
                    <CardTitle>{t("privacy.visibility.title")}</CardTitle>
                    <CardDescription>{t("privacy.visibility.desc")}</CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <SettingRow
                        layout="inline"
                        title={t("privacy.publicProfile.title")}
                        description={t("privacy.publicProfile.desc")}
                        control={<Switch checked={settings.public_profile} onCheckedChange={(v) => onChange({ ...settings, public_profile: v })} disabled={saving} aria-label={t("privacy.publicProfile.title")} />}
                    />
                    <SettingRow
                        layout="inline"
                        title={t("privacy.leaderboards.title")}
                        description={rt("privacy.leaderboards.desc", { path: <Mono>{t("privacy.leaderboards.path")}</Mono> })}
                        control={<Switch checked={settings.share_stats} onCheckedChange={(v) => onChange({ ...settings, share_stats: v })} disabled={saving} aria-label={t("privacy.leaderboards.title")} />}
                    />
                    <SettingRow layout="inline" title={t("privacy.gacha.title")} description={t("privacy.gacha.desc")} control={<Switch checked={settings.store_gacha} onCheckedChange={(v) => onChange({ ...settings, store_gacha: v })} disabled={saving} aria-label={t("privacy.gacha.title")} />} />
                    <SettingRow layout="inline" title={t("privacy.publicGacha.title")} description={t("privacy.publicGacha.desc")} control={<Switch checked={settings.public_gacha} onCheckedChange={(v) => onChange({ ...settings, public_gacha: v })} disabled={saving} aria-label={t("privacy.publicGacha.title")} />} />
                </CardContent>
            </Card>
        </div>
    );
}
