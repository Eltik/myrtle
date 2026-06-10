import { useQuery } from "@tanstack/react-query";

import { eliteIcon, moduleIconURL, skillIconURL } from "#/components/operators/detail/impl/assets";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { publicPlansQueryOptions } from "#/lib/api/planner";
import type { IRosterEntry } from "#/lib/api/user";
import { cn, formatSubProfession, rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";

function getSkillLevelLabel(level: number): string {
    if (level <= 7) return String(level);
    return `M${level - 7}`;
}

function getModuleStageLabel(stage: number): string {
    return stage === 0 ? "X" : String(stage);
}

interface IPlansTabProps {
    uid: string;
    roster: IRosterEntry[];
    operatorsStatic: IOperatorListItem[];
}

export function PlansTab({ uid, roster, operatorsStatic }: IPlansTabProps) {
    const { data: plans = [], isLoading } = useQuery(publicPlansQueryOptions(uid));

    if (isLoading) {
        return (
            <div className="grid grid-cols-[repeat(auto-fill,minmax(min(100%,320px),1fr))] gap-4">
                {["sk-p-1", "sk-p-2", "sk-p-3"].map((key) => (
                    <div key={key} className="h-24 animate-pulse rounded-xl border border-border/40 bg-card p-4" />
                ))}
            </div>
        );
    }

    if (plans.length === 0) {
        return (
            <div className="flex flex-col items-center justify-center gap-3 rounded-3xl border border-border bg-card px-8 py-16 text-center">
                <span className="font-mono text-[11px] text-muted-foreground uppercase tracking-widest">Plans</span>
                <h3 className="font-semibold text-lg tracking-tight">No public plans</h3>
                <p className="max-w-sm text-muted-foreground text-sm">This Doctor hasn't pinned any plans to their profile yet.</p>
            </div>
        );
    }

    return (
        <section aria-label="Operator plans" className="flex flex-col gap-4">
            <div className="grid grid-cols-[repeat(auto-fill,minmax(min(100%,320px),1fr))] gap-4">
                {plans.map((p) => {
                    const op = operatorsStatic.find((o) => o.id === p.operator_id);
                    if (!op) return null;

                    const rosterEntry = roster.find((re) => re.operator_id === p.operator_id);

                    const currElite = rosterEntry?.elite ?? 0;
                    const currLevel = rosterEntry?.level ?? 1;
                    const targetElite = p.target_elite;
                    const targetLevel = p.target_level;
                    const isLevelUpgraded = targetElite > currElite || (targetElite === currElite && targetLevel > currLevel);

                    return (
                        <div key={p.id} className="relative flex flex-col gap-4 rounded-xl border border-border bg-card p-4 transition-all hover:shadow-md">
                            <div className="flex items-start gap-3">
                                <span aria-hidden="true" className="relative flex size-10 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-border bg-muted/70">
                                    <OperatorAvatar charId={op.id} name={op.name} className="block h-full w-full object-cover" />
                                </span>
                                <div className="min-w-0 flex-1">
                                    <h3 className="truncate font-bold text-foreground text-sm leading-tight">{op.name}</h3>
                                    <p className="mt-0.5 truncate text-muted-foreground text-xs leading-normal">
                                        {rarityToNumber(op.rarity)}★ {formatSubProfession(op.subProfessionId)}
                                    </p>
                                </div>
                            </div>

                            <div className="flex flex-col gap-3 border-border/40 border-t pt-3 text-xs">
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
                        </div>
                    );
                })}
            </div>
        </section>
    );
}
