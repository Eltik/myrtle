import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ChevronDown, Pencil, Plus, Trash } from "lucide-react";
import * as React from "react";

import { eliteIcon, moduleIconURL, skillIconURL, specializedIcon } from "#/components/operators/detail/impl/assets";
import { getSkillTypeLabel, getSpTypeLabel } from "#/components/operators/detail/impl/helpers";
import { Button } from "#/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Combobox, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxPrimitive } from "#/components/ui/combobox";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Input } from "#/components/ui/input";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Slider } from "#/components/ui/slider";
import { Switch } from "#/components/ui/switch";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useAuth } from "#/hooks/use-auth";
import { operatorQueryOptions, operatorsListQueryOptions } from "#/lib/api/operators";
import { deleteGroupFn, plansQueryOptions, upsertGroupFn, upsertPlanFn } from "#/lib/api/planner";
import { upcomingQueryOptions } from "#/lib/api/upcoming";
import { userRosterQueryOptions } from "#/lib/api/user";
import { professionLabel } from "#/lib/registry/operator-display";
import { searchAndRank } from "#/lib/search/fuzzy";
import { cn, formatSubProfession, rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem, OperatorProfession } from "#/types/operators";

function getMaxLevel(rarity: number, elite: number): number {
    if (rarity <= 2) return 30;
    if (rarity === 3) return elite === 0 ? 40 : 55;
    if (rarity === 4) return elite === 0 ? 45 : elite === 1 ? 60 : 70;
    if (rarity === 5) return elite === 0 ? 50 : elite === 1 ? 70 : 80;
    return elite === 0 ? 50 : elite === 1 ? 80 : 90;
}

function isSkillLevelAllowed(operator: IOperatorListItem, skillIdx: number, targetLevel: number, currentElite: number, currentLevel: number): boolean {
    if (targetLevel <= 1) return true;

    const phaseMap = ["PHASE_0", "PHASE_1", "PHASE_2"];
    const currentPhaseStr = phaseMap[currentElite] || "PHASE_0";

    const isPhaseAndLevelAllowed = (reqPhase: string, reqLevel: number) => {
        const reqElite = phaseMap.indexOf(reqPhase);
        const currEliteVal = phaseMap.indexOf(currentPhaseStr);
        if (currEliteVal < reqElite) return false;
        if (currEliteVal === reqElite && currentLevel < reqLevel) return false;
        return true;
    };

    if (targetLevel <= 7) {
        const upgradeIndex = targetLevel - 2;
        const upgrade = operator.allSkillLevelUp?.[upgradeIndex];
        if (!upgrade) return false;
        const cond = upgrade.unlockCond;
        if (!cond) return true;
        return isPhaseAndLevelAllowed(cond.phase, cond.level);
    }

    const masteryIndex = targetLevel - 8;
    const skill = operator.skills?.[skillIdx];
    if (!skill) return false;
    const upgrade = skill.levelUpCostCond?.[masteryIndex];
    if (!upgrade) return false;
    const cond = upgrade.unlockCond;
    if (!cond) return true;
    return isPhaseAndLevelAllowed(cond.phase, cond.level);
}

function isModuleAllowed(operator: IOperatorListItem, uniEquipId: string, currentElite: number, currentLevel: number): boolean {
    const mod = operator.modules?.find((m) => m.uniEquipId === uniEquipId);
    if (!mod) return false;

    const phaseMap = ["PHASE_0", "PHASE_1", "PHASE_2"];
    const currentPhaseStr = phaseMap[currentElite] || "PHASE_0";

    const reqPhase = mod.unlockEvolvePhase;
    const reqLevel = mod.unlockLevel;

    const reqElite = phaseMap.indexOf(reqPhase);
    const currEliteVal = phaseMap.indexOf(currentPhaseStr);
    if (currEliteVal < reqElite) return false;
    if (currEliteVal === reqElite && currentLevel < reqLevel) return false;
    return true;
}

interface IOperatorPlannerDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    initialOperatorId?: string;
}

/** Modal dialog for customizing operator targets. */
interface IOperatorSelectionItem {
    id: string;
    name: string;
    appellation: string;
    rarity: number;
    profession: OperatorProfession;
    subProfessionId: string;
    tagList?: string[];
    nationId: string;
    isUpcoming: boolean;
}

/** Modal dialog for customizing operator targets. */
export function OperatorPlannerDialog({ open, onOpenChange, initialOperatorId }: IOperatorPlannerDialogProps): React.ReactElement {
    const [selectedOperatorId, setSelectedOperatorId] = React.useState<string | null>(null);
    const { data: selectedOperator = null, isLoading: isOperatorDetailLoading } = useQuery({
        ...operatorQueryOptions(selectedOperatorId ?? ""),
        enabled: !!selectedOperatorId,
    });
    const [elite, setElite] = React.useState<number>(0);
    const [level, setLevel] = React.useState<number>(1);
    const [skillTargets, setSkillTargets] = React.useState<Record<number, number>>({});
    const [moduleTargets, setModuleTargets] = React.useState<Record<string, number>>({});
    const [displayOnProfile, setDisplayOnProfile] = React.useState<boolean>(false);
    const [skillsOpen, setSkillsOpen] = React.useState<boolean>(true);
    const [modulesSectionOpen, setModulesSectionOpen] = React.useState<boolean>(true);
    const [isSaving, setIsSaving] = React.useState<boolean>(false);
    const [selectedGroups, setSelectedGroups] = React.useState<string[]>([]);
    const [groupSearchQuery, setGroupSearchQuery] = React.useState<string>("");
    const [comboboxOpen, setComboboxOpen] = React.useState<boolean>(false);
    const [isCreatingGroup, setIsCreatingGroup] = React.useState<boolean>(false);
    const [newGroupName, setNewGroupName] = React.useState<string>("");

    const { data: operators = [], isLoading: isOperatorsLoading } = useQuery(operatorsListQueryOptions());
    const { data: upcoming = [], isLoading: isUpcomingLoading } = useQuery(upcomingQueryOptions());

    const isLoading = isOperatorsLoading || isUpcomingLoading;

    const { user } = useAuth();
    const queryClient = useQueryClient();
    const { data: roster = [], isSuccess: isRosterLoaded } = useQuery({
        ...userRosterQueryOptions(user?.uid ?? ""),
        enabled: !!user?.uid,
    });

    const { data: plannerData } = useQuery({
        ...plansQueryOptions(),
        enabled: !!user?.uid,
    });
    const plans = plannerData?.plans ?? [];

    const allGroupNames = React.useMemo(() => {
        return (plannerData?.groups ?? []).map((g) => g.name);
    }, [plannerData?.groups]);

    const filteredGroupNames = React.useMemo(() => {
        if (!groupSearchQuery.trim()) return allGroupNames;
        return allGroupNames.filter((name) => name.toLowerCase().includes(groupSearchQuery.toLowerCase()));
    }, [allGroupNames, groupSearchQuery]);

    const existingPlan = plans.find((p) => p.operator_id === selectedOperator?.id);
    const isEditMode = !!existingPlan;

    const lastInitializedStateRef = React.useRef<{
        operatorId: string;
        elite: number;
        level: number;
        isFullyApplied: boolean;
    } | null>(null);
    const hasRosterData = !user?.uid || isRosterLoaded;

    const [searchQuery, setSearchQuery] = React.useState("");

    const sortedOperators = React.useMemo(() => {
        const mappedUpcoming = upcoming
            .filter((op) => !op.isNotObtainable)
            .map((op) => ({
                id: op.id,
                name: op.name,
                appellation: op.appellation,
                rarity: op.rarity,
                profession: op.profession,
                subProfessionId: op.subProfessionId,
                tagList: op.tagList,
                nationId: op.nationId,
                isUpcoming: true,
            }));

        const mappedGlobal = operators
            .filter((op) => !op.isNotObtainable)
            .map((op) => ({
                id: op.id ?? "",
                name: op.name,
                appellation: op.appellation,
                rarity: rarityToNumber(op.rarity),
                profession: op.profession,
                subProfessionId: op.subProfessionId,
                tagList: op.tagList,
                nationId: op.nationId,
                isUpcoming: false,
            }));

        return [...mappedUpcoming, ...mappedGlobal].sort((a, b) => {
            if (a.isUpcoming !== b.isUpcoming) {
                return a.isUpcoming ? -1 : 1;
            }
            if (b.rarity !== a.rarity) return b.rarity - a.rarity;
            return a.name.localeCompare(b.name);
        });
    }, [operators, upcoming]);

    const filteredAndSortedOperators = React.useMemo(() => {
        if (!searchQuery.trim()) {
            return sortedOperators;
        }
        const results = searchAndRank(searchQuery, sortedOperators, (op) => ({
            name: op.name,
            extra: `${op.appellation} ${professionLabel(op.profession)} ${op.subProfessionId} ${op.rarity}★ ${(op.tagList ?? []).join(" ")} ${op.nationId}`,
        }));
        return results.map((r) => r.item);
    }, [searchQuery, sortedOperators]);

    const selectedItem = React.useMemo(() => {
        if (!selectedOperatorId) return null;
        return sortedOperators.find((item) => item.id === selectedOperatorId) || null;
    }, [selectedOperatorId, sortedOperators]);

    const opRarity = selectedOperator ? rarityToNumber(selectedOperator.rarity) : 6;
    const maxElite = opRarity <= 2 ? 0 : opRarity === 3 ? 1 : 2;
    const maxL = selectedOperator ? getMaxLevel(opRarity, elite) : 90;

    React.useEffect(() => {
        if (!open) {
            setSelectedOperatorId(null);
            setElite(0);
            setLevel(1);
            setSkillTargets({});
            setModuleTargets({});
            setDisplayOnProfile(false);
            setSkillsOpen(true);
            setModulesSectionOpen(true);
            setSearchQuery("");
            setIsSaving(false);
            lastInitializedStateRef.current = null;
            setSelectedGroups([]);
            setGroupSearchQuery("");
            setComboboxOpen(false);
            setIsCreatingGroup(false);
            setNewGroupName("");
        }
    }, [open]);

    React.useEffect(() => {
        if (open && initialOperatorId) {
            setSelectedOperatorId(initialOperatorId);
        }
    }, [open, initialOperatorId]);

    React.useEffect(() => {
        if (selectedOperator && hasRosterData) {
            const isInitialized = lastInitializedStateRef.current?.operatorId === selectedOperator.id;
            if (!isInitialized) {
                const r = rarityToNumber(selectedOperator.rarity);
                const maxE = r <= 2 ? 0 : r === 3 ? 1 : 2;
                const rosterEntry = roster?.find((re) => re.operator_id === selectedOperator.id);

                let initialElite = 0;
                let initialLevel = 1;
                const initialSkills: Record<number, number> = {};
                const initialModules: Record<string, number> = {};

                if (existingPlan) {
                    initialElite = Math.min(existingPlan.target_elite, maxE);
                    const maxLForElite = getMaxLevel(r, initialElite);
                    initialLevel = Math.min(existingPlan.target_level, maxLForElite);

                    selectedOperator.skills.forEach((_, idx) => {
                        if (existingPlan.target_skill_level < 7) {
                            initialSkills[idx] = existingPlan.target_skill_level;
                        } else {
                            const masteryEntry = existingPlan.target_skills?.find((s) => s.skill_index === idx);
                            initialSkills[idx] = masteryEntry && masteryEntry.mastery_level > 0 ? 7 + masteryEntry.mastery_level : 7;
                        }
                    });

                    selectedOperator.modules.forEach((m) => {
                        if (m.typeName1 !== "ORIGINAL") {
                            const targetMod = existingPlan.target_modules?.find((mod) => mod.module_id === m.uniEquipId);
                            initialModules[m.uniEquipId] = targetMod ? targetMod.module_stage : 0;
                        }
                    });

                    setDisplayOnProfile(existingPlan.display_on_profile);
                    setSelectedGroups(existingPlan.groups ?? []);
                } else if (rosterEntry) {
                    initialElite = Math.min(rosterEntry.elite, maxE);
                    const maxLForElite = getMaxLevel(r, initialElite);
                    initialLevel = Math.min(rosterEntry.level, maxLForElite);

                    selectedOperator.skills.forEach((_, idx) => {
                        if (rosterEntry.skill_level < 7) {
                            initialSkills[idx] = rosterEntry.skill_level;
                        } else {
                            const masteryEntry = rosterEntry.masteries?.find((m) => m.index === idx);
                            initialSkills[idx] = masteryEntry && masteryEntry.mastery > 0 ? 7 + masteryEntry.mastery : 7;
                        }
                    });

                    selectedOperator.modules.forEach((m) => {
                        if (m.typeName1 !== "ORIGINAL") {
                            const modEntry = rosterEntry.modules?.find((mod) => mod.id === m.uniEquipId);
                            initialModules[m.uniEquipId] = modEntry && !modEntry.locked ? modEntry.level : 0;
                        }
                    });

                    setDisplayOnProfile(false);
                    setSelectedGroups([]);
                } else {
                    initialElite = 0;
                    initialLevel = 1;

                    selectedOperator.skills.forEach((_, idx) => {
                        initialSkills[idx] = 1;
                    });

                    selectedOperator.modules.forEach((m) => {
                        if (m.typeName1 !== "ORIGINAL") {
                            initialModules[m.uniEquipId] = 0;
                        }
                    });

                    setDisplayOnProfile(false);
                    setSelectedGroups([]);
                }

                lastInitializedStateRef.current = {
                    operatorId: selectedOperator.id ?? "",
                    elite: initialElite,
                    level: initialLevel,
                    isFullyApplied: false,
                };

                setElite(initialElite);
                setLevel(initialLevel);
                setSkillTargets(initialSkills);
                setModuleTargets(initialModules);
            }
        }
    }, [selectedOperator, roster, hasRosterData, existingPlan]);

    const handleEliteChange = (newElite: number) => {
        setElite(newElite);
        const maxLForElite = getMaxLevel(opRarity, newElite);
        setLevel((prev) => Math.min(prev, maxLForElite));
    };

    const handleLevelChange = (val: number) => {
        const targetVal = Number.isNaN(val) ? 1 : val;
        const safeVal = Math.min(Math.max(1, targetVal), maxL);
        setLevel(safeVal);
    };

    const handleSkillTargetChange = (skillIdx: number, val: number) => {
        setSkillTargets((prev) => {
            const next = { ...prev };
            if (val < 7) {
                selectedOperator?.skills.forEach((_, idx) => {
                    next[idx] = val;
                });
            } else {
                next[skillIdx] = val;
                selectedOperator?.skills.forEach((_, idx) => {
                    if (idx !== skillIdx) {
                        const current = next[idx] ?? 1;
                        if (current < 7) {
                            next[idx] = 7;
                        }
                    }
                });
            }
            return next;
        });
    };

    const handleSave = async () => {
        if (!selectedOperator) return;
        setIsSaving(true);
        try {
            const targetSkillLevel = Object.values(skillTargets).length > 0 ? Math.min(7, Math.max(1, ...Object.values(skillTargets))) : 1;

            const targetSkills = Object.entries(skillTargets).map(([idxStr, val]) => ({
                skill_index: parseInt(idxStr, 10),
                mastery_level: val > 7 ? val - 7 : 0,
            }));

            const targetModules = Object.entries(moduleTargets).map(([moduleId, val]) => ({
                module_id: moduleId,
                module_stage: val,
            }));

            await upsertPlanFn({
                data: {
                    operatorId: selectedOperator.id ?? "",
                    targetElite: elite,
                    targetLevel: level,
                    targetSkillLevel,
                    targetSkills,
                    targetModules,
                    displayOnProfile,
                    groups: selectedGroups,
                },
            });
            queryClient.invalidateQueries({ queryKey: ["user", "plans"] });
            onOpenChange(false);
        } catch (err) {
            console.error(err);
        } finally {
            setIsSaving(false);
        }
    };

    const handleCreateGroup = async () => {
        if (!newGroupName.trim()) return;
        try {
            await upsertGroupFn({ data: { name: newGroupName.trim() } });
            setSelectedGroups((prev) => {
                if (prev.includes(newGroupName.trim())) return prev;
                return [...prev, newGroupName.trim()];
            });
            setNewGroupName("");
            setIsCreatingGroup(false);
            queryClient.invalidateQueries({ queryKey: ["user", "plans"] });
        } catch (err) {
            console.error(err);
        }
    };

    const handleRenameGroup = async (oldName: string) => {
        const newName = window.prompt("Enter new group name:", oldName);
        if (newName === null) return;
        const trimmed = newName.trim();
        if (!trimmed || trimmed === oldName) return;
        try {
            await upsertGroupFn({ data: { oldName, name: trimmed } });
            setSelectedGroups((prev) => prev.map((g) => (g === oldName ? trimmed : g)));
            queryClient.invalidateQueries({ queryKey: ["user", "plans"] });
        } catch (err) {
            console.error(err);
        }
    };

    const handleDeleteGroup = async (name: string) => {
        if (!window.confirm(`Are you sure you want to delete group "${name}"?`)) return;
        try {
            await deleteGroupFn({ data: { name } });
            setSelectedGroups((prev) => prev.filter((g) => g !== name));
            queryClient.invalidateQueries({ queryKey: ["user", "plans"] });
        } catch (err) {
            console.error(err);
        }
    };

    React.useEffect(() => {
        if (selectedOperator) {
            const initializedState = lastInitializedStateRef.current;
            if (!initializedState || initializedState.operatorId !== selectedOperator.id) {
                return;
            }

            if (!initializedState.isFullyApplied) {
                if (elite === initializedState.elite && level === initializedState.level) {
                    initializedState.isFullyApplied = true;
                } else {
                    return;
                }
            }

            setSkillTargets((prev) => {
                let changed = false;
                const next = { ...prev };
                selectedOperator.skills.forEach((_, idx) => {
                    const currentVal = next[idx] ?? 1;
                    let allowedVal = currentVal;
                    while (allowedVal > 1 && !isSkillLevelAllowed(selectedOperator, idx, allowedVal, elite, level)) {
                        allowedVal--;
                    }
                    if (allowedVal !== currentVal) {
                        next[idx] = allowedVal;
                        changed = true;
                    }
                });

                if (changed) {
                    let minBelow7 = 7;
                    selectedOperator.skills.forEach((_, sIdx) => {
                        const sVal = next[sIdx] ?? 1;
                        if (sVal < minBelow7) {
                            minBelow7 = sVal;
                        }
                    });
                    if (minBelow7 < 7) {
                        selectedOperator.skills.forEach((_, sIdx) => {
                            next[sIdx] = minBelow7;
                        });
                    }
                }
                return next;
            });

            setModuleTargets((prev) => {
                let changed = false;
                const next = { ...prev };
                selectedOperator.modules.forEach((mod) => {
                    if (mod.typeName1 !== "ORIGINAL") {
                        const currentStage = next[mod.uniEquipId] ?? 0;
                        if (currentStage > 0 && !isModuleAllowed(selectedOperator, mod.uniEquipId, elite, level)) {
                            next[mod.uniEquipId] = 0;
                            changed = true;
                        }
                    }
                });
                return changed ? next : prev;
            });
        }
    }, [elite, level, selectedOperator]);

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup bottomStickOnMobile={false} className="flex h-[min(840px,calc(100vh-4rem))] w-full max-w-[min(1152px,calc(100vw-2rem))] flex-col overflow-hidden p-0">
                <DialogHeader>
                    <DialogTitle>{isEditMode ? "Edit plan" : "Create new plan"}</DialogTitle>
                    <DialogDescription>{isEditMode ? `Customize and update your target goals for ${selectedOperator?.name ?? "this operator"}.` : "Add a new operator target plan to your planner list."}</DialogDescription>
                </DialogHeader>

                <DialogPanel className="min-h-0 flex-1">
                    <div className="space-y-6">
                        <div className="space-y-2">
                            <label className="block font-medium text-[13px] text-muted-foreground leading-none" htmlFor="operator-selector">
                                Select Operator
                            </label>
                            <Combobox<IOperatorSelectionItem, false>
                                items={filteredAndSortedOperators}
                                value={selectedItem}
                                onValueChange={(item) => setSelectedOperatorId(item?.id ?? null)}
                                filter={null}
                                onInputValueChange={setSearchQuery}
                                itemToStringLabel={(op) => op?.name ?? ""}
                                itemToStringValue={(op) => op?.id ?? ""}
                            >
                                <ComboboxInput id="operator-selector" placeholder={isLoading ? "Loading operators..." : "Search operators by name, class, or tag..."} />
                                <ComboboxPopup className="max-w-100">
                                    <ComboboxEmpty>No operators found.</ComboboxEmpty>
                                    <ComboboxList>
                                        {(op: IOperatorSelectionItem) => {
                                            const rarity = op.rarity;
                                            return (
                                                <ComboboxItem key={op.id} value={op}>
                                                    <span className="flex w-full items-center gap-3">
                                                        <span aria-hidden="true" className="relative flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-lg border border-border bg-muted/50">
                                                            <OperatorAvatar charId={op.id} name={op.name} className="block h-full w-full object-cover" server={op.isUpcoming ? "cn" : undefined} />
                                                        </span>
                                                        <span className="flex-1 font-medium text-foreground text-sm">{op.name}</span>
                                                        <span className="font-normal text-muted-foreground text-xs">
                                                            {rarity}★ · {professionLabel(op.profession)}
                                                        </span>
                                                    </span>
                                                </ComboboxItem>
                                            );
                                        }}
                                    </ComboboxList>
                                </ComboboxPopup>
                            </Combobox>
                        </div>

                        {selectedOperatorId && isOperatorDetailLoading && (
                            <div className="flex flex-col items-center justify-center gap-3 py-16">
                                <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                                <p className="text-muted-foreground text-xs">Loading operator details...</p>
                            </div>
                        )}

                        {selectedOperator && !isOperatorDetailLoading ? (
                            <div className="space-y-6">
                                <div className="rounded-xl border border-border bg-card p-4">
                                    <div className="flex items-center gap-4">
                                        <span aria-hidden="true" className="relative flex size-14 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-border bg-muted/70">
                                            <OperatorAvatar charId={selectedOperator.id} name={selectedOperator.name} className="block h-full w-full object-cover" server={selectedOperator.server} />
                                        </span>
                                        <div>
                                            <h3 className="font-bold text-foreground text-lg">{selectedOperator.name}</h3>
                                            <p className="text-muted-foreground text-xs">
                                                {rarityToNumber(selectedOperator.rarity)}★ · {formatSubProfession(selectedOperator.subProfessionId)}
                                            </p>
                                        </div>
                                    </div>
                                </div>

                                {selectedOperator.id === "char_4195_radian" ? (
                                    <div className="rounded-xl border border-amber-500/20 bg-amber-500/5 p-4 text-amber-600 text-sm leading-relaxed dark:text-amber-400">Raidian's upgrades are dependent on Integrated Strategies 6 - Sui's Garden of Grotesqueries progression and cannot be planned.</div>
                                ) : (
                                    <>
                                        <div className="grid grid-cols-1 items-center gap-6 rounded-xl border border-border bg-card/40 p-4 sm:grid-cols-[auto_1fr]">
                                            <div className="flex flex-col gap-2">
                                                <span className="font-semibold text-muted-foreground text-xs uppercase tracking-wider">Promotion</span>
                                                <div className="mt-1 flex items-center gap-2">
                                                    {[0, 1, 2].map((e) => {
                                                        const isAvailable = e <= maxElite;
                                                        const isActive = elite === e;
                                                        if (!isAvailable) return null;
                                                        return (
                                                            <button
                                                                key={e}
                                                                type="button"
                                                                onClick={() => handleEliteChange(e)}
                                                                className={cn(
                                                                    "relative flex size-12 cursor-pointer items-center justify-center rounded-lg border transition-all",
                                                                    isActive ? "border-primary bg-primary/10 opacity-100 ring-2 ring-primary/20" : "border-border bg-muted/40 opacity-40 hover:bg-muted/80 hover:opacity-75",
                                                                )}
                                                                title={`Elite ${e}`}
                                                            >
                                                                <img src={eliteIcon(e)} alt={`Elite ${e}`} className="icon-theme-aware size-7 object-contain" />
                                                            </button>
                                                        );
                                                    })}
                                                </div>
                                            </div>

                                            <div className="flex w-full flex-col gap-2">
                                                <span className="font-semibold text-muted-foreground text-xs uppercase tracking-wider">Level (Max {maxL})</span>
                                                <div className="mt-2 flex items-center gap-4">
                                                    <Slider value={[level]} onValueChange={(vals) => handleLevelChange(Array.isArray(vals) ? (vals[0] ?? 1) : vals)} min={1} max={maxL} className="flex-1" />
                                                    <Input type="number" min={1} max={maxL} value={level} onChange={(e) => handleLevelChange(parseInt(e.target.value, 10))} className="w-16 text-center font-mono" />
                                                </div>
                                            </div>
                                        </div>

                                        {selectedOperator.skills.length > 0 && (
                                            <Collapsible open={skillsOpen} onOpenChange={setSkillsOpen} className="space-y-3">
                                                <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                                                    <span className="flex items-center gap-2">
                                                        <span className="font-semibold text-foreground text-sm">Skills</span>
                                                    </span>
                                                    <ChevronDown className={cn("h-4 w-4 transition-transform", skillsOpen && "rotate-180")} />
                                                </CollapsibleTrigger>
                                                <CollapsibleContent>
                                                    <div className="grid grid-cols-1 gap-4 pt-1">
                                                        {selectedOperator.skills.map((skill, idx) => {
                                                            const currentTarget = skillTargets[idx] ?? 1;
                                                            const skillLevelInfo = skill.static?.levels?.[currentTarget - 1] ?? skill.static?.levels?.[0];
                                                            const name = skillLevelInfo?.name ?? `Skill ${idx + 1}`;
                                                            const recoveryType = skillLevelInfo?.spData?.spType ? getSpTypeLabel(skillLevelInfo.spData.spType) : "";
                                                            const triggerType = skillLevelInfo?.skillType ? getSkillTypeLabel(skillLevelInfo.skillType) : "";
                                                            const initialSp = skillLevelInfo?.spData?.initSp ?? 0;
                                                            const totalSp = skillLevelInfo?.spData?.spCost ?? 0;

                                                            return (
                                                                <div key={skill.skillId} className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-1.5 rounded-xl border border-border bg-card p-4 sm:grid-cols-[auto_1fr_auto]">
                                                                    <img src={skillIconURL(skill, selectedOperator.server)} alt={name} className="col-start-1 row-span-2 row-start-1 size-12 rounded-lg border border-border bg-muted/30 object-contain p-0.5" />

                                                                    <div className="col-start-2 row-start-1 flex items-center gap-2">
                                                                        <span className="font-bold text-base text-foreground leading-tight">{name}</span>
                                                                    </div>

                                                                    <div className="col-start-2 row-start-2 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-muted-foreground text-xs leading-none sm:hidden md:flex">
                                                                        {recoveryType && (
                                                                            <>
                                                                                <span>{recoveryType}</span>
                                                                                <span className="text-muted-foreground/30" aria-hidden="true">
                                                                                    •
                                                                                </span>
                                                                            </>
                                                                        )}
                                                                        {triggerType && <span>{triggerType}</span>}
                                                                        {totalSp > 0 && (
                                                                            <>
                                                                                <span className="text-muted-foreground/30" aria-hidden="true">
                                                                                    •
                                                                                </span>
                                                                                <span>
                                                                                    SP: {initialSp}/{totalSp}
                                                                                </span>
                                                                            </>
                                                                        )}
                                                                    </div>

                                                                    <div className="col-span-2 col-start-1 row-start-3 mt-2 grid grid-cols-10 gap-1 sm:col-span-1 sm:col-start-3 sm:row-span-2 sm:row-start-1 sm:mt-0 sm:flex sm:w-auto sm:justify-end sm:gap-1.5">
                                                                        {Array.from(
                                                                            {
                                                                                length: 1 + (selectedOperator.allSkillLevelUp?.length ?? 0) + (skill.levelUpCostCond?.length ?? 0),
                                                                            },
                                                                            (_, i) => i + 1,
                                                                        ).map((val) => {
                                                                            const isMastery = val > 7;
                                                                            const isActive = currentTarget === val;
                                                                            const isAllowed = isSkillLevelAllowed(selectedOperator, idx, val, elite, level);

                                                                            let tooltipText = "";
                                                                            if (!isAllowed) {
                                                                                const cond = isMastery ? skill.levelUpCostCond?.[val - 8]?.unlockCond : selectedOperator.allSkillLevelUp?.[val - 2]?.unlockCond;
                                                                                if (cond) {
                                                                                    const phaseLabel = cond.phase === "PHASE_0" ? "0" : cond.phase === "PHASE_1" ? "1" : "2";
                                                                                    tooltipText = isMastery ? `Skill Mastery ${val - 7} is available after Elite ${phaseLabel} Level ${cond.level}` : `Skill Level ${val} is available after Elite ${phaseLabel} Level ${cond.level}`;
                                                                                }
                                                                            }

                                                                            if (!isAllowed && tooltipText) {
                                                                                return (
                                                                                    <Tooltip key={val}>
                                                                                        <TooltipTrigger
                                                                                            render={(props) => (
                                                                                                <span {...props} className="flex aspect-square w-full shrink-0 cursor-help sm:size-9">
                                                                                                    <button
                                                                                                        type="button"
                                                                                                        disabled
                                                                                                        className="flex h-full w-full cursor-help items-center justify-center rounded-md border border-border/40 bg-muted/20 opacity-20 transition-all"
                                                                                                        title={isMastery ? `Mastery ${val - 7}` : `Level ${val}`}
                                                                                                    >
                                                                                                        {isMastery ? <img src={specializedIcon(val - 7)} alt={`Mastery ${val - 7}`} className="size-5 object-contain sm:size-6" /> : <span className="font-semibold text-xs sm:text-[13px]">{val}</span>}
                                                                                                    </button>
                                                                                                </span>
                                                                                            )}
                                                                                        />
                                                                                        <TooltipPopup>{tooltipText}</TooltipPopup>
                                                                                    </Tooltip>
                                                                                );
                                                                            }

                                                                            return (
                                                                                <button
                                                                                    key={val}
                                                                                    type="button"
                                                                                    onClick={() => handleSkillTargetChange(idx, val)}
                                                                                    className={cn(
                                                                                        "flex aspect-square w-full shrink-0 cursor-pointer items-center justify-center rounded-md border transition-all sm:size-9",
                                                                                        isActive ? "border-primary bg-primary/10 opacity-100 ring-2 ring-primary/20" : "border-border bg-muted/40 opacity-40 hover:bg-muted/80 hover:opacity-75",
                                                                                    )}
                                                                                    title={isMastery ? `Mastery ${val - 7}` : `Level ${val}`}
                                                                                >
                                                                                    {isMastery ? <img src={specializedIcon(val - 7)} alt={`Mastery ${val - 7}`} className="size-5 object-contain sm:size-6" /> : <span className="font-semibold text-xs sm:text-[13px]">{val}</span>}
                                                                                </button>
                                                                            );
                                                                        })}
                                                                    </div>
                                                                </div>
                                                            );
                                                        })}
                                                    </div>
                                                </CollapsibleContent>
                                            </Collapsible>
                                        )}

                                        {selectedOperator.modules.filter((m) => m.typeName1 !== "ORIGINAL").length > 0 && (
                                            <Collapsible open={modulesSectionOpen} onOpenChange={setModulesSectionOpen} className="space-y-3">
                                                <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                                                    <span className="flex items-center gap-2">
                                                        <span className="font-semibold text-foreground text-sm">Modules</span>
                                                    </span>
                                                    <ChevronDown className={cn("h-4 w-4 transition-transform", modulesSectionOpen && "rotate-180")} />
                                                </CollapsibleTrigger>
                                                <CollapsibleContent>
                                                    <div className="grid grid-cols-1 gap-4 pt-1">
                                                        {selectedOperator.modules
                                                            .filter((m) => m.typeName1 !== "ORIGINAL")
                                                            .map((mod) => {
                                                                const currentTarget = moduleTargets[mod.uniEquipId] ?? 0;
                                                                const name = mod.uniEquipName;
                                                                const moduleTag = mod.typeName1 && mod.typeName2 ? `${mod.typeName1}-${mod.typeName2}` : (mod.typeName1 ?? "Module");

                                                                return (
                                                                    <div key={mod.uniEquipId} className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-1.5 rounded-xl border border-border bg-card p-4 sm:grid-cols-[auto_1fr_auto]">
                                                                        <img src={moduleIconURL(mod, selectedOperator.server)} alt={name} className="col-start-1 row-span-2 row-start-1 size-12 rounded-lg border border-border bg-muted/30 object-contain p-0.5" />

                                                                        <div className="col-start-2 row-start-1 flex items-center gap-2">
                                                                            <span className="font-bold text-base text-foreground leading-tight">{name}</span>
                                                                        </div>

                                                                        <div className="col-start-2 row-start-2 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-muted-foreground text-xs leading-none">
                                                                            <span>{moduleTag}</span>
                                                                        </div>

                                                                        <div className="col-span-2 col-start-1 row-start-3 mt-2 flex gap-1.5 sm:col-span-1 sm:col-start-3 sm:row-span-2 sm:row-start-1 sm:mt-0 sm:justify-end">
                                                                            {[0, 1, 2, 3].map((val) => {
                                                                                const isActive = currentTarget === val;
                                                                                const isAllowed = val === 0 || isModuleAllowed(selectedOperator, mod.uniEquipId, elite, level);

                                                                                let tooltipText = "";
                                                                                if (!isAllowed) {
                                                                                    const phaseLabel = mod.unlockEvolvePhase === "PHASE_0" ? "0" : mod.unlockEvolvePhase === "PHASE_1" ? "1" : "2";
                                                                                    tooltipText = `Module unlocks at Elite ${phaseLabel} Level ${mod.unlockLevel}`;
                                                                                }

                                                                                if (!isAllowed && tooltipText) {
                                                                                    return (
                                                                                        <Tooltip key={val}>
                                                                                            <TooltipTrigger
                                                                                                render={(props) => (
                                                                                                    <span {...props} className="flex size-9 shrink-0 cursor-help">
                                                                                                        <button type="button" disabled className="flex h-full w-full cursor-help items-center justify-center rounded-md border border-border/40 bg-muted/20 opacity-20 transition-all" title={`Stage ${val}`}>
                                                                                                            <span className="font-semibold text-xs sm:text-[13px]">{val}</span>
                                                                                                        </button>
                                                                                                    </span>
                                                                                                )}
                                                                                            />
                                                                                            <TooltipPopup>{tooltipText}</TooltipPopup>
                                                                                        </Tooltip>
                                                                                    );
                                                                                }

                                                                                return (
                                                                                    <button
                                                                                        key={val}
                                                                                        type="button"
                                                                                        onClick={() =>
                                                                                            setModuleTargets((prev) => ({
                                                                                                ...prev,
                                                                                                [mod.uniEquipId]: val,
                                                                                            }))
                                                                                        }
                                                                                        className={cn(
                                                                                            "flex size-9 shrink-0 cursor-pointer items-center justify-center rounded-md border transition-all",
                                                                                            isActive ? "border-primary bg-primary/10 opacity-100 ring-2 ring-primary/20" : "border-border bg-muted/40 opacity-40 hover:bg-muted/80 hover:opacity-75",
                                                                                        )}
                                                                                        title={val === 0 ? "Not planned" : `Stage ${val}`}
                                                                                    >
                                                                                        <span className="font-semibold text-xs sm:text-[13px]">{val === 0 ? "X" : val}</span>
                                                                                    </button>
                                                                                );
                                                                            })}
                                                                        </div>
                                                                    </div>
                                                                );
                                                            })}
                                                    </div>
                                                </CollapsibleContent>
                                            </Collapsible>
                                        )}

                                        <div className="space-y-2">
                                            <label className="block font-medium text-[13px] text-muted-foreground leading-none" htmlFor="group-selector">
                                                Groups
                                            </label>
                                            <Combobox<string, true>
                                                multiple
                                                items={filteredGroupNames}
                                                value={selectedGroups}
                                                onValueChange={setSelectedGroups}
                                                filter={null}
                                                open={comboboxOpen}
                                                onOpenChange={(isOpen) => {
                                                    setComboboxOpen(isOpen);
                                                    if (!isOpen) {
                                                        setGroupSearchQuery("");
                                                        setIsCreatingGroup(false);
                                                        setNewGroupName("");
                                                    }
                                                }}
                                                inputValue={comboboxOpen ? groupSearchQuery : selectedGroups.join(", ")}
                                                onInputValueChange={(val) => {
                                                    if (comboboxOpen) {
                                                        setGroupSearchQuery(val);
                                                    }
                                                }}
                                                itemToStringLabel={(g) => g ?? ""}
                                                itemToStringValue={(g) => g ?? ""}
                                            >
                                                <ComboboxInput id="group-selector" placeholder="Select plan groups..." className="truncate text-ellipsis" />
                                                <ComboboxPopup className="max-w-100">
                                                    {isCreatingGroup ? (
                                                        <div className="flex items-center gap-2 border-border border-b p-2">
                                                            <Input
                                                                value={newGroupName}
                                                                onChange={(e) => setNewGroupName(e.target.value)}
                                                                placeholder="New group name..."
                                                                className="h-8 flex-1 text-xs"
                                                                autoFocus
                                                                onKeyDown={(e) => {
                                                                    if (e.key === "Enter") {
                                                                        e.preventDefault();
                                                                        handleCreateGroup();
                                                                    } else if (e.key === "Escape") {
                                                                        e.preventDefault();
                                                                        setIsCreatingGroup(false);
                                                                        setNewGroupName("");
                                                                    }
                                                                }}
                                                            />
                                                            <Button size="sm" className="h-8 px-2 text-xs" onClick={handleCreateGroup}>
                                                                Create
                                                            </Button>
                                                            <Button
                                                                variant="outline"
                                                                size="sm"
                                                                className="h-8 px-2 text-xs"
                                                                onClick={() => {
                                                                    setIsCreatingGroup(false);
                                                                    setNewGroupName("");
                                                                }}
                                                            >
                                                                Cancel
                                                            </Button>
                                                        </div>
                                                    ) : (
                                                        <div className="border-border border-b p-1">
                                                            <button type="button" onClick={() => setIsCreatingGroup(true)} className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left font-medium text-primary text-xs hover:bg-primary/10 hover:text-primary">
                                                                <Plus className="h-3.5 w-3.5" />
                                                                Create new group
                                                            </button>
                                                        </div>
                                                    )}
                                                    <ComboboxEmpty>No groups found.</ComboboxEmpty>
                                                    <ComboboxList>
                                                        {(name: string) => (
                                                            <ComboboxPrimitive.Item
                                                                key={name}
                                                                value={name}
                                                                className="flex min-h-8 cursor-default items-center justify-between gap-2 rounded-sm px-2 py-1 text-sm outline-none data-disabled:pointer-events-none data-highlighted:bg-accent data-highlighted:text-accent-foreground data-disabled:opacity-64 sm:min-h-7"
                                                            >
                                                                <div className="flex items-center gap-2">
                                                                    <span className={cn("flex size-4.5 shrink-0 items-center justify-center rounded-sm border border-input transition-colors sm:size-4", selectedGroups.includes(name) ? "border-primary bg-primary text-primary-foreground" : "border-input bg-background")}>
                                                                        {selectedGroups.includes(name) && (
                                                                            <svg aria-hidden="true" className="size-3 sm:size-2.5" fill="none" height="24" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="3" viewBox="0 0 24 24" width="24" xmlns="http://www.w3.org/2000/svg">
                                                                                <path d="M5.252 12.7 10.2 18.63 18.748 5.37" />
                                                                            </svg>
                                                                        )}
                                                                    </span>
                                                                    <span>{name}</span>
                                                                </div>
                                                                <div className="flex items-center gap-1">
                                                                    <button
                                                                        type="button"
                                                                        onClick={(e) => {
                                                                            e.stopPropagation();
                                                                            e.preventDefault();
                                                                            handleRenameGroup(name);
                                                                        }}
                                                                        className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
                                                                    >
                                                                        <Pencil className="h-3.5 w-3.5" />
                                                                    </button>
                                                                    <button
                                                                        type="button"
                                                                        onClick={(e) => {
                                                                            e.stopPropagation();
                                                                            e.preventDefault();
                                                                            handleDeleteGroup(name);
                                                                        }}
                                                                        className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-destructive"
                                                                    >
                                                                        <Trash className="h-3.5 w-3.5" />
                                                                    </button>
                                                                </div>
                                                            </ComboboxPrimitive.Item>
                                                        )}
                                                    </ComboboxList>
                                                </ComboboxPopup>
                                            </Combobox>
                                        </div>

                                        <div className="flex items-center justify-between rounded-xl border border-border bg-card/40 p-4">
                                            <div className="space-y-0.5">
                                                <label htmlFor="display-on-profile" className="cursor-pointer font-semibold text-foreground text-sm">
                                                    Display on profile
                                                </label>
                                                <p className="text-muted-foreground text-xs">Show this target plan on your public user profile.</p>
                                            </div>
                                            <Switch id="display-on-profile" checked={displayOnProfile} onCheckedChange={setDisplayOnProfile} />
                                        </div>
                                    </>
                                )}
                            </div>
                        ) : !selectedOperatorId ? (
                            <p className="text-muted-foreground text-sm italic">Please select an operator to customize targets.</p>
                        ) : null}
                    </div>
                </DialogPanel>

                <DialogFooter className="pb-8 sm:pb-8">
                    <DialogClose render={<Button variant="outline" className="w-full sm:w-auto" />}>Cancel</DialogClose>
                    <Button onClick={handleSave} disabled={isSaving || !selectedOperator || selectedOperator.id === "char_4195_radian"} className="w-full sm:w-auto">
                        {isSaving ? "Saving..." : isEditMode ? "Save changes" : "Create plan"}
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}
