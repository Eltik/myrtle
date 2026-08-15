import type * as React from "react";
import { Autocomplete, AutocompleteInput, AutocompleteItem, AutocompleteList, OperatorAvatar } from "frontend";
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
];

const Panel = ({ children }: { children: React.ReactNode }) => <div className="mt-2 overflow-hidden rounded-lg border bg-popover shadow-xs">{children}</div>;

const OperatorItem = ({ op, disabled }: { op: IOperator; disabled?: boolean }) => (
    <AutocompleteItem className="flex flex-row gap-2" disabled={disabled} value={op.id}>
        <span aria-hidden="true" className="op-chip">
            <OperatorAvatar charId={op.id} name={op.name} />
        </span>
        <span className="flex-1 font-medium">{op.name}</span>
        <span className="text-muted-foreground text-xs">{disabled ? "Not owned" : op.meta}</span>
    </AutocompleteItem>
);

export const OperatorResults = () => (
    <div className="w-96">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Add an operator to the calculator…" startAddon={<SearchIcon />} />
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

export const WithDisabledItem = () => (
    <div className="w-96">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Add an operator to the calculator…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <OperatorItem op={OPERATORS[0]} />
                    <OperatorItem disabled op={OPERATORS[1]} />
                    <OperatorItem op={OPERATORS[2]} />
                    <OperatorItem disabled op={OPERATORS[3]} />
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const PlainItems = () => (
    <div className="w-80">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Jump to a stage…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteItem value="1-7">1-7 — Orirock Cube</AutocompleteItem>
                    <AutocompleteItem value="4-6">4-6 — Sugar Pack</AutocompleteItem>
                    <AutocompleteItem value="S4-1">S4-1 — Oriron Block</AutocompleteItem>
                    <AutocompleteItem value="CE-6">CE-6 — LMD</AutocompleteItem>
                    <AutocompleteItem value="LS-6">LS-6 — Battle Record</AutocompleteItem>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);
