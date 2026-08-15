import { Combobox, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup } from "frontend";
import { Search } from "lucide-react";

const OPERATORS = ["Mlynar", "Skadi the Corrupting Heart", "Muelsyse", "Texas the Omertosa", "Eyjafjalla", "Exusiai", "Kal'tsit", "Myrtle"];

export const NoMatches = () => (
    <div className="h-72 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="empty-basic">
            Add an operator
        </label>
        <Combobox<string, false> defaultOpen defaultInputValue="Nightingale" items={OPERATORS}>
            <ComboboxInput id="empty-basic" placeholder="Search your roster..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching operators.</ComboboxEmpty>
                <ComboboxList>
                    {(name: string) => (
                        <ComboboxItem key={name} value={name}>
                            {name}
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const WithHint = () => (
    <div className="h-72 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="empty-hint">
            Add an operator
        </label>
        <Combobox<string, false> defaultOpen defaultInputValue="Amiya (Guard)" items={OPERATORS}>
            <ComboboxInput id="empty-hint" placeholder="Search your roster..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>
                    <span className="block font-medium text-foreground text-sm">No operator matches “Amiya (Guard)”</span>
                    <span className="mt-1 block text-[11px] text-muted-foreground">Try the appellation, class, or a recruitment tag instead.</span>
                </ComboboxEmpty>
                <ComboboxList>
                    {(name: string) => (
                        <ComboboxItem key={name} value={name}>
                            {name}
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const EmptyRoster = () => (
    <div className="h-72 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="empty-roster">
            Add an operator
        </label>
        <Combobox<string, false> defaultOpen items={[]}>
            <ComboboxInput id="empty-roster" placeholder="Search your roster..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>Import your roster to pick from operators you own.</ComboboxEmpty>
                <ComboboxList>
                    {(name: string) => (
                        <ComboboxItem key={name} value={name}>
                            {name}
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const HiddenWhenResults = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="empty-results">
            Add an operator
        </label>
        <Combobox<string, false> defaultOpen items={OPERATORS}>
            <ComboboxInput id="empty-results" placeholder="Search your roster..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching operators.</ComboboxEmpty>
                <ComboboxList>
                    {(name: string) => (
                        <ComboboxItem key={name} value={name}>
                            {name}
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);
