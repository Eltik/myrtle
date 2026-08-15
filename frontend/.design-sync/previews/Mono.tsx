import { Card, CardContent, CardDescription, CardHeader, CardTitle, Mono, SettingRow, Switch } from "frontend";

const noop = () => {};

/** `Mono` marks up a column or table name inside a card description. */
export const InCardDescription = () => (
    <Card>
        <CardHeader>
            <CardTitle>Profile visibility</CardTitle>
            <CardDescription>
                Who can see your roster and account stats. Backed by <Mono>user_settings.public_profile</Mono>.
            </CardDescription>
        </CardHeader>
        <CardContent className="pt-0">
            <SettingRow layout="inline" title="Public profile" description="Lets anyone with your UID open your profile, see your roster, and look up your scores." control={<Switch checked={true} onCheckedChange={noop} />} />
        </CardContent>
    </Card>
);

/** …and routes or e-mail addresses inside a setting row's description. */
export const InSettingRow = () => (
    <Card>
        <CardHeader>
            <CardTitle>Leaderboards &amp; deletion</CardTitle>
            <CardDescription>Two places where the copy has to name an exact identifier.</CardDescription>
        </CardHeader>
        <CardContent className="pt-0">
            <SettingRow
                layout="inline"
                title="Show on leaderboards"
                description={
                    <>
                        Opt in to ranked appearance on <Mono>/user/leaderboard</Mono>. Scores are still calculated either way. Backed by <Mono>share_stats</Mono>.
                    </>
                }
                control={<Switch checked={true} onCheckedChange={noop} />}
            />
            <SettingRow
                layout="inline"
                title="Delete account"
                description={
                    <>
                        Email <Mono>privacy@myrtle.moe</Mono> from the address linked to your Yostar account and we&rsquo;ll process it within 30 days.
                    </>
                }
                control={<span className="font-mono text-[12px] text-muted-foreground">manual</span>}
            />
        </CardContent>
    </Card>
);
