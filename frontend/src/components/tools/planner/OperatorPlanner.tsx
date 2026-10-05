import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ChevronDown, CircleCheck, Lock, Pencil, Pin, Plus, Trash } from "lucide-react";
import * as React from "react";

import { eliteIcon, moduleIconURL, skillIconURL } from "#/components/operators/detail/impl/assets";
import { Button } from "#/components/ui/button";
import { Checkbox } from "#/components/ui/checkbox";
import { FilterChip } from "#/components/ui/filter-chip";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { PageHeader } from "#/components/ui/page-header";
import { Skeleton } from "#/components/ui/skeleton";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "#/components/ui/tabs";
import { useAuth } from "#/hooks/use-auth";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { useOperatorName } from "#/hooks/use-operator-name";
import { deleteGroupFn, deletePlanFn, deletePlansFn, type IOperatorPlanResponse, plansQueryOptions, setGroupPinnedFn, upsertGroupFn } from "#/lib/api/planner";
import { type IRosterEntry, userRosterQueryOptions } from "#/lib/api/user";
import { authActions } from "#/lib/auth/store";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, formatSubProfession, rarityToNumber } from "#/lib/utils";
import { BulkPlanButton } from "./BulkPlanDialog";
import { CompletedPlansDialog } from "./CompletedPlansDialog";
import { DeletePlansDialog, type IDeletePlansTarget } from "./DeletePlansDialog";
import type { messages } from "./OperatorPlanner.messages";
import { OperatorPlannerDialog } from "./OperatorPlannerDialog";
import { activePlansForFilter, type GroupFilter, groupFilterCounts, isPlanActive, renameGroupFilterKey, toggleGroupFilterKey, UNGROUPED_FILTER_KEY } from "./planFilters";
import { PLANS_QUERY_PREFIX } from "./planTargets";
import { RequirementsPanel } from "./RequirementsPanel";
import { MAX_TIER_STORAGE_KEY, type MaxTierFilter, parseMaxTier } from "./requirements";

type PlannerT = TypedT<typeof messages>;

function UnauthenticatedState() {
    const t: PlannerT = useT("tools");
    return (
        <div className="mt-8 flex flex-col items-center justify-center gap-6 rounded-[14px] border border-border bg-card px-8 py-16 text-center">
            <div className="flex flex-col items-center gap-3">
                <div className="flex h-14 w-14 items-center justify-center rounded-2xl border border-border bg-muted text-2xl">
                    <Lock />
                </div>
                <div>
                    <h2 className="font-sans font-semibold text-[20px] text-foreground tracking-[-0.02em]">{t("planner.signIn.title")}</h2>
                    <p className="mt-1.5 max-w-[42ch] font-sans text-muted-foreground text-sm">{t("planner.signIn.desc")}</p>
                </div>
            </div>
            <button type="button" onClick={() => authActions.openLoginDialog()} className="inline-flex h-9 cursor-pointer items-center rounded-lg bg-primary px-5 font-medium font-sans text-primary-foreground text-sm transition-opacity hover:opacity-90">
                {t("planner.signIn.action")}
            </button>
        </div>
    );
}

function getSkillLevelLabel(level: number, t: PlannerT): string {
    if (level <= 7) return String(level);
    return t("planner.card.mastery", { mastery: level - 7 });
}

/**
 * Stage 0 means "not unlocked / not planned". Not "X": it sat next to the module's designator
 * and X is itself a designator letter (typeName2 is one of A, B, D, X, Y), so "SUM-X  X ➔ 3" read
 * as though the stage column were naming the module. An em dash cannot be mistaken for one.
 */
function getModuleStageLabel(stage: number): string {
    return stage === 0 ? "\u2014" : String(stage);
}

function currentSkillValue(rosterEntry: IRosterEntry | undefined, skillIndex: number): number {
    if (!rosterEntry) return 1;
    if (rosterEntry.skill_level < 7) return rosterEntry.skill_level;
    const mastery = rosterEntry.masteries?.find((m) => m.index === skillIndex)?.mastery ?? 0;
    return mastery > 0 ? 7 + mastery : 7;
}

function targetSkillValue(plan: IOperatorPlanResponse, skillIndex: number): number {
    if (plan.target_skill_level < 7) return plan.target_skill_level;
    const mastery = plan.target_skills?.find((s) => s.skill_index === skillIndex)?.mastery_level ?? 0;
    return mastery > 0 ? 7 + mastery : 7;
}

interface PlanCardHeaderProps {
    op: IOperatorPlanResponse["operator"];
    isActive: boolean;
    onToggleActive: () => void;
    isExpanded: boolean;
    onToggleExpanded: () => void;
}

function PlanCardHeader({ op, isActive, onToggleActive, isExpanded, onToggleExpanded }: PlanCardHeaderProps) {
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
            <button
                type="button"
                onClick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    onToggleExpanded();
                }}
                className="flex size-7 shrink-0 items-center justify-center rounded-md border border-border bg-muted/40 text-foreground shadow-xs transition-all hover:border-border/80 hover:bg-muted"
            >
                <ChevronDown className={cn("size-4 transition-transform", isExpanded && "rotate-180")} />
            </button>
        </div>
    );
}

interface PlanDetailsProps {
    plan: IOperatorPlanResponse;
    op: IOperatorPlanResponse["operator"];
    rosterEntry: IRosterEntry | undefined;
    className?: string;
}

function PlanDetails({ plan, op, rosterEntry, className }: PlanDetailsProps) {
    const t: PlannerT = useT("tools");
    const currElite = rosterEntry?.elite ?? 0;
    const currLevel = rosterEntry?.level ?? 1;
    const isLevelUpgraded = plan.target_elite > currElite || (plan.target_elite === currElite && plan.target_level > currLevel);
    const modules = op.modules.filter((m) => m.typeName1 !== "ORIGINAL");

    return (
        <div className={cn("fade-in slide-in-from-top-2 flex animate-in flex-col gap-3 text-xs duration-200", className)}>
            <div className="flex items-center justify-between">
                <span className="font-medium text-muted-foreground">{t("planner.card.level")}</span>
                <div className="flex items-center gap-2">
                    <div className="flex items-center gap-1 font-medium">
                        <img src={eliteIcon(currElite)} alt={t("planner.card.eliteAlt", { elite: currElite })} className="icon-theme-aware size-5 object-contain" />
                        <span>{t("planner.card.levelValue", { level: currLevel })}</span>
                    </div>
                    <span className="text-muted-foreground/50">➔</span>
                    <div className={cn("flex items-center gap-1 font-bold", isLevelUpgraded ? "text-primary" : "text-muted-foreground")}>
                        <img src={eliteIcon(plan.target_elite)} alt={t("planner.card.eliteAlt", { elite: plan.target_elite })} className={cn("icon-theme-aware size-5 object-contain", !isLevelUpgraded && "opacity-50")} />
                        <span>{t("planner.card.levelValue", { level: plan.target_level })}</span>
                    </div>
                </div>
            </div>

            {op.skills.length > 0 && (
                <div className="flex flex-col gap-2">
                    <span className="font-medium text-muted-foreground">{t("planner.card.skills")}</span>
                    <div className="flex flex-col gap-1.5 pl-1">
                        {op.skills.map((skill, idx) => {
                            const currSkillVal = currentSkillValue(rosterEntry, idx);
                            const targetSkillVal = targetSkillValue(plan, idx);
                            const isSkillUpgraded = targetSkillVal > currSkillVal;

                            return (
                                <div key={skill.skillId} className="flex items-center justify-between">
                                    <div className="flex min-w-0 flex-1 items-center gap-1.5">
                                        <img src={skillIconURL(skill, op.server)} alt={skill.static?.levels?.[0]?.name} className="size-5 rounded border border-border/40 object-contain" />
                                        <span className="truncate font-medium text-foreground">{skill.static?.levels?.[0]?.name ?? t("planner.card.skillFallback", { index: idx + 1 })}</span>
                                    </div>
                                    <div className="ml-2 flex shrink-0 items-center gap-2">
                                        <span className="font-medium">{getSkillLevelLabel(currSkillVal, t)}</span>
                                        <span className="text-muted-foreground/50">➔</span>
                                        <span className={cn("font-bold", isSkillUpgraded ? "text-primary" : "text-muted-foreground")}>{getSkillLevelLabel(targetSkillVal, t)}</span>
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                </div>
            )}

            {modules.length > 0 && (
                <div className="flex flex-col gap-2">
                    <span className="font-medium text-muted-foreground">{t("planner.card.modules")}</span>
                    <div className="flex flex-col gap-1.5 pl-1">
                        {modules.map((mod) => {
                            const modEntry = rosterEntry?.modules?.find((rm) => rm.id === mod.uniEquipId);
                            const currModStage = modEntry && !modEntry.locked ? modEntry.level : 0;
                            const targetModStage = plan.target_modules?.find((tm) => tm.module_id === mod.uniEquipId)?.module_stage ?? 0;
                            const isModUpgraded = targetModStage > currModStage;

                            return (
                                <div key={mod.uniEquipId} className="flex items-center justify-between">
                                    <div className="flex min-w-0 flex-1 items-center gap-1.5">
                                        <img src={moduleIconURL(mod, op.server)} alt={mod.uniEquipName} className="size-5 rounded object-contain" />
                                        <span className="truncate font-medium text-foreground">{mod.uniEquipName}</span>
                                    </div>
                                    <div className="ml-2 flex shrink-0 items-center gap-2">
                                        <span className="font-medium">{getModuleStageLabel(currModStage)}</span>
                                        <span className="text-muted-foreground/50">➔</span>
                                        <span className={cn("font-bold", isModUpgraded ? "text-primary" : "text-muted-foreground")}>{getModuleStageLabel(targetModStage)}</span>
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                </div>
            )}
        </div>
    );
}

interface PlanActionsProps {
    onEdit: () => void;
    onDelete: () => void;
    className?: string;
    dense?: boolean;
}

function PlanActions({ onEdit, onDelete, className, dense = false }: PlanActionsProps) {
    const t: PlannerT = useT("tools");
    const buttonBase = cn("flex flex-1 cursor-pointer items-center justify-center gap-1.5 rounded-lg border font-medium font-sans text-xs transition-all", dense ? "py-1" : "py-1.5");
    const handle = (action: () => void) => (e: React.MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
        action();
    };

    return (
        <div className={cn("flex items-center gap-2", className)}>
            <button type="button" onClick={handle(onEdit)} className={cn(buttonBase, "border-border bg-muted/40 text-foreground hover:border-border/80 hover:bg-muted")}>
                <Pencil className="size-3.5" />
                <span>{t("planner.card.edit")}</span>
            </button>
            <button type="button" onClick={handle(onDelete)} className={cn(buttonBase, "border-red-500/20 bg-red-500/5 text-red-600 hover:border-red-500/40 hover:bg-red-500/15 dark:text-red-400")}>
                <Trash className="size-3.5" />
                <span>{t("planner.card.delete")}</span>
            </button>
        </div>
    );
}

/** Group cards toggle on background clicks only - clicks on nested controls must not double-fire. */
function isInteractiveGroupChild(target: HTMLElement): boolean {
    return Boolean(target.closest("button") || target.closest("input") || target.closest("[role='checkbox']") || target.closest(".group-plans-list"));
}

export function OperatorPlanner(): React.ReactElement {
    const operatorName = useOperatorName();
    const t: PlannerT = useT("tools");
    const { isAuthenticated, user } = useAuth();
    const queryClient = useQueryClient();
    const [open, setOpen] = React.useState(false);
    const [expandedGroups, setExpandedGroups] = React.useState<Record<string, boolean>>({});
    const [activePlans, setActivePlans] = React.useState<Record<string, boolean>>({});
    const [groupFilter, setGroupFilter] = React.useState<GroupFilter>(() => new Set());
    const [expandedPlans, setExpandedPlans] = React.useState<Record<string, boolean>>({});
    const [editOperatorId, setEditOperatorId] = React.useState<string | null>(null);
    const [deleteTarget, setDeleteTarget] = React.useState<IDeletePlansTarget | null>(null);
    const [isDeleting, setIsDeleting] = React.useState(false);
    const [deleteError, setDeleteError] = React.useState<string | null>(null);
    const [reviewPlans, setReviewPlans] = React.useState<IOperatorPlanResponse[] | null>(null);
    const [isDeletingCompleted, setIsDeletingCompleted] = React.useState(false);
    const [completedError, setCompletedError] = React.useState<string | null>(null);
    const [maxTier, setMaxTier] = useLocalStorageState<MaxTierFilter>(MAX_TIER_STORAGE_KEY, 0, { parse: parseMaxTier, serialize: String });

    const { data: initialPlannerData, isLoading: initialPlansLoading } = useQuery({
        ...plansQueryOptions(),
        enabled: isAuthenticated,
    });

    const activeIds = React.useMemo(() => {
        if (!initialPlannerData?.plans) return undefined;
        const totalPlansCount = initialPlannerData.plans.length;
        const activeList = initialPlannerData.plans.filter((p) => isPlanActive(p, activePlans, groupFilter)).map((p) => p.operator_id);

        if (activeList.length === totalPlansCount) {
            return undefined;
        }
        if (totalPlansCount > 0 && activeList.length === 0) {
            return ["none"];
        }
        return activeList;
    }, [initialPlannerData?.plans, activePlans, groupFilter]);

    const { data: roster = [], isLoading: rosterLoading } = useQuery({
        ...userRosterQueryOptions(user?.uid ?? ""),
        enabled: !!user?.uid,
    });

    const isPlansListLoading = initialPlansLoading || rosterLoading;

    const { data: plannerData, isLoading: requirementsLoading } = useQuery({
        ...plansQueryOptions(activeIds, maxTier || undefined),
        enabled: isAuthenticated && !isPlansListLoading,
    });

    const isRequirementsLoading = requirementsLoading || isPlansListLoading;

    const plans = plannerData?.plans ?? initialPlannerData?.plans ?? [];
    const aggregatedRequirements = plannerData?.aggregatedRequirements ?? [];
    const groups = plannerData?.groups ?? initialPlannerData?.groups ?? [];
    const isActive = (p: IOperatorPlanResponse) => isPlanActive(p, activePlans, groupFilter);
    const activePlansList = plans.filter(isActive);
    const filterCounts = groupFilterCounts(plans);
    const ungroupedCount = filterCounts.get(UNGROUPED_FILTER_KEY) ?? 0;

    const pinnedGroupCount = groups.filter((g) => g.pinned).length;
    const showPinDivider = pinnedGroupCount > 0 && pinnedGroupCount < groups.length;

    const allSelected = plans.length > 0 && activePlansList.length === plans.length;
    const selectedPlansCount = activePlansList.length;

    const toggleSelectAll = () => {
        const next: Record<string, boolean> = {};
        for (const p of plans) {
            next[p.operator_id] = !allSelected;
        }
        setGroupFilter(new Set());
        setActivePlans(next);
    };

    /** Engaging the filter rewrites every plan's entry, so per-plan picks made before it do not survive it. */
    const applyGroupFilter = (next: GroupFilter) => {
        setGroupFilter(next);
        setActivePlans(activePlansForFilter(plans, next));
    };

    const requestDeletePlan = (plan: IOperatorPlanResponse) => {
        setDeleteError(null);
        setDeleteTarget({ ids: [plan.operator_id], names: [plan.operator ? operatorName(plan.operator) : t("planner.unknownOperator")] });
    };

    const requestDeleteSelected = () => {
        if (activePlansList.length === 0) return;
        setDeleteError(null);
        setDeleteTarget({
            ids: activePlansList.map((p) => p.operator_id),
            names: activePlansList.map((p) => (p.operator ? operatorName(p.operator) : t("planner.unknownOperator"))),
        });
    };

    const handleConfirmDelete = async (ids: string[]) => {
        setIsDeleting(true);
        setDeleteError(null);
        try {
            await Promise.all(ids.map((id) => deletePlanFn({ data: id })));
            setActivePlans((prev) => {
                const next = { ...prev };
                for (const id of ids) {
                    delete next[id];
                }
                return next;
            });
            queryClient.invalidateQueries({ queryKey: ["user", "plans"] });
            setDeleteTarget(null);
        } catch (err) {
            console.error(err);
            setDeleteError(t("planner.deleteFailed"));
        } finally {
            setIsDeleting(false);
        }
    };

    // From the unfiltered list, so narrowing the selection never hides a completed plan.
    const openCompletedReview = () => {
        setCompletedError(null);
        setReviewPlans(plans.filter((p) => p.met));
    };

    const handleDeleteCompleted = async (ids: string[]) => {
        setIsDeletingCompleted(true);
        setCompletedError(null);
        try {
            await deletePlansFn({ data: ids });
            setActivePlans((prev) => {
                const next = { ...prev };
                for (const id of ids) {
                    delete next[id];
                }
                return next;
            });
            queryClient.invalidateQueries({ queryKey: PLANS_QUERY_PREFIX });
            setReviewPlans(null);
        } catch (err) {
            console.error(err);
            setCompletedError(t("planner.deleteFailed"));
        } finally {
            setIsDeletingCompleted(false);
        }
    };

    const handleTogglePin = async (name: string, pinned: boolean) => {
        try {
            await setGroupPinnedFn({ data: { name, pinned } });
            queryClient.invalidateQueries({ queryKey: PLANS_QUERY_PREFIX });
        } catch (err) {
            console.error(err);
        }
    };

    const handleEditPlan = (opId: string) => {
        setEditOperatorId(opId);
        setOpen(true);
    };

    /** The chip row and the Groups tab's checkboxes are one control: both toggle a key of the group filter. */
    const toggleGroupFilter = (key: string) => applyGroupFilter(toggleGroupFilterKey(groupFilter, key));

    const handleRenameGroup = async (oldName: string) => {
        const newName = window.prompt(t("planner.group.renamePrompt"), oldName);
        if (newName === null) return;
        const trimmed = newName.trim();
        if (!trimmed || trimmed === oldName) return;
        try {
            await upsertGroupFn({ data: { oldName, name: trimmed } });
            setGroupFilter((prev) => renameGroupFilterKey(prev, oldName, trimmed));
            queryClient.invalidateQueries({ queryKey: ["user", "plans"] });
        } catch (err) {
            console.error(err);
        }
    };

    const handleDeleteGroup = async (name: string) => {
        if (!window.confirm(t("planner.group.deleteConfirm", { name }))) return;
        try {
            await deleteGroupFn({ data: { name } });
            const nextFilter = renameGroupFilterKey(groupFilter, name, null);
            if (nextFilter !== groupFilter) applyGroupFilter(nextFilter);
            queryClient.invalidateQueries({ queryKey: ["user", "plans"] });
        } catch (err) {
            console.error(err);
        }
    };

    const togglePlan = (plan: IOperatorPlanResponse) => {
        setActivePlans((prev) => ({
            ...prev,
            [plan.operator_id]: !isPlanActive(plan, prev, groupFilter),
        }));
    };

    const togglePlanExpanded = (planId: string) => {
        setExpandedPlans((prev) => ({
            ...prev,
            [planId]: !prev[planId],
        }));
    };

    return (
        <div className="page-shell [--page-max:1400px]">
            <PageHeader
                breadcrumbLabel="breadcrumb"
                breadcrumb={[t("planner.breadcrumb.tools"), t("planner.title")]}
                title={t("planner.title")}
                description={t("planner.intro")}
                actions={
                    isAuthenticated && plans.length > 0 ? (
                        <div className="flex flex-wrap items-center justify-end gap-2">
                            <Button size="sm" variant="outline" onClick={openCompletedReview}>
                                <CircleCheck className="mr-1.5 size-4" />
                                {t("planner.completed.check")}
                            </Button>
                            <BulkPlanButton />
                            <Button onClick={() => setOpen(true)} size="sm">
                                <Plus className="mr-1.5 size-4" />
                                {t("planner.createPlan")}
                            </Button>
                        </div>
                    ) : null
                }
            />

            {!isAuthenticated ? (
                <UnauthenticatedState />
            ) : !isPlansListLoading && plans.length === 0 ? (
                <div className="mt-16 flex flex-col items-center justify-center gap-4 py-20 text-center sm:mt-24 sm:py-28">
                    <Button size="xl" className="shadow-lg" onClick={() => setOpen(true)}>
                        <Plus className="size-5" />
                        {t("planner.createFirstPlan")}
                    </Button>
                    <BulkPlanButton size="xl" variant="outline" />
                </div>
            ) : (
                <div className="mt-8 flex flex-col gap-6 min-[720px]:flex-row">
                    <div className="w-full shrink-0 min-[720px]:w-90">
                        <div className="rounded-xl border border-border bg-card p-4">
                            <Tabs defaultValue="plans">
                                <TabsList className="w-full">
                                    <TabsTrigger value="plans" className="flex-1">
                                        {t("planner.tab.plans")}
                                    </TabsTrigger>
                                    <TabsTrigger value="groups" className="flex-1">
                                        {t("planner.tab.groups")}
                                    </TabsTrigger>
                                </TabsList>

                                <TabsContent value="plans" className="mt-4">
                                    {isPlansListLoading ? (
                                        <div className="flex flex-col gap-4">
                                            {["sk-1", "sk-2", "sk-3"].map((key) => (
                                                <div key={key} className="flex items-center gap-3 rounded-xl border border-border/40 p-4">
                                                    <Skeleton className="size-4 rounded" />
                                                    <Skeleton className="size-10 rounded-xl" />
                                                    <div className="flex flex-1 flex-col gap-1.5">
                                                        <Skeleton className="h-3.5 w-24 rounded" />
                                                        <Skeleton className="h-3 w-16 rounded" />
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                    ) : (
                                        <div className="flex flex-col gap-4">
                                            {groups.length > 0 && (
                                                // biome-ignore lint/a11y/useSemanticElements: a labelled row of toggle chips, not a form fieldset
                                                <div role="group" aria-label={t("planner.filter.aria")} className="flex flex-wrap items-center gap-2">
                                                    <FilterChip label={t("planner.filter.all")} active={groupFilter.size === 0} count={plans.length} onSelect={() => applyGroupFilter(new Set())} />
                                                    {groups.map((g, i) => (
                                                        <React.Fragment key={g.name}>
                                                            {showPinDivider && i === pinnedGroupCount && <span aria-hidden="true" className="h-5 w-px bg-border" />}
                                                            <FilterChip label={g.name} active={groupFilter.has(g.name)} count={filterCounts.get(g.name) ?? 0} onSelect={() => toggleGroupFilter(g.name)} />
                                                        </React.Fragment>
                                                    ))}
                                                    {(ungroupedCount > 0 || groupFilter.has(UNGROUPED_FILTER_KEY)) && <FilterChip label={t("planner.filter.ungrouped")} active={groupFilter.has(UNGROUPED_FILTER_KEY)} count={ungroupedCount} onSelect={() => toggleGroupFilter(UNGROUPED_FILTER_KEY)} />}
                                                </div>
                                            )}
                                            <div className="flex items-center justify-between border-border/40 border-b pb-3">
                                                {/* biome-ignore lint/a11y/noLabelWithoutControl: Checkbox component internally renders the input control */}
                                                <label className="flex cursor-pointer items-center gap-2 font-medium text-muted-foreground text-xs hover:text-foreground">
                                                    <Checkbox checked={allSelected} onCheckedChange={toggleSelectAll} />
                                                    <span>{allSelected ? t("planner.unselectAll") : t("planner.selectAll")}</span>
                                                </label>
                                                {selectedPlansCount > 0 && (
                                                    <Button
                                                        variant="ghost"
                                                        size="xs"
                                                        className="fade-in zoom-in-95 h-7 animate-in cursor-pointer border border-red-500/20 bg-red-500/5 text-red-600 duration-150 hover:border-red-500/40 hover:bg-red-500/15 hover:text-red-600 dark:text-red-400 dark:hover:text-red-400"
                                                        onClick={requestDeleteSelected}
                                                    >
                                                        <Trash className="mr-1 size-3.5" />
                                                        {t("planner.deleteSelected", { count: selectedPlansCount })}
                                                    </Button>
                                                )}
                                            </div>
                                            {plans.map((p) => {
                                                const op = p.operator;
                                                if (!op) return null;
                                                const rosterEntry = roster?.find((re) => re.operator_id === p.operator_id);
                                                const planActive = isActive(p);

                                                return (
                                                    /* biome-ignore lint/a11y/noLabelWithoutControl: PlanCardHeader renders the plan's Checkbox inside this label */
                                                    <label key={p.id} className={cn("relative flex cursor-pointer flex-col gap-4 rounded-xl border p-4 transition-all hover:shadow-md", planActive ? "border-primary bg-primary/5 shadow-sm ring-2 ring-primary/20" : "border-border/40 bg-muted/20 opacity-60")}>
                                                        <PlanCardHeader op={op} isActive={planActive} onToggleActive={() => togglePlan(p)} isExpanded={!!expandedPlans[p.id]} onToggleExpanded={() => togglePlanExpanded(p.id)} />

                                                        {expandedPlans[p.id] && <PlanDetails plan={p} op={op} rosterEntry={rosterEntry} className="border-border/40 border-t pt-3" />}

                                                        <PlanActions onEdit={() => handleEditPlan(p.operator_id)} onDelete={() => requestDeletePlan(p)} className="mt-auto border-border/40 border-t pt-3" />
                                                    </label>
                                                );
                                            })}
                                        </div>
                                    )}
                                </TabsContent>

                                <TabsContent value="groups" className="mt-4">
                                    {isPlansListLoading ? (
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
                                    ) : !plannerData?.groups || plannerData.groups.length === 0 ? (
                                        <p className="py-4 text-center text-muted-foreground text-xs">{t("planner.group.none")}</p>
                                    ) : (
                                        <div className="flex flex-col gap-6">
                                            {plannerData.groups.map((g, i) => {
                                                const plansInGroup = plans.filter((p) => p.groups?.includes(g.name));
                                                const isGroupSelected = groupFilter.has(g.name);
                                                const isExpanded = expandedGroups[g.name] ?? true;

                                                return (
                                                    <React.Fragment key={g.name}>
                                                        {showPinDivider && i === pinnedGroupCount && <hr className="-my-3 border-border/60 border-dashed" />}
                                                        {/* biome-ignore lint/a11y/useSemanticElements: custom interactive group wrapper */}
                                                        <div
                                                            role="button"
                                                            tabIndex={0}
                                                            onClick={(e) => {
                                                                if (isInteractiveGroupChild(e.target as HTMLElement)) return;
                                                                toggleGroupFilter(g.name);
                                                            }}
                                                            onKeyDown={(e) => {
                                                                if (e.key !== "Enter" && e.key !== " ") return;
                                                                if (isInteractiveGroupChild(e.target as HTMLElement)) return;
                                                                e.preventDefault();
                                                                toggleGroupFilter(g.name);
                                                            }}
                                                            className={cn(
                                                                "relative flex cursor-pointer flex-col rounded-xl border p-4 transition-all hover:shadow-md focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary/50",
                                                                isExpanded ? "gap-4" : "gap-0",
                                                                isGroupSelected ? "border-primary bg-primary/5 shadow-sm ring-2 ring-primary/20" : groupFilter.size > 0 ? "border-border/40 bg-card opacity-60 hover:opacity-100" : "border-border/40 bg-card",
                                                            )}
                                                        >
                                                            <div className={cn("flex items-center justify-between", isExpanded && "border-border/40 border-b pb-3")}>
                                                                <div className="flex items-center gap-3">
                                                                    <Checkbox checked={isGroupSelected} onCheckedChange={() => toggleGroupFilter(g.name)} />
                                                                    <span className="font-semibold text-foreground text-sm">{g.name}</span>
                                                                </div>
                                                                <div className="flex items-center gap-1">
                                                                    <button
                                                                        type="button"
                                                                        aria-label={g.pinned ? t("planner.group.unpin", { name: g.name }) : t("planner.group.pin", { name: g.name })}
                                                                        aria-pressed={g.pinned}
                                                                        title={g.pinned ? t("planner.group.unpin", { name: g.name }) : t("planner.group.pin", { name: g.name })}
                                                                        onClick={() => handleTogglePin(g.name, !g.pinned)}
                                                                        className={cn(
                                                                            "flex size-7 items-center justify-center rounded-md border transition-all",
                                                                            g.pinned ? "border-primary/40 bg-primary/10 text-primary hover:bg-primary/15" : "border-border bg-muted/40 text-muted-foreground hover:border-border/80 hover:bg-muted hover:text-foreground",
                                                                        )}
                                                                    >
                                                                        <Pin className={cn("size-3.5", g.pinned && "fill-current")} />
                                                                    </button>
                                                                    <button
                                                                        type="button"
                                                                        onClick={() => {
                                                                            setExpandedGroups((prev) => ({
                                                                                ...prev,
                                                                                [g.name]: !isExpanded,
                                                                            }));
                                                                        }}
                                                                        className="flex size-7 items-center justify-center rounded-md border border-border bg-muted/40 text-foreground transition-all hover:border-border/80 hover:bg-muted"
                                                                    >
                                                                        <ChevronDown className={cn("size-3.5 transition-transform", isExpanded && "rotate-180")} />
                                                                    </button>
                                                                    <button type="button" onClick={() => handleRenameGroup(g.name)} className="flex size-7 items-center justify-center rounded-md border border-border bg-muted/40 text-foreground transition-all hover:border-border/80 hover:bg-muted">
                                                                        <Pencil className="size-3.5" />
                                                                    </button>
                                                                    <button
                                                                        type="button"
                                                                        onClick={() => handleDeleteGroup(g.name)}
                                                                        className="flex size-7 items-center justify-center rounded-md border border-red-500/20 bg-red-500/5 text-red-600 transition-all hover:border-red-500/40 hover:bg-red-500/15 dark:text-red-400"
                                                                    >
                                                                        <Trash className="size-3.5" />
                                                                    </button>
                                                                </div>
                                                            </div>

                                                            {isExpanded && (
                                                                <div className="group-plans-list mt-3 divide-y divide-border/30">
                                                                    {plansInGroup.length === 0 ? (
                                                                        <p className="py-4 text-center text-muted-foreground text-xs">{t("planner.group.empty")}</p>
                                                                    ) : (
                                                                        plansInGroup.map((p) => {
                                                                            const op = p.operator;
                                                                            if (!op) return null;
                                                                            const rosterEntry = roster?.find((re) => re.operator_id === p.operator_id);
                                                                            const planActive = isActive(p);

                                                                            return (
                                                                                <div key={p.id} className="flex flex-col gap-3 py-4 first:pt-1 last:pb-1">
                                                                                    <PlanCardHeader op={op} isActive={planActive} onToggleActive={() => togglePlan(p)} isExpanded={!!expandedPlans[p.id]} onToggleExpanded={() => togglePlanExpanded(p.id)} />

                                                                                    {expandedPlans[p.id] && <PlanDetails plan={p} op={op} rosterEntry={rosterEntry} className="pl-7" />}

                                                                                    <PlanActions onEdit={() => handleEditPlan(p.operator_id)} onDelete={() => requestDeletePlan(p)} className="pl-7" dense />
                                                                                </div>
                                                                            );
                                                                        })
                                                                    )}
                                                                </div>
                                                            )}
                                                        </div>
                                                    </React.Fragment>
                                                );
                                            })}
                                        </div>
                                    )}
                                </TabsContent>
                            </Tabs>
                        </div>
                    </div>
                    <div className="flex-1">
                        <RequirementsPanel aggregatedRequirements={aggregatedRequirements} isLoading={isRequirementsLoading} activePlans={activePlansList} maxTier={maxTier} onMaxTierChange={setMaxTier} lastSyncedAt={plannerData?.lastSyncedAt ?? initialPlannerData?.lastSyncedAt ?? null} />
                    </div>
                </div>
            )}

            <DeletePlansDialog
                target={deleteTarget}
                onOpenChange={(isOpen) => {
                    if (!isOpen && !isDeleting) setDeleteTarget(null);
                }}
                onConfirm={handleConfirmDelete}
                isSubmitting={isDeleting}
                errorMessage={deleteError}
            />

            <CompletedPlansDialog
                plans={reviewPlans}
                onOpenChange={(isOpen) => {
                    if (!isOpen && !isDeletingCompleted) setReviewPlans(null);
                }}
                onDelete={handleDeleteCompleted}
                onKeep={() => setReviewPlans(null)}
                isSubmitting={isDeletingCompleted}
                errorMessage={completedError}
            />

            {isAuthenticated && (
                <OperatorPlannerDialog
                    open={open}
                    onOpenChange={(isOpen) => {
                        setOpen(isOpen);
                        if (!isOpen) {
                            setEditOperatorId(null);
                        }
                    }}
                    initialOperatorId={editOperatorId ?? undefined}
                />
            )}
        </div>
    );
}
