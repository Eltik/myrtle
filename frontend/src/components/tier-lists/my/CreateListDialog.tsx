import { useEffect, useId, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Field, FieldDescription, FieldLabel } from "#/components/ui/field";
import { Input } from "#/components/ui/input";
import { Textarea } from "#/components/ui/textarea";
import { LIST_DESCRIPTION_MAX as DESC_MAX, LIST_NAME_MAX as NAME_MAX } from "../shared";

interface ICreateListDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSubmit: (input: { name: string; description: string }) => void;
    isSubmitting: boolean;
    errorMessage: string | null;
}

export function CreateListDialog({ open, onOpenChange, onSubmit, isSubmitting, errorMessage }: ICreateListDialogProps) {
    const [name, setName] = useState("");
    const [description, setDescription] = useState("");
    const nameId = useId();
    const descId = useId();

    useEffect(() => {
        if (open) {
            setName("");
            setDescription("");
        }
    }, [open]);

    const trimmedName = name.trim();
    const canSubmit = trimmedName.length > 0 && trimmedName.length <= NAME_MAX && !isSubmitting;

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (!canSubmit) return;
        onSubmit({ name: trimmedName, description: description.trim() });
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup>
                <form onSubmit={handleSubmit}>
                    <DialogHeader>
                        <DialogTitle>New tier list</DialogTitle>
                        <DialogDescription>Give your list a name and a short description. You can change these later, and your edits will be visible to everyone who has the share link.</DialogDescription>
                    </DialogHeader>

                    <div className="flex flex-col gap-5 px-6 pb-2">
                        <Field>
                            <FieldLabel htmlFor={nameId}>
                                Name
                                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                    {name.length} / {NAME_MAX}
                                </span>
                            </FieldLabel>
                            <Input id={nameId} value={name} onChange={(e) => setName((e.target as HTMLInputElement).value.slice(0, NAME_MAX))} placeholder="e.g. Endgame DPS rankings" autoFocus required aria-invalid={trimmedName.length === 0 && name.length > 0 ? true : undefined} />
                            <FieldDescription>Shown on browse cards and on the public detail page.</FieldDescription>
                        </Field>

                        <Field>
                            <FieldLabel htmlFor={descId}>
                                Description
                                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                    {description.length} / {DESC_MAX}
                                </span>
                            </FieldLabel>
                            <Textarea id={descId} value={description} onChange={(e) => setDescription(e.target.value.slice(0, DESC_MAX))} placeholder="What's this list about? Who is it for?" rows={3} />
                            <FieldDescription>Optional. A sentence or two helps readers know what to expect.</FieldDescription>
                        </Field>

                        {errorMessage && (
                            <div role="alert" className="rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 font-sans text-destructive-foreground text-xs">
                                {errorMessage}
                            </div>
                        )}
                    </div>

                    <DialogFooter>
                        <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                        <Button type="submit" disabled={!canSubmit} loading={isSubmitting}>
                            Create list
                        </Button>
                    </DialogFooter>
                </form>
            </DialogPopup>
        </Dialog>
    );
}
