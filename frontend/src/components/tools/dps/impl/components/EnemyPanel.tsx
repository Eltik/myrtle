import * as React from "react";
import { Card, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { Label } from "#/components/ui/label";
import { NumberField, NumberFieldDecrement, NumberFieldGroup, NumberFieldIncrement, NumberFieldInput } from "#/components/ui/number-field";
import { Switch } from "#/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { ENEMY_PRESETS } from "../constants";
import type { messages as dpsConstantsMessages } from "../constants.messages";
import type { IEnemyConfig, ISecondaryConfig } from "../types";
import type { messages } from "./EnemyPanel.messages";

/** Own chrome plus the preset labels from `constants.ts`. */
type EnemyT = TypedT<typeof messages & typeof dpsConstantsMessages>;

interface IEnemyPanelProps {
    enemy: IEnemyConfig;
    /** Bumped once after `localStorage` hydration so uncontrolled NumberFields remount with the persisted values. */
    hydrationToken: number;
    onChangeEnemy: (patch: Partial<Omit<IEnemyConfig, "secondary">>) => void;
    onChangeSecondary: (patch: Partial<ISecondaryConfig>) => void;
}

export function EnemyPanel({ enemy, hydrationToken, onChangeEnemy, onChangeSecondary }: IEnemyPanelProps): React.ReactElement {
    const t: EnemyT = useT("tools");
    const sec = enemy.secondary;
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
                <CardTitle className="text-[15px]">{t("dps.enemy.title")}</CardTitle>
                <p className="text-[12px] text-muted-foreground">{t("dps.enemy.desc")}</p>
            </CardHeader>
            <CardPanel className="space-y-3 pt-0">
                <div>
                    <h3 className="mb-2 font-semibold text-foreground text-sm leading-none">{t("dps.enemy.presets")}</h3>
                    <ToggleGroup
                        aria-label={t("dps.enemy.presets")}
                        size="sm"
                        value={activePreset ? [activePreset.id] : []}
                        onValueChange={(v) => {
                            const preset = ENEMY_PRESETS.find((p) => p.id === v[0]);
                            if (preset) onApplyPreset(preset);
                        }}
                        className="flex-wrap gap-1"
                    >
                        {ENEMY_PRESETS.map((preset) => (
                            <Tooltip key={preset.id}>
                                <TooltipTrigger
                                    render={(triggerProps) => (
                                        <ToggleGroupItem {...triggerProps} value={preset.id} className="border-input text-[12px]">
                                            {t(preset.labelKey)}
                                        </ToggleGroupItem>
                                    )}
                                />
                                <TooltipPopup>{t(preset.summaryKey)}</TooltipPopup>
                            </Tooltip>
                        ))}
                    </ToggleGroup>
                </div>

                <div className="grid grid-cols-2 gap-3">
                    <NumField key={`def-${hydrationToken}-${presetVersion}`} label={t("dps.enemy.snapshotDef")} value={enemy.defense} min={0} max={5000} step={100} onChange={(v) => onChangeEnemy({ defense: v })} />
                    <NumField key={`res-${hydrationToken}-${presetVersion}`} label={t("dps.enemy.snapshotRes")} value={enemy.res} min={0} max={100} step={5} onChange={(v) => onChangeEnemy({ res: v })} />
                    <NumField key={`targets-${hydrationToken}-${presetVersion}`} label={t("dps.enemy.targets")} value={enemy.targets} min={1} max={20} step={1} disabled={sec.enabled} onChange={(v) => onChangeEnemy({ targets: Math.max(1, v) })} />
                </div>

                <div className="space-y-2 sm:rounded-md sm:border sm:border-border sm:bg-muted/30 sm:px-2.5 sm:py-2">
                    <div className="flex items-center justify-between gap-2">
                        <span id="dps-secondary-label" className="font-semibold text-foreground text-sm">
                            {t("dps.enemy.secondary")}
                        </span>
                        <Switch aria-labelledby="dps-secondary-label" checked={sec.enabled} onCheckedChange={(enabled) => onChangeSecondary({ enabled })} />
                    </div>
                    <p className="text-[11px] text-muted-foreground leading-snug">{sec.enabled ? t("dps.enemy.secondary.targetsNote") : t("dps.enemy.secondary.desc")}</p>
                    {sec.enabled && (
                        <div className="grid grid-cols-3 gap-2">
                            <NumField key={`sec-def-${hydrationToken}`} label={t("dps.enemy.secondary.def")} value={sec.def} min={0} max={5000} step={100} onChange={(v) => onChangeSecondary({ def: v })} compact />
                            <NumField key={`sec-res-${hydrationToken}`} label={t("dps.enemy.secondary.res")} value={sec.res} min={0} max={100} step={5} onChange={(v) => onChangeSecondary({ res: v })} compact />
                            <NumField key={`sec-count-${hydrationToken}`} label={t("dps.enemy.secondary.count")} value={sec.count} min={1} max={49} step={1} onChange={(v) => onChangeSecondary({ count: Math.max(1, Math.round(v)) })} compact />
                        </div>
                    )}
                </div>

                <div className="space-y-1.5">
                    <h3 className="font-semibold text-foreground text-sm leading-none">{t("dps.enemy.cycle")}</h3>
                    <ToggleGroup aria-label={t("dps.enemy.cycle")} variant="outline" size="sm" value={[enemy.cycleModel]} onValueChange={(v) => v[0] && onChangeEnemy({ cycleModel: v[0] === "v2" ? "v2" : "legacy" })}>
                        <ToggleGroupItem value="legacy" className="text-[12px]">
                            {t("dps.enemy.cycle.legacy")}
                        </ToggleGroupItem>
                        <ToggleGroupItem value="v2" className="text-[12px]">
                            {t("dps.enemy.cycle.v2")}
                        </ToggleGroupItem>
                    </ToggleGroup>
                    <p className="text-[11px] text-muted-foreground leading-snug">{enemy.cycleModel === "v2" ? t("dps.enemy.cycle.desc.v2") : t("dps.enemy.cycle.desc.legacy")}</p>
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
    compact?: boolean;
    disabled?: boolean;
}

function NumField({ label, value, min, max, step = 1, onChange, compact = false, disabled = false }: INumFieldProps): React.ReactElement {
    return (
        <NumberField defaultValue={value} min={min} max={max} step={step} onValueChange={(v) => onChange(v ?? 0)} size={compact ? "sm" : "default"} disabled={disabled}>
            <Label className="block text-subtle-foreground text-xs leading-none">{label}</Label>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
    );
}
