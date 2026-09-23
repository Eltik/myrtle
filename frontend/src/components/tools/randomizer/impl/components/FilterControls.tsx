import { Lock } from "lucide-react";
import type React from "react";
import { Switch } from "#/components/ui/switch";
import { cn } from "#/lib/utils";

/** A labelled block of controls inside a settings tab. */
export function FieldGroup({ label, children }: { label: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex flex-col gap-2.5">
            <p className="font-mono text-[10.5px] text-muted-foreground/90 uppercase tracking-[0.18em]">{label}</p>
            {children}
        </div>
    );
}

interface ISwitchRowProps {
    label: string;
    description: string;
    checked: boolean;
    onChange: (v: boolean) => void;
    /** Renders off and disabled with a lock, whatever `checked` says: the filter needs data this session lacks (a profile, the stage clears). */
    locked?: boolean;
}

/** A boolean setting: label and description on the left, the switch on the right, the whole row clickable. */
export function SwitchRow({ label, description, checked, onChange, locked = false }: ISwitchRowProps): React.ReactElement {
    return (
        // biome-ignore lint/a11y/noLabelWithoutControl: Switch is a Base UI primitive; wrapping label provides click target and is correctly associated at runtime
        <label className={cn("flex items-start justify-between gap-3 rounded-md border border-border/50 bg-card/60 px-3 py-2.5 transition-colors hover:bg-accent/30", locked && "cursor-not-allowed opacity-60 hover:bg-card/60")}>
            <div className="min-w-0 flex-1">
                <p className="flex items-center gap-1.5 font-medium text-[12.5px] text-foreground">
                    {label}
                    {locked && <Lock aria-hidden="true" className="h-3 w-3 text-muted-foreground/70" />}
                </p>
                <p className="mt-0.5 text-[11.5px] text-muted-foreground leading-snug">{description}</p>
            </div>
            <Switch checked={locked ? false : checked} disabled={locked} onCheckedChange={onChange} />
        </label>
    );
}
