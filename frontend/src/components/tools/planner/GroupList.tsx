import { ChevronDown, Pencil, Pin, Trash } from "lucide-react";
import * as React from "react";

import { Checkbox } from "#/components/ui/checkbox";
import { Skeleton } from "#/components/ui/skeleton";
import type { IOperatorPlanResponse, IPlanGroup } from "#/lib/api/planner";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./OperatorPlanner.messages";
import { type IPlanEntryContext, PlanEntry } from "./PlanEntry";
import type { IPlannerSelection } from "./usePlannerSelection";

type PlannerT = TypedT<typeof messages>;

/** Group cards toggle on background clicks only - clicks on nested controls must not double-fire. */
function isInteractiveGroupChild(target: HTMLElement): boolean {
    return Boolean(target.closest("button") || target.closest("input") || target.closest("[role='checkbox']") || target.closest(".group-plans-list"));
}

interface IGroupListProps {
    plans: IOperatorPlanResponse[];
    groups: IPlanGroup[];
    /** Index of the first unpinned group when pinned and unpinned groups are both present; -1 hides the divider. */
    pinDividerAt: number;
    isLoading: boolean;
    selection: IPlannerSelection;
    entryContext: IPlanEntryContext;
    /** Held by the page: the tab panel unmounts when hidden, and a group's open state outlives that. */
    expandedGroups: Readonly<Record<string, boolean>>;
    onExpandedGroupsChange: React.Dispatch<React.SetStateAction<Record<string, boolean>>>;
    onTogglePin: (name: string, pinned: boolean) => void;
    onRename: (name: string) => void;
    onDelete: (name: string) => void;
}

/** The Groups tab: a card per group that toggles it in the group filter, with its plans listed inside. */
export function GroupList({ plans, groups, pinDividerAt, isLoading, selection, entryContext, expandedGroups, onExpandedGroupsChange, onTogglePin, onRename, onDelete }: IGroupListProps): React.ReactElement {
    const t: PlannerT = useT("tools");

    if (isLoading) {
        return (
            <div className="flex flex-col gap-4">
                {["sk-grp-1", "sk-grp-2", "sk-grp-3"].map((key) => (
                    <div key={key} className="flex items-center gap-3 rounded-xl border border-border/40 p-4">
                        <Skeleton className="size-4 rounded" />
                        <div className="flex flex-1 flex-col gap-1.5">
                            <Skeleton className="h-3.5 w-24 rounded" />
                        </div>
                    </div>
                ))}
            </div>
        );
    }

    if (groups.length === 0) {
        return <p className="py-4 text-center text-muted-foreground text-xs">{t("planner.group.none")}</p>;
    }

    return (
        <div className="flex flex-col gap-6">
            {groups.map((g, i) => {
                const isExpanded = expandedGroups[g.name] ?? true;
                return (
                    <React.Fragment key={g.name}>
                        {i === pinDividerAt && <hr className="-my-3 border-border/60 border-dashed" />}
                        <GroupCard
                            group={g}
                            plans={plans.filter((p) => p.groups?.includes(g.name))}
                            isSelected={selection.groupFilter.has(g.name)}
                            isFilterEngaged={selection.groupFilter.size > 0}
                            isExpanded={isExpanded}
                            onToggleSelected={() => selection.toggleGroupFilter(plans, g.name)}
                            onToggleExpanded={() => onExpandedGroupsChange((prev) => ({ ...prev, [g.name]: !isExpanded }))}
                            onTogglePin={() => onTogglePin(g.name, !g.pinned)}
                            onRename={() => onRename(g.name)}
                            onDelete={() => onDelete(g.name)}
                            entryContext={entryContext}
                        />
                    </React.Fragment>
                );
            })}
        </div>
    );
}

interface IGroupCardProps {
    group: IPlanGroup;
    /** The plans in this group. */
    plans: IOperatorPlanResponse[];
    isSelected: boolean;
    /** Whether any group is selected, which dims the unselected cards. */
    isFilterEngaged: boolean;
    isExpanded: boolean;
    onToggleSelected: () => void;
    onToggleExpanded: () => void;
    onTogglePin: () => void;
    onRename: () => void;
    onDelete: () => void;
    entryContext: IPlanEntryContext;
}

function GroupCard({ group, plans, isSelected, isFilterEngaged, isExpanded, onToggleSelected, onToggleExpanded, onTogglePin, onRename, onDelete, entryContext }: IGroupCardProps): React.ReactElement {
    const t: PlannerT = useT("tools");
    const pinLabel = group.pinned ? t("planner.group.unpin", { name: group.name }) : t("planner.group.pin", { name: group.name });
    const iconButton = "flex size-7 items-center justify-center rounded-md border transition-all";
    const neutralButton = "border-border bg-muted/40 text-foreground hover:border-border/80 hover:bg-muted";

    return (
        // biome-ignore lint/a11y/useSemanticElements: custom interactive group wrapper
        <div
            role="button"
            tabIndex={0}
            onClick={(e) => {
                if (isInteractiveGroupChild(e.target as HTMLElement)) return;
                onToggleSelected();
            }}
            onKeyDown={(e) => {
                if (e.key !== "Enter" && e.key !== " ") return;
                if (isInteractiveGroupChild(e.target as HTMLElement)) return;
                e.preventDefault();
                onToggleSelected();
            }}
            className={cn(
                "relative flex cursor-pointer flex-col rounded-xl border p-4 transition-all hover:shadow-md focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary/50",
                isExpanded ? "gap-4" : "gap-0",
                isSelected ? "border-primary bg-primary/5 shadow-sm ring-2 ring-primary/20" : isFilterEngaged ? "border-border/40 bg-card opacity-60 hover:opacity-100" : "border-border/40 bg-card",
            )}
        >
            <div className={cn("flex items-center justify-between", isExpanded && "border-border/40 border-b pb-3")}>
                <div className="flex items-center gap-3">
                    <Checkbox checked={isSelected} onCheckedChange={onToggleSelected} />
                    <span className="font-semibold text-foreground text-sm">{group.name}</span>
                </div>
                <div className="flex items-center gap-1">
                    <button
                        type="button"
                        aria-label={pinLabel}
                        aria-pressed={group.pinned}
                        title={pinLabel}
                        onClick={onTogglePin}
                        className={cn(iconButton, group.pinned ? "border-primary/40 bg-primary/10 text-primary hover:bg-primary/15" : "border-border bg-muted/40 text-muted-foreground hover:border-border/80 hover:bg-muted hover:text-foreground")}
                    >
                        <Pin className={cn("size-3.5", group.pinned && "fill-current")} />
                    </button>
                    <button type="button" onClick={onToggleExpanded} className={cn(iconButton, neutralButton)}>
                        <ChevronDown className={cn("size-3.5 transition-transform", isExpanded && "rotate-180")} />
                    </button>
                    <button type="button" onClick={onRename} className={cn(iconButton, neutralButton)}>
                        <Pencil className="size-3.5" />
                    </button>
                    <button type="button" onClick={onDelete} className={cn(iconButton, "border-red-500/20 bg-red-500/5 text-red-600 hover:border-red-500/40 hover:bg-red-500/15 dark:text-red-400")}>
                        <Trash className="size-3.5" />
                    </button>
                </div>
            </div>

            {isExpanded && <div className="group-plans-list mt-3 divide-y divide-border/30">{plans.length === 0 ? <p className="py-4 text-center text-muted-foreground text-xs">{t("planner.group.empty")}</p> : plans.map((p) => <PlanEntry key={p.id} plan={p} variant="nested" context={entryContext} />)}</div>}
        </div>
    );
}
