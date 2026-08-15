import type * as React from "react";
import { Autocomplete, AutocompleteInput, AutocompleteItem, AutocompleteList, AutocompleteTrigger, Button } from "frontend";
import { ChevronsUpDownIcon, SearchIcon } from "lucide-react";

const Panel = ({ children }: { children: React.ReactNode }) => <div className="mt-2 overflow-hidden rounded-lg border bg-popover shadow-xs">{children}</div>;

export const InsideInput = () => (
    <div className="w-80">
        <Autocomplete mode="none">
            <AutocompleteInput placeholder="Filter by operator" showTrigger startAddon={<SearchIcon />} />
        </Autocomplete>
    </div>
);

export const ButtonTrigger = () => (
    <div className="flex w-96 items-end gap-2">
        <Autocomplete mode="none">
            <AutocompleteInput className="flex-1" placeholder="Search operators…" startAddon={<SearchIcon />} />
            <AutocompleteTrigger render={<Button variant="outline" />}>
                Browse all
                <ChevronsUpDownIcon />
            </AutocompleteTrigger>
        </Autocomplete>
    </div>
);

export const OpenList = () => (
    <div className="w-80">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Filter by operator" showTrigger startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteItem value="mlynar">Młynar</AutocompleteItem>
                    <AutocompleteItem value="mlyss">Muelsyse</AutocompleteItem>
                    <AutocompleteItem value="texas2">Texas the Omertosa</AutocompleteItem>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);
