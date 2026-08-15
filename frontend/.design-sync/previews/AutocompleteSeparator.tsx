import type * as React from "react";
import { Autocomplete, AutocompleteGroup, AutocompleteGroupLabel, AutocompleteInput, AutocompleteItem, AutocompleteList, AutocompleteSeparator } from "frontend";
import { SearchIcon } from "lucide-react";

const Panel = ({ children }: { children: React.ReactNode }) => <div className="mt-2 overflow-hidden rounded-lg border bg-popover shadow-xs">{children}</div>;

export const BetweenGroups = () => (
    <div className="w-80">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Search everything…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Operators</AutocompleteGroupLabel>
                        <AutocompleteItem value="mlynar">Młynar</AutocompleteItem>
                        <AutocompleteItem value="muelsyse">Muelsyse</AutocompleteItem>
                    </AutocompleteGroup>
                    <AutocompleteSeparator />
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Stages</AutocompleteGroupLabel>
                        <AutocompleteItem value="1-7">1-7 — Orirock Cube</AutocompleteItem>
                        <AutocompleteItem value="S4-1">S4-1 — Oriron Block</AutocompleteItem>
                    </AutocompleteGroup>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const ThreeSections = () => (
    <div className="w-80">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Search everything…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Recent</AutocompleteGroupLabel>
                        <AutocompleteItem value="recent-dps">DPS calculator</AutocompleteItem>
                    </AutocompleteGroup>
                    <AutocompleteSeparator />
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Tools</AutocompleteGroupLabel>
                        <AutocompleteItem value="planner">Operator planner</AutocompleteItem>
                        <AutocompleteItem value="recruitment">Recruitment calculator</AutocompleteItem>
                    </AutocompleteGroup>
                    <AutocompleteSeparator />
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Pages</AutocompleteGroupLabel>
                        <AutocompleteItem value="leaderboard">Leaderboard</AutocompleteItem>
                    </AutocompleteGroup>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const TrailingSeparatorHidden = () => (
    <div className="w-80">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Filter materials…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Tier 3</AutocompleteGroupLabel>
                        <AutocompleteItem value="orirock-cluster">Orirock Cluster</AutocompleteItem>
                        <AutocompleteItem value="oriron-cluster">Oriron Cluster</AutocompleteItem>
                    </AutocompleteGroup>
                    {/* `last:hidden` on the separator suppresses the trailing rule. */}
                    <AutocompleteSeparator />
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);
