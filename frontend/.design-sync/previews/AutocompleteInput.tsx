import { Autocomplete, AutocompleteInput } from "frontend";
import { SearchIcon, SwordsIcon } from "lucide-react";

export const Default = () => (
    <div className="w-80">
        <Autocomplete mode="none">
            <AutocompleteInput placeholder="Search operators…" />
        </Autocomplete>
    </div>
);

export const WithAddonAndTrigger = () => (
    <div className="w-80">
        <Autocomplete mode="none">
            <AutocompleteInput placeholder="Filter by operator" showTrigger startAddon={<SearchIcon />} />
        </Autocomplete>
    </div>
);

export const WithClear = () => (
    <div className="w-80">
        <Autocomplete defaultValue="Skadi the Corrupting Heart" mode="none">
            <AutocompleteInput showClear startAddon={<SwordsIcon />} />
        </Autocomplete>
    </div>
);

export const Sizes = () => (
    <div className="flex w-80 flex-col gap-3">
        <Autocomplete mode="none">
            <AutocompleteInput placeholder="Small — filter stages" showTrigger size="sm" startAddon={<SearchIcon />} />
        </Autocomplete>
        <Autocomplete mode="none">
            <AutocompleteInput placeholder="Default — filter stages" showTrigger startAddon={<SearchIcon />} />
        </Autocomplete>
        <Autocomplete mode="none">
            <AutocompleteInput placeholder="Large — filter stages" showTrigger size="lg" startAddon={<SearchIcon />} />
        </Autocomplete>
    </div>
);

export const Disabled = () => (
    <div className="w-80">
        <Autocomplete defaultValue="Muelsyse" disabled mode="none">
            <AutocompleteInput showTrigger startAddon={<SearchIcon />} />
        </Autocomplete>
    </div>
);
