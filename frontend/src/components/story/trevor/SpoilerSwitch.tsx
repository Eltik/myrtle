/**
 * Spoiler mode in the chat (contract v1.1 section 8): the "Spoilers" switch
 * beside the spoiler-safe label, the label that replaces it while the mode is
 * on, and the badge on an answer asked in spoiler mode.
 *
 * Turning the switch on asks for confirmation the first time in this browser;
 * until the reader confirms, the switch stays off. Turning it off never asks.
 */
import { EyeIcon } from "lucide-react";
import type React from "react";
import { useState } from "react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Label } from "#/components/ui/label";
import { Switch } from "#/components/ui/switch";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { markSpoilersConfirmed, spoilersConfirmed } from "#/lib/trevor/spoilerMode";
import type { messages } from "./trevor.messages";

type TrevorT = TypedT<typeof messages>;

export function SpoilerSwitch({ on, onChange }: { on: boolean; onChange: (on: boolean) => void }): React.ReactElement {
    const t: TrevorT = useT("story");
    const [confirming, setConfirming] = useState(false);
    const request = (next: boolean) => {
        if (!next) onChange(false);
        else if (spoilersConfirmed()) onChange(true);
        else setConfirming(true);
    };
    const confirm = () => {
        markSpoilersConfirmed();
        onChange(true);
        setConfirming(false);
    };
    return (
        <>
            {/* The switch's name is "Spoilers"; the visible text repeats its state for sighted readers only. */}
            <Label className="ms-auto shrink-0 cursor-pointer select-none font-normal text-muted-foreground text-xs/4 max-sm:min-h-11 sm:text-xs/4" data-trevor-spoiler-switch={on ? "on" : "off"}>
                <span aria-hidden>{on ? t("trevor.spoilers.on") : t("trevor.spoilers.off")}</span>
                <Switch checked={on} onCheckedChange={request} aria-label={t("trevor.spoilers.name")} className="data-checked:bg-warning" />
            </Label>
            <AlertDialog open={confirming} onOpenChange={setConfirming}>
                <AlertDialogPopup data-trevor-spoiler-confirm>
                    <AlertDialogHeader>
                        <AlertDialogTitle>{t("trevor.spoilers.confirmTitle")}</AlertDialogTitle>
                        <AlertDialogDescription>{t("trevor.spoilers.confirmBody")}</AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                        <AlertDialogClose render={<Button type="button" variant="outline" />}>{t("trevor.spoilers.cancel")}</AlertDialogClose>
                        <Button type="button" onClick={confirm} data-trevor-spoiler-confirm-on>
                            {t("trevor.spoilers.confirm")}
                        </Button>
                    </AlertDialogFooter>
                </AlertDialogPopup>
            </AlertDialog>
        </>
    );
}

/** The label above the composer while spoiler mode is on, in place of "Spoiler-safe up to ...". */
export function SpoilersOnLabel(): React.ReactElement {
    const t: TrevorT = useT("story");
    return (
        <Badge variant="outline" className="h-auto max-w-full gap-1.5 whitespace-normal border-warning/40 bg-warning/6 py-0.5 text-start" data-trevor-spoiler="all">
            <EyeIcon aria-hidden className="size-3.5 shrink-0 text-warning" />
            {t("trevor.spoilers.label")}
        </Badge>
    );
}

/** The tag on an answer asked in spoiler mode. */
export function SpoilersBadge(): React.ReactElement {
    const t: TrevorT = useT("story");
    return (
        <span className="self-start rounded-full border border-warning/32 bg-warning/8 px-2 py-0.5 font-medium text-warning-foreground text-xs dark:bg-warning/16" data-trevor-spoilers-badge>
            {t("trevor.spoilers.badge")}
        </span>
    );
}
