import { useQuery, useQueryClient } from "@tanstack/react-query";
import * as React from "react";

import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Switch } from "#/components/ui/switch";
import { useAuth } from "#/hooks/use-auth";
import { useOperatorName } from "#/hooks/use-operator-name";
import { operatorQueryOptions } from "#/lib/api/operators";
import { plansQueryOptions, upsertPlanFn } from "#/lib/api/planner";
import { userRosterQueryOptions } from "#/lib/api/user";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { formatSubProfession, rarityToNumber } from "#/lib/utils";
import type { messages } from "./OperatorPlannerDialog.messages";
import { OperatorSelector, useOperatorOptions } from "./OperatorSelector";
import { PlanGroupsField } from "./PlanGroupsField";
import { ModuleTargetsSection, PromotionLevelPanel, SkillTargetsSection } from "./PlanTargetSections";
import { PLANS_QUERY_PREFIX, plannableModules, planTargetPayload, UNPLANNABLE_OPERATOR_ID } from "./planTargets";
import { usePlanTargets } from "./usePlanTargets";

interface IOperatorPlannerDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    initialOperatorId?: string;
}

/** Modal dialog for creating or editing one operator's plan targets. */
export function OperatorPlannerDialog({ open, onOpenChange, initialOperatorId }: IOperatorPlannerDialogProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    const operatorName = useOperatorName();
    const server = useGamedataServer();
    const { user } = useAuth();
    const queryClient = useQueryClient();

    const [selectedOperatorId, setSelectedOperatorId] = React.useState<string | null>(null);
    const [searchQuery, setSearchQuery] = React.useState("");
    const [skillsOpen, setSkillsOpen] = React.useState<boolean>(true);
    const [modulesOpen, setModulesOpen] = React.useState<boolean>(true);
    const [isSaving, setIsSaving] = React.useState<boolean>(false);

    const { data: selectedOperator = null, isLoading: isOperatorDetailLoading } = useQuery({
        ...operatorQueryOptions(selectedOperatorId ?? "", server),
        enabled: !!selectedOperatorId,
    });
    const { options, selectedOption, isLoading: isOptionsLoading } = useOperatorOptions(server, searchQuery, selectedOperatorId);

    const { data: roster = [], isSuccess: isRosterLoaded } = useQuery({
        ...userRosterQueryOptions(user?.uid ?? ""),
        enabled: !!user?.uid,
    });
    const { data: plannerData } = useQuery({
        ...plansQueryOptions(),
        enabled: !!user?.uid,
    });
    const groupNames = React.useMemo(() => (plannerData?.groups ?? []).map((g) => g.name), [plannerData?.groups]);

    const existingPlan = (plannerData?.plans ?? []).find((p) => p.operator_id === selectedOperator?.id);
    const isEditMode = !!existingPlan;
    const isUnplannable = selectedOperator?.id === UNPLANNABLE_OPERATOR_ID;

    React.useEffect(() => {
        if (open) return;
        setSelectedOperatorId(null);
        setSkillsOpen(true);
        setModulesOpen(true);
        setSearchQuery("");
        setIsSaving(false);
    }, [open]);

    React.useEffect(() => {
        if (open && initialOperatorId) setSelectedOperatorId(initialOperatorId);
    }, [open, initialOperatorId]);

    // Declared after the two effects above so its own reset and seeding effects run after them, as they did inline.
    const targets = usePlanTargets({
        open,
        operator: selectedOperator,
        existingPlan,
        roster,
        hasRosterData: !user?.uid || isRosterLoaded,
    });

    const handleSave = async () => {
        if (!selectedOperator) return;
        setIsSaving(true);
        try {
            await upsertPlanFn({
                data: {
                    operatorId: selectedOperator.id ?? "",
                    targetElite: targets.elite,
                    targetLevel: targets.level,
                    ...planTargetPayload(targets.skillTargets, targets.moduleTargets),
                    displayOnProfile: targets.displayOnProfile,
                    groups: targets.selectedGroups,
                },
            });
            queryClient.invalidateQueries({ queryKey: PLANS_QUERY_PREFIX });
            onOpenChange(false);
        } catch (err) {
            console.error(err);
        } finally {
            setIsSaving(false);
        }
    };

    const selectedName = selectedOperator ? operatorName(selectedOperator) : null;
    const saveLabel = isSaving ? t("planner.dialog.saving") : isEditMode ? t("planner.dialog.save") : t("planner.dialog.create");

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup bottomStickOnMobile={false} className="flex h-[min(840px,calc(100vh-4rem))] w-full max-w-[min(1152px,calc(100vw-2rem))] flex-col overflow-hidden p-0">
                <DialogHeader>
                    <DialogTitle>{isEditMode ? t("planner.dialog.editTitle") : t("planner.dialog.createTitle")}</DialogTitle>
                    <DialogDescription>{isEditMode ? t("planner.dialog.editDesc", { operator: selectedName ?? t("planner.dialog.editDesc.fallback") }) : t("planner.dialog.createDesc")}</DialogDescription>
                </DialogHeader>

                <DialogPanel className="min-h-0 flex-1">
                    <div className="space-y-6">
                        <OperatorSelector options={options} selectedOption={selectedOption} isLoading={isOptionsLoading} onSelect={setSelectedOperatorId} onSearchQueryChange={setSearchQuery} />

                        {selectedOperatorId && isOperatorDetailLoading && (
                            <div className="flex flex-col items-center justify-center gap-3 py-16">
                                <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                                <p className="text-muted-foreground text-xs">{t("planner.dialog.loadingDetails")}</p>
                            </div>
                        )}

                        {selectedOperator && !isOperatorDetailLoading ? (
                            <div className="space-y-6">
                                <div className="rounded-xl border border-border bg-card p-4">
                                    <div className="flex items-center gap-4">
                                        <span aria-hidden="true" className="relative flex size-14 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-border bg-muted/70">
                                            <OperatorAvatar charId={selectedOperator.id} name={operatorName(selectedOperator)} className="block h-full w-full object-cover" server={selectedOperator.server} />
                                        </span>
                                        <div>
                                            <h3 className="font-bold text-foreground text-lg">{operatorName(selectedOperator)}</h3>
                                            <p className="text-muted-foreground text-xs">{t("planner.dialog.rarityArchetype", { rarity: rarityToNumber(selectedOperator.rarity), archetype: formatSubProfession(selectedOperator.subProfessionId) })}</p>
                                        </div>
                                    </div>
                                </div>

                                {isUnplannable ? (
                                    <div className="rounded-xl border border-amber-500/20 bg-amber-500/5 p-4 text-amber-600 text-sm leading-relaxed dark:text-amber-400">{t("planner.dialog.radian")}</div>
                                ) : (
                                    <>
                                        <PromotionLevelPanel targets={targets} />

                                        {selectedOperator.skills.length > 0 && <SkillTargetsSection operator={selectedOperator} targets={targets} open={skillsOpen} onOpenChange={setSkillsOpen} />}

                                        {plannableModules(selectedOperator).length > 0 && <ModuleTargetsSection operator={selectedOperator} targets={targets} open={modulesOpen} onOpenChange={setModulesOpen} />}

                                        <PlanGroupsField groupNames={groupNames} selectedGroups={targets.selectedGroups} onSelectedGroupsChange={targets.setSelectedGroups} />

                                        <div className="flex items-center justify-between rounded-xl border border-border bg-card/40 p-4">
                                            <div className="space-y-0.5">
                                                <label htmlFor="display-on-profile" className="cursor-pointer font-semibold text-foreground text-sm">
                                                    {t("planner.dialog.displayOnProfile")}
                                                </label>
                                                <p className="text-muted-foreground text-xs">{t("planner.dialog.displayOnProfile.desc")}</p>
                                            </div>
                                            <Switch id="display-on-profile" checked={targets.displayOnProfile} onCheckedChange={targets.setDisplayOnProfile} />
                                        </div>
                                    </>
                                )}
                            </div>
                        ) : !selectedOperatorId ? (
                            <p className="text-muted-foreground text-sm italic">{t("planner.dialog.pickOperator")}</p>
                        ) : null}
                    </div>
                </DialogPanel>

                <DialogFooter className="pb-8 sm:pb-8">
                    <DialogClose render={<Button variant="outline" className="w-full sm:w-auto" />}>{t("planner.dialog.cancel")}</DialogClose>
                    <Button onClick={handleSave} disabled={isSaving || !selectedOperator || isUnplannable} className="w-full sm:w-auto">
                        {saveLabel}
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}
