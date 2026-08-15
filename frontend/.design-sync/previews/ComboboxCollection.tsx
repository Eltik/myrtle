import { Fragment } from "react";
import { Combobox, ComboboxCollection, ComboboxEmpty, ComboboxGroup, ComboboxGroupLabel, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxSeparator } from "frontend";
import { Search } from "lucide-react";

interface IGroup {
    key: string;
    label: string;
    items: string[];
}

const MATERIAL_TIERS: IGroup[] = [
    { key: "t5", label: "Tier 5 — Elite", items: ["D32 Steel", "Bipolar Nanoflake", "Crystalline Electronic Unit"] },
    { key: "t4", label: "Tier 4 — Advanced", items: ["Grindstone Pentahydrate", "RMA70-24", "Polymerization Preparation"] },
    { key: "t3", label: "Tier 3 — Common", items: ["Orirock Cluster", "Sugar Pack", "Oriron Cluster"] },
];

const EVENTS: IGroup[] = [
    { key: "rerun", label: "Reruns", items: ["Under Tides", "Dossoles Holiday", "Near Light"] },
    { key: "side", label: "Side stories", items: ["Lone Trail", "Where Vernal Winds Will Never Blow"] },
];

export const TieredMaterials = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="coll-mats">
            Material to farm
        </label>
        <Combobox<string, false> defaultOpen items={MATERIAL_TIERS}>
            <ComboboxInput id="coll-mats" placeholder="Search 143 materials..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching materials.</ComboboxEmpty>
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

export const IndexedRows = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="coll-indexed">
            Event archive
        </label>
        <Combobox<string, false> defaultOpen items={EVENTS}>
            <ComboboxInput id="coll-indexed" placeholder="Search 68 events..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching events.</ComboboxEmpty>
                <ComboboxList>
                    {(group: IGroup) => (
                        <Fragment key={group.key}>
                            <ComboboxGroup items={group.items}>
                                <ComboboxGroupLabel>{group.label}</ComboboxGroupLabel>
                                <ComboboxCollection>
                                    {(name: string, index: number) => (
                                        <ComboboxItem key={name} value={name}>
                                            <span className="flex items-baseline gap-2">
                                                <span className="font-mono text-[10px] text-muted-foreground tabular-nums">{String(index + 1).padStart(2, "0")}</span>
                                                <span>{name}</span>
                                            </span>
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

export const MultiSelectCollection = () => (
    <div className="h-96 w-80 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Materials to track</span>
        <Combobox<string, true> defaultOpen multiple defaultValue={["D32 Steel", "Sugar Pack"]} items={MATERIAL_TIERS}>
            <ComboboxInput placeholder="2 selected" startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching materials.</ComboboxEmpty>
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
