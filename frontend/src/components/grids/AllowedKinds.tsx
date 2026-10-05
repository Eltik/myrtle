import { useEffect, useState } from "react";
import { useEntityLabels } from "#/components/tier-lists/kinds";
import { Button } from "#/components/ui/button";
import { Checkbox } from "#/components/ui/checkbox";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { ALL_ENTITY_KINDS, type TierEntityKind } from "#/lib/api/tier-entities";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./AllowedKinds.messages";
import { orderKinds } from "./shared";

// A grid's allowed types: the checklist the create dialog and the editor pick
// them with, and the chips the page, the editor and the cards show them as.
// The checklist follows the tier-list kinds dialog (`PoolKindsDialog`): one
// checkbox per kind in `ALL_ENTITY_KINDS` order, the last one never unticked.

type KindsT = TypedT<typeof messages>;

interface IAllowedKindsChecklistProps {
    value: readonly TierEntityKind[];
    onChange: (kinds: TierEntityKind[]) => void;
    /** How many cells hold a pick of each kind, shown beside it. */
    placedByKind?: Partial<Record<TierEntityKind, number>>;
    disabled?: boolean;
}

export function AllowedKindsChecklist({ value, onChange, placedByKind, disabled = false }: IAllowedKindsChecklistProps) {
    const t: KindsT = useT("grids");
    const labels = useEntityLabels();
    const [refused, setRefused] = useState(false);

    const toggle = (kind: TierEntityKind, on: boolean) => {
        if (!on && value.length === 1 && value.includes(kind)) {
            setRefused(true);
            return;
        }
        setRefused(false);
        onChange(orderKinds(on ? [...value, kind] : value.filter((k) => k !== kind)));
    };

    return (
        <fieldset className="m-0 flex flex-col gap-2 border-0 p-0" disabled={disabled}>
            <legend className="mb-1 font-medium font-sans text-foreground text-sm">{t("kinds.label")}</legend>
            <p className="m-0 font-sans text-muted-foreground text-xs">{t("kinds.hint")}</p>
            <div className="grid grid-cols-2 gap-1.5 sm:grid-cols-3">
                {ALL_ENTITY_KINDS.map((kind) => {
                    const checked = value.includes(kind);
                    const placed = placedByKind?.[kind] ?? 0;
                    return (
                        // biome-ignore lint/a11y/noLabelWithoutControl: the Checkbox inside is the control; base-ui renders it as a checkbox button biome cannot see through
                        <label
                            key={kind}
                            title={labels.description(kind)}
                            className={cn("flex min-w-0 cursor-pointer items-center gap-2 rounded-lg border px-2.5 py-2 transition-colors", checked ? "border-[color-mix(in_srgb,var(--ring)_45%,var(--border))] bg-accent/30" : "border-border hover:bg-accent/40", disabled && "cursor-not-allowed opacity-60")}
                        >
                            <Checkbox checked={checked} onCheckedChange={(on) => toggle(kind, on)} disabled={disabled} />
                            <span className="min-w-0 flex-1 truncate font-medium font-sans text-[13px] text-foreground leading-none">{labels.plural(kind)}</span>
                            {placed > 0 && <span className="shrink-0 font-mono text-[10.5px] text-muted-foreground tabular-nums leading-none">{t("kinds.placed", { count: placed })}</span>}
                        </label>
                    );
                })}
            </div>
            {refused && (
                <p role="alert" className="m-0 font-sans text-[12.5px] text-destructive-foreground">
                    {t("kinds.atLeastOne")}
                </p>
            )}
        </fieldset>
    );
}

interface IAllowedKindsDialogProps {
    open: boolean;
    kinds: readonly TierEntityKind[];
    placedByKind: Partial<Record<TierEntityKind, number>>;
    onClose: () => void;
    onApply: (kinds: TierEntityKind[]) => void;
}

/** The editor's types dialog: a draft of the checklist, applied to the editor state on Apply. */
export function AllowedKindsDialog({ open, kinds, placedByKind, onClose, onApply }: IAllowedKindsDialogProps) {
    const t: KindsT = useT("grids");
    const [draft, setDraft] = useState<TierEntityKind[]>(() => orderKinds(kinds));

    useEffect(() => {
        if (open) setDraft(orderKinds(kinds));
    }, [open, kinds]);

    return (
        <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
            <DialogPopup className="sm:max-w-lg">
                <form
                    className="flex min-h-0 flex-col"
                    onSubmit={(e) => {
                        e.preventDefault();
                        if (draft.length > 0) onApply(draft);
                    }}
                >
                    <DialogHeader>
                        <DialogTitle>{t("kinds.dialog.title")}</DialogTitle>
                        <DialogDescription>{t("kinds.dialog.description")}</DialogDescription>
                    </DialogHeader>
                    <DialogPanel>
                        <AllowedKindsChecklist value={draft} onChange={setDraft} placedByKind={placedByKind} />
                    </DialogPanel>
                    <DialogFooter>
                        <DialogClose render={<Button type="button" variant="outline" />}>{t("kinds.dialog.cancel")}</DialogClose>
                        <Button type="submit" disabled={draft.length === 0}>
                            {t("kinds.dialog.apply")}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogPopup>
        </Dialog>
    );
}

interface IKindChipsProps {
    kinds: readonly TierEntityKind[];
    /** Show at most this many chips, then `+N`. Unset shows every kind. */
    max?: number;
    className?: string;
}

/** The allowed types as small chips, in tab order. */
export function KindChips({ kinds, max, className }: IKindChipsProps) {
    const t: KindsT = useT("grids");
    const labels = useEntityLabels();
    const ordered = orderKinds(kinds);
    // `+1` would take the room of the chip it hides, so a list one over the cap shows whole.
    const cut = max !== undefined && ordered.length > max + 1 ? max : ordered.length;
    const shown = ordered.slice(0, cut);
    const hidden = ordered.slice(cut);

    return (
        <ul className={cn("m-0 flex list-none flex-wrap items-center gap-1 p-0", className)} aria-label={t("kinds.label")}>
            {shown.map((kind) => (
                <li key={kind} className="rounded-sm border border-border bg-muted/60 px-1.5 py-0.5 font-sans text-[10.5px] text-muted-foreground leading-none">
                    {labels.plural(kind)}
                </li>
            ))}
            {hidden.length > 0 && (
                <li className="rounded-sm border border-border border-dashed px-1.5 py-0.5 font-mono text-[10.5px] text-muted-foreground tabular-nums leading-none" title={t("kinds.moreTitle", { kinds: hidden.map((k) => labels.plural(k)).join(", ") })}>
                    {t("kinds.more", { count: hidden.length })}
                </li>
            )}
        </ul>
    );
}
