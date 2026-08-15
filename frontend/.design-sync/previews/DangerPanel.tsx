import { DangerPanel } from "frontend";

export const Default = () => <DangerPanel />;

/** DangerPanel takes no props; the only axis is the surface it sits on. */
export const OnMutedSurface = () => (
    <div className="rounded-2xl border border-sidebar-border bg-muted/30 p-4">
        <DangerPanel />
    </div>
);
