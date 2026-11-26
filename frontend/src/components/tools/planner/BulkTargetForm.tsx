import type * as React from "react";

import { eliteIcon, specializedIcon } from "#/components/operators/detail/impl/assets";
import { Input } from "#/components/ui/input";
import { Kicker } from "#/components/ui/kicker";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import type { IPresetTarget } from "#/lib/api/planner";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./BulkPlanDialog.messages";
import { presetLevelCap, withPresetChange } from "./bulkTargets";
import type { messages as dialogMessages } from "./OperatorPlannerDialog.messages";
import { STEP_TOGGLE } from "./PlanTargetSections";
import { ELITE_PHASES, MAX_SKILL_LEVEL, MODULE_STAGES } from "./planTargets";

type BulkT = TypedT<typeof messages & typeof dialogMessages>;

const SKILL_LEVELS = Array.from({ length: MAX_SKILL_LEVEL }, (_, i) => i + 1);
const MASTERY_RANKS = [0, 1, 2, 3] as const;
const SKILL_SLOTS = [0, 1, 2] as const;

interface IBulkTargetFormProps {
    target: IPresetTarget;
    onChange: (target: IPresetTarget) => void;
}

/** The rarity-free target: promotion, level, shared skill level, a mastery per skill slot and one module stage. */
export function BulkTargetForm({ target, onChange }: IBulkTargetFormProps): React.ReactElement {
    const t: BulkT = useT("tools");
    const cap = presetLevelCap(target.elite);

    const changeLevel = (raw: number) => onChange(withPresetChange(target, { field: "level", value: Number.isNaN(raw) ? 1 : raw }));

    return (
        <div className="space-y-5 rounded-2xl border border-border bg-card/40 p-4">
            <Kicker className="block">{t("planner.bulk.target")}</Kicker>

            <div className="grid grid-cols-1 gap-5 sm:grid-cols-[auto_1fr]">
                <FieldBlock label={t("planner.dialog.promotion")}>
                    <ToggleGroup aria-label={t("planner.dialog.promotion")} className="gap-2" onValueChange={(next) => next[0] !== undefined && onChange(withPresetChange(target, { field: "elite", value: Number(next[0]) }))} value={[String(target.elite)]} variant="outline">
                        {ELITE_PHASES.map((e) => (
                            <Tooltip key={e}>
                                <TooltipTrigger render={<ToggleGroupItem className={cn(STEP_TOGGLE, "size-12 sm:size-12")} value={String(e)} />}>
                                    <img src={eliteIcon(e)} alt={t("planner.dialog.elite", { elite: e })} className="icon-theme-aware size-7 object-contain" />
                                </TooltipTrigger>
                                <TooltipPopup>{t("planner.dialog.elite", { elite: e })}</TooltipPopup>
                            </Tooltip>
                        ))}
                    </ToggleGroup>
                </FieldBlock>

                <FieldBlock label={t("planner.bulk.level")}>
                    <Input type="number" min={1} max={cap} value={target.level ?? cap} onChange={(e) => changeLevel(Number.parseInt(e.target.value, 10))} className="w-20 text-center font-mono" aria-label={t("planner.bulk.level")} />
                    <p className="text-muted-foreground text-xs">{t("planner.bulk.levelCapNote", { max: cap })}</p>
                </FieldBlock>
            </div>

            <FieldBlock label={t("planner.bulk.skillLevel")}>
                <ToggleGroup aria-label={t("planner.bulk.skillLevel")} className="flex-wrap gap-1.5" onValueChange={(next) => next[0] !== undefined && onChange(withPresetChange(target, { field: "skill_level", value: Number(next[0]) }))} value={[String(target.skill_level)]} variant="outline">
                    {SKILL_LEVELS.map((value) => (
                        <StepButton key={value} title={t("planner.dialog.skillLevel", { level: value })} value={value}>
                            <span className="font-semibold text-[13px]">{value}</span>
                        </StepButton>
                    ))}
                </ToggleGroup>
            </FieldBlock>

            <FieldBlock label={t("planner.bulk.masteries")}>
                <div className="space-y-1.5">
                    {SKILL_SLOTS.map((slot) => (
                        <div key={slot} className="flex items-center gap-3">
                            <span className="w-6 font-semibold text-muted-foreground text-xs">{t("planner.bulk.skillSlot", { index: slot + 1 })}</span>
                            <ToggleGroup
                                aria-label={t("planner.bulk.masteryGroup", { index: slot + 1 })}
                                className="gap-1.5"
                                onValueChange={(next) => next[0] !== undefined && onChange(withPresetChange(target, { field: "mastery", index: slot, value: Number(next[0]) }))}
                                value={[String(target.masteries[slot])]}
                                variant="outline"
                            >
                                {MASTERY_RANKS.map((rank) => (
                                    <StepButton key={rank} title={rank === 0 ? t("planner.dialog.notPlanned") : t("planner.dialog.mastery", { mastery: rank })} value={rank}>
                                        {rank === 0 ? <span className="font-semibold text-[13px]">-</span> : <img src={specializedIcon(rank)} alt={t("planner.dialog.mastery", { mastery: rank })} className="icon-theme-aware size-6 object-contain" />}
                                    </StepButton>
                                ))}
                            </ToggleGroup>
                        </div>
                    ))}
                </div>
            </FieldBlock>

            <FieldBlock label={t("planner.bulk.moduleStage")}>
                <ToggleGroup aria-label={t("planner.bulk.moduleStage")} className="gap-1.5" onValueChange={(next) => next[0] !== undefined && onChange(withPresetChange(target, { field: "module_stage", value: Number(next[0]) }))} value={[String(target.module_stage)]} variant="outline">
                    {MODULE_STAGES.map((stage) => (
                        <StepButton key={stage} title={stage === 0 ? t("planner.dialog.notPlanned") : t("planner.dialog.stage", { stage })} value={stage}>
                            <span className="font-semibold text-[13px]">{stage === 0 ? "-" : stage}</span>
                        </StepButton>
                    ))}
                </ToggleGroup>
            </FieldBlock>

            <p className="text-muted-foreground text-xs">{t("planner.bulk.targetNote")}</p>
        </div>
    );
}

function FieldBlock({ label, children }: { label: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex flex-col gap-2">
            <span className="font-medium text-[13px] text-muted-foreground leading-none">{label}</span>
            {children}
        </div>
    );
}

interface IStepButtonProps {
    title: string;
    /** The step's number; its group holds the selection. */
    value: number;
    children: React.ReactNode;
}

function StepButton({ title, value, children }: IStepButtonProps): React.ReactElement {
    return (
        <Tooltip>
            <TooltipTrigger render={<ToggleGroupItem aria-label={title} className={cn(STEP_TOGGLE, "size-9 sm:size-9")} value={String(value)} />}>{children}</TooltipTrigger>
            <TooltipPopup>{title}</TooltipPopup>
        </Tooltip>
    );
}
