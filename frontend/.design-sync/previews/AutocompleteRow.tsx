import type * as React from "react";
import { Autocomplete, AutocompleteInput, AutocompleteItem, AutocompleteList, AutocompleteRow } from "frontend";
import { PackageIcon, SearchIcon } from "lucide-react";

const MATERIAL_ROWS = [
    ["Orirock Cube", "Sugar Pack", "Polyester Pack"],
    ["Oriron Cluster", "Keton Colloid", "Grindstone"],
    ["Manganese Ore", "Loxic Kohl", "RMA70-12"],
];

const CLASS_ROWS = [
    ["Vanguard", "Guard", "Defender", "Sniper"],
    ["Caster", "Medic", "Supporter", "Specialist"],
];

const Panel = ({ children }: { children: React.ReactNode }) => <div className="mt-2 overflow-hidden rounded-lg border bg-popover shadow-xs">{children}</div>;

export const MaterialGrid = () => (
    <div className="w-96">
        <Autocomplete grid mode="none" open>
            <AutocompleteInput placeholder="Pick a material to farm…" startAddon={<PackageIcon />} />
            <Panel>
                <AutocompleteList>
                    {MATERIAL_ROWS.map((row) => (
                        <AutocompleteRow className="flex gap-1" key={row[0]}>
                            {row.map((material) => (
                                <AutocompleteItem className="flex-1 justify-center text-center text-xs" key={material} value={material}>
                                    {material}
                                </AutocompleteItem>
                            ))}
                        </AutocompleteRow>
                    ))}
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const ClassGrid = () => (
    <div className="w-96">
        <Autocomplete grid mode="none" open>
            <AutocompleteInput placeholder="Filter roster by class…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    {CLASS_ROWS.map((row) => (
                        <AutocompleteRow className="flex gap-1" key={row[0]}>
                            {row.map((cls) => (
                                <AutocompleteItem className="flex-1 justify-center text-center text-xs" key={cls} value={cls}>
                                    {cls}
                                </AutocompleteItem>
                            ))}
                        </AutocompleteRow>
                    ))}
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const SingleRow = () => (
    <div className="w-96">
        <Autocomplete grid mode="none" open>
            <AutocompleteInput placeholder="Promotion target…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteRow className="flex gap-1">
                        <AutocompleteItem className="flex-1 justify-center text-xs" value="e0">
                            Elite 0
                        </AutocompleteItem>
                        <AutocompleteItem className="flex-1 justify-center text-xs" value="e1">
                            Elite 1
                        </AutocompleteItem>
                        <AutocompleteItem className="flex-1 justify-center text-xs" value="e2">
                            Elite 2
                        </AutocompleteItem>
                    </AutocompleteRow>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);
