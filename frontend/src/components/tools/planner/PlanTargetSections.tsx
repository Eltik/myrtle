import { ChevronDown } from "lucide-react";
import type * as React from "react";

import { eliteIcon, moduleIconURL, skillIconURL, specializedIcon } from "#/components/operators/detail/impl/assets";
import { getSkillTypeLabel, getSpTypeLabel, type HelperT } from "#/components/operators/detail/impl/helpers";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Input } from "#/components/ui/input";
import { Slider } from "#/components/ui/slider";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IOperatorListItem, IOperatorModule } from "#/types/operators";
import type { messages } from "./OperatorPlannerDialog.messages";
import { ELITE_PHASES, type IOperatorSkill, isModuleAllowed, isSkillLevelAllowed, MAX_SKILL_LEVEL, MODULE_STAGES, masteryOf, phaseLabel, plannableModules, skillStepCount, skillUnlockCond } from "./planTargets";
import type { IPlanTargets } from "./usePlanTargets";

type DialogT = TypedT<typeof messages>;

/** The selected/unselected look shared by the promotion buttons and every target step. */
function stepStateClass(isActive: boolean): string {
    return isActive ? "border-primary bg-primary/10 opacity-100 ring-2 ring-primary/20" : "border-border bg-muted/40 opacity-40 hover:bg-muted/80 hover:opacity-75";
}

interface IPromotionLevelPanelProps {
    targets: IPlanTargets;
}

/** The Elite buttons (only those the rarity reaches) and the level slider with its number input. */
export function PromotionLevelPanel({ targets }: IPromotionLevelPanelProps): React.ReactElement {
    const t: DialogT = useT("tools");
    const { elite, level, maxElite, maxLevel, changeElite, changeLevel } = targets;

    return (
        <div className="grid grid-cols-1 items-center gap-6 rounded-xl border border-border bg-card/40 p-4 sm:grid-cols-[auto_1fr]">
            <div className="flex flex-col gap-2">
                <span className="font-semibold text-muted-foreground text-xs uppercase tracking-wider">{t("planner.dialog.promotion")}</span>
                <div className="mt-1 flex items-center gap-2">
                    {ELITE_PHASES.filter((e) => e <= maxElite).map((e) => (
                        <button key={e} type="button" onClick={() => changeElite(e)} className={cn("relative flex size-12 cursor-pointer items-center justify-center rounded-lg border transition-all", stepStateClass(elite === e))} title={t("planner.dialog.elite", { elite: e })}>
                            <img src={eliteIcon(e)} alt={t("planner.dialog.elite", { elite: e })} className="icon-theme-aware size-7 object-contain" />
                        </button>
                    ))}
                </div>
            </div>

            <div className="flex w-full flex-col gap-2">
                <span className="font-semibold text-muted-foreground text-xs uppercase tracking-wider">{t("planner.dialog.level", { max: maxLevel })}</span>
                <div className="mt-2 flex items-center gap-4">
                    <Slider value={[level]} onValueChange={(vals) => changeLevel(Array.isArray(vals) ? (vals[0] ?? 1) : vals)} min={1} max={maxLevel} className="flex-1" />
                    <Input type="number" min={1} max={maxLevel} value={level} onChange={(e) => changeLevel(Number.parseInt(e.target.value, 10))} className="w-16 text-center font-mono" />
                </div>
            </div>
        </div>
    );
}

interface ITargetSectionProps {
    title: string;
    open: boolean;
    onOpenChange: (open: boolean) => void;
    children: React.ReactNode;
}

/** A collapsible block of target rows (skills, modules). */
function TargetSection({ title, open, onOpenChange, children }: ITargetSectionProps): React.ReactElement {
    return (
        <Collapsible open={open} onOpenChange={onOpenChange} className="space-y-3">
            <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                <span className="flex items-center gap-2">
                    <span className="font-semibold text-foreground text-sm">{title}</span>
                </span>
                <ChevronDown className={cn("h-4 w-4 transition-transform", open && "rotate-180")} />
            </CollapsibleTrigger>
            <CollapsibleContent>
                <div className="grid grid-cols-1 gap-4 pt-1">{children}</div>
            </CollapsibleContent>
        </Collapsible>
    );
}

interface ITargetRowProps {
    iconSrc: string;
    name: string;
    /** The subtitle line's items; its container is laid out here. */
    meta: React.ReactNode;
    metaClassName?: string;
    stepsClassName: string;
    /** The row's step buttons. */
    children: React.ReactNode;
}

/** One skill or module: icon, name and subtitle on the left, its step buttons on the right (below on mobile). */
function TargetRow({ iconSrc, name, meta, metaClassName, stepsClassName, children }: ITargetRowProps): React.ReactElement {
    return (
        <div className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-1.5 rounded-xl border border-border bg-card p-4 sm:grid-cols-[auto_1fr_auto]">
            <img src={iconSrc} alt={name} className="col-start-1 row-span-2 row-start-1 size-12 rounded-lg border border-border bg-muted/30 object-contain p-0.5" />

            <div className="col-start-2 row-start-1 flex items-center gap-2">
                <span className="font-bold text-base text-foreground leading-tight">{name}</span>
            </div>

            <div className={cn("col-start-2 row-start-2 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-muted-foreground text-xs leading-none", metaClassName)}>{meta}</div>

            <div className={cn("col-span-2 col-start-1 row-start-3 mt-2 sm:col-span-1 sm:col-start-3 sm:row-span-2 sm:row-start-1 sm:mt-0 sm:justify-end", stepsClassName)}>{children}</div>
        </div>
    );
}

interface ITargetStepButtonProps {
    isActive: boolean;
    /** Why the step is out of reach; when set, the step renders disabled with this as its tooltip. */
    lockedReason: string;
    title: string;
    /** Size classes, which differ between the skill and module grids. */
    sizeClassName: string;
    onSelect: () => void;
    children: React.ReactNode;
}

/**
 * One selectable target step. A locked step is a disabled button wrapped in a
 * span, because a disabled button fires no pointer events and the tooltip
 * trigger needs them.
 */
function TargetStepButton({ isActive, lockedReason, title, sizeClassName, onSelect, children }: ITargetStepButtonProps): React.ReactElement {
    if (lockedReason) {
        return (
            <Tooltip>
                <TooltipTrigger
                    render={(props) => (
                        <span {...props} className={cn("flex shrink-0 cursor-help", sizeClassName)}>
                            <button type="button" disabled className="flex h-full w-full cursor-help items-center justify-center rounded-md border border-border/40 bg-muted/20 opacity-20 transition-all" title={title}>
                                {children}
                            </button>
                        </span>
                    )}
                />
                <TooltipPopup>{lockedReason}</TooltipPopup>
            </Tooltip>
        );
    }

    return (
        <button type="button" onClick={onSelect} className={cn("flex shrink-0 cursor-pointer items-center justify-center rounded-md border transition-all", sizeClassName, stepStateClass(isActive))} title={title}>
            {children}
        </button>
    );
}

function MetaSeparator(): React.ReactElement {
    return (
        <span className="text-muted-foreground/30" aria-hidden="true">
            •
        </span>
    );
}

interface ISkillTargetsSectionProps {
    operator: IOperatorListItem;
    targets: IPlanTargets;
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function SkillTargetsSection({ operator, targets, open, onOpenChange }: ISkillTargetsSectionProps): React.ReactElement {
    const t: DialogT = useT("tools");
    return (
        <TargetSection title={t("planner.dialog.skills")} open={open} onOpenChange={onOpenChange}>
            {operator.skills.map((skill, idx) => (
                <SkillTargetRow key={skill.skillId} operator={operator} skill={skill} skillIdx={idx} targets={targets} />
            ))}
        </TargetSection>
    );
}

interface ISkillTargetRowProps {
    operator: IOperatorListItem;
    skill: IOperatorSkill;
    skillIdx: number;
    targets: IPlanTargets;
}

function SkillTargetRow({ operator, skill, skillIdx, targets }: ISkillTargetRowProps): React.ReactElement {
    const t: DialogT = useT("tools");
    // The skill recovery/trigger labels belong to the operator-detail feature,
    // so they are resolved against that namespace rather than this one.
    const operatorsT: HelperT = useT("operators");
    const { elite, level, skillTargets, changeSkillTarget } = targets;

    const currentTarget = skillTargets[skillIdx] ?? 1;
    // The name and SP numbers follow the targeted level, since a skill's SP cost changes as it levels.
    const levelInfo = skill.static?.levels?.[currentTarget - 1] ?? skill.static?.levels?.[0];
    const name = levelInfo?.name ?? t("planner.dialog.skillFallback", { index: skillIdx + 1 });
    const recoveryType = levelInfo?.spData?.spType ? getSpTypeLabel(levelInfo.spData.spType, operatorsT) : "";
    const triggerType = levelInfo?.skillType ? getSkillTypeLabel(levelInfo.skillType, operatorsT) : "";
    const initialSp = levelInfo?.spData?.initSp ?? 0;
    const totalSp = levelInfo?.spData?.spCost ?? 0;

    const lockedReason = (value: number): string => {
        if (isSkillLevelAllowed(operator, skillIdx, value, elite, level)) return "";
        const cond = skillUnlockCond(operator, skillIdx, value);
        if (!cond) return "";
        const requirement = { elite: phaseLabel(cond.phase), level: cond.level };
        return value > MAX_SKILL_LEVEL ? t("planner.dialog.masteryLocked", { mastery: masteryOf(value), ...requirement }) : t("planner.dialog.skillLevelLocked", { skillLevel: value, ...requirement });
    };

    const steps = Array.from({ length: skillStepCount(operator, skill) }, (_, i) => i + 1);

    return (
        <TargetRow
            iconSrc={skillIconURL(skill, operator.server)}
            name={name}
            metaClassName="sm:hidden md:flex"
            stepsClassName="grid grid-cols-10 gap-1 sm:flex sm:w-auto sm:gap-1.5"
            meta={
                <>
                    {recoveryType && (
                        <>
                            <span>{recoveryType}</span>
                            <MetaSeparator />
                        </>
                    )}
                    {triggerType && <span>{triggerType}</span>}
                    {totalSp > 0 && (
                        <>
                            <MetaSeparator />
                            <span>{t("planner.dialog.sp", { initial: initialSp, total: totalSp })}</span>
                        </>
                    )}
                </>
            }
        >
            {steps.map((value) => {
                const isMastery = value > MAX_SKILL_LEVEL;
                return (
                    <TargetStepButton
                        key={value}
                        isActive={currentTarget === value}
                        lockedReason={lockedReason(value)}
                        title={isMastery ? t("planner.dialog.mastery", { mastery: masteryOf(value) }) : t("planner.dialog.skillLevel", { level: value })}
                        sizeClassName="aspect-square w-full sm:size-9"
                        onSelect={() => changeSkillTarget(skillIdx, value)}
                    >
                        {isMastery ? <img src={specializedIcon(masteryOf(value))} alt={t("planner.dialog.mastery", { mastery: masteryOf(value) })} className="icon-theme-aware size-5 object-contain sm:size-6" /> : <span className="font-semibold text-xs sm:text-[13px]">{value}</span>}
                    </TargetStepButton>
                );
            })}
        </TargetRow>
    );
}

interface IModuleTargetsSectionProps {
    operator: IOperatorListItem;
    targets: IPlanTargets;
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function ModuleTargetsSection({ operator, targets, open, onOpenChange }: IModuleTargetsSectionProps): React.ReactElement {
    const t: DialogT = useT("tools");
    return (
        <TargetSection title={t("planner.dialog.modules")} open={open} onOpenChange={onOpenChange}>
            {plannableModules(operator).map((mod) => (
                <ModuleTargetRow key={mod.uniEquipId} operator={operator} mod={mod} targets={targets} />
            ))}
        </TargetSection>
    );
}

interface IModuleTargetRowProps {
    operator: IOperatorListItem;
    mod: IOperatorModule;
    targets: IPlanTargets;
}

function ModuleTargetRow({ operator, mod, targets }: IModuleTargetRowProps): React.ReactElement {
    const t: DialogT = useT("tools");
    const { elite, level, moduleTargets, changeModuleTarget } = targets;

    const currentTarget = moduleTargets[mod.uniEquipId] ?? 0;
    const moduleTag = mod.typeName1 && mod.typeName2 ? `${mod.typeName1}-${mod.typeName2}` : (mod.typeName1 ?? t("planner.dialog.moduleFallback"));
    // Every stage shares the module's one unlock requirement; stage 0 (not planned) is always reachable.
    const isUnlocked = isModuleAllowed(mod, elite, level);
    const lockedText = t("planner.dialog.moduleLocked", { elite: phaseLabel(mod.unlockEvolvePhase), level: mod.unlockLevel });

    return (
        <TargetRow iconSrc={moduleIconURL(mod, operator.server)} name={mod.uniEquipName} stepsClassName="flex gap-1.5" meta={<span>{moduleTag}</span>}>
            {MODULE_STAGES.map((stage) => {
                const isLocked = stage !== 0 && !isUnlocked;
                return (
                    <TargetStepButton key={stage} isActive={currentTarget === stage} lockedReason={isLocked ? lockedText : ""} title={stage === 0 ? t("planner.dialog.notPlanned") : t("planner.dialog.stage", { stage })} sizeClassName="size-9" onSelect={() => changeModuleTarget(mod.uniEquipId, stage)}>
                        <span className="font-semibold text-xs sm:text-[13px]">{stage === 0 ? "—" : stage}</span>
                    </TargetStepButton>
                );
            })}
        </TargetRow>
    );
}
