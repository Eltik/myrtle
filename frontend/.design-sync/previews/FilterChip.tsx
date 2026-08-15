import { FilterChip } from "frontend";

const noop = () => undefined;

export const EnemyStatusFilters = () => (
    <div className="flex flex-wrap items-center gap-2">
        <FilterChip active={true} count={312} label="All" onSelect={noop} />
        <FilterChip active={false} count={247} label="Encountered" onSelect={noop} />
        <FilterChip active={false} count={65} label="Missing" onSelect={noop} />
    </div>
);

export const InventoryCategories = () => (
    <div className="flex flex-wrap items-center gap-2">
        <FilterChip active={false} count={1284} label="All" onSelect={noop} />
        <FilterChip active={false} count={42} label="EXP" onSelect={noop} />
        <FilterChip active={true} count={863} label="Materials" onSelect={noop} />
        <FilterChip active={false} count={118} label="Skill Books" onSelect={noop} />
        <FilterChip active={false} count={64} label="Module Mats" onSelect={noop} />
        <FilterChip active={false} count={97} label="Chips" onSelect={noop} />
    </div>
);

export const WithoutCounts = () => (
    <div className="flex flex-wrap items-center gap-2">
        <FilterChip active={true} label="All Tiers" onSelect={noop} />
        <FilterChip active={false} label="Normal" onSelect={noop} />
        <FilterChip active={false} label="Elite" onSelect={noop} />
        <FilterChip active={false} label="Boss" onSelect={noop} />
    </div>
);

export const MultipleActive = () => (
    <div className="flex flex-wrap items-center gap-2">
        <FilterChip active={true} count={38} label="Vanguard" onSelect={noop} />
        <FilterChip active={false} count={54} label="Guard" onSelect={noop} />
        <FilterChip active={true} count={29} label="Sniper" onSelect={noop} />
        <FilterChip active={false} count={31} label="Caster" onSelect={noop} />
        <FilterChip active={false} count={26} label="Defender" onSelect={noop} />
        <FilterChip active={false} count={22} label="Medic" onSelect={noop} />
    </div>
);
