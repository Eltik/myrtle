import { Fragment } from "react";
import { Combobox, ComboboxCollection, ComboboxEmpty, ComboboxGroup, ComboboxGroupLabel, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxSeparator, ComboboxStatus } from "frontend";
import { Search } from "lucide-react";

const MATERIALS = ["Orirock Cube", "Sugar Pack", "Polyester Pack", "Oriron Cluster", "Grindstone", "RMA70-12", "D32 Steel", "Bipolar Nanoflake", "Crystalline Electronic Unit"];

const GROUPS = [
    { key: "t5", label: "Tier 5 — Elite", items: ["D32 Steel", "Bipolar Nanoflake", "Crystalline Electronic Unit"] },
    { key: "t4", label: "Tier 4 — Advanced", items: ["Grindstone Pentahydrate", "RMA70-24", "Polymerization Preparation"] },
    { key: "t3", label: "Tier 3 — Common", items: ["Orirock Cluster", "Sugar Pack", "Oriron Cluster"] },
];

export const OpenResults = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="mat-open">
            Material to farm
        </label>
        <Combobox<string, false> defaultOpen items={MATERIALS}>
            <ComboboxInput id="mat-open" placeholder="Search materials..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching materials.</ComboboxEmpty>
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

export const GroupedResults = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="mat-grouped">
            Material to farm
        </label>
        <Combobox<string, true> defaultOpen multiple items={GROUPS} defaultValue={["D32 Steel"]}>
            <ComboboxInput id="mat-grouped" placeholder="1 selected" startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matches</ComboboxEmpty>
                <ComboboxList>
                    {(group: (typeof GROUPS)[number]) => (
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

export const WithStatus = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="mat-status">
            Material to farm
        </label>
        <Combobox<string, false> defaultOpen items={MATERIALS}>
            <ComboboxInput id="mat-status" placeholder="Search materials..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxStatus>Showing 9 of 143 materials</ComboboxStatus>
                <ComboboxEmpty>No matching materials.</ComboboxEmpty>
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

export const NoMatches = () => (
    <div className="h-72 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="mat-empty">
            Material to farm
        </label>
        <Combobox<string, false> defaultOpen defaultInputValue="Ultimate Origin Stone" items={MATERIALS}>
            <ComboboxInput id="mat-empty" placeholder="Search materials..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching materials.</ComboboxEmpty>
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
