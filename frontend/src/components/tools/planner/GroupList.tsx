import { ChevronDown, Pencil, Pin, Trash } from "lucide-react";
import * as React from "react";

import { Button } from "#/components/ui/button";
import { Checkbox } from "#/components/ui/checkbox";
import { Skeleton } from "#/components/ui/skeleton";
import { Toggle } from "#/components/ui/toggle";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import type { IOperatorPlanResponse, IPlanGroup } from "#/lib/api/planner";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./OperatorPlanner.messages";
import { type IPlanEntryContext, PlanEntry } from "./PlanEntry";
import type { IPlannerSelection } from "./usePlannerSelection";

type PlannerT = TypedT<typeof messages>;

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
                    <div key={key} className="flex items-center gap-3 rounded-2xl border border-border/40 p-4">
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
    const expandLabel = isExpanded ? t("planner.group.collapse", { name: group.name }) : t("planner.group.expand", { name: group.name });
    // The 28px face stays for a mouse; a coarse pointer gets a centred 44px hit area on each, and the
    // row's gap widens to 16px so neighbouring areas (28 + 16 = 44 apart) do not overlap.
    const iconButton = "size-7 min-w-7 p-0 sm:size-7 sm:min-w-7";

    return (
        // The card is not itself a button: it holds the pin, expand, rename and delete buttons and the
        // plans' own checkboxes, and a role=button around them made every one a nested control. The
        // header label (checkbox plus name) is the selecting control.
        <div
            className={cn("relative flex flex-col rounded-2xl border p-4 transition-all hover:shadow-md", isExpanded ? "gap-4" : "gap-0", isSelected ? "border-primary bg-primary/5 shadow-sm ring-2 ring-primary/20" : isFilterEngaged ? "border-border/40 bg-card opacity-60 hover:opacity-100" : "border-border/40 bg-card")}
        >
            <div className={cn("flex items-center justify-between", isExpanded && "border-border/40 border-b pb-3")}>
                {/* biome-ignore lint/a11y/noLabelWithoutControl: Checkbox renders the control inside this label */}
                <label className="flex min-w-0 flex-1 cursor-pointer items-center gap-3">
                    <Checkbox checked={isSelected} onCheckedChange={onToggleSelected} />
                    <span className="truncate font-semibold text-foreground text-sm">{group.name}</span>
                </label>
                <div className="flex items-center gap-1 pointer-coarse:gap-4">
                    <Tooltip>
                        <TooltipTrigger render={<Toggle variant="outline" size="sm" aria-label={pinLabel} pressed={group.pinned} onPressedChange={() => onTogglePin()} className={iconButton} />}>
                            <Pin aria-hidden="true" className={cn("size-3.5", group.pinned && "fill-current")} />
                        </TooltipTrigger>
                        <TooltipPopup>{pinLabel}</TooltipPopup>
                    </Tooltip>
                    <Button variant="outline" size="icon-sm" aria-label={expandLabel} aria-expanded={isExpanded} onClick={onToggleExpanded} className={iconButton}>
                        <ChevronDown aria-hidden="true" className={cn("size-3.5 transition-transform", isExpanded && "rotate-180")} />
                    </Button>
                    <Button variant="outline" size="icon-sm" aria-label={t("planner.group.rename", { name: group.name })} onClick={onRename} className={iconButton}>
                        <Pencil aria-hidden="true" className="size-3.5" />
                    </Button>
                    <Button variant="destructive-outline" size="icon-sm" aria-label={t("planner.group.delete", { name: group.name })} onClick={onDelete} className={iconButton}>
                        <Trash aria-hidden="true" className="size-3.5" />
                    </Button>
                </div>
            </div>

            {isExpanded && <div className="group-plans-list mt-3 divide-y divide-border/30">{plans.length === 0 ? <p className="py-4 text-center text-muted-foreground text-xs">{t("planner.group.empty")}</p> : plans.map((p) => <PlanEntry key={p.id} plan={p} variant="nested" context={entryContext} />)}</div>}
        </div>
    );
}
