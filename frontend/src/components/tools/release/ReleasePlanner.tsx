import { CalendarClock, CalendarDays, ChartGantt, ChevronRight, Dices, Palette, Shirt, Ticket } from "lucide-react";
import * as React from "react";
import { ScrollArea } from "#/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "#/components/ui/tabs";
import { AutoTranslateProvider, useAutoTranslateSetting } from "./impl/autoTranslate";
import { BannersTab } from "./impl/components/BannersTab";
import { CalendarTab } from "./impl/components/CalendarTab";
import { EventsTab } from "./impl/components/EventsTab";
import { PlannerTab } from "./impl/components/PlannerTab";
import { PullsPlannerTab } from "./impl/components/PullsPlannerTab";
import { SkinPopupProvider } from "./impl/components/SkinPopup";
import { SkinsTab } from "./impl/components/SkinsTab";
import { ToggleField } from "./impl/components/shared";
import { TimelineTab } from "./impl/components/TimelineTab";

type ReleaseView = "planner" | "pulls" | "events" | "skins" | "banners" | "timeline" | "calendar";

export function ReleasePlanner(): React.ReactElement {
    const today = React.useMemo(() => new Date(), []);
    const [view, setView] = React.useState<ReleaseView>("planner");
    const [autoTranslate, setAutoTranslate] = useAutoTranslateSetting();

    return (
        <div className="relative z-1 mx-auto w-[min(1200px,calc(100%-2rem))] py-5 pb-20" translate="no">
            <nav aria-label="breadcrumb" className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none">
                <span>Tools</span>
                <ChevronRight className="size-2.5" />
                <span className="text-foreground">Release Planner</span>
            </nav>
            <div className="flex flex-wrap items-end justify-between gap-3">
                <div className="min-w-0 flex-1">
                    <h1 className="m-0 font-bold font-sans text-[24px] text-foreground leading-[1.1] tracking-tight sm:text-[30px]">Release Planner</h1>
                    <p className="mt-1.5 max-w-2xl font-sans text-[13.5px] text-muted-foreground leading-normal">
                        When CN content lands on EN. Confirmed rows come from EN game data, announced rows from a manual override, estimated rows from the trailing CN-to-EN lag with a band; outfits ship with the event they ran under on CN. The Skins planner budgets Originite Prime for outfits against event first
                        clears.
                    </p>
                </div>
            </div>

            <div className="mt-5 flex flex-col gap-1.5">
                <ToggleField id="release-auto-translate" label="Auto-translate CN names" checked={autoTranslate} onChange={setAutoTranslate} />
                <p className="m-0 font-sans text-[12px] text-muted-foreground leading-normal">Names come from EN game data where an id exists on both servers; anything still in Chinese is marked for your browser's translator.</p>
            </div>

            <AutoTranslateProvider value={autoTranslate}>
                <SkinPopupProvider>
                    <Tabs value={view} onValueChange={(v) => setView(v as ReleaseView)} className="mt-5">
                        <ScrollArea className="mb-4 w-full" scrollFade>
                            <TabsList className="w-max">
                                <TabsTrigger value="planner" className="max-sm:shrink-0">
                                    <Palette />
                                    Planner
                                </TabsTrigger>
                                <TabsTrigger value="pulls" className="max-sm:shrink-0">
                                    <Dices />
                                    Pulls planner
                                </TabsTrigger>
                                <TabsTrigger value="events" className="max-sm:shrink-0">
                                    <CalendarClock />
                                    Events
                                </TabsTrigger>
                                <TabsTrigger value="skins" className="max-sm:shrink-0">
                                    <Shirt />
                                    Skins
                                </TabsTrigger>
                                <TabsTrigger value="banners" className="max-sm:shrink-0">
                                    <Ticket />
                                    Banners
                                </TabsTrigger>
                                <TabsTrigger value="timeline" className="max-sm:shrink-0">
                                    <ChartGantt />
                                    Timeline
                                </TabsTrigger>
                                <TabsTrigger value="calendar" className="max-sm:shrink-0">
                                    <CalendarDays />
                                    Calendar
                                </TabsTrigger>
                            </TabsList>
                        </ScrollArea>
                        <TabsContent value="planner">
                            <PlannerTab today={today} />
                        </TabsContent>
                        <TabsContent value="pulls">
                            <PullsPlannerTab />
                        </TabsContent>
                        <TabsContent value="events">
                            <EventsTab today={today} />
                        </TabsContent>
                        <TabsContent value="skins">
                            <SkinsTab today={today} />
                        </TabsContent>
                        <TabsContent value="banners">
                            <BannersTab today={today} />
                        </TabsContent>
                        <TabsContent value="timeline">
                            <TimelineTab today={today} />
                        </TabsContent>
                        <TabsContent value="calendar">
                            <CalendarTab today={today} />
                        </TabsContent>
                    </Tabs>
                </SkinPopupProvider>
            </AutoTranslateProvider>
        </div>
    );
}
