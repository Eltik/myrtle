import { TriangleAlertIcon } from "lucide-react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";

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
                            <AlertDialogTitle>Delete tier list?</AlertDialogTitle>
                            <AlertDialogDescription>
                                <span className="font-medium text-foreground">{target?.name ?? "This list"}</span> and all of its tiers, placements, and stats will be permanently removed. Anyone with the link will see a 404 page. This cannot be undone.
                            </AlertDialogDescription>
                        </div>
                    </div>
                </AlertDialogHeader>

                {errorMessage && (
                    <div role="alert" className="mx-6 mb-2 rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 font-sans text-xs text-destructive-foreground">
                        {errorMessage}
                    </div>
                )}

                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" disabled={isSubmitting} />}>Cancel</AlertDialogClose>
                    <Button type="button" variant="destructive" loading={isSubmitting} onClick={handleConfirm}>
                        Delete list
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    );
}
