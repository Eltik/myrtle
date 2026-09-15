import { MailIcon } from "lucide-react";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./DangerPanel.messages";
import { Mono } from "./SettingsShell";

export function DangerPanel() {
    const t: TypedT<typeof messages> = useT("settings");
    const rt: TypedRichT<typeof messages> = useRichT("settings");

    return (
        <Card className="border-[color-mix(in_srgb,var(--destructive)_32%,var(--border))]">
            <CardHeader>
                <CardTitle className="text-destructive">{t("danger.delete.title")}</CardTitle>
                <CardDescription>{t("danger.delete.desc")}</CardDescription>
            </CardHeader>
            <CardContent className="pt-0">
                <div className="mt-4 flex flex-wrap gap-2.5">
                    {/* biome-ignore lint/a11y/useAnchorContent: anchor children are slotted in by Button via render prop */}
                    <Button variant="destructive-outline" className="w-full sm:w-auto" render={<a href="mailto:privacy@myrtle.moe?subject=Account%20deletion%20request" />}>
                        <MailIcon className="size-4" />
                        {t("danger.delete.action")}
                    </Button>
                </div>
                <p className="mt-3 font-sans text-[12.5px] text-muted-foreground leading-[1.55]">{rt("danger.delete.note", { email: <Mono>{t("danger.delete.noteEmail")}</Mono> })}</p>
            </CardContent>
        </Card>
    );
}
