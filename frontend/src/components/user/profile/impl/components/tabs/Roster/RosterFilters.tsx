import { FilterPanel } from "#/components/operators/list/impl/components/FilterPanel";
import { OperatorFilterFields, TagRow } from "#/components/operators/list/impl/components/OperatorFilterFields";
import type { ArrayFilterKey, IFilterOptions, ISharedFilters } from "#/components/operators/list/impl/types";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { PROFILE_STICKY_OFFSET_PX } from "../../ProfileTabs";
import type { messages } from "./RosterFilters.messages";
import type { OwnershipFilter } from "./types";

/** A key in `RosterFilters.messages.ts`; resolved by the panel below. */
type MessageKey = keyof typeof messages & string;

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

const OWNERSHIP_OPTIONS: { value: OwnershipFilter; labelKey: MessageKey }[] = [
    { value: "owned", labelKey: "profile.roster.filters.owned" },
    { value: "unowned", labelKey: "profile.roster.filters.unowned" },
    { value: "all", labelKey: "profile.roster.filters.all" },
];

export function RosterFilters(props: IRosterFiltersProps) {
    const t: TypedT<typeof messages> = useT("user");
    const ownershipOptions = OWNERSHIP_OPTIONS.map((o) => ({ value: o.value, label: t(o.labelKey) }));
    return (
        <FilterPanel collapsed={props.collapsed} onToggle={props.onToggle} hasActiveFilters={props.hasActiveFilters} onClearAll={props.onClearAll} activeFilterCount={props.activeFilterCount} ariaLabel={t("profile.roster.filters.aria")} stickyOffset={PROFILE_STICKY_OFFSET_PX}>
            <OperatorFilterFields filters={props.filters} options={props.options} onChange={props.onChange} basicLeading={<TagRow label={t("profile.roster.filters.ownership")} options={ownershipOptions} value={props.ownership} onChange={props.onOwnershipChange} />} />
        </FilterPanel>
    );
}
