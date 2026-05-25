import * as React from "react";
import { Card, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { Label } from "#/components/ui/label";
import { NumberField, NumberFieldDecrement, NumberFieldGroup, NumberFieldIncrement, NumberFieldInput } from "#/components/ui/number-field";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import type { IHpsCalculateBuffs } from "#/lib/api/hps";
import { cn } from "#/lib/utils";
import { BUFF_PRESETS } from "../constants";
import type { IHpsBuffConfig } from "../types";

interface IBuffsPanelProps {
    buffs: IHpsBuffConfig;
    /** Bumped once after `localStorage` hydration so uncontrolled NumberFields remount with the persisted values. */
    hydrationToken: number;
    onChangeBuffs: (patch: Partial<IHpsBuffConfig>) => void;
    onChangeBuffValues: (patch: Partial<IHpsCalculateBuffs>) => void;
}

export function BuffsPanel({ buffs, hydrationToken, onChangeBuffs, onChangeBuffValues }: IBuffsPanelProps): React.ReactElement {
    // Bumped each time a preset is applied so the uncontrolled NumberFields
    // remount with new defaultValues. Manual edits don't change this counter,
    // so typing/incrementing keeps focus.
    const [presetVersion, setPresetVersion] = React.useState(0);

    const atkPct = Math.round((buffs.buffs.atk ?? 0) * 100);
    const aspd = buffs.buffs.aspd ?? 0;

    const activePreset = BUFF_PRESETS.find((p) => Math.round(p.atk * 100) === atkPct && p.aspd === aspd && p.targets === buffs.targets);

    const onApplyPreset = React.useCallback(
        (preset: (typeof BUFF_PRESETS)[number]) => {
            onChangeBuffValues({ atk: preset.atk, aspd: preset.aspd });
            onChangeBuffs({ targets: preset.targets });
            setPresetVersion((v) => v + 1);
        },
        [onChangeBuffs, onChangeBuffValues],
    );

    return (
        <Card>
            <CardHeader>
                <CardTitle className="text-[15px]">Team buffs & targets</CardTitle>
                <p className="text-[12px] text-muted-foreground">Healing ignores enemy DEF/RES, so these team-side buffs drive output. The axis you're sweeping is held at this value for the snapshot.</p>
            </CardHeader>
            <CardPanel className="space-y-3 pt-0">
                <div>
                    <Label className="mb-1.5 block font-medium text-[11px] text-muted-foreground leading-none">Quick presets</Label>
                    <div className="flex flex-wrap gap-1">
                        {BUFF_PRESETS.map((preset) => {
                            const selected = activePreset?.id === preset.id;
                            return (
                                <Tooltip key={preset.id}>
                                    <TooltipTrigger
                                        render={(triggerProps) => (
                                            <button
                                                {...triggerProps}
                                                type="button"
                                                onClick={() => onApplyPreset(preset)}
                                                aria-pressed={selected}
                                                className={cn(
                                                    "flex h-7 cursor-pointer items-center justify-center rounded-md border bg-card px-2 font-medium text-[11px] transition-colors",
                                                    selected ? "border-primary bg-primary/10 text-foreground ring-1 ring-primary/30" : "border-border text-muted-foreground hover:border-primary/50 hover:text-foreground",
                                                )}
                                            >
                                                {preset.label}
                                            </button>
                                        )}
                                    />
                                    <TooltipPopup>{preset.summary}</TooltipPopup>
                                </Tooltip>
                            );
                        })}
                    </div>
                </div>

                <div className="grid grid-cols-2 gap-3">
                    <NumField key={`atk-${hydrationToken}-${presetVersion}`} label="ATK %" value={atkPct} min={0} max={400} step={5} onChange={(v) => onChangeBuffValues({ atk: v / 100 })} />
                    <NumField key={`flatAtk-${hydrationToken}-${presetVersion}`} label="Flat ATK" value={buffs.buffs.flatAtk ?? 0} min={0} max={2000} step={10} onChange={(v) => onChangeBuffValues({ flatAtk: v })} />
                    <NumField key={`aspd-${hydrationToken}-${presetVersion}`} label="ASPD" value={aspd} min={0} max={200} step={5} onChange={(v) => onChangeBuffValues({ aspd: v })} />
                    <NumField key={`heal-${hydrationToken}-${presetVersion}`} label="Heal amp %" value={Math.round((buffs.buffs.fragile ?? 0) * 100)} min={0} max={200} step={5} onChange={(v) => onChangeBuffValues({ fragile: v / 100 })} />
                    <NumField key={`targets-${hydrationToken}-${presetVersion}`} label="Targets healed" value={buffs.targets} min={1} max={12} step={1} onChange={(v) => onChangeBuffs({ targets: Math.max(1, v) })} />
                    <NumField key={`sp-${hydrationToken}`} label="Bonus SP/s" value={buffs.spBoost} min={0} max={5} step={0.1} onChange={(v) => onChangeBuffs({ spBoost: v })} />
                </div>
            </CardPanel>
        </Card>
    );
}

interface INumFieldProps {
    label: string;
    value: number;
    min?: number;
    max?: number;
    step?: number;
    onChange: (value: number) => void;
}

function NumField({ label, value, min, max, step = 1, onChange }: INumFieldProps): React.ReactElement {
    return (
        <NumberField defaultValue={value} min={min} max={max} step={step} onValueChange={(v) => onChange(v ?? 0)} size="default">
            <Label className="block font-medium text-[12px] text-muted-foreground leading-none">{label}</Label>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
    );
}
