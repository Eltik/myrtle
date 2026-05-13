import { Filter, MapPinned, Users } from "lucide-react";
import type React from "react";
import { Sheet, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "#/components/ui/tabs";
import type { IStage, IZone, StageClearsMap } from "#/types/stages";
import type { IActivityLookup } from "../activity-lookup";
import type { IRandomizerOperator, IRandomizerSettings, IRosterIndex } from "../types";
import { OperatorFiltersPanel } from "./OperatorFiltersPanel";
import { RosterPicker } from "./RosterPicker";
import { StageFiltersPanel } from "./StageFiltersPanel";

interface ISettingsSheetProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    settings: IRandomizerSettings;
    onChange: (next: Partial<IRandomizerSettings>) => void;
    /** Complete operator list, used by the roster picker as the "universe". */
    allOperators: IRandomizerOperator[];
    /** Operators that pass the operator-tab constraints (class/rarity/unplayable). */
    rosterPickerOperators: IRandomizerOperator[];
    rosterSelection: Set<string>;
    /** True once the user has explicitly set the roster (vs. the "all" default). */
    rosterIsExplicit: boolean;
    onRosterChange: (next: Set<string>) => void;
    onRosterReset: () => void;
    rosterIndex: IRosterIndex;
    hasProfile: boolean;
    stages: IStage[];
    zones: IZone[];
    activityLookup: IActivityLookup;
    stageClears: StageClearsMap | null;
}

export function SettingsSheet(props: ISettingsSheetProps): React.ReactElement {
    return (
        <Sheet open={props.open} onOpenChange={props.onOpenChange}>
            <SheetPopup side="right" variant="inset" className="w-[min(520px,100vw)] sm:max-w-130">
                <SheetHeader>
                    <SheetTitle className="flex items-center gap-2">
                        <Filter aria-hidden="true" className="size-4 text-muted-foreground" />
                        Settings
                    </SheetTitle>
                    <p className="text-sm text-muted-foreground">Constrain the randomizer - by class, rarity, owned operators, or stage availability.</p>
                </SheetHeader>
                <SheetPanel className="px-5 pb-6">
                    <Tabs defaultValue="operators" className="gap-4">
                        <TabsList className="w-full">
                            <TabsTrigger value="operators" className="flex-1">
                                <Filter aria-hidden="true" />
                                Operators
                            </TabsTrigger>
                            <TabsTrigger value="stages" className="flex-1">
                                <MapPinned aria-hidden="true" />
                                Stages
                            </TabsTrigger>
                            <TabsTrigger value="roster" className="flex-1">
                                <Users aria-hidden="true" />
                                Roster
                            </TabsTrigger>
                        </TabsList>

                        <TabsContent value="operators">
                            <OperatorFiltersPanel settings={props.settings} onChange={props.onChange} hasProfile={props.hasProfile} />
                        </TabsContent>

                        <TabsContent value="stages">
                            <StageFiltersPanel settings={props.settings} onChange={props.onChange} hasProfile={props.hasProfile} stages={props.stages} zones={props.zones} activityLookup={props.activityLookup} stageClears={props.stageClears} />
                        </TabsContent>

                        <TabsContent value="roster">
                            <RosterPicker
                                allOperators={props.allOperators}
                                visibleOperators={props.rosterPickerOperators}
                                selected={props.rosterSelection}
                                isExplicit={props.rosterIsExplicit}
                                onChange={props.onRosterChange}
                                onReset={props.onRosterReset}
                                rosterIndex={props.rosterIndex}
                                hasProfile={props.hasProfile}
                                settings={props.settings}
                            />
                        </TabsContent>
                    </Tabs>
                </SheetPanel>
            </SheetPopup>
        </Sheet>
    );
}
