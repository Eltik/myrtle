import { useId } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Field, FieldDescription, FieldLabel } from "#/components/ui/field";
import { MarkdownEditor } from "#/components/ui/markdown-editor";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ITierOperator } from "#/lib/api/tier-lists";
import { RARITY_HEX_MUTED } from "#/lib/utils";
import { readableTextColor } from "../detail/contrast";
import { PLACEMENT_DESCRIPTION_MAX } from "../shared";
import type { IEditTier } from "./state";

interface IPickTierDialogProps {
    operator: ITierOperator | null;
    currentTierId: string | null;
    description: string;
    tiers: IEditTier[];
    onClose: () => void;
    onPick: (tierId: string | null) => void;
    onDescriptionChange: (description: string) => void;
}

export function PickTierDialog({ operator, currentTierId, description, tiers, onClose, onPick, onDescriptionChange }: IPickTierDialogProps) {
    const accent = operator ? (RARITY_HEX_MUTED[operator.rarity] ?? RARITY_HEX_MUTED[1]) : null;
    const descId = useId();
    const isPlaced = currentTierId !== null;

    return (
        <Dialog open={operator !== null} onOpenChange={(o) => !o && onClose()}>
            <DialogPopup className="sm:max-w-sm">
                <DialogHeader>
                    <DialogTitle>
                        <span className="inline-flex items-center gap-2.5">
                            {operator && (
                                <span className="inline-flex h-8 w-8 shrink-0 items-center justify-center overflow-hidden rounded-md border-2" style={{ borderColor: accent ?? undefined, background: "var(--muted)" }}>
                                    <OperatorAvatar charId={operator.id} name={operator.name} />
                                </span>
                            )}
                            <span className="font-sans">{operator?.name ?? ""}</span>
                        </span>
                    </DialogTitle>
                    <DialogDescription>Choose a tier, then add an optional note. You can also drag operators directly onto a tier.</DialogDescription>
                </DialogHeader>

                <DialogPanel className="flex flex-col gap-4">
                    <div className="flex flex-col gap-1.5">
                        <p className="m-0 font-bold font-mono text-[10.5px] text-muted-foreground/80 uppercase leading-none tracking-[0.16em]">Placement</p>
                        {tiers.map((t) => {
                            const isCurrent = t.id === currentTierId;
                            const fg = readableTextColor(t.color);
                            return (
                                <button
                                    key={t.id}
                                    type="button"
                                    className="flex items-center gap-3 rounded-lg border border-border px-2.5 py-2 text-left font-sans text-sm transition-colors hover:bg-accent/50 data-[selected=true]:border-ring data-[selected=true]:bg-accent/30"
                                    data-selected={isCurrent || undefined}
                                    aria-pressed={isCurrent}
                                    onClick={() => onPick(t.id)}
                                >
                                    <span
                                        className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md font-extrabold font-sans text-[15px] leading-none tracking-tight"
                                        style={{
                                            background: t.color,
                                            color: fg,
                                            textShadow: fg === "white" ? "0 1px 0 oklch(0 0 0 / 0.25)" : "0 1px 0 oklch(1 0 0 / 0.5)",
                                        }}
                                        aria-hidden="true"
                                    >
                                        {t.name.length <= 2 ? t.name : t.name.charAt(0)}
                                    </span>
                                    <span className="min-w-0 flex-1 truncate">{t.name}</span>
                                    <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{t.operatorIds.length}</span>
                                    {isCurrent && <span className="font-bold font-mono text-[10px] text-primary uppercase tracking-wider">Current</span>}
                                </button>
                            );
                        })}
                    </div>

                    <Field>
                        <FieldLabel htmlFor={descId}>
                            Description
                            <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                {description.length} / {PLACEMENT_DESCRIPTION_MAX}
                            </span>
                        </FieldLabel>
                        <MarkdownEditor id={descId} value={description} onChange={onDescriptionChange} placeholder="Why does this operator land here?" rows={4} maxLength={PLACEMENT_DESCRIPTION_MAX} showHint={false} />
                        <FieldDescription>{isPlaced ? "Optional. Shown to viewers on this operator's tile." : "Pick a tier above to save this note with the placement."}</FieldDescription>
                    </Field>
                </DialogPanel>

                <DialogFooter className="justify-between sm:justify-between">
                    {isPlaced ? (
                        <Button type="button" variant="destructive-outline" size="sm" onClick={() => onPick(null)}>
                            Unplace
                        </Button>
                    ) : (
                        <span />
                    )}
                    <DialogClose render={<Button type="button" variant="outline" />}>Done</DialogClose>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}
