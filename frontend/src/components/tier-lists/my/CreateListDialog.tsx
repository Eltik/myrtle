import { useEffect, useId, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Field, FieldDescription, FieldLabel } from "#/components/ui/field";
import { Input } from "#/components/ui/input";
import { Textarea } from "#/components/ui/textarea";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { LIST_DESCRIPTION_MAX as DESC_MAX, LIST_NAME_MAX as NAME_MAX } from "../shared";
import type { messages } from "./CreateListDialog.messages";

interface ICreateListDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSubmit: (input: { name: string; description: string }) => void;
    isSubmitting: boolean;
    errorMessage: string | null;
}

export function CreateListDialog({ open, onOpenChange, onSubmit, isSubmitting, errorMessage }: ICreateListDialogProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
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
                        <DialogTitle>{t("my.create.title")}</DialogTitle>
                        <DialogDescription>{t("my.create.description")}</DialogDescription>
                    </DialogHeader>

                    <div className="flex flex-col gap-5 px-6 pb-2">
                        <Field>
                            <FieldLabel htmlFor={nameId}>
                                {t("my.create.name")}
                                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                    {name.length} / {NAME_MAX}
                                </span>
                            </FieldLabel>
                            <Input id={nameId} value={name} onChange={(e) => setName((e.target as HTMLInputElement).value.slice(0, NAME_MAX))} placeholder={t("my.create.namePlaceholder")} autoFocus required aria-invalid={trimmedName.length === 0 && name.length > 0 ? true : undefined} />
                            <FieldDescription>{t("my.create.nameHint")}</FieldDescription>
                        </Field>

                        <Field>
                            <FieldLabel htmlFor={descId}>
                                {t("my.create.descriptionLabel")}
                                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                    {description.length} / {DESC_MAX}
                                </span>
                            </FieldLabel>
                            <Textarea id={descId} value={description} onChange={(e) => setDescription(e.target.value.slice(0, DESC_MAX))} placeholder={t("my.create.descriptionPlaceholder")} rows={3} />
                            <FieldDescription>{t("my.create.descriptionHint")}</FieldDescription>
                        </Field>

                        {errorMessage && (
                            <div role="alert" className="rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 font-sans text-destructive-foreground text-xs">
                                {errorMessage}
                            </div>
                        )}
                    </div>

                    <DialogFooter>
                        <DialogClose render={<Button type="button" variant="outline" />}>{t("my.create.cancel")}</DialogClose>
                        <Button type="submit" disabled={!canSubmit} loading={isSubmitting}>
                            {t("my.create.submit")}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogPopup>
        </Dialog>
    );
}
