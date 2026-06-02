import { ChevronDownIcon } from "lucide-react";
import type * as React from "react";
import { Select, SelectContent, SelectItem, SelectPrimitive, SelectValue, selectTriggerIconClassName, selectTriggerVariants } from "#/components/ui/select";
import { Switch } from "#/components/ui/switch";
import { cn } from "#/lib/utils";
import type { ICalculatorOptions, OperatorSortMode } from "../types";

interface ICalculatorOptionsPanelProps {
    options: Required<ICalculatorOptions>;
    onChangeIncludeRobots: (value: boolean) => void;
    onChangePrioritizeFiveStar: (value: boolean) => void;
    onChangeSortMode: (value: OperatorSortMode) => void;
}

const SORT_LABELS: Record<OperatorSortMode, string> = {
    "rarity-desc": "Highest rarity first",
    "common-first": "Most common first",
};

export function CalculatorOptionsPanel({ options, onChangeIncludeRobots, onChangePrioritizeFiveStar, onChangeSortMode }: ICalculatorOptionsPanelProps): React.ReactElement {
    return (
        <div className="flex flex-col gap-3">
            <ToggleRow id="recruit-include-robots" label="Include robots" checked={options.includeRobots} onCheckedChange={onChangeIncludeRobots} />
            <ToggleRow id="recruit-prioritize-five" label="Prioritize 5★ chance" hint="Surface tag combos that can yield a 5★ first" checked={options.prioritizeFiveStarChance} onCheckedChange={onChangePrioritizeFiveStar} />
            <div className="flex flex-col gap-1.5 border-border/60 border-t pt-3">
                <label htmlFor="recruit-sort" className="font-medium text-foreground text-sm">
                    Sort operators
                </label>
                <Select value={options.operatorSortMode} onValueChange={(v) => onChangeSortMode(v as OperatorSortMode)}>
                    <SelectPrimitive.Trigger id="recruit-sort" data-slot="select-trigger" className={cn(selectTriggerVariants({ size: "sm" }), "w-fit min-w-0")}>
                        <SelectValue placeholder="Sort">{(v: string) => SORT_LABELS[v as OperatorSortMode]}</SelectValue>
                        <SelectPrimitive.Icon data-slot="select-icon">
                            <ChevronDownIcon className={selectTriggerIconClassName} />
                        </SelectPrimitive.Icon>
                    </SelectPrimitive.Trigger>
                    <SelectContent>
                        {(Object.keys(SORT_LABELS) as OperatorSortMode[]).map((mode) => (
                            <SelectItem key={mode} value={mode}>
                                {SORT_LABELS[mode]}
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
