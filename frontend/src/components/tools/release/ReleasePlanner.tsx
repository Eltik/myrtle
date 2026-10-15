import { CalendarClock, CalendarDays, Dices, Palette, Ticket } from "lucide-react";
import * as React from "react";
import { PageHeader } from "#/components/ui/page-header";
import { ScrollArea } from "#/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "#/components/ui/tabs";
import { useTheme } from "#/hooks/use-theme";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { AutoTranslateProvider } from "./impl/autoTranslate";
import { BannersTab } from "./impl/components/BannersTab";
import { CalendarTab } from "./impl/components/CalendarTab";
import { EventsTab } from "./impl/components/EventsTab";
import { PlannerTab } from "./impl/components/PlannerTab";
import { PullsPlannerTab } from "./impl/components/PullsPlannerTab";
import { SkinPopupProvider } from "./impl/components/SkinPopup";
import type { messages } from "./ReleasePlanner.messages";

type ReleaseView = "planner" | "pulls" | "events" | "banners" | "calendar";

const RELEASE_VIEWS: readonly ReleaseView[] = ["planner", "pulls", "events", "banners", "calendar"];

export function ReleasePlanner(): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    const today = React.useMemo(() => new Date(), []);
    const [view, setView] = React.useState<ReleaseView>("planner");
    const { latinNames } = useTheme();

    /**
     * A tab mounts on its first visit and then STAYS mounted, hidden, so going back to it
     * is an attribute flip instead of a rebuild. Remounting was the cost of every switch:
     * the Planner tab rebuilt its 36 cards and the detail pane (about 170 ms of handler time
     * in the dev build), and the Pulls tab reran the plan DP, the Monte Carlo and three
     * charts (about 400 ms). Mounting all five up front would put that work on page load
     * for tabs most visits never open, hence first-visit.
     */
    const [visited, setVisited] = React.useState<ReadonlySet<ReleaseView>>(() => new Set<ReleaseView>(["planner"]));
    const select = React.useCallback((next: ReleaseView) => {
        setView(next);
        setVisited((prev) => (prev.has(next) ? prev : new Set(prev).add(next)));
    }, []);

    // Built once, so a tab switch re-renders the strip and not the hidden tabs.
    const panels = React.useMemo<Record<ReleaseView, React.ReactElement>>(
        () => ({
            planner: <PlannerTab today={today} />,
            pulls: <PullsPlannerTab today={today} />,
            events: <EventsTab today={today} />,
            banners: <BannersTab today={today} />,
            calendar: <CalendarTab today={today} />,
        }),
        [today],
    );

    return (
        <div className="page-shell [--page-max:1200px]" translate="no">
            <PageHeader breadcrumbLabel="breadcrumb" breadcrumb={[t("release.breadcrumb.tools"), t("release.title")]} title={t("release.title")} />

            <AutoTranslateProvider value={latinNames}>
                <SkinPopupProvider>
                    <Tabs value={view} onValueChange={(v) => select(v as ReleaseView)} className="mt-5">
                        <ScrollArea className="mb-4 h-auto w-full shrink-0" scrollFade>
                            <TabsList className="w-max">
                                <TabsTrigger value="planner" className="max-sm:shrink-0">
                                    <Palette />
                                    {t("release.tab.planner")}
                                </TabsTrigger>
                                <TabsTrigger value="pulls" className="max-sm:shrink-0">
                                    <Dices />
                                    {t("release.tab.pulls")}
                                </TabsTrigger>
                                <TabsTrigger value="events" className="max-sm:shrink-0">
                                    <CalendarClock />
                                    {t("release.tab.events")}
                                </TabsTrigger>
                                <TabsTrigger value="banners" className="max-sm:shrink-0">
                                    <Ticket />
                                    {t("release.tab.banners")}
                                </TabsTrigger>
                                <TabsTrigger value="calendar" className="max-sm:shrink-0">
                                    <CalendarDays />
                                    {t("release.tab.calendar")}
                                </TabsTrigger>
                            </TabsList>
                        </ScrollArea>
                        {RELEASE_VIEWS.map((v) => (
                            <TabsContent key={v} value={v} keepMounted>
                                {visited.has(v) && panels[v]}
                            </TabsContent>
                        ))}
                    </Tabs>
                </SkinPopupProvider>
            </AutoTranslateProvider>
        </div>
    );
}
