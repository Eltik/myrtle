import type * as React from "react";
import { Autocomplete, AutocompleteGroup, AutocompleteGroupLabel, AutocompleteInput, AutocompleteItem, AutocompleteList, AutocompleteSeparator, OperatorAvatar } from "frontend";
import { SearchIcon } from "lucide-react";

interface IOperator {
    id: string;
    name: string;
    meta: string;
}

const OPERATORS: IOperator[] = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", meta: "6★ · Supporter" },
    { id: "char_249_mlyss", name: "Muelsyse", meta: "6★ · Vanguard" },
    { id: "char_1028_texas2", name: "Texas the Omertosa", meta: "6★ · Specialist" },
    { id: "char_180_amgoat", name: "Eyjafjalla", meta: "6★ · Caster" },
];

const Panel = ({ children }: { children: React.ReactNode }) => <div className="mt-2 overflow-hidden rounded-lg border bg-popover shadow-xs">{children}</div>;

const OperatorItem = ({ op }: { op: IOperator }) => (
    <AutocompleteItem className="flex flex-row gap-2" value={op.id}>
        <span aria-hidden="true" className="op-chip">
            <OperatorAvatar charId={op.id} name={op.name} />
        </span>
        <span className="flex-1 font-medium">{op.name}</span>
        <span className="text-muted-foreground text-xs">{op.meta}</span>
    </AutocompleteItem>
);

export const Inline = () => (
    <div className="w-96">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Search operators…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    {OPERATORS.map((op) => (
                        <OperatorItem key={op.id} op={op} />
                    ))}
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const Closed = () => (
    <div className="w-96">
        <Autocomplete mode="none">
            <AutocompleteInput placeholder="Search operators…" showTrigger startAddon={<SearchIcon />} />
        </Autocomplete>
    </div>
);

export const Grouped = () => (
    <div className="w-96">
        <Autocomplete defaultValue="ceo" mode="none" open>
            <AutocompleteInput showClear startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Operators</AutocompleteGroupLabel>
                        <AutocompleteItem value="ceobe">Ceobe</AutocompleteItem>
                        <AutocompleteItem value="ceylon">Ceylon</AutocompleteItem>
                    </AutocompleteGroup>
                    <AutocompleteSeparator />
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Stages</AutocompleteGroupLabel>
                        <AutocompleteItem value="CE-6">CE-6 — Cargo Escort</AutocompleteItem>
                    </AutocompleteGroup>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const Disabled = () => (
    <div className="w-96">
        <Autocomplete defaultValue="Młynar" disabled mode="none">
            <AutocompleteInput showTrigger startAddon={<SearchIcon />} />
        </Autocomplete>
    </div>
);
