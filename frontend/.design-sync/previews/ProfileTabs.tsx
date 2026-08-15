import { ProfileTabs } from "frontend";

const TABS = [
    { id: "stats", label: "Stats" },
    { id: "score", label: "Score" },
    { id: "roster", label: "Roster", count: 231 },
    { id: "plans", label: "Plans", count: 4 },
    { id: "inventory", label: "Inventory", count: 418 },
    { id: "enemies", label: "Enemies", count: 187 },
];

const NO_COUNTS = [
    { id: "stats", label: "Stats" },
    { id: "score", label: "Score" },
    { id: "roster", label: "Roster" },
    { id: "plans", label: "Plans" },
    { id: "inventory", label: "Inventory" },
    { id: "enemies", label: "Enemies" },
];

const noop = () => {};

export const StatsActive = () => <ProfileTabs active="stats" onChange={noop} tabs={TABS} />;

export const RosterActive = () => <ProfileTabs active="roster" onChange={noop} tabs={TABS} />;

export const BeforeCountsLoad = () => <ProfileTabs active="score" onChange={noop} tabs={NO_COUNTS} />;
