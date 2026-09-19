import { CalendarClock, CalendarDays, Dices, Palette, Shirt, Ticket } from "lucide-react";
import * as React from "react";
import { PageHeader } from "#/components/ui/page-header";
import { ScrollArea } from "#/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "#/components/ui/tabs";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { AutoTranslateProvider, useAutoTranslateSetting } from "./impl/autoTranslate";
import { BannersTab } from "./impl/components/BannersTab";
import { CalendarTab } from "./impl/components/CalendarTab";
import { EventsTab } from "./impl/components/EventsTab";
import { PlannerTab } from "./impl/components/PlannerTab";
import { PullsPlannerTab } from "./impl/components/PullsPlannerTab";
import { SkinPopupProvider } from "./impl/components/SkinPopup";
import { SkinsTab } from "./impl/components/SkinsTab";
import { ToggleField } from "./impl/components/shared";
import type { messages } from "./ReleasePlanner.messages";

type ReleaseView = "planner" | "pulls" | "events" | "skins" | "banners" | "calendar";

export function ReleasePlanner(): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    const today = React.useMemo(() => new Date(), []);
    const [view, setView] = React.useState<ReleaseView>("planner");
    const [autoTranslate, setAutoTranslate] = useAutoTranslateSetting();

    return (
        <div className="page-shell [--page-max:1200px]" translate="no">
            <PageHeader breadcrumbLabel="breadcrumb" breadcrumb={[t("release.breadcrumb.tools"), t("release.title")]} title={t("release.title")} description={t("release.intro")} />

            {/* The explanation of where the names come from was a permanent
                paragraph under a one-word toggle, on a page already dense enough
                that readers called it too much. It is the toggle's tooltip now:
                the same sentence, reachable, not occupying the page every visit. */}
            <div className="mt-5 flex flex-col gap-1.5">
                <span title={t("release.autoTranslate.desc")}>
                    <ToggleField id="release-auto-translate" label={t("release.autoTranslate")} checked={autoTranslate} onChange={setAutoTranslate} />
                </span>
            </div>

            <AutoTranslateProvider value={autoTranslate}>
                <SkinPopupProvider>
                    <Tabs value={view} onValueChange={(v) => setView(v as ReleaseView)} className="mt-5">
                        <ScrollArea className="mb-4 w-full" scrollFade>
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
                                <TabsTrigger value="skins" className="max-sm:shrink-0">
                                    <Shirt />
                                    {t("release.tab.skins")}
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
                        <TabsContent value="planner">
                            <PlannerTab today={today} />
                        </TabsContent>
                        <TabsContent value="pulls">
                            <PullsPlannerTab today={today} />
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
                        <TabsContent value="calendar">
                            <CalendarTab today={today} />
                        </TabsContent>
                    </Tabs>
                </SkinPopupProvider>
            </AutoTranslateProvider>
        </div>
    );
}
