import { Autocomplete, AutocompleteInput, AutocompleteItem, AutocompleteList, AutocompleteValue } from "frontend";
import { SearchIcon } from "lucide-react";

export const CurrentQuery = () => (
    <div className="w-96">
        <Autocomplete defaultValue="Skadi" mode="none">
            <AutocompleteInput showClear startAddon={<SearchIcon />} />
            <p className="mt-2 text-muted-foreground text-sm">
                Showing roster matches for <span className="font-medium font-mono text-foreground">{<AutocompleteValue />}</span>
            </p>
        </Autocomplete>
    </div>
);

export const EmptyQuery = () => (
    <div className="w-96">
        <Autocomplete mode="none">
            <AutocompleteInput placeholder="Search your roster…" startAddon={<SearchIcon />} />
            {/* An empty query still counts as "has value" in Base UI, so `placeholder` never fires — fall back in the children fn instead. */}
            <p className="mt-2 text-muted-foreground text-sm">
                Showing <span className="font-medium text-foreground">{<AutocompleteValue>{(query: string) => query || "all 231 owned operators"}</AutocompleteValue>}</span>
            </p>
        </Autocomplete>
    </div>
);

export const AboveResults = () => (
    <div className="w-96">
        <Autocomplete defaultValue="ori" mode="none" open>
            <AutocompleteInput showClear startAddon={<SearchIcon />} />
            <div className="mt-2 overflow-hidden rounded-lg border bg-popover shadow-xs">
                <div className="border-b px-3 py-2 text-muted-foreground text-xs">
                    3 materials match <span className="font-medium font-mono text-foreground">{<AutocompleteValue />}</span>
                </div>
                <AutocompleteList>
                    <AutocompleteItem value="orirock">Orirock</AutocompleteItem>
                    <AutocompleteItem value="orirock-cube">Orirock Cube</AutocompleteItem>
                    <AutocompleteItem value="oriron">Oriron</AutocompleteItem>
                </AutocompleteList>
            </div>
        </Autocomplete>
    </div>
);
