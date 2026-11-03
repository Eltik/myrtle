import { ChevronDown } from "lucide-react";
import type * as React from "react";

import { eliteIcon, moduleIconURL, skillIconURL, specializedIcon } from "#/components/operators/detail/impl/assets";
import { getSkillTypeLabel, getSpTypeLabel, type HelperT } from "#/components/operators/detail/impl/helpers";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { InfoHint } from "#/components/ui/info-hint";
import { Input } from "#/components/ui/input";
import { Kicker } from "#/components/ui/kicker";
import { Slider } from "#/components/ui/slider";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IOperatorListItem, IOperatorModule } from "#/types/operators";
import type { messages } from "./OperatorPlannerDialog.messages";
import { ELITE_PHASES, type IOperatorSkill, isEliteBelowFloor, isModuleAllowed, isModuleBelowFloor, isSkillBelowFloor, isSkillLevelAllowed, MAX_SKILL_LEVEL, MODULE_STAGES, masteryOf, phaseLabel, plannableModules, skillStepCount, skillUnlockCond } from "./planTargets";
import type { IPlanTargets } from "./usePlanTargets";

type DialogT = TypedT<typeof messages>;

/** The selected/unselected look shared by the promotion buttons and every target step. */
/** A target step is a kit outline toggle; the `sm:` sizes repeat so the toggle's own `sm:h-8` does not shrink a step. */
export const STEP_TOGGLE = "min-w-0 shrink-0 p-0";

interface IPromotionLevelPanelProps {
    targets: IPlanTargets;
}

/** The Elite buttons (only those the rarity reaches) and the level slider with its number input. */
export function PromotionLevelPanel({ targets }: IPromotionLevelPanelProps): React.ReactElement {
    const t: DialogT = useT("tools");
    const { elite, level, maxElite, maxLevel, minLevel, floor, changeElite, changeLevel } = targets;
    const phases = ELITE_PHASES.filter((e) => e <= maxElite);

    return (
        <div className="grid grid-cols-1 items-center gap-6 rounded-2xl border border-border bg-card/40 p-4 sm:grid-cols-[auto_1fr]">
            <div className="flex flex-col gap-2">
                <Kicker>{t("planner.dialog.promotion")}</Kicker>
                <div className="mt-1 flex items-center gap-2">
                    <ToggleGroup aria-label={t("planner.dialog.promotion")} className="gap-2" onValueChange={(next) => next[0] !== undefined && changeElite(Number(next[0]))} value={[String(elite)]} variant="outline">
                        {phases.map((e) => {
                            const isReached = isEliteBelowFloor(floor, e);
                            const icon = <img src={eliteIcon(e)} alt={t("planner.dialog.elite", { elite: e })} className="icon-theme-aware size-7 object-contain" />;
                            // A reached phase is a disabled button, which fires no pointer events, so its tooltip trigger is a wrapping span.
                            return (
                                <Tooltip key={e}>
                                    {isReached ? (
                                        <TooltipTrigger render={<span className="inline-flex" />}>
                                            <ToggleGroupItem className={cn(STEP_TOGGLE, "size-12 disabled:opacity-20 sm:size-12")} disabled value={String(e)}>
                                                {icon}
                                            </ToggleGroupItem>
                                        </TooltipTrigger>
                                    ) : (
                                        <TooltipTrigger render={<ToggleGroupItem className={cn(STEP_TOGGLE, "size-12 sm:size-12")} value={String(e)} />}>{icon}</TooltipTrigger>
                                    )}
                                    <TooltipPopup>{isReached ? t("planner.dialog.alreadyReached") : t("planner.dialog.elite", { elite: e })}</TooltipPopup>
                                </Tooltip>
                            );
                        })}
                    </ToggleGroup>
                    <LockedHint reasons={phases.filter((e) => isEliteBelowFloor(floor, e)).map((e) => `${t("planner.dialog.elite", { elite: e })}: ${t("planner.dialog.alreadyReached")}`)} />
                </div>
            </div>

            <div className="flex w-full flex-col gap-2">
                <Kicker>{t("planner.dialog.level", { max: maxLevel })}</Kicker>
                <div className="mt-2 flex items-center gap-4">
                    <Slider value={[level]} onValueChange={(vals) => changeLevel(Array.isArray(vals) ? (vals[0] ?? 1) : vals)} min={minLevel} max={maxLevel} className="flex-1" />
                    <Input type="number" min={minLevel} max={maxLevel} value={level} onChange={(e) => changeLevel(Number.parseInt(e.target.value, 10))} className="w-16 text-center font-mono" />
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
    /** The selected step's value; the steps are one single-select group named after the row. */
    value: number;
    onValueChange: (value: number) => void;
    /** Each locked step's reason, offered beside the group so a keyboard reaches it too. */
    lockedReasons: string[];
    children: React.ReactNode;
}

/** One skill or module: icon, name and subtitle on the left, its step buttons on the right (below on mobile). */
function TargetRow({ iconSrc, name, meta, metaClassName, stepsClassName, value, onValueChange, lockedReasons, children }: ITargetRowProps): React.ReactElement {
    return (
        <div className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-1.5 rounded-2xl border border-border bg-card p-4 sm:grid-cols-[auto_1fr_auto]">
            <img src={iconSrc} alt={name} className="col-start-1 row-span-2 row-start-1 size-12 rounded-lg border border-border bg-muted/30 object-contain p-0.5" />

            <div className="col-start-2 row-start-1 flex items-center gap-2">
                <span className="font-bold text-base text-foreground leading-tight">{name}</span>
            </div>

            <div className={cn("col-start-2 row-start-2 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-muted-foreground text-xs leading-none", metaClassName)}>{meta}</div>

            <div className="col-span-2 col-start-1 row-start-3 mt-2 flex items-center gap-2 sm:col-span-1 sm:col-start-3 sm:row-span-2 sm:row-start-1 sm:mt-0 sm:justify-end">
                <ToggleGroup aria-label={name} className={stepsClassName} onValueChange={(next) => next[0] !== undefined && onValueChange(Number(next[0]))} value={[String(value)]} variant="outline">
                    {children}
                </ToggleGroup>
                <LockedHint reasons={lockedReasons} />
            </div>
        </div>
    );
}

interface ITargetStepButtonProps {
    value: number;
    /** Why the step is out of reach; when set, the step renders disabled with this as its tooltip. */
    lockedReason: string;
    title: string;
    /** Size classes, which differ between the skill and module grids. */
    sizeClassName: string;
    children: React.ReactNode;
}

/**
 * One selectable target step. A locked step is a disabled button wrapped in a
 * span, because a disabled button fires no pointer events and the tooltip
 * trigger needs them.
 */
function TargetStepButton({ value, lockedReason, title, sizeClassName, children }: ITargetStepButtonProps): React.ReactElement {
    if (lockedReason) {
        return (
            <Tooltip>
                <TooltipTrigger
                    render={(props) => (
                        <span {...props} className={cn("flex shrink-0", sizeClassName)}>
                            <ToggleGroupItem aria-label={title} className={cn(STEP_TOGGLE, "h-full w-full disabled:opacity-20 sm:h-full")} disabled value={String(value)}>
                                {children}
                            </ToggleGroupItem>
                        </span>
                    )}
                />
                <TooltipPopup>{lockedReason}</TooltipPopup>
            </Tooltip>
        );
    }

    return (
        <Tooltip>
            <TooltipTrigger render={<ToggleGroupItem aria-label={title} className={cn(STEP_TOGGLE, sizeClassName)} value={String(value)} />}>{children}</TooltipTrigger>
            <TooltipPopup>{title}</TooltipPopup>
        </Tooltip>
    );
}

/** The locked steps' reasons in one focusable hint: a disabled step takes no focus, so its own tooltip never reaches a keyboard. */
function LockedHint({ reasons }: { reasons: string[] }): React.ReactElement | null {
    const t: DialogT = useT("tools");
    const unique = [...new Set(reasons)];
    if (unique.length === 0) return null;
    return (
        <InfoHint label={t("planner.dialog.lockedHint")}>
            {unique.map((reason) => (
                <span className="block" key={reason}>
                    {reason}
                </span>
            ))}
        </InfoHint>
    );
}

function MetaSeparator(): React.ReactElement {
    return (
        <span className="text-subtle-foreground" aria-hidden="true">
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
    const { elite, level, floor, skillTargets, changeSkillTarget } = targets;

    const currentTarget = skillTargets[skillIdx] ?? 1;
    // The name and SP numbers follow the targeted level, since a skill's SP cost changes as it levels.
    const levelInfo = skill.static?.levels?.[currentTarget - 1] ?? skill.static?.levels?.[0];
    const name = levelInfo?.name ?? t("planner.dialog.skillFallback", { index: skillIdx + 1 });
    const recoveryType = levelInfo?.spData?.spType ? getSpTypeLabel(levelInfo.spData.spType, operatorsT) : "";
    const triggerType = levelInfo?.skillType ? getSkillTypeLabel(levelInfo.skillType, operatorsT) : "";
    const initialSp = levelInfo?.spData?.initSp ?? 0;
    const totalSp = levelInfo?.spData?.spCost ?? 0;

    const lockedReason = (value: number): string => {
        if (isSkillBelowFloor(floor, skillIdx, value)) return t("planner.dialog.alreadyReached");
        if (isSkillLevelAllowed(operator, skillIdx, value, elite, level)) return "";
        const cond = skillUnlockCond(operator, skillIdx, value);
        if (!cond) return "";
        const requirement = { elite: phaseLabel(cond.phase), level: cond.level };
        return value > MAX_SKILL_LEVEL ? t("planner.dialog.masteryLocked", { mastery: masteryOf(value), ...requirement }) : t("planner.dialog.skillLevelLocked", { skillLevel: value, ...requirement });
    };

    const steps = Array.from({ length: skillStepCount(operator, skill) }, (_, i) => i + 1);
    const stepTitle = (value: number): string => (value > MAX_SKILL_LEVEL ? t("planner.dialog.mastery", { mastery: masteryOf(value) }) : t("planner.dialog.skillLevel", { level: value }));

    return (
        <TargetRow
            iconSrc={skillIconURL(skill, operator.server)}
            name={name}
            metaClassName="sm:hidden md:flex"
            stepsClassName="grid min-w-0 flex-1 grid-cols-10 gap-1 sm:flex sm:w-auto sm:flex-none sm:gap-1.5"
            value={currentTarget}
            onValueChange={(value) => changeSkillTarget(skillIdx, value)}
            lockedReasons={steps.flatMap((value) => {
                const reason = lockedReason(value);
                return reason ? [`${stepTitle(value)}: ${reason}`] : [];
            })}
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
                    <TargetStepButton key={value} value={value} lockedReason={lockedReason(value)} title={stepTitle(value)} sizeClassName="aspect-square h-auto w-full sm:size-9">
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
    const { elite, level, floor, moduleTargets, changeModuleTarget } = targets;

    const currentTarget = moduleTargets[mod.uniEquipId] ?? 0;
    const moduleTag = mod.typeName1 && mod.typeName2 ? `${mod.typeName1}-${mod.typeName2}` : (mod.typeName1 ?? t("planner.dialog.moduleFallback"));
    // Every stage shares the module's one unlock requirement; stage 0 (not planned) is always reachable.
    const isUnlocked = isModuleAllowed(mod, elite, level);
    const lockedText = t("planner.dialog.moduleLocked", { elite: phaseLabel(mod.unlockEvolvePhase), level: mod.unlockLevel });
    const stageLockedReason = (stage: number): string => (isModuleBelowFloor(floor, mod.uniEquipId, stage) ? t("planner.dialog.alreadyReached") : stage !== 0 && !isUnlocked ? lockedText : "");
    const stageTitle = (stage: number): string => (stage === 0 ? t("planner.dialog.notPlanned") : t("planner.dialog.stage", { stage }));

    return (
        <TargetRow
            iconSrc={moduleIconURL(mod, operator.server)}
            name={mod.uniEquipName}
            stepsClassName="gap-1.5"
            meta={<span>{moduleTag}</span>}
            value={currentTarget}
            onValueChange={(stage) => changeModuleTarget(mod.uniEquipId, stage)}
            lockedReasons={MODULE_STAGES.flatMap((stage) => {
                const reason = stageLockedReason(stage);
                return reason ? [`${stageTitle(stage)}: ${reason}`] : [];
            })}
        >
            {MODULE_STAGES.map((stage) => {
                return (
                    <TargetStepButton key={stage} value={stage} lockedReason={stageLockedReason(stage)} title={stageTitle(stage)} sizeClassName="size-9 sm:size-9">
                        <span className="font-semibold text-xs sm:text-[13px]">{stage === 0 ? "-" : stage}</span>
                    </TargetStepButton>
                );
            })}
        </TargetRow>
    );
}
