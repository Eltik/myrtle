import { TriangleAlertIcon } from "lucide-react";
import type { ReactNode } from "react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";

// The in-app confirmation every grid screen asks through. Never
// `window.confirm`: a native dialog blocks the page and hangs browser
// automation. The caller supplies the words, already translated.

interface IConfirmDialogProps {
    open: boolean;
    title: string;
    body: ReactNode;
    confirmLabel: string;
    cancelLabel: string;
    destructive?: boolean;
    pending?: boolean;
    errorMessage?: string | null;
    onConfirm: () => void;
    onCancel: () => void;
}

export function ConfirmDialog({ open, title, body, confirmLabel, cancelLabel, destructive = false, pending = false, errorMessage, onConfirm, onCancel }: IConfirmDialogProps) {
    return (
        <AlertDialog open={open} onOpenChange={(next) => !next && !pending && onCancel()}>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <div className="flex items-center gap-3">
                        {destructive && (
                            <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-destructive/12 text-destructive-foreground">
                                <TriangleAlertIcon className="h-4.5 w-4.5" aria-hidden="true" />
                            </span>
                        )}
                        <div className="flex min-w-0 flex-col gap-1">
                            <AlertDialogTitle>{title}</AlertDialogTitle>
                            <AlertDialogDescription>{body}</AlertDialogDescription>
                        </div>
                    </div>
                </AlertDialogHeader>

                {errorMessage && (
                    <div role="alert" className="mx-6 mb-2 rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 font-sans text-destructive-foreground text-xs">
                        {errorMessage}
                    </div>
                )}

                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" disabled={pending} />}>{cancelLabel}</AlertDialogClose>
                    <Button type="button" variant={destructive ? "destructive" : "default"} loading={pending} onClick={onConfirm}>
                        {confirmLabel}
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    );
}
