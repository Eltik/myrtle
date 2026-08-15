import { EditHero } from "frontend";

const TITLE = "Global 6★ meta — May 2024";

const DESCRIPTION =
    "The team's standing ranking of every 6★ operator on Global, rebuilt after each banner. Placements assume E2 level 60, module stage 2 and no potential above 1.\n\nTiers describe how much of a stage an operator can solve on their own, not raw damage numbers.";

const PENDING = [
    { kind: "placement-add" as const, label: "Placed Logos in S+" },
    { kind: "placement-move" as const, label: "Moved Degenbrecher from S to S+" },
    { kind: "tier-update" as const, label: "Renamed tier B to Budget" },
    { kind: "placement-desc" as const, label: "Edited note on Wiš'adel" },
    { kind: "placement-remove" as const, label: "Removed Rosmontis from A" },
    { kind: "tier-move" as const, label: "Moved tier A above B" },
    { kind: "title-desc" as const, label: "List details" },
];

const noop = () => {};

const handlers = {
    onTitleChange: noop,
    onDescriptionChange: noop,
    onSave: noop,
    onReset: noop,
    onAddTier: noop,
};

export const AllSaved = () => <EditHero slug="global-6-star-meta" title={TITLE} description={DESCRIPTION} pendingChanges={[]} saving={false} saveError={null} saveProgress={null} {...handlers} />;

export const UnsavedChanges = () => <EditHero slug="global-6-star-meta" title={TITLE} description={DESCRIPTION} pendingChanges={PENDING} saving={false} saveError={null} saveProgress={null} {...handlers} />;

export const Saving = () => <EditHero slug="global-6-star-meta" title={TITLE} description={DESCRIPTION} pendingChanges={PENDING} saving saveError={null} saveProgress={{ step: 3, total: 7, label: "Updating placements" }} {...handlers} />;

export const SaveFailed = () => <EditHero slug="global-6-star-meta" title={TITLE} description={DESCRIPTION} pendingChanges={PENDING.slice(0, 3)} saving={false} saveError="Couldn't save tier B — the tier was deleted by another session. Reload and try again." saveProgress={null} {...handlers} />;
