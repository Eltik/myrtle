import { SeparatorPrimitive } from "frontend";

/**
 * `SeparatorPrimitive` is the raw Base UI `Separator` re-export — it ships no
 * styles of its own, so every story supplies the rule's own size and colour.
 * `Separator` is the token-styled wrapper you normally want.
 */

export const HorizontalRule = () => (
    <div className="w-full max-w-sm rounded-lg border border-border bg-card p-4">
        <span className="block font-medium text-foreground text-sm">Base — Trading Post 1</span>
        <SeparatorPrimitive className="my-3 h-px w-full bg-border" />
        <div className="space-y-1 text-sm">
            <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Order efficiency</span>
                <span className="font-mono tabular-nums">+45%</span>
            </div>
            <div className="flex items-center justify-between">
                <span className="text-muted-foreground">LMD / day</span>
                <span className="font-mono tabular-nums">1,520</span>
            </div>
        </div>
    </div>
);

export const VerticalRule = () => (
    <div className="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3 text-sm">
        <span className="font-mono tabular-nums">4,812</span>
        <span className="text-muted-foreground text-xs">sanity</span>
        <SeparatorPrimitive className="h-5 w-px bg-border" orientation="vertical" />
        <span className="font-mono tabular-nums">231</span>
        <span className="text-muted-foreground text-xs">operators</span>
        <SeparatorPrimitive className="h-5 w-px bg-border" orientation="vertical" />
        <span className="font-mono tabular-nums">87</span>
        <span className="text-muted-foreground text-xs">E2</span>
    </div>
);

export const CustomEmphasis = () => (
    <div className="w-full max-w-sm rounded-lg border border-border bg-card p-4">
        <span className="block font-medium text-foreground text-sm">Danger zone</span>
        <p className="mt-1 text-muted-foreground text-xs">Deleting a roster snapshot cannot be undone.</p>
        <SeparatorPrimitive className="my-3 h-px w-full bg-destructive/40" />
        <p className="text-destructive text-xs">3 snapshots will be removed.</p>
    </div>
);
