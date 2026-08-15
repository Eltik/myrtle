import { Fragment } from "react";
import { Combobox, ComboboxCollection, ComboboxEmpty, ComboboxGroup, ComboboxGroupLabel, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxSeparator } from "frontend";
import { Search } from "lucide-react";

interface IGroup {
    key: string;
    label: string;
    items: string[];
}

const BY_CLASS: IGroup[] = [
    { key: "guard", label: "Guard", items: ["Mlynar", "Thorns", "Chongyue"] },
    { key: "sniper", label: "Sniper", items: ["Exusiai", "Pozemka", "Rosmontis"] },
    { key: "medic", label: "Medic", items: ["Kal'tsit", "Shining", "Ptilopsis"] },
];

const BY_NATION: IGroup[] = [
    { key: "kazimierz", label: "Kazimierz", items: ["Mlynar", "Nearl the Radiant Knight", "Blemishine"] },
    { key: "lungmen", label: "Lungmen", items: ["Texas the Omertosa", "Ch.en the Holungday", "Swire the Elegant Wit"] },
];

const Grouped = ({ separated }: { separated: boolean }) => (
    <ComboboxList>
        {(group: IGroup) => (
            <Fragment key={group.key}>
                <ComboboxGroup items={group.items}>
                    <ComboboxGroupLabel>{group.label}</ComboboxGroupLabel>
                    <ComboboxCollection>
                        {(name: string) => (
                            <ComboboxItem key={name} value={name}>
                                {name}
                            </ComboboxItem>
                        )}
                    </ComboboxCollection>
                </ComboboxGroup>
                {separated && <ComboboxSeparator />}
            </Fragment>
        )}
    </ComboboxList>
);

export const GroupedByClass = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="group-class">
            Add an operator
        </label>
        <Combobox<string, false> defaultOpen items={BY_CLASS}>
            <ComboboxInput id="group-class" placeholder="Search 231 operators..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching operators.</ComboboxEmpty>
                <Grouped separated={false} />
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const WithSeparators = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="group-sep">
            Add an operator
        </label>
        <Combobox<string, false> defaultOpen items={BY_CLASS}>
            <ComboboxInput id="group-sep" placeholder="Search 231 operators..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching operators.</ComboboxEmpty>
                <Grouped separated />
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const GroupedMultiSelect = () => (
    <div className="h-96 w-80 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Squad shortlist</span>
        <Combobox<string, true> defaultOpen multiple defaultValue={["Mlynar", "Texas the Omertosa"]} items={BY_NATION}>
            <ComboboxInput placeholder="2 selected" startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching operators.</ComboboxEmpty>
                <Grouped separated />
            </ComboboxPopup>
        </Combobox>
    </div>
);
