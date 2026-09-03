import { FilterPanel } from "#/components/operators/list/impl/components/FilterPanel";
import { OperatorFilterFields, TagRow } from "#/components/operators/list/impl/components/OperatorFilterFields";
import type { ArrayFilterKey, IFilterOptions, ISharedFilters } from "#/components/operators/list/impl/types";
import { PROFILE_STICKY_OFFSET_PX } from "../../ProfileTabs";
import type { OwnershipFilter } from "./types";

interface IRosterFiltersProps {
    filters: ISharedFilters;
    options: IFilterOptions;
    onChange: <K extends ArrayFilterKey>(key: K, value: ISharedFilters[K]) => void;
    ownership: OwnershipFilter;
    onOwnershipChange: (v: OwnershipFilter) => void;
    onClearAll: () => void;
    hasActiveFilters: boolean;
    collapsed: boolean;
    onToggle: () => void;
    activeFilterCount: number;
}

const OWNERSHIP_OPTIONS: { value: OwnershipFilter; label: string }[] = [
    { value: "owned", label: "Owned" },
    { value: "unowned", label: "Unowned" },
    { value: "all", label: "All" },
];

export function RosterFilters(props: IRosterFiltersProps) {
    return (
        <FilterPanel collapsed={props.collapsed} onToggle={props.onToggle} hasActiveFilters={props.hasActiveFilters} onClearAll={props.onClearAll} activeFilterCount={props.activeFilterCount} ariaLabel="Roster filters" stickyOffset={PROFILE_STICKY_OFFSET_PX}>
            <OperatorFilterFields filters={props.filters} options={props.options} onChange={props.onChange} basicLeading={<TagRow label="Ownership" options={OWNERSHIP_OPTIONS} value={props.ownership} onChange={props.onOwnershipChange} />} />
        </FilterPanel>
    );
}
