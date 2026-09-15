import { useEffect, useId, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Field, FieldDescription, FieldLabel } from "#/components/ui/field";
import { Input } from "#/components/ui/input";
import { Textarea } from "#/components/ui/textarea";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { LIST_DESCRIPTION_MAX as DESC_MAX, LIST_NAME_MAX as NAME_MAX } from "../shared";
import type { messages } from "./EditListDialog.messages";

export interface IEditListInitial {
    slug: string;
    name: string;
    description: string;
}

interface IEditListDialogProps {
    initial: IEditListInitial | null;
    onOpenChange: (open: boolean) => void;
    onSubmit: (input: { slug: string; name: string; description: string }) => void;
    isSubmitting: boolean;
    errorMessage: string | null;
}

export function EditListDialog({ initial, onOpenChange, onSubmit, isSubmitting, errorMessage }: IEditListDialogProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const open = initial !== null;
    const [name, setName] = useState("");
    const [description, setDescription] = useState("");
    const nameId = useId();
    const descId = useId();

    useEffect(() => {
        if (initial) {
            setName(initial.name);
            setDescription(initial.description);
        }
    }, [initial]);

    const trimmedName = name.trim();
    const changed = initial !== null && (trimmedName !== initial.name.trim() || description.trim() !== initial.description.trim());
    const canSubmit = initial !== null && trimmedName.length > 0 && trimmedName.length <= NAME_MAX && changed && !isSubmitting;

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (!canSubmit || !initial) return;
        onSubmit({ slug: initial.slug, name: trimmedName, description: description.trim() });
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup>
                <form onSubmit={handleSubmit}>
                    <DialogHeader>
                        <DialogTitle>{t("my.edit.title")}</DialogTitle>
                        <DialogDescription>{t("my.edit.description")}</DialogDescription>
                    </DialogHeader>

                    <div className="flex flex-col gap-5 px-6 pb-2">
                        <Field>
                            <FieldLabel htmlFor={nameId}>
                                {t("my.edit.name")}
                                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                    {name.length} / {NAME_MAX}
                                </span>
                            </FieldLabel>
                            <Input id={nameId} value={name} onChange={(e) => setName((e.target as HTMLInputElement).value.slice(0, NAME_MAX))} required autoFocus />
                            <FieldDescription>{t("my.edit.nameHint")}</FieldDescription>
                        </Field>

                        <Field>
                            <FieldLabel htmlFor={descId}>
                                {t("my.edit.descriptionLabel")}
                                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                    {description.length} / {DESC_MAX}
                                </span>
                            </FieldLabel>
                            <Textarea id={descId} value={description} onChange={(e) => setDescription(e.target.value.slice(0, DESC_MAX))} rows={3} placeholder={t("my.edit.descriptionPlaceholder")} />
                        </Field>

                        {errorMessage && (
                            <div role="alert" className="rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 font-sans text-destructive-foreground text-xs">
                                {errorMessage}
                            </div>
                        )}
                    </div>

                    <DialogFooter>
                        <DialogClose render={<Button type="button" variant="outline" />}>{t("my.edit.cancel")}</DialogClose>
                        <Button type="submit" disabled={!canSubmit} loading={isSubmitting}>
                            {t("my.edit.submit")}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogPopup>
        </Dialog>
    );
}
