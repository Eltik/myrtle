import { EraserIcon, Trash2Icon } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Field, FieldDescription, FieldLabel } from "#/components/ui/field";
import { Input } from "#/components/ui/input";
import { MarkdownEditor } from "#/components/ui/markdown-editor";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { TIER_DESCRIPTION_MAX as DESC_MAX, isHexColor, TIER_NAME_MAX as NAME_MAX } from "../shared";
import { ColorPicker } from "./ColorPicker";
import type { IEditTier } from "./state";
import type { messages } from "./TierSettingsDialog.messages";

interface ITierSettingsDialogProps {
    tier: IEditTier | null;
    canDelete: boolean;
    onClose: () => void;
    onSave: (next: { name: string; color: string; description: string }) => void;
    onDelete: () => void;
    onClear: () => void;
}

export function TierSettingsDialog({ tier, canDelete, onClose, onSave, onDelete, onClear }: ITierSettingsDialogProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const nameId = useId();
    const descId = useId();
    const nameInputRef = useRef<HTMLInputElement>(null);
    const [name, setName] = useState("");
    const [color, setColor] = useState("#dc4d56");
    const [description, setDescription] = useState("");
    const [confirmingDelete, setConfirmingDelete] = useState(false);
    const [confirmingClear, setConfirmingClear] = useState(false);

    useEffect(() => {
        if (tier) {
            setName(tier.name);
            setColor(tier.color);
            setDescription(tier.description);
            setConfirmingDelete(false);
            setConfirmingClear(false);

            const frame = requestAnimationFrame(() => {
                nameInputRef.current?.focus();
                nameInputRef.current?.select();
            });
            return () => cancelAnimationFrame(frame);
        }
    }, [tier]);

    const operatorCount = tier?.operatorIds.length ?? 0;

    const trimmedName = name.trim();
    const validColor = isHexColor(color);
    const canSave = trimmedName.length > 0 && trimmedName.length <= NAME_MAX && validColor;

    const submit = () => {
        if (!canSave) return;
        onSave({ name: trimmedName, color: color.toLowerCase(), description: description.trim() });
    };

    const handleSave = (e: React.FormEvent) => {
        e.preventDefault();
        submit();
    };

    // Enter in the label input already submits (implicit submission); this makes
    // Ctrl/Cmd+Enter save from the description textarea too, where a bare Enter
    // has to stay a newline.
    const handleKeyDown = (e: React.KeyboardEvent<HTMLFormElement>) => {
        if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
            e.preventDefault();
            submit();
        }
    };

    return (
        <Dialog open={tier !== null} onOpenChange={(o) => !o && onClose()}>
            <DialogPopup className="sm:max-w-md">
                <form onSubmit={handleSave} onKeyDown={handleKeyDown} className="flex min-h-0 flex-col">
                    <DialogHeader>
                        <DialogTitle>{t("edit.tierSettings.title")}</DialogTitle>
                        <DialogDescription>{t("edit.tierSettings.description")}</DialogDescription>
                    </DialogHeader>

                    <DialogPanel className="flex flex-col gap-5">
                        <Field>
                            <FieldLabel htmlFor={nameId}>
                                {t("edit.tierSettings.label")}
                                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                    {name.length} / {NAME_MAX}
                                </span>
                            </FieldLabel>
                            <Input ref={nameInputRef} id={nameId} value={name} onChange={(e) => setName((e.target as HTMLInputElement).value.slice(0, NAME_MAX))} placeholder={t("edit.tierSettings.labelPlaceholder")} autoFocus required aria-invalid={trimmedName.length === 0 && name.length > 0 ? true : undefined} />
                            <FieldDescription>{t("edit.tierSettings.labelHint", { max: NAME_MAX })}</FieldDescription>
                        </Field>

                        <Field>
                            <FieldLabel>{t("edit.tierSettings.color")}</FieldLabel>
                            <ColorPicker value={color} onChange={setColor} />
                        </Field>

                        <Field>
                            <FieldLabel htmlFor={descId}>
                                {t("edit.tierSettings.descriptionLabel")}
                                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                    {description.length} / {DESC_MAX}
                                </span>
                            </FieldLabel>
                            <MarkdownEditor id={descId} value={description} onChange={setDescription} placeholder={t("edit.tierSettings.descriptionPlaceholder")} rows={3} maxLength={DESC_MAX} showHint={false} textareaClassName="min-h-20 sm:min-h-24" />
                            <FieldDescription>{t("edit.tierSettings.descriptionHint")}</FieldDescription>
                        </Field>

                        {operatorCount > 0 && (
                            <div className="-mx-6 -mb-6 flex flex-wrap items-center justify-between gap-2 border-border border-t px-6 py-3">
                                <p className="m-0 font-sans text-[12.5px] text-muted-foreground">
                                    <span className="font-medium text-foreground tabular-nums">{operatorCount}</span> {t("edit.tierSettings.operatorCount", { count: operatorCount })}
                                </p>
                                {confirmingClear ? (
                                    <div className="flex items-center gap-2">
                                        <span className="font-sans text-[12.5px] text-muted-foreground">{t("edit.tierSettings.clearConfirmPrompt")}</span>
                                        <Button type="button" size="sm" variant="destructive" onClick={onClear}>
                                            {t("edit.tierSettings.confirm")}
                                        </Button>
                                        <Button type="button" size="sm" variant="ghost" onClick={() => setConfirmingClear(false)}>
                                            {t("edit.tierSettings.cancel")}
                                        </Button>
                                    </div>
                                ) : (
                                    <Button type="button" size="sm" variant="destructive-outline" onClick={() => setConfirmingClear(true)}>
                                        <EraserIcon />
                                        {t("edit.tierSettings.clear")}
                                    </Button>
                                )}
                            </div>
                        )}
                    </DialogPanel>

                    <DialogFooter className="justify-between sm:justify-between">
                        {canDelete ? (
                            confirmingDelete ? (
                                <div className="flex items-center gap-2">
                                    <span className="font-sans text-[12.5px] text-muted-foreground">{t("edit.tierSettings.deleteConfirmPrompt")}</span>
                                    <Button type="button" size="sm" variant="destructive" onClick={onDelete}>
                                        {t("edit.tierSettings.confirm")}
                                    </Button>
                                    <Button type="button" size="sm" variant="ghost" onClick={() => setConfirmingDelete(false)}>
                                        {t("edit.tierSettings.cancel")}
                                    </Button>
                                </div>
                            ) : (
                                <Button type="button" variant="destructive-outline" size="sm" onClick={() => setConfirmingDelete(true)}>
                                    <Trash2Icon />
                                    {t("edit.tierSettings.delete")}
                                </Button>
                            )
                        ) : (
                            <span className="font-sans text-[11.5px] text-muted-foreground italic">{t("edit.tierSettings.lastTier")}</span>
                        )}

                        <div className="flex items-center gap-2">
                            <DialogClose render={<Button type="button" variant="outline" />}>{t("edit.tierSettings.cancel")}</DialogClose>
                            <Button type="submit" disabled={!canSave}>
                                {t("edit.tierSettings.save")}
                            </Button>
                        </div>
                    </DialogFooter>
                </form>
            </DialogPopup>
        </Dialog>
    );
}
