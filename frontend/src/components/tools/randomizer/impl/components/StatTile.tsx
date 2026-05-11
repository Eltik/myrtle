import type React from "react";
import { cn } from "#/lib/utils";

interface IStatTileProps {
    label: string;
    value: number | string;
    sub?: string;
    accent?: "default" | "warning";
}

export function StatTile({ label, value, sub, accent = "default" }: IStatTileProps): React.ReactElement {
    return (
        <div className={cn("min-w-0 rounded-lg border border-border/60 bg-card/60 px-3 py-2 not-dark:bg-clip-padding", accent === "warning" && "border-warning/40 bg-warning/8")}>
            <p className="font-mono text-[10px] uppercase tracking-[0.18em] text-muted-foreground/80">{label}</p>
            <p className="mt-1 truncate font-[var(--font-display)] text-[20px] font-semibold leading-none tracking-tight text-foreground sm:text-[22px]">{value}</p>
            {sub && <p className="mt-0.5 truncate font-mono text-[10.5px] tracking-tight text-muted-foreground">{sub}</p>}
        </div>
    );
}
