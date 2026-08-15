import type * as React from "react";
import { Autocomplete, AutocompleteEmpty, AutocompleteInput, AutocompleteItem, AutocompleteList } from "frontend";
import { SearchIcon } from "lucide-react";

const OPERATORS = ["Mlynar", "Skadi the Corrupting Heart", "Muelsyse", "Texas the Omertosa", "Eyjafjalla"];

const Panel = ({ children }: { children: React.ReactNode }) => <div className="mt-2 overflow-hidden rounded-lg border bg-popover shadow-xs">{children}</div>;

export const NoMatches = () => (
    <div className="w-80">
        <Autocomplete defaultValue="Amyia" items={OPERATORS} open>
            <AutocompleteInput startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>{(name: string) => <AutocompleteItem key={name} value={name}>{name}</AutocompleteItem>}</AutocompleteList>
                <AutocompleteEmpty>No operators match "Amyia".</AutocompleteEmpty>
            </Panel>
        </Autocomplete>
    </div>
);

export const EmptyRoster = () => (
    <div className="w-80">
        <Autocomplete items={[]} open>
            <AutocompleteInput placeholder="Search your roster…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>{(name: string) => <AutocompleteItem key={name} value={name}>{name}</AutocompleteItem>}</AutocompleteList>
                <AutocompleteEmpty className="py-6">
                    <div className="font-medium text-foreground text-sm">No operators synced</div>
                    <div className="mt-1 text-xs">Link a Yostar account to search your roster.</div>
                </AutocompleteEmpty>
            </Panel>
        </Autocomplete>
    </div>
);

export const WithMatches = () => (
    <div className="w-80">
        <Autocomplete defaultValue="the" items={OPERATORS} open>
            <AutocompleteInput startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>{(name: string) => <AutocompleteItem key={name} value={name}>{name}</AutocompleteItem>}</AutocompleteList>
                <AutocompleteEmpty>No operators match that name.</AutocompleteEmpty>
            </Panel>
        </Autocomplete>
    </div>
);
