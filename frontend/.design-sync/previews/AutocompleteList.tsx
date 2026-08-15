import type * as React from "react";
import { Autocomplete, AutocompleteGroup, AutocompleteGroupLabel, AutocompleteInput, AutocompleteItem, AutocompleteList, AutocompletePopup } from "frontend";
import { SearchIcon } from "lucide-react";

const STAGES = ["1-7 — Orirock Cube", "1-12 — Damaged Device", "2-8 — Sugar", "3-2 — Polyester", "4-6 — Sugar Pack", "4-8 — Aketon", "5-9 — RMA70-12", "6-9 — Manganese Ore", "7-4 — Grindstone", "8-14 — Crystalline Component", "S2-6 — Orirock Cube", "S3-4 — Oriron", "S4-1 — Oriron Block", "S5-7 — Loxic Kohl", "S6-2 — Coagulating Gel", "CE-6 — LMD", "LS-6 — Battle Record", "AP-5 — Carbon Brick", "SK-5 — Grindstone", "PR-A-2 — Medic Chip"];

const Panel = ({ children }: { children: React.ReactNode }) => <div className="mt-2 overflow-hidden rounded-lg border bg-popover shadow-xs">{children}</div>;

export const Scrollable = () => (
    <div className="w-80">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Jump to a stage…" showTrigger startAddon={<SearchIcon />} />
            <AutocompletePopup>
                <AutocompleteList>
                    {STAGES.map((stage) => (
                        <AutocompleteItem key={stage} value={stage}>
                            {stage}
                        </AutocompleteItem>
                    ))}
                </AutocompleteList>
            </AutocompletePopup>
        </Autocomplete>
    </div>
);

export const Short = () => (
    <div className="w-80">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Filter by server…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteItem value="en">Global (EN)</AutocompleteItem>
                    <AutocompleteItem value="cn">Mainland (CN)</AutocompleteItem>
                    <AutocompleteItem value="jp">Japan (JP)</AutocompleteItem>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const Grouped = () => (
    <div className="w-80">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Search materials…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Tier 4</AutocompleteGroupLabel>
                        <AutocompleteItem value="grindstone">Grindstone</AutocompleteItem>
                        <AutocompleteItem value="rma70-12">RMA70-12</AutocompleteItem>
                    </AutocompleteGroup>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Tier 3</AutocompleteGroupLabel>
                        <AutocompleteItem value="orirock-cluster">Orirock Cluster</AutocompleteItem>
                        <AutocompleteItem value="oriron-cluster">Oriron Cluster</AutocompleteItem>
                        <AutocompleteItem value="loxic-kohl">Loxic Kohl</AutocompleteItem>
                    </AutocompleteGroup>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);
