import { Fragment } from "react";
import { Combobox, ComboboxCollection, ComboboxEmpty, ComboboxGroup, ComboboxGroupLabel, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxSeparator } from "frontend";
import { Search } from "lucide-react";

interface IGroup {
    key: string;
    label: string;
    items: string[];
}

const BY_RARITY: IGroup[] = [
    { key: "6", label: "6★", items: ["Mlynar", "Eyjafjalla", "Kal'tsit"] },
    { key: "5", label: "5★", items: ["Lappland", "Ptilopsis", "Cerberus"] },
    { key: "4", label: "4★", items: ["Myrtle", "Gummy", "Perfumer"] },
];

const BY_ROOM: IGroup[] = [
    { key: "factory", label: "Manufacturing station", items: ["Originium Shard", "LMD", "Battle Record"] },
    { key: "trading", label: "Trading post", items: ["Order — LMD", "Order — Originium"] },
    { key: "dorm", label: "Dormitory", items: ["Morale recovery", "Ambience"] },
];

export const RarityHeaders = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="label-rarity">
            Add an operator
        </label>
        <Combobox<string, false> defaultOpen items={BY_RARITY}>
            <ComboboxInput id="label-rarity" placeholder="Search 231 operators..." startAddon={<Search />} />
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
    </div>
);

export const CountedHeaders = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="label-counted">
            Base facility output
        </label>
        <Combobox<string, false> defaultOpen items={BY_ROOM}>
            <ComboboxInput id="label-counted" placeholder="Search facilities..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching facilities.</ComboboxEmpty>
                <ComboboxList>
                    {(group: IGroup) => (
                        <Fragment key={group.key}>
                            <ComboboxGroup items={group.items}>
                                <ComboboxGroupLabel className="flex items-center justify-between">
                                    <span>{group.label}</span>
                                    <span className="font-mono text-[10px] tabular-nums">{group.items.length}</span>
                                </ComboboxGroupLabel>
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

export const SingleGroup = () => (
    <div className="h-72 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="label-single">
            Add an operator
        </label>
        <Combobox<string, false> defaultOpen items={[BY_RARITY[0]]}>
            <ComboboxInput id="label-single" placeholder="Search 231 operators..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching operators.</ComboboxEmpty>
                <ComboboxList>
                    {(group: IGroup) => (
                        <ComboboxGroup key={group.key} items={group.items}>
                            <ComboboxGroupLabel>Recently viewed — {group.label}</ComboboxGroupLabel>
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
    </div>
);
