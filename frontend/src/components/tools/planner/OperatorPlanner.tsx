import { useQuery } from "@tanstack/react-query";
import { ChevronDown, ChevronRight, Lock, Plus } from "lucide-react";
import * as React from "react";

import { eliteIcon, itemIcon, moduleIconURL, skillIconURL } from "#/components/operators/detail/impl/assets";
import { Button } from "#/components/ui/button";
import { Checkbox } from "#/components/ui/checkbox";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { operatorsListQueryOptions } from "#/lib/api/operators";
import { type IOperatorPlanResponse, type IPlanRequirementItem, plansQueryOptions } from "#/lib/api/planner";
import { userRosterQueryOptions } from "#/lib/api/user";
import { authActions } from "#/lib/auth/store";
import { cn, formatSubProfession, rarityToNumber } from "#/lib/utils";

import { OperatorPlannerDialog } from "./OperatorPlannerDialog";

function UnauthenticatedState() {
    return (
        <div className="mt-8 flex flex-col items-center justify-center gap-6 rounded-[14px] border border-border bg-card px-8 py-16 text-center">
            <div className="flex flex-col items-center gap-3">
                <div className="flex h-14 w-14 items-center justify-center rounded-2xl border border-border bg-muted text-2xl">
                    <Lock />
                </div>
                <div>
                    <h2 className="font-sans font-semibold text-[20px] text-foreground tracking-[-0.02em]">Sign in to use the planner</h2>
                    <p className="mt-1.5 max-w-[42ch] font-sans text-muted-foreground text-sm">Your operator promotion, level, skill, and module goals are saved to your account so you can access them anywhere.</p>
                </div>
            </div>
            <button type="button" onClick={() => authActions.openLoginDialog()} className="inline-flex h-9 cursor-pointer items-center rounded-lg bg-primary px-5 font-medium font-sans text-primary-foreground text-sm transition-opacity hover:opacity-90">
                Sign in
            </button>
        </div>
    );
}

function getSkillLevelLabel(level: number): string {
    if (level <= 7) return String(level);
    return `M${level - 7}`;
}

function getModuleStageLabel(stage: number): string {
    return stage === 0 ? "X" : String(stage);
}

/** Props for the recursive requirement row component. */
interface PlannerRequirementRowProps {
    item: IPlanRequirementItem;
    depth: number;
    path: string;
    expandedPaths: Record<string, boolean>;
    onToggleExpand: (path: string) => void;
}

/** Recursively renders requirement rows. */
function PlannerRequirementRow({ item, depth, path, expandedPaths, onToggleExpand }: PlannerRequirementRowProps) {
    const hasRecipe = !!(item.recipe && item.recipe.costs.length > 0);
    const isExpanded = expandedPaths[path];

    const handleClick = () => {
        if (hasRecipe) {
            onToggleExpand(path);
        }
    };

    return (
        <>
            <tr className={cn("group transition-colors hover:bg-muted/20", hasRecipe && "cursor-pointer")} onClick={handleClick}>
                <td className="py-2.5 pr-4">
                    <div className="flex items-center gap-2" style={{ paddingLeft: `${depth * 1.25}rem` }}>
                        {hasRecipe ? isExpanded ? <ChevronDown className="size-3.5 shrink-0 text-muted-foreground" /> : <ChevronRight className="size-3.5 shrink-0 text-muted-foreground" /> : <div className="size-3.5 shrink-0" />}
                        <img
                            src={itemIcon(item.id, item.iconId, item.image)}
                            alt={item.name}
                            className="size-8 shrink-0 rounded-md object-contain"
                            onError={(e) => {
                                (e.currentTarget as HTMLImageElement).style.display = "none";
                            }}
                        />
                        <span className="font-medium text-foreground text-xs leading-tight">{item.name}</span>
                    </div>
                </td>
                <td className="py-2.5 pr-2 text-right text-foreground text-xs tabular-nums">{item.requiredCount.toLocaleString()}</td>
                <td className={cn("px-2 py-2.5 text-right text-xs tabular-nums", item.inventoryCount >= item.requiredCount ? "text-emerald-400" : "text-muted-foreground")}>{item.inventoryCount.toLocaleString()}</td>
                <td className="px-2 py-2.5 text-right text-xs tabular-nums">
                    {item.canCraft ? (
                        <span className="text-sky-400">{item.craftableCount.toLocaleString()}</span>
                    ) : (
                        <span className="text-muted-foreground/50" title={item.craftReason}>
                            —
                        </span>
                    )}
                </td>
                <td className="py-2.5 pl-2 text-right text-xs tabular-nums">{item.missingCount > 0 ? <span className="font-semibold text-red-400 text-xs">{item.missingCount.toLocaleString()}</span> : <span className="font-semibold text-emerald-400">✓</span>}</td>
            </tr>
            {hasRecipe &&
                isExpanded &&
                item.recipe?.costs.map((cost) => {
                    const childPath = `${path}/${cost.item.id}`;
                    return <PlannerRequirementRow key={childPath} item={cost.item} depth={depth + 1} path={childPath} expandedPaths={expandedPaths} onToggleExpand={onToggleExpand} />;
                })}
        </>
    );
}

/** Page wrapper for the operator planner tool. */
export function OperatorPlanner(): React.ReactElement {
    const { isAuthenticated, user } = useAuth();
    const [open, setOpen] = React.useState(false);
    const [plansExpanded, setPlansExpanded] = React.useState(true);
    const [activePlans, setActivePlans] = React.useState<Record<string, boolean>>({});
    const [expandedPlans, setExpandedPlans] = React.useState<Record<string, boolean>>({});
    const [expandedPaths, setExpandedPaths] = React.useState<Record<string, boolean>>({});

    const { data: initialPlannerData, isLoading: initialPlansLoading } = useQuery({
        ...plansQueryOptions(),
        enabled: isAuthenticated,
    });

    const activeIds = React.useMemo(() => {
        if (!initialPlannerData?.plans) return undefined;
        const totalPlansCount = initialPlannerData.plans.length;
        const activeList = initialPlannerData.plans.filter((p) => activePlans[p.operator_id] ?? true).map((p) => p.operator_id);

        if (activeList.length === totalPlansCount) {
            return undefined;
        }
        if (totalPlansCount > 0 && activeList.length === 0) {
            return ["none"];
        }
        return activeList;
    }, [initialPlannerData?.plans, activePlans]);

    const { data: operators = [], isLoading: operatorsLoading } = useQuery(operatorsListQueryOptions());

    const { data: roster = [], isLoading: rosterLoading } = useQuery({
        ...userRosterQueryOptions(user?.uid ?? ""),
        enabled: !!user?.uid,
    });

    const isPlansListLoading = initialPlansLoading || operatorsLoading || rosterLoading;

    const { data: plannerData, isLoading: requirementsLoading } = useQuery({
        ...plansQueryOptions(activeIds),
        enabled: isAuthenticated && !isPlansListLoading,
    });

    const isRequirementsLoading = requirementsLoading || isPlansListLoading;

    const plans = plannerData?.plans ?? initialPlannerData?.plans ?? [];
    const aggregatedRequirements = plannerData?.aggregatedRequirements ?? [];

    const togglePlan = (opId: string) => {
        setActivePlans((prev) => ({
            ...prev,
            [opId]: !(prev[opId] ?? true),
        }));
    };

    const togglePlanExpanded = (planId: string) => {
        setExpandedPlans((prev) => ({
            ...prev,
            [planId]: !prev[planId],
        }));
    };

    const handleToggleExpand = (path: string) => {
        setExpandedPaths((prev) => ({
            ...prev,
            [path]: !prev[path],
        }));
    };

    return (
        <div className="relative z-1 mx-auto w-[min(1400px,calc(100%-1.5rem))] py-4 pb-24 sm:w-[min(1400px,calc(100%-2rem))] sm:py-5 sm:pb-20">
            <nav aria-label="breadcrumb" className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none">
                <span>Tools</span>
                <ChevronRight className="size-2.5" />
                <span className="text-foreground">Operator Planner</span>
            </nav>
            <div className="flex flex-wrap items-end justify-between gap-3">
                <div className="min-w-0 flex-1">
                    <h1 className="m-0 font-bold font-sans text-[24px] text-foreground leading-[1.1] tracking-tight sm:text-[30px]">Operator Planner</h1>
                    <p className="mt-1.5 max-w-2xl font-sans text-[13px] text-muted-foreground leading-normal sm:text-[13.5px]">Plan your operator promotion, level, skill, and module goals.</p>
                </div>
                {isAuthenticated && plans.length > 0 && (
                    <Button onClick={() => setOpen(true)} size="sm">
                        <Plus className="mr-1.5 size-4" />
                        Create plan
                    </Button>
                )}
            </div>

            {!isAuthenticated ? (
                <UnauthenticatedState />
            ) : !isPlansListLoading && plans.length === 0 ? (
                <>
                    <div className="mt-16 flex flex-col items-center justify-center gap-4 py-20 text-center sm:mt-24 sm:py-28">
                        <Button size="xl" className="shadow-lg" onClick={() => setOpen(true)}>
                            <Plus className="size-5" />
                            Create new plan
                        </Button>
                    </div>

                    <OperatorPlannerDialog open={open} onOpenChange={setOpen} />
                </>
            ) : (
                <>
                    <div className="mt-8 flex flex-col gap-6 min-[720px]:flex-row">
                        <div className="w-full shrink-0 min-[720px]:w-90">
                            <div className="rounded-xl border border-border bg-card p-4">
                                <button
                                    type="button"
                                    className="flex w-full cursor-pointer items-center justify-between text-left font-semibold min-[720px]:cursor-default"
                                    onClick={() => {
                                        if (window.innerWidth < 720) {
                                            setPlansExpanded((prev) => !prev);
                                        }
                                    }}
                                >
                                    <span className="font-semibold text-foreground text-sm">Your plans</span>
                                    <ChevronDown className={cn("size-4 transition-transform min-[720px]:hidden", plansExpanded && "rotate-180")} />
                                </button>
                                <div className={cn("mt-4 min-[720px]:block", plansExpanded ? "block" : "hidden")}>
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
                                        <div className="flex max-h-[calc(100vh-16rem)] flex-col gap-4 overflow-y-auto pr-1">
                                            {plans.map((p: IOperatorPlanResponse) => {
                                                const op = operators.find((o) => o.id === p.operator_id);
                                                if (!op) return null;
                                                const rosterEntry = roster?.find((re) => re.operator_id === p.operator_id);
                                                const isActive = activePlans[p.operator_id] ?? true;

                                                const currElite = rosterEntry?.elite ?? 0;
                                                const currLevel = rosterEntry?.level ?? 1;
                                                const targetElite = p.target_elite;
                                                const targetLevel = p.target_level;
                                                const isLevelUpgraded = targetElite > currElite || (targetElite === currElite && targetLevel > currLevel);

                                                return (
                                                    <label key={p.id} className={cn("relative flex cursor-pointer flex-col gap-4 rounded-xl border p-4 transition-all hover:shadow-md", isActive ? "border-primary bg-primary/5 shadow-sm ring-2 ring-primary/20" : "border-border/40 bg-muted/20 opacity-60")}>
                                                        <div className="flex items-start gap-3">
                                                            <Checkbox checked={isActive} onCheckedChange={() => togglePlan(p.operator_id)} />
                                                            <span aria-hidden="true" className="relative flex size-10 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-border bg-muted/70">
                                                                <OperatorAvatar charId={op.id} name={op.name} className="block h-full w-full object-cover" />
                                                            </span>
                                                            <div className="min-w-0 flex-1">
                                                                <h3 className="truncate font-bold text-foreground text-sm leading-tight">{op.name}</h3>
                                                                <p className="mt-0.5 truncate text-muted-foreground text-xs leading-normal">
                                                                    {rarityToNumber(op.rarity)}★ {formatSubProfession(op.subProfessionId)}
                                                                </p>
                                                            </div>
                                                            <button
                                                                type="button"
                                                                onClick={(e) => {
                                                                    e.preventDefault();
                                                                    togglePlanExpanded(p.id);
                                                                }}
                                                                className="flex size-7 shrink-0 items-center justify-center rounded-md border border-border bg-muted/40 text-foreground shadow-xs transition-all hover:border-border/80 hover:bg-muted"
                                                            >
                                                                <ChevronDown className={cn("size-4 transition-transform", expandedPlans[p.id] && "rotate-180")} />
                                                            </button>
                                                        </div>

                                                        {expandedPlans[p.id] && (
                                                            <div className="fade-in slide-in-from-top-2 flex animate-in flex-col gap-3 border-border/40 border-t pt-3 text-xs duration-200">
                                                                <div className="flex items-center justify-between">
                                                                    <span className="font-medium text-muted-foreground">Level</span>
                                                                    <div className="flex items-center gap-2">
                                                                        <div className="flex items-center gap-1 font-medium">
                                                                            <img src={eliteIcon(currElite)} alt={`Elite ${currElite}`} className="icon-theme-aware size-5 object-contain" />
                                                                            <span>Lv.{currLevel}</span>
                                                                        </div>
                                                                        <span className="text-muted-foreground/50">➔</span>
                                                                        <div className={cn("flex items-center gap-1 font-bold", isLevelUpgraded ? "text-primary" : "text-muted-foreground")}>
                                                                            <img src={eliteIcon(targetElite)} alt={`Elite ${targetElite}`} className={cn("icon-theme-aware size-5 object-contain", !isLevelUpgraded && "opacity-50")} />
                                                                            <span>Lv.{targetLevel}</span>
                                                                        </div>
                                                                    </div>
                                                                </div>

                                                                {op.skills.length > 0 && (
                                                                    <div className="flex flex-col gap-2">
                                                                        <span className="font-medium text-muted-foreground">Skills</span>
                                                                        <div className="flex flex-col gap-1.5 pl-1">
                                                                            {op.skills.map((skill, idx) => {
                                                                                let currSkillVal = 1;
                                                                                if (rosterEntry) {
                                                                                    if (rosterEntry.skill_level < 7) {
                                                                                        currSkillVal = rosterEntry.skill_level;
                                                                                    } else {
                                                                                        const masteryEntry = rosterEntry.masteries?.find((m) => m.index === idx);
                                                                                        currSkillVal = masteryEntry && masteryEntry.mastery > 0 ? 7 + masteryEntry.mastery : 7;
                                                                                    }
                                                                                }

                                                                                let targetSkillVal = 1;
                                                                                if (p.target_skill_level < 7) {
                                                                                    targetSkillVal = p.target_skill_level;
                                                                                } else {
                                                                                    const masteryEntry = p.target_skills?.find((s: { skill_index: number; mastery_level: number }) => s.skill_index === idx);
                                                                                    targetSkillVal = masteryEntry && masteryEntry.mastery_level > 0 ? 7 + masteryEntry.mastery_level : 7;
                                                                                }

                                                                                const isSkillUpgraded = targetSkillVal > currSkillVal;

                                                                                return (
                                                                                    <div key={skill.skillId} className="flex items-center justify-between">
                                                                                        <div className="flex min-w-0 flex-1 items-center gap-1.5">
                                                                                            <img src={skillIconURL(skill)} alt={skill.static?.levels?.[0]?.name} className="size-5 rounded border border-border/40 object-contain" />
                                                                                            <span className="truncate font-medium text-foreground">{skill.static?.levels?.[0]?.name ?? `Skill ${idx + 1}`}</span>
                                                                                        </div>
                                                                                        <div className="ml-2 flex shrink-0 items-center gap-2">
                                                                                            <span className="font-medium">{getSkillLevelLabel(currSkillVal)}</span>
                                                                                            <span className="text-muted-foreground/50">➔</span>
                                                                                            <span className={cn("font-bold", isSkillUpgraded ? "text-primary" : "text-muted-foreground")}>{getSkillLevelLabel(targetSkillVal)}</span>
                                                                                        </div>
                                                                                    </div>
                                                                                );
                                                                            })}
                                                                        </div>
                                                                    </div>
                                                                )}

                                                                {op.modules.filter((m) => m.typeName1 !== "ORIGINAL").length > 0 && (
                                                                    <div className="flex flex-col gap-2">
                                                                        <span className="font-medium text-muted-foreground">Modules</span>
                                                                        <div className="flex flex-col gap-1.5 pl-1">
                                                                            {op.modules
                                                                                .filter((m) => m.typeName1 !== "ORIGINAL")
                                                                                .map((mod) => {
                                                                                    let currModStage = 0;
                                                                                    if (rosterEntry) {
                                                                                        const modEntry = rosterEntry.modules?.find((rm) => rm.id === mod.uniEquipId);
                                                                                        currModStage = modEntry && !modEntry.locked ? modEntry.level : 0;
                                                                                    }

                                                                                    const targetModStage = p.target_modules?.find((tm: { module_id: string; module_stage: number }) => tm.module_id === mod.uniEquipId)?.module_stage ?? 0;
                                                                                    const isModUpgraded = targetModStage > currModStage;

                                                                                    return (
                                                                                        <div key={mod.uniEquipId} className="flex items-center justify-between">
                                                                                            <div className="flex min-w-0 flex-1 items-center gap-1.5">
                                                                                                <img src={moduleIconURL(mod)} alt={mod.uniEquipName} className="size-5 rounded object-contain" />
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
                                                        )}
                                                    </label>
                                                );
                                            })}
                                        </div>
                                    )}
                                </div>
                            </div>
                        </div>
                        <div className="flex-1">
                            <div className="rounded-xl border border-border bg-card p-4">
                                <h2 className="font-semibold text-foreground text-sm">Requirements</h2>
                                {isRequirementsLoading ? (
                                    <div className="mt-4 flex flex-col divide-y divide-border/40">
                                        {["sk-req-1", "sk-req-2", "sk-req-3", "sk-req-4", "sk-req-5", "sk-req-6", "sk-req-7", "sk-req-8"].map((key) => (
                                            <div key={key} className="flex items-center gap-3 py-3 first:pt-1">
                                                <Skeleton className="size-9 shrink-0 rounded-lg" />
                                                <Skeleton className="h-3.5 w-28 rounded" />
                                                <div className="ml-auto flex gap-6">
                                                    <Skeleton className="h-3 w-8 rounded" />
                                                    <Skeleton className="h-3 w-8 rounded" />
                                                    <Skeleton className="h-3 w-8 rounded" />
                                                    <Skeleton className="h-3 w-8 rounded" />
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                ) : aggregatedRequirements.length === 0 ? (
                                    <p className="mt-6 text-center text-muted-foreground text-sm">No requirements for the selected plans.</p>
                                ) : (
                                    <div className="mt-3 overflow-x-auto">
                                        <table className="w-full text-sm">
                                            <thead>
                                                <tr className="border-border/40 border-b">
                                                    <th className="pb-2 text-left font-medium text-muted-foreground text-xs">Item</th>
                                                    <th className="pr-2 pb-2 text-right font-medium text-muted-foreground text-xs">Required</th>
                                                    <th className="px-2 pb-2 text-right font-medium text-muted-foreground text-xs">Have</th>
                                                    <th className="px-2 pb-2 text-right font-medium text-muted-foreground text-xs">Craftable</th>
                                                    <th className="pb-2 pl-2 text-right font-medium text-muted-foreground text-xs">Missing</th>
                                                </tr>
                                            </thead>
                                            <tbody className="divide-y divide-border/30">
                                                {aggregatedRequirements.map((req) => (
                                                    <PlannerRequirementRow key={req.id} item={req} depth={0} path={req.id} expandedPaths={expandedPaths} onToggleExpand={handleToggleExpand} />
                                                ))}
                                            </tbody>
                                        </table>
                                    </div>
                                )}
                            </div>
                        </div>
                    </div>

                    <OperatorPlannerDialog open={open} onOpenChange={setOpen} />
                </>
            )}
        </div>
    );
}
