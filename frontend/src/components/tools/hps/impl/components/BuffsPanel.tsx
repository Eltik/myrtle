import * as React from "react";
import { Card, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { Label } from "#/components/ui/label";
import { NumberField, NumberFieldDecrement, NumberFieldGroup, NumberFieldIncrement, NumberFieldInput } from "#/components/ui/number-field";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import type { IHpsCalculateBuffs } from "#/lib/api/hps";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { BUFF_PRESETS } from "../constants";
import type { messages as hpsConstantsMessages } from "../constants.messages";
import type { IHpsBuffConfig } from "../types";
import type { messages } from "./BuffsPanel.messages";

type BuffsT = TypedT<typeof messages & typeof hpsConstantsMessages>;

interface IBuffsPanelProps {
    buffs: IHpsBuffConfig;
    /** Bumped once after `localStorage` hydration so uncontrolled NumberFields remount with the persisted values. */
    hydrationToken: number;
    onChangeBuffs: (patch: Partial<IHpsBuffConfig>) => void;
    onChangeBuffValues: (patch: Partial<IHpsCalculateBuffs>) => void;
}

export function BuffsPanel({ buffs, hydrationToken, onChangeBuffs, onChangeBuffValues }: IBuffsPanelProps): React.ReactElement {
    const t: BuffsT = useT("tools");
    // Bumped per preset so the uncontrolled NumberFields remount with new defaults;
    // manual edits leave it alone so typing keeps focus.
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
                <CardTitle className="text-[15px]">{t("hps.buffs.title")}</CardTitle>
                <p className="text-[12px] text-muted-foreground">{t("hps.buffs.desc")}</p>
            </CardHeader>
            <CardPanel className="space-y-3 pt-0">
                <div>
                    <h3 className="mb-2 font-semibold text-foreground text-sm leading-none">{t("hps.buffs.presets")}</h3>
                    <ToggleGroup
                        aria-label={t("hps.buffs.presets")}
                        size="sm"
                        value={activePreset ? [activePreset.id] : []}
                        onValueChange={(v) => {
                            const preset = BUFF_PRESETS.find((p) => p.id === v[0]);
                            if (preset) onApplyPreset(preset);
                        }}
                        className="flex-wrap gap-1"
                    >
                        {BUFF_PRESETS.map((preset) => (
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
                    <NumField key={`atk-${hydrationToken}-${presetVersion}`} label={t("hps.buffs.atkPct")} value={atkPct} min={0} max={400} step={5} onChange={(v) => onChangeBuffValues({ atk: v / 100 })} />
                    <NumField key={`flatAtk-${hydrationToken}-${presetVersion}`} label={t("hps.buffs.flatAtk")} value={buffs.buffs.flatAtk ?? 0} min={0} max={2000} step={10} onChange={(v) => onChangeBuffValues({ flatAtk: v })} />
                    <NumField key={`aspd-${hydrationToken}-${presetVersion}`} label={t("hps.buffs.aspd")} value={aspd} min={0} max={200} step={5} onChange={(v) => onChangeBuffValues({ aspd: v })} />
                    <NumField key={`heal-${hydrationToken}-${presetVersion}`} label={t("hps.buffs.healAmp")} value={Math.round((buffs.buffs.fragile ?? 0) * 100)} min={0} max={200} step={5} onChange={(v) => onChangeBuffValues({ fragile: v / 100 })} />
                    <NumField key={`targets-${hydrationToken}-${presetVersion}`} label={t("hps.buffs.targets")} value={buffs.targets} min={1} max={12} step={1} onChange={(v) => onChangeBuffs({ targets: Math.max(1, v) })} />
                    <NumField key={`sp-${hydrationToken}`} label={t("hps.buffs.spBoost")} value={buffs.spBoost} min={0} max={5} step={0.1} onChange={(v) => onChangeBuffs({ spBoost: v })} />
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
            <Label className="block text-subtle-foreground text-xs leading-none">{label}</Label>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
    );
}
