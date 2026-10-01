import { useEffect, useState } from "react";
import { Button } from "#/components/ui/button";
import { Checkbox } from "#/components/ui/checkbox";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { ALL_ENTITY_KINDS, type TierEntityKind } from "#/lib/api/tier-entities";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { useEntityLabels } from "../kinds";
import type { messages } from "./PoolKindsDialog.messages";

interface IPoolKindsDialogProps {
    open: boolean;
    /** The kinds the list offers now (edited, not yet saved, included). */
    kinds: readonly TierEntityKind[];
    /** How many placements of each kind are on the board. */
    placedByKind: Partial<Record<TierEntityKind, number>>;
    onClose: () => void;
    onApply: (kinds: TierEntityKind[]) => void;
}

/**
 * Which kinds a list's pool offers. Unticking a kind never removes what is
 * already on the board: those placements stay, and only the pool tab goes. At
 * least one kind stays ticked, which the backend also enforces.
 */
export function PoolKindsDialog({ open, kinds, placedByKind, onClose, onApply }: IPoolKindsDialogProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const labels = useEntityLabels();
    const [draft, setDraft] = useState<Set<TierEntityKind>>(() => new Set(kinds));
    const [refused, setRefused] = useState(false);

    useEffect(() => {
        if (!open) return;
        setDraft(new Set(kinds));
        setRefused(false);
    }, [open, kinds]);

    const toggle = (kind: TierEntityKind, on: boolean) => {
        if (!on && draft.size === 1 && draft.has(kind)) {
            setRefused(true);
            return;
        }
        setRefused(false);
        setDraft((prev) => {
            const next = new Set(prev);
            if (on) next.add(kind);
            else next.delete(kind);
            return next;
        });
    };

    // Kinds keep the list's own order; a newly ticked kind joins at its place in the canonical order.
    const ordered = (): TierEntityKind[] => {
        const kept = kinds.filter((k) => draft.has(k));
        const added = ALL_ENTITY_KINDS.filter((k) => draft.has(k) && !kinds.includes(k));
        return [...kept, ...added];
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (draft.size === 0) return;
        onApply(ordered());
    };

    return (
        <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
            <DialogPopup className="sm:max-w-md">
                <form onSubmit={handleSubmit} className="flex min-h-0 flex-col">
                    <DialogHeader>
                        <DialogTitle>{t("edit.kinds.title")}</DialogTitle>
                        <DialogDescription>{t("edit.kinds.description")}</DialogDescription>
                    </DialogHeader>

                    <DialogPanel>
                        <fieldset className="m-0 flex flex-col gap-1 border-0 p-0">
                            <legend className="sr-only">{t("edit.kinds.groupLabel")}</legend>
                            {ALL_ENTITY_KINDS.map((kind) => {
                                const checked = draft.has(kind);
                                const placed = placedByKind[kind] ?? 0;
                                return (
                                    // biome-ignore lint/a11y/noLabelWithoutControl: the Checkbox inside is the control; base-ui renders it as a checkbox button biome cannot see through
                                    <label key={kind} className={cn("flex cursor-pointer items-start gap-3 rounded-lg border px-3 py-2.5 transition-colors", checked ? "border-[color-mix(in_srgb,var(--ring)_45%,var(--border))] bg-accent/30" : "border-border hover:bg-accent/40")}>
                                        <Checkbox checked={checked} onCheckedChange={(on) => toggle(kind, on)} className="mt-0.5" />
                                        <span className="flex min-w-0 flex-1 flex-col gap-1">
                                            <span className="flex items-baseline justify-between gap-2">
                                                <span className="font-medium font-sans text-[13.5px] text-foreground leading-none">{labels.plural(kind)}</span>
                                                {placed > 0 && <span className="shrink-0 font-mono text-[10.5px] text-muted-foreground tabular-nums leading-none">{t("edit.kinds.placed", { count: placed })}</span>}
                                            </span>
                                            <span className="font-sans text-[12px] text-muted-foreground leading-snug">{labels.description(kind)}</span>
                                        </span>
                                    </label>
                                );
                            })}
                        </fieldset>
                        {refused && (
                            <p role="alert" className="m-0 mt-3 font-sans text-[12.5px] text-destructive-foreground">
                                {t("edit.kinds.atLeastOne")}
                            </p>
                        )}
                    </DialogPanel>

                    <DialogFooter>
                        <DialogClose render={<Button type="button" variant="outline" />}>{t("edit.kinds.cancel")}</DialogClose>
                        <Button type="submit" disabled={draft.size === 0}>
                            {t("edit.kinds.apply")}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogPopup>
        </Dialog>
    );
}
