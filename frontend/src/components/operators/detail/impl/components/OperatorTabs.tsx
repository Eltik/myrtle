import { Card, CardPanel } from "#/components/ui/card";
import { ScrollArea } from "#/components/ui/scroll-area";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { Tooltip, TooltipPopup, TooltipProvider, TooltipTrigger } from "#/components/ui/tooltip";
import { useOperatorName } from "#/hooks/use-operator-name";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { TABS, type TabType } from "../constants";
import type { messages as detailConstantsMessages } from "../constants.messages";
import type { messages } from "./OperatorTabs.messages";
import { AudioContent } from "./tabs/AudioContent";
import { InfoContent } from "./tabs/InfoContent";
import { LevelUpContent } from "./tabs/LevelUpContent";
import { LoreContent } from "./tabs/LoreContent";
import { SkillsContent } from "./tabs/SkillsContent";
import { SkinsContent } from "./tabs/SkinsContent";

interface IOperatorTabsProps {
    operator: IOperatorListItem;
    activeTab: TabType;
    onTabChange: (tab: TabType) => void;
}

const PANEL_ID = "operator-tabpanel";
const TAB_ID_PREFIX = "operator-tab-";
const MOBILE_TAB_ID_PREFIX = "operator-tab-m-";

const CONTENT_MAP = {
    info: InfoContent,
    skills: SkillsContent,
    levelup: LevelUpContent,
    skins: SkinsContent,
    audio: AudioContent,
    lore: LoreContent,
} as const;

export function OperatorTabs({ operator, activeTab, onTabChange }: IOperatorTabsProps) {
    const t: TypedT<typeof messages & typeof detailConstantsMessages> = useT("operators");
    const operatorName = useOperatorName();
    const ActiveContent = CONTENT_MAP[activeTab];
    const handleValueChange = (value: unknown) => {
        if (typeof value === "string") onTabChange(value as TabType);
    };

    const filteredTabs = TABS.filter((tab) => !(tab.type === "levelup" && operator.isNotObtainable));

    return (
        <TooltipProvider>
            <div className="flex min-w-0 flex-col lg:flex-row lg:gap-8">
                <nav aria-label={t("tabs.aria")} className="min-w-0 shrink-0 lg:w-56">
                    <Tabs className="page-bleed sticky top-14 z-30 mb-4 border-border/50 border-b bg-background/80 backdrop-blur-xl sm:top-16 lg:hidden" onValueChange={handleValueChange} value={activeTab}>
                        {/* At 375px the six tabs run to 713px and a screenshot showed the strip cut "Level-Up
                            Cost" mid-word with no sign that it scrolls, although the kit fade was on. At 1.5rem
                            it spans about three glyphs of text-sm; 3rem spans about six, so the cut reads as a
                            fade into more rather than as sliced text. */}
                        <ScrollArea className="w-full" scrollFade viewportClassName="[--fade-size:3rem]">
                            <TabsList className={cn("w-max gap-1 rounded-none bg-transparent px-[var(--page-gutter)] py-2.5", "**:data-[slot=tab-indicator]:rounded-full **:data-[slot=tab-indicator]:bg-primary! **:data-[slot=tab-indicator]:shadow-sm")}>
                                {filteredTabs.map((tab) => {
                                    const Icon = tab.icon;
                                    return (
                                        <TabsTab
                                            aria-controls={tab.type === activeTab ? PANEL_ID : undefined}
                                            className="h-auto pointer-coarse:min-h-11 grow-0 rounded-full px-4 py-1.5 font-medium text-muted-foreground text-sm hover:bg-accent hover:text-foreground data-active:text-primary-foreground! data-active:hover:bg-transparent"
                                            id={`${MOBILE_TAB_ID_PREFIX}${tab.type}`}
                                            key={tab.type}
                                            value={tab.type}
                                        >
                                            <Icon className="h-4 w-4" />
                                            <span>{t(tab.labelKey)}</span>
                                        </TabsTab>
                                    );
                                })}
                            </TabsList>
                        </ScrollArea>
                    </Tabs>

                    <Tabs className="sticky top-20 hidden lg:block" onValueChange={handleValueChange} orientation="vertical" value={activeTab}>
                        <TabsList
                            className={cn("w-full gap-0.5 rounded-xl border border-border/60 bg-card p-2", "**:data-[slot=tab-indicator]:rounded-lg **:data-[slot=tab-indicator]:bg-primary/10 **:data-[slot=tab-indicator]:shadow-none **:data-[slot=tab-indicator]:ring-1 **:data-[slot=tab-indicator]:ring-primary/30")}
                        >
                            {filteredTabs.map((tab) => {
                                const Icon = tab.icon;
                                return (
                                    <Tooltip key={tab.type}>
                                        <TooltipTrigger
                                            render={
                                                <TabsTab
                                                    aria-controls={tab.type === activeTab ? PANEL_ID : undefined}
                                                    className="group h-11 justify-start gap-3 rounded-lg px-3 text-left font-medium text-muted-foreground text-sm hover:text-foreground data-active:text-foreground!"
                                                    id={`${TAB_ID_PREFIX}${tab.type}`}
                                                    value={tab.type}
                                                />
                                            }
                                        >
                                            <Icon className="h-4 w-4 shrink-0 text-subtle-foreground transition-colors group-hover:text-foreground group-data-active:text-primary" />
                                            <span className="truncate">{t(tab.labelKey)}</span>
                                        </TooltipTrigger>
                                        <TooltipPopup side="right">{t(tab.labelKey)}</TooltipPopup>
                                    </Tooltip>
                                );
                            })}
                        </TabsList>

                        <div className="mt-3 px-3">
                            <p className="truncate text-muted-foreground text-xs">{operatorName(operator) ?? t("tabs.fallbackName")}</p>
                        </div>
                    </Tabs>
                </nav>

                {/* The panel sits outside both Tabs roots (one strip per breakpoint), so Base UI cannot
                    wire it: it is named by the active desktop tab, whose text is the same as the mobile
                    one and which still names it while hidden below lg. */}
                <Card aria-labelledby={`${TAB_ID_PREFIX}${activeTab}`} className="min-w-0 flex-1 overflow-hidden border-border/60 bg-card" id={PANEL_ID} render={<section />} role="tabpanel">
                    <CardPanel className="fade-in-50 animate-in p-0 duration-300" key={activeTab}>
                        <ActiveContent key={operator.id ?? operator.name} operator={operator} />
                    </CardPanel>
                </Card>
            </div>
        </TooltipProvider>
    );
}
