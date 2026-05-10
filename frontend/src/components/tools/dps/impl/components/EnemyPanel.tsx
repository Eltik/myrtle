import { ChevronDown, ChevronUp } from "lucide-react";
import * as React from "react";
import { Card, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { Collapsible, CollapsibleContent } from "#/components/ui/collapsible";
import { Label } from "#/components/ui/label";
import { NumberField, NumberFieldDecrement, NumberFieldGroup, NumberFieldIncrement, NumberFieldInput } from "#/components/ui/number-field";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import type { IDpsCalculateShred } from "#/lib/api/dps";
import { cn } from "#/lib/utils";
import { ENEMY_PRESETS } from "../constants";
import type { IEnemyConfig } from "../types";

interface IEnemyPanelProps {
    enemy: IEnemyConfig;
    /** Bumped once after `localStorage` hydration so uncontrolled NumberFields remount with the persisted values. */
    hydrationToken: number;
    onChangeEnemy: (patch: Partial<IEnemyConfig>) => void;
    onChangeShred: (patch: Partial<IDpsCalculateShred>) => void;
}

export function EnemyPanel({ enemy, hydrationToken, onChangeEnemy, onChangeShred }: IEnemyPanelProps): React.ReactElement {
    const hasShred = (enemy.shred.def ?? 0) > 0 || (enemy.shred.defFlat ?? 0) > 0 || (enemy.shred.res ?? 0) > 0 || (enemy.shred.resFlat ?? 0) > 0;
    const [shredOpen, setShredOpen] = React.useState(hasShred);
    // Bumped each time a preset is applied so the uncontrolled NumberFields
    // remount with new defaultValues. Manual edits don't change this counter,
    // so typing/incrementing keeps focus.
    const [presetVersion, setPresetVersion] = React.useState(0);

    const activePreset = ENEMY_PRESETS.find((p) => p.defense === enemy.defense && p.res === enemy.res && p.targets === enemy.targets);

    const onApplyPreset = React.useCallback(
        (preset: (typeof ENEMY_PRESETS)[number]) => {
            onChangeEnemy({ defense: preset.defense, res: preset.res, targets: preset.targets });
            setPresetVersion((v) => v + 1);
        },
        [onChangeEnemy],
    );

    return (
        <Card>
            <CardHeader>
                <CardTitle className="text-[15px]">Enemy & target</CardTitle>
                <p className="text-[12px] text-muted-foreground">DEF and RES set the KPI snapshot, and the axis you're not sweeping is held at this value.</p>
            </CardHeader>
            <CardPanel className="space-y-3 pt-0">
                <div>
                    <Label className="mb-1.5 block font-medium text-[11px] leading-none text-muted-foreground">Quick presets</Label>
                    <div className="flex flex-wrap gap-1">
                        {ENEMY_PRESETS.map((preset) => {
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
                                                    "flex h-7 cursor-pointer items-center justify-center rounded-md border bg-card px-2 text-[11px] font-medium transition-colors",
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
                    <NumField key={`def-${hydrationToken}-${presetVersion}`} label="Snapshot DEF" value={enemy.defense} min={0} max={5000} step={100} onChange={(v) => onChangeEnemy({ defense: v })} />
                    <NumField key={`res-${hydrationToken}-${presetVersion}`} label="Snapshot RES (%)" value={enemy.res} min={0} max={100} step={5} onChange={(v) => onChangeEnemy({ res: v })} />
                    <NumField key={`targets-${hydrationToken}-${presetVersion}`} label="Targets hit (AoE)" value={enemy.targets} min={1} max={20} step={1} onChange={(v) => onChangeEnemy({ targets: Math.max(1, v) })} />
                    <NumField key={`sp-${hydrationToken}`} label="Bonus SP/s" value={enemy.spBoost} min={0} max={5} step={0.1} onChange={(v) => onChangeEnemy({ spBoost: v })} />
                </div>

                <Collapsible open={shredOpen} onOpenChange={setShredOpen}>
                    <button type="button" onClick={() => setShredOpen((v) => !v)} className="flex w-full cursor-pointer items-center justify-between rounded py-1 text-left font-medium text-[11.5px] text-muted-foreground hover:text-foreground">
                        <span className="flex items-center gap-1.5">
                            Shred
                            {hasShred && <span className="rounded bg-primary/15 px-1.5 py-0.5 font-mono text-[9.5px] text-primary">on</span>}
                        </span>
                        {shredOpen ? <ChevronUp className="size-3.5" /> : <ChevronDown className="size-3.5" />}
                    </button>
                    <CollapsibleContent>
                        <div className="mt-2 grid grid-cols-2 gap-3">
                            <NumField key={`shred-def-${hydrationToken}`} label="DEF % shred" value={enemy.shred.def ?? 0} min={0} max={100} step={5} onChange={(v) => onChangeShred({ def: v })} compact />
                            <NumField key={`shred-defFlat-${hydrationToken}`} label="DEF flat" value={enemy.shred.defFlat ?? 0} min={0} max={3000} step={50} onChange={(v) => onChangeShred({ defFlat: v })} compact />
                            <NumField key={`shred-res-${hydrationToken}`} label="RES % shred" value={enemy.shred.res ?? 0} min={0} max={100} step={5} onChange={(v) => onChangeShred({ res: v })} compact />
                            <NumField key={`shred-resFlat-${hydrationToken}`} label="RES flat" value={enemy.shred.resFlat ?? 0} min={0} max={100} step={5} onChange={(v) => onChangeShred({ resFlat: v })} compact />
                        </div>
                    </CollapsibleContent>
                </Collapsible>
            </CardPanel>
        </Card>
    );
}

interface NumFieldProps {
    label: string;
    value: number;
    min?: number;
    max?: number;
    step?: number;
    onChange: (value: number) => void;
    compact?: boolean;
}

function NumField({ label, value, min, max, step = 1, onChange, compact = false }: NumFieldProps): React.ReactElement {
    return (
        <NumberField defaultValue={value} min={min} max={max} step={step} onValueChange={(v) => onChange(v ?? 0)} size={compact ? "sm" : "default"}>
            <Label className="block font-medium text-[12px] leading-none text-muted-foreground">{label}</Label>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
    );
}
