import { useQuery, useQueryClient } from "@tanstack/react-query";
import { CircleCheck, Lock, Plus } from "lucide-react";
import * as React from "react";

import { Button } from "#/components/ui/button";
import { PageHeader } from "#/components/ui/page-header";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "#/components/ui/tabs";
import { useAuth } from "#/hooks/use-auth";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { useOperatorName } from "#/hooks/use-operator-name";
import { deleteGroupFn, deletePlansFn, type IOperatorPlanResponse, PLANS_QUERY_PREFIX, plansQueryOptions, setGroupPinnedFn, upsertGroupFn } from "#/lib/api/planner";
import { userRosterQueryOptions } from "#/lib/api/user";
import { authActions } from "#/lib/auth/store";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { BulkPlanButton } from "./BulkPlanDialog";
import { CompletedPlansDialog } from "./CompletedPlansDialog";
import { DeletePlansDialog, type IDeletePlansTarget } from "./DeletePlansDialog";
import { GroupList } from "./GroupList";
import type { messages } from "./OperatorPlanner.messages";
import { OperatorPlannerDialog } from "./OperatorPlannerDialog";
import type { IPlanEntryContext } from "./PlanEntry";
import { PlanList } from "./PlanList";
import { activePlanIds } from "./planFilters";
import { RequirementsPanel } from "./RequirementsPanel";
import { MAX_TIER_STORAGE_KEY, type MaxTierFilter, parseMaxTier } from "./requirements";
import { usePlannerSelection } from "./usePlannerSelection";

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

export function OperatorPlanner(): React.ReactElement {
    const operatorName = useOperatorName();
    const t: PlannerT = useT("tools");
    const { isAuthenticated, user } = useAuth();
    const queryClient = useQueryClient();
    const selection = usePlannerSelection();
    const [open, setOpen] = React.useState(false);
    const [expandedGroups, setExpandedGroups] = React.useState<Record<string, boolean>>({});
    const [expandedPlans, setExpandedPlans] = React.useState<Record<string, boolean>>({});
    const [editOperatorId, setEditOperatorId] = React.useState<string | null>(null);
    const [deleteTarget, setDeleteTarget] = React.useState<IDeletePlansTarget | null>(null);
    const [isDeleting, setIsDeleting] = React.useState(false);
    const [deleteError, setDeleteError] = React.useState<string | null>(null);
    const [reviewPlans, setReviewPlans] = React.useState<IOperatorPlanResponse[] | null>(null);
    const [isDeletingCompleted, setIsDeletingCompleted] = React.useState(false);
    const [completedError, setCompletedError] = React.useState<string | null>(null);
    const [maxTier, setMaxTier] = useLocalStorageState<MaxTierFilter>(MAX_TIER_STORAGE_KEY, 0, { parse: parseMaxTier, serialize: String });

    // The unfiltered list decides which ids the filtered requirements query asks for.
    const { data: initialPlannerData, isLoading: initialPlansLoading } = useQuery({
        ...plansQueryOptions(),
        enabled: isAuthenticated,
    });

    const activeIds = React.useMemo(() => (initialPlannerData?.plans ? activePlanIds(initialPlannerData.plans, selection.activePlans, selection.groupFilter) : undefined), [initialPlannerData?.plans, selection.activePlans, selection.groupFilter]);

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
    // Groups never depend on the selection, so the unfiltered query keeps them steady while the filtered one refetches.
    const groups = initialPlannerData?.groups ?? [];
    const activePlansList = plans.filter(selection.isActive);

    const pinnedGroupCount = groups.filter((g) => g.pinned).length;
    const pinDividerAt = pinnedGroupCount > 0 && pinnedGroupCount < groups.length ? pinnedGroupCount : -1;

    const invalidatePlans = () => queryClient.invalidateQueries({ queryKey: PLANS_QUERY_PREFIX });
    const planName = (p: IOperatorPlanResponse) => (p.operator ? operatorName(p.operator) : t("planner.unknownOperator"));

    const requestDelete = (targets: IOperatorPlanResponse[]) => {
        setDeleteError(null);
        setDeleteTarget({ ids: targets.map((p) => p.operator_id), names: targets.map(planName) });
    };

    const requestDeleteSelected = () => {
        if (activePlansList.length === 0) return;
        requestDelete(activePlansList);
    };

    const handleConfirmDelete = async (ids: string[]) => {
        setIsDeleting(true);
        setDeleteError(null);
        try {
            // One statement, so a failure deletes nothing.
            await deletePlansFn({ data: ids });
            selection.dropPlans(ids);
            setDeleteTarget(null);
        } catch (err) {
            console.error(err);
            setDeleteError(t("planner.deleteFailed"));
        } finally {
            invalidatePlans();
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
            selection.dropPlans(ids);
            invalidatePlans();
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
            invalidatePlans();
        } catch (err) {
            console.error(err);
        }
    };

    const handleRenameGroup = async (oldName: string) => {
        const newName = window.prompt(t("planner.group.renamePrompt"), oldName);
        if (newName === null) return;
        const trimmed = newName.trim();
        if (!trimmed || trimmed === oldName) return;
        try {
            await upsertGroupFn({ data: { oldName, name: trimmed } });
            selection.renameGroup(oldName, trimmed);
            invalidatePlans();
        } catch (err) {
            console.error(err);
        }
    };

    const handleDeleteGroup = async (name: string) => {
        if (!window.confirm(t("planner.group.deleteConfirm", { name }))) return;
        try {
            await deleteGroupFn({ data: { name } });
            selection.dropGroup(plans, name);
            invalidatePlans();
        } catch (err) {
            console.error(err);
        }
    };

    const entryContext: IPlanEntryContext = {
        roster,
        isActive: selection.isActive,
        isExpanded: (p) => !!expandedPlans[p.id],
        onToggleActive: selection.togglePlan,
        onToggleExpanded: (p) => setExpandedPlans((prev) => ({ ...prev, [p.id]: !prev[p.id] })),
        onEdit: (p) => {
            setEditOperatorId(p.operator_id);
            setOpen(true);
        },
        onDelete: (p) => requestDelete([p]),
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
                                    <PlanList plans={plans} groups={groups} pinDividerAt={pinDividerAt} isLoading={isPlansListLoading} selection={selection} entryContext={entryContext} onDeleteSelected={requestDeleteSelected} />
                                </TabsContent>

                                <TabsContent value="groups" className="mt-4">
                                    <GroupList
                                        plans={plans}
                                        groups={groups}
                                        pinDividerAt={pinDividerAt}
                                        isLoading={isPlansListLoading}
                                        selection={selection}
                                        entryContext={entryContext}
                                        expandedGroups={expandedGroups}
                                        onExpandedGroupsChange={setExpandedGroups}
                                        onTogglePin={handleTogglePin}
                                        onRename={handleRenameGroup}
                                        onDelete={handleDeleteGroup}
                                    />
                                </TabsContent>
                            </Tabs>
                        </div>
                    </div>
                    <div className="flex-1">
                        <RequirementsPanel aggregatedRequirements={plannerData?.aggregatedRequirements ?? []} isLoading={isRequirementsLoading} activePlans={activePlansList} maxTier={maxTier} onMaxTierChange={setMaxTier} lastSyncedAt={plannerData?.lastSyncedAt ?? initialPlannerData?.lastSyncedAt ?? null} />
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
                        if (!isOpen) setEditOperatorId(null);
                    }}
                    initialOperatorId={editOperatorId ?? undefined}
                />
            )}
        </div>
    );
}
