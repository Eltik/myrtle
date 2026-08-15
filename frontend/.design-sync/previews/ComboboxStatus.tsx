import { Combobox, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxStatus } from "frontend";
import { Search } from "lucide-react";

const OPERATORS = ["Mlynar", "Skadi the Corrupting Heart", "Muelsyse", "Texas the Omertosa", "Eyjafjalla", "Exusiai", "Kal'tsit", "Myrtle"];

export const ResultCount = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="status-count">
            Add an operator
        </label>
        <Combobox<string, false> defaultOpen items={OPERATORS}>
            <ComboboxInput id="status-count" placeholder="Search 231 operators..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxStatus>Showing 8 of 231 operators</ComboboxStatus>
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

export const Loading = () => (
    <div className="h-72 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="status-loading">
            Add an operator
        </label>
        <Combobox<string, false> defaultOpen items={[]}>
            <ComboboxInput id="status-loading" placeholder="Loading operators..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxStatus>Loading the operator index…</ComboboxStatus>
                <ComboboxEmpty />
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

export const TruncatedResults = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="status-truncated">
            Add an operator
        </label>
        <Combobox<string, false> defaultOpen items={OPERATORS}>
            <ComboboxInput id="status-truncated" placeholder="Search 231 operators..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxStatus>Best matches first — keep typing to narrow</ComboboxStatus>
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
