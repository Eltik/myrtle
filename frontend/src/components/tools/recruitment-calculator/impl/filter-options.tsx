"use client";

import { Label } from "~/components/ui/shadcn/label";
import { Switch } from "~/components/ui/shadcn/switch";

interface FilterOptionsProps {
    includeRobots: boolean;
    onIncludeRobotsChange: (value: boolean) => void;
}

export function FilterOptions({ includeRobots, onIncludeRobotsChange }: FilterOptionsProps) {
    return (
        <div className="flex flex-wrap items-center gap-6">
            <div className="flex items-center gap-2">
                <Switch checked={includeRobots} id="include-robots" onCheckedChange={onIncludeRobotsChange} />
                <Label className="cursor-pointer text-sm" htmlFor="include-robots">
                    Include Robots
                </Label>
            </div>
        </div>
    );
}
