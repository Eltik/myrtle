import { StatsPageHeader } from "frontend";

// The capture clock is pinned to 2024-05-15T12:00Z, so this reads "19m ago".
const COMPUTED_AT = "2024-05-15T11:41:00Z";

export const Default = () => (
    <div className="px-8 pt-4">
        <StatsPageHeader computedAt={COMPUTED_AT} />
    </div>
);

export const NotYetComputed = () => (
    <div className="px-8 pt-4">
        <StatsPageHeader computedAt={null} />
    </div>
);
