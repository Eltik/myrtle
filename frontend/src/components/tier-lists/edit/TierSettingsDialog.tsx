import { Trash2Icon } from "lucide-react";
import { useEffect, useId, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Field, FieldDescription, FieldLabel } from "#/components/ui/field";
import { Input } from "#/components/ui/input";
import { Textarea } from "#/components/ui/textarea";
import { ColorPicker } from "./ColorPicker";
import type { IEditTier } from "./state";

const NAME_MAX = 24;
const DESC_MAX = 200;
const HEX_RE = /^#[0-9a-fA-F]{6}$/;

interface ITierSettingsDialogProps {
    tier: IEditTier | null;
    canDelete: boolean;
    onClose: () => void;
    onSave: (next: { name: string; color: string; description: string }) => void;
    onDelete: () => void;
}

export function TierSettingsDialog({ tier, canDelete, onClose, onSave, onDelete }: ITierSettingsDialogProps) {
    const nameId = useId();
    const descId = useId();
    const [name, setName] = useState("");
    const [color, setColor] = useState("#dc4d56");
    const [description, setDescription] = useState("");
    const [confirmingDelete, setConfirmingDelete] = useState(false);

    useEffect(() => {
        if (tier) {
            setName(tier.name);
            setColor(tier.color);
            setDescription(tier.description);
            setConfirmingDelete(false);
        }
    }, [tier]);

    const trimmedName = name.trim();
    const validColor = HEX_RE.test(color);
    const canSave = trimmedName.length > 0 && trimmedName.length <= NAME_MAX && validColor;

    const handleSave = (e: React.FormEvent) => {
        e.preventDefault();
        if (!canSave) return;
        onSave({ name: trimmedName, color: color.toLowerCase(), description: description.trim() });
    };

    return (
        <Dialog open={tier !== null} onOpenChange={(o) => !o && onClose()}>
            <DialogPopup className="sm:max-w-md">
                <form onSubmit={handleSave}>
                    <DialogHeader>
                        <DialogTitle>Tier settings</DialogTitle>
                        <DialogDescription>Customize the label, color, and description for this tier. Changes take effect when you save the list.</DialogDescription>
                    </DialogHeader>

                    <div className="flex flex-col gap-5 px-6 pb-2">
                        <Field>
                            <FieldLabel htmlFor={nameId}>
                                Label
                                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                    {name.length} / {NAME_MAX}
                                </span>
                            </FieldLabel>
                            <Input id={nameId} value={name} onChange={(e) => setName((e.target as HTMLInputElement).value.slice(0, NAME_MAX))} placeholder="S, A, Pick-One, etc." autoFocus required aria-invalid={trimmedName.length === 0 && name.length > 0 ? true : undefined} />
                            <FieldDescription>Short labels like "S" or "A" read clearest; up to {NAME_MAX} characters.</FieldDescription>
                        </Field>

                        <Field>
                            <FieldLabel>Color</FieldLabel>
                            <ColorPicker value={color} onChange={setColor} />
                        </Field>

                        <Field>
                            <FieldLabel htmlFor={descId}>
                                Description
                                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                    {description.length} / {DESC_MAX}
                                </span>
                            </FieldLabel>
                            <Textarea id={descId} value={description} onChange={(e) => setDescription(e.target.value.slice(0, DESC_MAX))} placeholder="When should an operator land here?" rows={3} />
                            <FieldDescription>Optional. Shown to viewers in the tier hover/detail.</FieldDescription>
                        </Field>
                    </div>

                    <DialogFooter className="justify-between sm:justify-between">
                        {canDelete ? (
                            confirmingDelete ? (
                                <div className="flex items-center gap-2">
                                    <span className="font-sans text-[12.5px] text-muted-foreground">Delete this tier?</span>
                                    <Button type="button" size="sm" variant="destructive" onClick={onDelete}>
                                        Confirm
                                    </Button>
                                    <Button type="button" size="sm" variant="ghost" onClick={() => setConfirmingDelete(false)}>
                                        Cancel
                                    </Button>
                                </div>
                            ) : (
                                <Button type="button" variant="destructive-outline" size="sm" onClick={() => setConfirmingDelete(true)}>
                                    <Trash2Icon />
                                    Delete tier
                                </Button>
                            )
                        ) : (
                            <span className="font-sans text-[11.5px] text-muted-foreground italic">Lists must have at least one tier.</span>
                        )}

                        <div className="flex items-center gap-2">
                            <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                            <Button type="submit" disabled={!canSave}>
                                Save tier
                            </Button>
                        </div>
                    </DialogFooter>
                </form>
            </DialogPopup>
        </Dialog>
    );
}
