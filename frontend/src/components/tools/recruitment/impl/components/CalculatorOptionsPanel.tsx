import type * as React from "react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Switch } from "#/components/ui/switch";
import type { ICalculatorOptions, OperatorSortMode } from "../types";

interface ICalculatorOptionsPanelProps {
    options: Required<ICalculatorOptions>;
    onChangeShowLowRarity: (value: boolean) => void;
    onChangeIncludeRobots: (value: boolean) => void;
    onChangeSortMode: (value: OperatorSortMode) => void;
}

const SORT_LABELS: Record<OperatorSortMode, string> = {
    "rarity-desc": "Highest rarity first",
    "common-first": "Most common first",
};

export function CalculatorOptionsPanel({ options, onChangeShowLowRarity, onChangeIncludeRobots, onChangeSortMode }: ICalculatorOptionsPanelProps): React.ReactElement {
    return (
        <div className="flex flex-col gap-3">
            <ToggleRow id="recruit-show-low" label="Show 1★ / 2★ operators" checked={options.showLowRarity} onCheckedChange={onChangeShowLowRarity} />
            <ToggleRow id="recruit-include-robots" label="Include robots" checked={options.includeRobots} onCheckedChange={onChangeIncludeRobots} />
            <div className="flex items-center justify-between gap-3">
                <label htmlFor="recruit-sort" className="text-foreground text-sm">
                    Sort operators
                </label>
                <Select value={options.operatorSortMode} onValueChange={(v) => onChangeSortMode(v as OperatorSortMode)}>
                    <SelectTrigger id="recruit-sort" size="sm" className="min-w-44">
                        <SelectValue placeholder="Sort">{(v: string) => SORT_LABELS[v as OperatorSortMode]}</SelectValue>
                    </SelectTrigger>
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
    checked: boolean;
    onCheckedChange: (value: boolean) => void;
}

function ToggleRow({ id, label, checked, onCheckedChange }: IToggleRowProps): React.ReactElement {
    return (
        <label htmlFor={id} className="flex cursor-pointer items-center justify-between gap-3 text-foreground text-sm">
            <span>{label}</span>
            <Switch id={id} checked={checked} onCheckedChange={onCheckedChange} />
        </label>
    );
}
