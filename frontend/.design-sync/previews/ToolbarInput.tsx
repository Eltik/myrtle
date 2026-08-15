import { Button, Input, Toolbar, ToolbarButton, ToolbarInput, ToolbarSeparator } from "frontend";
import { FilterIcon, SearchIcon } from "lucide-react";

/** A search field inside the toolbar, rendered as the DS's Input. */
export const SearchField = () => (
    <Toolbar className="w-fit" aria-label="Roster">
        <ToolbarInput render={<Input className="w-64" placeholder="Search 231 operators…" size="sm" type="search" />} />
        <ToolbarSeparator />
        <ToolbarButton aria-label="Filters" render={<Button size="icon-sm" variant="ghost" />}>
            <FilterIcon />
        </ToolbarButton>
    </Toolbar>
);

/** Pre-filled, with the matching action beside it. */
export const WithValue = () => (
    <Toolbar className="w-fit" aria-label="Requirements">
        <ToolbarInput render={<Input className="w-64" defaultValue="Bipolar Nanoflake" size="sm" type="search" />} />
        <ToolbarSeparator />
        <ToolbarButton render={<Button size="sm" variant="outline" />}>
            <SearchIcon />
            Find stages
        </ToolbarButton>
    </Toolbar>
);

/** Disabled while the depot sync is running. */
export const DisabledInput = () => (
    <Toolbar className="w-fit" aria-label="Depot">
        <ToolbarInput disabled render={<Input className="w-64" placeholder="Syncing depot…" size="sm" type="search" />} />
        <ToolbarSeparator />
        <ToolbarButton disabled render={<Button size="sm" variant="outline" />}>
            Clear
        </ToolbarButton>
    </Toolbar>
);
