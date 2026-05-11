import { Filter, MapPinned, Users } from "lucide-react";
import type React from "react";
import { Sheet, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "#/components/ui/tabs";
import type { IRandomizerOperator, IRandomizerSettings, IRosterIndex } from "../types";
import { OperatorFiltersPanel } from "./OperatorFiltersPanel";
import { RosterPicker } from "./RosterPicker";
import { StageFiltersPanel } from "./StageFiltersPanel";

interface ISettingsSheetProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    settings: IRandomizerSettings;
    onChange: (next: Partial<IRandomizerSettings>) => void;
    operators: IRandomizerOperator[];
    rosterSelection: Set<string>;
    onRosterChange: (next: Set<string>) => void;
    rosterIndex: IRosterIndex;
    hasProfile: boolean;
}

export function SettingsSheet(props: ISettingsSheetProps): React.ReactElement {
    return (
        <Sheet open={props.open} onOpenChange={props.onOpenChange}>
            <SheetPopup side="right" variant="inset" className="w-[min(520px,100vw)] sm:max-w-[520px]">
                <SheetHeader>
                    <SheetTitle className="flex items-center gap-2">
                        <Filter aria-hidden="true" className="size-4 text-muted-foreground" />
                        Briefing settings
                    </SheetTitle>
                    <p className="text-sm text-muted-foreground">Constrain the draw — by class, rarity, owned operators, or stage availability.</p>
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
                            <StageFiltersPanel settings={props.settings} onChange={props.onChange} hasProfile={props.hasProfile} />
                        </TabsContent>

                        <TabsContent value="roster">
                            <RosterPicker operators={props.operators} selected={props.rosterSelection} onChange={props.onRosterChange} rosterIndex={props.rosterIndex} hasProfile={props.hasProfile} />
                        </TabsContent>
                    </Tabs>
                </SheetPanel>
            </SheetPopup>
        </Sheet>
    );
}
