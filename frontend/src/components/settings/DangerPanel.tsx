import { Trash2Icon, TriangleAlertIcon } from "lucide-react";
import { useId, useState } from "react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./DangerPanel.messages";
import { Mono } from "./SettingsShell";

interface IDangerPanelProps {
    onDelete: () => void;
    deleting: boolean;
    /** The last failed attempt, shown inside the dialog so it can be retried. */
    errorMessage: string | null;
    onDialogClose: () => void;
}

export function DangerPanel({ onDelete, deleting, errorMessage, onDialogClose }: IDangerPanelProps) {
    const t: TypedT<typeof messages> = useT("settings");
    const rt: TypedRichT<typeof messages> = useRichT("settings");
    const [open, setOpen] = useState(false);
    const [typed, setTyped] = useState("");
    const inputId = useId();

    const word = t("danger.dialog.confirmWord");
    const confirmed = typed.trim().toLowerCase() === word.trim().toLowerCase();

    const handleOpenChange = (next: boolean) => {
        // A delete in flight can't be walked away from mid-request.
        if (!next && deleting) return;
        setOpen(next);
        if (!next) {
            setTyped("");
            onDialogClose();
        }
    };

    const handleConfirm = () => {
        if (confirmed && !deleting) onDelete();
    };

    return (
        <Card className="border-[color-mix(in_srgb,var(--destructive)_32%,var(--border))]">
            <CardHeader>
                <CardTitle className="text-destructive">{t("danger.delete.title")}</CardTitle>
                <CardDescription>{t("danger.delete.desc")}</CardDescription>
            </CardHeader>
            <CardContent className="pt-0">
                <div className="mt-4 flex flex-wrap gap-2.5">
                    <Button variant="destructive-outline" className="w-full sm:w-auto" onClick={() => setOpen(true)}>
                        <Trash2Icon className="size-4" />
                        {t("danger.delete.action")}
                    </Button>
                </div>
                <p className="mt-3 font-sans text-[12.5px] text-muted-foreground leading-[1.55]">{rt("danger.delete.note", { email: <Mono>{t("danger.delete.noteEmail")}</Mono> })}</p>
            </CardContent>

            <AlertDialog open={open} onOpenChange={handleOpenChange}>
                {/* Scrolls inside itself: on a phone held sideways (375 px tall) the header, the typed confirmation, an error and the footer outgrow the popup's max-height. */}
                <AlertDialogPopup className="overflow-y-auto">
                    <AlertDialogHeader>
                        <div className="flex items-center gap-3">
                            <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-destructive/12 text-destructive-foreground">
                                <TriangleAlertIcon className="h-4.5 w-4.5" aria-hidden="true" />
                            </span>
                            <AlertDialogTitle>{t("danger.dialog.title")}</AlertDialogTitle>
                        </div>
                        <AlertDialogDescription>{t("danger.dialog.body")}</AlertDialogDescription>
                        <p className="m-0 font-sans text-muted-foreground text-sm">{t("danger.dialog.kept")}</p>
                    </AlertDialogHeader>

                    <form
                        className="flex flex-col gap-2 px-6 pb-4"
                        onSubmit={(e) => {
                            e.preventDefault();
                            handleConfirm();
                        }}
                    >
                        <Label htmlFor={inputId}>{rt("danger.dialog.confirmLabel", { word: <Mono>{word}</Mono> })}</Label>
                        <Input id={inputId} value={typed} onChange={(e) => setTyped(e.target.value)} autoComplete="off" autoCapitalize="off" spellCheck={false} disabled={deleting} />
                    </form>

                    {errorMessage && (
                        <div role="alert" className="mx-6 mb-2 rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 font-sans text-destructive-foreground text-xs">
                            {errorMessage}
                        </div>
                    )}

                    <AlertDialogFooter>
                        <AlertDialogClose render={<Button type="button" variant="outline" disabled={deleting} />}>{t("danger.dialog.cancel")}</AlertDialogClose>
                        <Button type="button" variant="destructive" disabled={!confirmed} loading={deleting} onClick={handleConfirm}>
                            {t("danger.dialog.submit")}
                        </Button>
                    </AlertDialogFooter>
                </AlertDialogPopup>
            </AlertDialog>
        </Card>
    );
}
