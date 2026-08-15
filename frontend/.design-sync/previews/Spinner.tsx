import { Card, CardHeader, CardPanel, CardTitle, Spinner } from "frontend";

/** Size is set with a utility class — the sizes the app actually uses. */
export const Sizes = () => (
    <div className="flex items-center gap-6">
        <Spinner className="size-3 text-muted-foreground" />
        <Spinner className="size-4 text-muted-foreground" />
        <Spinner className="size-5" />
        <Spinner className="size-8 text-primary" />
    </div>
);

/** Inline status, as the DPS chart overlays it while a run recalculates. */
export const InlineStatus = () => (
    <div className="flex flex-col items-start gap-3">
        <span className="inline-flex items-center gap-1.5 rounded-md bg-muted/50 px-2 py-1 text-[11px] text-muted-foreground">
            <Spinner className="size-3" />
            Calculating
        </span>
        <span className="inline-flex items-center gap-2 text-muted-foreground text-sm">
            <Spinner className="size-4" />
            Fetching depot from the EN server…
        </span>
    </div>
);

/** A KPI tile whose value has not arrived yet. */
export const StatPlaceholder = () => (
    <div className="grid w-full max-w-2xl gap-4 sm:grid-cols-2">
        <Card>
            <CardHeader>
                <CardTitle>Sanity spent</CardTitle>
            </CardHeader>
            <CardPanel className="flex items-center gap-2 font-mono text-2xl text-primary tabular-nums">
                <Spinner className="size-5 text-muted-foreground" />
            </CardPanel>
        </Card>
        <Card>
            <CardHeader>
                <CardTitle>Operators owned</CardTitle>
            </CardHeader>
            <CardPanel className="font-mono text-2xl tabular-nums">231</CardPanel>
        </Card>
    </div>
);

/** Centred in a panel that owns the whole loading state. */
export const PanelLoading = () => (
    <div className="flex h-40 w-full max-w-md flex-col items-center justify-center gap-3 rounded-xl border border-dashed bg-muted/20">
        <Spinner className="size-6 text-muted-foreground" />
        <span className="text-muted-foreground text-sm">Loading Chapter 8 stage list…</span>
    </div>
);
