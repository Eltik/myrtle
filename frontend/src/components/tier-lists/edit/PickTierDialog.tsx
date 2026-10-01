import { useId } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Field, FieldDescription, FieldLabel } from "#/components/ui/field";
import { MarkdownEditor } from "#/components/ui/markdown-editor";
import { entityOwner, type ITierEntity } from "#/lib/api/tier-entities";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { readableTextColor } from "../detail/contrast";
import { EntityAvatar, entityAccent } from "../entities";
import { PLACEMENT_DESCRIPTION_MAX } from "../shared";
import type { messages } from "./PickTierDialog.messages";
import type { IEditTier } from "./state";

interface IPickTierDialogProps {
    entity: ITierEntity | null;
    currentTierId: string | null;
    description: string;
    tiers: IEditTier[];
    onClose: () => void;
    onPick: (tierId: string | null) => void;
    onDescriptionChange: (description: string) => void;
}

export function PickTierDialog({ entity, currentTierId, description, tiers, onClose, onPick, onDescriptionChange }: IPickTierDialogProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const accent = entity ? entityAccent(entity) : null;
    const descId = useId();
    const isPlaced = currentTierId !== null;
    const hasTiers = tiers.length > 0;
    const owner = entity ? entityOwner(entity) : null;

    return (
        <Dialog open={entity !== null} onOpenChange={(o) => !o && onClose()}>
            <DialogPopup className="sm:max-w-sm">
                <DialogHeader>
                    <DialogTitle>
                        <span className="inline-flex items-center gap-2.5">
                            {entity && (
                                <span className="inline-flex h-8 w-8 shrink-0 items-center justify-center overflow-hidden rounded-md border-2" style={{ borderColor: accent ?? undefined, background: "var(--muted)" }}>
                                    <EntityAvatar entity={entity} />
                                </span>
                            )}
                            <span className="flex min-w-0 flex-col">
                                <span className="font-sans">{entity?.name ?? ""}</span>
                                {owner && <span className="font-normal font-sans text-muted-foreground text-xs leading-snug">{t("edit.pick.owner", { name: owner })}</span>}
                            </span>
                        </span>
                    </DialogTitle>
                    <DialogDescription>{hasTiers ? t("edit.pick.description") : t("edit.pick.descriptionNoTiers")}</DialogDescription>
                </DialogHeader>

                <DialogPanel className="flex flex-col gap-4">
                    <div className="flex flex-col gap-1.5">
                        <p className="m-0 font-bold font-mono text-[10.5px] text-muted-foreground/80 uppercase leading-none tracking-[0.16em]">{t("edit.pick.placement")}</p>
                        {!hasTiers && <p className="m-0 rounded-lg border border-border border-dashed px-3 py-3 text-muted-foreground text-sm">{t("edit.pick.noTiers")}</p>}
                        {tiers.map((tier) => {
                            const isCurrent = tier.id === currentTierId;
                            const fg = readableTextColor(tier.color);
                            return (
                                <button
                                    key={tier.id}
                                    type="button"
                                    className="flex items-center gap-3 rounded-lg border border-border px-2.5 py-2 text-left font-sans text-sm transition-colors hover:bg-accent/50 data-[selected=true]:border-ring data-[selected=true]:bg-accent/30"
                                    data-selected={isCurrent || undefined}
                                    aria-pressed={isCurrent}
                                    onClick={() => onPick(tier.id)}
                                >
                                    <span
                                        className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md font-extrabold font-sans text-[15px] leading-none tracking-tight"
                                        style={{
                                            background: tier.color,
                                            color: fg,
                                            textShadow: fg === "white" ? "0 1px 0 oklch(0 0 0 / 0.25)" : "0 1px 0 oklch(1 0 0 / 0.5)",
                                        }}
                                        aria-hidden="true"
                                    >
                                        {tier.name.length <= 2 ? tier.name : tier.name.charAt(0)}
                                    </span>
                                    <span className="min-w-0 flex-1 truncate">{tier.name}</span>
                                    <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{tier.entityKeys.length}</span>
                                    {isCurrent && <span className="font-bold font-mono text-[10px] text-primary uppercase tracking-wider">{t("edit.pick.current")}</span>}
                                </button>
                            );
                        })}
                    </div>

                    <Field>
                        <FieldLabel htmlFor={descId}>
                            {t("edit.pick.noteLabel")}
                            <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                {description.length} / {PLACEMENT_DESCRIPTION_MAX}
                            </span>
                        </FieldLabel>
                        <MarkdownEditor id={descId} value={description} onChange={onDescriptionChange} placeholder={t("edit.pick.notePlaceholder")} rows={4} maxLength={PLACEMENT_DESCRIPTION_MAX} showHint={false} />
                        <FieldDescription>{isPlaced ? t("edit.pick.noteHintPlaced") : hasTiers ? t("edit.pick.noteHintUnplaced") : t("edit.pick.noteHintNoTiers")}</FieldDescription>
                    </Field>
                </DialogPanel>

                <DialogFooter className="justify-between sm:justify-between">
                    {isPlaced ? (
                        <Button type="button" variant="destructive-outline" size="sm" onClick={() => onPick(null)}>
                            {t("edit.pick.unplace")}
                        </Button>
                    ) : (
                        <span />
                    )}
                    <DialogClose render={<Button type="button" variant="outline" />}>{t("edit.pick.done")}</DialogClose>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}
