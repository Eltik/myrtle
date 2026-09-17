import { CalendarClock, CalendarDays, ChevronRight, Dices, Palette, Shirt, Ticket } from "lucide-react";
import * as React from "react";
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
        <div className="relative z-1 mx-auto w-[min(1200px,calc(100%-2rem))] py-5 pb-20" translate="no">
            <nav aria-label="breadcrumb" className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none">
                <span>{t("release.breadcrumb.tools")}</span>
                <ChevronRight className="size-2.5" />
                <span className="text-foreground">{t("release.title")}</span>
            </nav>
            <div className="flex flex-wrap items-end justify-between gap-3">
                <div className="min-w-0 flex-1">
                    <h1 className="m-0 font-bold font-sans text-[24px] text-foreground leading-[1.1] tracking-tight sm:text-[30px]">{t("release.title")}</h1>
                    <p className="mt-1.5 max-w-2xl font-sans text-[13.5px] text-muted-foreground leading-normal">{t("release.intro")}</p>
                </div>
            </div>

            <div className="mt-5 flex flex-col gap-1.5">
                <ToggleField id="release-auto-translate" label={t("release.autoTranslate")} checked={autoTranslate} onChange={setAutoTranslate} />
                <p className="m-0 font-sans text-[12px] text-muted-foreground leading-normal">{t("release.autoTranslate.desc")}</p>
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
