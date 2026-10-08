import { RemovedBadge } from "frontend";

// RemovedBadge is the small destructive-tinted mono chip the profile Showcase
// puts on a block or entity whose target is gone (a deleted grid, tier list
// or plan). Only the owner ever sees one; the block it marks is dropped on the
// next save. The removed-block row below is how ShowcaseTab composes it.

/** The chip alone. */
export const Badge = () => <RemovedBadge />;

/** As the Showcase draws a removed block: dashed, muted, the badge leading the note. */
export const RemovedBlockRow = () => (
    <div style={{ width: 520 }}>
        <div className="flex h-full items-start gap-3 rounded-2xl border border-border border-dashed bg-muted/20 px-4 py-3">
            <RemovedBadge />
            <p className="m-0 text-muted-foreground text-sm">This tier list was deleted. Visitors don't see this block; remove it in the editor.</p>
        </div>
    </div>
);

/** In an editor row header, after the block's title. */
export const InEditorRow = () => (
    <div style={{ width: 520 }} className="flex items-center gap-2 rounded-xl border bg-card px-3 py-2.5">
        <span className="min-w-0 flex-1 truncate font-medium text-sm">Operation Lucent Arrowhead — CC picks</span>
        <RemovedBadge />
    </div>
);
