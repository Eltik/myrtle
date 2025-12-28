"use client";

import { Info } from "lucide-react";
import { Label } from "~/components/ui/shadcn/label";
import { Switch } from "~/components/ui/shadcn/switch";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import type { OperatorSortMode } from "./types";

interface FilterOptionsProps {
    includeRobots: boolean;
    onIncludeRobotsChange: (value: boolean) => void;
    operatorSortMode: OperatorSortMode;
    onOperatorSortModeChange: (value: OperatorSortMode) => void;
}

export function FilterOptions({ includeRobots, onIncludeRobotsChange, operatorSortMode, onOperatorSortModeChange }: FilterOptionsProps) {
    return (
        <div className="flex flex-wrap items-center gap-6">
            <div className="flex items-center gap-2">
                <Switch checked={includeRobots} id="include-robots" onCheckedChange={onIncludeRobotsChange} />
                <Label className="cursor-pointer text-sm" htmlFor="include-robots">
                    Include Robots
                </Label>
            </div>

            <div className="flex items-center gap-2">
                <Switch checked={operatorSortMode === "common-first"} id="common-first" onCheckedChange={(checked) => onOperatorSortModeChange(checked ? "common-first" : "rarity-desc")} />
                <Label className="cursor-pointer text-sm" htmlFor="common-first">
                    Show Common First
                </Label>
                <Tooltip>
                    <TooltipTrigger asChild>
                        <Info className="h-3.5 w-3.5 cursor-help text-muted-foreground" />
                    </TooltipTrigger>
                    <TooltipContent side="top" sideOffset={4} variant="dark">
                        <p>Shows common operators (4★→3★→2★) before uncommon ones (5★→1★)</p>
                    </TooltipContent>
                </Tooltip>
            </div>
        </div>
    );
}
