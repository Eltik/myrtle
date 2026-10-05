import { CircleCheckIcon } from "lucide-react";
import { useState } from "react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { Checkbox } from "#/components/ui/checkbox";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { IOperatorPlanResponse } from "#/lib/api/planner";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./CompletedPlansDialog.messages";
import { formatPlanTarget } from "./requirements";
import type { messages as requirementMessages } from "./requirements.messages";

type CompletedT = TypedT<typeof messages & typeof requirementMessages>;

interface ICompletedPlansDialogProps {
    /** The met plans to review, or null when closed. A snapshot, so a refetch mid-dialog cannot reshuffle the list. */
    plans: IOperatorPlanResponse[] | null;
    onOpenChange: (open: boolean) => void;
    onDelete: (ids: string[]) => void;
    onKeep: () => void;
    isSubmitting: boolean;
    errorMessage: string | null;
}

export function CompletedPlansDialog({ plans, onOpenChange, onDelete, onKeep, isSubmitting, errorMessage }: ICompletedPlansDialogProps) {
    const t: CompletedT = useT("tools");
    const open = plans !== null;

    // Same close-animation hold as DeletePlansDialog, and a fresh "all checked"
    // each time a new list arrives.
    const [lastPlans, setLastPlans] = useState<IOperatorPlanResponse[] | null>(null);
    const [unchecked, setUnchecked] = useState<Set<string>>(() => new Set());
    if (plans !== null && plans !== lastPlans) {
        setLastPlans(plans);
        setUnchecked(new Set());
    }
    const shown = plans ?? lastPlans ?? [];
    const checkedIds = shown.map((p) => p.operator_id).filter((id) => !unchecked.has(id));

    const toggle = (id: string) =>
        setUnchecked((prev) => {
            const next = new Set(prev);
            if (next.has(id)) next.delete(id);
            else next.add(id);
            return next;
        });

    return (
        <AlertDialog open={open} onOpenChange={onOpenChange}>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <div className="flex items-center gap-3">
                        <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-success/12 text-success-foreground">
                            <CircleCheckIcon className="h-4.5 w-4.5" aria-hidden="true" />
                        </span>
                        <div className="flex min-w-0 flex-col gap-1">
                            <AlertDialogTitle>{t("planner.completed.title", { count: shown.length })}</AlertDialogTitle>
                            <AlertDialogDescription>{t("planner.completed.desc")}</AlertDialogDescription>
                        </div>
                    </div>
                </AlertDialogHeader>

                <ul className="mx-6 mb-2 flex max-h-[min(50vh,360px)] flex-col divide-y divide-border/40 overflow-y-auto rounded-lg border border-border/60">
                    {shown.map((p) => {
                        const op = p.operator;
                        const checked = !unchecked.has(p.operator_id);
                        return (
                            <li key={p.operator_id}>
                                {/* biome-ignore lint/a11y/noLabelWithoutControl: Checkbox component internally renders the input control */}
                                <label className="flex cursor-pointer items-center gap-3 px-3 py-2 transition-colors hover:bg-muted/30">
                                    <Checkbox checked={checked} onCheckedChange={() => toggle(p.operator_id)} disabled={isSubmitting} />
                                    <span aria-hidden="true" className="relative flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-lg border border-border bg-muted/70">
                                        {op && <OperatorAvatar charId={op.id} name={op.name} className="block h-full w-full object-cover" server={op.server} />}
                                    </span>
                                    <span className="min-w-0 flex-1">
                                        <span className="block truncate font-semibold text-foreground text-xs leading-tight">{op?.name ?? p.operator_id}</span>
                                        <span className="block truncate text-[11px] text-muted-foreground leading-normal">{formatPlanTarget(p, t)}</span>
                                    </span>
                                </label>
                            </li>
                        );
                    })}
                </ul>

                {errorMessage && (
                    <div role="alert" className="mx-6 mb-2 rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 font-sans text-destructive-foreground text-xs">
                        {errorMessage}
                    </div>
                )}

                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" disabled={isSubmitting} onClick={onKeep} />}>{t("planner.completed.keep")}</AlertDialogClose>
                    <Button type="button" variant="destructive" loading={isSubmitting} disabled={checkedIds.length === 0} onClick={() => onDelete(checkedIds)}>
                        {t("planner.completed.delete", { count: checkedIds.length })}
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    );
}
