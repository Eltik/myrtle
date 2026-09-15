import { ChevronDownIcon } from "lucide-react";
import type * as React from "react";
import { Select, SelectContent, SelectItem, SelectPrimitive, SelectValue, selectTriggerIconClassName, selectTriggerVariants } from "#/components/ui/select";
import { Switch } from "#/components/ui/switch";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { ICalculatorOptions, OperatorSortMode } from "../types";
import type { messages } from "./CalculatorOptionsPanel.messages";

interface ICalculatorOptionsPanelProps {
    options: Required<ICalculatorOptions>;
    onChangeIncludeRobots: (value: boolean) => void;
    onChangeIncludeTwoStars: (value: boolean) => void;
    onChangeIncludeThreeStars: (value: boolean) => void;
    onChangeSortMode: (value: OperatorSortMode) => void;
}

type OptionsT = TypedT<typeof messages>;

/** Sort order, with the message key each mode's label lives under. */
const SORT_MODES: readonly { value: OperatorSortMode; labelKey: keyof typeof messages & string }[] = [
    { value: "rarity-desc", labelKey: "recruit.options.sort.rarityDesc" },
    { value: "common-first", labelKey: "recruit.options.sort.commonFirst" },
];

export function CalculatorOptionsPanel({ options, onChangeIncludeRobots, onChangeIncludeTwoStars, onChangeIncludeThreeStars, onChangeSortMode }: ICalculatorOptionsPanelProps): React.ReactElement {
    const t: OptionsT = useT("tools");
    const sortLabel = (mode: OperatorSortMode) => {
        const entry = SORT_MODES.find((m) => m.value === mode);
        return entry ? t(entry.labelKey) : mode;
    };
    return (
        <div className="flex flex-col gap-3">
            <ToggleRow id="recruit-include-robots" label={t("recruit.options.includeRobots")} checked={options.includeRobots} onCheckedChange={onChangeIncludeRobots} />
            <ToggleRow id="recruit-include-two-stars" label={t("recruit.options.includeTwoStars")} checked={options.includeTwoStars} onCheckedChange={onChangeIncludeTwoStars} />
            <ToggleRow id="recruit-include-three-stars" label={t("recruit.options.includeThreeStars")} checked={options.includeThreeStars} onCheckedChange={onChangeIncludeThreeStars} />
            <div className="flex flex-col gap-1.5 border-border/60 border-t pt-3">
                <label htmlFor="recruit-sort" className="font-medium text-foreground text-sm">
                    {t("recruit.options.sort")}
                </label>
                <Select value={options.operatorSortMode} onValueChange={(v) => onChangeSortMode(v as OperatorSortMode)}>
                    <SelectPrimitive.Trigger id="recruit-sort" data-slot="select-trigger" className={cn(selectTriggerVariants({ size: "sm" }), "w-fit min-w-0")}>
                        <SelectValue placeholder={t("recruit.options.sort.placeholder")}>{(v: string) => sortLabel(v as OperatorSortMode)}</SelectValue>
                        <SelectPrimitive.Icon data-slot="select-icon">
                            <ChevronDownIcon className={selectTriggerIconClassName} />
                        </SelectPrimitive.Icon>
                    </SelectPrimitive.Trigger>
                    <SelectContent>
                        {SORT_MODES.map((mode) => (
                            <SelectItem key={mode.value} value={mode.value}>
                                {t(mode.labelKey)}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>
        </div>
    );
}

interface IToggleRowProps {
    id: string;
    label: string;
    hint?: string;
    checked: boolean;
    onCheckedChange: (value: boolean) => void;
}

function ToggleRow({ id, label, hint, checked, onCheckedChange }: IToggleRowProps): React.ReactElement {
    return (
        <label htmlFor={id} className="flex cursor-pointer items-center justify-between gap-3 text-foreground text-sm">
            <span className="flex min-w-0 flex-col">
                <span>{label}</span>
                {hint ? <span className="text-[11px] text-muted-foreground leading-snug">{hint}</span> : null}
            </span>
            <Switch id={id} checked={checked} onCheckedChange={onCheckedChange} />
        </label>
    );
}
