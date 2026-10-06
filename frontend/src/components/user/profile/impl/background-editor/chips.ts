import { cn } from "#/lib/utils";

/** A count after a label: smaller, monospaced and muted, so the label reads first. */
export const COUNT = "font-mono text-[10.5px] tabular-nums opacity-55";

/** A filter chip: a toggle pill, outlined off, tinted on. */
export const chip = (on: boolean) =>
    cn(
        "inline-flex h-7 shrink-0 cursor-pointer items-center gap-1.5 rounded-full border px-2.5 font-medium font-sans text-xs leading-none outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring",
        on ? "border-primary/50 bg-primary/12 text-foreground" : "border-border text-muted-foreground hover:bg-accent hover:text-foreground",
    );

/** A source tab of the browser's segmented row: icon, label and muted count, the chosen one raised. */
export const sourceTab = (active: boolean) =>
    cn(
        "inline-flex h-8 shrink-0 cursor-pointer items-center gap-2 rounded-md px-3 font-medium font-sans text-[13px] leading-none outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring [&_svg]:size-4 [&_svg]:opacity-75",
        active ? "bg-background text-foreground shadow-sm ring-1 ring-border" : "text-muted-foreground hover:text-foreground",
    );

/** A toggle-group item whose pressed state reads at a glance: tinted with the primary, not the input grey. */
export const SEGMENT_ITEM = "data-pressed:border-primary/50 data-pressed:bg-primary/15 data-pressed:text-foreground dark:data-pressed:bg-primary/15";
