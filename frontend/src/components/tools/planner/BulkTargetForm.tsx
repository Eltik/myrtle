import type * as React from "react";

import { eliteIcon, specializedIcon } from "#/components/operators/detail/impl/assets";
import { Input } from "#/components/ui/input";
import type { IPresetTarget } from "#/lib/api/planner";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./BulkPlanDialog.messages";
import { presetLevelCap, withPresetChange } from "./bulkTargets";
import type { messages as dialogMessages } from "./OperatorPlannerDialog.messages";
import { stepStateClass } from "./PlanTargetSections";
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
        <div className="space-y-5 rounded-xl border border-border bg-card/40 p-4">
            <span className="font-semibold text-muted-foreground text-xs uppercase tracking-wider">{t("planner.bulk.target")}</span>

            <div className="grid grid-cols-1 gap-5 sm:grid-cols-[auto_1fr]">
                <FieldBlock label={t("planner.dialog.promotion")}>
                    <div className="flex items-center gap-2">
                        {ELITE_PHASES.map((e) => (
                            <button
                                key={e}
                                type="button"
                                onClick={() => onChange(withPresetChange(target, { field: "elite", value: e }))}
                                className={cn("flex size-12 cursor-pointer items-center justify-center rounded-lg border transition-all", stepStateClass(target.elite === e))}
                                title={t("planner.dialog.elite", { elite: e })}
                            >
                                <img src={eliteIcon(e)} alt={t("planner.dialog.elite", { elite: e })} className="icon-theme-aware size-7 object-contain" />
                            </button>
                        ))}
                    </div>
                </FieldBlock>

                <FieldBlock label={t("planner.bulk.level")}>
                    <Input type="number" min={1} max={cap} value={target.level ?? cap} onChange={(e) => changeLevel(Number.parseInt(e.target.value, 10))} className="w-20 text-center font-mono" aria-label={t("planner.bulk.level")} />
                    <p className="text-muted-foreground text-xs">{t("planner.bulk.levelCapNote", { max: cap })}</p>
                </FieldBlock>
            </div>

            <FieldBlock label={t("planner.bulk.skillLevel")}>
                <div className="flex flex-wrap gap-1.5">
                    {SKILL_LEVELS.map((value) => (
                        <StepButton key={value} isActive={target.skill_level === value} title={t("planner.dialog.skillLevel", { level: value })} onSelect={() => onChange(withPresetChange(target, { field: "skill_level", value }))}>
                            <span className="font-semibold text-[13px]">{value}</span>
                        </StepButton>
                    ))}
                </div>
            </FieldBlock>

            <FieldBlock label={t("planner.bulk.masteries")}>
                <div className="space-y-1.5">
                    {SKILL_SLOTS.map((slot) => (
                        <div key={slot} className="flex items-center gap-3">
                            <span className="w-6 font-semibold text-muted-foreground text-xs">{t("planner.bulk.skillSlot", { index: slot + 1 })}</span>
                            <div className="flex gap-1.5">
                                {MASTERY_RANKS.map((rank) => (
                                    <StepButton key={rank} isActive={target.masteries[slot] === rank} title={rank === 0 ? t("planner.dialog.notPlanned") : t("planner.dialog.mastery", { mastery: rank })} onSelect={() => onChange(withPresetChange(target, { field: "mastery", index: slot, value: rank }))}>
                                        {rank === 0 ? <span className="font-semibold text-[13px]">-</span> : <img src={specializedIcon(rank)} alt={t("planner.dialog.mastery", { mastery: rank })} className="icon-theme-aware size-6 object-contain" />}
                                    </StepButton>
                                ))}
                            </div>
                        </div>
                    ))}
                </div>
            </FieldBlock>

            <FieldBlock label={t("planner.bulk.moduleStage")}>
                <div className="flex gap-1.5">
                    {MODULE_STAGES.map((stage) => (
                        <StepButton key={stage} isActive={target.module_stage === stage} title={stage === 0 ? t("planner.dialog.notPlanned") : t("planner.dialog.stage", { stage })} onSelect={() => onChange(withPresetChange(target, { field: "module_stage", value: stage }))}>
                            <span className="font-semibold text-[13px]">{stage === 0 ? "-" : stage}</span>
                        </StepButton>
                    ))}
                </div>
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
    isActive: boolean;
    title: string;
    onSelect: () => void;
    children: React.ReactNode;
}

function StepButton({ isActive, title, onSelect, children }: IStepButtonProps): React.ReactElement {
    return (
        <button type="button" onClick={onSelect} aria-pressed={isActive} className={cn("flex size-9 shrink-0 cursor-pointer items-center justify-center rounded-md border transition-all", stepStateClass(isActive))} title={title}>
            {children}
        </button>
    );
}
