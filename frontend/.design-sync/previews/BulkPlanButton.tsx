import { BulkPlanButton } from "frontend";

// The planner toolbar's "Bulk add" button. It owns the open state of the bulk
// dialog it mounts, so the page only renders this. Size and variant follow
// the toolbar (small outline by default).

/** The toolbar default: small, outline. */
export const Default = () => <BulkPlanButton />;

/** The size axis: xs, the toolbar sm, and default. */
export const Sizes = () => (
    <div className="flex items-center gap-3">
        <BulkPlanButton size="xs" />
        <BulkPlanButton />
        <BulkPlanButton size="default" />
    </div>
);

/** The secondary variant some hosts use. */
export const Secondary = () => <BulkPlanButton variant="secondary" />;
