import type * as React from "react";
import { Autocomplete, AutocompleteInput, AutocompleteItem, AutocompleteList, Label } from "frontend";
import { PackageIcon, SearchIcon } from "lucide-react";

const Panel = ({ children }: { children: React.ReactNode }) => <div className="mt-2 overflow-hidden rounded-lg border bg-popover shadow-xs">{children}</div>;

export const WithValue = () => (
    <div className="w-80">
        <Autocomplete defaultValue="Skadi the Corrupting Heart" mode="none">
            <AutocompleteInput showClear startAddon={<SearchIcon />} />
        </Autocomplete>
    </div>
);

export const Small = () => (
    <div className="w-64">
        <Autocomplete defaultValue="Orirock Cube" mode="none">
            <AutocompleteInput showClear size="sm" startAddon={<PackageIcon />} />
        </Autocomplete>
    </div>
);

export const InFilterBar = () => (
    <div className="flex w-96 flex-col gap-1.5 rounded-xl border bg-card p-4">
        <Label htmlFor="roster-filter">Filter roster</Label>
        <Autocomplete defaultValue="Texas" mode="none" open>
            <AutocompleteInput id="roster-filter" showClear startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteItem value="texas">Texas</AutocompleteItem>
                    <AutocompleteItem value="texas2">Texas the Omertosa</AutocompleteItem>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
        <p className="text-muted-foreground text-xs">Clear the field to show all 231 owned operators.</p>
    </div>
);
