import { ChevronDown, Pencil, Trash } from "lucide-react";
import type * as React from "react";

import { Checkbox } from "#/components/ui/checkbox";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { useOperatorName } from "#/hooks/use-operator-name";
import type { IOperatorPlanResponse } from "#/lib/api/planner";
import type { IRosterEntry } from "#/lib/api/user";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, formatSubProfession, rarityToNumber } from "#/lib/utils";
import type { messages } from "./OperatorPlanner.messages";
import { type IPlanProgressLabels, PlanProgressRows } from "./PlanProgressRows";
import { planProgress } from "./planProgress";
import { MAX_SKILL_LEVEL, masteryOf } from "./planTargets";

type PlannerT = TypedT<typeof messages>;

/** Everything a plan entry reads from or reports to the planner page; both tabs pass the same object. */
export interface IPlanEntryContext {
    /** Null when the player has no synced roster. */
    roster: IRosterEntry[] | null;
    isActive: (plan: IOperatorPlanResponse) => boolean;
    isExpanded: (plan: IOperatorPlanResponse) => boolean;
    onToggleActive: (plan: IOperatorPlanResponse) => void;
    onToggleExpanded: (plan: IOperatorPlanResponse) => void;
    onEdit: (plan: IOperatorPlanResponse) => void;
    onDelete: (plan: IOperatorPlanResponse) => void;
}

interface IPlanEntryProps {
    plan: IOperatorPlanResponse;
    /** `card` is a standalone card in the Plans tab; `nested` is a row inside a group card. */
    variant: "card" | "nested";
    context: IPlanEntryContext;
}

/** One plan: its header with the active checkbox, the current ➔ target rows when expanded, and its edit and delete buttons. */
export function PlanEntry({ plan, variant, context }: IPlanEntryProps): React.ReactElement | null {
    const op = plan.operator;
    if (!op) return null;
    const rosterEntry = context.roster?.find((re) => re.operator_id === plan.operator_id);
    const isActive = context.isActive(plan);
    const isExpanded = context.isExpanded(plan);

    const header = <PlanCardHeader op={op} isActive={isActive} onToggleActive={() => context.onToggleActive(plan)} isExpanded={isExpanded} onToggleExpanded={() => context.onToggleExpanded(plan)} />;
    const onEdit = () => context.onEdit(plan);
    const onDelete = () => context.onDelete(plan);

    if (variant === "nested") {
        return (
            <div className="flex flex-col gap-3 py-4 first:pt-1 last:pb-1">
                {header}
                {isExpanded && <PlanDetails plan={plan} op={op} rosterEntry={rosterEntry} className="pl-7" />}
                <PlanActions onEdit={onEdit} onDelete={onDelete} className="pl-7" dense />
            </div>
        );
    }

    return (
        /* biome-ignore lint/a11y/noLabelWithoutControl: PlanCardHeader renders the plan's Checkbox inside this label */
        <label className={cn("relative flex cursor-pointer flex-col gap-4 rounded-xl border p-4 transition-all hover:shadow-md", isActive ? "border-primary bg-primary/5 shadow-sm ring-2 ring-primary/20" : "border-border/40 bg-muted/20 opacity-60")}>
            {header}
            {isExpanded && <PlanDetails plan={plan} op={op} rosterEntry={rosterEntry} className="border-border/40 border-t pt-3" />}
            <PlanActions onEdit={onEdit} onDelete={onDelete} className="mt-auto border-border/40 border-t pt-3" />
        </label>
    );
}

/** Stops a click on an entry's own button from also toggling the label or group card around it. */
function handled(action: () => void) {
    return (e: React.MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
        action();
    };
}

interface IPlanCardHeaderProps {
    op: IOperatorPlanResponse["operator"];
    isActive: boolean;
    onToggleActive: () => void;
    isExpanded: boolean;
    onToggleExpanded: () => void;
}

function PlanCardHeader({ op, isActive, onToggleActive, isExpanded, onToggleExpanded }: IPlanCardHeaderProps): React.ReactElement {
    const operatorName = useOperatorName();
    const t: PlannerT = useT("tools");
    return (
        <div className="flex items-start gap-3">
            <Checkbox checked={isActive} onCheckedChange={onToggleActive} />
            <span aria-hidden="true" className="relative flex size-10 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-border bg-muted/70">
                <OperatorAvatar charId={op.id} name={op.name} className="block h-full w-full object-cover" server={op.server} />
            </span>
            <div className="min-w-0 flex-1">
                <h3 className="truncate font-bold text-foreground text-sm leading-tight">{operatorName(op)}</h3>
                <p className="mt-0.5 truncate text-muted-foreground text-xs leading-normal">{t("planner.card.rarityClass", { rarity: rarityToNumber(op.rarity), archetype: formatSubProfession(op.subProfessionId) })}</p>
            </div>
            <button type="button" onClick={handled(onToggleExpanded)} className="flex size-7 shrink-0 items-center justify-center rounded-md border border-border bg-muted/40 text-foreground shadow-xs transition-all hover:border-border/80 hover:bg-muted">
                <ChevronDown className={cn("size-4 transition-transform", isExpanded && "rotate-180")} />
            </button>
        </div>
    );
}

interface IPlanDetailsProps {
    plan: IOperatorPlanResponse;
    op: IOperatorPlanResponse["operator"];
    rosterEntry: IRosterEntry | undefined;
    className?: string;
}

function PlanDetails({ plan, op, rosterEntry, className }: IPlanDetailsProps): React.ReactElement {
    const t: PlannerT = useT("tools");
    const labels: IPlanProgressLabels = {
        level: t("planner.card.level"),
        skills: t("planner.card.skills"),
        modules: t("planner.card.modules"),
        eliteAlt: (elite) => t("planner.card.eliteAlt", { elite }),
        levelValue: (level) => t("planner.card.levelValue", { level }),
        skillFallback: (index) => t("planner.card.skillFallback", { index }),
        skillTarget: (value) => (value <= MAX_SKILL_LEVEL ? String(value) : t("planner.card.mastery", { mastery: masteryOf(value) })),
        // Not "X": it sat next to the module's designator and X is itself a designator letter
        // (typeName2 is one of A, B, D, X, Y), so "SUM-X  X ➔ 3" read as though the stage column
        // were naming the module. An em dash cannot be mistaken for one.
        moduleStage: (stage) => (stage === 0 ? "\u2014" : String(stage)),
    };

    return <PlanProgressRows progress={planProgress(op, plan, rosterEntry)} server={op.server} labels={labels} className={cn("fade-in slide-in-from-top-2 animate-in duration-200", className)} />;
}

interface IPlanActionsProps {
    onEdit: () => void;
    onDelete: () => void;
    className?: string;
    dense?: boolean;
}

function PlanActions({ onEdit, onDelete, className, dense = false }: IPlanActionsProps): React.ReactElement {
    const t: PlannerT = useT("tools");
    const buttonBase = cn("flex flex-1 cursor-pointer items-center justify-center gap-1.5 rounded-lg border font-medium font-sans text-xs transition-all", dense ? "py-1" : "py-1.5");

    return (
        <div className={cn("flex items-center gap-2", className)}>
            <button type="button" onClick={handled(onEdit)} className={cn(buttonBase, "border-border bg-muted/40 text-foreground hover:border-border/80 hover:bg-muted")}>
                <Pencil className="size-3.5" />
                <span>{t("planner.card.edit")}</span>
            </button>
            <button type="button" onClick={handled(onDelete)} className={cn(buttonBase, "border-red-500/20 bg-red-500/5 text-red-600 hover:border-red-500/40 hover:bg-red-500/15 dark:text-red-400")}>
                <Trash className="size-3.5" />
                <span>{t("planner.card.delete")}</span>
            </button>
        </div>
    );
}
