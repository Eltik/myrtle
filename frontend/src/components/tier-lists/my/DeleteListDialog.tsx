import { TriangleAlertIcon } from "lucide-react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./DeleteListDialog.messages";

export interface IDeleteListTarget {
    slug: string;
    name: string;
}

interface IDeleteListDialogProps {
    target: IDeleteListTarget | null;
    onOpenChange: (open: boolean) => void;
    onConfirm: (slug: string) => void;
    isSubmitting: boolean;
    errorMessage: string | null;
}

export function DeleteListDialog({ target, onOpenChange, onConfirm, isSubmitting, errorMessage }: IDeleteListDialogProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const rt: TypedRichT<typeof messages> = useRichT("tierLists");
    const open = target !== null;

    const handleConfirm = () => {
        if (target) onConfirm(target.slug);
    };

    return (
        <AlertDialog open={open} onOpenChange={onOpenChange}>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <div className="flex items-center gap-3">
                        <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-destructive/12 text-destructive-foreground">
                            <TriangleAlertIcon className="h-4.5 w-4.5" aria-hidden="true" />
                        </span>
                        <div className="flex min-w-0 flex-col gap-1">
                            <AlertDialogTitle>{t("my.delete.title")}</AlertDialogTitle>
                            <AlertDialogDescription>{rt("my.delete.body", { name: <span className="font-medium text-foreground">{target?.name ?? t("my.delete.fallbackName")}</span> })}</AlertDialogDescription>
                        </div>
                    </div>
                </AlertDialogHeader>

                {errorMessage && (
                    <div role="alert" className="mx-6 mb-2 rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 font-sans text-destructive-foreground text-xs">
                        {errorMessage}
                    </div>
                )}

                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" disabled={isSubmitting} />}>{t("my.delete.cancel")}</AlertDialogClose>
                    <Button type="button" variant="destructive" loading={isSubmitting} onClick={handleConfirm}>
                        {t("my.delete.submit")}
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    );
}
