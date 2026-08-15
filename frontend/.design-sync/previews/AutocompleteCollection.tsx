import type * as React from "react";
import { Autocomplete, AutocompleteCollection, AutocompleteGroup, AutocompleteGroupLabel, AutocompleteInput, AutocompleteItem, AutocompleteList } from "frontend";
import { SearchIcon } from "lucide-react";

interface IStage {
    value: string;
    label: string;
}

const STAGES: IStage[] = [
    { value: "1-7", label: "1-7 — Orirock Cube" },
    { value: "4-6", label: "4-6 — Sugar Pack" },
    { value: "S4-1", label: "S4-1 — Oriron Block" },
    { value: "CE-6", label: "CE-6 — LMD" },
    { value: "LS-6", label: "LS-6 — Battle Record" },
];

const GROUPS = [
    { value: "Operators", items: ["Mlynar", "Muelsyse", "Mudrock"] },
    { value: "Stages", items: ["1-7", "4-6", "S4-1"] },
];

const Panel = ({ children }: { children: React.ReactNode }) => <div className="mt-2 overflow-hidden rounded-lg border bg-popover shadow-xs">{children}</div>;

export const FlatCollection = () => (
    <div className="w-80">
        <Autocomplete items={STAGES} open>
            <AutocompleteInput placeholder="Search stages…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteCollection>
                        {(stage: IStage) => (
                            <AutocompleteItem key={stage.value} value={stage}>
                                {stage.label}
                            </AutocompleteItem>
                        )}
                    </AutocompleteCollection>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const Filtered = () => (
    <div className="w-80">
        <Autocomplete defaultValue="S4" items={STAGES} open>
            <AutocompleteInput startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteCollection>
                        {(stage: IStage) => (
                            <AutocompleteItem key={stage.value} value={stage}>
                                {stage.label}
                            </AutocompleteItem>
                        )}
                    </AutocompleteCollection>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const GroupedCollection = () => (
    <div className="w-80">
        <Autocomplete items={GROUPS} open>
            <AutocompleteInput placeholder="Search everything…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteCollection>
                        {(group: { value: string; items: string[] }) => (
                            <AutocompleteGroup key={group.value} items={group.items}>
                                <AutocompleteGroupLabel>{group.value}</AutocompleteGroupLabel>
                                <AutocompleteCollection>
                                    {(item: string) => (
                                        <AutocompleteItem key={item} value={item}>
                                            {item}
                                        </AutocompleteItem>
                                    )}
                                </AutocompleteCollection>
                            </AutocompleteGroup>
                        )}
                    </AutocompleteCollection>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);
