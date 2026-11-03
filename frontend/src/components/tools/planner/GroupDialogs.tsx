import { TriangleAlertIcon } from "lucide-react";
import * as React from "react";
import { AlertDialog, AlertDialogClose, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Field, FieldLabel } from "#/components/ui/field";
import { Input } from "#/components/ui/input";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./GroupDialogs.messages";

// In-app rename and delete for a plan group. Not `window.prompt` /
// `window.confirm`: a native dialog blocks the page and hangs browser automation.

type GroupDialogT = TypedT<typeof messages>;

/** Keeps the last non-null group name while the dialog plays its close animation. */
function useLastName(name: string | null): string {
    const [last, setLast] = React.useState<string>(name ?? "");
    if (name !== null && name !== last) setLast(name);
    return name ?? last;
}

function DialogError({ message }: { message: string | null }) {
    if (!message) return null;
    return (
        <div role="alert" className="mx-6 mb-2 rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 font-sans text-destructive-foreground text-xs">
            {message}
        </div>
    );
}

interface IRenameGroupDialogProps {
    /** The group being renamed; null closes the dialog. */
    name: string | null;
    onClose: () => void;
    /** Saves the new name; a rejection keeps the dialog open with an error. */
    onRename: (oldName: string, newName: string) => Promise<void>;
}

export function RenameGroupDialog({ name, onClose, onRename }: IRenameGroupDialogProps): React.ReactElement {
    const t: GroupDialogT = useT("tools");
    const shown = useLastName(name);
    const [value, setValue] = React.useState<string>(shown);
    const [pending, setPending] = React.useState<boolean>(false);
    const [error, setError] = React.useState<string | null>(null);

    const [openedFor, setOpenedFor] = React.useState<string | null>(null);
    if (name !== openedFor) {
        setOpenedFor(name);
        if (name !== null) {
            setValue(name);
            setError(null);
        }
    }

    const trimmed = value.trim();
    const canSave = trimmed.length > 0 && trimmed !== shown;

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (name === null) return;
        if (!canSave) {
            if (trimmed === name) onClose();
            return;
        }
        setPending(true);
        setError(null);
        try {
            await onRename(name, trimmed);
            onClose();
        } catch (err) {
            console.error(err);
            setError(t("planner.groupDialog.rename.failed"));
        } finally {
            setPending(false);
        }
    };

    return (
        <Dialog open={name !== null} onOpenChange={(next) => !next && !pending && onClose()}>
            <DialogPopup className="max-w-sm">
                <form onSubmit={handleSubmit}>
                    <DialogHeader>
                        <DialogTitle>{t("planner.groupDialog.rename.title")}</DialogTitle>
                    </DialogHeader>
                    <DialogPanel scrollFade={false}>
                        <Field>
                            <FieldLabel>{t("planner.groupDialog.rename.label")}</FieldLabel>
                            <Input value={value} onChange={(e) => setValue(e.target.value)} autoFocus disabled={pending} />
                        </Field>
                    </DialogPanel>
                    <DialogError message={error} />
                    <DialogFooter>
                        <DialogClose render={<Button type="button" variant="outline" disabled={pending} />}>{t("planner.groupDialog.cancel")}</DialogClose>
                        <Button type="submit" loading={pending} disabled={!canSave}>
                            {t("planner.groupDialog.rename.save")}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogPopup>
        </Dialog>
    );
}

interface IDeleteGroupDialogProps {
    /** The group being deleted; null closes the dialog. */
    name: string | null;
    onClose: () => void;
    /** Deletes the group; a rejection keeps the dialog open with an error. */
    onDelete: (name: string) => Promise<void>;
}

export function DeleteGroupDialog({ name, onClose, onDelete }: IDeleteGroupDialogProps): React.ReactElement {
    const t: GroupDialogT = useT("tools");
    const shown = useLastName(name);
    const [pending, setPending] = React.useState<boolean>(false);
    const [error, setError] = React.useState<string | null>(null);

    const [openedFor, setOpenedFor] = React.useState<string | null>(null);
    if (name !== openedFor) {
        setOpenedFor(name);
        if (name !== null) setError(null);
    }

    const handleConfirm = async () => {
        if (name === null) return;
        setPending(true);
        setError(null);
        try {
            await onDelete(name);
            onClose();
        } catch (err) {
            console.error(err);
            setError(t("planner.groupDialog.delete.failed"));
        } finally {
            setPending(false);
        }
    };

    return (
        <AlertDialog open={name !== null} onOpenChange={(next) => !next && !pending && onClose()}>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <div className="flex items-center gap-3">
                        <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-destructive/12 text-destructive-foreground">
                            <TriangleAlertIcon className="h-4.5 w-4.5" aria-hidden="true" />
                        </span>
                        <AlertDialogTitle className="min-w-0">{t("planner.groupDialog.delete.title", { name: shown })}</AlertDialogTitle>
                    </div>
                </AlertDialogHeader>
                <DialogError message={error} />
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" disabled={pending} />}>{t("planner.groupDialog.cancel")}</AlertDialogClose>
                    <Button type="button" variant="destructive" loading={pending} onClick={handleConfirm}>
                        {t("planner.groupDialog.delete.confirm")}
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    );
}
