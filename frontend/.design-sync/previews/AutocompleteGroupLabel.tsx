import type * as React from "react";
import { Autocomplete, AutocompleteGroup, AutocompleteGroupLabel, AutocompleteInput, AutocompleteItem, AutocompleteList } from "frontend";
import { SearchIcon } from "lucide-react";

const Panel = ({ children }: { children: React.ReactNode }) => <div className="mt-2 overflow-hidden rounded-lg border bg-popover shadow-xs">{children}</div>;

export const Default = () => (
    <div className="w-80">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Search everything…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Operators</AutocompleteGroupLabel>
                        <AutocompleteItem value="mlynar">Młynar</AutocompleteItem>
                        <AutocompleteItem value="mlyss">Muelsyse</AutocompleteItem>
                    </AutocompleteGroup>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Tools</AutocompleteGroupLabel>
                        <AutocompleteItem value="dps">DPS calculator</AutocompleteItem>
                        <AutocompleteItem value="planner">Operator planner</AutocompleteItem>
                    </AutocompleteGroup>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const WithCount = () => (
    <div className="w-80">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Search operators…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel className="flex items-center justify-between">
                            <span>Owned</span>
                            <span className="font-mono text-[10px] tabular-nums">231</span>
                        </AutocompleteGroupLabel>
                        <AutocompleteItem value="mlynar">Młynar</AutocompleteItem>
                        <AutocompleteItem value="texas2">Texas the Omertosa</AutocompleteItem>
                    </AutocompleteGroup>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel className="flex items-center justify-between">
                            <span>Not owned</span>
                            <span className="font-mono text-[10px] tabular-nums">89</span>
                        </AutocompleteGroupLabel>
                        <AutocompleteItem value="ling">Ling</AutocompleteItem>
                    </AutocompleteGroup>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);

export const ChapterLabels = () => (
    <div className="w-80">
        <Autocomplete mode="none" open>
            <AutocompleteInput placeholder="Jump to a stage…" startAddon={<SearchIcon />} />
            <Panel>
                <AutocompleteList>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Chapter 8 — Roaring Flare</AutocompleteGroupLabel>
                        <AutocompleteItem value="8-14">8-14 — Crystalline Component</AutocompleteItem>
                        <AutocompleteItem value="8-16">8-16 — Crystalline Circuit</AutocompleteItem>
                    </AutocompleteGroup>
                    <AutocompleteGroup>
                        <AutocompleteGroupLabel>Supply — Weekly</AutocompleteGroupLabel>
                        <AutocompleteItem value="CE-6">CE-6 — LMD</AutocompleteItem>
                        <AutocompleteItem value="AP-5">AP-5 — Carbon Brick</AutocompleteItem>
                    </AutocompleteGroup>
                </AutocompleteList>
            </Panel>
        </Autocomplete>
    </div>
);
