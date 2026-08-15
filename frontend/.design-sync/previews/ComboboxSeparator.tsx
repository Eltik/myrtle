import { Fragment } from "react";
import { Combobox, ComboboxCollection, ComboboxEmpty, ComboboxGroup, ComboboxGroupLabel, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxSeparator } from "frontend";
import { Search } from "lucide-react";

interface IGroup {
    key: string;
    label: string;
    items: string[];
}

const GROUPS: IGroup[] = [
    { key: "recent", label: "Recently planned", items: ["Mlynar", "Skadi the Corrupting Heart"] },
    { key: "owned", label: "In your roster", items: ["Myrtle", "Ptilopsis", "Lappland"] },
    { key: "missing", label: "Not recruited", items: ["Ling", "Dusk"] },
];

export const BetweenGroups = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="sep-groups">
            Add an operator
        </label>
        <Combobox<string, false> defaultOpen items={GROUPS}>
            <ComboboxInput id="sep-groups" placeholder="Search 231 operators..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching operators.</ComboboxEmpty>
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
                            <ComboboxSeparator />
                        </Fragment>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">The trailing separator hides itself, so the last group never ends on a rule.</p>
    </div>
);

export const WithoutSeparators = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="sep-none">
            Add an operator
        </label>
        <Combobox<string, false> defaultOpen items={GROUPS}>
            <ComboboxInput id="sep-none" placeholder="Search 231 operators..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching operators.</ComboboxEmpty>
                <ComboboxList>
                    {(group: IGroup) => (
                        <ComboboxGroup key={group.key} items={group.items}>
                            <ComboboxGroupLabel>{group.label}</ComboboxGroupLabel>
                            <ComboboxCollection>
                                {(name: string) => (
                                    <ComboboxItem key={name} value={name}>
                                        {name}
                                    </ComboboxItem>
                                )}
                            </ComboboxCollection>
                        </ComboboxGroup>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">Group spacing alone reads fine for short lists.</p>
    </div>
);

export const MultiSelectGroups = () => (
    <div className="h-96 w-80 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Squad shortlist</span>
        <Combobox<string, true> defaultOpen multiple defaultValue={["Mlynar", "Ptilopsis"]} items={GROUPS}>
            <ComboboxInput placeholder="2 selected" startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching operators.</ComboboxEmpty>
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
                            <ComboboxSeparator />
                        </Fragment>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);
