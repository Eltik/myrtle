import { TriangleAlertIcon } from "lucide-react";
import { useState } from "react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";

export interface IDeletePlansTarget {
    ids: string[];
    names: string[];
}

interface IDeletePlansDialogProps {
    target: IDeletePlansTarget | null;
    onOpenChange: (open: boolean) => void;
    onConfirm: (ids: string[]) => void;
    isSubmitting: boolean;
    errorMessage: string | null;
}

function formatNames(names: string[]): string {
    if (names.length <= 3) return names.join(", ");
    return `${names.slice(0, 3).join(", ")} and ${names.length - 3} more`;
}

export function DeletePlansDialog({ target, onOpenChange, onConfirm, isSubmitting, errorMessage }: IDeletePlansDialogProps) {
    const open = target !== null;

    // The target goes null the moment the dialog starts closing, but the close
    // animation keeps it visible for a few frames - keep rendering the last
    // real target so the copy doesn't collapse to "Delete 0 plans" mid-fade.
    const [lastTarget, setLastTarget] = useState<IDeletePlansTarget | null>(null);
    if (target !== null && target !== lastTarget) setLastTarget(target);
    const shown = target ?? lastTarget;
    const count = shown?.ids.length ?? 0;

    const handleConfirm = () => {
        if (target) onConfirm(target.ids);
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
                            <AlertDialogTitle>{count === 1 ? "Delete plan?" : `Delete ${count} plans?`}</AlertDialogTitle>
                            <AlertDialogDescription>
                                <span className="font-medium text-foreground">{shown ? formatNames(shown.names) : "This plan"}</span> and {count === 1 ? "its" : "their"} promotion, level, skill, and module goals will be permanently removed. This cannot be undone.
                            </AlertDialogDescription>
                        </div>
                    </div>
                </AlertDialogHeader>

                {errorMessage && (
                    <div role="alert" className="mx-6 mb-2 rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 font-sans text-destructive-foreground text-xs">
                        {errorMessage}
                    </div>
                )}

                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" disabled={isSubmitting} />}>Cancel</AlertDialogClose>
                    <Button type="button" variant="destructive" loading={isSubmitting} onClick={handleConfirm}>
                        {count === 1 ? "Delete plan" : `Delete ${count} plans`}
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    );
}
