import { EyeIcon } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "#/components/ui/alert";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Switch } from "#/components/ui/switch";
import type { IUpdateUserSettingsInput } from "#/lib/api/auth";
import { Mono, SettingRow } from "./SettingsShell";

interface IPrivacyPanelProps {
    settings: IUpdateUserSettingsInput;
    onChange: (next: IUpdateUserSettingsInput) => void;
    saving: boolean;
}

export function PrivacyPanel({ settings, onChange, saving }: IPrivacyPanelProps) {
    return (
        <div className="flex flex-col gap-4">
            <Alert variant="info">
                <EyeIcon />
                <AlertTitle>What's public, and what isn't</AlertTitle>
                <AlertDescription>Public profiles show your nickname, UID, roster, account scores, and leaderboard rankings. Authentication tokens, settings, saved DPS configs, and email are always private.</AlertDescription>
            </Alert>

            <Card>
                <CardHeader>
                    <CardTitle>Profile visibility</CardTitle>
                    <CardDescription>
                        Who can see your roster and account stats. Backed by <Mono>user_settings.public_profile</Mono>.
                    </CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <SettingRow layout="inline" title="Public profile" description="Lets anyone with your UID open your profile, see your roster, and look up your scores." control={<Switch checked={settings.public_profile} onCheckedChange={(v) => onChange({ ...settings, public_profile: v })} disabled={saving} />} />
                    <SettingRow
                        layout="inline"
                        title="Show on leaderboards"
                        description={
                            <>
                                Opt in to ranked appearance on <Mono>/user/leaderboard</Mono>. Scores are still calculated either way. Backed by <Mono>share_stats</Mono>.
                            </>
                        }
                        control={<Switch checked={settings.share_stats} onCheckedChange={(v) => onChange({ ...settings, share_stats: v })} disabled={saving} />}
                    />
                    <SettingRow
                        layout="inline"
                        title="Store gacha history"
                        description={
                            <>
                                Saves your synced pulls so you can browse them in Gacha → History and contribute aggregate stats. Backed by <Mono>store_gacha</Mono>.
                            </>
                        }
                        control={<Switch checked={settings.store_gacha} onCheckedChange={(v) => onChange({ ...settings, store_gacha: v })} disabled={saving} />}
                    />
                </CardContent>
            </Card>
        </div>
    );
}
