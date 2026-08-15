import type * as React from "react";
import { Autocomplete, AutocompleteGroup, AutocompleteGroupLabel, AutocompleteInput, AutocompleteItem, AutocompleteList } from "frontend";
import { SearchIcon } from "lucide-react";

const Panel = ({ children }: { children: React.ReactNode }) => <div className="mt-2 overflow-hidden rounded-lg border bg-popover shadow-xs">{children}</div>;

export const TwoGroups = () => (
    <div className="w-80">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Search operators and stages…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Operators</AutocompleteGroupLabel>
                        <AutocompleteItem value="mlynar">Młynar</AutocompleteItem>
                        <AutocompleteItem value="skadi2">Skadi the Corrupting Heart</AutocompleteItem>
                        <AutocompleteItem value="mlyss">Muelsyse</AutocompleteItem>
                    </AutocompleteGroup>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Stages</AutocompleteGroupLabel>
                        <AutocompleteItem value="1-7">1-7 — Orirock Cube</AutocompleteItem>
                        <AutocompleteItem value="CE-6">CE-6 — LMD</AutocompleteItem>
                    </AutocompleteGroup>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const SingleGroup = () => (
    <div className="w-80">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Filter by class…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Classes</AutocompleteGroupLabel>
                        <AutocompleteItem value="vanguard">Vanguard</AutocompleteItem>
                        <AutocompleteItem value="guard">Guard</AutocompleteItem>
                        <AutocompleteItem value="defender">Defender</AutocompleteItem>
                        <AutocompleteItem value="sniper">Sniper</AutocompleteItem>
                    </AutocompleteGroup>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const ByRarity = () => (
    <div className="w-80">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Add to tier list…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>6★</AutocompleteGroupLabel>
                        <AutocompleteItem value="mlynar">Młynar</AutocompleteItem>
                        <AutocompleteItem value="ceobe">Ceobe</AutocompleteItem>
                    </AutocompleteGroup>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>5★</AutocompleteGroupLabel>
                        <AutocompleteItem value="ptilopsis">Ptilopsis</AutocompleteItem>
                        <AutocompleteItem value="lappland">Lappland</AutocompleteItem>
                    </AutocompleteGroup>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>4★</AutocompleteGroupLabel>
                        <AutocompleteItem value="myrtle">Myrtle</AutocompleteItem>
                    </AutocompleteGroup>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);
