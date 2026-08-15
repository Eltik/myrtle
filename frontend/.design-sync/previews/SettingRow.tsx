import { CheckIcon, RefreshCwIcon } from "lucide-react";
import { Badge, Button, Card, CardContent, CardDescription, CardHeader, CardTitle, Input, SettingRow, Switch } from "frontend";

const noop = () => {};

/** `layout="auto"`: label above the control on mobile, two columns from `sm:` up. */
export const Stacked = () => (
    <Card>
        <CardHeader>
            <CardTitle>Game-synced info</CardTitle>
            <CardDescription>Read-only. Pulled from Yostar when you sync - change it in-game.</CardDescription>
        </CardHeader>
        <CardContent className="pt-0">
            <SettingRow title="Arknights nickname" description="Shown on your profile, leaderboard, and tier lists you publish." control={<Input value="Eltik#1734" readOnly className="w-full sm:w-65" />} />
            <SettingRow title="Account level" description="Doctor level from the in-game profile." control={<Input value="Lv. 120" readOnly className="w-full sm:w-30" />} />
            <SettingRow title="Game server" description="Which Arknights server you're synced from. Re-link your account to change this." control={<Input value="EN · Yostar Global" readOnly className="w-full sm:w-55" />} />
        </CardContent>
    </Card>
);

/** `layout="inline"`: label and control always share one row - used for switches and badges. */
export const Inline = () => (
    <Card>
        <CardHeader>
            <CardTitle>Profile visibility</CardTitle>
            <CardDescription>Who can see your roster and account stats.</CardDescription>
        </CardHeader>
        <CardContent className="pt-0">
            <SettingRow layout="inline" title="Public profile" description="Lets anyone with your UID open your profile, see your roster, and look up your scores." control={<Switch checked={true} onCheckedChange={noop} />} />
            <SettingRow layout="inline" title="Show on leaderboards" description="Opt in to ranked appearance on /user/leaderboard. Scores are still calculated either way." control={<Switch checked={false} onCheckedChange={noop} />} />
        </CardContent>
    </Card>
);

export const WithButtonControls = () => (
    <Card>
        <CardHeader>
            <CardTitle>Sync &amp; refresh</CardTitle>
            <CardDescription>Pull the latest from Yostar. Re-sync any time - we replace your stored snapshot.</CardDescription>
        </CardHeader>
        <CardContent className="pt-0">
            <SettingRow
                title="Re-sync game data"
                description="Operators, stage progress, IS/Sandbox, base, medals, inventory."
                control={
                    <Button size="sm" className="w-full sm:w-auto">
                        <RefreshCwIcon className="size-3.5" />
                        Re-sync now
                    </Button>
                }
            />
            <SettingRow
                title="Re-syncing"
                description="The control is disabled and shows a spinner while the mutation is in flight."
                control={
                    <Button size="sm" disabled loading className="w-full sm:w-auto">
                        <RefreshCwIcon className="size-3.5" />
                        Re-syncing…
                    </Button>
                }
            />
            <SettingRow
                layout="inline"
                title="Yostar OAuth"
                description="Authenticated via your in-game email verification code."
                control={
                    <Badge variant="success">
                        <CheckIcon className="size-3" /> Active
                    </Badge>
                }
            />
        </CardContent>
    </Card>
);
