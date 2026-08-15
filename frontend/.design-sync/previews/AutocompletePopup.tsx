import { Autocomplete, AutocompleteGroup, AutocompleteGroupLabel, AutocompleteInput, AutocompleteItem, AutocompleteList, AutocompletePopup, OperatorAvatar } from "frontend";
import { SearchIcon } from "lucide-react";

const OPERATORS = ["Młynar", "Muelsyse", "Mudrock", "Mostima", "Myrtle"];

const ROSTER = [
    { id: "char_4064_mlynar", meta: "E2 90 · S3M3", name: "Młynar" },
    { id: "char_249_mlyss", meta: "E2 80 · S2M3", name: "Muelsyse" },
    { id: "char_1028_texas2", meta: "E2 90 · S2M3", name: "Texas the Omertosa" },
];

export const Open = () => (
    <div className="w-80">
        <Autocomplete defaultValue="m" mode="none" open>
            <AutocompleteInput placeholder="Search operators…" showTrigger startAddon={<SearchIcon />} />
            <AutocompletePopup>
                <AutocompleteList>
                    {OPERATORS.map((name) => (
                        <AutocompleteItem key={name} value={name}>
                            {name}
                        </AutocompleteItem>
                    ))}
                </AutocompleteList>
            </AutocompletePopup>
        </Autocomplete>
    </div>
);

export const Grouped = () => (
    <div className="w-80">
        <Autocomplete defaultValue="ori" mode="none" open>
            <AutocompleteInput showTrigger startAddon={<SearchIcon />} />
            <AutocompletePopup>
                <AutocompleteList>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Materials</AutocompleteGroupLabel>
                        <AutocompleteItem value="Orirock">Orirock</AutocompleteItem>
                        <AutocompleteItem value="Orirock Cube">Orirock Cube</AutocompleteItem>
                        <AutocompleteItem value="Orirock Cluster">Orirock Cluster</AutocompleteItem>
                    </AutocompleteGroup>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Stages</AutocompleteGroupLabel>
                        <AutocompleteItem value="1-7">1-7 — Orirock Cube</AutocompleteItem>
                    </AutocompleteGroup>
                </AutocompleteList>
            </AutocompletePopup>
        </Autocomplete>
    </div>
);

export const OperatorResults = () => (
    <div className="w-96">
        <Autocomplete defaultValue="m" mode="none" open>
            <AutocompleteInput showClear startAddon={<SearchIcon />} />
            <AutocompletePopup>
                <AutocompleteList>
                    {ROSTER.map((op) => (
                        <AutocompleteItem className="flex flex-row gap-2" key={op.id} value={op.id}>
                            <span aria-hidden="true" className="op-chip">
                                <OperatorAvatar charId={op.id} name={op.name} />
                            </span>
                            <span className="flex-1 font-medium">{op.name}</span>
                            <span className="font-mono text-muted-foreground text-xs">{op.meta}</span>
                        </AutocompleteItem>
                    ))}
                </AutocompleteList>
            </AutocompletePopup>
        </Autocomplete>
    </div>
);
