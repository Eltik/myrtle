import { MailIcon } from "lucide-react";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Mono } from "./SettingsShell";

export function DangerPanel() {
    return (
        <Card className="border-[color-mix(in_srgb,var(--destructive)_32%,var(--border))]">
            <CardHeader>
                <CardTitle className="text-destructive">Delete account</CardTitle>
                <CardDescription>Permanently remove your profile, synced game data, scores, and any saved configurations from our servers. This cannot be undone.</CardDescription>
            </CardHeader>
            <CardContent className="pt-0">
                <div className="mt-4 flex flex-wrap gap-2.5">
                    {/* biome-ignore lint/a11y/useAnchorContent: anchor children are slotted in by Button via render prop */}
                    <Button variant="destructive-outline" className="w-full sm:w-auto" render={<a href="mailto:privacy@myrtle.moe?subject=Account%20deletion%20request" />}>
                        <MailIcon className="size-4" />
                        Email privacy@myrtle.moe to delete
                    </Button>
                </div>
                <p className="mt-3 font-sans text-[12.5px] text-muted-foreground leading-[1.55]">
                    Self-serve deletion isn't exposed via the API yet. Email <Mono>privacy@myrtle.moe</Mono> from the address linked to your Yostar account and we'll process it within 30 days, per the privacy policy.
                </p>
            </CardContent>
        </Card>
    );
}
