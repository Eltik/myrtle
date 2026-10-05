import type * as React from "react";

import { Switch } from "#/components/ui/switch";
import { cn } from "#/lib/utils";

interface ISwitchRowProps {
    id: string;
    label: string;
    description: string;
    checked: boolean;
    onCheckedChange: (checked: boolean) => void;
    className?: string;
}

/** A labelled on/off setting in its own card. */
export function SwitchRow({ id, label, description, checked, onCheckedChange, className }: ISwitchRowProps): React.ReactElement {
    return (
        <div className={cn("flex items-center justify-between rounded-xl border border-border bg-card/40 p-4", className)}>
            <div className="space-y-0.5">
                <label htmlFor={id} className="cursor-pointer font-semibold text-foreground text-sm">
                    {label}
                </label>
                <p className="text-muted-foreground text-xs">{description}</p>
            </div>
            <Switch id={id} checked={checked} onCheckedChange={onCheckedChange} />
        </div>
    );
}
