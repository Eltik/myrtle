import type * as React from "react";
import { Autocomplete, AutocompleteInput, AutocompleteItem, AutocompleteList, AutocompleteStatus } from "frontend";
import { SearchIcon } from "lucide-react";

const Panel = ({ children }: { children: React.ReactNode }) => <div className="mt-2 overflow-hidden rounded-lg border bg-popover shadow-xs">{children}</div>;

export const ResultCount = () => (
    <div className="w-80">
        <Autocomplete defaultValue="ori" mode="none" open>
            <AutocompleteInput showClear startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteStatus>3 of 96 materials</AutocompleteStatus>
                <AutocompleteList>
                    <AutocompleteItem value="orirock">Orirock</AutocompleteItem>
                    <AutocompleteItem value="orirock-cube">Orirock Cube</AutocompleteItem>
                    <AutocompleteItem value="oriron">Oriron</AutocompleteItem>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const Loading = () => (
    <div className="w-80">
        <Autocomplete defaultValue="mly" mode="none" open>
            <AutocompleteInput startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteStatus>Searching 342 operators…</AutocompleteStatus>
            </Panel>
        </Autocomplete>
    </div>
);

export const TruncatedResults = () => (
    <div className="w-80">
        <Autocomplete defaultValue="a" mode="none" open>
            <AutocompleteInput showClear startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteItem value="amiya">Amiya</AutocompleteItem>
                    <AutocompleteItem value="angelina">Angelina</AutocompleteItem>
                    <AutocompleteItem value="ansel">Ansel</AutocompleteItem>
                    <AutocompleteItem value="aak">Aak</AutocompleteItem>
                </AutocompleteList>
                <AutocompleteStatus className="border-t">Showing 4 of 61 matches — keep typing to narrow</AutocompleteStatus>
            </Panel>
        </Autocomplete>
    </div>
);
